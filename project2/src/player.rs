use bevy::app::{Plugin, PostUpdate};
use bevy::prelude::*;

use crate::{
    common::LoadSet,
    log_error,
    player::{
        resources::PlayerData,
        systems::{load_player_system, save_player_system},
    },
    GameState,
};

pub mod resources;
pub mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(PlayerData::new(0))
            .add_systems(
                PostUpdate,
                save_player_system
                    .pipe(log_error)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnEnter(GameState::Loading),
                load_player_system.pipe(log_error).in_set(LoadSet::Load),
            );
    }
}
