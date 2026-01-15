use bevy::prelude::*;

use crate::{
    GameState,
    common::{
        messages::{NextTurnMessage, SaveGameMessage},
        systems::{despawn_everything, go_to_in_game_state},
    },
};

pub mod components;
pub mod messages;
pub mod systems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LoadSet {
    Load,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GenerateSet {
    Generate,
}

pub struct CommonPlugin {}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NextTurnMessage>()
            .add_message::<SaveGameMessage>()
            .configure_sets(OnEnter(GameState::Loading), (LoadSet::Load,))
            .configure_sets(OnEnter(GameState::Generating), (GenerateSet::Generate,))
            .add_systems(OnEnter(GameState::Loading), despawn_everything)
            .add_systems(
                Update,
                go_to_in_game_state
                    .after(LoadSet::Load)
                    .run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                Update,
                go_to_in_game_state
                    .after(GenerateSet::Generate)
                    .run_if(in_state(GameState::Generating)),
            );
    }
}
