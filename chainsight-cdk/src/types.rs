use candid::CandidType;
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, CandidType, Deserialize)]
pub enum EcdsaKeyEnvs {
    #[default]
    LocalDevelopment,
    Test,
    Production,
}

impl EcdsaKeyEnvs {
    pub fn to_key_name(&self) -> String {
        match self {
            EcdsaKeyEnvs::LocalDevelopment => "dfx_test_key",
            EcdsaKeyEnvs::Test => "test_key_1",
            EcdsaKeyEnvs::Production => "key_1",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum TimeUnit {
    Minute = 60,
    Hour = 3600,
    Day = 86400,
    Week = 604800,
}
