use bevy::app::Plugin;

use crate::player::resources::PlayerData;

pub mod resources;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(PlayerData::new(0));
    }
}
