use std::fmt::Display;

use bevy::{
    color::Color,
    ecs::{resource::Resource, world::FromWorld},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Country {
    pub name: String,
    pub color: Color,
    pub money: i32,
}

impl Country {
    pub fn new(name: &str, color: Color) -> Self {
        Self {
            name: name.to_string(),
            color,
            money: 0,
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct Countries {
    pub countries: Vec<Country>,
}

impl Countries {
    fn new() -> Self {
        Self {
            countries: Vec::new(),
        }
    }
}

impl FromWorld for Countries {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RelationStatus {
    Neutral,
    AtWar,
}

impl RelationStatus {
    fn to_str(self) -> String {
        match self {
            RelationStatus::Neutral => "Neutral".into(),
            RelationStatus::AtWar => "At war".into(),
        }
    }
}

impl Display for RelationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[serde_as]
#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct Diplomacy {
    #[serde_as(as = "Vec<(_, _)>")]
    relatons: std::collections::HashMap<(usize, usize), RelationStatus>,
}

impl Diplomacy {
    fn handle_key(c1: usize, c2: usize) -> (usize, usize) {
        if c1 < c2 { (c1, c2) } else { (c2, c1) }
    }

    pub fn new() -> Self {
        Self {
            relatons: std::collections::HashMap::new(),
        }
    }

    pub fn get_relation(&self, country_a_idx: usize, country_b_idx: usize) -> RelationStatus {
        let key = Diplomacy::handle_key(country_a_idx, country_b_idx);
        let Some(relation) = self.relatons.get(&key) else {
            return RelationStatus::Neutral;
        };

        *relation
    }

    pub fn set_relation(
        &mut self,
        country_a_idx: usize,
        country_b_idx: usize,
        relation: RelationStatus,
    ) {
        let key = Diplomacy::handle_key(country_a_idx, country_b_idx);

        self.relatons.insert(key, relation);
    }
}
