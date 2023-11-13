// NOTE: Originally imported from bindings
#[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]
pub struct CalculateArgs {
    pub index: u64,
    pub target: String
}
#[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]

pub struct LensArgs {
    pub targets: Vec<String>,
    pub args: CalculateArgs
}

// logics
pub fn call_args() -> CalculateArgs {
    CalculateArgs {
        index: 99,
        target: "TARGET_CANISTER_ID".to_string()
    }
}
