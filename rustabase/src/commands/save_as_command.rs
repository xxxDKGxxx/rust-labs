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

impl<'a> Command for SaveAsCommand<'a> {
    fn execute(self) -> Result<CommandResult, CommandError> {
        let mut file = match File::create(&self.file_name) {
            Ok(file) => file,
            Err(e) => return Err(CommandError::IoError(e.to_string())),
        };

        for line in self.lines.iter() {
            match file.write_all(line.as_bytes()) {
                Ok(_) => (),
                Err(e) => return Err(CommandError::IoError(e.to_string())),
            }
        }

        Ok(CommandResult::Void)
    }
}
