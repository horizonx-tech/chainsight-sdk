use std::collections::BTreeMap;

#[derive(Clone, Debug, candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct SnapshotValue { pub dummy: u64 }

pub fn get_query_parameters() -> std::collections::BTreeMap<String, String> {
    BTreeMap::from([
        ("param1".to_string(), "value1".to_string()),
        ("param2".to_string(), "value2".to_string())
    ])
}
