use bevy::{
    color::Color,
    ecs::{resource::Resource, world::FromWorld},
};

#[derive(Default)]
pub struct Country {
    pub name: String,
    pub color: Color,
    pub money: i32,
}

impl Country {
    pub fn new(name: String, color: Color) -> Self {
        Self {
            name,
            color,
            money: 0,
        }
    }
}

#[derive(Resource)]
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
