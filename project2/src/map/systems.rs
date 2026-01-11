use std::cmp::min;

use anyhow::{Result, anyhow};
use bevy::{
    ecs::system::{IntoResult, SystemParam},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;
use noise::NoiseFn;
use rand::random;

use crate::{
    common::messages::NextTurnMessage,
    country::{
        components::OwnershipTile,
        resources::{Countries, Diplomacy, RelationStatus},
    },
    map::{
        components::*,
        messages::{BuildBuildingMessage, MoveArmyMessage, SpawnArmyMessage},
        resources::*,
    },
};

pub fn setup_map(
    mut commands: Commands,
    map_settings: Res<super::resources::MapSettings>,
    mut tile_grid: ResMut<TileMapGrid>,
) {
    let half_tile = map_settings.tile_size as f32 / 2.0;
    let offset_x = -((map_settings.width * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let offset_y = -((map_settings.height * map_settings.tile_size) as f32) / 2.0 + half_tile;

    let perlin = noise::Perlin::new(random());
    let scale = 0.05f32;

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

#[derive(SystemParam)]
pub struct TileSelectionSystemResources<'w> {
    button_input: Res<'w, ButtonInput<MouseButton>>,
    tile_grid: Res<'w, TileMapGrid>,
    map_settings: Res<'w, MapSettings>,
}

pub fn tile_selection_system(
    mut egui_contexts: EguiContexts,
    camera_query: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    cursor_visibility_query: Single<(&mut Visibility, &mut Transform), With<SelectionCursor>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut selected_state: ResMut<SelectionState>,
    read_resources: TileSelectionSystemResources,
) -> Result<()> {
    if !read_resources.button_input.just_pressed(MouseButton::Left) {
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

    let world_pos = camera_query.viewport_to_world_2d(camera_global_transform, cursor_pos)?;

    let (offset_x, offset_y, x, y) =
        calculate_x_y_indicies(&read_resources.map_settings, world_pos);

    let cursor_visibility = cursor_visibility_query.as_mut();

    let Some(tile) = read_resources.tile_grid.grid.get(&(x, y)) else {
        *cursor_visibility = Visibility::Hidden;
        selected_state.selected_entity = None;
        selected_state.selected_tile = None;
        return Ok(());
    };

    update_selection_and_cursor(
        cursor_transform_query.as_mut(),
        read_resources.map_settings,
        selected_state,
        (offset_x, offset_y),
        (x, y),
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

pub fn build_building_system(
    mut msgr: MessageReader<BuildBuildingMessage>,
    mut commands: Commands,
    mut countries: ResMut<Countries>,
    asset_server: Res<AssetServer>,
    map_settings: Res<MapSettings>,
) {
    for msg in msgr.read() {
        if countries.countries[msg.country_idx].money < map_settings.building_cost {
            continue;
        }

        countries.countries[msg.country_idx].money -= map_settings.building_cost;

        commands
            .entity(msg.tile_entity)
            .insert(Building {})
            .with_children(|parent| {
                let building_texture = asset_server.load("building_texture.png");

                parent.spawn((
                    Sprite {
                        image: building_texture,
                        custom_size: Some(Vec2::new(
                            map_settings.tile_size as f32,
                            map_settings.tile_size as f32,
                        )),
                        ..Default::default()
                    },
                    Transform::from_xyz(0.0, 0.0, 4.0),
                ));
            });
    }
}

pub fn spawn_army_system(
    mut commands: Commands,
    mut msgr: MessageReader<SpawnArmyMessage>,
    mut countries: ResMut<Countries>,
    mut map_tile_query: Query<(&GridPosition, Option<&mut Army>), With<MapTile>>,
    ownership_tiles_query: Query<(&OwnershipTile, &GridPosition)>,
    map_settings: Res<MapSettings>,
    asset_server: Res<AssetServer>,
) -> anyhow::Result<()> {
    for spawn_army_message in msgr.read() {
        let (amount, spawn_army_cost) =
            clamp_number_of_units_to_country_budget(&countries, &map_settings, spawn_army_message);

        if amount < 1 {
            continue;
        }

        let (map_tile_grid_position, army_option) =
            map_tile_query.get_mut(spawn_army_message.tile_entity)?;

        if !check_if_on_owned_land(
            spawn_army_message.country_idx,
            map_tile_grid_position,
            &ownership_tiles_query,
        ) {
            return Err(anyhow!("Tried spawning units on foreign land"));
        }

        spawn_army_unit(
            &mut commands,
            amount,
            spawn_army_message.country_idx,
            &spawn_army_message.tile_entity,
            &mut army_option.map(|a| a.into_inner()),
            &asset_server,
            &map_settings,
        )?;

        countries.countries[spawn_army_message.country_idx].money -= spawn_army_cost;
    }

    Ok(())
}

#[derive(SystemParam)]
pub struct ShowMovementRangeSystemResources<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    selection: Res<'w, SelectionState>,
    map_settings: Res<'w, MapSettings>,
    diplomacy: Res<'w, Diplomacy>,
}

pub fn show_movement_range_system(
    mut commands: Commands,
    mut resources: ShowMovementRangeSystemResources,
    army_query: Query<&GridPosition, With<Army>>,
    tiles_query: Query<(&GridPosition, &MapTile)>,
    army_option_query: Query<Option<&mut Army>>,
    ownership_tiles_query: Query<(&OwnershipTile, &GridPosition)>,
) -> anyhow::Result<()> {
    let Some(army_entity) = resources.selection.selected_entity else {
        return Ok(());
    };

    let Ok(start_pos) = army_query.get(army_entity) else {
        return Ok(());
    };

    for (tile_pos, map_tile) in tiles_query.iter() {
        if start_pos.distance(tile_pos) <= 2.0
            && start_pos != tile_pos
            && validate_army_movement(
                &army_option_query,
                &MoveArmyMessage {
                    moved_army_entity: army_entity,
                    target_position: (*tile_pos).clone(),
                    number_of_units_to_move: 0,
                },
                &ownership_tiles_query,
                &resources.diplomacy,
            )?
        {
            let world_pos = grid_to_world(tile_pos, &resources.map_settings);
            let mut color = Color::from(&map_tile.tile_type).to_linear();

            color = color
                .with_red(1.0 - color.red)
                .with_green(1.0 - color.green)
                .with_blue(1.0 - color.blue);

            commands.spawn((
                Mesh2d(resources.meshes.add(Circle::new(10.0))),
                MeshMaterial2d(resources.materials.add(Color::from(color).with_alpha(0.5))),
                Transform::from_translation(world_pos.with_z(60.0)),
                HighlightOverlay,
                GridPosition::new(tile_pos.x, tile_pos.y),
            ));
        }
    }

    Ok(())
}

pub fn hide_movement_range_system(
    mut commands: Commands,
    overlay_query: Query<Entity, With<HighlightOverlay>>,
) {
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(SystemParam)]
pub struct MoveArmySystemQueries<'w, 's> {
    army_query: Query<'w, 's, Option<&'static mut Army>>,
    map_pos_query: Query<'w, 's, (Entity, &'static GridPosition), With<MapTile>>,
    army_sprite_query: Query<'w, 's, (Entity, &'static ChildOf), With<ArmySpriteTag>>,
    ownership_tiles_query: Query<'w, 's, (&'static OwnershipTile, &'static GridPosition)>,
}

pub fn move_army_system(
    mut commands: Commands,
    mut next_turn_msgr: MessageReader<NextTurnMessage>,
    mut army_movements: ResMut<ArmyMovements>,
    mut queries: MoveArmySystemQueries,
    asset_server: Res<AssetServer>,
    map_settings: Res<MapSettings>,
    diplomacy_resource: Res<Diplomacy>,
) -> anyhow::Result<()> {
    for _ in next_turn_msgr.read() {
        while let Some(move_army_message) = army_movements.get_movement() {
            if !validate_army_movement(
                &queries.army_query,
                &move_army_message,
                &queries.ownership_tiles_query,
                &diplomacy_resource,
            )? {
                continue;
            }

            let Some((target_entity, _)) = queries
                .map_pos_query
                .iter()
                .find(|(_, pos)| **pos == move_army_message.target_position)
            else {
                return Err(anyhow!(
                    "Map tile entity not found on position {:?}",
                    move_army_message.target_position
                ));
            };

            let moved_army_entity = move_army_message.moved_army_entity;
            let (country_idx, units_taken) = update_source_army_entity(
                &mut commands,
                &mut queries.army_query,
                queries.army_sprite_query,
                moved_army_entity,
                move_army_message,
            )?;
            let army_presence_on_target_field_option = queries.army_query.get_mut(target_entity)?;
            let Ok(_) = spawn_army_unit(
                &mut commands,
                units_taken,
                country_idx,
                &target_entity,
                &mut army_presence_on_target_field_option.map(|a| a.into_inner()),
                &asset_server,
                &map_settings,
            ) else {
                // battle - another country's unit present on the tile
                return Ok(());
            };
        }
    }

    Ok(())
}

pub fn army_ownership_claim_system(
    mut ownership_tiles_query: Query<(&mut OwnershipTile, &GridPosition)>,
    army_query: Query<(&Army, &GridPosition)>,
) -> anyhow::Result<()> {
    for (army, position) in army_query.iter() {
        let (mut ownership_tile, _) = ownership_tiles_query
            .iter_mut()
            .find(|(_, pos)| *pos == position)
            .ok_or(anyhow!("Map tile without ownership tile found"))?;

        if let Some(country_idx) = ownership_tile.country_id
            && country_idx != army.country_idx
        {
            ownership_tile.country_id = Some(army.country_idx);
        }
    }

    Ok(())
}

// helpers

fn validate_army_movement(
    army_query: &Query<Option<&mut Army>>,
    move_army_message: &MoveArmyMessage,
    ownership_tiles_query: &Query<(&OwnershipTile, &GridPosition)>,
    diplomacy_resource: &Res<Diplomacy>,
) -> anyhow::Result<bool> {
    let Some(army) = army_query.get(move_army_message.moved_army_entity)? else {
        return Err(anyhow!(
            "Did not found an army entity to move that was provided in the message"
        ));
    };

    let Some((ownership_tile, _)) = ownership_tiles_query
        .iter()
        .find(|(_, pos)| **pos == move_army_message.target_position)
    else {
        return Err(anyhow!(
            "Ownership tile not found on position {:?}",
            move_army_message.target_position
        ));
    };

    let Some(target_tile_country_idx) = ownership_tile.country_id else {
        return Ok(false); // cant move on unowned land
    };

    if army.country_idx != target_tile_country_idx
        && !matches!(
            diplomacy_resource.get_relation(army.country_idx, target_tile_country_idx),
            RelationStatus::AtWar
        )
    {
        return Ok(false); // moving only on own land while not at war
    }

    Ok(true)
}

fn update_source_army_entity(
    commands: &mut Commands<'_, '_>,
    army_query: &mut Query<'_, '_, Option<&mut Army>>,
    army_sprite_query: Query<'_, '_, (Entity, &ChildOf), With<ArmySpriteTag>>,
    moved_army_entity: Entity,
    move_army_message: super::messages::MoveArmyMessage,
) -> Result<(usize, i32), anyhow::Error> {
    let (country_idx, units_taken) = {
        let Some(moved_army) = &mut army_query.get_mut(moved_army_entity)? else {
            return Err(anyhow!("Source entity does not contain an Army"));
        };

        if move_army_message.number_of_units_to_move >= moved_army.number_of_units {
            commands.entity(moved_army_entity).remove::<Army>();

            let Some((army_sprite_entity, _)) = army_sprite_query
                .iter()
                .find(|(_, parent)| parent.0 == moved_army_entity)
            else {
                return Err(anyhow!("Could not find the sprite of the army component"));
            };

            commands.entity(army_sprite_entity).despawn();
        };
        let units_to_take = min(
            move_army_message.number_of_units_to_move,
            moved_army.number_of_units,
        );

        moved_army.number_of_units -= units_to_take;
        (moved_army.country_idx, units_to_take)
    };
    Ok((country_idx, units_taken))
}

fn clamp_number_of_units_to_country_budget(
    countries: &ResMut<'_, Countries>,
    map_settings: &Res<'_, MapSettings>,
    spawn_army_message: &SpawnArmyMessage,
) -> (i32, i32) {
    let mut amount = spawn_army_message.amount;
    let mut spawn_army_cost = map_settings.unit_cost * amount;
    let spawning_country_money = countries.countries[spawn_army_message.country_idx].money;

    if spawning_country_money < spawn_army_cost {
        amount = spawning_country_money / map_settings.unit_cost;
        spawn_army_cost = map_settings.unit_cost * amount;
    }
    (amount, spawn_army_cost)
}

fn grid_to_world(grid_position: &GridPosition, map_settings: &Res<MapSettings>) -> Vec3 {
    let half_tile = map_settings.tile_size as f32 / 2.0;
    let offset_x = -((map_settings.width * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let offset_y = -((map_settings.height * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let world_pos_x = (grid_position.x * map_settings.tile_size) as f32 + offset_x;
    let world_pos_y = (grid_position.y * map_settings.tile_size) as f32 + offset_y;
    Vec3::new(world_pos_x, world_pos_y, 0.0)
}

fn check_if_on_owned_land(
    country_idx: usize,
    position: &GridPosition,
    ownership_tiles_query: &Query<(&OwnershipTile, &GridPosition)>,
) -> bool {
    let ownership_tile = ownership_tiles_query.iter().find(|(tile, pos)| {
        if let Some(tile_country_id) = tile.country_id {
            return *pos == position && tile_country_id == country_idx;
        }

        false
    });

    ownership_tile.is_some()
}

fn spawn_army_unit(
    commands: &mut Commands,
    number_of_units: i32,
    country_idx: usize,
    map_tile_entity: &Entity,
    army_option: &mut Option<&mut Army>,
    asset_server: &Res<AssetServer>,
    map_settings: &Res<MapSettings>,
) -> anyhow::Result<()> {
    match army_option {
        Some(army) => {
            if army.country_idx != country_idx {
                return Err(anyhow!(
                    "Tried spawning units where foreign ones are present"
                ));
            }

            army.number_of_units += number_of_units;
        }
        None => {
            commands
                .entity(*map_tile_entity)
                .insert(Army {
                    country_idx,
                    number_of_units,
                })
                .with_children(|parent| {
                    parent.spawn((
                        Sprite {
                            image: asset_server.load("army_texture.png"),
                            custom_size: Some(Vec2 {
                                x: map_settings.tile_size as f32,
                                y: map_settings.tile_size as f32,
                            }),
                            ..Default::default()
                        },
                        Transform::from_xyz(0.0, 0.0, 5.0),
                        ArmySpriteTag {},
                    ));
                });
        }
    }

    Ok(())
}

fn calculate_x_y_indicies(
    map_settings: &Res<'_, MapSettings>,
    world_pos: Vec2,
) -> (f32, f32, i32, i32) {
    let half_tile = map_settings.tile_size as f32 / 2.0;

    let offset_x = -((map_settings.width * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let offset_y = -((map_settings.height * map_settings.tile_size) as f32) / 2.0 + half_tile;

    let x = ((world_pos.x - offset_x) / map_settings.tile_size as f32).round() as i32;
    let y = ((world_pos.y - offset_y) / map_settings.tile_size as f32).round() as i32;
    (offset_x, offset_y, x, y)
}

fn update_selection_and_cursor(
    cursor_transform: &mut Transform,
    map_settings: Res<'_, MapSettings>,
    mut selected_state: ResMut<'_, SelectionState>,
    (offset_x, offset_y): (f32, f32),
    (x, y): (i32, i32),
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
    x: i32,
    y: i32,
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
    scale: f32,
    x: i32,
    y: i32,
) -> MapTileType {
    let moisture_noise = perlin.get([
        ((x + map_settings.width) as f32 * scale) as f64,
        ((y + map_settings.height) as f32 * scale) as f64,
    ]);

    let elevation_noise = perlin.get([(x as f32 * scale) as f64, (y as f32 * scale) as f64]);

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
