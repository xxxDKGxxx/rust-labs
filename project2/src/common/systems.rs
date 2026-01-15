use bevy::prelude::*;

use crate::{
    GameState,
    country::components::{CountryFlag, OwnershipTile},
    map::components::{Army, Building, MapTile, SelectionCursor},
};

pub const SAVE_PATH: &str = "saves";

pub fn get_save_path(save_name: &str) -> String {
    format!("{SAVE_PATH}/{save_name}")
}

type RelevantEntitiesForDespawnQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<MapTile>,
        With<Army>,
        With<Building>,
        With<OwnershipTile>,
        With<CountryFlag>,
        With<SelectionCursor>,
    )>,
>;

pub fn despawn_everything(mut commands: Commands, entities: RelevantEntitiesForDespawnQuery) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn go_to_in_game_state(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}
