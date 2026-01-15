use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};

use crate::map::messages::MoveArmyMessage;

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct MapSettings {
    pub width: i32,
    pub height: i32,
    pub tile_size: i32,
    pub building_cost: i32,
    pub unit_cost: i32,
}

impl MapSettings {
    pub fn new(
        width: i32,
        height: i32,
        tile_size: i32,
        building_cost: i32,
        unit_cost: i32,
    ) -> Self {
        Self {
            width,
            height,
            tile_size,
            building_cost,
            unit_cost,
        }
    }
}

#[derive(Resource)]
pub struct TileMapGrid {
    pub grid: HashMap<(i32, i32), Entity>,
}

impl TileMapGrid {
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
        }
    }
}

impl FromWorld for TileMapGrid {
    fn from_world(_: &mut World) -> Self {
        Self::new()
    }
}

#[derive(Resource)]
pub struct SelectionState {
    pub selected_tile: Option<(i32, i32)>,
    pub selected_entity: Option<Entity>,
}

impl FromWorld for SelectionState {
    fn from_world(_: &mut World) -> Self {
        Self {
            selected_tile: None,
            selected_entity: None,
        }
    }
}

#[derive(Resource, Default)]
pub enum MapVisibilityState {
    #[default]
    Terrain,
    PoliticalOnly,
}

#[derive(Resource)]
pub struct ArmyMovements {
    queue: VecDeque<MoveArmyMessage>,
}

impl FromWorld for ArmyMovements {
    fn from_world(_: &mut World) -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl ArmyMovements {
    pub fn add_movement(&mut self, movement: MoveArmyMessage) {
        self.queue.push_back(movement);
    }

    pub fn get_movement(&mut self) -> Option<MoveArmyMessage> {
        self.queue.pop_front()
    }
}
