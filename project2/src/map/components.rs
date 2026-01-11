use bevy::prelude::*;

#[derive(PartialEq)]
pub enum MapTileType {
    Water,
    Sand,
    Flat,
    Mountain,
    Forest,
}

impl From<&MapTileType> for Color {
    fn from(value: &MapTileType) -> Self {
        match value {
            MapTileType::Water => Color::srgb(0f32, 0f32, 1f32),
            MapTileType::Flat => Color::srgb(0f32, 1f32, 0f32),
            MapTileType::Mountain => Color::srgb(0.5f32, 0.5f32, 0.5f32),
            MapTileType::Forest => Color::srgb(0f32, 0.5f32, 0f32),
            MapTileType::Sand => Color::srgb(1.0, 1.0, 0.75),
        }
    }
}

#[derive(Component)]
pub struct MapTile {
    pub tile_type: MapTileType,
}

impl MapTile {
    pub fn new(tile_type: MapTileType) -> Self {
        Self { tile_type }
    }
}

#[derive(Component, Debug, PartialEq, Clone)]
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

#[derive(Component)]
pub struct SelectionCursor {}

#[derive(Component)]
pub struct Building {}

#[derive(Component)]
pub struct Army {
    pub country_idx: usize,
    pub number_of_units: i32,
}

#[derive(Component)]
pub struct ArmySpriteTag {}

#[derive(Component)]
pub struct HighlightOverlay;
