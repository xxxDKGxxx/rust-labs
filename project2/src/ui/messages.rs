use bevy::ecs::{
    message::{Message, MessageWriter},
    system::SystemParam,
};

use crate::{
    ai::systems::AiTurnMessage,
    common::messages::{NextTurnMessage, SaveGameMessage},
    country::messages::{ChangeRelationMessage, ProposePeaceMessage},
    map::messages::{BuildBuildingMessage, SaveMapMessage, SpawnArmyMessage},
};

#[derive(SystemParam)]
pub struct UiGameMessages<'w> {
    pub ai_turn: MessageWriter<'w, AiTurnMessage>,
    pub build_building: MessageWriter<'w, BuildBuildingMessage>,
    pub spawn_army: MessageWriter<'w, SpawnArmyMessage>,
    pub change_relation: MessageWriter<'w, ChangeRelationMessage>,
    pub save_map: MessageWriter<'w, SaveMapMessage>,
    pub save_game: MessageWriter<'w, SaveGameMessage>,
    pub ui_click_message: MessageWriter<'w, UiClickMessage>,
    pub next_turn_message: MessageWriter<'w, NextTurnMessage>,
    pub propose_peace_message: MessageWriter<'w, ProposePeaceMessage>,
}

#[derive(Message)]
pub struct UiClickMessage {}
