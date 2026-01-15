use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize)]
pub struct PlayerData {
    pub country_idx: usize,
}

impl PlayerData {
    pub fn new(country_idx: usize) -> Self {
        Self { country_idx }
    }
}
