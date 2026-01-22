use anyhow::{Result, anyhow};
use bevy::{
    app::AppExit,
    color::Color,
    ecs::{
        change_detection::DetectChanges,
        entity::Entity,
        hierarchy::Children,
        lifecycle::RemovedComponents,
        message::{MessageReader, MessageWriter},
        system::*,
    },
    prelude::*,
    sprite::Text2d,
    state::state::{NextState, State},
    tasks::IoTaskPool,
    transform::components::Transform,
    ui::widget::Text,
    utils::default,
};
use bevy_egui::{
    EguiContexts,
    egui::{self, DragValue, Window},
};
use std::fs;

use crate::common::messages::SaveGameMessage;
use crate::common::systems::{SAVE_PATH, get_save_path};
use crate::country::messages::{
    AcceptPeaceMessage, ChangeRelationMessage, ProposePeaceMessage, RejectPeaceMessage,
};
use crate::map::messages::{ArmyBattleMessage, SaveMapMessage};
use crate::ui::messages::UiClickMessage;
use crate::ui::resources::UiSounds;
use crate::{
    GameState, InGameStates,
    common::messages::NextTurnMessage,
    country::{components::OwnershipTile, resources::*},
    map::{
        components::*,
        messages::{BuildBuildingMessage, MoveArmyMessage, SpawnArmyMessage},
        resources::{ArmyMovements, MapSettings, SelectionState},
    },
    player::resources::PlayerData,
    ui::{
        components::{ArmySizeLabel, CountryLabel},
        messages::UiGameMessages,
        resources::{GameLoadState, MenuIcons, TurnCounter, UiModel},
    },
};
use crate::{common::components::GridPosition, log_error};

pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let click_handle = asset_server.load("ui-click.mp3");
    let war_handle = asset_server.load("war_short.mp3");
    let peace_handle = asset_server.load("peace_short.mp3");
    let battle_handle = asset_server.load("battle.mp3");

    commands.insert_resource(UiSounds {
        click_sound: click_handle,
        war_sound: war_handle,
        peace_sound: peace_handle,
        battle_sound: battle_handle,
    });
}

