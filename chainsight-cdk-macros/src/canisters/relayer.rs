use anyhow::bail;
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
    let common = common_code();
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
    let sync_data_ident =
        generate_ident_sync_to_oracle(config.canister_method_value_type, config.oracle_type);

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
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .next()
        .unwrap();
    let oracle_ident = format_ident!("{}", oracle_name);
    let canister_name = config.common.canister_name.clone();
    let proxy_method_name = "proxy_".to_string() + &method_name;
    let generated = quote! {
        #relayer_source_ident
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

        did_export!(#canister_name);


    };
    generated
}
fn common_code() -> proc_macro2::TokenStream {
    quote! {
        use std::str::FromStr;
        use chainsight_cdk_macros::{manage_single_state, setup_func, init_in, timer_task_func, define_web3_ctx, define_transform_for_web3, define_get_ethereum_address, chainsight_common, did_export,relayer_source};
        use ic_web3_rs::types::{Address, U256};
        use chainsight_cdk::rpc::{CallProvider, Caller, Message};

        chainsight_common!(60);
        define_web3_ctx!();
        define_transform_for_web3!();
        manage_single_state!("target_addr", String, false);
        define_get_ethereum_address!();
        manage_single_state!("target_canister", String, false);
        timer_task_func!("set_task", "sync", true);
        init_in!();
    }
}

fn generate_ident_sync_to_oracle(
    canister_response_type: CanisterMethodValueType,
    oracle_type: String,
) -> proc_macro2::TokenStream {
    let res = match canister_response_type {
        CanisterMethodValueType::Scalar(ty, _) => {
            let arg_ident = format_ident!("datum");
            match oracle_type.as_str() {
                "uint256" => generate_quote_to_convert_datum_to_u256(arg_ident, &ty).unwrap(),
                "uint128" => {
                    generate_quote_to_convert_datum_to_integer(arg_ident, &ty, "u128").unwrap()
                }
                "uint640" => {
                    generate_quote_to_convert_datum_to_integer(arg_ident, &ty, "u64").unwrap()
                }
                "string" => quote! { datum.clone().to_string() },
                _ => panic!("This type cannot be converted to {}", oracle_type),
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
    };
    res
}

fn generate_quote_to_convert_datum_to_u256(
    arg_ident: proc_macro2::Ident,
    datum_scalar_type: &str,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let res = match datum_scalar_type {
        "u8" | "u16" | "u32" | "u64" | "u128" | "U256" | "chainsight_cdk::core::U256" => {
            quote! { U256::from(#arg_ident) }
        }
        "i8" | "i16" | "i32" | "i64" | "i128" => quote! { U256::from(#arg_ident) }, // NOTE: a positive value check needs to be performed on the generated code
        "String" => quote! { U256::from_dec_str(&#arg_ident).unwrap() },
        _ => bail!("This type cannot be converted to U256"),
    };
    Ok(res)
}

fn generate_quote_to_convert_datum_to_integer(
    arg_ident: proc_macro2::Ident,
    datum_scalar_type: &str,
    converted_datum_type: &str,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let converted_datum_type_ident = format_ident!("{}", converted_datum_type);
    let res = match datum_scalar_type {
        "u8" | "u16" | "u32" | "u64" | "u128" => {
            quote! { #arg_ident as #converted_datum_type_ident }
        }
        "i8" | "i16" | "i32" | "i64" | "i128" => {
            quote! { #arg_ident as #converted_datum_type_ident }
        } // NOTE: a positive value check needs to be performed on the generated code
        "String" => quote! { #converted_datum_type_ident::from_str(&#arg_ident).unwrap() },
        _ => bail!(format!(
            "This type cannot be converted to {}",
            converted_datum_type
        )),
    };
    Ok(res)
}
