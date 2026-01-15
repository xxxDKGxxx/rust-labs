use bevy::prelude::*;

use crate::{
    common::{messages::SaveGameMessage, systems::SAVE_PATH},
    player::resources::PlayerData,
};

const SAVE_FILE_NAME: &str = "save_player.json";

pub fn save_player_system(
    mut save_game_message_reader: MessageReader<SaveGameMessage>,
    player_data: Res<PlayerData>,
) -> anyhow::Result<()> {
    for save_game_message in save_game_message_reader.read() {
        let save_json = serde_json::to_string_pretty(&(*player_data))?;
        std::fs::create_dir_all(format!("{}/{}", SAVE_PATH, save_game_message.save_name))?;
        std::fs::write(
            format!(
                "{}/{}/{}",
                SAVE_PATH, save_game_message.save_name, SAVE_FILE_NAME
            ),
            save_json,
        )?;
    }

    Ok(())
}
