use std::collections::HashMap;

use ic_cdk::api::{
    call::CallResult,
    management_canister::http_request::{
        self, http_request, CanisterHttpRequestArgument, HttpHeader, HttpResponse,
    },
};
use serde::de::DeserializeOwned;

use super::HTTPSResponseTransformProcessor;
pub struct Web2HttpsSnapshotIndexer {
    pub url: String,
    retry_strategy: RetryStrategy,
}

#[derive(Debug, Clone, Copy)]
struct RetryStrategy {
    max_retries: usize,
    // delay: u64,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            //            delay: 1000,
        }
    }
}

pub struct HttpsSnapshotParam {
    pub queries: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

async fn retry<Fut, F: FnMut() -> Fut>(
    strategy: RetryStrategy,
    mut f: F,
) -> CallResult<(HttpResponse,)>
where
    Fut: std::future::Future<Output = CallResult<(HttpResponse,)>>,
{
    let mut count = 0;
    loop {
        let res = f().await;
        match res {
            Ok(res) => return Ok(res),
            Err(err) => {
                count += 1;
                if count > strategy.max_retries {
                    return Err(err);
                }
                //let back_off = strategy.delay * count as u64; //TODO: add backoff
                //std::thread::sleep(std::time::Duration::from_secs(back_off));
            }
        }
    }
}

impl Web2HttpsSnapshotIndexer {
    pub fn new(url: String) -> Self {
        Self {
            url,
            retry_strategy: RetryStrategy::default(),
        }
    }

    pub async fn get<T, V>(&self, param: HttpsSnapshotParam) -> anyhow::Result<V>
    where
        V: DeserializeOwned + serde::Serialize,
    {
        use crate::web3::processors::TransformProcessor;
        let headers: Vec<HttpHeader> = param
            .headers
            .iter()
            .map(|(k, v)| HttpHeader {
                name: k.to_string(),
                value: v.to_string(),
            })
            .collect();
        let arg = CanisterHttpRequestArgument {
            url: build_url(self.url.clone().as_str(), param.queries),
            method: http_request::HttpMethod::GET,
            headers,
            max_response_bytes: None,
            transform: Some(HTTPSResponseTransformProcessor::<V>::new().context()),
            body: None,
        };
        let cycles = http_request_required_cycles(&arg);
        let result = retry(self.retry_strategy, || http_request(arg.clone(), cycles))
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

// Calcurate cycles for http_request
// NOTE:
//   v0.11: https://github.com/dfinity/cdk-rs/blob/0b14facb80e161de79264c8f88b1a0c8e18ffcb6/examples/management_canister/src/caller/lib.rs#L7-L19
//   v0.8: https://github.com/dfinity/cdk-rs/blob/a8454cb37420c200c7b224befd6f68326a01442e/src/ic-cdk/src/api/management_canister/http_request.rs#L290-L299
fn http_request_required_cycles(arg: &CanisterHttpRequestArgument) -> u128 {
    let max_response_bytes = match arg.max_response_bytes {
        Some(ref n) => *n as u128,
        None => 2 * 1024 * 1024u128, // default 2MiB
    };
    let arg_raw = candid::utils::encode_args((arg,)).expect("Failed to encode arguments.");
    // The fee is for a 13-node subnet to demonstrate a typical usage.
    (3_000_000u128
        + 60_000u128 * 13
        + (arg_raw.len() as u128 + "http_request".len() as u128) * 400
        + max_response_bytes * 800)
        * 13
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
