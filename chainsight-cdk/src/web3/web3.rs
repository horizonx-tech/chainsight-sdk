use std::{collections::HashMap, marker::PhantomData};

use crate::{
    core::Env,
    indexer::{Error, Event, Indexer, IndexingConfig},
    storage::{KeyValuesStore, Persist},
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
pub struct Web3Indexer<E>
where
    E: Event<EventLog>,
{
    _phantom: PhantomData<E>,
    finder: Web3LogFinder,
    storage: KeyValuesStore,
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
    find: FindFunc,
}
impl Web3LogFinder {
    async fn find(&self, args: (u64, u64)) -> Result<HashMap<u64, Vec<EventLog>>, Error> {
        (self.find)(args.0, args.1, self.call_options.clone()).await
    }
}
pub type FindFunc =
    fn(u64, u64, CallOptions) -> BoxFuture<'static, Result<HashMap<u64, Vec<EventLog>>, Error>>;
impl<E> Web3Indexer<E>
where
    E: Event<EventLog> + Persist,
{
    pub fn new(find: FindFunc, call_options: Option<CallOptions>) -> Self {
        Self {
            _phantom: PhantomData,
            finder: Web3LogFinder {
                call_options: match call_options {
                    Some(options) => options,
                    None => Self::default_indexer_call_options(),
                },
                find,
            },
            storage: KeyValuesStore::new(1),
        }
    }
    fn finder(&self) -> Web3LogFinder {
        self.finder.clone()
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
    pub fn between(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<E>>, Error> {
        Ok(self
            .storage
            .between(from.to_string().as_str(), to.to_string().as_str())
            .into_iter()
            .map(|(block_number, tokens)| ((block_number.parse::<u64>().unwrap(), tokens)))
            .fold(HashMap::new(), |mut acc, (block_number, tokens)| {
                acc.insert(block_number, tokens);
                acc
            }))
    }
    fn on_update(&self, logs: HashMap<u64, Vec<EventLog>>) {
        logs.iter().for_each(|(block_number, logs)| {
            let tokens: Vec<E> = logs
                .iter()
                .map(|log| {
                    E::from(EventLog {
                        event: log.event.clone(),
                        log: log.log.clone(),
                    })
                })
                .collect();
            self.storage.set(block_number.to_string().as_str(), tokens)
        })
    }

    pub fn get_last_indexed(&self) -> Result<u64, Error> {
        Ok(self
            .storage
            .last(1)
            .last()
            .map(|(block_number, _)| block_number.parse::<u64>().unwrap())
            .unwrap_or_default())
    }
}

#[async_trait]
impl<E> Indexer<EventLog, HashMap<u64, Vec<EventLog>>, (u64, u64)> for Web3Indexer<E>
where
    E: Event<EventLog> + Persist,
{
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error> {
        let last_indexed = self.get_last_indexed()?;
        let chunk_size = cfg.chunk_size.unwrap_or(500);
        let from = cfg.start_from.max(last_indexed + 1);
        let to = from + chunk_size;
        let elements = self.finder().find((from, to)).await?;
        self.on_update(elements);
        Ok(())
    }
}
