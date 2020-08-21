macro_rules! into_anyerr {
    ($expression:expr) => {
        anyhow::Error::from($expression)
    };
}

pub mod add;
pub mod graph;
pub mod sink;
pub mod source;
pub mod sql;

use async_trait::async_trait;
use std::any::TypeId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OperatorError {
    #[error("pin not exists")]
    PinNotExists,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait Operator: Send + Sync {
    async fn process(&mut self) -> Result<(), OperatorError>;
    fn get_in_count(&self) -> u8;
    fn get_out_count(&self) -> u8;
    fn get_in_type(&self, i: u8) -> Result<TypeId, OperatorError>;
    fn get_out_type(&self, i: u8) -> Result<TypeId, OperatorError>;
    fn get_in(&self, i: u8) -> Result<usize, OperatorError>;
    unsafe fn add_out(&mut self, i: u8, sender_ref: usize) -> Result<(), OperatorError>;
}
