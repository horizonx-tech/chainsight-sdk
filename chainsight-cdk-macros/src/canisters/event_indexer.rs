use std::{fs::File, path::Path};

use chainsight_cdk::{
    config::components::{CommonConfig, EventIndexerConfig, EventIndexerEventDefinition},
    convert::evm::{convert_type_from_ethabi_param_type, ADDRESS_TYPE, U256_TYPE},
};
use ethabi;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::canisters::utils::camel_to_snake;

use super::utils::extract_contract_name_from_path;

pub fn def_event_indexer_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: EventIndexerConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    event_indexer_canister(config).into()
}

fn event_indexer_canister(config: EventIndexerConfig) -> proc_macro2::TokenStream {
    let common = common_code(&config.common);
    let custom = custom_code(config);

    quote! {
        #common
        #custom
    }
}

fn common_code(common: &CommonConfig) -> proc_macro2::TokenStream {
    let CommonConfig { canister_name } = common;

    quote! {
        use candid::{CandidType, Decode, Encode};
        use chainsight_cdk::{
            core::{U256},
            indexer::{Event, Indexer, IndexingConfig},
            storage::Data,
            web3::Web3CtxParam,
        };
        use chainsight_cdk_macros::{
            define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export, init_in,
            manage_single_state, chainsight_common, setup_func, web3_event_indexer, timer_task_func, prepare_stable_structure, stable_memory_for_scalar, web3_event_indexer_source,
            ContractEvent, Persist, StableMemoryStorable, CborSerde
        };
        use ic_solidity_bindgen::{types::EventLog};
        use ic_stable_structures::writer::Writer;
        use ic_web3_rs::{
            ethabi::Address,
            futures::{future::BoxFuture, FutureExt},
            transports::ic_http_client::CallOptions,
        };
        use serde::Serialize;
        use std::{collections::HashMap, str::FromStr};
        did_export!(#canister_name);

        // NOTE: The memory id in canister is used from a number that does not duplicate the memory id declared in the storage module of the cdk.
        // https://github.com/horizonx-tech/chainsight-sdk/blob/8aa1d1dd1cb8e3d0adde2fa9d27f374d430f663a/chainsight-cdk/src/storage/storage.rs#L97
        init_in!(11);
        chainsight_common!();
        define_web3_ctx!(12);
        define_transform_for_web3!();
        define_get_ethereum_address!();
        prepare_stable_structure!();
        stable_memory_for_scalar!("target_addr", String, 13, false);
        setup_func!({
            target_addr: String,
            web3_ctx_param: Web3CtxParam,
            config: IndexingConfig,
        }, 14);
        timer_task_func!("set_task", "index", 15);
    }
}

fn custom_code(config: EventIndexerConfig) -> proc_macro2::TokenStream {
    let EventIndexerConfig {
        common: _,
        def:
            EventIndexerEventDefinition {
                identifier,
                abi_file_path,
            },
    } = config;

    let contract_struct_name_ident =
        format_ident!("{}", extract_contract_name_from_path(&abi_file_path));

    let event = {
        let reader = File::open(Path::new(&abi_file_path))
            .unwrap_or_else(|_| panic!("Couldn't open {}", &abi_file_path));
        let binding = ethabi::Contract::load(reader)
            .unwrap_or_else(|e| panic!("Failed to load contract from abi file: {}", e));
        // NOTE: Can I use .event? https://docs.rs/ethabi/latest/ethabi/struct.Contract.html#method.event
        binding
            .events_by_name(&identifier)
            .unwrap_or_else(|_| panic!("Failed to find event by name {}", &identifier))
            .first()
            .unwrap_or_else(|| panic!("Failed to no events, name is {}", &identifier))
            .clone()
    };

    let call_func_ident = format_ident!("event_{}", camel_to_snake(&event.name));
    let (event_struct_ident, event_struct) = generate_event_struct(event);

    quote! {
        ic_solidity_bindgen::contract_abi!(#abi_file_path);
        web3_event_indexer_source!(#event_struct_ident);
        web3_event_indexer!(#event_struct_ident, 16);
        #event_struct

        fn get_logs(
            from: u64,
            to: u64,
            call_options: CallOptions,
        ) -> BoxFuture<'static, Result<HashMap<u64, Vec<EventLog>>, chainsight_cdk::indexer::Error>> {
            async move {
                let res = #contract_struct_name_ident::new(
                    Address::from_str(get_target_addr().as_str()).expect("Failed to parse target addr to Address"),
                    &web3_ctx().expect("Failed to get web3_ctx"),
                ).#call_func_ident(from, to, call_options).await;
                match res {
                    Ok(logs) => Ok(logs),
                    Err(e) => Err(chainsight_cdk::indexer::Error::OtherError(e.to_string())),
                }
            }.boxed()
        }
    }
}

fn generate_event_struct(event: ethabi::Event) -> (proc_macro2::Ident, proc_macro2::TokenStream) {
    let ethabi::Event { name, inputs, .. } = event;
    let struct_ident = format_ident!("{}", &name);
    let struct_field_tokens = inputs
        .clone()
        .into_iter()
        .map(|event_param| {
            let field_name_ident = format_ident!("{}", &event_param.name);
            let field_ty_ident = convert_event_param_type_to_field_ty_ident(&event_param.kind)
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to convert event's ParamType `{}` to field type ident",
                        &event_param.kind
                    )
                });
            quote! { pub #field_name_ident: #field_ty_ident }
        })
        .collect::<Vec<_>>();

    (
        struct_ident.clone(),
        quote! {
            #[derive(Clone, Debug,  Default, candid::CandidType, ContractEvent, Serialize, Persist)]
            pub struct #struct_ident {
                #(#struct_field_tokens),*
            }

            impl chainsight_cdk::indexer::Event<EventLog> for #struct_ident {
                fn tokenize(&self) -> chainsight_cdk::storage::Data {
                    self._tokenize()
                }

                fn untokenize(data: chainsight_cdk::storage::Data) -> Self {
                    #struct_ident::_untokenize(data)
                }
            }
        },
    )
}

fn convert_event_param_type_to_field_ty_ident(
    param_type: &ethabi::ParamType,
) -> anyhow::Result<proc_macro2::Ident> {
    let field_ty =
        convert_type_from_ethabi_param_type(param_type).map_err(|e| anyhow::anyhow!(e))?;
    let field_ty_ident = match field_ty.as_str() {
        ADDRESS_TYPE => format_ident!("String"),
        U256_TYPE => format_ident!("U256"),
        _ => format_ident!("{}", field_ty),
    };
    Ok(field_ty_ident)
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
