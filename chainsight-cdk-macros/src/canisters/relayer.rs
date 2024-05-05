use std::ops::Deref;

use candid::types::{internal::is_primitive, Type, TypeInner};
use chainsight_cdk::{
    config::components::{
        LensParameter, RelayerConfig, RelayerConversionParameter, LENS_FUNCTION_ARGS_TYPE,
    },
    convert::candid::{extract_elements, get_candid_type_from_str},
    web3::ContractFunction,
};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use regex::Regex;
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

fn custom_code(config: RelayerConfig) -> proc_macro2::TokenStream {
    let RelayerConfig {
        common,
        method_identifier,
        abi_file_path,
        lens_parameter,
        conversion_parameter,
        method_name,
        ..
    } = config;

    let canister_name_ident = format_ident!("{}", common.canister_name);
    let (source_method_name, _, canister_response_type) =
        extract_elements(&method_identifier).expect("Failed to extract_elements");
    let call_option_ident = call_option();

    let call_args_ident =
        inter_canister_call_args_ident(canister_name_ident.clone(), lens_parameter.clone());
    let source_ident = source_ident(&source_method_name, lens_parameter.clone());
    let proxy_method_name = "proxy_".to_string() + &source_method_name;

    let (extracted_datum_ident, converted_datum_ident) =
        if let Some(conversion_parameter) = conversion_parameter {
            let RelayerConversionParameter {
                extracted_field,
                destination_type_to_convert,
                exponent_of_power10,
            } = conversion_parameter;

            let extracted_datum_ident = if let Some(chaining) = extracted_field {
                let chaining = if let Some(stripped) = chaining.strip_prefix('.') {
                    stripped.to_string()
                } else {
                    chaining
                };
                convert_chaining_str_to_token(&("datum.".to_string() + &chaining))
            } else {
                quote! { datum }
            };
            let converted_datum_ident = if let Some(dst_type_str) = destination_type_to_convert {
                let dst_ty = format_ident!("{}", dst_type_str);
                let exp_pow10 = exponent_of_power10.unwrap_or(0);
                quote! { {
                    let converted: #dst_ty = datum.convert(#exp_pow10);
                    converted
                } }
            } else if let Some(exp_pow10) = exponent_of_power10 {
                quote! { datum.scale(#exp_pow10) }
            } else {
                quote! { datum }
            };

            (extracted_datum_ident, converted_datum_ident)
        } else {
            (quote! { datum }, quote! { datum })
        };

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
            let datum = #extracted_datum_ident;
            ic_cdk::println!("val extracted from response = {:?}", datum.clone());
            let datum = #converted_datum_ident;
            ic_cdk::println!("val converted from extracted = {:?}", datum.clone());

            #call_option_ident
            #method_call_ident
        }
    };

    generated
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
                               targets: get_lens_targets().into(),
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
                           get_lens_targets().into()
                       }
                    }
                }
            };
            quote! {
                stable_memory_for_scalar!("lens_targets", chainsight_cdk::storage::StorableStrings, 5, false);
                #call_args_ident
            }
        }
    }
}

fn source_ident(
    source_method_name: &String,
    lens_param: Option<LensParameter>,
) -> proc_macro2::TokenStream {
    match lens_param {
        None => quote! {
             relayer_source!(#source_method_name);
        },
        _ => quote! { relayer_source!(#source_method_name, "get_lens_targets"); },
    }
}

fn method_call(
    call: ContractCall,
    abi_file_path: &str,
    response_type_str: &str,
    canister_name_ident: Ident,
) -> proc_macro2::TokenStream {
    let func = call.function().function();
    let oracle_name = extract_contract_name_from_path(abi_file_path);
    let oracle_ident = format_ident!("{}", oracle_name);
    let oracle_func = camel_to_snake(&call.function().function().name.clone());
    let oracle_func_ident = format_ident!("{}", oracle_func);

    match func.inputs.len() {
        0 => {
            quote! {
                let result = #oracle_ident::new(
                    Address::from_str(&get_target_addr()).expect("Failed to parse target addr to Address"),
                    &relayer_web3_ctx().await.expect("Failed to get web3_ctx")
                ).#oracle_func_ident(call_option).await.expect("Failed to call update_state for oracle");
            }
        }
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
                    &relayer_web3_ctx().await.expect("Failed to get web3_ctx")
                ).#oracle_func_ident(#data, call_option).await.expect("Failed to call update_state for oracle");
                ic_cdk::println!("value_to_sync={:?}", result);
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
                    &relayer_web3_ctx().await.expect("Failed to get web3_ctx")
                ).#oracle_func_ident(#(value.#args_ident),*, call_option).await.expect("Failed to call update_state for oracle");
            }
        }
    }
}