pub fn handle_audio(
    mut commands: Commands,
    mut ui_click_message_reader: MessageReader<UiClickMessage>,
    ui_sounds: Res<UiSounds>,
) {
    for _ in ui_click_message_reader.read() {
        commands.spawn((
            AudioPlayer(ui_sounds.click_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

pub fn handle_change_relation_audio(
    mut commands: Commands,
    mut relation_change_message_reader: MessageReader<ChangeRelationMessage>,
    ui_sounds: Res<UiSounds>,
    player_data: Res<PlayerData>,
) {
    for msg in relation_change_message_reader.read() {
        if msg.country_a_idx != player_data.country_idx
            && msg.country_b_idx != player_data.country_idx
        {
            continue;
        }

        match msg.relation {
            RelationStatus::Neutral => {
                commands.spawn((
                    AudioPlayer(ui_sounds.peace_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
            RelationStatus::AtWar => {
                commands.spawn((
                    AudioPlayer(ui_sounds.war_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}

pub fn handle_battle_audio(
    mut commands: Commands,
    mut army_battle_message_reader: MessageReader<ArmyBattleMessage>,
    ui_sounds: Res<UiSounds>,
) {
    for _ in army_battle_message_reader.read() {
        commands.spawn((
            AudioPlayer(ui_sounds.battle_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

pub fn remove_army_label_system(
    mut commands: Commands,
    mut removed_army: RemovedComponents<Army>,
    children_query: Query<&Children>,
    label_query: Query<Entity, With<ArmySizeLabel>>,
    army_query: Query<&Army>,
) {
    for entity in removed_army.read() {
        if army_query.contains(entity) {
            continue;
        }

        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if label_query.contains(child) {
                    commands.entity(child).despawn();
                }
            }
        }
    }
}

pub fn display_unit_count(
    mut commands: Commands,
    army_query: Query<(Entity, &Army, Option<&Children>), Changed<Army>>,
    mut text_query: Query<&mut Text2d, With<ArmySizeLabel>>,
) {
    for (entity, army, children_opt) in army_query.iter() {
        let mut found_existing = false;

        if let Some(children) = children_opt {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = army.number_of_units.to_string();
                    found_existing = true;
                    break;
                }
            }
        }

        if !found_existing {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text2d::new(army.number_of_units.to_string()),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 30.0, 10.0),
                    ArmySizeLabel {},
                ));
            });
        }
    }
}

pub fn setup_ui_label(mut commands: Commands) {
    commands.spawn((
        Text::new("No country selected"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            ..default()
        },
        CountryLabel {},
    ));
}

#[derive(SystemParam)]
pub struct ControlsUiResources<'w> {
    selection_state: Res<'w, SelectionState>,
    countries: Res<'w, Countries>,
    turn_counter: Res<'w, TurnCounter>,
    player_data: ResMut<'w, PlayerData>,
    map_settings: Res<'w, MapSettings>,
    current_state: Res<'w, State<InGameStates>>,
    diplomacy: Res<'w, Diplomacy>,
    next_state: ResMut<'w, NextState<InGameStates>>,
    ui_model: ResMut<'w, UiModel>,
}

fn save_popup_ui(ctx: &egui::Context, ui_model: &mut UiModel, msgs: &mut UiGameMessages) {
    let mut is_open = ui_model.save_popup_open;
    let mut saved = false;

    if ui_model.save_popup_open {
        egui::Window::new("Zapisz grę")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label("Save name:");
                ui.text_edit_singleline(&mut ui_model.save_file_name);
                if ui.button("Save").clicked() {
                    msgs.ui_click_message.write(UiClickMessage {});
                    msgs.save_game.write(SaveGameMessage {
                        save_name: ui_model.save_file_name.clone(),
                    });
                    saved = true;
                    ui.close();
                }
            });
    }

    if saved {
        is_open = false;
    }

    ui_model.save_popup_open = is_open;
}

fn save_map_popup_ui(ctx: &egui::Context, ui_model: &mut UiModel, msgs: &mut UiGameMessages) {
    let mut is_open = ui_model.save_map_popup_open;
    let mut saved = false;

    if ui_model.save_map_popup_open {
        egui::Window::new("Zapisz mapę")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label("Nazwa mapy:");
                ui.text_edit_singleline(&mut ui_model.map_file_name);
                if ui.button("Zapisz").clicked() {
                    msgs.ui_click_message.write(UiClickMessage {});
                    msgs.save_map.write(SaveMapMessage {
                        map_name: ui_model.map_file_name.clone(),
                    });
                    saved = true;
                    ui.close();
                }
            });
    }

    if saved {
        is_open = false;
    }

    ui_model.save_map_popup_open = is_open;
}

pub fn setup_ui(
    mut contexts: EguiContexts,
    mut msgs: UiGameMessages,
    mut resources: ControlsUiResources,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    map_tiles: Query<(&MapTile, Has<Building>)>,
    army_query: Query<(Entity, &Army, &GridPosition)>,
) -> Result<()> {
    let ctx = contexts.ctx_mut()?;
    Window::new("Managing Centre").show(ctx, |ui| -> Result<()> {
        selection_ui_components(
            &mut msgs,
            &mut resources,
            ownership_tiles,
            map_tiles,
            army_query,
            ui,
        )?;
        turn_ui(&mut msgs, &mut resources, ui);
        save_ui(&mut msgs, &mut resources.ui_model, ui);
        save_popup_ui(ctx, &mut resources.ui_model, &mut msgs);
        save_map_popup_ui(ctx, &mut resources.ui_model, &mut msgs);
        Ok(())
    });
    Ok(())
}

fn selection_ui_components(
    msgs: &mut UiGameMessages<'_>,
    resources: &mut ControlsUiResources<'_>,
    ownership_tiles: Query<'_, '_, (&OwnershipTile, &GridPosition)>,
    map_tiles: Query<'_, '_, (&MapTile, Has<Building>)>,
    army_query: Query<'_, '_, (Entity, &Army, &GridPosition)>,
    ui: &mut egui::Ui,
) -> Result<(), anyhow::Error> {
    let _: () = if let Some((country, idx)) = get_country_from_selection_state(
        &resources.selection_state,
        &ownership_tiles,
        &resources.countries,
    ) && let Some((selected_tile_pos, selected_tile_entity)) =
        get_selected_tile_from_selection_state(&resources.selection_state)
    {
        country_ui(ui, country);
        let (_, has_building) = map_tiles.get(selected_tile_entity)?;
        let army_at_pos = army_query
            .iter()
            .find(|(_, _, pos)| pos.x == selected_tile_pos.0 && pos.y == selected_tile_pos.1);
        if resources.player_data.country_idx == idx {
            player_own_country_management_ui(
                msgs,
                resources,
                ui,
                has_building,
                army_at_pos,
                idx,
                selected_tile_entity,
            );
        } else {
            diplomacy_ui(msgs, &*resources, ui, idx);
        }
    };
    Ok(())
}

fn player_own_country_management_ui(
    msgs: &mut UiGameMessages<'_>,
    resources: &mut ControlsUiResources<'_>,
    ui: &mut egui::Ui,
    has_building: bool,
    army_at_pos: Option<(Entity, &Army, &GridPosition)>,
    idx: usize,
    selected_tile_entity: Entity,
) {
    if !has_building {
        building_ui(
            msgs,
            resources.map_settings.building_cost,
            ui,
            idx,
            selected_tile_entity,
        );
    }
    army_ui(
        resources,
        msgs,
        army_at_pos.map(|(e, a, _)| (e, a)),
        ui,
        idx,
        selected_tile_entity,
    );
}

fn save_ui(msgs: &mut UiGameMessages<'_>, ui_model: &mut ResMut<'_, UiModel>, ui: &mut egui::Ui) {
    if ui.button("Save").clicked() {
        msgs.ui_click_message.write(UiClickMessage {});
        ui_model.save_popup_open = true;
    }
    if ui.button("Save Map").clicked() {
        msgs.ui_click_message.write(UiClickMessage {});
        ui_model.save_map_popup_open = true;
    }
}

const SAVE_FILE_NAME: &str = "save_turn.json";

pub fn save_turn_counter_system(
    mut save_game_message_reader: MessageReader<SaveGameMessage>,
    turn_counter: Res<TurnCounter>,
) -> anyhow::Result<()> {
    for save_game_message in save_game_message_reader.read() {
        let save_name = save_game_message.save_name.clone();
        let pool = IoTaskPool::get();
        let turn_counter_cloned = (*turn_counter).clone();
        pool.spawn(async move {
            let save_json = match serde_json::to_string_pretty(&turn_counter_cloned) {
                Ok(save_json) => save_json,
                Err(e) => {
                    log_error(In(anyhow::Result::Err(e.into())));
                    return;
                }
            };
            if let Err(e) = std::fs::create_dir_all(format!("{}/{}", SAVE_PATH, save_name)) {
                log_error(In(anyhow::Result::Err(e.into())));
                return;
            };
            if let Err(e) = std::fs::write(
                format!("{}/{}/{}", SAVE_PATH, save_name, SAVE_FILE_NAME),
                save_json,
            ) {
                log_error(In(anyhow::Result::Err(e.into())));
                return;
            };
        })
        .detach();
    }
    Ok(())
}

pub fn load_turn_counter_system(
    mut turn_counter: ResMut<TurnCounter>,
    load_state: Res<GameLoadState>,
) -> anyhow::Result<()> {
    if let Some(save_name) = &load_state.save_name {
        let path = format!("{}/{}", get_save_path(save_name), SAVE_FILE_NAME);
        let data = std::fs::read_to_string(path)?;
        let turn_counter_saved: TurnCounter = serde_json::from_str(&data)?;

        *turn_counter = turn_counter_saved;
    }

    Ok(())
}

fn diplomacy_ui(
    msgs: &mut UiGameMessages<'_>,
    resources: &ControlsUiResources<'_>,
    ui: &mut egui::Ui,
    idx: usize,
) {
    ui.heading("Diplomacy");
    let relation = resources
        .diplomacy
        .get_relation(resources.player_data.country_idx, idx);
    ui.label(format!("Relation: {}", relation));
    if let RelationStatus::AtWar = relation
        && ui.button("Peace").clicked()
    {
        msgs.ui_click_message.write(UiClickMessage {});
        msgs.propose_peace_message.write(ProposePeaceMessage {
            from: resources.player_data.country_idx,
            to: idx,
        });
    }
    if let RelationStatus::Neutral = relation
        && ui.button("Declare war").clicked()
    {
        msgs.ui_click_message.write(UiClickMessage {});
        msgs.change_relation.write(ChangeRelationMessage {
            country_a_idx: resources.player_data.country_idx,
            country_b_idx: idx,
            relation: RelationStatus::AtWar,
        });
    }
    ui.separator();
}

pub fn display_country_name(
    _: Query<&SelectionCursor, Changed<SelectionCursor>>,
    mut label: Single<&mut Text, With<CountryLabel>>,
    select_state: Res<SelectionState>,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    countries: Res<Countries>,
    player_data: Res<PlayerData>,
) {
    if let Some((country, selected_country_idx)) =
        get_country_from_selection_state(&select_state, &ownership_tiles, &countries)
    {
        label.as_mut().0 = country.name.clone();

        if player_data.country_idx == selected_country_idx {
            label.as_mut().0.push_str(" (Player)");
        }

        return;
    }

    label.as_mut().0 = "No country selected ".into();
}

pub fn update_turn_counter(
    mut turn_counter: ResMut<TurnCounter>,
    mut msgr: MessageReader<NextTurnMessage>,
) {
    for _ in msgr.read() {
        turn_counter.as_mut().count += 1;
    }
}

fn main_menu_buttons(
    ui: &mut egui::Ui,
    next_state: &mut ResMut<NextState<GameState>>,
    exit: &mut MessageWriter<AppExit>,
    sound: &mut MessageWriter<UiClickMessage>,
    ui_model: &mut UiModel,
) {
    ui.heading("Project 2");
    ui.add_space(10.0);
    if ui
        .add(egui::Button::new("Start Game").min_size([100.0, 30.0].into()))
        .clicked()
    {
        sound.write(UiClickMessage {});
        next_state.set(GameState::CountrySelection);
    }
    loading_menu_items(ui, next_state, sound);
    ui.add_space(5.0);
    ui.checkbox(&mut ui_model.ai_on, "AI on?");
    ui.add_space(5.0);
    if ui.button("Quit").clicked() {
        sound.write(UiClickMessage {});
        exit.write(AppExit::Success);
    }
}

fn loading_menu_items(
    ui: &mut egui::Ui,
    next_state: &mut ResMut<'_, NextState<GameState>>,
    sound: &mut MessageWriter<'_, UiClickMessage>,
) {
    ui.add_space(5.0);
    if ui
        .add(egui::Button::new("Load Game").min_size([100.0, 30.0].into()))
        .clicked()
    {
        sound.write(UiClickMessage {});
        next_state.set(GameState::LoadGame);
    }
    ui.add_space(5.0);
    if ui
        .add(egui::Button::new("Load Map").min_size([100.0, 30.0].into()))
        .clicked()
    {
        sound.write(UiClickMessage {});
        next_state.set(GameState::LoadMap);
    }
}

pub fn main_menu_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut sound: MessageWriter<UiClickMessage>,
    mut ui_model: ResMut<UiModel>,
) -> anyhow::Result<()> {
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Main Menu")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            main_menu_buttons(ui, &mut next_state, &mut exit, &mut sound, &mut ui_model);
        });

    Ok(())
}

pub fn load_map_menu_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut load_state: ResMut<GameLoadState>,
    mut ui_click_message_writer: MessageWriter<UiClickMessage>,
) -> anyhow::Result<()> {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Load Map")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Select a map");
            ui.add_space(10.0);
            if let Ok(paths) = fs::read_dir("./maps") {
                for path in paths.flatten() {
                    let path = path.path();
                    if path.is_file()
                        && let Some(map_name) = path.file_stem()
                        && let Some(map_name_str) = map_name.to_str()
                        && ui.button(map_name_str).clicked()
                    {
                        ui_click_message_writer.write(UiClickMessage {});
                        load_state.map_name = Some(map_name_str.to_owned());
                        next_state.set(GameState::CountrySelection);
                    }
                }
            }
            ui.add_space(10.0);
            if ui.button("Back").clicked() {
                ui_click_message_writer.write(UiClickMessage {});
                next_state.set(GameState::Menu);
            }
        });
    Ok(())
}

