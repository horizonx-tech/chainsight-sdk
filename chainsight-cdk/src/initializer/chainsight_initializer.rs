use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::api::call::CallResult;

use super::{CycleManagements, InitConfig, InitError, InitResult, Initializer};

pub struct ChainsightInitializer {
    config: InitConfig,
}

impl ChainsightInitializer {
    pub fn new(config: InitConfig) -> Self {
        Self { config }
    }
}

#[derive(CandidType, serde::Deserialize, Clone, Copy)]
pub struct InitializeOutput {
    pub proxy: Principal,
    pub db: Principal,
}

#[async_trait]
impl Initializer for ChainsightInitializer {
    async fn initialize(&self, cycles: CycleManagements) -> Result<InitResult, InitError> {
        let out: CallResult<(InitializeOutput,)> = ic_cdk::api::call::call(
            self.config.env.initializer(),
            "initialize",
            (ic_cdk::caller(), cycles),
        )
        .await;
        Ok(InitResult {
            proxy: out.unwrap().0.proxy,
        })
    }
}
