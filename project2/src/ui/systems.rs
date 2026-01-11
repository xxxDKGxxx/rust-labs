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
        resources::{TurnCounter, UiModel},
    },
};

pub fn remove_army_label_system(
    mut commands: Commands,
    mut removed_army: RemovedComponents<Army>,
    children_query: Query<&Children>,
    label_query: Query<Entity, With<ArmySizeLabel>>,
) {
    for entity in removed_army.read() {
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
    ui_model: ResMut<'w, UiModel>,
    next_state: ResMut<'w, NextState<InGameStates>>,
}

pub fn setup_ui(
    mut contexts: EguiContexts,
    mut msgs: UiGameMessages,
    mut resources: ControlsUiResources,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    map_tiles: Query<(&MapTile, Has<Building>)>,
    army: Query<&Army, With<MapTile>>,
) -> Result<()> {
    let ctx = contexts.ctx_mut()?;
    let building_cost = resources.map_settings.building_cost;

    Window::new("Managing Centre").show(ctx, |ui| -> Result<()> {
        if let Some((country, idx)) = get_country_from_selection_state(
            &resources.selection_state,
            &ownership_tiles,
            &resources.countries,
        ) && let Some((_, selected_tile_entity)) =
            get_selected_tile_from_selection_state(&resources.selection_state)
        {
            country_ui(ui, country);

            let (_, has_building) = map_tiles.get(selected_tile_entity)?;

            if resources.player_data.country_idx == idx {
                if !has_building {
                    building_ui(&mut msgs, building_cost, ui, idx, selected_tile_entity);
                }

                army_ui(
                    &mut resources,
                    &mut msgs.spawn_army,
                    army,
                    ui,
                    idx,
                    selected_tile_entity,
                );
            } else {
                ui.heading("Diplomacy");

                let relation = resources
                    .diplomacy
                    .get_relation(resources.player_data.country_idx, idx);

                ui.label(format!("Relation: {}", relation));

                if let RelationStatus::AtWar = relation {
                    if ui.button("Peace").clicked() {
                        msgs.change_relation.write(ChangeRelationMessage {
                            country_a_idx: resources.player_data.country_idx,
                            country_b_idx: idx,
                            relation: RelationStatus::Neutral,
                        });
                    }
                }

                if let RelationStatus::Neutral = relation {
                    if ui.button("Declare war").clicked() {
                        msgs.change_relation.write(ChangeRelationMessage {
                            country_a_idx: resources.player_data.country_idx,
                            country_b_idx: idx,
                            relation: RelationStatus::AtWar,
                        });
                    }
                }

                ui.separator();
            }
        }

        turn_ui(&mut msgs, &mut resources, ui);

        Ok(())
    });

    Ok(())
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

    if ui.button("Next Turn").clicked() {
        if resources.player_data.country_idx == resources.countries.countries.len() - 1 {
            resources.player_data.country_idx = 0;
            msgs.next_turn.write(NextTurnMessage {});
        } else {
            resources.player_data.country_idx += 1;
        }
    }

    ui.separator();
}

fn army_ui(
    resources: &mut ControlsUiResources<'_>,
    msgw: &mut MessageWriter<'_, SpawnArmyMessage>,
    army: Query<'_, '_, &Army, With<MapTile>>,
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
        msgw.write(SpawnArmyMessage {
            tile_entity: selected_tile_entity,
            country_idx: idx,
            amount: resources.ui_model.selected_number_of_units,
        });
    }

    if ui.button("Move").clicked()
        && *resources.current_state != InGameStates::MovingArmy
        && army.get(selected_tile_entity).is_ok()
    {
        resources.ui_model.army_entity_being_moved = Some(selected_tile_entity);
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
