use std::collections::HashMap;
use chainsight_cdk::storage::Data;
#[derive(Clone, Debug, Default, candid :: CandidType, serde :: Serialize, serde :: Deserialize)]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub value: String,
}
#[derive(Clone, Debug, Default, candid :: CandidType, serde :: Serialize, serde :: Deserialize, chainsight_cdk_macros::Persist, chainsight_cdk_macros::KeyValueStore)]
#[memory_id(1i32)]
pub struct OutputType1 {
    pub address: String,
}
#[derive(Clone, Debug, Default, candid :: CandidType, serde :: Serialize, serde :: Deserialize, chainsight_cdk_macros::Persist, chainsight_cdk_macros::KeyValuesStore)]
#[memory_id(2i32)]
pub struct OutputType2 {
    pub id: String,
    pub balance: u64,
}

pub fn persist(_elem: HashMap<u64, Vec<Transfer>>) {
    // do nothing
}
