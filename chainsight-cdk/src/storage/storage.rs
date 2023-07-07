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
pub struct Values(Vec<Data>);
#[derive(Deserialize, CandidType, Clone)]

pub struct Data(HashMap<String, Token>);

impl Data {
    pub fn new(val: HashMap<String, Token>) -> Self {
        Self(val)
    }
}

impl Storable for Values {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl BoundedStorable for Values {
    const MAX_SIZE: u32 = 100_000;
    const IS_FIXED_SIZE: bool = false;
}

impl Values {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn append(&mut self, data: Data) {
        self.0.push(data);
    }
    pub fn to_vec(&self) -> Vec<Data> {
        self.0.clone()
    }
}

impl Data {
    pub fn get(&self, key: &str) -> Option<&Token> {
        self.0.get(key)
    }
}

thread_local! {
    static MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static MAP: RefCell<StableBTreeMap<u64, Values, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(0))),
        )
    )
}

pub fn insert(key: u64, data: Data) {
    MAP.with(|m| {
        m.borrow_mut()
            .get(&key)
            .get_or_insert(Values::new())
            .append(data)
    })
}

pub fn between(from: u64, to: u64) -> Vec<(u64, Values)> {
    MAP.with(|m| {
        m.borrow()
            .range(from..to)
            .map(|(k, v)| (k, v.clone()))
            .collect()
    })
}

// get last n
pub fn last(n: u64) -> Vec<(u64, Values)> {
    let length = MAP.with(|m| m.borrow().len());
    if length <= n {
        MAP.with(|m| m.borrow().iter().map(|(k, v)| (k, v.clone())).collect())
    } else {
        MAP.with(
            |m: &RefCell<
                StableBTreeMap<u64, Values, VirtualMemory<std::rc::Rc<RefCell<Vec<u8>>>>>,
            >| {
                m.borrow()
                    .iter()
                    .skip((length - n) as usize)
                    .map(|(k, v)| (k, v.clone()))
                    .collect()
            },
        )
    }
}
