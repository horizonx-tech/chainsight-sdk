use async_trait::async_trait;
use candid::CandidType;
use derive_more::Display;
use serde::Deserialize;
use syn::parse::{Parse, ParseStream};

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

#[derive(CandidType, Clone, Debug, Default, PartialEq, Deserialize)]
pub struct IndexingConfig {
    pub start_from: u64,
    pub chunk_size: Option<u64>,
}

pub trait Event<T>: CandidType + Send + Sync + Clone + From<T> + Persist + 'static {
    fn tokenize(&self) -> Data;
    fn untokenize(data: Data) -> Self;
}

#[async_trait]
pub trait Indexer<Log, Events, Args> {
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error>;
}
