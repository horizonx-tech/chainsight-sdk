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

impl Parse for IndexingConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let mut indexing = IndexingConfig::default();
        let start_from = input.parse::<syn::LitInt>()?;
        indexing.start_from = start_from.base10_parse()?;
        let chunk_size = input.parse::<syn::LitInt>()?;
        indexing.chunk_size = Some(chunk_size.base10_parse()?);
        Ok(indexing)
    }
}

pub trait Event<T>: CandidType + Send + Sync + Clone + From<T> + Persist + 'static {
    fn tokenize(&self) -> Data;
    fn untokenize(data: Data) -> Self;
}

#[async_trait]
pub trait Indexer<Log, Events, Args> {
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error>;
}
