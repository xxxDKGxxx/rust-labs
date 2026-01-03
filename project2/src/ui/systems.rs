use anyhow::Result;
use bevy::{
    app::AppExit,
    color::Color,
    ecs::{
        entity::Entity,
        message::{MessageReader, MessageWriter},
        query::*,
        system::*,
    },
    state::state::NextState,
    text::*,
    ui::{widget::Text, *},
    utils::default,
};
use bevy_egui::{
    EguiContexts,
    egui::{self, Window},
};

use crate::{
    GameState,
    country::{components::OwnershipTile, resources::*},
    map::{
        components::*,
        messages::BuildBuildingMessage,
        resources::{MapSettings, SelectionState},
    },
    player::resources::PlayerData,
    ui::{
        components::CountryLabel,
        messages::{NextTurnMessage, UiGameMessages},
        resources::TurnCounter,
    },
};

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

pub fn setup_controls_ui(
    mut contexts: EguiContexts,
    mut msgs: UiGameMessages,
    select_state: Res<SelectionState>,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    map_tiles: Query<(&MapTile, Has<Building>)>,
    countries: Res<Countries>,
    turn_counter_resource: Res<TurnCounter>,
    player_data: Res<PlayerData>,
    map_settings: Res<MapSettings>,
) -> Result<()> {
    let ctx = contexts.ctx_mut()?;
    let building_cost = map_settings.building_cost;

    Window::new("Managing Centre").show(ctx, |ui| -> Result<()> {
        ui.heading("Game Options");

        if let Some((country, idx)) =
            get_country_from_selection_state(&select_state, &ownership_tiles, &countries)
            && let Some((_, selected_tile_entity)) =
                get_selected_tile_from_selection_state(&select_state)
        {
            let money = country.money;

            ui.label(format!("Money: {money}"));

            let (_, has_building) = map_tiles.get(selected_tile_entity)?;
            if player_data.country_idx == idx && !has_building {
                ui.label(format!("Building cost: {building_cost}"));
                if ui.button("Build").clicked() {
                    msgs.build_building.write(BuildBuildingMessage {
                        tile_entity: selected_tile_entity,
                        country_idx: idx,
                    });
                }
            }
        }

        let turn_number = turn_counter_resource.count;

        ui.label(format!("Turn number: {turn_number}"));

        if ui.button("Next Turn").clicked() {
            msgs.next_turn.write(NextTurnMessage {});
        }

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
