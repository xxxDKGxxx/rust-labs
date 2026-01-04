use anyhow::anyhow;
use bevy::{
    color::*,
    ecs::{
        message::MessageReader,
        query::{Changed, Has, With},
        system::*,
    },
    math::Vec2,
    platform::collections::HashSet,
    sprite::Sprite,
    transform::components::Transform,
};
use rand::{Rng, rng};

use crate::{
    country::{components::OwnershipTile, resources::*},
    map::{components::*, resources::MapSettings},
    ui::messages::NextTurnMessage,
};

pub fn setup_countries_system(mut countries: ResMut<Countries>) {
    let countries = countries.as_mut();

    const COUNTRY_NUM: u8 = 3;

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
    let mut countries_capitals_set: HashSet<(u64, u64)> = HashSet::new();

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
