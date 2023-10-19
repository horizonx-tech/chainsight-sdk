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

pub trait Persist {
    fn untokenize(data: Data) -> Self;
    fn tokenize(&self) -> Data;
}

#[derive(Deserialize, CandidType, Clone)]
pub struct Values(Vec<Data>);
#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct Data(HashMap<String, Token>);

#[derive(Deserialize, CandidType, Clone, Hash, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Id(String);
impl Id {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl Storable for Id {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Id {
    const MAX_SIZE: u32 = 100_000;
    const IS_FIXED_SIZE: bool = false;
}

impl Data {
    pub fn new(val: HashMap<String, Token>) -> Self {
        Self(val)
    }
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
    );
    static KEY_VALUE_STORE_1: RefCell<StableBTreeMap<Id, Data, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(1))),
        )
    );
    static KEY_VALUE_STORE_2: RefCell<StableBTreeMap<Id, Data, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(2))),
        )
    );
    static KEY_VALUE_STORE_3: RefCell<StableBTreeMap<Id, Data, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(3))),
        )
    );
    static KEY_VALUE_STORE_4: RefCell<StableBTreeMap<Id, Data, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(4))),
        )
    );
    static KEY_VALUE_STORE_5: RefCell<StableBTreeMap<Id, Data, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(5))),
        )
    );

    static KEY_VALUES_STORE_1: RefCell<StableBTreeMap<Id, Values, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(6))),
        )
    );
    static KEY_VALUES_STORE_2: RefCell<StableBTreeMap<Id, Values, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(7))),
        )
    );
    static KEY_VALUES_STORE_3: RefCell<StableBTreeMap<Id, Values, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(8))),
        )
    );
    static KEY_VALUES_STORE_4: RefCell<StableBTreeMap<Id, Values, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(9))),
        )
    );
    static KEY_VALUES_STORE_5: RefCell<StableBTreeMap<Id, Values, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MANAGER.with(|m|m.borrow().get(MemoryId::new(10))),
        )
    );
    static LAST_KEY_STORE: RefCell<String> = RefCell::new(String::new());

}

pub struct KeyValuesStore {
    store: &'static std::thread::LocalKey<RefCell<StableBTreeMap<Id, Values, Memory>>>,
}

pub fn set_last_key(key: String) {
    LAST_KEY_STORE.with(|m| {
        *m.borrow_mut() = key;
    });
}
pub fn get_last_key() -> String {
    LAST_KEY_STORE.with(|m| m.borrow().clone())
}

