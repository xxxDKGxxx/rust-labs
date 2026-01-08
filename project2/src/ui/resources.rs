use bevy::ecs::{entity::Entity, resource::Resource, world::FromWorld};

#[derive(Resource)]
pub struct TurnCounter {
    pub count: u32,
}

impl FromWorld for TurnCounter {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        Self { count: 0 }
    }
}

#[derive(Resource)]
pub struct UiModel {
    pub selected_number_of_units: i32,
    pub army_entity_being_moved: Option<Entity>,
}

impl FromWorld for UiModel {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        Self {
            selected_number_of_units: 1,
            army_entity_being_moved: None,
        }
    }
}
