use crate::{commands::hash_map::HASH_MAP, resp::types::RespType};

pub(crate) async fn keys() -> RespType {
    let mut keys = vec![];
    let map_read = HASH_MAP.read().await;

    for k in map_read.keys() {
        keys.push(k.clone());
    }

    RespType::Array(Some(keys))
}
