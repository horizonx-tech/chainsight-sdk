use std::collections::HashMap;

use anyhow::bail;
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

#[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct CanisterMethodIdentifier {
    pub identifier: String,
    pub params: Vec<String>,
    pub return_value: CanisterMethodValueType,
}

impl CanisterMethodIdentifier {
    pub fn parse_from_str(s: &str) -> anyhow::Result<Self> {
        let captures = REGEX_CANDID_FUNC.captures(s).ok_or(anyhow::anyhow!(
            "method.identifier does not satisfy the supported expression: {}, supplied={}",
            REGEX_CANDID_FUNC.to_string(),
            s
        ))?;

        let identifier = captures.name("identifier").unwrap().as_str().to_string();

        let params_str = captures.name("params").unwrap().as_str();
        let params_result: Vec<String> = if params_str.is_empty() {
            vec![]
        } else {
            params_str
                .split(',')
                .map(|s| convert_type_from_candid_type(s.trim()).0)
                .collect()
        };
        let params = params_result;

        let return_value_str = captures.name("return").unwrap().as_str();
        let return_value = Self::parse_return_value(return_value_str)?;

        Ok(CanisterMethodIdentifier {
            identifier,
            params,
            return_value,
        })
    }

    fn parse_return_value(s: &str) -> anyhow::Result<CanisterMethodValueType> {
        let record_type_count = REGEX_MULTIPLE_RECORD_TYPE.find_iter(s).count();
        if record_type_count >= 2 {
            bail!(
                "Sorry, Currently nested `record` types are not supported. This will be supported in the future.\nTarget literal = {}",
                s
            ); // TODO: Support nested `record` types.
        }
        // vector
        if s.starts_with("vec") {
            let captures = REGEX_VECTOR.captures(s);
            if let Some(captures_value) = captures {
                let ty = captures_value.name("type").unwrap().as_str();
                let val = convert_type_from_candid_type(ty);
                return Ok(CanisterMethodValueType::Vector(val.0, val.1));
            }
            bail!("Invalid candid's result types:{}", s);
        }

        // Scalar
        if !s.starts_with("record") {
            let val = convert_type_from_candid_type(s);
            return Ok(CanisterMethodValueType::Scalar(val.0, val.1));
        }

        // Tuple
        let captures = REGEX_TUPLE.captures(s);
        if let Some(captures_value) = captures {
            let items = captures_value.name("items").unwrap().as_str();
            let tuple_result: Vec<(String, bool)> = items
                .split(';')
                .map(|s| convert_type_from_candid_type(s.trim()))
                .collect();
            let tuple = tuple_result;
            return Ok(CanisterMethodValueType::Tuple(tuple));
        }

        // Struct
        let items = REGEX_STRUCT.captures_iter(s);
        let mut struct_items = vec![];
        for cap in items {
            let field = cap.name("field").unwrap().as_str().to_string();
            let ty = convert_type_from_candid_type(cap.name("type").unwrap().as_str());
            struct_items.push((field, ty.0, ty.1));
        }
        if struct_items.is_empty() {
            bail!("Invalid candid's result types: {}", s);
        }
        Ok(CanisterMethodValueType::Struct(struct_items))
    }
}

fn convert_type_from_candid_type(s: &str) -> (String, bool) {
    // ref: https://internetcomputer.org/docs/current/references/candid-ref
    let ty_str = MAPPING_CANDID_TY.get(&s);
    if let Some(ty_str) = ty_str {
        return (ty_str.to_string(), true);
    }
    (s.to_string(), false)
}
