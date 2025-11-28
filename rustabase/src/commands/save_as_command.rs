use std::{fs::File, io::Write};

use crate::{
    commands::command::{AnyCommand, Command, CommandError, CommandResult},
    database::key::DatabaseKey,
};

pub struct SaveAsCommand<'a> {
    pub file_name: String,
    pub lines: &'a Vec<String>,
}

impl<'a, K: DatabaseKey> From<SaveAsCommand<'a>> for AnyCommand<'a, K> {
    fn from(value: SaveAsCommand<'a>) -> Self {
        Self::SaveAsCommand(value)
    }
}

impl Command for SaveAsCommand<'_> {
    fn execute(self) -> Result<CommandResult, CommandError> {
        let mut file = match File::create(&self.file_name) {
            Ok(file) => file,
            Err(e) => return Err(CommandError::IoError(e.to_string())),
        };

        for line in self.lines {
            match writeln!(file, "{line}") {
                Ok(()) => (),
                Err(e) => return Err(CommandError::IoError(e.to_string())),
            }
        }

        Ok(CommandResult::Void)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn save_as_command_success_test() {
        let mut commands = vec![
            "CREATE Users KEY UserId FIELDS Firstname:STRING, Lastname:STRING, Age:INT, Married:BOOL".to_string(),
            "INSERT UserId=1, Firstname=\"John\", Lastname=\"Doe\", Age=16, Married=false INTO Users".to_string(),
        ];

        let file_name = "test_save_i64.txt".to_string();
        let command = SaveAsCommand {
            file_name: file_name.clone(),
            lines: &mut commands,
        };

        let result = command.execute();
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_name).unwrap();
        assert!(contents.contains("CREATE Users KEY UserId"));
        assert!(contents.contains("INSERT UserId=1"));

        fs::remove_file(&file_name).unwrap();
    }

    #[test]
    fn save_as_command_failure_test() {
        let mut commands = vec!["Some command".to_string()];

        let file_name = "/nonexistent/directory/test_failure.txt".to_string();
        let command = SaveAsCommand {
            file_name,
            lines: &mut commands,
        };

        let result = command.execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::IoError(_) => (),
            _ => panic!("Expected IO error"),
        }
    }
}
