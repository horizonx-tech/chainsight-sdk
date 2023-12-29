use std::ops::Deref;

use candid::types::{internal::is_primitive, Type, TypeInner};
use chainsight_cdk::{
    config::components::{LensParameter, RelayerConfig, LENS_FUNCTION_ARGS_TYPE},
    convert::candid::{extract_elements, get_candid_type_from_str},
    web3::ContractFunction,
};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::{canisters::utils::camel_to_snake, web3::ContractCall};

use super::utils::extract_contract_name_from_path;

pub fn def_relayer_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: RelayerConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    relayer_canister(config).into()
}

fn relayer_canister(config: RelayerConfig) -> proc_macro2::TokenStream {
    let common = common_code(config.clone());
    let custom = custom_code(config);
    quote! {
        #common
        #custom
    }
}

fn call_option() -> proc_macro2::TokenStream {
    quote! {
        let w3_ctx_param = get_web3_ctx_param();
        let call_option_builder = chainsight_cdk::web3::EVMTransactionOptionBuilder::new(
            w3_ctx_param.url,
            w3_ctx_param.chain_id,
            w3_ctx_param.env.ecdsa_key_name(),
        );
        use chainsight_cdk::web3::TransactionOptionBuilder;
        let call_option = call_option_builder.build().await.expect("Failed to build call_option");
    }
}

fn custom_code(config: RelayerConfig) -> proc_macro2::TokenStream {
    let RelayerConfig {
        common,
        method_identifier,
        abi_file_path,
        lens_parameter,
        method_name,
        ..
    } = config;

    let canister_name_ident = format_ident!("{}", common.canister_name);
    let (source_method_name, _, canister_response_type) =
        extract_elements(&method_identifier).expect("Failed to extract_elements");
    let call_option_ident = call_option();

    let call_args_ident =
        inter_canister_call_args_ident(canister_name_ident.clone(), lens_parameter.clone());
    let source_ident = source_ident(source_method_name.clone(), lens_parameter);
    let proxy_method_name = "proxy_".to_string() + &source_method_name;
    let contract_call = ContractCall::new(ContractFunction::new(
        abi_file_path.clone(),
        method_name.clone(),
    ));
    let method_call_ident = method_call(
        contract_call.clone(),
        &abi_file_path.clone(),
        &canister_response_type,
        canister_name_ident.clone(),
    );
    let generated = quote! {
        ic_solidity_bindgen::contract_abi!(#abi_file_path);
        use #canister_name_ident::{CallCanisterResponse, filter};
        #call_args_ident
        #source_ident
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted");
            }
            let target_canister = candid::Principal::from_text(get_target_canister()).expect("Failed to parse to candid::Principal");
            let call_result = CallProvider::new()
                .call(
                    Message::new::<CallCanisterArgs>(
                        call_args(),
                        _get_target_proxy(target_canister.clone()).await,
                        #proxy_method_name
                    ).expect("failed to create message")
                )
                .await.expect("failed to call by CallProvider");
            let datum = call_result.reply::<CallCanisterResponse>().expect("failed to get reply");


            ic_cdk::println!("response from canister = {:?}", datum.clone());

            if !filter(&datum) {
                return;
            }
            #call_option_ident
            #method_call_ident
        }

    };
    generated
}

fn inter_canister_call_args_ident(
    canister_name_ident: Ident,
    lens_param: Option<LensParameter>,
) -> proc_macro2::TokenStream {
    match lens_param {
        None => {
            quote! {
                type CallCanisterArgs = #canister_name_ident::CallCanisterArgs;
                pub fn call_args() -> CallCanisterArgs {
                    #canister_name_ident::call_args()
                }
            }
        }
        Some(p) => {
            let call_args_ident = match p.with_args {
                true => {
                    let lens_args_ident = format_ident!("{}", LENS_FUNCTION_ARGS_TYPE);
                    quote! {
                       type CallCanisterArgs = #canister_name_ident::#lens_args_ident;
                       #[ic_cdk::query]
                       #[candid::candid_method(query)]
                       pub fn call_args() -> CallCanisterArgs {
                           #canister_name_ident::#lens_args_ident {
                               targets: get_lens_targets(),
                               args: #canister_name_ident::call_args(),
                           }
                       }
                    }
                }
                _ => {
                    quote! {
                       type CallCanisterArgs = Vec<String>;
                       #[ic_cdk::query]
                       #[candid::candid_method(query)]
                       pub fn call_args() -> CallCanisterArgs {
                           get_lens_targets()
                       }
                    }
                }
            };
            quote! {
                manage_single_state!("lens_targets", Vec<String>, false);
                #call_args_ident
            }
        }
    }
}

