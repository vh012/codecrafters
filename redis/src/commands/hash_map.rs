use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::resp::types::RespType;

pub type Key = RespType;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    data: RespType,
    ttl: Option<Duration>,
    created_at: Instant,
}

impl Value {
    pub fn new(data: RespType, ttl: Option<Duration>) -> Self {
        Self {
            data,
            ttl,
            created_at: Instant::now(),
        }
    }

    pub fn get_data(&self) -> Option<RespType> {
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