impl KeyValuesStore {
    pub fn new(mem_id: u8) -> Self {
        assert!(mem_id > 0 && mem_id < 6);
        match mem_id {
            1 => Self {
                store: &KEY_VALUES_STORE_1,
            },
            2 => Self {
                store: &KEY_VALUES_STORE_2,
            },
            3 => Self {
                store: &KEY_VALUES_STORE_3,
            },
            4 => Self {
                store: &KEY_VALUES_STORE_4,
            },
            5 => Self {
                store: &KEY_VALUES_STORE_5,
            },
            _ => panic!("Invalid store id"),
        }
    }
    pub fn get<T>(&self, id: &str) -> Vec<T>
    where
        T: Persist,
    {
        self.store.with(|m| {
            m.borrow()
                .get(&Id(id.to_string()))
                .map(|v| {
                    let elems: Vec<T> = v.0.iter().map(|e| T::untokenize(e.clone())).collect();
                    elems
                })
                .unwrap_or_default()
        })
    }
    pub fn set<T>(&self, id: &str, values: Vec<T>)
    where
        T: Persist,
    {
        let values: Vec<Data> = values.into_iter().map(|v| v.tokenize()).collect();
        self.store.with(|m| {
            m.borrow_mut().insert(Id(id.to_string()), Values(values));
        })
    }
    pub fn between<T>(&self, from: &str, to: &str) -> HashMap<String, Vec<T>>
    where
        T: Persist,
    {
        self.store.with(|m| {
            m.borrow()
                .range(Id(from.to_string())..Id(to.to_string()))
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.0.iter().map(|e| T::untokenize(e.clone())).collect(),
                    )
                })
                .fold(HashMap::new(), |mut acc, (k, v)| {
                    acc.insert(k.0, v);
                    acc
                })
        })
    }
    pub fn last(&self, n: u64) -> Vec<(String, Values)> {
        let length = self.store.with(|m| m.borrow().len());
        let skip = if length <= n { 0 } else { length - n };
        self.store
            .with(|m| {
                m.borrow()
                    .iter()
                    .skip(skip as usize)
                    .map(|(k, v)| (k.clone().0, v.clone()))
                    .collect::<Vec<_>>()
            })
            .into_iter()
            .collect()
    }

    pub fn last_elems<T>(&self, n: u64) -> HashMap<String, Vec<T>>
    where
        T: Persist,
    {
        let length = self.store.with(|m| m.borrow().len());
        let mut result = HashMap::new();
        self.store.with(|m| {
            let mut processed = 0;
            for i in 0..length {
                let idx = length - i - 1;
                let elems = m.borrow().iter().nth(idx as usize);
                if let Some((k, v)) = elems {
                    let elems_len = v.0.len() as u64;
                    let elems = v.0;
                    if processed + elems_len > n {
                        let elems = elems
                            .iter()
                            .skip((elems_len - (n - processed)) as usize)
                            .map(|e| T::untokenize(e.clone()))
                            .collect::<Vec<_>>();
                        result.insert(k.clone().0, elems);
                        break;
                    } else {
                        result.insert(
                            k.clone().0,
                            elems
                                .iter()
                                .map(|e| T::untokenize(e.clone()))
                                .collect::<Vec<_>>(),
                        );
                        processed += elems.len() as u64;
                    }
                } else {
                    break;
                }
            }
        });
        result
    }
}

pub struct KeyValueStore {
    store: &'static std::thread::LocalKey<RefCell<StableBTreeMap<Id, Data, Memory>>>,
}

impl KeyValueStore {
    pub fn new(mem_id: u8) -> Self {
        assert!(mem_id > 0 && mem_id < 6);
        match mem_id {
            1 => Self {
                store: &KEY_VALUE_STORE_1,
            },
            2 => Self {
                store: &KEY_VALUE_STORE_2,
            },
            3 => Self {
                store: &KEY_VALUE_STORE_3,
            },
            4 => Self {
                store: &KEY_VALUE_STORE_4,
            },
            5 => Self {
                store: &KEY_VALUE_STORE_5,
            },
            _ => panic!("Invalid store id"),
        }
    }
    pub fn get<T>(&self, id: &str) -> Option<T>
    where
        T: Persist,
    {
        self.store
            .with(|m| m.borrow().get(&Id(id.to_string())).map(T::untokenize))
    }
    pub fn set<T>(&self, id: &str, data: T)
    where
        T: Persist,
    {
        self.store.with(|m| {
            m.borrow_mut().insert(Id(id.to_string()), data.tokenize());
        })
    }
    pub fn between<T>(&self, from: &str, to: &str) -> Vec<(String, T)>
    where
        T: Persist,
    {
        self.store.with(|m| {
            m.borrow()
                .range(Id(from.to_string())..Id(to.to_string()))
                .map(|(k, v)| (k.clone().0, T::untokenize(v.clone())))
                .collect()
        })
    }
    pub fn last<T>(&self, n: u64) -> Vec<(String, T)>
    where
        T: Persist,
    {
        let length = self.store.with(|m| m.borrow().len());
        let skip = if length <= n { 0 } else { length - n };
        self.store
            .with(|m| {
                m.borrow()
                    .iter()
                    .skip(skip as usize)
                    .map(|(k, v)| (k.clone().0, T::untokenize(v.clone())))
                    .collect::<Vec<_>>()
            })
            .into_iter()
            .collect()
    }
}
