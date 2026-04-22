use std::time::Duration;

use bytes::BytesMut;

use crate::{
    commands::{
        errors::CommandExecutionError,
        hash_map::{HASH_MAP, Value},
    },
    resp::types::RespType,
};

pub(crate) async fn set(
    key: &RespType,
    value: &RespType,
    time_op: Option<&BytesMut>,
    time: Option<&BytesMut>,
) -> Result<RespType, CommandExecutionError> {
    let parse_ttl = |ttl: &BytesMut| {
        str::from_utf8(ttl.as_ref())
            .map_err(|e| {
                CommandExecutionError::IncorrectOptionsError(
                    format!("cannot parse ttl value from bytes: {e}").to_string(),
                )
            })
            .and_then(|str| {
                str::parse::<u64>(str).map_err(|e| {
                    CommandExecutionError::IncorrectOptionsError(
                        format!("cannot parse ttl string into valid u64: {e}").to_string(),
                    )
                })
            })
    };

    let mut map_write = HASH_MAP.write().await;

    match (time_op, time) {
        (Some(op), Some(ttl)) => match op.to_ascii_lowercase().as_slice() {
            b"px" => {
                map_write.insert(
                    key.clone(),
                    Value::new(value.clone(), Some(Duration::from_millis(parse_ttl(ttl)?))),
                );
            }
            b"ex" => {
                map_write.insert(
                    key.clone(),
                    Value::new(value.clone(), Some(Duration::from_secs(parse_ttl(ttl)?))),
                );
            }
            _ => {}
        },
        _ => {
            map_write.insert(key.clone(), Value::new(value.clone(), None));
        }
    }

    Ok(RespType::SimpleString(Some("OK".into())))
}
