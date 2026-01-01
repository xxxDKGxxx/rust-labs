use anyhow::Result;
use bevy::{
    color::Color,
    ecs::{
        message::{MessageReader, MessageWriter},
        query::*,
        system::*,
    },
    text::*,
    ui::{widget::Text, *},
    utils::default,
};
use bevy_egui::{EguiContexts, egui::Window};

use crate::{
    country::{components::OwnershipTile, resources::*},
    map::{components::*, resources::SelectionState},
    player::resources::PlayerData,
    ui::{components::CountryLabel, messages::NextTurnMessage, resources::TurnCounter},
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
    mut msgs: MessageWriter<NextTurnMessage>,
    select_state: Res<SelectionState>,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
    countries: Res<Countries>,
    turn_counter_resource: Res<TurnCounter>,
) -> Result<()> {
    let ctx = contexts.ctx_mut()?;

    Window::new("Managing Centre").show(ctx, |ui| {
        ui.heading("Game Options");

        if let Some((country, _)) =
            get_country_from_selection_state(&select_state, &ownership_tiles, &countries)
        {
            let money = country.money;

            ui.label(format!("Money: {money}"));
        }

        let turn_number = turn_counter_resource.count;

        ui.label(format!("Turn number: {turn_number}"));

        if ui.button("Next Turn").clicked() {
            msgs.write(NextTurnMessage {});
        }
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

// helpers

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
