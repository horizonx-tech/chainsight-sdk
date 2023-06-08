use std::collections::HashMap;

use async_trait::async_trait;
use candid::CandidType;
use derive_more::{Display, From};

#[derive(Debug, Display, From)]
pub enum Error {}

pub trait Event: CandidType + Send + Sync + Clone + 'static {
    /// Create an event from a log.
    fn from<T>(log: T) -> Self;
}

#[async_trait]
pub trait Indexer {
    /// Get events from `from` to `to`. e.g. `from = 1`, `to = 10` will return events with id from 1 to 10.
    async fn get_from_to<U>(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<U>>, Error>;
    /// Save events to database.
    fn save<T>(&self, id: u64, elements: Vec<T>)
    where
        T: Event;
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
    async fn index<T, U>(&self) -> Result<(), Error>
    where
        T: Event,
    {
        let last_indexed = self.get_last_indexed()?;
        let chunk_size = self.get_event_chunk_size()?;
        let from = last_indexed + 1;
        let to = last_indexed + chunk_size;
        self.get_from_to(from, to)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(Event::from::<U>).collect::<Vec<T>>()))
            .for_each(|(k, v)| {
                self.save(k, v);
            });
        self.set_last_indexed(to)?;
        Ok(())
    }
}
