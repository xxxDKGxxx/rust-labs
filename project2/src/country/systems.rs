use anyhow::anyhow;
use bevy::{
    asset::AssetServer,
    color::*,
    ecs::{
        entity::Entity,
        message::MessageReader,
        query::{Changed, Has, With, Without},
        system::*,
    },
    math::Vec2,
    platform::collections::HashSet,
    sprite::Sprite,
    transform::components::Transform,
};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        components::GridPosition,
        messages::{NextTurnMessage, SaveGameMessage},
        systems::SAVE_PATH,
    },
    country::{
        components::{CountryFlag, OwnershipTile},
        messages::ChangeRelationMessage,
        resources::*,
    },
    map::{components::*, resources::MapSettings},
};

pub fn setup_countries_system(mut countries: ResMut<Countries>) {
    let countries = countries.as_mut();

    const COUNTRY_NUM: u8 = 5;

    for i in 0..COUNTRY_NUM {
        countries.countries.push(Country::new(
            format!("C{i}"),
            Color::Hsva(Hsva::hsv(360.0 / COUNTRY_NUM as f32 * i as f32, 1.0, 1.0)),
        ));
    }
}

pub fn setup_ownership_tiles(
    mut commands: Commands,
    countries: Res<Countries>,
    tiles_query: Query<(&MapTile, &GridPosition, &Transform)>,
    map_settings: Res<MapSettings>,
) {
    let mut countries_capitals_set: HashSet<(i32, i32)> = HashSet::new();

    let tile_poses_without_water: Vec<_> = tiles_query
        .iter()
        .filter(|(tile, _, _)| tile.tile_type != MapTileType::Water)
        .map(|(_, pos, _)| pos)
        .collect();

    let mut rng = rng();

    if tile_poses_without_water.len() < countries.countries.len() {
        return;
    }

    for _ in 0..countries.countries.len() {
        let capital_pos_idx = rng.random_range(0..tile_poses_without_water.len());

        if let Some(capital_pos) = tile_poses_without_water.get(capital_pos_idx) {
            countries_capitals_set.insert((capital_pos.x, capital_pos.y));
        }
    }

    for (tile, pos, transform) in tiles_query {
        if tile.tile_type == MapTileType::Water {
            commands.spawn((
                Sprite {
                    ..Default::default()
                },
                Transform::from_xyz(transform.translation.x, transform.translation.y, 50.0),
                OwnershipTile::new(None),
                GridPosition::new(pos.x, pos.y),
            ));
            continue;
        }

        if let Some(closest_country_idx) = countries_capitals_set
            .iter()
            .enumerate()
            .min_by_key(|(_, (x, y))| {
                (*x as i64 - pos.x as i64).pow(2) + (*y as i64 - pos.y as i64).pow(2)
            })
            .map(|(idx, _)| idx)
        {
            commands.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(
                        map_settings.tile_size as f32,
                        map_settings.tile_size as f32,
                    )),
                    ..Default::default()
                },
                Transform::from_xyz(transform.translation.x, transform.translation.y, 0.0),
                OwnershipTile::new(Some(closest_country_idx)),
                GridPosition::new(pos.x, pos.y),
            ));
        }
    }
}

pub fn update_ownership_tiles(
    ownership_tiles_query: Query<(&mut Sprite, &OwnershipTile), Changed<OwnershipTile>>,
    countries: Res<Countries>,
) {
    for (mut sprite, tile) in ownership_tiles_query {
        match tile.country_id {
            Some(id) => {
                let country = &countries.countries[id];
                sprite.color = country.color;
                sprite.color.set_alpha(1.0);
            }
            None => sprite.color = Color::NONE,
        }
    }
}

