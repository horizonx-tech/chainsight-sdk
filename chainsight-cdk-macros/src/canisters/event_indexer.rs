use std::{fs::File, path::Path};

use chainsight_cdk::config::components::EventIndexerConfig;
use ethabi::{self, ParamType};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;
pub fn def_event_indexer_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: EventIndexerConfig = serde_json::from_str(&input_json_string).unwrap();
    event_indexer_canister(config)
}

fn event_indexer_canister(config: EventIndexerConfig) -> TokenStream {
    let monitor_duration = config.common.monitor_duration;
    let common = quote! {
        use candid::CandidType;
        use chainsight_cdk::{
            core::{U256},
            indexer::{Event, Indexer, IndexingConfig},
            storage::Data,
            web3::Web3CtxParam,
        };
        use chainsight_cdk_macros::{
            define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export, init_in,
            manage_single_state, chainsight_common, setup_func, web3_event_indexer,timer_task_func,
            ContractEvent, Persist,
        };
        use ic_solidity_bindgen::{types::EventLog};
        use ic_web3_rs::{
            ethabi::Address,
            futures::{future::BoxFuture, FutureExt},
            transports::ic_http_client::CallOptions,
        };
        use serde::Serialize;
        use std::{collections::HashMap, str::FromStr};
        chainsight_common!(#monitor_duration);
        define_web3_ctx!();
        define_transform_for_web3!();
        define_get_ethereum_address!();
        timer_task_func!("set_task", "index", true);
        manage_single_state!("target_addr", String, false);
        setup_func!({
            target_addr: String,
            web3_ctx_param: Web3CtxParam,
            config: IndexingConfig,
        });
        init_in!();
    };
    let custom = custom_code(config);
    quote! {
        #common
        #custom
    }
    .into()
}

fn custom_code(config: EventIndexerConfig) -> proc_macro2::TokenStream {
    let canister_name = config.common.canister_name.clone();
    let abi_file_path = config.def.abi_file_path;
    let contract_struct_name = abi_file_path
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .next()
        .unwrap();
    let contract_struct_name_ident = format_ident!("{}", contract_struct_name);
    let binding = ethabi::Contract::load(File::open(Path::new(&abi_file_path)).unwrap()).unwrap();
    let event = binding
        .events_by_name(config.def.identifier.as_str())
        .unwrap()
        .first()
        .unwrap();
    let event_struct_ident = format_ident!("{}", event.name);
    let event_struct_field_tokens = event
        .inputs
        .clone()
        .into_iter()
        .map(|event| {
            let field_name_ident = format_ident!("{}", event.name);
            let field_ty = convert_type_from_ethabi_param_type(event.kind).unwrap();
            let field_ty_ident = if field_ty == ADDRESS_TYPE {
                format_ident!("String")
            } else if field_ty == U256_TYPE {
                format_ident!("U256")
            } else {
                format_ident!("{}", field_ty)
            }; // todo: refactor
            quote! { pub #field_name_ident: #field_ty_ident }
        })
        .collect::<Vec<_>>();
    let event_struct = quote! {

        #[derive(Clone, Debug,  Default, candid::CandidType, ContractEvent, Serialize, Persist)]
        pub struct #event_struct_ident {
            #(#event_struct_field_tokens),*
        }

        impl chainsight_cdk::indexer::Event<EventLog> for #event_struct_ident {
            fn tokenize(&self) -> chainsight_cdk::storage::Data {
                self._tokenize()
            }

            fn untokenize(data: chainsight_cdk::storage::Data) -> Self {
                #event_struct_ident::_untokenize(data)
            }
        }
    };
    let call_func_ident = format_ident!("event_{}", camel_to_snake(&event.name));
    quote! {
        ic_solidity_bindgen::contract_abi!(#abi_file_path);
        web3_event_indexer!(#event_struct_ident);
        #event_struct

        fn get_logs(
            from: u64,
            to: u64,
            call_options: CallOptions,
        ) -> BoxFuture<'static, Result<HashMap<u64, Vec<EventLog>>, chainsight_cdk::indexer::Error>> {
            async move {
                let res = #contract_struct_name_ident::new(
                    Address::from_str(get_target_addr().as_str()).unwrap(),
                    &web3_ctx().unwrap()
                ).#call_func_ident(from, to, call_options).await;
                match res {
                    Ok(logs) => Ok(logs),
                    Err(e) => Err(chainsight_cdk::indexer::Error::OtherError(e.to_string())),
                }
            }.boxed()
        }
        did_export!(#canister_name);
    }
}

/// To handle 256bits Unsigned Integer type in ic_web3_rs
pub const U256_TYPE: &str = "ic_web3_rs::types::U256";
/// To handle Address type in ic_web3_rs
pub const ADDRESS_TYPE: &str = "ic_web3_rs::types::Address";

pub fn convert_type_from_ethabi_param_type(param: ethabi::ParamType) -> Result<String, String> {
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
/// Convert camelCase String to snake_case
pub fn camel_to_snake(val: &str) -> String {
    // NOTE: use Inflator in ic-solidity-bindgen
    // https://github.com/horizonx-tech/ic-solidity-bindgen/blob/0972bede5957927bcb8f675decd93878b849dc76/ic-solidity-bindgen-macros/src/abi_gen.rs#L192
    inflector::cases::snakecase::to_snake_case(val)
}