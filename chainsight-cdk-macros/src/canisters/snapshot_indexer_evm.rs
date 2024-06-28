use std::{borrow::Borrow, str::FromStr as _};

use anyhow::Error;
use chainsight_cdk::config::components::{CommonConfig, SnapshotIndexerEVMConfig};
use ic_web3_rs::ethabi::{ParamType, StateMutability};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, ToTokens as _};
use syn::parse_macro_input;

use crate::canisters::utils::{
    camel_to_snake, extract_contract_name_from_path, generate_queries_without_timestamp,
};

pub fn def_snapshot_indexer_evm(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: SnapshotIndexerEVMConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    snapshot_indexer_evm(config).into()
}

fn snapshot_indexer_evm(config: SnapshotIndexerEVMConfig) -> proc_macro2::TokenStream {
    let common = common_code(&config.common);
    let custom = custom_code(config);
    quote! {
        #common
        #custom
    }
}

fn common_code(config: &CommonConfig) -> proc_macro2::TokenStream {
    let CommonConfig { canister_name } = config;

    quote! {
        use std::str::FromStr;
        use candid::{Decode, Encode};
        use chainsight_cdk_macros::{init_in, manage_single_state, setup_func, prepare_stable_structure, stable_memory_for_scalar, stable_memory_for_btree_map, StableMemoryStorable, timer_task_func, define_transform_for_web3, define_web3_ctx, chainsight_common, did_export, CborSerde, snapshot_indexer_web3_source};
        use ic_stable_structures::writer::Writer;
        use ic_web3_rs::types::Address;

        did_export!(#canister_name); // NOTE: need to be declared before query, update

        init_in!(2);
        chainsight_common!();

        define_web3_ctx!(3);
        define_transform_for_web3!();
        stable_memory_for_scalar!("target_addr", String, 4, false);
        setup_func!({
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
        }, 5);

        prepare_stable_structure!();
        stable_memory_for_btree_map!("snapshot", Snapshot, 1, true);
        timer_task_func!("set_task", "index", 6);
    }
}

fn custom_code(config: SnapshotIndexerEVMConfig) -> proc_macro2::TokenStream {
    let SnapshotIndexerEVMConfig {
        method_identifier,
        method_args,
        abi_file_path,
        ..
    } = config;

    let abi_bytes = std::fs::read(&abi_file_path)
        .map_err(|e| anyhow::anyhow!("Failed to load abi: {}", e))
        .unwrap();
    let contract = ic_web3_rs::ethabi::Contract::load(&abi_bytes[..])
        .map_err(|e| anyhow::anyhow!("Failed to parse abi: {}", e))
        .unwrap();

    let signature: String = method_identifier
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let name = signature
        .split('(')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid function identifier: {}", signature))
        .unwrap();
    let functions = contract
        .functions_by_name(name)
        .unwrap_or_else(|_| panic!("function not found. name: {}", name));
    let function = functions
        .iter()
        .find(|f| f.signature() == signature)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "function not found. identifier: {}, available: {}",
                signature,
                functions
                    .iter()
                    .map(|f| f.signature())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .unwrap();

    let method_ident_str = camel_to_snake(name);
    let method_ident = format_ident!("{}", method_ident_str);
    let method_quote = match function.state_mutability {
        StateMutability::Pure | StateMutability::View => quote! { #method_ident },
        StateMutability::NonPayable | StateMutability::Payable => {
            quote! { static_call.#method_ident }
        }
    };

    let contract_struct_ident =
        format_ident!("{}", extract_contract_name_from_path(&abi_file_path));

    // for request values
    assert!(function.inputs.len() == method_args.len(), "datatource.method is not valid: The number of params in 'identifier' and 'args' must be the same");

    let request_arg_tokens = serde_to_token_streams(
        function
            .inputs
            .iter()
            .map(|p| p.kind.clone())
            .collect::<Vec<_>>()
            .as_slice(),
        &method_args,
    )
    .map_err(|e| anyhow::anyhow!("Failed to parse args: {}", e))
    .unwrap();

    let response_types: Vec<proc_macro2::TokenStream> = function
        .outputs
        .iter()
        .map(|p| to_candid_type(&p.kind).0)
        .collect();

    let response_values = to_candid_values(
        &function
            .outputs
            .iter()
            .map(|e| e.kind.clone())
            .collect::<Vec<_>>(),
        quote! { res },
    );

    // consider whether to add timestamp information to the snapshot
    let (snapshot_idents, queries_expect_timestamp) = (
        quote! {
            #[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize, StableMemoryStorable)]
            #[stable_mem_storable_opts(max_size = 10000, is_fixed_size = false)] // temp: max_size
            pub struct Snapshot {
                pub value: SnapshotValue,
                pub timestamp: u64,
            }
            type SnapshotValue = (#(#response_types),*);
        },
        generate_queries_without_timestamp(format_ident!("SnapshotValue")),
    );

    quote! {
        #snapshot_idents

        #queries_expect_timestamp

        ic_solidity_bindgen::contract_abi!(#abi_file_path);
        snapshot_indexer_web3_source!(#method_ident_str);

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }

            let current_ts_sec = ic_cdk::api::time() / 1000000;
            let res = #contract_struct_ident::new(
                Address::from_str(&get_target_addr()).expect("Failed to parse target addr to Address"),
                &web3_ctx().expect("Failed to get web3_ctx"),
            ).#method_quote(#(#request_arg_tokens,)*None).await.expect("Failed to call contract");

            let datum = Snapshot {
                value: #response_values,
                timestamp: current_ts_sec,
            };
            add_snapshot(datum.clone());

            ic_cdk::println!("timestamp={}, value={:?}", datum.timestamp, datum.value);
        }
    }
}

pub fn serde_to_token_streams(
    inputs: &[ParamType],
    method_args: &[serde_json::Value],
) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut tokens = vec![];
    for (i, input) in inputs.iter().enumerate() {
        let value = method_args[i].clone();
        let t = match input {
            ParamType::Bool => {
                let val = value.as_bool().unwrap();
                format_ident!("{}", val).to_token_stream()
            }
            ParamType::Uint(size) => match size {
                1..=128 => {
                    let val = if value.is_string() {
                        let value_str = value.as_str().unwrap();
                        if let Some(stripped) = value_str.strip_prefix("0x") {
                            u128::from_str_radix(stripped, 16).unwrap()
                        } else {
                            u128::from_str(value_str).unwrap()
                        }
                    } else {
                        value.as_u64().unwrap() as u128
                    };
                    proc_macro2::Literal::u128_unsuffixed(val).to_token_stream()
                }
                _ => {
                    let val = if value.is_string() {
                        value.as_str().unwrap().to_string()
                    } else {
                        value.as_u64().unwrap().to_string()
                    };
                    let radix_lit = if val.starts_with("0x") {
                        proc_macro2::Literal::u8_unsuffixed(16)
                    } else {
                        proc_macro2::Literal::u8_unsuffixed(10)
                    };
                    quote! { ic_web3_rs::types::U256::from_str_radix(#val, #radix_lit).unwrap() }
                }
            },
            ParamType::Int(size) => match size {
                1..=128 => {
                    let val = if value.is_string() {
                        let value_str = value.as_str().unwrap();
                        if value_str.contains("0x") {
                            i128::from_str_radix(&value_str.replace("0x", ""), 16).unwrap()
                        } else {
                            i128::from_str(value_str).unwrap()
                        }
                    } else {
                        value.as_i64().unwrap() as i128
                    };
                    proc_macro2::Literal::i128_unsuffixed(val).to_token_stream()
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Sorry, not supported: Int more than 128bits"
                    ));
                }
            },
            ParamType::Address => {
                let val = value.as_str().unwrap();
                quote! { Address::from_str(#val).unwrap() }
            }
            ParamType::Tuple(param_types) => {
                let args = value.as_array().unwrap();
                let types = serde_to_token_streams(param_types, args)?;
                quote! { (#(#types),*) }
            }
            ParamType::FixedArray(param_type, _) => {
                let args = value.as_array().unwrap();
                let param_types = args
                    .iter()
                    .map(|_| param_type.as_ref().clone())
                    .collect::<Vec<_>>();
                let types = serde_to_token_streams(param_types.as_slice(), args)?;
                quote! { [#(#types),*] }
            }
            ParamType::Array(param_type) => {
                let args = value.as_array().unwrap();
                let param_types = args
                    .iter()
                    .map(|_| param_type.as_ref().clone())
                    .collect::<Vec<_>>();
                let types = serde_to_token_streams(param_types.as_slice(), args)?;
                quote! { vec![#(#types),*] }
            }
            ParamType::FixedBytes(_) => {
                let bytes = if value.is_string() {
                    let value_str = value.as_str().unwrap();
                    if let Some(stripped) = value_str.strip_prefix("0x") {
                        hex::decode(stripped).unwrap()
                    } else {
                        value_str.as_bytes().to_vec()
                    }
                } else if value.is_array() {
                    let vals = value.as_array().unwrap();
                    vals.iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect::<Vec<_>>()
                } else {
                    panic!("Unexpected value for FixedBytes: {}", value)
                };
                let bytes = bytes
                    .iter()
                    .map(|b| proc_macro2::Literal::u8_unsuffixed(*b).to_token_stream())
                    .collect::<Vec<_>>();
                quote! { [#(#bytes),*] }
            }
            ParamType::Bytes => {
                let bytes = if value.is_string() {
                    let value_str = value.as_str().unwrap();
                    if let Some(stripped) = value_str.strip_prefix("0x") {
                        hex::decode(stripped).unwrap()
                    } else {
                        value_str.as_bytes().to_vec()
                    }
                } else if value.is_array() {
                    let vals = value.as_array().unwrap();
                    vals.iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect::<Vec<_>>()
                } else {
                    panic!("Unexpected value for FixedBytes: {}", value)
                };
                let bytes = bytes
                    .iter()
                    .map(|b| proc_macro2::Literal::u8_unsuffixed(*b).to_token_stream())
                    .collect::<Vec<_>>();
                quote! { vec![#(#bytes),*] }
            }
            ParamType::String => {
                let token = value.as_str().unwrap().to_token_stream();
                quote! { #token.into() } // from String to &str
            }
        };
        tokens.push(t);
    }
    Ok(tokens)
}

fn ident<S: Borrow<str>>(name: S) -> Ident {
    Ident::new(name.borrow(), Span::call_site())
}

fn to_candid_type(kind: &ParamType) -> (proc_macro2::TokenStream, usize) {
    match kind {
        ParamType::Bool => (quote! { bool }, 0),
        ParamType::Uint(size) => match size {
            129..=256 => (quote! {  ::std::string::String }, 0),
            65..=128 => (ident("u128").to_token_stream(), 0),
            33..=64 => (ident("u64").to_token_stream(), 0),
            17..=32 => (ident("u32").to_token_stream(), 0),
            9..=16 => (ident("u16").to_token_stream(), 0),
            1..=8 => (ident("u8").to_token_stream(), 0),
            _ => (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0),
        },
        ParamType::Int(size) => match size {
            129..=256 => (quote! {  ::std::string::String }, 0),
            65..=128 => (ident("i128").to_token_stream(), 0),
            33..=64 => (ident("i64").to_token_stream(), 0),
            17..=32 => (ident("i32").to_token_stream(), 0),
            9..=16 => (ident("i16").to_token_stream(), 0),
            1..=8 => (ident("i8").to_token_stream(), 0),
            _ => (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0),
        },
        ParamType::Address => (quote! { ::std::string::String }, 0),
        ParamType::Tuple(members) => match members.len() {
            0 => (quote! { ::ic_solidity_bindgen::internal::Empty }, 1),
            _ => {
                let members: Vec<_> = members.iter().map(to_candid_type).collect();
                // Unwrap is ok because in this branch there must be at least 1 item.
                let nesting = 1 + members.iter().map(|(_, n)| *n).max().unwrap();
                let types = members.iter().map(|(ty, _)| ty);
                (quote! { (#(#types,)*) }, nesting)
            }
        },
        ParamType::FixedBytes(len) => (quote! { [ u8; #len ] }, 0),
        ParamType::Bytes => (quote! { ::std::vec::Vec<u8> }, 0),
        ParamType::FixedArray(inner, _) => {
            let (inner, nesting) = to_candid_type(inner);
            (quote! { ::std::vec::Vec<#inner> }, nesting)
        }
        ParamType::Array(inner) => {
            let (inner, nesting) = to_candid_type(inner);
            if nesting > 0 {
                (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0)
            } else {
                (quote! { ::std::vec::Vec<#inner> }, nesting)
            }
        }
        ParamType::String => (quote! { ::std::string::String }, 0),
    }
}

fn to_candid_values(
    outputs: &[ParamType],
    accessor: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if outputs.len() == 1 {
        let value = to_candid_value(&outputs[0], quote! { #accessor });
        return quote! { #value };
    }
    let mut values = vec![];
    for (i, output) in outputs.iter().enumerate() {
        let idx_lit = proc_macro2::Literal::usize_unsuffixed(i);
        values.push(to_candid_value(output, quote! { #accessor.#idx_lit }));
    }
    quote! { (#(#values,)*) }
}

fn to_candid_value(
    kind: &ParamType,
    accessor: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match kind {
        ParamType::Bool => quote! { #accessor },
        ParamType::Uint(size) => match size {
            1..=128 => quote! { #accessor },
            129..=256 => quote! { #accessor.to_string() },
            _ => quote! {  #accessor.to_string() },
        },
        ParamType::Int(size) => match size {
            1..=128 => quote! { #accessor },
            129..=256 => quote! {  #accessor.0.to_string() }, // NOTE: Support for I256 in ic-web3-rs
            _ => quote! { #accessor.to_string()},
        },
        ParamType::Address => quote! {  #accessor.to_string() },
        ParamType::Tuple(members) => match members.len() {
            0 => quote! { ::ic_solidity_bindgen::internal::Empty },
            _ => {
                let mut values = vec![];
                for (i, kind) in members.iter().enumerate() {
                    let idx_lit = proc_macro2::Literal::usize_unsuffixed(i);
                    values.push(to_candid_value(kind, quote! {#accessor.#idx_lit}));
                }
                quote! { (#(#values,)*) }
            }
        },
        ParamType::FixedBytes(_) => quote! { #accessor },
        ParamType::Bytes => quote! { #accessor },
        ParamType::FixedArray(param_type, _) => {
            if should_convert(param_type.as_ref()) {
                let inner = to_candid_value(param_type.as_ref(), quote! { e });
                quote! { #accessor.iter().map(|e| #inner).collect::<Vec<_>>() }
            } else {
                quote! { #accessor }
            }
        }
        ParamType::Array(param_type) => {
            if should_convert(param_type.as_ref()) {
                let inner = to_candid_value(param_type.as_ref(), quote! { e });
                quote! { #accessor.iter().map(|e| #inner).collect::<Vec<_>>() }
            } else {
                quote! { #accessor }
            }
        }
        ParamType::String => quote! { #accessor },
    }
}
fn should_convert(kind: &ParamType) -> bool {
    match kind {
        ParamType::Address => true,
        ParamType::Bytes => false,
        ParamType::Int(size) => match size {
            1..=128 => false,
            129..=256 => true,
            _ => true,
        },
        ParamType::Uint(size) => match size {
            1..=128 => false,
            129..=256 => true,
            _ => true,
        },
        ParamType::Bool => false,
        ParamType::String => false,
        ParamType::Array(_) => true,
        ParamType::FixedBytes(_) => false,
        ParamType::FixedArray(_, _) => true,
        ParamType::Tuple(_) => true,
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_display_snapshot;
    use rust_format::{Formatter, RustFmt};
    use serde_json::json;

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = SnapshotIndexerEVMConfig {
            common: CommonConfig {
                canister_name: "sample_snapshot_indexer_evm".to_string(),
            },
            method_identifier: "totalSupply():(uint256)".to_string(),
            method_args: vec![],
            abi_file_path: "examples/minimum_indexers/src/snapshot_indexer_evm/abi/ERC20.json"
                .to_string(),
        };
        let generated = snapshot_indexer_evm(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!("snapshot__snapshot_indexer_evm", formatted);
    }

    #[test]
    fn test_serde_to_token_streams_bool() {
        assert_eq!(
            serde_to_token_streams(&[ParamType::Bool], &[json!(true)],).unwrap()[0].to_string(),
            "true"
        );
        assert_eq!(
            serde_to_token_streams(&[ParamType::Bool], &[json!(false)],).unwrap()[0].to_string(),
            "false"
        );
    }

    #[test]
    fn test_serde_to_token_streams_uint() {
        let inputs = vec![
            ParamType::Uint(1),
            ParamType::Uint(64),
            ParamType::Uint(64),
            ParamType::Uint(64),
            ParamType::Uint(72),
            ParamType::Uint(128),
            ParamType::Uint(136),
            ParamType::Uint(256),
            ParamType::Uint(256),
            ParamType::Uint(256),
            ParamType::Uint(256),
            ParamType::Uint(256),
            ParamType::Uint(256),
        ];
        let method_args = vec![
            json!(0x0u8),
            json!(0xffffffffffffffffu64),                 // 64bit
            json!(0xffffffffffffffffu64.to_string()),     // decimal
            json!("0xffffffffffffffff"),                  // hexadecimal
            json!("0x10000000000000000"),                 // 64bit + 1
            json!("0xffffffffffffffffffffffffffffffff"),  // 128bit
            json!("0x100000000000000000000000000000000"), // 128bit + 1
            json!("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"), // 256bit
            json!("115792089237316195423570985008687907853269984665640564039457584007913129639935"), // 256bit
            json!("0"),                    // 256bit
            json!("18446744073709551615"), // 256bit
            json!("0xffffffffffffffff"),   // 256bit
            json!("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"), // 256bit
        ];
        let tokens = serde_to_token_streams(&inputs, &method_args).unwrap();

        let expected = [
            "0",
            "18446744073709551615",
            "18446744073709551615",
            "18446744073709551615",
            "18446744073709551616",
            "340282366920938463463374607431768211455",
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("0x100000000000000000000000000000000" , 16) . unwrap ()"#,
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" , 16) . unwrap ()"#,
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("115792089237316195423570985008687907853269984665640564039457584007913129639935" , 10) . unwrap ()"#,
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("0" , 10) . unwrap ()"#,
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("18446744073709551615" , 10) . unwrap ()"#,
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("0xffffffffffffffff" , 16) . unwrap ()"#,
            r#"ic_web3_rs :: types :: U256 :: from_str_radix ("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" , 16) . unwrap ()"#,
        ];

        assert_eq!(tokens.len(), expected.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.to_string(), expected[i].to_string());
        }
        assert_eq!(
            ic_web3_rs::types::U256::from_str_radix(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935",
                10
            )
            .unwrap(),
            ic_web3_rs::types::U256::MAX
        );
        assert_eq!(
            ic_web3_rs::types::U256::from_str_radix(
                "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                16
            )
            .unwrap(),
            ic_web3_rs::types::U256::MAX
        );
    }

    #[test]
    fn test_serde_to_token_streams_int() {
        let inputs = vec![
            ParamType::Int(1),
            ParamType::Int(64),
            ParamType::Int(64),
            ParamType::Int(64),
            ParamType::Int(64),
            ParamType::Int(72),
            ParamType::Int(72),
            ParamType::Int(128),
            ParamType::Int(128),
        ];
        let method_args = vec![
            json!(0x0u8),
            json!(0x7fffffffffffffffi64),                 // 64bit hex
            json!(0x7fffffffffffffffi64.to_string()),     // decimal
            json!(-0x8000000000000000i64),                // negative hex
            json!((-0x8000000000000000i64).to_string()),  // negative decimal
            json!("0x8000000000000000"),                  // 64bit + 1
            json!("-0x8000000000000001"),                 // 64bit - 1
            json!("0x7fffffffffffffffffffffffffffffff"),  // 128bit
            json!("-0x80000000000000000000000000000000"), // 128bit
        ];
        let tokens = serde_to_token_streams(&inputs, &method_args).unwrap();

        let expected = [
            "0",
            "9223372036854775807",
            "9223372036854775807",
            "- 9223372036854775808",
            "- 9223372036854775808",
            "9223372036854775808",
            "- 9223372036854775809",
            "170141183460469231731687303715884105727",
            "- 170141183460469231731687303715884105728",
        ];

        assert_eq!(tokens.len(), expected.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.to_string(), expected[i].to_string());
        }
    }

    #[test]
    fn test_serde_to_token_streams_address() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::Address],
                &[json!("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE")],
            )
            .unwrap()[0]
                .to_string(),
            r#"Address :: from_str ("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE") . unwrap ()"#
        );
    }

    #[test]
    fn test_serde_to_token_streams_tuple() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::Tuple(vec![ParamType::Address, ParamType::Bool])],
                &[json!(["0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", false])],
            )
            .unwrap()[0]
                .to_string(),
            r#"(Address :: from_str ("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE") . unwrap () , false)"#
        );
    }

    #[test]
    fn test_serde_to_token_streams_tuple_nested() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::Tuple(vec![
                    ParamType::Address,
                    ParamType::Tuple(vec![ParamType::Uint(8), ParamType::Bool])
                ])],
                &[json!([
                    "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
                    [255, false]
                ])],
            )
            .unwrap()[0]
                .to_string(),
            r#"(Address :: from_str ("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE") . unwrap () , (255 , false))"#
        );
    }

    #[test]
    fn test_serde_to_token_streams_fixed_array() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::FixedArray(Box::new(ParamType::Uint(8)), 2)],
                &[json!([0, 255])],
            )
            .unwrap()[0]
                .to_string(),
            "[0 , 255]"
        );
    }

    #[test]
    fn test_serde_to_token_streams_fixed_array_nested() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::FixedArray(
                    Box::new(ParamType::FixedArray(
                        Box::new(ParamType::Tuple(vec![ParamType::Bool, ParamType::Bool])),
                        2
                    )),
                    2
                )],
                &[json!([
                    [(true, true), (false, false)],
                    [(true, false), (false, true)]
                ])],
            )
            .unwrap()[0]
                .to_string(),
            "[[(true , true) , (false , false)] , [(true , false) , (false , true)]]"
        );
    }

    #[test]
    fn test_serde_to_token_streams_array() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::Array(Box::new(ParamType::Bool))],
                &[json!([true, true, false, false, false])],
            )
            .unwrap()[0]
                .to_string(),
            "vec ! [true , true , false , false , false]"
        );
    }

    #[test]
    fn test_serde_to_token_streams_array_nested() {
        assert_eq!(
            serde_to_token_streams(
                &[ParamType::Array(Box::new(ParamType::FixedArray(
                    Box::new(ParamType::Tuple(vec![ParamType::Array(Box::new(
                        ParamType::Bool
                    ))])),
                    2
                )))],
                &[json!([[[[true, true]], [[false, false, false]]]])],
            )
            .unwrap()[0]
                .to_string(),
            "vec ! [[(vec ! [true , true]) , (vec ! [false , false , false])]]"
        );
    }

    #[test]
    fn test_serde_to_token_streams_fixed_bytes() {
        assert_eq!(
            serde_to_token_streams(&[ParamType::FixedBytes(2)], &[json!("0xffff")],).unwrap()[0]
                .to_string(),
            "[255 , 255]"
        );
        assert_eq!(
            serde_to_token_streams(&[ParamType::FixedBytes(2)], &[json!([255, 255])],).unwrap()[0]
                .to_string(),
            "[255 , 255]"
        );
        assert_eq!(
            serde_to_token_streams(&[ParamType::FixedBytes(2)], &[json!(" ~")],).unwrap()[0]
                .to_string(),
            "[32 , 126]"
        );
    }

    #[test]
    fn test_serde_to_token_streams_bytes() {
        assert_eq!(
            serde_to_token_streams(&[ParamType::Bytes], &[json!("0xffff")],).unwrap()[0]
                .to_string(),
            "vec ! [255 , 255]"
        );
        assert_eq!(
            serde_to_token_streams(&[ParamType::Bytes], &[json!([255, 255])],).unwrap()[0]
                .to_string(),
            "vec ! [255 , 255]"
        );
        assert_eq!(
            serde_to_token_streams(&[ParamType::Bytes], &[json!(" ~")],).unwrap()[0].to_string(),
            "vec ! [32 , 126]"
        );
    }

    #[test]
    fn test_serde_to_token_streams_string() {
        assert_eq!(
            serde_to_token_streams(&[ParamType::String], &[json!("0xffff")],).unwrap()[0]
                .to_string(),
            r#""0xffff" . into ()"#,
        );
    }

    #[test]
    fn test_to_candid_type() {
        assert_eq!(to_candid_type(&ParamType::Bool).0.to_string(), "bool");
        assert_eq!(to_candid_type(&ParamType::Uint(8)).0.to_string(), "u8");
        assert_eq!(to_candid_type(&ParamType::Uint(16)).0.to_string(), "u16");
        assert_eq!(to_candid_type(&ParamType::Uint(32)).0.to_string(), "u32");
        assert_eq!(to_candid_type(&ParamType::Uint(64)).0.to_string(), "u64");
        assert_eq!(to_candid_type(&ParamType::Uint(128)).0.to_string(), "u128");
        assert_eq!(
            to_candid_type(&ParamType::Uint(136)).0.to_string(),
            ":: std :: string :: String"
        );
        assert_eq!(
            to_candid_type(&ParamType::Uint(256)).0.to_string(),
            ":: std :: string :: String"
        );
        assert_eq!(to_candid_type(&ParamType::Int(8)).0.to_string(), "i8");
        assert_eq!(to_candid_type(&ParamType::Int(16)).0.to_string(), "i16");
        assert_eq!(to_candid_type(&ParamType::Int(32)).0.to_string(), "i32");
        assert_eq!(to_candid_type(&ParamType::Int(64)).0.to_string(), "i64");
        assert_eq!(to_candid_type(&ParamType::Int(128)).0.to_string(), "i128");
        assert_eq!(
            to_candid_type(&ParamType::Int(136)).0.to_string(),
            ":: std :: string :: String"
        );
        assert_eq!(
            to_candid_type(&ParamType::Int(256)).0.to_string(),
            ":: std :: string :: String"
        );
        assert_eq!(
            to_candid_type(&ParamType::Address).0.to_string(),
            ":: std :: string :: String"
        );
        assert_eq!(
            to_candid_type(&ParamType::Tuple(vec![])).0.to_string(),
            ":: ic_solidity_bindgen :: internal :: Empty"
        );
        assert_eq!(
            to_candid_type(&ParamType::Tuple(vec![ParamType::Uint(8), ParamType::Bool]))
                .0
                .to_string(),
            "(u8 , bool ,)"
        );
        assert_eq!(
            to_candid_type(&ParamType::Tuple(vec![
                ParamType::Bool,
                ParamType::Tuple(vec![ParamType::Bool])
            ]))
            .0
            .to_string(),
            "(bool , (bool ,) ,)"
        );
        assert_eq!(
            to_candid_type(&ParamType::FixedBytes(8)).0.to_string(),
            "[u8 ; 8usize]"
        );
        assert_eq!(
            to_candid_type(&ParamType::FixedBytes(256)).0.to_string(),
            "[u8 ; 256usize]"
        );
        assert_eq!(
            to_candid_type(&ParamType::Bytes).0.to_string(),
            ":: std :: vec :: Vec < u8 >"
        );
        assert_eq!(
            to_candid_type(&ParamType::FixedArray(Box::new(ParamType::Uint(64)), 2))
                .0
                .to_string(),
            ":: std :: vec :: Vec < u64 >"
        );
        assert_eq!(
            to_candid_type(&ParamType::Array(Box::new(ParamType::Uint(256))))
                .0
                .to_string(),
            ":: std :: vec :: Vec < :: std :: string :: String >"
        );
        assert_eq!(
            to_candid_type(&ParamType::Array(Box::new(ParamType::FixedArray(
                Box::new(ParamType::Uint(64)),
                2
            ))))
            .0
            .to_string(),
            ":: std :: vec :: Vec < :: std :: vec :: Vec < u64 > >"
        );
        assert_eq!(
            to_candid_type(&ParamType::FixedArray(
                Box::new(ParamType::Array(Box::new(ParamType::Uint(64)))),
                2
            ))
            .0
            .to_string(),
            ":: std :: vec :: Vec < :: std :: vec :: Vec < u64 > >"
        );
    }

    #[test]
    fn test_to_candid_value() {
        let accessor = quote! { res.0 };
        assert_eq!(
            to_candid_value(&ParamType::Bool, accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Uint(128), accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Uint(256), accessor.clone()).to_string(),
            quote! { res.0.to_string() }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Int(128), accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Int(256), accessor.clone()).to_string(),
            quote! { res.0 . 0 .to_string() }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Address, accessor.clone()).to_string(),
            quote! { res.0.to_string() }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Tuple(vec![]), accessor.clone()).to_string(),
            quote! { ::ic_solidity_bindgen::internal::Empty }.to_string()
        );
        assert_eq!(
            to_candid_value(
                &ParamType::Tuple(vec![
                    ParamType::Bool,
                    ParamType::Tuple(vec![ParamType::Uint(256)])
                ]),
                accessor.clone()
            )
            .to_string(),
            quote! { (res.0 . 0, (res.0 . 1 . 0.to_string(),),) }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::FixedBytes(8), accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::Bytes, accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(
                &ParamType::FixedArray(Box::new(ParamType::Bool), 2),
                accessor.clone()
            )
            .to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(
                &ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2),
                accessor.clone()
            )
            .to_string(),
            quote! { res.0.iter().map(|e| e.to_string()).collect::<Vec<_>>() }.to_string()
        );
        assert_eq!(
            to_candid_value(
                &ParamType::Array(Box::new(ParamType::Bool)),
                accessor.clone()
            )
            .to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_value(
                &ParamType::Array(Box::new(ParamType::Uint(256))),
                accessor.clone()
            )
            .to_string(),
            quote! { res.0.iter().map(|e| e.to_string()).collect::<Vec<_>>() }.to_string()
        );
        assert_eq!(
            to_candid_value(&ParamType::String, accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
    }

    #[test]
    fn test_to_candid_values() {
        let accessor = quote! { res.0 };
        assert_eq!(
            to_candid_values(&vec![ParamType::Bool,], accessor.clone()).to_string(),
            quote! { res.0 }.to_string()
        );
        assert_eq!(
            to_candid_values(
                &vec![
                    ParamType::Bool,
                    ParamType::Array(Box::new(ParamType::Uint(256)))
                ],
                accessor.clone()
            )
            .to_string(),
            quote! { (res.0 . 0, res.0 . 1.iter().map(|e| e.to_string()).collect::<Vec<_>>(),) }
                .to_string()
        );
    }
}
