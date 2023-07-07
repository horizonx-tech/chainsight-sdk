use async_trait::async_trait;
use candid::Principal;
use ic_cdk::api::call::CallResult;

use super::{InitConfig, InitError, InitResult, Initializer};

pub struct ChainsightInitializer {
    config: InitConfig,
}

impl ChainsightInitializer {
    pub fn new(config: InitConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Initializer for ChainsightInitializer {
    async fn initialize(&self) -> Result<InitResult, InitError> {
        let _: CallResult<(Principal,)> = ic_cdk::api::call::call(
            self.config.env.initializer(),
            "deploy_vault_of",
            (ic_cdk::id(),),
        )
        .await;
        let px: CallResult<(Principal,)> =
            ic_cdk::api::call::call(self.config.env.initializer(), "get_proxy", ()).await;
        Ok(InitResult {
            proxy: px.unwrap().0,
        })
    }
}