pub fn load_game_menu_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut load_state: ResMut<GameLoadState>,
    mut ui_click_message_writer: MessageWriter<UiClickMessage>,
) -> anyhow::Result<()> {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Load Game")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Select a save");
            ui.add_space(10.0);
            if let Ok(paths) = fs::read_dir("./saves") {
                for path in paths.flatten() {
                    let path = path.path();
                    if path.is_dir()
                        && let Some(save_name) = path.file_name()
                        && let Some(save_name_str) = save_name.to_str()
                        && ui.button(save_name_str).clicked()
                    {
                        ui_click_message_writer.write(UiClickMessage {});
                        load_state.save_name = Some(save_name_str.to_owned());
                        next_state.set(GameState::Loading);
                    }
                }
            }
            ui.add_space(10.0);
            if ui.button("Back").clicked() {
                ui_click_message_writer.write(UiClickMessage {});
                next_state.set(GameState::Menu);
            }
        });
    Ok(())
}

pub fn handle_selection_change_when_moving_army(
    mut army_movements: ResMut<ArmyMovements>,
    mut next_state: ResMut<NextState<InGameStates>>,
    highlight_overlay: Query<&GridPosition, With<HighlightOverlay>>,
    selection: Res<SelectionState>,
    ui_model: Res<UiModel>,
    button_input: Res<ButtonInput<KeyCode>>,
) -> anyhow::Result<()> {
    if button_input.just_pressed(KeyCode::Escape) {
        next_state.set(InGameStates::Idle);
        return Ok(());
    }

    if selection.is_changed() {
        let Some(army_entity_being_moved) = ui_model.army_entity_being_moved else {
            return Err(anyhow!("No moving army entity in the ui model"));
        };

        if let Some((selected_pos_x, selected_pos_y)) = selection.selected_tile {
            let Some(move_position) = highlight_overlay
                .iter()
                .find(|pos| pos.x == selected_pos_x && pos.y == selected_pos_y)
            else {
                return Ok(());
            };

            army_movements.add_movement(MoveArmyMessage {
                moved_army_entity: army_entity_being_moved,
                target_position: GridPosition::new(move_position.x, move_position.y),
                number_of_units_to_move: ui_model.selected_number_of_units,
            });
            next_state.set(InGameStates::Idle);
        }
    }

    Ok(())
}

