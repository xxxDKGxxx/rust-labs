use bevy::{
    app::*,
    ecs::{schedule::IntoScheduleConfigs, system::IntoSystem},
    state::{condition::in_state, state::OnEnter},
};

use crate::{
    GameState,
    country::{resources::Countries, systems::*},
    log_error,
    map::systems::setup_map,
};

pub mod components;
pub mod resources;
pub mod systems;

pub struct CountryPlugin {}

impl Plugin for CountryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Countries>()
            .add_systems(
                OnEnter(GameState::InGame),
                (
                    setup_countries_system.after(setup_map),
                    setup_ownership_tiles.after(setup_countries_system),
                ),
            )
            .add_systems(
                Update,
                money_gathering_system
                    .pipe(log_error)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PostUpdate,
                update_ownership_tiles
                    .after(setup_ownership_tiles)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}
