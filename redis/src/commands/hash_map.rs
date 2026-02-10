use std::{collections::HashMap, sync::LazyLock};

use tokio::sync::RwLock;

use crate::resp::resp_types::RespDataType;

pub(crate) static HASH_MAP: LazyLock<RwLock<HashMap<RespDataType, RespDataType>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
