use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::api::call::{msg_cycles_accept128, CallResult};

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
    pub vault: Principal,
}

#[async_trait]
impl Initializer for ChainsightInitializer {
    async fn initialize(
        &self,
        deployer: &Principal,
        cycles: &CycleManagements,
        subnet: &Option<Principal>,
    ) -> Result<InitResult, InitError> {
        let res: CallResult<(InitializeOutput,)> = ic_cdk::api::call::call_with_payment128(
            self.config.env.initializer(),
            "initialize",
            (deployer, cycles, subnet),
            msg_cycles_accept128(cycles.initial_supply()),
        )
        .await;
        let out = res.unwrap().0;

        Ok(InitResult {
            proxy: out.proxy,
            db: out.db,
            vault: out.vault,
        })
    }
}
