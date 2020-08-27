use async_std::sync::{Arc, Weak};
use async_std::task;
use codecombinator::operator::{http_api, sql, sql_runner as runner, Monad, Operator};
use sqlx::mysql::MySql;

#[async_std::main]
async fn main() {
    let sql = sql::SQL::<MySql>::new(sql::Config {
        dsn: "mysql://openkg:some_pass@172.168.3.46:3307/openkg?readTimeout=3s&charset=utf8mb4"
            .to_string(),
    })
    .await;

    let sql = Arc::new(sql);

    let mut sql_runner = runner::SqlRunner::<MySql>::new(runner::Config {
        sql: "select * from user limit 10".to_string(),
    });

    sql_runner.apply(Arc::downgrade(&sql));
    let sql_runner = Arc::new(sql_runner);

    let mut api = http_api::HTTPAPI::new(http_api::Config {
        uri: "/",
        method: http_api::Method::Get,
    });

    api.apply(Arc::downgrade(&sql_runner));

    // let graph_task = task::spawn(async move {
    //     match g.process().await {
    //         Ok(_) => {}
    //         Err(err) => println!("err : {:?}", err),
    //     };
    // });
    // task::block_on(graph_task);
}
