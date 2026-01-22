use std::collections::HashMap;

use anyhow::{Result, anyhow};
use bevy::{ecs::system::SystemParam, prelude::*};
use rand::{Rng, rng};

use crate::{
    InGameStates,
    ai::resources::AiProcessing,
    common::{components::GridPosition, messages::NextTurnMessage},
    country::{
        components::OwnershipTile,
        messages::{
            AcceptPeaceMessage, ChangeRelationMessage, ProposePeaceMessage, RejectPeaceMessage,
        },
        resources::{Countries, Country, Diplomacy, PeaceOffers, RelationStatus},
    },
    map::{
        components::{Army, Building, MapTile},
        messages::{BuildBuildingMessage, MoveArmyMessage, SpawnArmyMessage},
        resources::{ArmyMovements, MapSettings, TileMapGrid},
    },
    player::resources::PlayerData,
};

#[derive(Message)]
pub struct AiTurnMessage {}

#[derive(SystemParam)]
pub struct AiSystemParams<'w, 's> {
    msgr: MessageWriter<'w, NextTurnMessage>,
    ai_msgr: MessageReader<'w, 's, AiTurnMessage>,
    player_data: Res<'w, PlayerData>,
    countries: Res<'w, Countries>,
    diplomacy: Res<'w, Diplomacy>,
    map_settings: Res<'w, MapSettings>,
    tile_grid: Res<'w, TileMapGrid>,
    ai_processing: ResMut<'w, AiProcessing>,
    army_movements: ResMut<'w, ArmyMovements>,
    next_state: ResMut<'w, NextState<InGameStates>>,
    build_msg: MessageWriter<'w, BuildBuildingMessage>,
    spawn_msg: MessageWriter<'w, SpawnArmyMessage>,
    relation_msg: MessageWriter<'w, ChangeRelationMessage>,
    propose_peace_msg: MessageWriter<'w, ProposePeaceMessage>,
    accept_peace_msg: MessageWriter<'w, AcceptPeaceMessage>,
    reject_peace_msg: MessageWriter<'w, RejectPeaceMessage>,
    peace_offers: Res<'w, PeaceOffers>,
    ownership_tiles: Query<'w, 's, (&'static OwnershipTile, &'static GridPosition)>,
    map_tiles: Query<'w, 's, Has<Building>, With<MapTile>>,
    armies: Query<'w, 's, (Entity, &'static Army, &'static GridPosition)>,
}

pub fn ai_system(mut params: AiSystemParams) -> Result<()> {
    let (ownership_map, country_owned_positions) = build_maps(&params.ownership_tiles);
    let country_strengths =
        calculate_country_strengths(&params.countries, &country_owned_positions, &params.armies);
    ai_evaluate_peace_offers(
        &params.player_data,
        &params.peace_offers,
        &params.diplomacy,
        &country_strengths,
        &mut params.accept_peace_msg,
        &mut params.reject_peace_msg,
    );
    let current_country_idx = params.ai_processing.country_idx;
    if current_country_idx >= params.countries.countries.len() {
        params.next_state.set(InGameStates::Idle);
        params.ai_processing.country_idx = 0;
        params.msgr.write(NextTurnMessage {});
        println!("Ai system is finished");
        return Ok(());
    }
    let current_country = &params.countries.countries[current_country_idx];
    if current_country_idx == params.player_data.country_idx {
        params.ai_processing.country_idx += 1;
        return Ok(());
    }
    process_diplomacy(
        current_country_idx,
        &params.countries,
        &params.diplomacy,
        &country_strengths,
        &params.armies,
        &mut params.relation_msg,
        &mut params.propose_peace_msg,
    )?;
    process_economy(
        (current_country, current_country_idx),
        &params.map_settings,
        &country_owned_positions,
        &params.tile_grid,
        &params.map_tiles,
        &mut params.build_msg,
        &ownership_map,
    )?;
    process_recruitment(
        (current_country, current_country_idx),
        &params.map_settings,
        &country_owned_positions,
        &params.tile_grid,
        &mut params.spawn_msg,
        &ownership_map,
        &params.armies,
    )?;
    process_army_movement(
        current_country_idx,
        &params.armies,
        &ownership_map,
        &params.diplomacy,
        &mut params.army_movements,
    )?;
    params.ai_processing.country_idx += 1;
    Ok(())
}

