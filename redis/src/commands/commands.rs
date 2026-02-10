use crate::{commands::hash_map::HASH_MAP, resp::resp_types::RespDataType};
use thiserror::Error;

pub enum Command {
    Echo(RespDataType),
    Ping(RespDataType),
    Set(RespDataType),
    Get(RespDataType),
}

impl Command {
    pub async fn try_perform(resp_value: RespDataType) -> Result<Self, CommandParseError> {
        match resp_value {
            RespDataType::Arrays(Some(ref arr)) => {
                if arr.is_empty() {
                    return Err(CommandParseError::RespParseError);
                }

                match arr[0] {
                    RespDataType::BulkStrings(Some(ref str_command)) => {
                        match str_command.trim().to_lowercase().as_str() {
                            "ping" if arr.len() == 1 => Ok(Command::Ping(
                                RespDataType::SimpleStrings(Some("PONG".to_string())),
                            )),
                            "echo" if arr.len() == 2 => Ok(Command::Echo(arr[1].clone())),
                            "set" if arr.len() == 3 => {
                                let mut map = HASH_MAP.write().await;

                                map.insert(arr[1].clone(), arr[2].clone());

                                return Ok(Command::Set(RespDataType::SimpleStrings(Some(
                                    "OK".to_string(),
                                ))));
                            }
                            "get" if arr.len() == 2 => {
                                let map = HASH_MAP.read().await;

                                let value = map.get(&arr[1]);

                                return match value {
                                    Some(value) => {
                                        let value = match value {
                                            RespDataType::BulkStrings(_)
                                            | RespDataType::Arrays(_) => value.clone(),
                                            RespDataType::SimpleStrings(Some(str)) => {
                                                RespDataType::BulkStrings(Some(str.clone()))
                                            }
                                            RespDataType::SimpleStrings(None) => {
                                                RespDataType::BulkStrings(None)
                                            }
                                        };

                                        return Ok(Command::Get(value));
                                    }
                                    None => Ok(Command::Set(RespDataType::BulkStrings(None))),
                                };
                            }
                            _ => Err(CommandParseError::UnsupportedCommandError),
                        }
                    }
                    _ => Err(CommandParseError::UnsupportedCommandError),
                }
            }
            _ => Err(CommandParseError::UnsupportedCommandError),
        }
    }
}

impl From<Command> for RespDataType {
    fn from(value: Command) -> Self {
        match value {
            Command::Echo(resp) => resp,
            Command::Ping(resp) => resp,
            Command::Get(resp) => resp,
            Command::Set(resp) => resp,
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandParseError {
    #[error("unable to parse command from resp value")]
    RespParseError,
    #[error("unsupported command was provided")]
    UnsupportedCommandError,
}
