// NOTE: Originally imported from bindings
#[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]
pub struct CalculateArgs {
    pub index: u64,
    pub target: String,
}
#[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]

pub struct LensArgs {
    pub targets: Vec<String>,
    pub args: CalculateArgs,
}

// logics
pub type CallCanisterResponse = (String, String);
pub fn call_args() -> CalculateArgs {
    CalculateArgs {
        index: 99,
        target: "TARGET_CANISTER_ID".to_string(),
    }
}
pub fn filter(_: &CallCanisterResponse) -> bool {
    true
}
pub fn convert(_: &CallCanisterResponse) -> ContractCallArgs {
    todo!()
}

pub struct ContractCallArgs {
    pub ids: Vec<ic_web3_rs::types::U256>,
    pub proposers: Vec<ic_web3_rs::types::Address>,
    pub chainIds: Vec<ic_web3_rs::types::U256>,
    pub startTimestamps: Vec<ic_web3_rs::types::U256>,
    pub endTimestamps: Vec<ic_web3_rs::types::U256>,
    pub proposedBlocks: Vec<ic_web3_rs::types::U256>,
}
