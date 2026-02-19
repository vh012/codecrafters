use std::sync::LazyLock;

use tokio::sync::RwLock;

pub(crate) struct Config {
    pub dir: Option<String>,
    pub dbfilename: Option<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            dir: None,
            dbfilename: None,
        }
    }
}

pub(crate) static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(Config::new()));
