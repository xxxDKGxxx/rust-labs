use bevy::prelude::*;

use crate::{
    GameState, InGameStates,
    common::{GenerateSet, LoadSet},
    log_error,
    map::{
        messages::{BuildBuildingMessage, SpawnArmyMessage},
        resources::*,
        systems::*,
    },
};

pub mod components;
pub mod messages;
pub mod resources;
pub mod systems;
pub struct MapPlugin {}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapSettings::new(100, 50, 100, 10000, 100))
            .init_resource::<TileMapGrid>()
            .init_resource::<SelectionState>()
            .init_resource::<MapVisibilityState>()
            .init_resource::<ArmyMovements>()
            .add_systems(
                OnEnter(GameState::Generating),
                setup_map.in_set(GenerateSet::Generate),
            )
            .add_systems(
                OnEnter(GameState::Loading),
                load_map_system.pipe(log_error).in_set(LoadSet::Load),
            )
            .add_systems(OnEnter(GameState::InGame), setup_cursor)
            .add_systems(
                Update,
                (
                    tile_selection_system.pipe(log_error),
                    update_visibility_system,
                    map_visibility_toggling_system,
                    build_building_system,
                    spawn_army_system
                        .pipe(log_error)
                        .before(army_position_sync_system),
                    army_ownership_claim_system.pipe(log_error),
                    army_position_sync_system,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnEnter(InGameStates::MovingArmy),
                show_movement_range_system.pipe(log_error),
            )
            .add_systems(OnExit(InGameStates::MovingArmy), hide_movement_range_system)
            .add_systems(
                PostUpdate,
                (
                    move_army_system.pipe(log_error),
                    save_map_system.pipe(log_error),
                    sync_army_colors_system,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_message::<BuildBuildingMessage>()
            .add_message::<SpawnArmyMessage>();
    }
}
