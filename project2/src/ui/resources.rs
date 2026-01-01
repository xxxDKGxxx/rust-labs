use bevy::ecs::{resource::Resource, world::FromWorld};

#[derive(Resource)]
pub struct TurnCounter {
    pub count: u32,
}

impl FromWorld for TurnCounter {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        Self { count: 0 }
    }
}
