use thiserror::Error;

use crate::resp::types::RespType;

#[derive(Error, Debug)]
pub enum CommandExecutionError {
    #[error("unsupported options was provided: {0}")]
    IncorrectOptionsError(String),
    #[error("unable to parse a command from the given resp type, supported types: arrays")]
    UnsupportedRespType,
    #[error("unsupported command was provided")]
    UnsupportedCommandError,
    #[error("provided array has incorrect formatting and cannot be parsed into valid command")]
    IncorrectCommandFormatError,
}

impl From<CommandExecutionError> for RespType {
    fn from(value: CommandExecutionError) -> Self {
        RespType::RError(value.to_string())
    }
}
