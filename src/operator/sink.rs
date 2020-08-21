use super::OperatorError;
use async_std::sync::{channel, Arc, Receiver, RecvError, Sender};
use async_trait::async_trait;
use std::any::TypeId;
use thiserror::Error;

pub struct Sink<T> {
    in_sender: Arc<Sender<T>>,
    in_recv: Receiver<T>,
}

#[derive(Error, Debug)]
pub enum SinkError {
    #[error("recv error")]
    Recv { source: RecvError },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<T> Sink<T> {
    pub fn new() -> Self {
        let (in_sender, in_recv) = channel::<T>(1);
        Sink {
            in_sender: Arc::new(in_sender),
            in_recv,
        }
    }
}

#[async_trait]
impl<T> super::Operator for Sink<T>
where
    T: Send + 'static,
{
    async fn process(&mut self) -> Result<(), OperatorError> {
        loop {
            let _ = self
                .in_recv
                .recv()
                .await
                .map_err(|source| into_anyerr!(SinkError::Recv { source }))?;

            println!("sink ok");
        }
    }
    fn get_in_count(&self) -> u8 {
        1
    }
    fn get_out_count(&self) -> u8 {
        0
    }
    fn get_in_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        if i >= 1 {
            Err(OperatorError::PinNotExists)?
        }

        Ok(TypeId::of::<T>())
    }
    fn get_out_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        Err(OperatorError::PinNotExists)
    }
    fn get_in(&self, i: u8) -> Result<usize, OperatorError> {
        match i {
            0 => Ok(Arc::downgrade(&self.in_sender).into_raw() as usize),
            _ => Err(OperatorError::PinNotExists)?,
        }
    }
    unsafe fn add_out(&mut self, i: u8, sender_ref: usize) -> Result<(), OperatorError> {
        Err(OperatorError::PinNotExists)
    }
}
