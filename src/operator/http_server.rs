use super::http_api::{Method, HTTPAPI};
use super::OperatorError;
use async_std::sync::{Arc, Weak};
use async_trait::async_trait;
use sqlx::mysql::MySql;
use tide::{Body, Request, Result as TResult};

pub struct HTTPServer {
    config: Config,
    ws: Vec<Weak<HTTPAPI<MySql>>>,
}

pub struct Config {
    listen_addr: String,
}

impl HTTPServer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ws: Vec::default(),
        }
    }
}

impl super::Operator for HTTPServer {}

#[async_trait]
impl super::Source for HTTPServer {
    async fn start(&mut self) -> Result<(), OperatorError> {
        let mut app = tide::new();

        for w in &self.ws {
            match w.upgrade() {
                Some(a) => {
                    let config = a.config();
                    let mut route = app.at(config.uri);

                    let api = w.clone();
                    let handler = async |req| {
                        let api = api.clone();

                        match api.upgrade() {
                            Some(a) => {
                                let body = a.handle(req).await.unwrap();
                                Ok(body)
                            }
                            None => Ok(Body::from_string("empty".to_string())),
                        }
                    };

                    match config.method {
                        Method::Get => {
                            route.get(handler);
                        }
                        Method::Post => {
                            route.post(handler);
                        }
                        Method::Delete => {
                            route.delete(handler);
                        }
                        Method::Put => {
                            route.put(handler);
                        }
                    }
                }
                None => {}
            }
        }
        app.listen(self.config.listen_addr.as_str()).await.unwrap();
        Ok(())
    }
}

impl super::Monad<HTTPAPI<MySql>> for HTTPServer {
    type Result = ();

    fn apply(&mut self, w: Weak<HTTPAPI<MySql>>) -> Self::Result {
        self.ws.push(w);
    }
}
