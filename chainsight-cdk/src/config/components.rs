use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;
use regex::Regex;

use crate::indexer::IndexingConfig;

lazy_static! {
    static ref MAPPING_CANDID_TY: HashMap<&'static str, &'static str> = [
        ("text", "String"),
        // ("blob", "&[u8]"),
        ("nat", "u128"),
        ("int", "i128"),
        ("nat8", "u8"),
        ("nat16", "u16"),
        ("nat32", "u32"),
        ("nat64", "u64"),
        ("int8", "i8"),
        ("int16", "i16"),
        ("int32", "i32"),
        ("int64", "i64"),
        ("float32", "f32"),
        ("float64", "f64"),
        ("bool", "bool"),
        // ("null", "()"),
    ].iter().cloned().collect();

    static ref REGEX_CANDID_FUNC: Regex = Regex::new(r"(?P<identifier>\w+)\s*:\s*\((?P<params>.*?)\)\s*(->\s*\((?P<return>.*?)\))?").unwrap();
    static ref REGEX_VECTOR: Regex = Regex::new(r"vec\s(?P<type>\w+)").unwrap();
    static ref REGEX_TUPLE: Regex = Regex::new(r"record\s\{\s(?P<items>(\w+(;\s|))+)\s\}").unwrap();
    static ref REGEX_STRUCT: Regex = Regex::new(r"(?P<field>\w+)\s*:\s*(?P<type>\w+)").unwrap();

    static ref REGEX_MULTIPLE_RECORD_TYPE: Regex = Regex::new(r"record\s*\{").unwrap();
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CommonConfig {
    pub monitor_duration: u32,
    pub canister_name: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AlgorithmIndexerInput {
    pub method_name: String,
    pub response_type: String,
    pub source_type: AlgorithmInputType,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AlgorithmInputType {
    #[serde(rename = "event_indexer")]
    EventIndexer,
    #[serde(rename = "key_value")]
    KeyValue,
    #[serde(rename = "key_values")]
    KeyValues,
}

impl Default for AlgorithmIndexerInput {
    fn default() -> Self {
        Self {
            method_name: "get_list".to_string(),
            response_type: "String".to_string(),
            source_type: AlgorithmInputType::EventIndexer,
        }
    }
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            monitor_duration: 3600,
            canister_name: "example_canister".to_string(),
        }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmIndexerConfig {
    pub common: CommonConfig,
    pub indexing: IndexingConfig,
    pub input: AlgorithmIndexerInput,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmLensConfig {
    pub common: CommonConfig,
    pub target_count: usize,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct EventIndexerConfig {
    pub common: CommonConfig,
    pub def: EventIndexerEventDefinition,
}
#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventIndexerEventDefinition {
    pub identifier: String,
    pub abi_file_path: String,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SnapshotIndexerEVMConfig {
    pub common: CommonConfig,
    pub method_identifier: String,
    pub method_args: Vec<serde_json::Value>,
    pub abi_file_path: String,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SnapshotIndexerHTTPSConfig {
    pub common: CommonConfig,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub queries: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SnapshotIndexerICPConfig {
    pub common: CommonConfig,
    pub method_identifier: String,
    pub lens_targets: Option<LensTargets>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RelayerConfig {
    pub common: CommonConfig,
    pub destination: String,
    pub oracle_type: String,
    pub method_identifier: String,
    pub abi_file_path: String,
    pub lens_targets: Option<LensTargets>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensTargets {
    pub identifiers: Vec<String>,
}
