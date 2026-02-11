use std::time::Duration;

use crate::{
    commands::hash_map::HASH_MAP, commands::hash_map::Value, resp::resp_types::RespDataType,
};
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
                            "set" if arr.len() == 3 || arr.len() == 5 => {
                                let mut map = HASH_MAP.write().await;

                                let key = arr[1].clone();

                                if arr.len() == 3 {
                                    map.insert(key, Value::new(arr[2].clone(), None));
                                } else {
                                    match arr[3] {
                                        RespDataType::BulkStrings(Some(ref str)) => {
                                            match str.trim().to_lowercase().as_str() {
                                                op @ "px" | op @ "ex" => {
                                                    match arr[4] {
                                                        RespDataType::BulkStrings(Some(ref str)) => {
                                                            let ttl = str::from_utf8(str.as_bytes())
                                                                .map_err(|e|
                                                                    CommandParseError::OptionParseError(
                                                                        format!("cannot parse ttl value from bytes: {e}").to_string()
                                                                    )
                                                                )
                                                                .and_then(|str|
                                                                    str::parse::<u64>(str)
                                                                        .map_err(|e|
                                                                            CommandParseError::OptionParseError(
                                                                                format!("cannot parse ttl string into valid u64: {e}").to_string()
                                                                            )
                                                                        )
                                                                ).and_then(|num| Ok(if op == "ex" { Duration::from_secs(num) } else { Duration::from_millis(num) }))?;

                                                            map.insert(key, Value::new(arr[2].clone(), Some(ttl)));
                                                        },
                                                        _ => return Err(
                                                            CommandParseError::OptionParseError(
                                                                "expected BulkStrings type that represents a numeric argument".to_string()
                                                            )
                                                        )
                                                    }
                                                },
                                                _ => return Err(CommandParseError::UnsupportedOptionsError)
                                            }
                                        }
                                        _ => {
                                            return Err(CommandParseError::UnsupportedOptionsError);
                                        }
                                    }
                                }

                                return Ok(Command::Set(RespDataType::SimpleStrings(Some(
                                    "OK".to_string(),
                                ))));
                            }
                            "get" if arr.len() == 2 => {
                                let key = &arr[1];

                                let mut map = HASH_MAP.write().await;

                                let value = map.get(key);

                                return match value {
                                    Some(value) => {
                                        return Ok(Command::Get(match value.get_data() {
                                            Some(value) => value,
                                            None => {
                                                map.remove(key);

                                                RespDataType::BulkStrings(None)
                                            }
                                        }));
                                    }
                                    _ => Ok(Command::Get(RespDataType::BulkStrings(None))),
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
    #[error("unsupported options was provided")]
    UnsupportedOptionsError,
    #[error("unable to parse option args: {0}")]
    OptionParseError(String),
}
