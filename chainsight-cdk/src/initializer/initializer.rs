use core::fmt;

use async_trait::async_trait;
use candid::{CandidType, Deserialize, Principal};

use crate::core::Env;
#[derive(Debug, CandidType)]
pub enum InitError {
    InvalidPrincipal(Principal),
    InvalidRequest(String),
    InvalidContent(String),
    InvalidDestination(String),
}
impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InitError::InvalidPrincipal(p) => write!(f, "Invalid principal: {}", p),
            InitError::InvalidRequest(s) => write!(f, "Invalid request: {}", s),
            InitError::InvalidContent(s) => write!(f, "Invalid content: {}", s),
            InitError::InvalidDestination(s) => write!(f, "Invalid destination: {}", s),
        }
    }
}
impl std::error::Error for InitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InitError::InvalidPrincipal(_) => None,
            InitError::InvalidRequest(_) => None,
            InitError::InvalidContent(_) => None,
            InitError::InvalidDestination(_) => None,
        }
    }
}

#[derive(CandidType, serde::Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct CycleManagement {
    pub initial_value: u128,
    pub refueling_value: u128,
    pub refueling_threshold: u128,
}

#[derive(CandidType, serde::Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct CycleManagements {
    pub refueling_interval: u64,
    pub vault_intial_supply: u128,
    pub indexer: CycleManagement,
    pub db: CycleManagement,
    pub proxy: CycleManagement,
}

impl CycleManagements {
    pub fn initial_supply(&self) -> u128 {
        self.vault_intial_supply
            + self.indexer.initial_value
            + self.db.initial_value
            + self.proxy.initial_value
    }
}

impl Default for CycleManagements {
    fn default() -> Self {
        Self {
            refueling_interval: 86400,
            vault_intial_supply: 1_000_000_000_000,
            indexer: CycleManagement {
                initial_value: 0,
                refueling_value: 1_000_000_000_000,
                refueling_threshold: 500_000_000_000,
            },
            db: CycleManagement {
                initial_value: 2_000_000_000_000,
                refueling_value: 1_00_000_000_000,
                refueling_threshold: 1_000_000_000_000,
            },
            proxy: CycleManagement {
                initial_value: 2_000_000_000,
                refueling_value: 1_000_000_000,
                refueling_threshold: 1_000_000_000,
            },
        }
    }
}

#[async_trait]
pub trait Initializer {
    async fn initialize(
        &self,
        deployer: &Principal,
        cycles: &CycleManagements,
    ) -> Result<InitResult, InitError>;
}
pub struct InitConfig {
    pub env: Env,
}

pub struct InitResult {
    pub proxy: Principal,
    pub db: Principal,
    pub vault: Principal,
}
