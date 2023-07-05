use std::{cell::RefCell, collections::HashMap};

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

thread_local! {
    static CHUNK_SIZE: RefCell<u64> = RefCell::new(500);
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
            storage::insert(id, data);
        });
    }
    /// Get the event chunk size.
    fn get_event_chunk_size(&self) -> Result<u64, Error> {
        Ok(CHUNK_SIZE.with(|c| *c.borrow()))
    }
    /// Set the event chunk size.
    fn set_event_chunk_size(&self, size: u64) -> Result<(), Error> {
        CHUNK_SIZE.with(|c| *c.borrow_mut() = size);
        Ok(())
    }
    fn between<E>(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<E>>, Error>
    where
        E: Event<T>,
    {
        let mut res = HashMap::new();
        storage::between(from, to).into_iter().for_each(|e| {
            let data =
                e.1.to_vec()
                    .iter()
                    .map(|e| Event::untokenize(e.clone()))
                    .collect::<Vec<E>>();
            res.insert(e.0, data);
        });
        Ok(res)
    }
    /// Get the last indexed event id.
    fn get_last_indexed(&self) -> Result<u64, Error> {
        Ok(storage::last(1).iter().map(|e| e.0).max().unwrap_or(0))
    }
    /// Index events.
    async fn index<E>(&self) -> Result<(), Error>
    where
        E: Event<T> + From<T>,
    {
        let last_indexed = self.get_last_indexed()?;
        let chunk_size = self.get_event_chunk_size()?;
        let from = last_indexed + 1;
        let to = last_indexed + chunk_size;
        let elements = self.finder().find(from, to).await?;
        self.on_update::<E>(elements);
        Ok(())
    }

    fn on_update<E>(&self, elements: HashMap<u64, Vec<T>>)
    where
        E: Event<T> + From<T>,
    {
        elements
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(Event::from).collect::<Vec<E>>()))
            .for_each(|(k, v)| {
                self.save(k, v);
            });
    }
}