type OwnershipMap = HashMap<(i32, i32), usize>;
type CountryOwnedPositionsMap = HashMap<usize, Vec<(i32, i32)>>;

fn build_maps(
    ownership_tiles: &Query<(&OwnershipTile, &GridPosition)>,
) -> (OwnershipMap, CountryOwnedPositionsMap) {
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

fn calculate_country_strengths(
    countries: &Countries,
    country_owned_positions: &CountryOwnedPositionsMap,
    armies: &Query<(Entity, &Army, &GridPosition)>,
) -> HashMap<usize, i32> {
    let mut country_strengths = HashMap::new();
    for (country_idx, _country) in countries.countries.iter().enumerate() {
        let owned_tiles_count = country_owned_positions
            .get(&country_idx)
            .map_or(0, |p| p.len());
        let army_strength: i32 = armies
            .iter()
            .filter(|(_, army, _)| army.country_idx == country_idx)
            .map(|(_, army, _)| army.number_of_units)
            .sum();
        let strength = owned_tiles_count as i32 + army_strength * 3;
        country_strengths.insert(country_idx, strength);
    }
    country_strengths
}

fn process_diplomacy(
    country_idx: usize,
    countries: &Countries,
    diplomacy: &Diplomacy,
    country_strengths: &HashMap<usize, i32>,
    armies: &Query<(Entity, &Army, &GridPosition)>,
    relation_msg: &mut MessageWriter<ChangeRelationMessage>,
    propose_peace_msg: &mut MessageWriter<ProposePeaceMessage>,
) -> Result<()> {
    for other_idx in 0..countries.countries.len() {
        if country_idx == other_idx {
            continue;
        }

        match diplomacy.get_relation(country_idx, other_idx) {
            RelationStatus::AtWar => {
                let my_army_strength: i32 = armies
                    .iter()
                    .filter(|(_, army, _)| army.country_idx == country_idx)
                    .map(|(_, army, _)| army.number_of_units)
                    .sum();
                let other_army_strength: i32 = armies
                    .iter()
                    .filter(|(_, army, _)| army.country_idx == other_idx)
                    .map(|(_, army, _)| army.number_of_units)
                    .sum();

                if my_army_strength > 0
                    && my_army_strength < other_army_strength / 2
                    && rng().random_bool(0.3)
                {
                    propose_peace_msg.write(ProposePeaceMessage {
                        from: country_idx,
                        to: other_idx,
                    });
                }
            }
            RelationStatus::Neutral => {
                let my_strength = country_strengths
                    .get(&country_idx)
                    .ok_or_else(|| anyhow!("Failed to get my strength"))?;
                let other_strength = country_strengths
                    .get(&other_idx)
                    .ok_or_else(|| anyhow!("Failed to get other strength"))?;

                if my_strength > other_strength && rng().random_bool(0.1) {
                    relation_msg.write(ChangeRelationMessage {
                        country_a_idx: country_idx,
                        country_b_idx: other_idx,
                        relation: RelationStatus::AtWar,
                    });
                }
            }
        }
    }
    Ok(())
}

fn process_economy(
    country_with_idx: (&Country, usize),
    map_settings: &MapSettings,
    country_owned_positions: &CountryOwnedPositionsMap,
    tile_grid: &TileMapGrid,
    map_tiles: &Query<Has<Building>, With<MapTile>>,
    build_msg: &mut MessageWriter<BuildBuildingMessage>,
    ownership_map: &OwnershipMap,
) -> Result<()> {
    let (country, country_idx) = country_with_idx;
    if country.money < map_settings.building_cost {
        return Ok(());
    }

    if let Some(positions) = country_owned_positions.get(&country_idx) {
        let candidates: Vec<Entity> = positions
            .iter()
            .filter(|&pos| !is_border_tile(pos, country_idx, ownership_map))
            .filter_map(|&(x, y)| tile_grid.grid.get(&(x, y)).copied())
            .filter(|&e| map_tiles.get(e).is_ok_and(|has| !has))
            .collect();

        if !candidates.is_empty() && rng().random_bool(0.5) {
            let tile_entity = candidates[rng().random_range(0..candidates.len())];
            build_msg.write(BuildBuildingMessage {
                tile_entity,
                country_idx,
            });
        }
    }
    Ok(())
}

fn choose_spawn_positions<'a>(
    country_idx: usize,
    positions: &'a [(i32, i32)],
    ownership_map: &OwnershipMap,
    armies: &Query<(Entity, &Army, &GridPosition)>,
) -> Vec<&'a (i32, i32)> {
    let border_positions: Vec<&(i32, i32)> = positions
        .iter()
        .filter(|&pos| is_border_tile(pos, country_idx, ownership_map))
        .collect();
    let border_armies_count = armies
        .iter()
        .filter(|(_, army, pos)| {
            army.country_idx == country_idx
                && is_border_tile(&(pos.x, pos.y), country_idx, ownership_map)
        })
        .count();
    let border_army_threshold = (border_positions.len() / 5).max(1);
    let mut potential_spawn_positions: Vec<&(i32, i32)> =
        if border_armies_count < border_army_threshold {
            border_positions
        } else {
            positions
                .iter()
                .filter(|p| !is_border_tile(p, country_idx, ownership_map))
                .collect()
        };
    if potential_spawn_positions.is_empty() {
        potential_spawn_positions = positions.iter().collect();
    }
    potential_spawn_positions
}

