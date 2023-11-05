use std::collections::HashMap;

use ic_cdk::api::management_canister::http_request::{
    self, http_request, CanisterHttpRequestArgument, HttpHeader,
};
use serde::de::DeserializeOwned;
pub struct Web2HttpsSnapshotIndexer {
    pub url: String,
}

pub struct HttpsSnapshotParam {
    pub queries: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

impl Web2HttpsSnapshotIndexer {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn get<T, V>(&self, param: HttpsSnapshotParam) -> anyhow::Result<V>
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

pub fn build_url(url: &str, queries: HashMap<String, String>) -> String {
    let mut url = url.to_string();
    if !queries.is_empty() {
        url.push('?');
        let mut queries_vec: Vec<(String, String)> = queries.into_iter().collect();
        queries_vec.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in queries_vec {
            url.push_str(&format!("{}={}&", k, v));
        }
        url.pop();
    }
    url
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::build_url;

    #[test]
    fn test_build_url() {
        let url = "https://api.coingecko.com/api/v3/simple/price";
        let mut queries = HashMap::new();
        queries.insert("vs_currencies".to_string(), "usd,eth".to_string());
        queries.insert("ids".to_string(), "dai".to_string());

        assert_eq!(
            build_url(url, queries),
            "https://api.coingecko.com/api/v3/simple/price?ids=dai&vs_currencies=usd,eth"
        );
    }
}
