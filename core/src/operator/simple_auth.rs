use serde::{Deserialize, Serialize};

pub struct SimpleAuth {
    config: Config,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub secret: &'static str,
}

impl SimpleAuth {
    pub fn new(config: Config) -> SimpleAuth {
        SimpleAuth { config }
    }

    pub fn auth(&self, key: &str) -> bool {
        self.config.secret == key
    }
}

impl super::Operator for SimpleAuth {}
