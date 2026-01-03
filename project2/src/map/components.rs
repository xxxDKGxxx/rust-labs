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

#[derive(Component, Debug, PartialEq)]
pub struct GridPosition {
    pub x: u64,
    pub y: u64,
}

impl GridPosition {
    pub fn new(x: u64, y: u64) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
pub struct SelectionCursor {}

#[derive(Component)]
pub struct Building {}
