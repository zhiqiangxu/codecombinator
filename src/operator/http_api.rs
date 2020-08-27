use super::sql_runner::SqlRunner;
use super::OperatorError;
use async_std::sync::{Arc, Weak};
use sqlx::mysql::MySql;
use tide::{Body, Request};

pub struct HTTPAPI<DB> {
    config: Config,
    w: Weak<SqlRunner<DB>>,
}

pub struct Config {
    pub uri: &'static str,
    pub method: Method,
}

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl HTTPAPI<MySql> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            w: Weak::default(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn handle(&self, req: Request<()>) -> Result<Body, OperatorError> {
        match self.w.upgrade() {
            Some(a) => {
                let json = a.run_sql().await.unwrap();
                Ok(Body::from_json(&json).unwrap())
            }
            None => Ok(Body::from_string("empty".to_string())),
        }
    }
}

impl super::Operator for HTTPAPI<MySql> {}

impl super::Monad<SqlRunner<MySql>> for HTTPAPI<MySql> {
    type Result = ();

    fn apply(&mut self, w: Weak<SqlRunner<MySql>>) -> Self::Result {
        self.w = w
    }
}
