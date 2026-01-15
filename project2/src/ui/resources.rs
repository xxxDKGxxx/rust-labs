use bevy::ecs::{entity::Entity, resource::Resource, world::FromWorld};
use bevy::image::Image;
use bevy::prelude::Handle;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default)]
pub struct MenuIcons {
    pub country_flags: Vec<Handle<Image>>,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct TurnCounter {
    pub count: u32,
}

impl FromWorld for TurnCounter {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        Self { count: 0 }
    }
}

#[derive(Resource, Default)]
pub struct GameLoadState {
    pub save_name: Option<String>,
}

#[derive(Resource)]
pub struct UiModel {
    pub selected_number_of_units: i32,
    pub army_entity_being_moved: Option<Entity>,
    pub save_popup_open: bool,
    pub save_file_name: String,
}

impl FromWorld for UiModel {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        Self {
            selected_number_of_units: 1,
            army_entity_being_moved: None,
            save_popup_open: false,
            save_file_name: "".into(),
        }
    }
}