// helpers

fn country_ui(ui: &mut egui::Ui, country: &Country) {
    ui.heading("Country");
    ui.label(format!("Name: {}", country.name));
    ui.label(format!("Money: {}", country.money));
    ui.separator();
}

fn turn_ui(
    msgs: &mut UiGameMessages<'_>,
    resources: &mut ControlsUiResources<'_>,
    ui: &mut egui::Ui,
) {
    ui.heading("Turn information");

    let turn_number = resources.turn_counter.count;

    ui.label(format!("Turn number: {turn_number}"));

    if ui.button("End Turn").clicked() {
        println!("End turn click");
        msgs.ui_click_message.write(UiClickMessage {});
        if !resources.ui_model.ai_on {
            if resources.player_data.country_idx == resources.countries.countries.len() - 1 {
                msgs.next_turn_message.write(NextTurnMessage {});
                resources.player_data.country_idx = 0;
            } else {
                resources.player_data.country_idx += 1;
            }
        } else {
            resources.next_state.set(InGameStates::AiTurn);
        }
    }

    ui.separator();
}

fn army_ui(
    resources: &mut ControlsUiResources<'_>,
    ui_game_messages: &mut UiGameMessages,
    army_at_pos: Option<(Entity, &Army)>,
    ui: &mut egui::Ui,
    idx: usize,
    selected_tile_entity: Entity,
) {
    ui.heading("Army");
    ui.add(DragValue::new(
        &mut resources.ui_model.selected_number_of_units,
    ));
    ui.label(format!("Unit cost: {}", resources.map_settings.unit_cost));

    if ui.button("Recruit").clicked() {
        ui_game_messages.ui_click_message.write(UiClickMessage {});
        ui_game_messages.spawn_army.write(SpawnArmyMessage {
            tile_entity: selected_tile_entity,
            country_idx: idx,
            amount: resources.ui_model.selected_number_of_units,
        });
    }

    if *resources.current_state != InGameStates::MovingArmy
        && army_at_pos.is_some()
        && let Some((entity, _)) = army_at_pos
        && ui.button("Move").clicked()
    {
        ui_game_messages.ui_click_message.write(UiClickMessage {});
        resources.ui_model.army_entity_being_moved = Some(entity);
        resources.next_state.set(InGameStates::MovingArmy);
    }

    ui.separator();
}

