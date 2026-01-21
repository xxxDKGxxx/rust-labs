use std::cmp::min;

use anyhow::{Result, anyhow};
use bevy::{
    ecs::system::SystemParam, platform::collections::HashMap, prelude::*, window::PrimaryWindow,
};
use bevy_egui::EguiContexts;
use itertools::Itertools;
use noise::NoiseFn;
use rand::random;
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        components::GridPosition,
        messages::{NextTurnMessage, SaveGameMessage},
        systems::{SAVE_PATH, get_save_path},
    },
    country::{
        components::OwnershipTile,
        resources::{Countries, Diplomacy, RelationStatus},
    },
    map::{
        components::*,
        messages::{ArmyBattleMessage, BuildBuildingMessage, MoveArmyMessage, SpawnArmyMessage},
        resources::*,
    },
    ui::resources::GameLoadState,
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

fn get_world_pos_from_cursor(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    if let Some(cursor_pos) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos)
    {
        return Some(world_pos);
    };

    None
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

    let (camera, camera_transform) = camera_query.into_inner();
    let (mut cursor_visibility, mut cursor_transform) = cursor_visibility_query.into_inner();

    let Some(world_pos) = get_world_pos_from_cursor(&window, camera, camera_transform) else {
        return Ok(());
    };

    let (offset_x, offset_y, x, y) =
        calculate_x_y_indicies(&read_resources.map_settings, world_pos);

    let cursor_visibility = cursor_visibility.as_mut();

    let Some(tile) = read_resources.tile_grid.grid.get(&(x, y)) else {
        *cursor_visibility = Visibility::Hidden;
        selected_state.selected_entity = None;
        selected_state.selected_tile = None;
        return Ok(());
    };

    update_selection_and_cursor(
        cursor_transform.as_mut(),
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
    mut map_tile_visibility: Query<&mut Visibility, With<MapTile>>,
) {
    if !map_state.is_changed() {
        return;
    }

    let vis = match *map_state {
        MapVisibilityState::Terrain => Visibility::Visible,
        MapVisibilityState::PoliticalOnly => Visibility::Hidden,
    };

    for mut tile_visibility in map_tile_visibility.iter_mut() {
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

#[derive(SystemParam)]
pub struct SpawnArmySystemQueries<'w, 's> {
    army_query: Query<'w, 's, (&'static mut Army, &'static GridPosition)>,
    map_tile_query: Query<'w, 's, &'static GridPosition, With<MapTile>>,
    ownership_tile_query: Query<'w, 's, (&'static GridPosition, &'static OwnershipTile)>,
}

fn validate_spawn_location(
    spawn_message: &SpawnArmyMessage,
    queries: &SpawnArmySystemQueries,
) -> Result<bool> {
    let map_tile_grid_position = queries.map_tile_query.get(spawn_message.tile_entity)?;
    let Some((_, ownership_tile)) = queries
        .ownership_tile_query
        .iter()
        .find(|(pos, _)| **pos == *map_tile_grid_position)
    else {
        return Err(anyhow!(
            "Did not find an ownership tile at pos: {} {}",
            map_tile_grid_position.x,
            map_tile_grid_position.y
        ));
    };

    if let Some(country_id) = ownership_tile.country_id {
        if country_id != spawn_message.country_idx {
            return Err(anyhow!("Tried spawning units on foreign land"));
        }
    } else {
        return Err(anyhow!("Tried spawning units on unowned land"));
    }
    Ok(true)
}

fn update_or_spawn_army(
    commands: &mut Commands,
    queries: &mut SpawnArmySystemQueries,
    spawn_army_message: &SpawnArmyMessage,
    amount: i32,
    asset_server: &AssetServer,
    map_settings: &MapSettings,
) -> Result<()> {
    let map_tile_grid_position = queries.map_tile_query.get(spawn_army_message.tile_entity)?;
    let existing_army = queries
        .army_query
        .iter_mut()
        .find(|(_, pos)| **pos == *map_tile_grid_position);

    if let Some((mut army, _)) = existing_army {
        if army.country_idx == spawn_army_message.country_idx {
            army.number_of_units += amount;
        } else {
            return Err(anyhow!(
                "Found foreign unit on the field while recruiting an army"
            ));
        }
    } else {
        spawn_army_unit(
            commands,
            Army {
                country_idx: spawn_army_message.country_idx,
                number_of_units: amount,
            },
            *map_tile_grid_position,
            asset_server,
            map_settings,
        );
    }
    Ok(())
}

pub fn spawn_army_system(
    mut commands: Commands,
    mut msgr: MessageReader<SpawnArmyMessage>,
    mut countries: ResMut<Countries>,
    mut queries: SpawnArmySystemQueries,
    map_settings: Res<MapSettings>,
    asset_server: Res<AssetServer>,
) -> anyhow::Result<()> {
    for spawn_army_message in msgr.read() {
        let (amount, spawn_army_cost) =
            clamp_number_of_units_to_country_budget(&countries, &map_settings, spawn_army_message);
        if amount < 1 {
            continue;
        }

        if !validate_spawn_location(spawn_army_message, &queries)? {
            continue;
        }

        update_or_spawn_army(
            &mut commands,
            &mut queries,
            spawn_army_message,
            amount,
            &asset_server,
            &map_settings,
        )?;

        countries.countries[spawn_army_message.country_idx].money -= spawn_army_cost;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct MapSaveState {
    map_tiles: Vec<(MapTile, GridPosition, bool)>,
    armies: Vec<(Army, GridPosition)>,
    map_settings: MapSettings,
}

const SAVE_FILE_NAME: &str = "save_map.json";

pub fn save_map_system(
    mut save_game_message_reader: MessageReader<SaveGameMessage>,
    armies_query: Query<(&Army, &GridPosition)>,
    map_tiles_query: Query<(&MapTile, &GridPosition, Has<Building>)>,
    map_settings: Res<MapSettings>,
) -> anyhow::Result<()> {
    for save_game_message in save_game_message_reader.read() {
        let mut armies: Vec<(Army, GridPosition)> = Vec::new();
        let mut map_tiles: Vec<(MapTile, GridPosition, bool)> = Vec::new();

        for (army, position) in armies_query.iter() {
            armies.push((army.clone(), *position));
        }

        for (map_tile, position, has_building) in map_tiles_query.iter() {
            map_tiles.push(((*map_tile).clone(), *position, has_building));
        }

        let map_save_state = MapSaveState {
            map_tiles,
            armies,
            map_settings: map_settings.clone(),
        };

        let save_json = serde_json::to_string_pretty(&map_save_state)?;

        std::fs::create_dir_all(format!("{}/{}", SAVE_PATH, save_game_message.save_name))?;
        std::fs::write(
            format!(
                "{}/{}/{}",
                SAVE_PATH, save_game_message.save_name, SAVE_FILE_NAME
            ),
            save_json,
        )?;
    }

    Ok(())
}

fn spawn_loaded_tiles(
    commands: &mut Commands,
    state: &MapSaveState,
    asset_server: &AssetServer,
    tile_grid: &mut TileMapGrid,
) {
    for (map_tile, grid_position, has_building) in &state.map_tiles {
        let world_pos = grid_to_world(grid_position, &state.map_settings);
        let mut entity_commands = commands.spawn((
            map_tile.clone(),
            *grid_position,
            Sprite {
                color: Color::from(&map_tile.tile_type),
                custom_size: Some(Vec2::new(
                    state.map_settings.tile_size as f32 - 3.0,
                    state.map_settings.tile_size as f32 - 3.0,
                )),
                ..Default::default()
            },
            Transform::from_translation(world_pos.with_z(50.0)),
        ));
        if *has_building {
            entity_commands.insert(Building {});
            entity_commands.with_children(|parent| {
                let building_texture = asset_server.load("building_texture.png");

                parent.spawn((
                    Sprite {
                        image: building_texture,
                        custom_size: Some(Vec2::new(
                            state.map_settings.tile_size as f32,
                            state.map_settings.tile_size as f32,
                        )),
                        ..Default::default()
                    },
                    Transform::from_xyz(0.0, 0.0, 4.0),
                ));
            });
        }
        tile_grid
            .grid
            .insert((grid_position.x, grid_position.y), entity_commands.id());
    }
}

fn spawn_loaded_armies(commands: &mut Commands, state: &MapSaveState, asset_server: &AssetServer) {
    for (army, grid_position) in &state.armies {
        spawn_army_unit(
            commands,
            army.clone(),
            *grid_position,
            asset_server,
            &state.map_settings,
        );
    }
}

pub fn load_map_system(
    mut commands: Commands,
    load_state: Res<GameLoadState>,
    asset_server: Res<AssetServer>,
    mut tile_grid: ResMut<TileMapGrid>,
) -> anyhow::Result<()> {
    if let Some(save_name) = &load_state.save_name {
        let path = format!("{}/{}", get_save_path(save_name), SAVE_FILE_NAME);
        let data = std::fs::read_to_string(path)?;
        let state: MapSaveState = serde_json::from_str(&data)?;

        commands.insert_resource(state.map_settings.clone());

        spawn_loaded_tiles(&mut commands, &state, &asset_server, &mut tile_grid);
        spawn_loaded_armies(&mut commands, &state, &asset_server);
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

fn highlight_tile(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
    tile_pos: &GridPosition,
    map_tile: &MapTile,
    map_settings: &MapSettings,
) {
    let world_pos = grid_to_world(tile_pos, map_settings);
    let mut color = Color::from(&map_tile.tile_type).to_linear();

    color = color
        .with_red(1.0 - color.red)
        .with_green(1.0 - color.green)
        .with_blue(1.0 - color.blue);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(Color::from(color).with_alpha(0.5))),
        Transform::from_translation(world_pos.with_z(60.0)),
        HighlightOverlay,
        GridPosition::new(tile_pos.x, tile_pos.y),
    ));
}

pub fn show_movement_range_system(
    mut commands: Commands,
    mut resources: ShowMovementRangeSystemResources,
    army_query: Query<(Entity, &GridPosition, &Army)>,
    tiles_query: Query<(&GridPosition, &MapTile)>,
    ownership_tiles_query: Query<(&OwnershipTile, &GridPosition), Without<Army>>,
) -> anyhow::Result<()> {
    let Some((selected_tile_x, selected_tile_y)) = resources.selection.selected_tile else {
        return Ok(());
    };

    let Some((army_entity, start_pos, army)) = army_query
        .iter()
        .find(|(_, pos, _)| pos.x == selected_tile_x && pos.y == selected_tile_y)
    else {
        return Ok(());
    };

    for (tile_pos, map_tile) in tiles_query.iter() {
        if start_pos.distance(tile_pos) <= 2.0
            && start_pos != tile_pos
            && validate_army_movement(
                army,
                &MoveArmyMessage {
                    moved_army_entity: army_entity,
                    target_position: *tile_pos,
                    number_of_units_to_move: 0,
                },
                &ownership_tiles_query,
                &resources.diplomacy,
            )?
        {
            highlight_tile(
                &mut commands,
                &mut resources.materials,
                &mut resources.meshes,
                tile_pos,
                map_tile,
                &resources.map_settings,
            );
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
    army_queries: Query<'w, 's, (&'static mut Army, &'static mut GridPosition)>,
    ownership_tiles_query:
        Query<'w, 's, (&'static OwnershipTile, &'static GridPosition), Without<Army>>,
}

pub fn detect_army_collisions_system(
    mut army_battles: ResMut<ArmyBattles>,
    mut army_movements: ResMut<ArmyMovements>,
    army_pos_query: Query<(Entity, &GridPosition, &Army)>,
) {
    if !army_movements.is_changed() {
        return;
    }
    let army_movements_cloned = army_movements.clone();
    let movements_with_source_positions: Vec<_> = army_movements_cloned
        .movements
        .iter()
        .filter_map(|movement| {
            let (_, source_pos, army) = army_pos_query
                .iter()
                .find(|(entity, _, _)| *entity == movement.moved_army_entity)?;
            Some((movement, source_pos, army.country_idx))
        })
        .collect();
    let collided_armies: Vec<_> = movements_with_source_positions
        .iter()
        .tuple_combinations()
        .filter_map(|(first_movement, second_movement)| {
            if first_movement.0.target_position == *second_movement.1
                && *first_movement.1 == second_movement.0.target_position // moving at each other positions
                && first_movement.2 != second_movement.2
            // different countries
            {
                return Some((first_movement.0, second_movement.0));
            }
            None
        })
        .collect();
    for (first_army_movement, second_army_movement) in collided_armies {
        army_movements
            .movements
            .retain(|v| v != first_army_movement && v != second_army_movement);
        army_battles.add_battle(ArmyBattleMessage {
            army_a_entity: first_army_movement.moved_army_entity,
            army_b_entity: second_army_movement.moved_army_entity,
        });
    }
}

pub fn resolve_army_battle_system(
    mut commands: Commands,
    mut next_turn_msgr: MessageReader<NextTurnMessage>,
    mut army_query: Query<&mut Army>,
    mut battles: ResMut<ArmyBattles>,
    mut army_battle_message_writer: MessageWriter<ArmyBattleMessage>,
) {
    for _ in next_turn_msgr.read() {
        while let Some(msg) = battles.get_battle() {
            let Ok([mut army_a, mut army_b]) =
                army_query.get_many_mut([msg.army_a_entity, msg.army_b_entity])
            else {
                continue;
            };

            army_battle(&mut commands, &mut army_a, &mut army_b, msg.clone());
            army_battle_message_writer.write(msg);
        }
    }
}

fn army_battle(
    commands: &mut Commands<'_, '_>,
    army_a: &mut Mut<'_, Army>,
    army_b: &mut Mut<'_, Army>,
    msg: ArmyBattleMessage,
) {
    let damage = min(army_a.number_of_units, army_b.number_of_units);

    army_a.number_of_units -= damage;
    army_b.number_of_units -= damage;

    if army_a.number_of_units <= 0 {
        commands.entity(msg.army_a_entity).despawn();
    }
    if army_b.number_of_units <= 0 {
        commands.entity(msg.army_b_entity).despawn();
    }
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
    if next_turn_msgr.is_empty() {
        return Ok(());
    }

    for _ in next_turn_msgr.read() {
        while let Some(move_army_message) = army_movements.get_movement() {
            process_army_movement(
                &mut commands,
                &mut queries,
                move_army_message,
                &diplomacy_resource,
                &asset_server,
                &map_settings,
            )?;
        }
    }
    println!("Move army system cleared movements");

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

pub fn sync_army_colors_system(
    mut army: Query<(&Army, &mut Sprite), Changed<Army>>,
    countries: Res<Countries>,
) {
    for (army, mut sprite) in army.iter_mut() {
        sprite.color = countries.countries[army.country_idx].color;
    }
}

// helpers
fn move_or_split_army(
    commands: &mut Commands,
    army: &mut Army,
    source_army_position: &mut GridPosition,
    move_army_message: &MoveArmyMessage,
    asset_server: &AssetServer,
    map_settings: &MapSettings,
) {
    let units_to_take = min(
        army.number_of_units,
        move_army_message.number_of_units_to_move,
    );

    if units_to_take == army.number_of_units {
        *source_army_position = move_army_message.target_position;
    } else {
        army.number_of_units -= units_to_take;

        spawn_army_unit(
            commands,
            Army {
                country_idx: army.country_idx,
                number_of_units: units_to_take,
            },
            move_army_message.target_position,
            asset_server,
            map_settings,
        );
    }
}

fn process_army_movement(
    commands: &mut Commands,
    queries: &mut MoveArmySystemQueries,
    move_army_message: MoveArmyMessage,
    diplomacy_resource: &Diplomacy,
    asset_server: &AssetServer,
    map_settings: &MapSettings,
) -> anyhow::Result<()> {
    let (mut army, mut source_army_position) = queries
        .army_queries
        .get_mut(move_army_message.moved_army_entity)?;

    if !validate_army_movement(
        &army,
        &move_army_message,
        &queries.ownership_tiles_query,
        diplomacy_resource,
    )? || move_army_message.number_of_units_to_move <= 0
    {
        return Ok(());
    }

    move_or_split_army(
        commands,
        &mut army,
        &mut source_army_position,
        &move_army_message,
        asset_server,
        map_settings,
    );

    Ok(())
}

fn validate_army_movement(
    army: &Army,
    move_army_message: &MoveArmyMessage,
    ownership_tiles_query: &Query<(&OwnershipTile, &GridPosition), Without<Army>>,
    diplomacy_resource: &Diplomacy,
) -> anyhow::Result<bool> {
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

fn grid_to_world(grid_position: &GridPosition, map_settings: &MapSettings) -> Vec3 {
    let half_tile = map_settings.tile_size as f32 / 2.0;
    let offset_x = -((map_settings.width * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let offset_y = -((map_settings.height * map_settings.tile_size) as f32) / 2.0 + half_tile;
    let world_pos_x = (grid_position.x * map_settings.tile_size) as f32 + offset_x;
    let world_pos_y = (grid_position.y * map_settings.tile_size) as f32 + offset_y;
    Vec3::new(world_pos_x, world_pos_y, 0.0)
}

fn spawn_army_unit(
    commands: &mut Commands,
    army: Army,
    grid_position: GridPosition,
    asset_server: &AssetServer,
    map_settings: &MapSettings,
) {
    let world_pos = grid_to_world(&grid_position, map_settings);

    commands.spawn((
        army,
        grid_position,
        Sprite {
            image: asset_server.load("army_texture.png"),
            custom_size: Some(Vec2 {
                x: map_settings.tile_size as f32,
                y: map_settings.tile_size as f32,
            }),
            ..Default::default()
        },
        Transform::from_translation(world_pos.with_z(70.0)),
        ArmySpriteTag {},
    ));
}

fn calculate_x_y_indicies(map_settings: &MapSettings, world_pos: Vec2) -> (f32, f32, i32, i32) {
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
    map_settings: &super::resources::MapSettings,
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
    map_settings: &super::resources::MapSettings,
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

pub fn army_position_sync_system(
    mut commands: Commands,
    mut army_query: Query<(Entity, &GridPosition, &mut Transform, &mut Army)>,
    map_settings: Res<MapSettings>,
) {
    let mut armies_by_pos: HashMap<GridPosition, Vec<(Entity, usize, i32)>> = HashMap::new();

    for (entity, grid_position, mut transform, army) in army_query.iter_mut() {
        let world_pos = grid_to_world(grid_position, &map_settings);
        transform.translation = world_pos.with_z(60.0);
        armies_by_pos.entry(*grid_position).or_default().push((
            entity,
            army.country_idx,
            army.number_of_units,
        ));
    }

    for (_, armies) in armies_by_pos {
        if armies.len() > 1 {
            resolve_armies_on_tile(&mut commands, &mut army_query, armies);
        }
    }
}

fn resolve_armies_on_tile(
    commands: &mut Commands,
    army_query: &mut Query<(Entity, &GridPosition, &mut Transform, &mut Army)>,
    armies: Vec<(Entity, usize, i32)>,
) {
    let mut armies_by_country: HashMap<usize, (Entity, i32)> = HashMap::new();
    for (entity, country_idx, units) in armies {
        if let Some((_existing_entity, existing_units)) = armies_by_country.get_mut(&country_idx) {
            *existing_units += units;
            commands.entity(entity).despawn();
        } else {
            armies_by_country.insert(country_idx, (entity, units));
        }
    }
    for (_, (entity, units)) in &armies_by_country {
        if let Ok((_, _, _, mut army)) = army_query.get_mut(*entity) {
            army.number_of_units = *units;
        }
    }
    if armies_by_country.len() > 1 {
        armies_by_country
            .values()
            .tuple_combinations()
            .for_each(|(army_entity1, army_entity2)| {
                let Ok(mut armies) = army_query.get_many_mut([army_entity1.0, army_entity2.0])
                else {
                    return;
                };
                let (army1, army2) = armies.split_at_mut(1);
                army_battle(
                    commands,
                    &mut army1[0].3,
                    &mut army2[0].3,
                    ArmyBattleMessage {
                        army_a_entity: army1[0].0,
                        army_b_entity: army2[0].0,
                    },
                );
            });
    }
}
