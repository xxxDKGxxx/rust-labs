use bevy::prelude::*;

use crate::{ai::systems::AiTurnMessage, log_error, map::systems::army_position_sync_system};

pub mod systems;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::ai_system
                .pipe(log_error)
                .after(army_position_sync_system),
        )
        .add_message::<AiTurnMessage>();
    }
}
