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
            .filter(|l| !l.is_empty())
            .collect::<Vec<String>>();

        Ok(CommandResult::CommandList(commands))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn read_from_command_success_test() {
        let file_name = "test_read.txt".to_string();
        let commands = vec![
            "CREATE Users KEY UserId FIELDS Name:STRING".to_string(),
            "INSERT Name=\"John\" INTO Users".to_string(),
        ];
        let content = commands.join("\n");
        fs::write(&file_name, content).unwrap();

        let command = ReadFromCommand {
            file_name: file_name.clone(),
        };

        let result = command.execute();
        assert!(result.is_ok());

        if let Ok(CommandResult::CommandList(read_commands)) = result {
            assert_eq!(read_commands.len(), 2);
            assert_eq!(
                read_commands[0],
                "CREATE Users KEY UserId FIELDS Name:STRING"
            );
            assert_eq!(read_commands[1], "INSERT Name=\"John\" INTO Users");
        } else {
            panic!("Expected CommandList");
        }

        fs::remove_file(&file_name).unwrap();
    }

    #[test]
    fn read_from_command_error_test() {
        let file_name = "/nonexistent/directory/test_read_error.txt".to_string();

        let command = ReadFromCommand { file_name };

        let result = command.execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::IoError(_) => (),
            _ => panic!("Expected IO error"),
        }
    }
}
