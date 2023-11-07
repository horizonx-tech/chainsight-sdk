use anyhow::bail;
use chainsight_cdk::{
    config::components::{CommonConfig, SnapshotIndexerEVMConfig},
    convert::evm::{ContractMethodIdentifier, ADDRESS_TYPE, U256_TYPE},
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
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
    let id = &config.canister_name;
    let duration = config.monitor_duration;

    quote! {
        use std::str::FromStr;
        use candid::{Decode, Encode};
        use chainsight_cdk_macros::{init_in, manage_single_state, setup_func, prepare_stable_structure, stable_memory_for_vec, StableMemoryStorable, timer_task_func, define_transform_for_web3, define_web3_ctx, chainsight_common, did_export, snapshot_web3_source};

        use ic_web3_rs::types::Address;

        did_export!(#id); // NOTE: need to be declared before query, update

        init_in!();
        chainsight_common!(#duration);

        define_web3_ctx!();
        define_transform_for_web3!();
        manage_single_state!("target_addr", String, false);
        setup_func!({
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
        });

        prepare_stable_structure!();
        stable_memory_for_vec!("snapshot", Snapshot, 0, true);
        timer_task_func!("set_task", "index", true);
    }
}

fn custom_code(config: SnapshotIndexerEVMConfig) -> proc_macro2::TokenStream {
    let SnapshotIndexerEVMConfig {
        method_identifier,
        method_args,
        abi_file_path,
        ..
    } = config;

    let method_identifier = ContractMethodIdentifier::parse_from_str(&method_identifier)
        .expect("Failed to parse method identifier");
    let method_ident_str = camel_to_snake(&method_identifier.identifier);
    let method_ident = format_ident!("{}", method_ident_str);

    let contract_struct_ident =
        format_ident!("{}", extract_contract_name_from_path(&abi_file_path));

    // for request values
    assert!(method_identifier.params.len() == method_args.len(), "datatource.method is not valid: The number of params in 'identifier' and 'args' must be the same");

    let method_args = method_args
        .iter()
        .enumerate()
        .map(|(idx, arg)| (method_identifier.params[idx].clone(), arg.clone()))
        .collect();
    let (request_val_idents, _) = generate_request_arg_idents(&method_args);

    // for response types & response values
    let mut response_type_idents: Vec<syn::Ident> = vec![];
    let mut response_val_idents: Vec<proc_macro2::TokenStream> = vec![];
    let response_types = method_identifier.return_value;
    match response_types.len() {
        0 => panic!("datatource.method.identifier is not valid: Response required"),
        1 => {
            // If it's a single type, we process it like we did before
            let ty =
                syn::parse_str::<syn::Type>(&response_types[0]).expect("Failed to syn::parse_str");
            let (response_type_ident, response_val_ident) =
                match_primitive_type(&ty, None).expect("Failed to match primitive type");
            response_type_idents.push(response_type_ident);
            response_val_idents.push(response_val_ident);
        }
        _ => {
            // If it's not a single type, it must be a tuple
            // In this case, we process it like we did before
            for (idx, elem) in response_types.iter().enumerate() {
                let ty = syn::parse_str::<syn::Type>(elem).expect("Failed to syn::parse_str");
                let idx_lit = proc_macro2::Literal::usize_unsuffixed(idx);
                let (response_type_ident, response_val_ident) =
                    match_primitive_type(&ty, Some(idx_lit))
                        .expect("Failed to match primitive type");
                response_type_idents.push(response_type_ident);
                response_val_idents.push(response_val_ident);
            }
        }
    };

    // consider whether to add timestamp information to the snapshot
    let (snapshot_idents, queries_expect_timestamp) = (
        quote! {
            #[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize, StableMemoryStorable)]
            #[stable_mem_storable_opts(max_size = 10000, is_fixed_size = false)] // temp: max_size
            pub struct Snapshot {
                pub value: SnapshotValue,
                pub timestamp: u64,
            }
            type SnapshotValue = (#(#response_type_idents),*);
        },
        generate_queries_without_timestamp(format_ident!("SnapshotValue")),
    );

    quote! {
        #snapshot_idents

        #queries_expect_timestamp

        ic_solidity_bindgen::contract_abi!(#abi_file_path);
        snapshot_web3_source!(#method_ident_str);

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }

            let current_ts_sec = ic_cdk::api::time() / 1000000;
            let res = #contract_struct_ident::new(
                Address::from_str(&get_target_addr()).unwrap(),
                &web3_ctx().unwrap()
            ).#method_ident(#(#request_val_idents,)*None).await.unwrap();

            let datum = Snapshot {
                value: (
                    #(#response_val_idents),*
                ),
                timestamp: current_ts_sec,
            };
            let _ = add_snapshot(datum.clone());

            ic_cdk::println!("ts={}, snapshot={:?}", datum.timestamp, datum.value);
        }
    }
}

fn match_primitive_type(
    ty: &syn::Type,
    idx: Option<proc_macro2::Literal>,
) -> anyhow::Result<(proc_macro2::Ident, proc_macro2::TokenStream)> {
    let res = match ty {
        syn::Type::Path(type_path) => {
            let mut type_string = quote! { #type_path }.to_string();
            type_string.retain(|c| !c.is_whitespace());

            match type_string.as_str() {
                U256_TYPE => (
                    format_ident!("String"),
                    match idx {
                        Some(idx_lit) => quote! { res.#idx_lit.to_string() },
                        None => quote! { res.to_string() },
                    },
                ),
                ADDRESS_TYPE => (
                    format_ident!("String"),
                    match idx {
                        Some(idx_lit) => quote! { hex::encode(res.#idx_lit) },
                        None => quote! { hex::encode(res) },
                    },
                ),
                _ => (
                    format_ident!("{}", type_string),
                    match idx {
                        Some(idx_lit) => quote! { res.#idx_lit },
                        None => quote! { res },
                    },
                ),
            }
        }
        _ => bail!("Unsupported type"),
    };
    Ok(res)
}

// Generate the part of data of the argument that calls the function of datasource contract/canister
pub fn generate_request_arg_idents(
    method_args: &Vec<(String, serde_json::Value)>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::Ident>) {
    let mut value_idents = vec![];
    let mut type_idents = vec![];
    for method_arg in method_args {
        let (type_, value) = method_arg;
        // temp
        let request_arg_value = match type_.clone().as_str() {
            U256_TYPE => match value {
                serde_json::Value::String(val) => {
                    quote! { ic_web3_rs::types::U256::from_dec_str(#val).unwrap() }
                }
                serde_json::Value::Number(val) => match val.as_u64() {
                    Some(val) => quote! { #val.into() },
                    None => quote! {},
                },
                _ => quote! {},
            },
            ADDRESS_TYPE => match value {
                serde_json::Value::String(val) => {
                    quote! { ic_web3_rs::types::Address::from_str(#val).unwrap() }
                }
                _ => quote! {},
            },
            _ => match value {
                serde_json::Value::String(val) => {
                    quote! { #val, }
                }
                serde_json::Value::Number(val) => match val.as_u64() {
                    Some(val) => {
                        let type_ident = format_ident!("{}", type_);
                        quote! { #val as #type_ident }
                    }
                    None => {
                        quote! {}
                    }
                },
                _ => {
                    quote! {}
                }
            },
        };
        value_idents.push(request_arg_value);
        if type_ == U256_TYPE || type_ == ADDRESS_TYPE {
            // In the case of contract, other than the primitive type (ic_web3_rs::types::U256 etc.) may be set, in which case type_idents is not used.
            type_idents.push(format_ident!("String")); // temp: thread 'main' panicked at '"ic_web3_rs::types::U256" is not a valid Ident'
        } else {
            type_idents.push(format_ident!("{}", type_));
        }
    }
    (value_idents, type_idents)
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
                monitor_duration: 60,
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
