use candid::CandidType;

#[derive(
    CandidType, Debug, Clone, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize, Default,
)]
pub struct U256 {
    value: String,
}

impl U256 {
    pub fn value(&self) -> primitive_types::U256 {
        primitive_types::U256::from_dec_str(&self.value).unwrap()
    }
}

impl From<ic_web3_rs::types::U256> for U256 {
    fn from(u256: ic_web3_rs::types::U256) -> Self {
        Self {
            value: u256.to_string(),
        }
    }
}

impl From<primitive_types::U256> for U256 {
    fn from(u256: primitive_types::U256) -> Self {
        Self {
            value: u256.to_string(),
        }
    }
}
