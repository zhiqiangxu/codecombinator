use async_std::stream::StreamExt;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use defer::defer;
use futures::stream::futures_unordered::FuturesUnordered;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, value::RawValue};
use std::collections::{HashMap, HashSet};
use surf;
use tide::{Body, Request, Result};
use waitgroup::WaitGroup;

pub struct SagaAggregator {
    api_info: HashMap<u32, APIInfo>,
    param_mapping: HashMap<Param, usize>,
    equal_sets: Vec<HashSet<Param>>,
}

impl super::Operator for SagaAggregator {}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub apis: Vec<API>,
    pub mapping: Option<HashMap<String, Vec<Param>>>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Param {
    pub id: u32,
    pub name: String,
}

#[derive(Clone)]
struct APIInfo {
    url: String,
    api: API,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct API {
    pub id: u32,
    pub key: String,
}
#[derive(Clone, Deserialize)]
struct OneRequest {
    id: u32,
    body: HashMap<String, Box<RawValue>>,
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
                    APIInfo {
                        url: detail.result.api_basic_info.api_url,
                        api: api.clone(),
                    },
                )
            };
        };
        let api_info: HashMap<_, _> = config
            .apis
            .iter()
            .map(convert)
            .collect::<FuturesUnordered<_>>()
            .collect()
            .await;

        let mut equal_sets = Vec::<HashSet<Param>>::new();
        let mut param_mapping = HashMap::<Param, usize>::new();
        match config.mapping {
            Some(mapping) => {
                for (from, tos) in mapping {
                    let mut set_idx = -1;
                    let from_param: Param = serde_json::from_str(from.as_str()).unwrap();
                    match param_mapping.get(&from_param) {
                        Some(idx) => set_idx = *idx as i32,
                        None => {}
                    }
                    for to in &tos {
                        match param_mapping.get(&to) {
                            Some(idx) => {
                                if set_idx >= 0 {
                                    if set_idx != *idx as i32 {
                                        panic!("bug happened");
                                    }
                                } else {
                                    set_idx = *idx as i32
                                }
                            }
                            None => {}
                        }
                    }
                    if set_idx < 0 {
                        equal_sets.push(HashSet::<Param>::new());
                        set_idx = (equal_sets.len() - 1) as i32;
                    }

                    let uidx = set_idx as usize;
                    let set = equal_sets.get_mut(uidx).unwrap();
                    if param_mapping.get(&from_param).is_none() {
                        if !set.insert(from_param.clone()) {
                            panic!("bug happened");
                        }
                        param_mapping.insert(from_param, uidx);
                    }
                    for to in tos {
                        if param_mapping.get(&to).is_none() {
                            if !set.insert(to.clone()) {
                                panic!("bug happened");
                            }
                            param_mapping.insert(to, uidx);
                        }
                    }
                }
            }
            None => {}
        };

        SagaAggregator {
            api_info,
            param_mapping,
            equal_sets,
        }
    }

    pub async fn handle(&self, mut req: Request<()>) -> Result<Body> {
        let mut batch: BatchRequest = req.body_json().await?;

        let batch_resp = Arc::new(Mutex::new(BatchResponse::default()));

        let mut id_to_idx = HashMap::<u32, usize>::new();
        for (idx, req) in (&batch.reqs).iter().enumerate() {
            if id_to_idx.insert(req.id, idx).is_some() {
                panic!(format!("duplicate api id found : {}", req.id));
            }
        }

        let clone_reqs = batch.reqs.clone();
        for req in clone_reqs {
            for (k, v) in &req.body {
                let param = Param {
                    id: req.id,
                    name: k.clone(),
                };
                match self.param_mapping.get(&param) {
                    Some(idx) => {
                        let set = self.equal_sets.get(*idx).unwrap();
                        for p in set {
                            if *p != param {
                                let other_idx = id_to_idx.get(&p.id).unwrap();
                                let other_req = batch.reqs.get_mut(*other_idx).unwrap();

                                other_req.body.insert(p.name.clone(), v.clone());
                            }
                        }
                    }
                    None => {}
                }
            }
        }

        let wg = WaitGroup::new();
        for req in batch.reqs {
            let api_ext = self.api_info[&req.id].clone();
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

    use super::*;
    use async_std;

    #[async_std::test]
    async fn it_works() {
        let mut mapping = HashMap::new();
        mapping.insert(
            serde_json::to_string(&Param {
                id: 1,
                name: "a".to_string(),
            })
            .unwrap(),
            vec![Param {
                id: 2,
                name: "b".to_string(),
            }],
        );
        let conf = Config {
            mapping: Some(mapping),
            apis: vec![],
        };
        println!("{:?}", serde_json::to_string(&conf));
    }
}
