use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use crate::{
    core::Env,
    indexer::{Error, Event, Indexer, IndexingConfig},
    storage::{KeyValuesStore, Persist},
};
use async_trait::async_trait;
use candid::{CandidType, Decode, Encode};
use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_solidity_bindgen::types::EventLog;
use ic_stable_structures::{BoundedStorable, Storable};
use ic_web3_rs::{
    futures::future::BoxFuture,
    transports::ic_http_client::{CallOptions, CallOptionsBuilder},
};
use serde::{Deserialize, Serialize};
pub struct Web3Indexer<E>
where
    E: Event<EventLog>,
{
    _phantom: PhantomData<E>,
    finder: Web3LogFinder,
    storage: KeyValuesStore,
}

#[derive(Default, Clone, Debug, PartialEq, CandidType, Serialize, Deserialize)]
pub struct Web3CtxParam {
    pub url: String,
    pub from: Option<String>,
    pub chain_id: u64,
    pub env: Env,
}
impl Storable for Web3CtxParam {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl BoundedStorable for Web3CtxParam {
    const MAX_SIZE: u32 = 128;
    const IS_FIXED_SIZE: bool = false;
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
            .map(|(block_number, tokens)| (block_number.parse::<u64>().unwrap(), tokens))
            .fold(HashMap::new(), |mut acc, (block_number, tokens)| {
                acc.insert(block_number, tokens);
                acc
            }))
    }
    pub fn on_update(&self, logs: HashMap<u64, Vec<EventLog>>) {
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
        let last = self.storage.last();
        if let Some(last) = last {
            Ok(last.0.parse::<u64>().unwrap())
        } else {
            Err(Error::OtherError("No last indexed".to_string()))
        }
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

#[cfg(test)]
mod tests {
    use std::pin::Pin;

    use futures::Future;

    use crate::storage::{Data, Token};

    use super::*;
    #[derive(Default, Clone, Debug, PartialEq, CandidType, Serialize, Deserialize)]
    struct SampleStruct {
        value: String,
    }
    impl Event<EventLog> for SampleStruct {
        fn tokenize(&self) -> Data {
            Data::new(
                vec![("value".to_string(), Token::from(self.value.clone()))]
                    .into_iter()
                    .collect(),
            )
        }
        fn untokenize(data: Data) -> Self {
            Self {
                value: data.get("value").unwrap().to_string(),
            }
        }
    }
    impl From<EventLog> for SampleStruct {
        fn from(_: EventLog) -> Self {
            Self {
                value: "dummy".to_string(),
            }
        }
    }
    impl Persist for SampleStruct {
        fn tokenize(&self) -> crate::storage::Data {
            Data::new(
                vec![("value".to_string(), Token::from(self.value.clone()))]
                    .into_iter()
                    .collect(),
            )
        }
        fn untokenize(data: Data) -> Self {
            Self {
                value: data.get("value").unwrap().to_string(),
            }
        }
    }
    #[test]
    fn test_indexer_between_empty() {
        let indexer = Web3Indexer::<SampleStruct>::new(
            |_from, _to, _call_options| {
                let result = HashMap::new();
                Box::pin(async move { Ok(result) })
                    as Pin<
                        Box<dyn Future<Output = Result<HashMap<u64, Vec<EventLog>>, Error>> + Send>,
                    >
            },
            Some(CallOptions::default()),
        );
        assert_eq!(indexer.between(0, 100).unwrap(), HashMap::new());
    }
}
