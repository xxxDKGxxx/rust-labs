use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Clone, Copy, Eq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Self) -> f32 {
        (((self.x - other.x) as f32).powi(2) + ((self.y - other.y) as f32).powi(2)).sqrt()
    }
}
