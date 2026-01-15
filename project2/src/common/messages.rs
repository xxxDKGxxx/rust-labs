use bevy::ecs::message::Message;

#[derive(Message)]
pub struct NextTurnMessage {}

#[derive(Message)]
pub struct SaveGameMessage {
    pub save_name: String,
}

#[derive(Message)]
pub struct LoadGameMessage {
    save_name: String,
}
