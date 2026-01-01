use bevy::ecs::resource::Resource;

#[derive(Resource)]
pub struct PlayerData {
    pub country_idx: usize,
}

impl PlayerData {
    pub fn new(country_idx: usize) -> Self {
        Self { country_idx }
    }
}
