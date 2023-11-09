use ethabi::{param_type::Reader, ParamType};
use regex::Regex;

/// Generate method identifiers from function expressions in abi format
#[derive(Debug, Clone, PartialEq)]
pub struct ContractMethodIdentifier {
    pub identifier: String,
    pub params: Vec<String>,
    pub return_value: Vec<String>,
}
impl ContractMethodIdentifier {
    pub fn parse_from_str(s: &str) -> anyhow::Result<Self> {
        let re =
            Regex::new(r"(?P<identifier>\w+)\((?P<params>[^)]*)\)(?::\((?P<return>[^)]*)\))?")?;
        let captures = re.captures(s).unwrap();

        let identifier = captures.name("identifier").unwrap().as_str().to_string();

        let params_str = captures.name("params").unwrap().as_str();
        let params_result: anyhow::Result<Vec<String>> = if params_str.is_empty() {
            Ok(vec![])
        } else {
            params_str
                .split(',')
                .map(|s| convert_type_from_abi_type(s.trim()))
                .collect()
        };
        let params = params_result?;

        let return_value_capture = captures.name("return");
        let return_value = if let Some(val) = return_value_capture {
            val.as_str()
                .split(',')
                .map(|s| convert_type_from_abi_type(s.trim()))
                .collect::<anyhow::Result<Vec<String>>>()
        } else {
            Ok(vec![])
        }?;

        Ok(ContractMethodIdentifier {
            identifier,
            params,
            return_value,
        })
    }
}

fn convert_type_from_abi_type(s: &str) -> anyhow::Result<String> {
    let param = Reader::read(s).map_err(|e| anyhow::anyhow!(e))?;
    convert_type_from_ethabi_param_type(&param).map_err(|e| anyhow::anyhow!(e))
}

/// To handle 256bits Unsigned Integer type in ic_web3_rs
pub const U256_TYPE: &str = "ic_web3_rs::types::U256";
/// To handle Address type in ic_web3_rs
pub const ADDRESS_TYPE: &str = "ic_web3_rs::types::Address";

pub fn convert_type_from_ethabi_param_type(param: &ethabi::ParamType) -> Result<String, String> {
    let err_msg = "ic_solidity_bindgen::internal::Unimplemented".to_string(); // temp
                                                                              // ref: https://github.com/horizonx-tech/ic-solidity-bindgen/blob/6c9ffb4354cee4c32b1df17a2210c90f16972c21/ic-solidity-bindgen-macros/src/abi_gen.rs#L124
    match param {
        ParamType::Address => Ok(ADDRESS_TYPE.to_string()),
        ParamType::Bytes => Ok("Vec<u8>".to_string()),
        ParamType::Int(size) => match size {
            129..=256 => Err(err_msg.to_string()),
            65..=128 => Ok("i128".to_string()),
            33..=64 => Ok("i64".to_string()),
            17..=32 => Ok("i32".to_string()),
            9..=16 => Ok("i16".to_string()),
            1..=8 => Ok("i8".to_string()),
            _ => Err(err_msg.to_string()),
        },
        ParamType::Uint(size) => match size {
            129..=256 => Ok(U256_TYPE.to_string()),
            65..=128 => Ok("u128".to_string()),
            33..=64 => Ok("u64".to_string()),
            17..=32 => Ok("u32".to_string()),
            1..=16 => Ok("u16".to_string()),
            _ => Err(err_msg),
        },
        ParamType::Bool => Ok("bool".to_string()),
        ParamType::String => Ok("String".to_string()),
        ParamType::Array(_) => Err(err_msg),         // temp
        ParamType::FixedBytes(_) => Err(err_msg),    // temp
        ParamType::FixedArray(_, _) => Err(err_msg), // temp
        ParamType::Tuple(_) => Err(err_msg),         // temp
    }
}
