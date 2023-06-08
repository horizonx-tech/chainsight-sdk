use std::collections::HashMap;

use async_trait::async_trait;
use ic_solidity_bindgen::types::EventLog;

use crate::indexer::{Error, Event, Indexer};
struct Web3Indexer {}

#[async_trait]
impl Indexer<EventLog> for Web3Indexer {
    async fn get_from_to(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<EventLog>>, Error> {
        unimplemented!()
    }

    fn save<E>(&self, id: u64, elements: Vec<E>)
    where
        E: Event,
    {
        unimplemented!()
    }

    fn get_last_number(&self) -> Result<u64, Error> {
        unimplemented!()
    }

    fn get_event_chunk_size(&self) -> Result<u64, Error> {
        unimplemented!()
    }

    fn set_event_chunk_size(&self, size: u64) -> Result<(), Error> {
        unimplemented!()
    }

    fn get_last_indexed(&self) -> Result<u64, Error> {
        unimplemented!()
    }

    fn set_last_indexed(&self, id: u64) -> Result<(), Error> {
        unimplemented!()
    }
}
