use bevy::{app::*, ecs::schedule::IntoScheduleConfigs};

use crate::{
    country::{resources::Countries, systems::*},
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
                Startup,
                (
                    setup_countries_system.after(setup_map),
                    setup_ownership_tiles.after(setup_countries_system),
                ),
            )
            .add_systems(Update, money_gathering_system)
            .add_systems(
                PostUpdate,
                update_ownership_tiles.after(setup_ownership_tiles),
            );
    }
}
