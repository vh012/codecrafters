use crate::{commands::hash_map::HASH_MAP, resp::types::RespType};

pub(crate) async fn get(key: &RespType) -> RespType {
    let map_read = HASH_MAP.read().await;

    let get_record = map_read.get(key);

    match get_record {
        Some(record) => match record.get_data() {
            Some(value) => value,
            None => {
                drop(map_read);

                let mut map_write = HASH_MAP.write().await;

                map_write.remove(key);

                RespType::BulkString(None)
            }
        },
        _ => RespType::BulkString(None),
    }
}