fn process_recruitment(
    country_with_idx: (&Country, usize),
    map_settings: &MapSettings,
    country_owned_positions: &CountryOwnedPositionsMap,
    tile_grid: &TileMapGrid,
    spawn_msg: &mut MessageWriter<SpawnArmyMessage>,
    ownership_map: &OwnershipMap,
    armies: &Query<(Entity, &Army, &GridPosition)>,
) -> Result<()> {
    let (country, country_idx) = country_with_idx;
    if country.money < map_settings.unit_cost * 5 {
        return Ok(());
    }
    if let Some(positions) = country_owned_positions.get(&country_idx) {
        if positions.is_empty() {
            return Ok(());
        }
        let potential_spawn_positions =
            choose_spawn_positions(country_idx, positions, ownership_map, armies);
        if potential_spawn_positions.is_empty() {
            return Ok(());
        }
        let spawn_pos =
            potential_spawn_positions[rng().random_range(0..potential_spawn_positions.len())];
        let tile_entity = tile_grid
            .grid
            .get(spawn_pos)
            .ok_or_else(|| anyhow!("Invalid spawn position selected"))?;
        let amount = (country.money as f32 * 0.3 / map_settings.unit_cost as f32) as i32;
        if amount > 0 {
            spawn_msg.write(SpawnArmyMessage {
                tile_entity: *tile_entity,
                country_idx,
                amount,
            });
        }
    }
    Ok(())
}

fn process_army_movement(
    country_idx: usize,
    armies: &Query<(Entity, &Army, &GridPosition)>,
    ownership_map: &OwnershipMap,
    diplomacy: &Diplomacy,
    army_movements: &mut ResMut<ArmyMovements>,
) -> Result<()> {
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

        if let Ok(target) = select_target(&valid_moves, army, ownership_map, armies)
            && target != *pos
        {
            army_movements.add_movement(MoveArmyMessage {
                moved_army_entity: entity,
                target_position: target,
                number_of_units_to_move: army.number_of_units,
            });
        }
    }
    Ok(())
}

fn is_valid_move(
    nx: i32,
    ny: i32,
    country_idx: usize,
    ownership_map: &OwnershipMap,
    dip: &Diplomacy,
) -> bool {
    match ownership_map.get(&(nx, ny)) {
        Some(&owner) => {
            owner == country_idx
                || matches!(dip.get_relation(country_idx, owner), RelationStatus::AtWar)
        }
        None => false, // Unowned tiles are water and not walkable
    }
}

