use chainsight_cdk::config::components::{CanisterMethodValueType, RelayerConfig};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;
pub fn def_relayer_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: RelayerConfig = serde_json::from_str(&input_json_string).unwrap();
    relayer_canister(config)
}

fn relayer_canister(config: RelayerConfig) -> TokenStream {
    let common = common_code(config.clone());
    let custom = custom_code(config);
    quote! {
        #common
        #custom
    }
    .into()
}

fn custom_code(config: RelayerConfig) -> proc_macro2::TokenStream {
    let canister_name_ident = format_ident!("{}", config.common.canister_name);
    let method_name = config.method_name;
    let sync_data_ident = generate_ident_sync_to_oracle(config.canister_method_value_type);

    let (args_type_ident, get_args_ident, relayer_source_ident) =
        match config.lens_targets.is_some() {
            true => (
                quote! {
                    type CallCanisterArgs = Vec<String>;

                },
                quote! {
                    pub fn call_args() -> Vec<String> {
                        get_lens_targets()
                    }
                },
                quote! {
                    relayer_source!(#method_name, true);
                    manage_single_state!("lens_targets", Vec<String>, false);
                },
            ),
            _ => (
                quote! {
                    type CallCanisterArgs = #canister_name_ident::CallCanisterArgs;

                },
                quote! {
                    pub fn call_args() -> CallCanisterArgs {
                        #canister_name_ident::call_args()
                    }
                },
                quote! {
                    relayer_source!(#method_name, false);
                },
            ),
        };
    let abi_path = config.abi_file_path;
    let oracle_name = abi_path
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .next()
        .unwrap();
    let oracle_ident = format_ident!("{}", oracle_name);
    let proxy_method_name = "proxy_".to_string() + &method_name;
    let generated = quote! {
        ic_solidity_bindgen::contract_abi!(#abi_path);
        use #canister_name_ident::*;
        #relayer_source_ident
        #args_type_ident
        #get_args_ident
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
            if !filter(&datum) {
                return;
            }

            #oracle_ident::new(
                Address::from_str(&get_target_addr()).unwrap(),
                &web3_ctx().unwrap()
            ).update_state(#sync_data_ident, None).await.unwrap();
            ic_cdk::println!("value_to_sync={:?}", datum);
        }

    };
    generated
}
fn common_code(config: RelayerConfig) -> proc_macro2::TokenStream {
    let canister_name = config.common.canister_name.clone();
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
            target_canister: String,
            target_addr: String,
            web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
        });
    }
}

fn generate_ident_sync_to_oracle(
    canister_response_type: CanisterMethodValueType,
) -> proc_macro2::TokenStream {
    match canister_response_type {
        CanisterMethodValueType::Scalar(_, _) => {
            let arg_ident = format_ident!("datum");
            quote! {
                chainsight_cdk::web3::abi::EthAbiEncoder.encode(#arg_ident.clone())
            }
        }
        CanisterMethodValueType::Tuple(_) => {
            quote! { format!("{:?}", &datum) }
        }
        CanisterMethodValueType::Struct(_) => {
            quote! { format!("{:?}", &datum) }
        }
        CanisterMethodValueType::Vector(_, _) => {
            quote! { format!("{:?}", &datum) }
        }
    }
}
