use super::saga_aggregator::SagaAggregator;
use super::simple_auth::SimpleAuth;
use super::sql_runner::SqlRunner;
use async_std::sync::Weak;
use serde::{Deserialize, Serialize};

use tide::{Body, Request, Result};

pub struct HTTPAPI {
    config: Config,
    w: WType,
    auth: AuthType,
}

enum AuthType {
    SimpleAuth(Weak<SimpleAuth>),
    None,
}

enum WType {
    SqlRunner(Weak<SqlRunner>),
    SagaAggregator(Weak<SagaAggregator>),
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

const AUTHKEY: &str = "Authorization";

impl HTTPAPI {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            w: WType::None,
            auth: AuthType::None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn handle(&self, req: Request<()>) -> Result<Body> {
        match &self.auth {
            AuthType::SimpleAuth(auth) => match auth.upgrade() {
                Some(a) => {
                    let mut ok = false;
                    match req.header(AUTHKEY) {
                        Some(header) => {
                            if a.auth(header.get(0).unwrap().as_str()) {
                                ok = true;
                            }
                        }
                        None => {}
                    };
                    if !ok {
                        return Ok(Body::from_string("permission denied".to_string()));
                    }
                }
                _ => {}
            },
            AuthType::None => {}
        }
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
            WType::SagaAggregator(w) => match w.upgrade() {
                Some(a) => {
                    return a.handle(req).await;
                }
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

impl super::Monad<SagaAggregator> for HTTPAPI {
    type Result = ();

    fn apply(&mut self, w: Weak<SagaAggregator>) -> Self::Result {
        self.w = WType::SagaAggregator(w);
    }
}

impl super::Monad<SimpleAuth> for HTTPAPI {
    type Result = ();

    fn apply(&mut self, w: Weak<SimpleAuth>) -> Self::Result {
        self.auth = AuthType::SimpleAuth(w);
    }
}