fn calculate_move_score(
    target_pos: &GridPosition,
    army: &Army,
    ownership_map: &OwnershipMap,
    armies: &Query<(Entity, &Army, &GridPosition)>,
) -> i32 {
    let mut score = 0;
    let target_key = &(target_pos.x, target_pos.y);
    let enemy_army_on_tile = armies.iter().find(|(_, other_army, other_pos)| {
        other_pos.x == target_pos.x
            && other_pos.y == target_pos.y
            && other_army.country_idx != army.country_idx
    });
    if let Some((_, enemy_army, _)) = enemy_army_on_tile {
        if army.number_of_units > enemy_army.number_of_units {
            score += 1000;
        } else if army.number_of_units > enemy_army.number_of_units / 2 {
            score += 200;
        } else {
            score -= 1000;
        }
    }
    if let Some(&owner) = ownership_map.get(target_key) {
        if owner != army.country_idx {
            score += 500;
        } else {
            score += 1;
        }
    }
    score += count_friendly_neighbors(target_pos, army.country_idx, ownership_map) * 10;
    score += rng().random_range(0..5);
    score
}

fn select_target(
    moves: &[GridPosition],
    army: &Army,
    ownership_map: &OwnershipMap,
    armies: &Query<(Entity, &Army, &GridPosition)>,
) -> Result<GridPosition> {
    if moves.is_empty() {
        return Err(anyhow!("No valid moves for army"));
    }

    moves
        .iter()
        .max_by_key(|&target_pos| calculate_move_score(target_pos, army, ownership_map, armies))
        .copied()
        .ok_or_else(|| anyhow!("Could not determine best move"))
}

fn is_border_tile(pos: &(i32, i32), owner_idx: usize, ownership_map: &OwnershipMap) -> bool {
    let neighbors = [
        (pos.0 + 1, pos.1),
        (pos.0 - 1, pos.1),
        (pos.0, pos.1 + 1),
        (pos.0, pos.1 - 1),
    ];
    for neighbor in &neighbors {
        match ownership_map.get(neighbor) {
            Some(&neighbor_owner) if neighbor_owner != owner_idx => return true,
            None => return true, // Adjacent to water is also a border
            _ => (),
        }
    }
    false
}

fn count_friendly_neighbors(
    pos: &GridPosition,
    owner_idx: usize,
    ownership_map: &OwnershipMap,
) -> i32 {
    let neighbors = [
        (pos.x + 1, pos.y),
        (pos.x - 1, pos.y),
        (pos.x, pos.y + 1),
        (pos.x, pos.y - 1),
    ];
    let mut count = 0;
    for neighbor in &neighbors {
        if let Some(&neighbor_owner) = ownership_map.get(neighbor)
            && neighbor_owner == owner_idx
        {
            count += 1;
        }
    }
    count
}

fn ai_evaluate_peace_offers(
    player_data: &Res<PlayerData>,
    peace_offers: &Res<PeaceOffers>,
    diplomacy: &Res<Diplomacy>,
    country_strengths: &HashMap<usize, i32>,
    accept_peace_msg: &mut MessageWriter<AcceptPeaceMessage>,
    reject_peace_msg: &mut MessageWriter<RejectPeaceMessage>,
) {
    let mut rng = rng();

    for offer in peace_offers.offers.iter() {
        if offer.to == player_data.country_idx {
            continue;
        }

        if diplomacy.get_relation(offer.to, offer.from) != RelationStatus::AtWar {
            continue;
        }

        let my_strength = country_strengths.get(&offer.to).copied().unwrap_or(0);
        let proposer_strength = country_strengths.get(&offer.from).copied().unwrap_or(0);

        let should_accept = if my_strength < proposer_strength / 2 {
            rng.random_bool(0.7)
        } else {
            rng.random_bool(0.2)
        };

        if should_accept {
            accept_peace_msg.write(AcceptPeaceMessage {
                from: offer.from,
                to: offer.to,
            });
        } else {
            reject_peace_msg.write(RejectPeaceMessage {
                from: offer.from,
                to: offer.to,
            });
        }
    }
}