pub fn money_gathering_system(
    mut msgr: MessageReader<NextTurnMessage>,
    mut countries_resource: ResMut<Countries>,
    map_tiles: Query<(&GridPosition, Has<Building>), With<MapTile>>,
    ownership_tiles: Query<(&OwnershipTile, &GridPosition)>,
) -> anyhow::Result<()> {
    for _ in msgr.read() {
        for (ownership_tile, ownership_tile_grid_pos) in ownership_tiles {
            if let Some(country_id) = ownership_tile.country_id {
                countries_resource.countries[country_id].money += 1;
                let map_tile_at_country_pos = map_tiles
                    .iter()
                    .find(|(pos, _)| *pos == ownership_tile_grid_pos);

                let (_, has_building) = match map_tile_at_country_pos {
                    Some(t) => t,
                    None => return Err(anyhow!("Found ownership tile without Map Tile")),
                };

                if has_building {
                    countries_resource.countries[country_id].money += 100;
                }
            }
        }
    }
    Ok(())
}

pub fn relation_managing_system(
    mut change_relation_message_reader: MessageReader<ChangeRelationMessage>,
    mut diplomacy_resource: ResMut<Diplomacy>,
) {
    for change_relation_message in change_relation_message_reader.read() {
        diplomacy_resource.set_relation(
            change_relation_message.country_a_idx,
            change_relation_message.country_b_idx,
            change_relation_message.relation,
        );
    }
}

pub fn setup_country_flags_system(
    mut commands: Commands,
    countries_resource: Res<Countries>,
    asset_server: Res<AssetServer>,
    map_settings: Res<MapSettings>,
) {
    for idx in 0..countries_resource.countries.len() {
        commands.spawn((
            Sprite {
                image: asset_server.load(format!("countries/{idx}.png")),
                custom_size: Some(Vec2::new(
                    (map_settings.tile_size * 3) as f32,
                    (map_settings.tile_size * 2) as f32,
                )),
                color: Color::WHITE.with_alpha(0.5),
                ..Default::default()
            },
            CountryFlag { idx },
        ));
    }
}

pub fn update_country_flag_system(
    mut commands: Commands,
    changed_ownership_tiles_query: Query<Entity, Changed<OwnershipTile>>,
    ownership_tiles_query: Query<(&OwnershipTile, &Transform)>,
    country_flags: Query<(Entity, &CountryFlag, &mut Transform), Without<OwnershipTile>>,
) {
    if changed_ownership_tiles_query.is_empty() {
        return;
    }

    for (entity, country_flag, mut transform) in country_flags {
        let owned_tiles = ownership_tiles_query
            .iter()
            .filter(|(o, _)| o.country_id == Some(country_flag.idx))
            .collect::<Vec<_>>();

        if owned_tiles.is_empty() {
            println!("Despawning");
            commands.entity(entity).despawn();
            continue;
        }

        let (x_sum, y_sum) = owned_tiles
            .iter()
            .fold((0.0, 0.0), |(x, y), (_, transform)| {
                (x + transform.translation.x, y + transform.translation.y)
            });

        transform.translation.x = x_sum / owned_tiles.len() as f32;
        transform.translation.y = y_sum / owned_tiles.len() as f32;
        transform.translation.z = 51.0;
    }
}

#[derive(Serialize, Deserialize)]
struct CountriesSaveState {
    ownership_tiles: Vec<(OwnershipTile, GridPosition)>,
    countries: Countries,
    diplomacy: Diplomacy,
}

const SAVE_FILE_NAME: &str = "save_country.json";

pub fn save_countries_system(
    mut save_game_message_reader: MessageReader<SaveGameMessage>,
    ownership_tiles_query: Query<(&OwnershipTile, &GridPosition)>,
    countries_resource: Res<Countries>,
    diplomacy_resource: Res<Diplomacy>,
) -> anyhow::Result<()> {
    for save_game_message in save_game_message_reader.read() {
        let mut ownership_tiles_vec: Vec<(OwnershipTile, GridPosition)> = Vec::new();
        for (ownership_tile, grid_position) in ownership_tiles_query {
            ownership_tiles_vec.push((*ownership_tile, *grid_position));
        }

        let state = CountriesSaveState {
            ownership_tiles: ownership_tiles_vec,
            countries: (*countries_resource).clone(),
            diplomacy: (*diplomacy_resource).clone(),
        };

        let save_json = serde_json::to_string_pretty(&state)?;
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
