use bevy::{
    app::*,
    ecs::{schedule::IntoScheduleConfigs, system::IntoSystem},
    state::{condition::in_state, state::OnEnter},
};

use crate::{
    GameState,
    country::{
        messages::ChangeRelationMessage,
        resources::{Countries, Diplomacy},
        systems::*,
    },
    log_error,
    map::systems::setup_map,
};

pub mod components;
pub mod messages;
pub mod resources;
pub mod systems;

pub struct CountryPlugin {}

impl Plugin for CountryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Countries>()
            .insert_resource(Diplomacy::new())
            .add_message::<ChangeRelationMessage>()
            .add_systems(
                OnEnter(GameState::InGame),
                (
                    setup_countries_system.after(setup_map),
                    setup_ownership_tiles.after(setup_countries_system),
                    setup_country_flags_system.after(setup_ownership_tiles),
                ),
            )
            .add_systems(
                Update,
                (
                    money_gathering_system.pipe(log_error),
                    relation_managing_system,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PostUpdate,
                (
                    update_ownership_tiles.after(setup_ownership_tiles),
                    update_country_flag_system.after(update_ownership_tiles),
                    save_countries_system.pipe(log_error),
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}
