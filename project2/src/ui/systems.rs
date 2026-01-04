use anyhow::Result;
use bevy::{
    app::AppExit,
    color::Color,
    ecs::{
        change_detection::DetectChanges,
        entity::Entity,
        hierarchy::Children,
        message::{MessageReader, MessageWriter},
        query::*,
        system::*,
    },
    sprite::Text2d,
    state::state::{NextState, State},
    text::*,
    transform::components::Transform,
    ui::{widget::Text, *},
    utils::default,
};
use bevy_egui::{
    EguiContexts,
    egui::{self, DragValue, Window},
};

use crate::{
    GameState, InGameStates,
    country::{components::OwnershipTile, resources::*},
    map::{
        components::*,
        messages::{BuildBuildingMessage, SpawnArmyMessage},
        resources::{MapSettings, SelectionState},
    },
    player::resources::PlayerData,
    ui::{
        components::{ArmySizeLabel, CountryLabel},
        messages::{NextTurnMessage, UiGameMessages},
        resources::{TurnCounter, UiModel},
    },
};

pub fn display_unit_count(
    mut commands: Commands,
    army_query: Query<(Entity, &Army, Option<&Children>), Changed<Army>>,
    mut text_query: Query<&mut Text2d, With<ArmySizeLabel>>,
) {
    for (entity, army, children_opt) in army_query.iter() {
        let mut found_existing = false;

        if let Some(children) = children_opt {
            for &child in children.iter() {
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
    player_data: Res<'w, PlayerData>,
    map_settings: Res<'w, MapSettings>,
}

pub fn setup_controls_ui(
    mut contexts: EguiContexts,
    mut msgs: UiGameMessages,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    map_tiles: Query<(&MapTile, Has<Building>)>,
    resources: ControlsUiResources,
) -> Result<()> {
    let ctx = contexts.ctx_mut()?;
    let building_cost = resources.map_settings.building_cost;

    Window::new("Managing Centre").show(ctx, |ui| -> Result<()> {
        ui.heading("Game Options");

        if let Some((country, idx)) = get_country_from_selection_state(
            &resources.selection_state,
            &ownership_tiles,
            &resources.countries,
        ) && let Some((_, selected_tile_entity)) =
            get_selected_tile_from_selection_state(&resources.selection_state)
        {
            let money = country.money;

            ui.label(format!("Money: {money}"));

            let (_, has_building) = map_tiles.get(selected_tile_entity)?;
            if resources.player_data.country_idx == idx && !has_building {
                ui.label(format!("Building cost: {building_cost}"));
                if ui.button("Build").clicked() {
                    msgs.build_building.write(BuildBuildingMessage {
                        tile_entity: selected_tile_entity,
                        country_idx: idx,
                    });
                }
            }
        }

        let turn_number = resources.turn_counter.count;

        ui.label(format!("Turn number: {turn_number}"));

        if ui.button("Next Turn").clicked() {
            msgs.next_turn.write(NextTurnMessage {});
        }

        Ok(())
    });

    Ok(())
}

#[derive(SystemParam)]
pub struct ArmyControlsUiResources<'w> {
    selection_state: Res<'w, SelectionState>,
    countries: Res<'w, Countries>,
    player_data: Res<'w, PlayerData>,
    map_settings: Res<'w, MapSettings>,
    current_state: Res<'w, State<InGameStates>>,
}

pub fn setup_army_controls_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<InGameStates>>,
    msgw: MessageWriter<SpawnArmyMessage>,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    army_movement: Query<Option<&ArmyMovement>, With<Army>>,
    resources: ArmyControlsUiResources,
    ui_model: ResMut<UiModel>,
) -> anyhow::Result<()> {
    let ctx = contexts.ctx_mut()?;

    if let Some((contry, idx)) = get_country_from_selection_state(
        &resources.selection_state,
        &ownership_tiles,
        &resources.countries,
    ) && let Some((_, selected_tile_entity)) =
        get_selected_tile_from_selection_state(&resources.selection_state)
    {
        if resources.player_data.country_idx == idx {
            display_army_controls_window(
                msgw,
                &resources,
                ui_model,
                ctx,
                contry,
                idx,
                selected_tile_entity,
                &mut next_state,
                &army_movement,
            );
        }
    }

    Ok(())
}

fn display_army_controls_window(
    mut msgw: MessageWriter<'_, SpawnArmyMessage>,
    resources: &ArmyControlsUiResources<'_>,
    mut ui_model: ResMut<'_, UiModel>,
    ctx: &mut egui::Context,
    contry: &Country,
    idx: usize,
    selected_tile_entity: Entity,
    next_state: &mut ResMut<NextState<InGameStates>>,
    army_movement: &Query<Option<&ArmyMovement>, With<Army>>,
) {
    let max_no_of_units_allowed_to_recruit = contry.money / resources.map_settings.unit_cost;

    Window::new("Army Controls").show(ctx, |ui| {
        ui.add(
            DragValue::new(&mut ui_model.selected_number_of_units)
                .range(1..=max_no_of_units_allowed_to_recruit),
        );

        if ui.button("Recruit").clicked() {
            msgw.write(SpawnArmyMessage {
                tile_entity: selected_tile_entity,
                country_idx: idx,
                amount: ui_model.selected_number_of_units,
            });
        }

        if ui.button("move").clicked() && *resources.current_state != InGameStates::MovingArmy {
            next_state.set(InGameStates::MovingArmy);
        }

        if let Some(selected_entity) = resources.selection_state.selected_entity
            && let Ok(army_movement_option) = army_movement.get(selected_entity)
            && let Some(army_movement) = army_movement_option
        {
            ui.label(format!(
                "Army moves {} units to: {} {}",
                army_movement.number_of_units_to_move,
                army_movement.target_position.x,
                army_movement.target_position.y
            ));
        }
    });
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

    egui::Window::new("Menu Główne")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Project 2");
            ui.add_space(10.0);

            if ui
                .add(egui::Button::new("Rozpocznij Grę").min_size([100.0, 30.0].into()))
                .clicked()
            {
                next_state.set(GameState::InGame);
            }

            ui.add_space(5.0);

            if ui.button("Opcje").clicked() {
                println!("Tu byłyby opcje...");
            }

            ui.add_space(5.0);

            if ui.button("Wyjdź").clicked() {
                exit.write(AppExit::Success);
            }
        });

    Ok(())
}

