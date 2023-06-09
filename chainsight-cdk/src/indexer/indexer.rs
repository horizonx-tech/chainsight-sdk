use std::collections::HashMap;

use async_trait::async_trait;
use candid::CandidType;
use derive_more::Display;

use crate::storage::{self, Data};

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

pub trait Event<T>: CandidType + Send + Sync + Clone + 'static {
    fn from(event: T) -> Self
    where
        T: Into<Self>;
    fn tokenize(&self) -> Data;
    fn untokenize(data: Data) -> Self;
}

#[async_trait]
pub trait Finder<T>: Send + Sync + Clone + 'static {
    /// Get events from `from` to `to`. e.g. `from = 1`, `to = 10` will return events with id from 1 to 10.
    async fn find(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<T>>, Error>;
}

#[async_trait]
pub trait Indexer<T> {
    type Finder: Finder<T>;
    fn finder(&self) -> Self::Finder;
    /// Save events to database.
    fn save<E>(&self, id: u64, elements: Vec<E>)
    where
        E: Event<T>,
    {
        elements.iter().map(|e| e.tokenize()).for_each(|data| {
            storage::insert(id, data).unwrap();
        });
    }
    /// Get the latest event id.
    fn get_last_number(&self) -> Result<u64, Error>;
    /// Get the event chunk size.
    fn get_event_chunk_size(&self) -> Result<u64, Error>;
    /// Set the event chunk size.
    fn set_event_chunk_size(&self, size: u64) -> Result<(), Error>;
    /// Get the last indexed event id.
    fn get_last_indexed(&self) -> Result<u64, Error>;
    /// Set the last indexed event id.
    fn set_last_indexed(&self, id: u64) -> Result<(), Error>;
    /// Index events.
    async fn index<E>(&self) -> Result<(), Error>
    where
        E: Event<T> + From<T>,
    {
        let last_indexed = self.get_last_indexed()?;
        let chunk_size = self.get_event_chunk_size()?;
        let from = last_indexed + 1;
        let to = last_indexed + chunk_size;
        self.finder()
            .find(from, to)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(Event::from).collect::<Vec<E>>()))
            .for_each(|(k, v)| {
                self.save(k, v);
            });
        self.set_last_indexed(to)?;
        Ok(())
    }
}
