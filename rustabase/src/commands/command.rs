use thiserror::Error;

use crate::{
    commands::{
        create_command::CreateCommand, delete_command::DeleteCommand,
        insert_command::InsertCommand, read_from_command::ReadFromCommand,
        save_as_command::SaveAsCommand, select_command::SelectCommand,
    },
    database::{
        DatabaseError,
        key::DatabaseKey,
        table::{
            TableError,
            record::{RecordError, Value},
        },
    },
};

#[derive(Error, Debug, PartialEq)]
pub enum CommandError {
    #[error("IO error occurred: {0}")]
    IoError(String),

    #[error("One or more record errors occurred: {0}")]
    RecordError(#[from] RecordError),

    #[error("One or more database errors occurred: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("One or more table errors occurred: {0}")]
    TableError(#[from] TableError),

    #[error("Unknown operator: {0}")]
    UnknownOperatorError(String),

    #[error("Invalid value for {column_name}, expected {expected_type}, got {got_type}")]
    InvalidValueError {
        column_name: String,
        expected_type: String,
        got_type: String,
    },
}

#[derive(Debug)]
pub enum CommandResult {
    Void,
    RecordValueList(Vec<String>, Vec<Vec<Value>>),
    CommandList(Vec<String>),
}

pub enum AnyCommand<'a, K: DatabaseKey> {
    CreateCommand(CreateCommand<'a, K>),
    DeleteCommand(DeleteCommand<'a, K>),
    InsertCommand(InsertCommand<'a, K>),
    SelectCommand(SelectCommand<'a, K>),
    SaveAsCommand(SaveAsCommand<'a>),
    ReadFromCommand(ReadFromCommand),
}

pub trait Command {
    fn execute(self) -> Result<CommandResult, CommandError>;
}

impl<K: DatabaseKey> Command for AnyCommand<'_, K> {
    fn execute(self) -> Result<CommandResult, CommandError> {
        match self {
            AnyCommand::CreateCommand(create_command) => create_command.execute(),
            AnyCommand::DeleteCommand(delete_command) => delete_command.execute(),
            AnyCommand::InsertCommand(insert_command) => insert_command.execute(),
            AnyCommand::SelectCommand(select_command) => select_command.execute(),
            AnyCommand::SaveAsCommand(save_as_command) => save_as_command.execute(),
            AnyCommand::ReadFromCommand(read_from_command) => read_from_command.execute(),
        }
    }
}
