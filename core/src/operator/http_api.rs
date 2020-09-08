use super::sql_runner::SqlRunner;
use super::OperatorError;
use async_std::sync::Weak;
use serde::{Deserialize, Serialize};

use tide::{Body, Request};

pub struct HTTPAPI {
    config: Config,
    w: wtype,
}

enum wtype {
    Mysql(Weak<SqlRunner>),
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
            w: wtype::None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn handle(&self, _req: Request<()>) -> Result<Body, OperatorError> {
        match &self.w {
            wtype::Mysql(w) => match w.upgrade() {
                Some(a) => {
                    let json = a.run_sql().await.unwrap();
                    return Ok(Body::from_json(&json).unwrap());
                }
                _ => {}
            },
            wtype::None => {}
        }

        Ok(Body::from_string("empty".to_string()))
    }
}

impl super::Operator for HTTPAPI {}

impl super::Monad<SqlRunner> for HTTPAPI {
    type Result = ();

    fn apply(&mut self, w: Weak<SqlRunner>) -> Self::Result {
        self.w = wtype::Mysql(w);
    }
}
