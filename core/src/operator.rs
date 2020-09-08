macro_rules! into_operr {
    ($expression:expr) => {
        match $expression {
            _ => OperatorError::Other(anyhow::Error::from($expression)),
        }
    };
}

pub mod http_api;
pub mod http_server;
pub mod sql;
pub mod sql_runner;

use async_std::sync::Weak;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OperatorError {
    #[error("pin not exists")]
    PinNotExists,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait Source: Sync {
    async fn start(&self) -> Result<(), OperatorError>;
}

impl<T: Source> Operator for T {}

pub trait Operator {}

pub trait Monad<O>
where
    O: Operator,
{
    type Result;

    fn apply(&mut self, op: Weak<O>) -> Self::Result;
}
