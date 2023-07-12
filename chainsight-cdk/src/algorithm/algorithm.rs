use crate::{
    indexer::{Error, Finder, Indexer, IndexingConfig},
    rpc::{CallProvider, Caller, Message},
};
use async_trait::async_trait;
use candid::{CandidType, Principal};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
pub struct AlgorithmIndexer<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    finder: AlgorithmLogFinder,
    persister: AlgorithmEventPersister<T>,
}

#[derive(Clone)]
pub struct AlgorithmEventPersister<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    persist: fn(HashMap<u64, Vec<T>>),
}

impl<T> AlgorithmEventPersister<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    pub fn new(func: fn(HashMap<u64, Vec<T>>)) -> Self {
        Self { persist: func }
    }
}

#[derive(Clone)]
pub struct AlgorithmLogFinder {
    proxy: Principal,
    target: Principal,
}

#[async_trait]
impl<T> Finder<T> for AlgorithmLogFinder
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    async fn find(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<T>>, Error> {
        let results = self.get_logs(from, to).await?;
        Ok(results)
    }
}

impl AlgorithmLogFinder {
    fn new(proxy: Principal, target: Principal) -> Self {
        Self { proxy, target }
    }

    async fn get_logs<T>(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<T>>, Error>
    where
        T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
    {
        let result = CallProvider::new(self.proxy)
            .call(Message::new::<(u64, u64)>((from, to), self.target, "proxy_call").unwrap())
            .await
            .unwrap();
        let rep: HashMap<u64, Vec<T>> = result.reply::<HashMap<u64, Vec<T>>>().unwrap();
        Ok(rep)
    }
}
impl<T> AlgorithmIndexer<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    pub fn new(proxy: Principal, src: Principal, persist: fn(HashMap<u64, Vec<T>>)) -> Self {
        Self {
            finder: AlgorithmLogFinder::new(proxy, src),
            persister: AlgorithmEventPersister::new(persist),
        }
    }
}

impl<T> AlgorithmIndexer<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    /// Index events.
    pub async fn index(&self, cfg: IndexingConfig) -> Result<(), Error> {
        let last_indexed = self.get_last_indexed()?;
        let chunk_size = cfg.chunk_size.unwrap_or(500);
        let from = cfg.start_from.max(last_indexed + 1);
        let to = from + chunk_size;
        let result: HashMap<u64, Vec<T>> = self.finder().find(from, to).await?;
        self.persister.persist.clone()(result);
        Ok(())
    }
}

#[async_trait]
impl<T> Indexer<T> for AlgorithmIndexer<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    type Finder = AlgorithmLogFinder;
    fn finder(&self) -> Self::Finder {
        self.finder.clone()
    }
}
