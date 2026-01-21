use bevy::prelude::*;

use crate::{
    InGameStates,
    ai::{resources::AiProcessing, systems::AiTurnMessage},
    log_error,
    map::systems::army_position_sync_system,
};

pub mod resources;
pub mod systems;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::ai_system
                .pipe(log_error)
                .after(army_position_sync_system)
                .run_if(in_state(InGameStates::AiTurn)),
        )
        .add_message::<AiTurnMessage>()
        .init_resource::<AiProcessing>();
    }
}
