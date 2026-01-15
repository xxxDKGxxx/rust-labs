use bevy::prelude::*;

use crate::ai::systems::AiTurnMessage;

pub mod systems;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, systems::ai_system)
            .add_message::<AiTurnMessage>();
    }
}
