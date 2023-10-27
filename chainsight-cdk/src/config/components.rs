use crate::indexer::IndexingConfig;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RelayerConfig {
    pub common: CommonConfig,
    pub destination: String,
    pub oracle_type: String,
    pub method_name: String,
    pub canister_method_value_type: CanisterMethodValueType,
    pub abi_file_path: String,
    pub lens_targets: Option<LensTargets>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensTargets {
    pub identifiers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CanisterMethodValueType {
    Scalar(String, bool),                // struct name, is_scalar
    Tuple(Vec<(String, bool)>),          // struct_name, is_scalar
    Struct(Vec<(String, String, bool)>), // temp: Only non-nested `record` are supported.
    Vector(String, bool),                // struct_name, is_scalar
}
