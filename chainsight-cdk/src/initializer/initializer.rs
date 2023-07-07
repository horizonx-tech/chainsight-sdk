use core::fmt;

use async_trait::async_trait;
use candid::{CandidType, Principal};

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

#[async_trait]
pub trait Initializer {
    async fn initialize(&self) -> Result<InitResult, InitError>;
}
pub struct InitConfig {
    pub env: Env,
}

pub struct InitResult {
    pub proxy: Principal,
}
