use candid::types::{internal::is_primitive, Type};
use chainsight_cdk::{
    config::components::RelayerConfig, convert::candid::CanisterMethodIdentifier,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::utils::extract_contract_name_from_path;

pub fn def_relayer_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: RelayerConfig = serde_json::from_str(&input_json_string).unwrap();
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
        lens_targets,
        ..
    } = config;

    let canister_name_ident = format_ident!("{}", common.canister_name);
    let canister_method = CanisterMethodIdentifier::new(&method_identifier)
        .expect("Failed to parse method_identifer");
    let method_name = canister_method.identifier.as_str();
    let canister_response_type = {
        let (_, response_type) = canister_method.get_types();
        response_type.expect("Failed to get canister_response_type")
    };
    let sync_data_ident = generate_ident_sync_to_oracle(canister_response_type);

    let (call_args_ident, relayer_source_ident) = if lens_targets.is_some() {
        (
            quote! {
                type CallCanisterArgs = Vec<String>;
                pub fn call_args() -> CallCanisterArgs {
                    get_lens_targets()
                }
            },
            quote! {
                relayer_source!(#method_name, true);
                manage_single_state!("lens_targets", Vec<String>, false);
            },
        )
    } else {
        (
            quote! {
                type CallCanisterArgs = #canister_name_ident::CallCanisterArgs;
                pub fn call_args() -> CallCanisterArgs {
                    #canister_name_ident::call_args()
                }
            },
            quote! { relayer_source!(#method_name, false); },
        )
    };
    let oracle_name = extract_contract_name_from_path(&abi_file_path);
    let oracle_ident = format_ident!("{}", oracle_name);
    let proxy_method_name = "proxy_".to_string() + method_name;
    let generated = quote! {
        ic_solidity_bindgen::contract_abi!(#abi_file_path);
        use #canister_name_ident::*;
        #relayer_source_ident
        #call_args_ident
        async fn sync() {
            let target_canister = candid::Principal::from_text(get_target_canister()).unwrap();
            let call_result = CallProvider::new()
                .call(Message::new::<CallCanisterArgs>(call_args(), _get_target_proxy(target_canister.clone()).await, #proxy_method_name).unwrap())
                .await;
            if let Err(err) = call_result {
                ic_cdk::println!("error: {:?}", err);
                return;
            }
            let val = call_result.unwrap().reply::<CallCanisterResponse>();
            if let Err(err) = val {
                ic_cdk::println!("error: {:?}", err);
                return;
            }
            let datum = val.unwrap();
            ic_cdk::println!("response from canister = {:?}", datum.clone());
            if !filter(&datum) {
                return;
            }
            let w3_ctx_param = get_web3_ctx_param();
            let call_option_builder = chainsight_cdk::web3::EVMTransactionOptionBuilder::new(
                w3_ctx_param.url,
                w3_ctx_param.chain_id,
                w3_ctx_param.env.ecdsa_key_name(),
            );
            use chainsight_cdk::web3::TransactionOptionBuilder;
            let call_option = call_option_builder.build().await;
            if call_option.is_err() {
                ic_cdk::println!("error: {:?}", call_option.err());
                return;
            }

            let result = #oracle_ident::new(
                Address::from_str(&get_target_addr()).unwrap(),
                &web3_ctx().unwrap()
            ).update_state(#sync_data_ident, call_option.unwrap()).await;
            if let Err(err) = result {
                ic_cdk::println!("error: {:?}", err);
                return;
            }
            ic_cdk::println!("value_to_sync={:?}", result.unwrap());
        }

    };
    generated
}

fn common_code(config: RelayerConfig) -> proc_macro2::TokenStream {
    let RelayerConfig {
        common,
        lens_targets,
        ..
    } = config;

    let canister_name = common.canister_name.clone();
    let lens_targets_quote = if lens_targets.is_some() {
        quote! { lens_targets: Vec<String> }
    } else {
        quote! {}
    };
    quote! {
        use std::str::FromStr;
        use chainsight_cdk_macros::{manage_single_state, setup_func, init_in, timer_task_func, define_web3_ctx, define_transform_for_web3, define_get_ethereum_address, chainsight_common, did_export,relayer_source};
        use ic_web3_rs::types::{Address, U256};
        use chainsight_cdk::rpc::{CallProvider, Caller, Message};
        use chainsight_cdk::web3::Encoder;
        did_export!(#canister_name);
        chainsight_common!(60);
        define_web3_ctx!();
        define_transform_for_web3!();
        manage_single_state!("target_addr", String, false);
        define_get_ethereum_address!();
        manage_single_state!("target_canister", String, false);
        timer_task_func!("set_task", "sync", true);
        init_in!();
        setup_func!({
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam,
            target_canister: String,
            #lens_targets_quote
        });
    }
}

fn generate_ident_sync_to_oracle(canister_response_type: &Type) -> proc_macro2::TokenStream {
    if is_primitive(canister_response_type) {
        let arg_ident = format_ident!("datum");
        quote! {
            chainsight_cdk::web3::abi::EthAbiEncoder.encode(#arg_ident.clone())
        }
    } else {
        // TODO: consider byte conversion for serialization
        quote! { format!("{:?}", &datum) }
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = RelayerConfig {
            common: CommonConfig {
                monitor_duration: 60,
                canister_name: "relayer".to_string(),
            },
            destination: "0539a0EF8e5E60891fFf0958A059E049e43020d9".to_string(),
            oracle_type: "".to_string(), // NOTE: unused
            method_identifier: "get_last_snapshot_value : () -> (text)".to_string(),
            abi_file_path: "__interfaces/Uint256Oracle.json".to_string(),
            lens_targets: None,
        };
        let generated = relayer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__relayer", formatted);
    }
}
