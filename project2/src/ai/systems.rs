use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::{Rng, rng};

use crate::{
    common::messages::NextTurnMessage,
    country::{
        components::OwnershipTile,
        messages::ChangeRelationMessage,
        resources::{Countries, Country, Diplomacy, RelationStatus},
    },
    map::{
        components::{Army, Building, GridPosition, MapTile},
        messages::{BuildBuildingMessage, MoveArmyMessage, SpawnArmyMessage},
        resources::{ArmyMovements, MapSettings, TileMapGrid},
    },
    player::resources::PlayerData,
};

#[derive(SystemParam)]
pub struct AiSystemParams<'w, 's> {
    msgr: MessageReader<'w, 's, NextTurnMessage>,
    player_data: Res<'w, PlayerData>,
    countries: Res<'w, Countries>,
    diplomacy: Res<'w, Diplomacy>,
    map_settings: Res<'w, MapSettings>,
    tile_grid: Res<'w, TileMapGrid>,
    army_movements: ResMut<'w, ArmyMovements>,
    build_msg: MessageWriter<'w, BuildBuildingMessage>,
    spawn_msg: MessageWriter<'w, SpawnArmyMessage>,
    relation_msg: MessageWriter<'w, ChangeRelationMessage>,
    ownership_tiles: Query<'w, 's, (&'static OwnershipTile, &'static GridPosition)>,
    map_tiles: Query<'w, 's, Has<Building>, With<MapTile>>,
    armies: Query<'w, 's, (Entity, &'static Army, &'static GridPosition)>,
}

pub fn ai_system(mut params: AiSystemParams) {
    if params.msgr.read().count() == 0 {
        return;
    }

    let (ownership_map, country_owned_positions) = build_maps(&params.ownership_tiles);

    for (country_idx, country) in params.countries.countries.iter().enumerate() {
        if country_idx == params.player_data.country_idx {
            continue;
        }

        process_diplomacy(
            country_idx,
            &params.countries,
            &params.diplomacy,
            &mut params.relation_msg,
        );
        process_economy(
            country_idx,
            country,
            &params.map_settings,
            &country_owned_positions,
            &params.tile_grid,
            &params.map_tiles,
            &mut params.build_msg,
        );
        process_recruitment(
            country_idx,
            country,
            &params.map_settings,
            &country_owned_positions,
            &params.tile_grid,
            &mut params.spawn_msg,
        );
        process_army_movement(
            country_idx,
            &params.armies,
            &ownership_map,
            &params.diplomacy,
            &mut params.army_movements,
        );
    }
}

fn build_maps(
    ownership_tiles: &Query<(&OwnershipTile, &GridPosition)>,
) -> (HashMap<(i32, i32), usize>, HashMap<usize, Vec<(i32, i32)>>) {
    let mut ownership_map = HashMap::new();
    let mut country_owned_positions = HashMap::new();

    for (tile, pos) in ownership_tiles.iter() {
        if let Some(owner) = tile.country_id {
            ownership_map.insert((pos.x, pos.y), owner);
            country_owned_positions
                .entry(owner)
                .or_insert_with(Vec::new)
                .push((pos.x, pos.y));
        }
    }
    (ownership_map, country_owned_positions)
}

fn process_diplomacy(
    country_idx: usize,
    countries: &Countries,
    diplomacy: &Diplomacy,
    relation_msg: &mut MessageWriter<ChangeRelationMessage>,
) {
    let mut rng = rng();
    for other_idx in 0..countries.countries.len() {
        if country_idx == other_idx {
            continue;
        }

        match diplomacy.get_relation(country_idx, other_idx) {
            RelationStatus::AtWar => {
                if rng.random_bool(0.05) {
                    relation_msg.write(ChangeRelationMessage {
                        country_a_idx: country_idx,
                        country_b_idx: other_idx,
                        relation: RelationStatus::Neutral,
                    });
                }
            }
            RelationStatus::Neutral => {
                if rng.random_bool(0.01) {
                    relation_msg.write(ChangeRelationMessage {
                        country_a_idx: country_idx,
                        country_b_idx: other_idx,
                        relation: RelationStatus::AtWar,
                    });
                }
            }
        }
    }
}

