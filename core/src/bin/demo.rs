// generated by build.php at 2020-09-12 04:50:07

use async_std::sync::Arc;
use core::operator::{
    http_api, http_server, saga_aggregator, simple_auth, Monad, Source,
};

#[async_std::main]
async fn main() {
    let config0 = serde_json::from_str(
        "{\"apis\":[{\"id\":2002,\"key\":\"testkey1\"},{\"id\":2003,\"key\":\"testkey2\"}]}",
    )
    .unwrap();

    let op0 = Arc::new(saga_aggregator::SagaAggregator::new(config0).await);

    let config1 = serde_json::from_str("{\"uri\":\"/\",\"method\":\"GET\"}").unwrap();

    let mut op1 = Arc::new(http_api::HTTPAPI::new(config1));

    let config2 = serde_json::from_str("{\"listen_addr\":\"127.0.0.1:8088\"}").unwrap();

    let mut op2 = Arc::new(http_server::HTTPServer::new(config2));

    let config3 = serde_json::from_str("{\"secret\":\"abcd\"}").unwrap();

    let op3 = Arc::new(simple_auth::SimpleAuth::new(config3));

    Arc::get_mut(&mut op1).unwrap().apply(Arc::downgrade(&op0));

    Arc::get_mut(&mut op1).unwrap().apply(Arc::downgrade(&op3));

    Arc::get_mut(&mut op2).unwrap().apply(Arc::downgrade(&op1));

    let mut handles = vec![];

    handles.push(::async_std::task::spawn(async move {
        match op2.start().await {
            Ok(_) => {}
            Err(err) => println!("err : {:?}", err),
        };
    }));

    ::futures::future::join_all(handles).await;
}
