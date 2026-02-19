use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, SystemTime},
};

use tokio::sync::RwLock;

use crate::resp::types::RespDataType;

pub type Key = RespDataType;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    data: RespDataType,
    ttl: Option<Duration>,
    created_at: SystemTime,
}

impl Value {
    pub fn new(data: RespDataType, ttl: Option<Duration>) -> Self {
        Self {
            data,
            ttl,
            created_at: SystemTime::now(),
        }
    }

    pub fn get_data(&self) -> Option<RespDataType> {
        match self.ttl {
            Some(ref ttl) => match self.created_at.elapsed() {
                Ok(elapsed) => {
                    if elapsed.as_millis() > ttl.as_millis() {
                        None
                    } else {
                        Some(self.data.clone())
                    }
                }
                Err(_) => None,
            },
            None => Some(self.data.clone()),
        }
    }
}

pub(crate) static HASH_MAP: LazyLock<RwLock<HashMap<Key, Value>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
