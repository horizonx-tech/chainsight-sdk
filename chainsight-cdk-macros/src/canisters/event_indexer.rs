use std::{fs::File, path::Path};

use chainsight_cdk::{
    config::components::EventIndexerConfig,
    convert::evm::{convert_type_from_ethabi_param_type, ADDRESS_TYPE, U256_TYPE},
};
use ethabi;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::utils::extract_contract_name_from_path;

pub fn def_event_indexer_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: EventIndexerConfig = serde_json::from_str(&input_json_string).unwrap();
    event_indexer_canister(config).into()
}

fn event_indexer_canister(config: EventIndexerConfig) -> proc_macro2::TokenStream {
    let monitor_duration = config.common.monitor_duration;
    let canister_name = config.common.canister_name.clone();
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
        did_export!(#canister_name);
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
}

fn custom_code(config: EventIndexerConfig) -> proc_macro2::TokenStream {
    let EventIndexerConfig { common: _, def } = config;
    let abi_file_path = def.abi_file_path;
    let contract_struct_name_ident =
        format_ident!("{}", extract_contract_name_from_path(&abi_file_path));
    let binding = ethabi::Contract::load(File::open(Path::new(&abi_file_path)).unwrap()).unwrap();
    let event = binding
        .events_by_name(def.identifier.as_str())
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
    }
}

/// Convert camelCase String to snake_case
pub fn camel_to_snake(val: &str) -> String {
    // NOTE: use Inflator in ic-solidity-bindgen
    // https://github.com/horizonx-tech/ic-solidity-bindgen/blob/0972bede5957927bcb8f675decd93878b849dc76/ic-solidity-bindgen-macros/src/abi_gen.rs#L192
    inflector::cases::snakecase::to_snake_case(val)
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::{CommonConfig, EventIndexerEventDefinition};
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = EventIndexerConfig {
            common: CommonConfig {
                monitor_duration: 1000,
                canister_name: "app".to_string(),
            },
            def: EventIndexerEventDefinition {
                identifier: "Transfer".to_string(),
                abi_file_path: "examples/minimum_indexers/src/event_indexer/abi/ERC20.json"
                    .to_string(),
            },
        };
        let generated = event_indexer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__event_indexer", formatted);
    }
}
