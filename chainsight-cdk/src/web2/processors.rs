use std::marker::PhantomData;

use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use serde::de::DeserializeOwned;

use crate::web3::TransformProcessor;

pub const HTTPS_SNAPSHOT_RESPONSE_TRANSFORM_METHOD: &str = "transform_https_response";

pub struct HTTPSResponseTransformProcessor<T> {
    _phantom: PhantomData<T>,
}

impl<T> HTTPSResponseTransformProcessor<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
impl<T> Default for HTTPSResponseTransformProcessor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TransformProcessor for HTTPSResponseTransformProcessor<T>
where
    T: DeserializeOwned + serde::Serialize,
{
    fn process_body(&self, body: &[u8]) -> Vec<u8> {
        let body = serde_json::from_slice::<T>(body);
        serde_json::to_vec(&body.unwrap()).unwrap()
    }
    fn context(&self) -> ic_cdk::api::management_canister::http_request::TransformContext {
        TransformContext {
            function: TransformFunc(candid::Func {
                method: HTTPS_SNAPSHOT_RESPONSE_TRANSFORM_METHOD.to_string(),
                principal: ic_cdk::api::id(),
            }),
            context: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        inner: String,
    }

    #[test]
    fn test_process_body() {
        let input = TestStruct {
            inner: "test".to_string(),
        };
        let i = serde_json::to_vec(&input).unwrap();
        let processor = HTTPSResponseTransformProcessor::<TestStruct>::new();
        let output = processor.process_body(&i);
        let output = serde_json::from_slice::<TestStruct>(&output).unwrap();
        assert_eq!(input.inner, output.inner);
    }
}
