use bevy::prelude::*;

use crate::{
    common::{
        messages::SaveGameMessage,
        systems::{get_save_path, SAVE_PATH},
    },
    player::resources::PlayerData,
    ui::resources::GameLoadState,
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

pub fn load_player_system(
    mut commands: Commands,
    load_state: Res<GameLoadState>,
) -> anyhow::Result<()> {
    if let Some(save_name) = &load_state.save_name {
        let path = format!("{}/{}", get_save_path(save_name), SAVE_FILE_NAME);
        let data = std::fs::read_to_string(path)?;
        let player_data: PlayerData = serde_json::from_str(&data)?;
        commands.insert_resource(player_data);
    }
    Ok(())
}
