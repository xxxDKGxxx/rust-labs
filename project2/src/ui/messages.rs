use bevy::ecs::{message::MessageWriter, system::SystemParam};

use crate::{
    ai::systems::AiTurnMessage,
    common::messages::SaveGameMessage,
    country::messages::ChangeRelationMessage,
    map::messages::{BuildBuildingMessage, SpawnArmyMessage},
};

#[derive(SystemParam)]
pub struct UiGameMessages<'w> {
    pub ai_turn: MessageWriter<'w, AiTurnMessage>,
    pub build_building: MessageWriter<'w, BuildBuildingMessage>,
    pub spawn_army: MessageWriter<'w, SpawnArmyMessage>,
    pub change_relation: MessageWriter<'w, ChangeRelationMessage>,
    pub save_game: MessageWriter<'w, SaveGameMessage>,
}
