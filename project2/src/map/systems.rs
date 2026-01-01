use anyhow::Result;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use noise::NoiseFn;
use rand::random;

use crate::map::{components::*, resources::*};

pub fn setup_map(
    mut commands: Commands,
    map_settings: Res<super::resources::MapSettings>,
    mut tile_grid: ResMut<TileMapGrid>,
) {
    let half_tile = map_settings.tile_size as f32 / 2.0;
    let offset_x = -((map_settings.width * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let offset_y = -((map_settings.height * map_settings.tile_size) as f32) / 2.0 + half_tile;

    let perlin = noise::Perlin::new(random());
    let scale = 0.05f64;

    for x in 0..map_settings.width {
        for y in 0..map_settings.height {
            let world_pos_x = (x * map_settings.tile_size) as f32 + offset_x;
            let world_pos_y = (y * map_settings.tile_size) as f32 + offset_y;

            let tile_type = tile_type_from_noise(&map_settings, perlin, scale, x, y);

            let entity = spawn_tile(
                &mut commands,
                &map_settings,
                x,
                y,
                world_pos_x,
                world_pos_y,
                tile_type,
            );

            tile_grid.grid.insert((x, y), entity);
        }
    }
}

pub fn setup_cursor(mut commands: Commands, map_settings: Res<MapSettings>) {
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.0, 0.5),
            custom_size: Some(Vec2::new(
                map_settings.tile_size as f32,
                map_settings.tile_size as f32,
            )),
            ..Default::default()
        },
        Visibility::Hidden,
        SelectionCursor {},
    ));
}

pub fn tile_selection_system(
    mut egui_contexts: EguiContexts,
    camera_query: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    button_input: Res<ButtonInput<MouseButton>>,
    tile_grid: Res<TileMapGrid>,
    cursor_visibility_query: Single<(&mut Visibility, &mut Transform), With<SelectionCursor>>,
    window: Single<&Window, With<PrimaryWindow>>,
    map_settings: Res<MapSettings>,
    mut selected_state: ResMut<SelectionState>,
) -> Result<()> {
    if !button_input.just_pressed(MouseButton::Left) {
        return Ok(());
    }

    if egui_contexts.ctx_mut()?.is_pointer_over_area() {
        return Ok(());
    }

    let (camera_query, camera_global_transform) = camera_query.into_inner();
    let (mut cursor_visibility_query, mut cursor_transform_query) =
        cursor_visibility_query.into_inner();

    let Some(cursor_pos) = window.cursor_position() else {
        return Ok(());
    };

    let world_pos = camera_query.viewport_to_world_2d(&camera_global_transform, cursor_pos)?;

    let (offset_x, offset_y, x, y) = calculate_x_y_indicies(&map_settings, world_pos);

    let cursor_visibility = cursor_visibility_query.as_mut();

    let Some(tile) = tile_grid.grid.get(&(x, y)) else {
        *cursor_visibility = Visibility::Hidden;
        selected_state.selected_entity = None;
        selected_state.selected_tile = None;
        return Ok(());
    };

    update_selection_and_cursor(
        cursor_transform_query.as_mut(),
        map_settings,
        selected_state,
        offset_x,
        offset_y,
        x,
        y,
        cursor_visibility,
        tile,
    );
    Ok(())
}

pub fn map_visibility_toggling_system(
    input: Res<ButtonInput<KeyCode>>,
    mut map_state: ResMut<MapVisibilityState>,
) {
    if !input.just_pressed(KeyCode::KeyM) {
        return;
    }

    *map_state = match *map_state {
        MapVisibilityState::Terrain => MapVisibilityState::PoliticalOnly,
        MapVisibilityState::PoliticalOnly => MapVisibilityState::Terrain,
    };
}

pub fn update_visibility_system(
    map_state: Res<MapVisibilityState>,
    map_tile_visibility: Query<&mut Visibility, With<MapTile>>,
) {
    if !map_state.is_changed() {
        return;
    }

    let vis = match *map_state {
        MapVisibilityState::Terrain => Visibility::Visible,
        MapVisibilityState::PoliticalOnly => Visibility::Hidden,
    };

    for mut tile_visibility in map_tile_visibility {
        *tile_visibility = vis;
    }
}

fn calculate_x_y_indicies(
    map_settings: &Res<'_, MapSettings>,
    world_pos: Vec2,
) -> (f32, f32, u64, u64) {
    let half_tile = map_settings.tile_size as f32 / 2.0;

    let offset_x = -((map_settings.width * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let offset_y = -((map_settings.height * map_settings.tile_size) as f32) / 2.0 + half_tile;

    let x = ((world_pos.x - offset_x) / map_settings.tile_size as f32).round() as u64;
    let y = ((world_pos.y - offset_y) / map_settings.tile_size as f32).round() as u64;
    (offset_x, offset_y, x, y)
}

fn update_selection_and_cursor(
    cursor_transform: &mut Transform,
    map_settings: Res<'_, MapSettings>,
    mut selected_state: ResMut<'_, SelectionState>,
    offset_x: f32,
    offset_y: f32,
    x: u64,
    y: u64,
    cursor_visibility: &mut Visibility,
    tile: &Entity,
) {
    selected_state.selected_entity = Some(*tile);
    selected_state.selected_tile = Some((x, y));

    cursor_transform.translation.x = (x * map_settings.tile_size) as f32 + offset_x;
    cursor_transform.translation.y = (y * map_settings.tile_size) as f32 + offset_y;
    cursor_transform.translation.z = 100.0;

    *cursor_visibility = Visibility::Visible;
}

fn spawn_tile(
    commands: &mut Commands<'_, '_>,
    map_settings: &Res<'_, super::resources::MapSettings>,
    x: u64,
    y: u64,
    world_pos_x: f32,
    world_pos_y: f32,
    tile_type: MapTileType,
) -> Entity {
    commands
        .spawn((
            Sprite {
                color: Color::from(&tile_type),
                custom_size: Some(Vec2::new(
                    map_settings.tile_size as f32 - 3.0,
                    map_settings.tile_size as f32 - 3.0,
                )),
                ..Default::default()
            },
            Transform::from_xyz(world_pos_x, world_pos_y, 50f32),
            MapTile::new(tile_type),
            GridPosition::new(x, y),
        ))
        .id()
}

fn tile_type_from_noise(
    map_settings: &Res<'_, super::resources::MapSettings>,
    perlin: noise::Perlin,
    scale: f64,
    x: u64,
    y: u64,
) -> MapTileType {
    let moisture_noise = perlin.get([
        (x + map_settings.width) as f64 * scale,
        (y + map_settings.height) as f64 * scale,
    ]);

    let elevation_noise = perlin.get([x as f64 * scale, y as f64 * scale]);

    if elevation_noise < -0.1 {
        super::components::MapTileType::Water
    } else if elevation_noise < 0.0 {
        super::components::MapTileType::Sand
    } else if elevation_noise < 0.5 {
        if moisture_noise > 0.0 {
            super::components::MapTileType::Forest
        } else {
            super::components::MapTileType::Flat
        }
    } else {
        super::components::MapTileType::Mountain
    }
}
