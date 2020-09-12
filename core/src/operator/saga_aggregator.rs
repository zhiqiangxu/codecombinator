use async_std::stream::StreamExt;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use defer::defer;
use futures::stream::futures_unordered::FuturesUnordered;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, value::RawValue};
use std::collections::HashMap;
use surf;
use tide::{Body, Request, Result};
use waitgroup::WaitGroup;

pub struct SagaAggregator {
    map: HashMap<u32, APIExt>,
}

impl super::Operator for SagaAggregator {}
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub apis: Vec<API>,
}
#[derive(Clone)]
struct APIExt {
    url: String,
    api: API,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct API {
    pub id: u32,
    pub key: String,
}
#[derive(Deserialize)]
struct OneRequest {
    id: u32,
    body: Box<RawValue>,
}
#[derive(Deserialize)]
struct BatchRequest {
    reqs: Vec<OneRequest>,
}

#[derive(Debug, Serialize)]
struct OneResponse {
    id: u32,
    body: String,
}

#[derive(Debug, Default, Serialize)]
struct BatchResponse {
    respes: Vec<OneResponse>,
}

#[derive(Deserialize)]
struct GetApiDetailByApiId {
    result: SagaResult,
}

#[derive(Deserialize)]
struct SagaResult {
    #[serde(rename = "apiBasicInfo")]
    api_basic_info: ApiBasicInfo,
}
#[derive(Deserialize)]
struct ApiBasicInfo {
    #[serde(rename = "apiUrl")]
    api_url: String,
}

impl SagaAggregator {
    pub async fn new(config: Config) -> SagaAggregator {
        let get_detail = |id: u32| {
            return async move {
                let mut resp = surf::get(format!(
                    "http://106.75.254.241:4000/api/v1/apilist/getApiDetailByApiId/{}",
                    id
                ))
                .await
                .unwrap();
                let body_str = resp.body_string().await.unwrap();

                let detail: GetApiDetailByApiId = from_str(body_str.as_str()).unwrap();
                detail
            };
        };

        // FYI: https://stackoverflow.com/questions/59237895/how-to-process-a-vector-as-an-asynchronous-stream

        let convert = |api: &API| {
            let api = api.clone();
            return async move {
                let detail = get_detail(api.id).await;
                (
                    api.id,
                    APIExt {
                        url: detail.result.api_basic_info.api_url,
                        api: api.clone(),
                    },
                )
            };
        };
        let map: HashMap<_, _> = config
            .apis
            .iter()
            .map(convert)
            .collect::<FuturesUnordered<_>>()
            .collect()
            .await;

        SagaAggregator { map }
    }

    pub async fn handle(&self, mut req: Request<()>) -> Result<Body> {
        let batch: BatchRequest = req.body_json().await?;

        let batch_resp = Arc::new(Mutex::new(BatchResponse::default()));

        let wg = WaitGroup::new();
        for req in batch.reqs {
            let api_ext = self.map[&req.id].clone();
            let w = wg.worker();
            let batch_resp = batch_resp.clone();
            task::spawn(async move {
                let _d = defer(|| drop(w));

                let mut response =
                    surf::post(api_ext.url.replace(":apikey", api_ext.api.key.as_str()))
                        .body_json(&req.body)
                        .unwrap()
                        .await
                        .unwrap();

                let body = response.body_string().await.unwrap();

                batch_resp
                    .lock()
                    .await
                    .respes
                    .push(OneResponse { id: req.id, body });
            });
        }
        wg.wait().await;

        let value = Arc::try_unwrap(batch_resp).unwrap().into_inner();
        return Ok(Body::from_json(&value).unwrap());
    }
}

#[cfg(test)]
mod tests {

    use async_std;

    #[async_std::test]
    async fn it_works() {}
}
