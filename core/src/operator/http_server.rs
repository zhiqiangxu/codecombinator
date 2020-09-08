use super::http_api::{Method, HTTPAPI};
use super::OperatorError;
use async_std::sync::Weak;
use async_trait::async_trait;
use futures::FutureExt;
use serde::{Deserialize, Serialize};

use tide::Body;

pub struct HTTPServer {
    config: Config,
    ws: Vec<Weak<HTTPAPI>>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub listen_addr: &'static str,
}

impl HTTPServer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ws: Vec::default(),
        }
    }
}

#[async_trait]
impl super::Source for HTTPServer {
    async fn start(&self) -> Result<(), OperatorError> {
        let mut app = tide::new();

        for w in &self.ws {
            match w.upgrade() {
                Some(a) => {
                    let config = a.config();
                    let mut route = app.at(config.uri);

                    let w = w.clone();
                    let handler = move |req| {
                        let w = w.clone();
                        return async move {
                            match w.upgrade() {
                                Some(a) => {
                                    let body = std::panic::AssertUnwindSafe(a.handle(req))
                                        .catch_unwind()
                                        .await;

                                    match body {
                                        Ok(result) => match result {
                                            Ok(body) => Ok(body),
                                            Err(err) => Ok(Body::from_string(format!(
                                                "handler error: {:?}",
                                                err
                                            ))),
                                        },
                                        Err(err) => Ok(Body::from_string(format!(
                                            "panic in handler: {:?}",
                                            err
                                        ))),
                                    }
                                }
                                _ => Ok(Body::from_string("handler is down".to_string())),
                            }
                        };
                    };

                    match config.method {
                        Method::GET => {
                            route.get(handler);
                        }
                        Method::POST => {
                            route.post(handler);
                        }
                        Method::DELETE => {
                            route.delete(handler);
                        }
                        Method::PUT => {
                            route.put(handler);
                        }
                        Method::ALL => {
                            route.all(handler);
                        }
                    }
                }
                None => {}
            }
        }
        app.listen(self.config.listen_addr).await.unwrap();
        Ok(())
    }
}

impl super::Monad<HTTPAPI> for HTTPServer {
    type Result = ();

    fn apply(&mut self, w: Weak<HTTPAPI>) -> Self::Result {
        self.ws.push(w);
    }
}
