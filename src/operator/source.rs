use super::OperatorError;
use arbitrary::{Arbitrary, Unstructured};
use async_std::sync::{Sender, Weak};
use async_trait::async_trait;
use std::any::TypeId;
use std::marker::PhantomData;

pub struct Source<T>
where
    T: Arbitrary + Clone,
{
    phantom: PhantomData<T>,
    out_sender: Vec<Weak<Sender<T>>>,
}

impl<T> Source<T>
where
    T: Arbitrary + Clone,
{
    pub fn new() -> Self {
        Source {
            phantom: PhantomData,
            out_sender: vec![],
        }
    }
}

#[async_trait]
impl<T> super::Operator for Source<T>
where
    T: Arbitrary + Clone + Send + Sync + std::fmt::Debug,
{
    async fn process(&mut self) -> Result<(), OperatorError> {
        loop {
            let raw = b"This is some raw, unstructured data!";
            let mut unstructured = Unstructured::new(raw);
            let v = T::arbitrary(&mut unstructured).unwrap();

            let last_idx = self.out_sender.len() - 1;

            for (pos, w) in self.out_sender.iter().enumerate() {
                if pos == last_idx {
                    break;
                }
                match w.upgrade() {
                    Some(s) => {
                        s.send(v.clone()).await;
                    }
                    None => {}
                }
            }
            unsafe {
                match self.out_sender.get_unchecked(last_idx).upgrade() {
                    Some(s) => {
                        s.send(v).await;
                    }
                    None => {}
                }
            }
        }
    }
    fn get_in_count(&self) -> u8 {
        0
    }
    fn get_out_count(&self) -> u8 {
        1
    }
    fn get_in_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        Err(OperatorError::PinNotExists)
    }
    fn get_out_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        Ok(TypeId::of::<T>())
    }
    fn get_in(&self, i: u8) -> Result<usize, OperatorError> {
        Err(OperatorError::PinNotExists)
    }
    unsafe fn add_out(&mut self, i: u8, sender_ref: usize) -> Result<(), OperatorError> {
        let sender = Weak::from_raw(sender_ref as *const _);
        self.out_sender.push(sender);
        Ok(())
    }
}
