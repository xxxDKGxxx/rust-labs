use bevy::ecs::{
    message::{Message, MessageWriter},
    system::SystemParam,
};

use crate::map::messages::BuildBuildingMessage;

#[derive(SystemParam)]
pub struct UiGameMessages<'w> {
    pub next_turn: MessageWriter<'w, NextTurnMessage>,
    pub build_building: MessageWriter<'w, BuildBuildingMessage>,
}

#[derive(Message)]
pub struct NextTurnMessage {}
