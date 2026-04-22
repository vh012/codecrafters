use bytes::BytesMut;

use crate::{commands::errors::CommandExecutionError, config::CONFIG, resp::types::RespType};

pub(crate) async fn config(
    action: &BytesMut,
    key: &BytesMut,
) -> Result<RespType, CommandExecutionError> {
    let config = CONFIG.read().await;

    match action.to_ascii_lowercase().as_slice() {
        b"get" => match key.to_ascii_lowercase().as_slice() {
            b"dir" if config.dir.is_some() => Ok(RespType::Array(Some(vec![
                RespType::BulkString(Some(action.clone())),
                RespType::BulkString(Some(BytesMut::from_iter::<Vec<u8>>(
                    config.dir.as_ref().unwrap().clone().into(),
                ))),
            ]))),
            b"dbfilename" => Ok(RespType::Array(Some(vec![
                RespType::BulkString(Some(action.clone())),
                RespType::BulkString(Some(BytesMut::from_iter::<Vec<u8>>(
                    config.dbfilename.as_ref().unwrap().clone().into(),
                ))),
            ]))),
            _ => Err(CommandExecutionError::IncorrectOptionsError(
                "unknown config key".to_string(),
            )),
        },
        _ => Err(CommandExecutionError::IncorrectOptionsError(
            "unknown config action".to_string(),
        )),
    }
}
