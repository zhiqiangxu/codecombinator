use super::sql_runner::SqlRunner;
use async_std::sync::Weak;
use serde::{Deserialize, Serialize};

use tide::{Body, Request, Result};

pub struct HTTPAPI {
    config: Config,
    w: WType,
}

enum WType {
    SqlRunner(Weak<SqlRunner>),
    None,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub uri: &'static str,
    pub method: Method,
}

#[derive(Deserialize, Serialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    ALL,
}

impl HTTPAPI {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            w: WType::None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn handle(&self, _req: Request<()>) -> Result<Body> {
        match &self.w {
            WType::SqlRunner(w) => match w.upgrade() {
                Some(a) => match a.run_sql().await {
                    Some(json) => {
                        return Body::from_json(&json);
                    }
                    _ => {}
                },
                _ => {}
            },
            WType::None => {}
        }

        Ok(Body::from_string("empty".to_string()))
    }
}

impl super::Operator for HTTPAPI {}

impl super::Monad<SqlRunner> for HTTPAPI {
    type Result = ();

    fn apply(&mut self, w: Weak<SqlRunner>) -> Self::Result {
        self.w = WType::SqlRunner(w);
    }
}
