use super::OperatorError;
use async_std::sync::Mutex;
use async_std::task;
use async_trait::async_trait;
use std::any::TypeId;
use std::sync::Arc;
use thiserror::Error;
use waitgroup::WaitGroup;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("pin type mismatch")]
    PinTypeNotMatch,
    #[error("operator not in graph")]
    OperatorNotInGraph,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct Graph {
    operators: Vec<Box<dyn super::Operator>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph { operators: vec![] }
    }

    pub fn add_operator(&mut self, operator: Box<dyn super::Operator>) -> usize {
        let len = self.operators.len();

        self.operators.push(operator);

        len
    }

    pub fn connect(&mut self, a: usize, i: u8, b: usize, j: u8) -> Result<(), OperatorError> {
        if a >= self.operators.len() || b >= self.operators.len() {
            return Err(into_anyerr!(GraphError::OperatorNotInGraph))?;
        }

        let ptr = self.operators.as_mut_ptr();
        unsafe {
            let in_pin = (*ptr.add(b)).get_in(j)?;
            (*ptr.add(a)).add_out(i, in_pin)
        }
    }
}

unsafe impl Sync for Graph {}

#[async_trait]
impl super::Operator for Graph {
    async fn process(&mut self) -> Result<(), OperatorError> {
        let wg = WaitGroup::new();

        let g_result = Arc::new(Mutex::new(Ok(())));

        for mut op in self.operators.drain(..) {
            let w = wg.worker();
            let s_result = g_result.clone();
            task::spawn(async move {
                let result = op.process().await;
                if result.is_err() {
                    *s_result.lock().await = result
                }
                drop(w);
            });
        }

        wg.wait().await;

        let lock = Arc::try_unwrap(g_result).expect("Lock still has multiple owners");
        lock.into_inner()
    }

    fn get_in_count(&self) -> u8 {
        panic!("todo")
    }
    fn get_out_count(&self) -> u8 {
        panic!("todo")
    }
    fn get_in_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        panic!("todo")
    }
    fn get_out_type(&self, i: u8) -> Result<TypeId, OperatorError> {
        panic!("todo")
    }

    fn get_in(&self, i: u8) -> Result<usize, OperatorError> {
        panic!("todo")
    }

    unsafe fn add_out(&mut self, i: u8, sender_ref: usize) -> Result<(), OperatorError> {
        panic!("todo")
    }
}
