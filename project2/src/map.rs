use bevy::prelude::*;

use crate::{
    log_error,
    map::{resources::*, systems::*},
};

pub mod components;
pub mod resources;
pub mod systems;

pub struct MapPlugin {}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapSettings::new(100, 50, 100))
            .init_resource::<TileMapGrid>()
            .init_resource::<SelectionState>()
            .init_resource::<MapVisibilityState>()
            .add_systems(Startup, (setup_map, setup_cursor))
            .add_systems(
                Update,
                (
                    tile_selection_system.pipe(log_error),
                    update_visibility_system,
                    map_visibility_toggling_system,
                ),
            );
    }
}
