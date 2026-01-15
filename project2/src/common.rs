use bevy::prelude::*;

use crate::common::messages::{LoadGameMessage, NextTurnMessage, SaveGameMessage};

pub mod components;
pub mod messages;
pub mod systems;

pub struct CommonPlugin {}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NextTurnMessage>()
            .add_message::<SaveGameMessage>()
            .add_message::<LoadGameMessage>();
    }
}
