use crate::{
    commands::{
        errors::CommandExecutionError,
        handlers::{config::config, echo::echo, get::get, keys::keys, ping::ping, set::set},
    },
    resp::types::RespType,
};

pub struct Processor;

impl Processor {
    pub async fn exec_from_resp(value: RespType) -> Result<RespType, CommandExecutionError> {
        match value {
            RespType::Array(Some(arr)) => match arr.as_slice() {
                [RespType::BulkString(Some(cmd)), params @ ..] => {
                    match cmd.to_ascii_lowercase().as_slice() {
                        b"keys" => Ok(keys().await),
                        b"echo" => match params {
                            [RespType::BulkString(Some(arg))] => Ok(echo(arg).await),
                            _ => Err(CommandExecutionError::IncorrectCommandFormatError),
                        },
                        b"ping" => Ok(ping().await),
                        b"set" => match params {
                            [
                                key @ RespType::BulkString(Some(_)),
                                value @ RespType::BulkString(Some(_)),
                            ] => Ok(set(key, value, None, None).await?),
                            [
                                key @ RespType::BulkString(Some(_)),
                                value @ RespType::BulkString(Some(_)),
                                RespType::BulkString(Some(time_op)),
                                RespType::BulkString(Some(time)),
                            ] => Ok(set(key, value, Some(time_op), Some(time)).await?),
                            _ => Err(CommandExecutionError::IncorrectCommandFormatError),
                        },
                        b"get" => match params {
                            [key @ RespType::BulkString(Some(_))] => Ok(get(key).await),
                            _ => Err(CommandExecutionError::IncorrectCommandFormatError),
                        },
                        b"config" => match params {
                            [
                                RespType::BulkString(Some(action)),
                                RespType::BulkString(Some(key)),
                            ] => Ok(config(action, key).await?),
                            _ => Err(CommandExecutionError::IncorrectCommandFormatError),
                        },
                        _ => Err(CommandExecutionError::UnsupportedCommandError),
                    }
                }
                _ => Err(CommandExecutionError::IncorrectCommandFormatError),
            },
            _ => Err(CommandExecutionError::UnsupportedRespType),
        }
    }
}