fn convert_chaining_str_to_token(base: &str) -> proc_macro2::TokenStream {
    let re_one_item_in_vec = Regex::new(r"^([^\[]+)\[(\d+)\]$").unwrap();

    let field_tokens = base
        .split('.')
        .map(|p| {
            let res: Box<dyn quote::ToTokens> = if p.parse::<i64>().is_ok() {
                // only number
                Box::new(proc_macro2::Literal::i64_unsuffixed(
                    p.parse::<i64>().unwrap(),
                ))
            } else if let Some(captures) = re_one_item_in_vec.captures(p) {
                // one item in vec
                let field = format_ident!("{}", captures.get(1).unwrap().as_str());
                let index = proc_macro2::Literal::u64_unsuffixed(
                    captures.get(2).unwrap().as_str().parse::<u64>().unwrap(),
                );
                Box::new(quote! { #field[#index] })
            } else {
                // only words
                Box::new(format_ident!("{}", p))
            };
            res
        })
        .collect::<Vec<Box<dyn quote::ToTokens>>>();

    quote! { #(#field_tokens).* }
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
        use candid::{Decode, Encode};
        use ic_cdk::api::call::result;
        use std::str::FromStr;
        use chainsight_cdk_macros::{manage_single_state, setup_func, init_in, timer_task_func, define_web3_ctx, define_relayer_web3_ctx, define_transform_for_web3, define_get_ethereum_address, chainsight_common, did_export, prepare_stable_structure, stable_memory_for_scalar, StableMemoryStorable, CborSerde, relayer_source};
        use chainsight_cdk::rpc::{CallProvider, Caller, Message};
        use chainsight_cdk::web3::Encoder;
        use chainsight_cdk::convert::scalar::{Convertible, Scalable};
        use ic_stable_structures::writer::Writer;
        use ic_web3_rs::types::{Address, U256};
        did_export!(#canister_name);  // NOTE: need to be declared before query, update
        chainsight_common!();
        define_relayer_web3_ctx!(2);
        define_transform_for_web3!();
        stable_memory_for_scalar!("target_addr", String, 3, false);
        define_get_ethereum_address!();
        stable_memory_for_scalar!("target_canister", String, 4, false);
        prepare_stable_structure!();
        timer_task_func!("set_task", "index", 7);
        init_in!(1);
        setup_func!({
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam,
            target_canister: String,
            #lens_targets_quote
        }, 6);
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
    use chainsight_cdk::config::components::{CommonConfig, RelayerConversionParameter};
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

    #[test]
    fn test_convert_chaining_str_to_token() {
        assert_eq!(convert_chaining_str_to_token("datum").to_string(), "datum");
        assert_eq!(
            convert_chaining_str_to_token("datum.value.dai.usd").to_string(),
            "datum . value . dai . usd"
        );
        assert_eq!(
            convert_chaining_str_to_token("datum.value.0").to_string(),
            "datum . value . 0"
        );
        assert_eq!(
            convert_chaining_str_to_token("chart.result[0].meta.regular_market_price").to_string(),
            "chart . result [0] . meta . regular_market_price"
        );
    }

    fn config() -> RelayerConfig {
        RelayerConfig {
            common: CommonConfig {
                canister_name: "relayer".to_string(),
            },
            method_identifier: "get_last_snapshot_value : () -> (text)".to_string(),
            destination: "0539a0EF8e5E60891fFf0958A059E049e43020d9".to_string(),
            abi_file_path: "__interfaces/Uint256Oracle.json".to_string(),
            method_name: "update_state".to_string(),
            conversion_parameter: None,
            lens_parameter: None,
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

    #[test]
    fn test_snapshot_with_extracted_val_from_response() {
        let mut config = config();
        config.method_identifier =
            "get_last_snapshot : () -> (record { value : text; timestamp : nat64; })".to_string();
        config.conversion_parameter = Some(RelayerConversionParameter {
            extracted_field: Some("value".to_string()),
            destination_type_to_convert: None,
            exponent_of_power10: None,
        });

        let generated = relayer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!(
            "snapshot__relayer__with_extracted_val_from_response",
            formatted
        );
    }

    #[test]
    fn test_snapshot_with_scaled_val_from_extracted() {
        let mut config = config();
        config.conversion_parameter = Some(RelayerConversionParameter {
            extracted_field: None,
            destination_type_to_convert: None,
            exponent_of_power10: Some(3),
        });

        let generated = relayer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!(
            "snapshot__relayer__with_scaled_val_from_extracted",
            formatted
        );
    }

    #[test]
    fn test_snapshot_with_converted_val_from_extracted() {
        let mut config = config();
        config.conversion_parameter = Some(RelayerConversionParameter {
            extracted_field: None,
            destination_type_to_convert: Some("U256".to_string()),
            exponent_of_power10: Some(3),
        });

        let generated = relayer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!(
            "snapshot__relayer__with_converted_val_from_extracted",
            formatted
        );
    }
}
