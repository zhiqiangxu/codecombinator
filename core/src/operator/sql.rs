use super::OperatorError;
use async_std::stream::StreamExt;
use async_std::sync::Weak;
use async_trait::async_trait;
use futures::TryStreamExt;
use futures_core::stream::BoxStream;
use serde::{Deserialize, Serialize};
use sqlx::database::Database;
use sqlx::mysql::{MySql, MySqlArguments, MySqlConnection, MySqlPool};
use sqlx::pool::PoolConnection;
use sqlx::{Execute, Executor};
use std::io::Cursor;
use std::marker::PhantomData;
use thiserror::Error;

pub struct Sql<DB> {
    mysql_pool: MySqlPool,
    phantom: PhantomData<DB>,
}

impl Sql<MySql> {
    pub async fn new(config: Config) -> Sql<MySql> {
        let pool_result = MySqlPool::connect(config.dsn.as_str()).await;

        let pool = match pool_result {
            Ok(p) => p,
            Err(_) => panic!("mysql pool create faild"),
        };

        Sql {
            mysql_pool: pool,
            phantom: PhantomData::<MySql>,
        }
    }

    pub async fn get_executor(&self) -> sqlx::Result<PoolConnection<MySql>> {
        self.mysql_pool.acquire().await
    }

    pub async fn call<'q, 'e, E>(
        &self,
        q: sqlx::query::Query<'q, MySql, MySqlArguments>,
        executor: E,
    ) -> BoxStream<'e, Result<<MySql as Database>::Row, sqlx::Error>>
    where
        E: Executor<'e, Database = MySql>,
        'q: 'e,
    {
        let rows = q.fetch(executor);

        rows
    }
}

#[derive(Error, Debug)]
pub enum SqlError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub dsn: String,
}

impl<DB> super::Operator for Sql<DB> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn mysql() {
        let sql = Sql::<MySql>::new(Config {
            dsn: "mysql://openkg:some_pass@172.168.3.46:3307/openkg?readTimeout=3s&charset=utf8mb4"
                .to_string(),
        })
        .await;

        let mut con = sql.get_executor().await.unwrap();

        let q = sqlx::query(
            r#"
        select * from user limit 10
        "#,
        );

        let mut cursor = sql.call(q, &mut con).await;
        assert!(cursor.next().await.unwrap().is_ok());
    }
}
