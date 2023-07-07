use std::{cell::RefCell, collections::HashMap};

use crate::{
    indexer::{Error, Finder, Indexer, IndexingConfig},
    rpc::{CallProvider, Caller, Message},
};
use async_trait::async_trait;
use candid::{CandidType, Principal};
use futures::{future::BoxFuture, FutureExt};
use serde::de::DeserializeOwned;
pub struct AlgorithmIndexer<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    finder: AlgorithmLogFinder<T>,
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
pub struct AlgorithmLogFinder<T> {
    find: fn(u64, u64) -> BoxFuture<'static, Result<HashMap<u64, Vec<T>>, Error>>,
}

thread_local! {
    static PROXY: RefCell<Principal> = RefCell::new(Principal::anonymous());
    static SOURCE: RefCell<Principal> = RefCell::new(Principal::anonymous());
}

fn proxy() -> Principal {
    PROXY.with(|p| p.borrow().clone())
}
fn source() -> Principal {
    SOURCE.with(|p| p.borrow().clone())
}
fn set_source(p: Principal) {
    SOURCE.with(|s| *s.borrow_mut() = p);
}
fn set_proxy(p: Principal) {
    PROXY.with(|s| *s.borrow_mut() = p);
}

#[async_trait]
impl<T> Finder<T> for AlgorithmLogFinder<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    async fn find(&self, from: u64, to: u64) -> Result<HashMap<u64, Vec<T>>, Error> {
        let results = (self.find)(from, to).await?;
        Ok(results)
    }
}
impl<T> AlgorithmIndexer<T>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    pub fn new(proxy: Principal, src: Principal, persister: AlgorithmEventPersister<T>) -> Self {
        set_proxy(proxy);
        set_source(src);
        Self {
            finder: AlgorithmLogFinder {
                find: get_logs::<T>,
            },
            persister: persister,
        }
    }
}
fn get_logs<T>(from: u64, to: u64) -> BoxFuture<'static, Result<HashMap<u64, Vec<T>>, Error>>
where
    T: CandidType + Send + Sync + Clone + DeserializeOwned + 'static,
{
    async move {
        let result = CallProvider::new(proxy())
            .call(Message::new::<(u64, u64)>((from, to), source(), "between").unwrap())
            .await
            .unwrap();
        let rep = result.reply::<HashMap<u64, Vec<T>>>().unwrap();
        Ok(rep)
    }
    .boxed()
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
    type Finder = AlgorithmLogFinder<T>;
    fn finder(&self) -> Self::Finder {
        self.finder.clone()
    }

    /// Index events.
    async fn index<E>(&self, config: IndexingConfig) -> Result<(), Error> {
        self.index(config).await
    }
}