fn building_ui(
    msgs: &mut UiGameMessages<'_>,
    building_cost: i32,
    ui: &mut egui::Ui,
    idx: usize,
    selected_tile_entity: Entity,
) {
    ui.heading("Building");
    ui.label(format!("Building cost: {building_cost}"));

    if ui.button("Build").clicked() {
        msgs.ui_click_message.write(UiClickMessage {});
        msgs.build_building.write(BuildBuildingMessage {
            tile_entity: selected_tile_entity,
            country_idx: idx,
        });
    }

    ui.separator();
}

fn get_selected_tile_from_selection_state(
    selection_state: &Res<SelectionState>,
) -> std::option::Option<((i32, i32), Entity)> {
    if let Some(selected_tile_pos) = selection_state.selected_tile
        && let Some(selected_tile_entity) = selection_state.selected_entity
    {
        return Some((selected_tile_pos, selected_tile_entity));
    }

    None
}

pub fn load_menu_icons(mut menu_icons: ResMut<MenuIcons>, asset_server: Res<AssetServer>) {
    for i in 0..5 {
        let path = format!("countries/{}.png", i);
        let handle = asset_server.load(path);
        menu_icons.country_flags.push(handle);
    }
}

pub fn country_selection_system(
    mut contexts: EguiContexts,
    next_state: ResMut<NextState<GameState>>,
    player_data: ResMut<PlayerData>,
    ui_click_message_writer: MessageWriter<UiClickMessage>,
    menu_icons: Res<MenuIcons>,
) -> anyhow::Result<()> {
    let texture_ids: Vec<_> = menu_icons
        .country_flags
        .iter()
        .map(|icon| contexts.add_image(bevy_egui::EguiTextureHandle::Strong(icon.clone())))
        .collect();
    let ctx = contexts.ctx_mut()?;
    select_country_window(
        next_state,
        player_data,
        ui_click_message_writer,
        texture_ids,
        ctx,
    );
    Ok(())
}

