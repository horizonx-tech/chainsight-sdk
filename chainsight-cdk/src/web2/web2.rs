use std::collections::HashMap;

use ic_cdk::api::management_canister::http_request::{
    self, http_request, CanisterHttpRequestArgument, HttpHeader,
};
use serde::de::DeserializeOwned;
pub struct Web2JsonRpcSnapshotIndexer {
    pub url: String,
}

pub struct JsonRpcSnapshotParam {
    pub queries: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

impl Web2JsonRpcSnapshotIndexer {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn get<T, V>(&self, param: JsonRpcSnapshotParam) -> anyhow::Result<V>
    where
        V: DeserializeOwned,
    {
        let headers: Vec<HttpHeader> = param
            .headers
            .iter()
            .map(|(k, v)| HttpHeader {
                name: k.to_string(),
                value: v.to_string(),
            })
            .collect();
        let result = http_request(CanisterHttpRequestArgument {
            url: build_url(self.url.clone().as_str(), param.queries),
            method: http_request::HttpMethod::GET,
            headers,
            max_response_bytes: None,
            transform: None,
            body: None,
        })
        .await
        .expect("http_request failed");
        let res: V = serde_json::from_slice(&result.0.body)?;
        Ok(res)
    }
}

fn build_url(url: &str, queries: HashMap<String, String>) -> String {
    let mut url = url.to_string();
    if !queries.is_empty() {
        url.push_str("?");
        for (k, v) in queries {
            url.push_str(&format!("{}={}&", k, v));
        }
        url.pop();
    }
    url
}
