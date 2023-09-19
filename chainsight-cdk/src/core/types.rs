use std::collections::HashMap;

use candid::CandidType;
use serde_json::json;

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

#[derive(Clone, CandidType, Debug, PartialEq)]
pub struct Sources<T>
where
    T: Clone + CandidType + serde::Serialize,
{
    pub source_type: SourceType,
    pub source: String,
    pub attributes: T,
    pub interval_sec: Option<u32>,
}
#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, CandidType)]

pub enum SourceType {
    #[serde(rename = "evm")]
    Evm,
    #[serde(rename = "chainsight")]
    Chainsight,
}
#[derive(Clone, CandidType, serde::Serialize)]
pub struct Web3EventIndexerSourceAttrs {
    pub chain_id: u64,
    pub event_name: String,
}

#[derive(Clone, CandidType, serde::Serialize)]
pub struct Web3AlgorithmIndexerSourceAttrs {
    pub chain_id: u64,
    pub function_name: String,
}

#[derive(Clone, CandidType, serde::Serialize)]
pub struct RelayerWithLensSourceAttrs {
    pub sources: Vec<String>,
}

pub type Web3SnapshotIndexerSourceAttrs = Web3AlgorithmIndexerSourceAttrs;
pub enum ChainsightCanisterType {
    Web3EventIndexer,
    AlgorithmIndexer,
    Web3SnapshotIndexer,
    ICSNapshotIndexer,
    Web3Relayer,
}

impl<T: Clone + CandidType + serde::Serialize> Sources<T> {
    fn new(source_type: SourceType, source: String, interval_sec: Option<u32>, attrs: T) -> Self
    where
        T: Clone + CandidType + serde::Serialize,
    {
        Self {
            source_type,
            source,
            attributes: attrs,
            interval_sec,
        }
    }
    pub fn new_event_indexer(
        address: String,
        interval: u32,
        attrs: Web3EventIndexerSourceAttrs,
    ) -> Sources<Web3EventIndexerSourceAttrs> {
        Sources::new(SourceType::Evm, address, Some(interval), attrs)
    }
    pub fn new_algorithm_indexer(
        principal: String,
        interval: u32,
    ) -> Sources<HashMap<String, String>> {
        Sources::new(
            SourceType::Chainsight,
            principal,
            Some(interval),
            HashMap::new(),
        )
    }
    pub fn new_snapshot_indexer(
        principal: String,
        interval: u32,
        method_identifier: String,
    ) -> Sources<HashMap<String, String>> {
        let mut method_id = match method_identifier.contains(':') {
            true => method_identifier.split(':').collect::<Vec<&str>>()[0].to_string(),
            false => method_identifier,
        };
        method_id = method_id.replace(' ', "").replace("()", "");
        let mut attrs = HashMap::new();
        attrs.insert("function_name".to_string(), method_id);
        Sources::new(SourceType::Chainsight, principal, Some(interval), attrs)
    }
    pub fn new_relayer(
        principal: String,
        interval: u32,
        lens_targets: Vec<String>,
    ) -> Sources<RelayerWithLensSourceAttrs> {
        Sources::new(
            SourceType::Chainsight,
            principal,
            Some(interval),
            RelayerWithLensSourceAttrs {
                sources: lens_targets,
            },
        )
    }
    pub fn new_web3_snapshot_indexer(
        address: String,
        interval: u32,
        chain_id: u64,
        function_name: String,
    ) -> Sources<Web3SnapshotIndexerSourceAttrs> {
        Sources::new(
            SourceType::Evm,
            address,
            Some(interval),
            Web3SnapshotIndexerSourceAttrs {
                chain_id,
                function_name,
            },
        )
    }
}
