use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Deserialize, Serialize, Clone, Copy)]
pub struct OwnershipTile {
    pub country_id: Option<usize>,
}

impl OwnershipTile {
    pub fn new(country_id: Option<usize>) -> Self {
        Self { country_id }
    }
}

#[derive(Component)]
pub struct CountryFlag {
    pub idx: usize,
}
