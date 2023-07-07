use candid::{CandidType, Principal};
use serde::Deserialize;
#[derive(Clone, Debug, Default, PartialEq, CandidType, Deserialize)]
pub enum Env {
    #[default]
    LocalDevelopment,
    Test,
    Production,
}

impl Env {
    pub fn initializer(&self) -> Principal {
        match self {
            Env::LocalDevelopment => Principal::from_text("7fpuj-hqaaa-aaaal-acg7q-cai").unwrap(),
            Env::Production => Principal::from_text("qoctq-giaaa-aaaaa-aaaea-cai").unwrap(), //TODO
            Env::Test => Principal::from_text("qoctq-giaaa-aaaaa-aaaea-cai").unwrap(),       //TODO
        }
    }
    pub fn ecdsa_key_name(&self) -> String {
        match self {
            Env::LocalDevelopment => "dfx_test_key",
            Env::Test => "test_key_1",
            Env::Production => "key_1",
        }
        .to_string()
    }
}
