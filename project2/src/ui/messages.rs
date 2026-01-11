use bevy::ecs::{message::MessageWriter, system::SystemParam};

use crate::{
    common::messages::NextTurnMessage,
    country::messages::ChangeRelationMessage,
    map::messages::{BuildBuildingMessage, SpawnArmyMessage},
};

#[derive(SystemParam)]
pub struct UiGameMessages<'w> {
    pub next_turn: MessageWriter<'w, NextTurnMessage>,
    pub build_building: MessageWriter<'w, BuildBuildingMessage>,
    pub spawn_army: MessageWriter<'w, SpawnArmyMessage>,
    pub change_relation: MessageWriter<'w, ChangeRelationMessage>,
}
