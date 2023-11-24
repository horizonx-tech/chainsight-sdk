use std::{borrow::Borrow, str::FromStr as _};

use anyhow::Error;
use chainsight_cdk::config::components::{CommonConfig, SnapshotIndexerEVMConfig};
use darling::FromMeta;
use ic_web3_rs::ethabi::{Param, ParamType};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, ToTokens as _};
use syn::parse_macro_input;

use crate::canisters::utils::{
    camel_to_snake, extract_contract_name_from_path, generate_queries_without_timestamp,
    update_funcs_to_upgrade,
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
        use chainsight_cdk_macros::{init_in, manage_single_state, setup_func, prepare_stable_structure, stable_memory_for_vec, StableMemoryStorable, timer_task_func, define_transform_for_web3, define_web3_ctx, chainsight_common, did_export, CborSerde, snapshot_indexer_web3_source};
        use ic_stable_structures::writer::Writer;
        use ic_web3_rs::types::Address;

        did_export!(#canister_name); // NOTE: need to be declared before query, update

        init_in!();
        chainsight_common!();

        define_web3_ctx!();
        define_transform_for_web3!();
        manage_single_state!("target_addr", String, false);
        setup_func!({
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
        });

        prepare_stable_structure!();
        stable_memory_for_vec!("snapshot", Snapshot, 1, true);
        timer_task_func!("set_task", "index");
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
        .ok_or_else(|| anyhow::anyhow!("Invalid function indetifier: {}", signature))
        .unwrap();
    let functions = contract
        .functions_by_name(name)
        .unwrap_or_else(|_| panic!("function not found. name: {}", name));
    let function = functions
        .iter()
        .find(|f| f.signature() == signature)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "function not found. indetifier: {}, available: {}",
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

    let response_values = to_candid_values(&function.outputs, quote! { res });

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

    let quote_to_upgradable = {
        let state_struct = quote! {
            #[derive(Clone, Debug, PartialEq, candid::CandidType, serde::Serialize, serde::Deserialize, CborSerde)]
            pub struct UpgradeStableState {
                pub proxy: candid::Principal,
                pub initialized: bool,
                pub env: chainsight_cdk::core::Env,
                pub web3_ctx_param: chainsight_cdk::web3::Web3CtxParam,
                pub target_addr: String,
                pub indexing_interval: u32,
            }
        };

        let update_funcs_to_upgrade = update_funcs_to_upgrade(
            quote! {
                UpgradeStableState {
                    proxy: get_proxy(),
                    initialized: is_initialized(),
                    env: get_env(),
                    web3_ctx_param: get_web3_ctx_param(),
                    target_addr: get_target_addr(),
                    indexing_interval: get_indexing_interval(),
                }
            },
            quote! {
                set_initialized(state.initialized);
                set_proxy(state.proxy);
                set_env(state.env);
                setup(
                    state.target_addr,
                    state.web3_ctx_param,
                ).expect("Failed to `setup` in post_upgrade");
                set_indexing_interval(state.indexing_interval);
            },
        );

        quote! {
            #state_struct
            #update_funcs_to_upgrade
        }
    };

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
            ).#method_ident(#(#request_arg_tokens,)*None).await.expect("Failed to call contract");

            let datum = Snapshot {
                value: #response_values,
                timestamp: current_ts_sec,
            };
            let _ = add_snapshot(datum.clone());

            ic_cdk::println!("ts={}, snapshot={:?}", datum.timestamp, datum.value);
        }

        #quote_to_upgradable
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
                let lit = proc_macro2::Literal::from_bool(val).unwrap();
                quote! { #lit }
            }
            ParamType::Uint(size) => match size {
                1..=128 => {
                    let val = if value.is_string() {
                        u128::from_str(value.as_str().unwrap()).unwrap()
                    } else {
                        value.as_u64().unwrap() as u128
                    };
                    let lit = proc_macro2::Literal::u128_unsuffixed(val);
                    quote! { #lit }
                }
                129..=256 => {
                    let val = value.as_str().unwrap().to_string();
                    quote! { ic_web3_rs::types::U256::from_dec_str(#val).unwrap() }
                }
                _ => {
                    let val = if value.is_string() {
                        value.as_str().unwrap().to_string()
                    } else {
                        value.as_u64().unwrap().to_string()
                    };
                    quote! { ic_web3_rs::types::U256::from_dec_str(#val.to_string()).unwrap() }
                }
            },
            ParamType::Int(size) => match size {
                1..=128 => {
                    let val = if value.is_string() {
                        i128::from_str(value.as_str().unwrap()).unwrap()
                    } else {
                        value.as_u64().unwrap() as i128
                    };
                    let lit = proc_macro2::Literal::i128_unsuffixed(val);
                    quote! { #lit }
                }
                129..=256 => {
                    let val = value.as_str().unwrap().to_string();
                    quote! { ic_web3_rs::types::U256::from_dec_str(#val).unwrap() }
                }
                _ => {
                    let val = if value.is_string() {
                        value.as_str().unwrap().to_string()
                    } else {
                        value.as_u64().unwrap().to_string()
                    };
                    quote! { ic_web3_rs::types::U256::from_dec_str(#val.to_string()).unwrap() }
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
                quote! { vec![#(#types),*] }
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
                if value.is_string() {
                    let bytes = value.as_str().unwrap();
                    quote! { b #bytes.to_vec() }
                } else if value.is_array() {
                    let vals = value.as_array().unwrap();
                    let bytes_vec = vals
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect::<Vec<_>>();
                    let bytes = hex::encode(bytes_vec);
                    quote! { b #bytes.to_vec() }
                } else {
                    panic!("Unexpected value for FixedBytes: {}", value)
                }
            }
            ParamType::Bytes => {
                if value.is_string() {
                    let bytes = value.as_str().unwrap();
                    quote! { b #bytes.to_vec() }
                } else if value.is_array() {
                    let vals = value.as_array().unwrap();
                    let bytes_vec = vals
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect::<Vec<_>>();
                    let bytes = hex::encode(bytes_vec);
                    quote! { b #bytes.to_vec() }
                } else {
                    panic!("Unexpected value for FixedBytes: {}", value)
                }
            }
            ParamType::String => {
                let val = value.as_str().unwrap();
                quote! { #val }
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
        ParamType::Address => (quote! { ::std::string::String }, 0),
        ParamType::Bytes => (quote! { ::std::vec::Vec<u8> }, 0),
        ParamType::Int(size) => match size {
            129..=256 => (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0),
            65..=128 => (ident("i128").to_token_stream(), 0),
            33..=64 => (ident("i64").to_token_stream(), 0),
            17..=32 => (ident("i32").to_token_stream(), 0),
            9..=16 => (ident("i16").to_token_stream(), 0),
            1..=8 => (ident("i8").to_token_stream(), 0),
            _ => (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0),
        },
        ParamType::Uint(size) => match size {
            129..=256 => (quote! {  ::std::string::String }, 0),
            65..=128 => {
                let name = ident("u128");
                (quote! { #name }, 0)
            }
            33..=64 => {
                let name = ident("u64");
                (quote! { #name }, 0)
            }
            17..=32 => {
                let name = ident("u32");
                (quote! { #name }, 0)
            }
            1..=16 => {
                let name = ident("u16");
                (quote! { #name }, 0)
            }
            _ => (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0),
        },
        ParamType::Bool => (quote! { bool }, 0),
        ParamType::String => (quote! { ::std::string::String }, 0),
        ParamType::Array(inner) => {
            let (inner, nesting) = to_candid_type(inner);
            if nesting > 0 {
                (quote! { ::ic_solidity_bindgen::internal::Unimplemented }, 0)
            } else {
                (quote! { ::std::vec::Vec<#inner> }, nesting)
            }
        }
        ParamType::FixedBytes(len) => (quote! { [ u8; #len ] }, 0),
        ParamType::FixedArray(inner, len) => {
            let (inner, nesting) = to_candid_type(inner);
            (quote! { [#inner; #len] }, nesting)
        }
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
    }
}

fn to_candid_values(
    outputs: &Vec<Param>,
    accessor: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if outputs.len() == 1 {
        let value = to_candid_value(&outputs[0].kind, quote! { #accessor });
        return quote! { #value };
    }
    let mut values = vec![];
    for (i, output) in outputs.iter().enumerate() {
        let idx_lit = proc_macro2::Literal::usize_unsuffixed(i);
        values.push(to_candid_value(&output.kind, quote! { #accessor.#idx_lit }));
    }
    quote! { (#(#values,)*) }
}

fn to_candid_value(
    kind: &ParamType,
    accessor: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match kind {
        ParamType::Address => quote! {  #accessor.to_string() },
        ParamType::Bytes => quote! { #accessor },
        ParamType::Int(size) => match size {
            1..=128 => quote! { #accessor },
            129..=256 => quote! {  #accessor.to_string() },
            _ => quote! { #accessor.to_string()},
        },
        ParamType::Uint(size) => match size {
            1..=128 => quote! { #accessor },
            129..=256 => quote! { #accessor.to_string() },
            _ => quote! {  #accessor.to_string() },
        },
        ParamType::Bool => quote! { #accessor },
        ParamType::String => quote! { #accessor },
        ParamType::Array(param_type) => {
            if should_convert(param_type.as_ref()) {
                let inner = to_candid_value(param_type.as_ref(), quote! { e });
                quote! { #accessor.iter().map(|e| #inner).collect() }
            } else {
                quote! { #accessor }
            }
        }
        ParamType::FixedBytes(_) => quote! { #accessor },
        ParamType::FixedArray(param_type, _) => {
            if should_convert(param_type.as_ref()) {
                let inner = to_candid_value(param_type.as_ref(), quote! { e });
                quote! { #accessor.iter().map(|e| #inner).collect() }
            } else {
                quote! { #accessor }
            }
        }
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
}
