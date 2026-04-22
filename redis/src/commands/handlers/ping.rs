use crate::resp::types::RespType;

pub(crate) async fn ping() -> RespType {
    RespType::SimpleString(Some("PONG".into()))
}
