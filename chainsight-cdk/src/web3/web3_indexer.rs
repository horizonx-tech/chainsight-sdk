use std::{collections::HashMap, future::Future};

use async_trait::async_trait;
use ic_solidity_bindgen::types::EventLog;
use ic_web3_rs::transports::ic_http_client::CallOptions;

use crate::indexer::{Error, Event, Indexer};
#[derive(Clone)]
struct Web3Indexer<F>
where
    F: Future<Output = Result<HashMap<u64, Vec<EventLog>>, ic_web3_rs::error::Error>>,
{
    get_logs_fn: fn(from: u64, to: u64, options: CallOptions) -> F,
    call_options: CallOptions,
}

#[async_trait]
impl<F> Indexer<EventLog> for Web3Indexer<F>
where
    F: Future<Output = Result<HashMap<u64, Vec<EventLog>>, ic_web3_rs::error::Error>> + Send,
{
    async fn get_from_to(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<EventLog>>, Error> {
        let f = self.get_logs_fn;
        f(from, to, self.call_options.clone())
            .await
            .map_err(|e| Error::Web3Error(e.to_string()))
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
