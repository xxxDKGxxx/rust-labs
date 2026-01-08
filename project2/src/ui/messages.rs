use bevy::ecs::{message::MessageWriter, system::SystemParam};

use crate::{common::messages::NextTurnMessage, map::messages::BuildBuildingMessage};

#[derive(SystemParam)]
pub struct UiGameMessages<'w> {
    pub next_turn: MessageWriter<'w, NextTurnMessage>,
    pub build_building: MessageWriter<'w, BuildBuildingMessage>,
}
