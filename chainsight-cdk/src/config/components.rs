use std::collections::HashMap;

use syn::parse::{Parse, ParseStream};

use crate::indexer::IndexingConfig;
#[derive(serde::Deserialize)]
pub struct CommonConfig {
    pub monitor_duration: u32,
    pub canister_name: String,
}

impl Parse for CommonConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = CommonConfig::default();
        let monitor_duration = input.parse::<syn::LitInt>()?;
        result.monitor_duration = monitor_duration.base10_parse()?;
        let canister_name = input.parse::<syn::LitStr>()?;
        result.canister_name = canister_name.value();
        Ok(result)
    }
}

#[derive(Default, serde::Deserialize)]
pub struct UserDefinedStruct {
    pub name: String,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize)]
pub struct AlgorithmIndexerInput {
    pub method_name: String,
    pub response_type: String,
    pub source_type: SourceType,
}
#[derive(Default, serde::Deserialize)]
pub enum SourceType {
    EventIndexer,
    KeyValue,
    #[default]
    KeyValues,
}

impl Default for AlgorithmIndexerInput {
    fn default() -> Self {
        Self {
            method_name: "get_list".to_string(),
            response_type: "String".to_string(),
            source_type: SourceType::EventIndexer,
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
