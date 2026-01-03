use bevy::ecs::{entity::Entity, message::Message};

#[derive(Message)]
pub struct BuildBuildingMessage {
    pub tile_entity: Entity,
    pub country_idx: usize,
}
