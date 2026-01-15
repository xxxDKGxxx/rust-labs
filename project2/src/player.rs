use bevy::app::{Plugin, PostUpdate};
use bevy::prelude::*;

use crate::GameState;
use crate::{
    log_error,
    player::{resources::PlayerData, systems::save_player_system},
};

pub mod resources;
pub mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(PlayerData::new(0)).add_systems(
            PostUpdate,
            save_player_system
                .pipe(log_error)
                .run_if(in_state(GameState::InGame)),
        );
    }
}
