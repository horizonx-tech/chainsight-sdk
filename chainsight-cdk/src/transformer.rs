use ic_solidity_bindgen::types::EventLog;

use crate::indexer::Event;

pub trait Transformer {
    fn transform<T>(&self, log: EventLog) -> T
    where
        T: Event;
}
