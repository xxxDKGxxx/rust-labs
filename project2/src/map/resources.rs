use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource)]
pub struct MapSettings {
    pub width: u64,
    pub height: u64,
    pub tile_size: u64,
}

impl MapSettings {
    pub fn new(width: u64, height: u64, tile_size: u64) -> Self {
        Self {
            width,
            height,
            tile_size,
        }
    }
}

#[derive(Resource)]
pub struct TileMapGrid {
    pub grid: HashMap<(u64, u64), Entity>,
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
    pub selected_tile: Option<(u64, u64)>,
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