pub fn handle_selection_change_when_moving_army(
    mut commands: Commands,
    highlight_overlay: Query<&GridPosition, With<HighlightOverlay>>,
    mut next_state: ResMut<NextState<InGameStates>>,
    selection: Res<SelectionState>,
    mut army_movement_query: Query<Option<&mut ArmyMovement>, With<Army>>,
    ui_model: Res<UiModel>,
) {
    if selection.is_changed() {
        if let Some(selected_entity) = selection.selected_entity
            && let Some((selected_pos_x, selected_pos_y)) = selection.selected_tile
        {
            let Some(move_position) = highlight_overlay
                .iter()
                .find(|pos| pos.x == selected_pos_x && pos.y == selected_pos_y)
            else {
                next_state.set(InGameStates::Idle);
                return;
            };

            if let Ok(army_movement_option) = army_movement_query.get_mut(selected_entity)
                && let Some(mut army_movement) = army_movement_option
            {
                army_movement.number_of_units_to_move = ui_model.selected_number_of_units;
                army_movement.target_position.x = move_position.x;
                army_movement.target_position.y = move_position.y;
            } else {
                commands.entity(selected_entity).insert(ArmyMovement {
                    target_position: GridPosition::new(move_position.x, move_position.y),
                    number_of_units_to_move: ui_model.selected_number_of_units,
                });
            }

            next_state.set(InGameStates::Idle);
        }
    }
}

// helpers

fn get_selected_tile_from_selection_state(
    selection_state: &Res<SelectionState>,
) -> std::option::Option<((u64, u64), Entity)> {
    if let Some(selected_tile_pos) = selection_state.selected_tile
        && let Some(selected_tile_entity) = selection_state.selected_entity
    {
        return Some((selected_tile_pos, selected_tile_entity));
    }

    None
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
