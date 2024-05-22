use std::borrow::Cow;

use async_trait::async_trait;
use candid::{CandidType, Decode, Encode};
use derive_more::Display;
use ic_stable_structures::storable::Bound;
use serde::Deserialize;

use crate::storage::{Data, Persist};

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Indexer error: {}", _0)]
    IndexerError(String),
    #[display(fmt = "Web3 error: {}", _0)]
    Web3Error(String),
    #[display(fmt = "Database error: {}", _0)]
    DatabaseError(String),
    #[display(fmt = "Other error: {}", _0)]
    OtherError(String),
}

#[derive(CandidType, Clone, Debug, Default, PartialEq, Deserialize, serde::Serialize)]
pub struct IndexingConfig {
    pub start_from: u64,
    pub chunk_size: Option<u64>,
}
impl ic_stable_structures::Storable for IndexingConfig {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,        // temp
        is_fixed_size: false, // temp
    };
}

pub trait Event<T>: CandidType + Send + Sync + Clone + From<T> + Persist + 'static {
    fn tokenize(&self) -> Data;
    fn untokenize(data: Data) -> Self;
}

#[async_trait]
pub trait Indexer<Log, Events, Args> {
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error>;
}
