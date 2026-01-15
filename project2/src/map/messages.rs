use bevy::ecs::{entity::Entity, message::Message};

use crate::common::components::GridPosition;

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

#[derive(Message, Debug)]
pub struct MoveArmyMessage {
    pub moved_army_entity: Entity,
    pub target_position: GridPosition,
    pub number_of_units_to_move: i32,
}
