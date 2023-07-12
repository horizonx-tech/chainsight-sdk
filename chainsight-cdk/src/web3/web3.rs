use std::collections::HashMap;

use crate::{
    core::Env,
    indexer::{Error, Finder, Indexer},
};
use async_trait::async_trait;
use candid::CandidType;
use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_solidity_bindgen::types::EventLog;
use ic_web3_rs::{
    futures::future::BoxFuture,
    transports::ic_http_client::{CallOptions, CallOptionsBuilder},
};
use serde::Deserialize;
pub struct Web3Indexer {
    finder: Web3LogFinder,
}

#[derive(Default, Clone, Debug, PartialEq, CandidType, Deserialize)]
pub struct Web3CtxParam {
    pub url: String,
    pub from: Option<String>,
    pub chain_id: u64,
    pub env: Env,
}

#[derive(Clone)]
pub struct Web3LogFinder {
    call_options: CallOptions,
    find:
        fn(u64, u64, CallOptions) -> BoxFuture<'static, Result<HashMap<u64, Vec<EventLog>>, Error>>,
}
#[async_trait]
impl Finder<EventLog> for Web3LogFinder {
    async fn find(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<EventLog>>, Error> {
        (self.find)(from, to, self.call_options.clone()).await
    }
}

impl Web3Indexer {
    pub fn new(
        find: fn(
            u64,
            u64,
            CallOptions,
        ) -> BoxFuture<'static, Result<HashMap<u64, Vec<EventLog>>, Error>>,

        call_options: Option<CallOptions>,
    ) -> Self {
        Self {
            finder: Web3LogFinder {
                call_options: match call_options {
                    Some(options) => options,
                    None => Web3Indexer::default_indexer_call_options(),
                },
                find,
            },
        }
    }
    fn default_indexer_call_options() -> CallOptions {
        CallOptionsBuilder::default()
            .max_resp(None)
            .cycles(None)
            .transform(Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform_get_filter_changes".to_string(),
                }),
                context: vec![],
            }))
            .build()
            .unwrap()
    }
}

#[async_trait]
impl Indexer<EventLog> for Web3Indexer {
    type Finder = Web3LogFinder;
    fn finder(&self) -> Self::Finder {
        self.finder.clone()
    }
}
