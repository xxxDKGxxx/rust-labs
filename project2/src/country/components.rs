use bevy::ecs::component::Component;

#[derive(Component)]
pub struct OwnershipTile {
    pub country_id: Option<usize>,
}

impl OwnershipTile {
    pub fn new(country_id: Option<usize>) -> Self {
        Self { country_id }
    }
}