fn select_country_window(
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut player_data: ResMut<'_, PlayerData>,
    mut ui_click_message_writer: MessageWriter<'_, UiClickMessage>,
    texture_ids: Vec<egui::TextureId>,
    ctx: &mut egui::Context,
) {
    egui::Window::new("Select Country")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Choose your country");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                for (i, texture_id) in texture_ids.into_iter().enumerate() {
                    if ui
                        .add(egui::Button::image_and_text(
                            egui::load::SizedTexture::new(texture_id, [50.0, 50.0]),
                            "",
                        ))
                        .clicked()
                    {
                        ui_click_message_writer.write(UiClickMessage {});
                        player_data.country_idx = i;
                        next_state.set(GameState::Generating);
                    }
                }
            });
        });
}

fn get_country_from_selection_state<'a>(
    select_state: &Res<SelectionState>,
    ownership_tiles: &Query<(&OwnershipTile, &GridPosition)>,
    countries: &'a Res<Countries>,
) -> Option<(&'a Country, usize)> {
    if let Some(selected_tile) = select_state.selected_tile {
        let ownership_tile: Vec<_> = ownership_tiles
            .iter()
            .filter(|(_, p)| (p.x, p.y) == selected_tile)
            .take(1)
            .collect();

        if let Some(country_id) = ownership_tile[0].0.country_id {
            return Some((&countries.countries[country_id], country_id));
        }
    }

    None
}

fn peace_offer_window_ui(
    ui: &mut egui::Ui,
    from_country_name: &str,
    offer: &PeaceOffer,
    accept_peace_msg: &mut MessageWriter<AcceptPeaceMessage>,
    reject_peace_msg: &mut MessageWriter<RejectPeaceMessage>,
    ui_click_message_writer: &mut MessageWriter<UiClickMessage>,
) {
    ui.label(format!("Country {} offers peace.", from_country_name));

    ui.horizontal(|ui| {
        if ui.button("Accept").clicked() {
            ui_click_message_writer.write(UiClickMessage {});
            accept_peace_msg.write(AcceptPeaceMessage {
                from: offer.from,
                to: offer.to,
            });
        }
        if ui.button("Reject").clicked() {
            ui_click_message_writer.write(UiClickMessage {});
            reject_peace_msg.write(RejectPeaceMessage {
                from: offer.from,
                to: offer.to,
            });
        }
    });
}

pub fn display_peace_offers_system(
    mut contexts: EguiContexts,
    player_data: Res<PlayerData>,
    peace_offers: Res<PeaceOffers>,
    countries: Res<Countries>,
    mut accept_peace_msg: MessageWriter<AcceptPeaceMessage>,
    mut reject_peace_msg: MessageWriter<RejectPeaceMessage>,
    mut ui_click_message_writer: MessageWriter<UiClickMessage>,
) -> anyhow::Result<()> {
    let ctx = contexts.ctx_mut()?;

    for offer in peace_offers.offers.iter() {
        if offer.to == player_data.country_idx {
            let from_country_name = &countries.countries[offer.from].name;

            Window::new(format!("Peace Offer from {}", from_country_name))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    peace_offer_window_ui(
                        ui,
                        from_country_name,
                        offer,
                        &mut accept_peace_msg,
                        &mut reject_peace_msg,
                        &mut ui_click_message_writer,
                    );
                });
        }
    }
    Ok(())
}
