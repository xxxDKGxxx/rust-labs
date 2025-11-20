use thiserror::Error;

use crate::database::table::record::RecordError;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("One or more record errors occured: {0}")]
    RecordError(#[from] RecordError),
}

pub trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
}