fn process_economy(
    country_idx: usize,
    country: &Country,
    map_settings: &MapSettings,
    country_owned_positions: &HashMap<usize, Vec<(i32, i32)>>,
    tile_grid: &TileMapGrid,
    map_tiles: &Query<Has<Building>, With<MapTile>>,
    build_msg: &mut MessageWriter<BuildBuildingMessage>,
) {
    if country.money < map_settings.building_cost {
        return;
    }

    if let Some(positions) = country_owned_positions.get(&country_idx) {
        let candidates: Vec<Entity> = positions
            .iter()
            .filter_map(|&(x, y)| tile_grid.grid.get(&(x, y)).copied())
            .filter(|&e| map_tiles.get(e).map_or(false, |has| !has))
            .collect();

        if !candidates.is_empty() && rng().random_bool(0.5) {
            build_msg.write(BuildBuildingMessage {
                tile_entity: candidates[rng().random_range(0..candidates.len())],
                country_idx,
            });
        }
    }
}

fn process_recruitment(
    country_idx: usize,
    country: &Country,
    map_settings: &MapSettings,
    country_owned_positions: &HashMap<usize, Vec<(i32, i32)>>,
    tile_grid: &TileMapGrid,
    spawn_msg: &mut MessageWriter<SpawnArmyMessage>,
) {
    if country.money < map_settings.unit_cost * 5 {
        return;
    }
    let mut rng = rng();

    if let Some(positions) = country_owned_positions.get(&country_idx) {
        if positions.is_empty() {
            return;
        }
        let (x, y) = positions[rng.random_range(0..positions.len())];

        if let Some(&tile_entity) = tile_grid.grid.get(&(x, y)) {
            let amount = (country.money as f32 * 0.3 / map_settings.unit_cost as f32) as i32;
            if amount > 0 {
                spawn_msg.write(SpawnArmyMessage {
                    tile_entity,
                    country_idx,
                    amount,
                });
            }
        }
    }
}

fn process_army_movement(
    country_idx: usize,
    armies: &Query<(Entity, &Army, &GridPosition)>,
    ownership_map: &HashMap<(i32, i32), usize>,
    diplomacy: &Diplomacy,
    army_movements: &mut ResMut<ArmyMovements>,
) {
    let mut rng = rng();
    for (entity, army, pos) in armies.iter() {
        if army.country_idx != country_idx {
            continue;
        }

        let neighbors = [
            (pos.x + 1, pos.y),
            (pos.x - 1, pos.y),
            (pos.x, pos.y + 1),
            (pos.x, pos.y - 1),
        ];
        let valid_moves: Vec<GridPosition> = neighbors
            .iter()
            .filter(|&&(nx, ny)| is_valid_move(nx, ny, country_idx, ownership_map, diplomacy))
            .map(|&(nx, ny)| GridPosition::new(nx, ny))
            .collect();

        if !valid_moves.is_empty() {
            let target = select_target(&valid_moves, country_idx, ownership_map, &mut rng);
            army_movements.add_movement(MoveArmyMessage {
                moved_army_entity: entity,
                target_position: target,
                number_of_units_to_move: army.number_of_units,
            });
        }
    }
}

fn is_valid_move(
    nx: i32,
    ny: i32,
    country_idx: usize,
    map: &HashMap<(i32, i32), usize>,
    dip: &Diplomacy,
) -> bool {
    map.get(&(nx, ny)).map_or(false, |&owner| {
        owner == country_idx
            || matches!(dip.get_relation(country_idx, owner), RelationStatus::AtWar)
    })
}

fn select_target(
    moves: &[GridPosition],
    idx: usize,
    map: &HashMap<(i32, i32), usize>,
    rng: &mut impl Rng,
) -> GridPosition {
    let enemies: Vec<&GridPosition> = moves
        .iter()
        .filter(|p| map.get(&(p.x, p.y)).map_or(false, |&o| o != idx))
        .collect();

    if !enemies.is_empty() {
        enemies[rng.random_range(0..enemies.len())].clone()
    } else {
        moves[rng.random_range(0..moves.len())].clone()
    }
}
