use anyhow::{Result, anyhow};
use bevy::prelude::*;
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
    sprite::Text2d,
    state::state::{NextState, State},
    transform::components::Transform,
    ui::widget::Text,
    utils::default,
};
use bevy_egui::{
    EguiContexts,
    egui::{self, DragValue, Window},
};

use crate::ai::systems::AiTurnMessage;
use crate::common::components::GridPosition;
use crate::common::messages::SaveGameMessage;
use crate::country::messages::ChangeRelationMessage;
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
        resources::{MenuIcons, TurnCounter, UiModel},
    },
};

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
}

pub fn setup_ui(
    mut contexts: EguiContexts,
    mut msgs: UiGameMessages,
    mut resources: ControlsUiResources,
    mut ui_model: ResMut<UiModel>,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    map_tiles: Query<(&MapTile, Has<Building>)>,
    army_query: Query<(Entity, &Army, &GridPosition)>,
) -> Result<()> {
    let ctx = contexts.ctx_mut()?;
    let building_cost = resources.map_settings.building_cost;

    Window::new("Managing Centre").show(ctx, |ui| -> Result<()> {
        if let Some((country, idx)) = get_country_from_selection_state(
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
                if !has_building {
                    building_ui(&mut msgs, building_cost, ui, idx, selected_tile_entity);
                }

                army_ui(
                    &mut resources,
                    &mut msgs.spawn_army,
                    army_at_pos.map(|(e, a, _)| (e, a)),
                    ui,
                    idx,
                    selected_tile_entity,
                    &mut ui_model,
                );
            } else {
                diplomacy_ui(&mut msgs, &resources, ui, idx);
            }
        }

        turn_ui(&mut msgs, &mut resources, ui);

        if ui.button("Save").clicked() {
            ui_model.save_popup_open = true;
        }

        let mut is_open = ui_model.save_popup_open;

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
                        msgs.save_game.write(SaveGameMessage {
                            save_name: ui_model.save_file_name.clone(),
                        });
                        ui_model.save_popup_open = false;
                        ui.close();
                    }
                });
        }

        ui_model.save_popup_open = is_open;

        Ok(())
    });

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
        msgs.change_relation.write(ChangeRelationMessage {
            country_a_idx: resources.player_data.country_idx,
            country_b_idx: idx,
            relation: RelationStatus::Neutral,
        });
    }

    if let RelationStatus::Neutral = relation
        && ui.button("Declare war").clicked()
    {
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
pub fn main_menu_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) -> anyhow::Result<()> {
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Main Menu")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Project 2");
            ui.add_space(10.0);

            if ui
                .add(egui::Button::new("Start Game").min_size([100.0, 30.0].into()))
                .clicked()
            {
                next_state.set(GameState::CountrySelection);
            }

            ui.add_space(5.0);

            if ui.button("Options").clicked() {
                println!("Tu byłyby opcje...");
            }

            ui.add_space(5.0);

            if ui.button("Quit").clicked() {
                exit.write(AppExit::Success);
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
        // if resources.player_data.country_idx == resources.countries.countries.len() - 1 {
        msgs.ai_turn.write(AiTurnMessage {});
        // resources.player_data.country_idx = 0;
        // } else {
        // resources.player_data.country_idx += 1;
        // }
    }

    ui.separator();
}

fn army_ui(
    resources: &mut ControlsUiResources<'_>,
    msgw: &mut MessageWriter<'_, SpawnArmyMessage>,
    army_at_pos: Option<(Entity, &Army)>,
    ui: &mut egui::Ui,
    idx: usize,
    selected_tile_entity: Entity,
    ui_model: &mut UiModel,
) {
    ui.heading("Army");
    ui.add(DragValue::new(&mut ui_model.selected_number_of_units));
    ui.label(format!("Unit cost: {}", resources.map_settings.unit_cost));

    if ui.button("Recruit").clicked() {
        msgw.write(SpawnArmyMessage {
            tile_entity: selected_tile_entity,
            country_idx: idx,
            amount: ui_model.selected_number_of_units,
        });
    }

    if ui.button("Move").clicked()
        && *resources.current_state != InGameStates::MovingArmy
        && army_at_pos.is_some()
        && let Some((entity, _)) = army_at_pos
    {
        ui_model.army_entity_being_moved = Some(entity);
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
    mut next_state: ResMut<NextState<GameState>>,
    mut player_data: ResMut<PlayerData>,
    menu_icons: Res<MenuIcons>,
) -> anyhow::Result<()> {
    let texture_ids: Vec<_> = menu_icons
        .country_flags
        .iter()
        .map(|icon| contexts.add_image(bevy_egui::EguiTextureHandle::Strong(icon.clone())))
        .collect();

    let ctx = contexts.ctx_mut()?;

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
                        player_data.country_idx = i;
                        next_state.set(GameState::InGame);
                    }
                }
            });
        });

    Ok(())
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
