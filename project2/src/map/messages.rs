use bevy::ecs::{entity::Entity, message::Message};

use crate::common::components::GridPosition;

#[derive(Message, Clone)]
pub struct SaveMapMessage {
    pub map_name: String,
}

#[derive(Message)]
pub struct BuildBuildingMessage {
    pub tile_entity: Entity,
    pub country_idx: usize,
}

#[derive(Message)]
pub struct SpawnArmyMessage {
    pub tile_entity: Entity,
    pub country_idx: usize,
    pub amount: i32,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct MoveArmyMessage {
    pub moved_army_entity: Entity,
    pub target_position: GridPosition,
    pub number_of_units_to_move: i32,
}

#[derive(Message, Clone)]
pub struct ArmyBattleMessage {
    pub army_a_entity: Entity,
    pub army_b_entity: Entity,
}
