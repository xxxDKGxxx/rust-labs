use thiserror::Error;

use crate::database::{
    DatabaseError,
    key::DatabaseKey,
    table::{TableError, record::RecordError},
};

#[derive(Error, Debug, PartialEq)]
pub enum CommandError<K: DatabaseKey> {
    #[error("One or more record errors occured: {0}")]
    RecordError(#[from] RecordError),

    #[error("One or more database errors occured: {0}")]
    DatabaseError(#[from] DatabaseError<K>),

    #[error("One or more table errors occured: {0}")]
    TableError(#[from] TableError<K>),

    #[error("Unknown operator: {0}")]
    UnknownOperatorError(String),

    #[error("Invalid value for {column_name}, expected {expected_type}, got {got_type}")]
    InvalidValueError {
        column_name: String,
        expected_type: String,
        got_type: String,
    },
}

pub trait Command<K: DatabaseKey> {
    fn execute(&mut self) -> Result<(), CommandError<K>>;
}
