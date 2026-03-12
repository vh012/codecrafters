use crate::commands::hash_map::{Key, Value};
use std::collections::HashMap;
use tokio::sync::RwLockWriteGuard;

type Map<'a> = RwLockWriteGuard<'a, HashMap<Key, Value>>;

pub struct RdbCodec<'a> {
    pub is_header_read: bool,
    pub map: Map<'a>,
}

impl<'a> RdbCodec<'a> {
    pub fn new(hash_map: Map<'a>) -> Self {
        Self {
            is_header_read: false,
            map: hash_map,
        }
    }
}
