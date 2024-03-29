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
    pub canister_name: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AlgorithmIndexerInput {
    pub method_name: String,
    pub response_type: String,
    pub source_type: AlgorithmInputType,
}

#[derive(Clone, Default, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AlgorithmIndexerOutput {
    pub types: Vec<AlgorithmIndexerOutputIdentifier>,
}
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AlgorithmIndexerOutputIdentifier {
    pub name: String,
    pub type_: AlgorithmOutputType,
}
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AlgorithmOutputType {
    #[serde(rename = "key_value")]
    KeyValue,
    #[serde(rename = "key_values")]
    KeyValues,
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
            canister_name: "example_canister".to_string(),
        }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmIndexerConfig {
    pub common: CommonConfig,
    pub indexing: IndexingConfig,
    pub input: AlgorithmIndexerInput,
    pub output: AlgorithmIndexerOutput,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmLensConfig {
    pub common: CommonConfig,
    pub target_count: usize,
    pub args_type: Option<String>,
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
    pub queries: SnapshotIndexerHTTPSConfigQueries,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SnapshotIndexerHTTPSConfigQueries {
    Const(BTreeMap<String, String>),
    Func(String),
}
impl Default for SnapshotIndexerHTTPSConfigQueries {
    fn default() -> Self {
        Self::Const(BTreeMap::new())
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SnapshotIndexerICPConfig {
    pub common: CommonConfig,
    pub method_identifier: String,
    pub is_target_component: bool,
    pub lens_parameter: Option<LensParameter>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RelayerConfig {
    pub common: CommonConfig,
    /// Method identifier for data source
    pub method_identifier: String,
    /// Address for destination
    pub destination: String,
    /// ABI for destination
    pub abi_file_path: String,
    /// Function name to call for destination
    pub method_name: String,
    /// Optional: Parameters for conversion
    pub conversion_parameter: Option<RelayerConversionParameter>,
    /// Optional: Parameters for using Lens as data source
    pub lens_parameter: Option<LensParameter>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelayerConversionParameter {
    /// Optional: Field extracted from response from data source, set by chaining based on the base object
    pub extracted_field: Option<String>,
    /// Optional: Set the destination type
    pub destination_type_to_convert: Option<String>,
    /// Optional: Set exponent for power10, this is N when value * 10^N
    pub exponent_of_power10: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensParameter {
    pub with_args: bool,
}
// Type name to be used in the computation function endpoints generated by algorithm_lens
// NOTE: also used in cli
pub const LENS_FUNCTION_ARGS_TYPE: &str = "LensArgs";
pub const LENS_FUNCTION_RESPONSE_TYPE: &str = "LensValue";

// NOTE: not use in sdk, use only in cli
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensTargets {
    pub identifiers: Vec<String>,
}
