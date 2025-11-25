use std::{fs::File, io::Read};

use crate::{
    commands::command::{AnyCommand, Command, CommandError, CommandResult},
    database::key::DatabaseKey,
};

pub struct ReadFromCommand {
    pub file_name: String,
}

impl<K: DatabaseKey> From<ReadFromCommand> for AnyCommand<'_, K> {
    fn from(command: ReadFromCommand) -> Self {
        AnyCommand::ReadFromCommand(command)
    }
}

impl Command for ReadFromCommand {
    fn execute(self) -> Result<CommandResult, CommandError> {
        let mut file =
            File::open(&self.file_name).map_err(|e| CommandError::IoError(e.to_string()))?;

        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .map_err(|e| CommandError::IoError(e.to_string()))?;

        let commands = contents
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<String>>();

        Ok(CommandResult::CommandList(commands))
    }
}
