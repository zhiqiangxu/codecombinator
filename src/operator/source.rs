use super::OperatorError;
use arbitrary::{Arbitrary, Unstructured};
use async_std::sync::{channel, Receiver, RecvError, Sender};
use async_trait::async_trait;
use std::any::TypeId;
use std::marker::PhantomData;

pub struct Source<'a, T>
where
    T: Arbitrary + Clone,
{
    phantom: PhantomData<T>,
    out_sender: Vec<&'a Sender<T>>,
}

impl<'a, T> Source<'a, T>
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
impl<'a, T> super::Operator for Source<'a, T>
where
    T: Arbitrary + Clone + Send + Sync + std::fmt::Debug,
{
    async fn process(&mut self) -> Result<(), OperatorError> {
        loop {
            let raw = b"This is some raw, unstructured data!";
            let mut unstructured = Unstructured::new(raw);
            let v = T::arbitrary(&mut unstructured).unwrap();

            let last_idx = self.out_sender.len() - 1;

            for (pos, s) in self.out_sender.iter().enumerate() {
                if pos == last_idx {
                    break;
                }
                s.send(v.clone()).await;
            }
            unsafe {
                self.out_sender.get_unchecked(last_idx).send(v).await;
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
        let sender: &'a Sender<T> = std::mem::transmute(sender_ref);
        self.out_sender.push(sender);
        Ok(())
    }
}
