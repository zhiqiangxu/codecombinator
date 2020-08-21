use super::OperatorError;
use async_std::sync::{channel, Arc, Receiver, RecvError, Sender, Weak};
use async_trait::async_trait;
use std::any::TypeId;
use std::marker::Send;
use std::ops::Add as StdAdd;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddError {
    #[error("recv1 error")]
    Recv1 { source: RecvError },
    #[error("recv2 error")]
    Recv2 { source: RecvError },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct Add<T>
where
    T: StdAdd + Copy + Send,
    <T as std::ops::Add>::Output: Send + Sync + Clone,
{
    in_sender1: Arc<Sender<T>>,
    in_recv1: Receiver<T>,
    in_sender2: Arc<Sender<T>>,
    in_recv2: Receiver<T>,
    out_sender: Vec<Weak<Sender<<T as std::ops::Add>::Output>>>,
}

impl<T> Add<T>
where
    T: StdAdd + Copy + Send,
    <T as std::ops::Add>::Output: Send + Sync + Clone,
{
    pub fn new() -> Self {
        let (in_sender1, in_recv1) = channel::<T>(1);
        let (in_sender2, in_recv2) = channel::<T>(1);
        Add {
            in_sender1: Arc::new(in_sender1),
            in_recv1,
            in_sender2: Arc::new(in_sender2),
            in_recv2,
            out_sender: vec![],
        }
    }
}

#[async_trait]
impl<T> super::Operator for Add<T>
where
    T: StdAdd + Copy + Send + std::fmt::Debug + 'static,
    <T as std::ops::Add>::Output: Send + Sync + Clone + std::fmt::Debug + 'static,
{
    async fn process(&mut self) -> Result<(), OperatorError> {
        loop {
            let v1 = self
                .in_recv1
                .recv()
                .await
                .map_err(|source| into_anyerr!(AddError::Recv1 { source }))?;

            let v2 = self
                .in_recv2
                .recv()
                .await
                .map_err(|source| into_anyerr!(AddError::Recv2 { source }))?;

            if self.out_sender.len() == 0 {
                continue;
            }

            let v3 = v1 + v2;

            println!("{:?} + {:?} = {:?}", v1, v2, v3);

            let last_idx = self.out_sender.len() - 1;

            for (pos, w) in self.out_sender.iter().enumerate() {
                if pos == last_idx {
                    break;
                }
                match w.upgrade() {
                    Some(s) => {
                        s.send(v3.clone()).await;
                    }
                    None => {}
                }
            }
            unsafe {
                match self.out_sender.get_unchecked(last_idx).upgrade() {
                    Some(s) => {
                        s.send(v3).await;
                    }
                    None => {}
                }
            }
        }
    }

    fn get_in_count(&self) -> u8 {
        2
    }

    fn get_out_count(&self) -> u8 {
        1
    }

    fn get_in_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        if i >= 2 {
            Err(OperatorError::PinNotExists)?
        }

        Ok(TypeId::of::<T>())
    }

    fn get_out_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        if i >= 1 {
            Err(OperatorError::PinNotExists)?
        }

        Ok(TypeId::of::<<T as std::ops::Add>::Output>())
    }

    fn get_in(&self, i: u8) -> Result<usize, OperatorError> {
        match i {
            0 => Ok(Arc::downgrade(&self.in_sender1).into_raw() as _),
            1 => Ok(Arc::downgrade(&self.in_sender2).into_raw() as _),
            _ => Err(OperatorError::PinNotExists)?,
        }
    }

    unsafe fn add_out(&mut self, i: u8, sender_ref: usize) -> Result<(), OperatorError> {
        let sender = Weak::from_raw(sender_ref as *const _);
        self.out_sender.push(sender);
        Ok(())
    }
}
