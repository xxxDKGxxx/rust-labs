use bevy::prelude::*;

pub mod systems;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, systems::ai_system);
    }
}
