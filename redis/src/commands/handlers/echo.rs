use bytes::BytesMut;

use crate::resp::types::RespType;

pub(crate) async fn echo(arg: &BytesMut) -> RespType {
    RespType::BulkString(Some(arg.clone()))
}
