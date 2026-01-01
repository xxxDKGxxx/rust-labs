use crate::{
    log_error,
    ui::{messages::NextTurnMessage, resources::TurnCounter, systems::*},
};
use bevy::{app::*, ecs::system::IntoSystem};
use bevy_egui::EguiPrimaryContextPass;

pub mod components;
pub mod messages;
pub mod resources;
pub mod systems;

pub struct UiPlugin {}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, setup_ui_label)
            .add_systems(EguiPrimaryContextPass, setup_controls_ui.pipe(log_error))
            .add_systems(Update, update_turn_counter)
            .add_systems(PostUpdate, display_country_name)
            .add_message::<NextTurnMessage>()
            .init_resource::<TurnCounter>();
    }
}