fn source_ident(
    source_method_name: String,
    lens_param: Option<LensParameter>,
) -> proc_macro2::TokenStream {
    match lens_param {
        None => quote! {
             relayer_source!(#source_method_name);
        },
        _ => {
            quote! { relayer_source!(#source_method_name, "get_lens_targets"); }
        }
    }
}

fn method_call(
    call: ContractCall,
    abi_file_path: &String,
    response_type_str: &str,
    canister_name_ident: Ident,
) -> proc_macro2::TokenStream {
    let func = call.function().function();
    let method_name_ident = format_ident!("{}", camel_to_snake(&func.name));
    let oracle_name = extract_contract_name_from_path(abi_file_path);
    let oracle_ident = format_ident!("{}", oracle_name);

    match func.inputs.len() {
        0 => quote! {
            let result = #oracle_ident::new(
                Address::from_str(&get_target_addr()).expect("Failed to parse target addr to Address"),
                &web3_ctx().expect("Failed to get web3_ctx")
            ).#method_name_ident(call_option).await.expect("Failed to call update_state for oracle");
        },
        1 => {
            let response_type = get_candid_type_from_str(response_type_str);
            let data = if response_type.is_ok() && is_ethabi_encodable_type(&response_type.unwrap())
            {
                quote! {
                    chainsight_cdk::web3::abi::EthAbiEncoder.encode(datum.clone())
                }
            } else {
                quote! { format!("{:?}", &datum).into_bytes() }
            };
            quote! {
                let result = #oracle_ident::new(
                    Address::from_str(&get_target_addr()).expect("Failed to parse target addr to Address"),
                    &web3_ctx().expect("Failed to get web3_ctx")
                ).#method_name_ident(#data, call_option).await.expect("Failed to call update_state for oracle");
            }
        }
        _ => {
            let args_ident: Vec<Ident> = call
                .field_names()
                .iter()
                .map(|p| format_ident!("{}", p))
                .collect();
            quote! {
                let value =  #canister_name_ident::convert(&datum.clone());
                let result = #oracle_ident::new(
                    Address::from_str(&get_target_addr()).expect("Failed to parse target addr to Address"),
                    &web3_ctx().expect("Failed to get web3_ctx")
                ).#method_name_ident(#(value.#args_ident),*, call_option).await.expect("Failed to call update_state for oracle");
            }
        }
    }
}

fn common_code(config: RelayerConfig) -> proc_macro2::TokenStream {
    let RelayerConfig {
        common,
        lens_parameter,
        ..
    } = config;

    let canister_name = &common.canister_name.clone();
    let lens_targets_quote = if lens_parameter.is_some() {
        quote! { lens_targets: Vec<String> }
    } else {
        quote! {}
    };
    quote! {
        use ic_cdk::api::call::result;
        use std::str::FromStr;
        use chainsight_cdk_macros::{manage_single_state, setup_func, init_in, timer_task_func, define_web3_ctx, define_transform_for_web3, define_get_ethereum_address, chainsight_common, did_export, relayer_source};
        use ic_web3_rs::types::{Address, U256};
        use chainsight_cdk::rpc::{CallProvider, Caller, Message};
        use chainsight_cdk::web3::Encoder;
        did_export!(#canister_name);  // NOTE: need to be declared before query, update
        chainsight_common!();
        define_web3_ctx!();
        define_transform_for_web3!();
        manage_single_state!("target_addr", String, false);
        define_get_ethereum_address!();
        manage_single_state!("target_canister", String, false);
        timer_task_func!("set_task", "index");
        init_in!();
        setup_func!({
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam,
            target_canister: String,
            #lens_targets_quote
        });
    }
}

/// Whether EthAbiEncoder is available
fn is_ethabi_encodable_type(canister_response_type: &Type) -> bool {
    // NOTE: Custom type is Unknown because it does not contain did definitions on which the custom type depends.
    //       Considering the possibility of panic with is_primitive if Unknown
    //       https://github.com/dfinity/candid/blob/2022-11-17/rust/candid/src/types/internal.rs#L353-L368
    match canister_response_type.deref() {
        TypeInner::Unknown | TypeInner::Var(_) => false,
        _ => is_primitive(canister_response_type),
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_is_ethabi_encodable_type() {
        assert!(is_ethabi_encodable_type(&TypeInner::Text.into()));
        assert!(is_ethabi_encodable_type(&TypeInner::Nat.into()));
        assert!(is_ethabi_encodable_type(&TypeInner::Int64.into()));
        assert!(is_ethabi_encodable_type(&TypeInner::Float32.into()));
        // check no panic
        assert!(!is_ethabi_encodable_type(&TypeInner::Unknown.into()));
        assert!(!is_ethabi_encodable_type(
            &TypeInner::Var("Snapshot".to_string()).into()
        ));
    }

    fn config() -> RelayerConfig {
        RelayerConfig {
            common: CommonConfig {
                canister_name: "relayer".to_string(),
            },
            destination: "0539a0EF8e5E60891fFf0958A059E049e43020d9".to_string(),
            method_identifier: "get_last_snapshot_value : () -> (text)".to_string(),
            abi_file_path: "__interfaces/Uint256Oracle.json".to_string(),
            lens_parameter: None,
            method_name: "update_state".to_string(),
        }
    }

    #[test]
    fn test_snapshot() {
        let generated = relayer_canister(config());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__relayer", formatted);
    }

    #[test]
    fn test_snapshot_with_lens() {
        let mut config = config();
        config.lens_parameter = Some(LensParameter { with_args: false });
        let generated = relayer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__relayer__with_lens", formatted);
    }

    #[test]
    fn test_snapshot_with_lens_with_args() {
        let mut config = config();
        config.lens_parameter = Some(LensParameter { with_args: true });
        let generated = relayer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__relayer__with_lens_with_args", formatted);
    }
}
