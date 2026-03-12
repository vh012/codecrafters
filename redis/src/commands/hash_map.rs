use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::resp::types::RespDataType;

pub type Key = RespDataType;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    data: RespDataType,
    ttl: Option<Duration>,
    created_at: Instant,
}

impl Value {
    pub fn new(data: RespDataType, ttl: Option<Duration>) -> Self {
        Self {
            data,
            ttl,
            created_at: Instant::now(),
        }
    }

    pub fn get_data(&self) -> Option<RespDataType> {
        match self.ttl {
            Some(ref ttl) => {
                if self.created_at.elapsed() > *ttl {
                    None
                } else {
                    Some(self.data.clone())
                }
            }
            None => Some(self.data.clone()),
        }
    }
}

pub(crate) static HASH_MAP: LazyLock<RwLock<HashMap<Key, Value>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
