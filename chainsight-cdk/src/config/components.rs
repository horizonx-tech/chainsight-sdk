use crate::indexer::IndexingConfig;
#[derive(serde::Deserialize)]
pub struct CommonConfig {
    pub monitor_duration: u32,
    pub canister_name: String,
}

#[derive(serde::Deserialize)]
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

#[derive(Default, serde::Deserialize)]
pub struct AlgorithmIndexerConfig {
    pub common: CommonConfig,
    pub indexing: IndexingConfig,
    pub input: AlgorithmIndexerInput,
}
