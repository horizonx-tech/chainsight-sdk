use std::{borrow::Cow, cell::RefCell, collections::HashMap};

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, VirtualMemory};
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::{
    memory_manager::MemoryManager, BoundedStorable, DefaultMemoryImpl, Storable,
};
use serde::Deserialize;

use super::token::Token;
type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Deserialize, CandidType, Clone)]
pub struct Data {
    values: HashMap<String, Token>,
}

impl Storable for Data {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl BoundedStorable for Data {
    const MAX_SIZE: u32 = 100_000;
    const IS_FIXED_SIZE: bool = false;
}

impl Data {
    pub fn get(&self, key: &str) -> Option<&Token> {
        self.values.get(key)
    }
}

thread_local! {
    static MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static MAP: RefCell<StableBTreeMap<u64, Data, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(0))),
        )
    )
}

pub fn get(key: u64) -> Option<Data> {
    MAP.with(|m| m.borrow().get(&key))
}

pub fn insert(key: u64) -> Option<Data> {
    MAP.with(|m| {
        m.borrow_mut().insert(
            key,
            Data {
                values: HashMap::new(),
            },
        )
    })
}

// get last n
pub fn last(n: u64) -> Vec<(u64, Data)> {
    let length = MAP.with(|m| m.borrow().len());
    if length <= n {
        MAP.with(|m| m.borrow().iter().map(|(k, v)| (k, v.clone())).collect())
    } else {
        MAP.with(|m| {
            m.borrow()
                .iter()
                .skip((length - n) as usize)
                .map(|(k, v)| (k, v.clone()))
                .collect()
        })
    }
}
