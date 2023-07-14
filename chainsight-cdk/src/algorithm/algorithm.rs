use crate::{
    indexer::{Error, Indexer, IndexingConfig},
    rpc::{CallProvider, Caller, Message},
    storage,
};
use async_trait::async_trait;
use candid::{CandidType, Principal};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
pub struct AlgorithmIndexer<Logs> {
    finder: AlgorithmLogFinder,
    persister: AlgorithmEventPersister<Logs>,
}

#[derive(Clone)]
pub struct AlgorithmEventPersister<Logs> {
    persist: fn(Logs),
}

impl<Logs> AlgorithmEventPersister<Logs> {
    pub fn new(func: fn(Logs)) -> Self {
        Self { persist: func }
    }
}

#[derive(Clone)]
pub struct AlgorithmLogFinder {
    proxy: Principal,
    target: Principal,
    method: String,
}

impl AlgorithmLogFinder {
    async fn find<T, Args, Reply>(&self, args: Args) -> Result<Reply, Error>
    where
        T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
        Args: serde::Serialize + Send,
        Reply: serde::de::DeserializeOwned,
    {
        let results = self.get_logs::<T, Args, Reply>(args).await?;
        Ok(results)
    }
}

impl AlgorithmLogFinder {
    fn new(proxy: Principal, target: Principal) -> Self {
        Self {
            proxy,
            target,
            method: "proxy_call".to_string(),
        }
    }
    fn new_with_method(proxy: Principal, target: Principal, method: &str) -> Self {
        Self {
            proxy,
            target,
            method: method.to_string(),
        }
    }

    async fn get_logs<T, Args, Reply>(&self, args: Args) -> Result<Reply, Error>
    where
        T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
        Args: serde::Serialize,
        Reply: serde::de::DeserializeOwned,
    {
        let result = CallProvider::new(self.proxy)
            .call(Message::new::<Args>(args, self.target, &self.method).unwrap())
            .await
            .unwrap();
        let rep = result.reply::<Reply>().unwrap();
        Ok(rep)
    }
}
impl<Logs> AlgorithmIndexer<Logs> {
    pub fn new(proxy: Principal, src: Principal, persist: fn(Logs)) -> Self {
        Self {
            finder: AlgorithmLogFinder::new(proxy, src),
            persister: AlgorithmEventPersister::new(persist),
        }
    }
    pub fn new_with_method(
        proxy: Principal,
        src: Principal,
        method: &str,
        persist: fn(Logs),
    ) -> Self {
        Self {
            finder: AlgorithmLogFinder::new_with_method(proxy, src, method),
            persister: AlgorithmEventPersister::new(persist),
        }
    }
}

#[async_trait]
impl<T> Indexer<T, HashMap<u64, Vec<T>>, (u64, u64)> for AlgorithmIndexer<HashMap<u64, Vec<T>>>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error> {
        let last_indexed = cfg.start_from;
        let chunk_size = cfg.chunk_size.unwrap_or(500);
        let from = cfg.start_from.max(last_indexed + 1);
        let to = from + chunk_size;
        ic_cdk::println!("from: {}, to: {}", from, to);
        let result: HashMap<u64, Vec<T>> = self
            .finder
            .find::<T, (u64, u64), HashMap<u64, Vec<T>>>((from, to))
            .await?;
        ic_cdk::println!("{:?}", result.len());
        self.persister.persist.clone()(result.clone());
        // get last result and update last indexed
        let last_indexed = result.keys().max();
        if let Some(last_indexed) = last_indexed {
            storage::set_last_key(last_indexed.to_string());
        }

        Ok(())
    }
}

#[async_trait]
impl<T> Indexer<T, HashMap<String, Vec<T>>, (String, String)>
    for AlgorithmIndexer<HashMap<String, Vec<T>>>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error> {
        let last_indexed = cfg.start_from;
        let chunk_size = cfg.chunk_size.unwrap_or(500);
        let from = cfg.start_from.max(last_indexed + 1);
        let to = from + chunk_size;
        ic_cdk::println!("from: {}, to: {}", from, to);

        let result: HashMap<String, Vec<T>> = self
            .finder
            .find::<T, (String, String), HashMap<String, Vec<T>>>((
                from.to_string(),
                to.to_string(),
            ))
            .await?;
        ic_cdk::println!("{:?}", result.len());
        self.persister.persist.clone()(result.clone());
        // get last result and update last indexed
        let last_indexed = result.keys().max();
        if let Some(last_indexed) = last_indexed {
            storage::set_last_key(last_indexed.to_string());
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Indexer<T, Vec<T>, (String, String)> for AlgorithmIndexer<Vec<T>>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    async fn index(&self, cfg: IndexingConfig) -> Result<(), Error> {
        let last_indexed = cfg.start_from;
        let chunk_size = cfg.chunk_size.unwrap_or(500);
        let from = cfg.start_from.max(last_indexed + 1);
        let to = from + chunk_size;
        ic_cdk::println!("from: {}, to: {}", from, to);

        let result: Vec<T> = self
            .finder
            .find::<T, (String, String), Vec<T>>((from.to_string(), to.to_string()))
            .await?;
        ic_cdk::println!("{:?}", result.len());
        self.persister.persist.clone()(result);
        Ok(())
    }
}
