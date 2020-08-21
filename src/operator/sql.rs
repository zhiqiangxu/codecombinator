use super::OperatorError;
use async_std::sync::{channel, Arc, Receiver, RecvError, Sender, Weak};
use async_trait::async_trait;
use sqlx::mysql::MySqlPool;
use std::any::TypeId;

pub struct SQL {
    mysql_pool: Arc<MySqlPool>,
    config: Config,
    out_sender: Vec<Weak<Sender<Arc<MySqlPool>>>>,
}

impl SQL {
    pub async fn new(config: Config) -> SQL {
        match config.kind {
            Kind::MySQL => {
                let pool_result = MySqlPool::new(config.dsn.as_str()).await;

                let pool = match pool_result {
                    Ok(p) => p,
                    Err(_) => panic!("mysql pool create faild"),
                };

                SQL {
                    mysql_pool: Arc::new(pool),
                    config: config,
                    out_sender: vec![],
                }
            }
            _ => panic!("only mysql supported"),
        }
    }
}
pub enum Kind {
    MySQL,
}

impl Default for Kind {
    fn default() -> Self {
        Kind::MySQL
    }
}

#[derive(Default)]
pub struct Config {
    pub kind: Kind,
    pub dsn: String,
}

#[async_trait]
impl super::Operator for SQL {
    async fn process(&mut self) -> Result<(), OperatorError> {
        loop {
            let last_idx = self.out_sender.len() - 1;

            for (pos, w) in self.out_sender.iter().enumerate() {
                if pos == last_idx {
                    break;
                }

                match w.upgrade() {
                    Some(s) => s.send(self.mysql_pool.clone()).await,
                    None => {}
                };
            }
            unsafe {
                match self.out_sender.get_unchecked(last_idx).upgrade() {
                    Some(s) => s.send(self.mysql_pool.clone()).await,
                    None => {}
                };
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
        Ok(TypeId::of::<&MySqlPool>())
    }
    fn get_in(&self, i: u8) -> Result<usize, OperatorError> {
        Err(OperatorError::PinNotExists)
    }
    unsafe fn add_out(&mut self, i: u8, sender_ref: usize) -> Result<(), OperatorError> {
        let sender = std::mem::transmute(sender_ref);
        self.out_sender.push(sender);
        Ok(())
    }
}
