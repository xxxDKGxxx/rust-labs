use crate::{
    GameState, InGameStates, log_error,
    ui::{
        messages::NextTurnMessage,
        resources::{TurnCounter, UiModel},
        systems::*,
    },
};
use bevy::{
    app::*,
    ecs::{schedule::IntoScheduleConfigs, system::IntoSystem},
    state::{condition::in_state, state::OnEnter},
};
use bevy_egui::EguiPrimaryContextPass;

pub mod components;
pub mod messages;
pub mod resources;
pub mod systems;

pub struct UiPlugin {}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(GameState::InGame), setup_ui_label)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    setup_controls_ui.pipe(log_error),
                    setup_army_controls_ui.pipe(log_error),
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                main_menu_system
                    .pipe(log_error)
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(
                Update,
                update_turn_counter.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PostUpdate,
                (
                    handle_selection_change_when_moving_army
                        .run_if(in_state(InGameStates::MovingArmy)),
                    (display_country_name, display_unit_count).run_if(in_state(GameState::InGame)),
                ),
            )
            .add_message::<NextTurnMessage>()
            .init_resource::<TurnCounter>()
            .init_resource::<UiModel>();
    }
}
