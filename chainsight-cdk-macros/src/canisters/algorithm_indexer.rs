use chainsight_cdk::config::components::{AlgorithmIndexerConfig, AlgorithmInputType};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::utils::update_funcs_to_upgrade;

pub fn def_algorithm_indexer_canister(input: TokenStream) -> TokenStream {
    let input_json_string = parse_macro_input!(input as syn::LitStr).value();
    let config: AlgorithmIndexerConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    algorithm_indexer_canister(config).into()
}

fn algorithm_indexer_canister(config: AlgorithmIndexerConfig) -> proc_macro2::TokenStream {
    let AlgorithmIndexerConfig {
        common,
        indexing: _,
        input,
    } = config;
    let canister_name = common.canister_name.clone();
    let canister_name_ident = format_ident!("{}", common.canister_name);
    let input_ty = input_type_ident(input.response_type, input.source_type);

    let quote_to_upgradable = {
        let state_struct = quote! {
            #[derive(Clone, Debug, PartialEq, candid::CandidType, serde::Serialize, serde::Deserialize, CborSerde)]
            pub struct UpgradeStableState {
                pub proxy: candid::Principal,
                pub initialized: bool,
                pub env: chainsight_cdk::core::Env,
                pub target_addr: String,
                pub config: IndexingConfig,
                pub indexing_interval: u32,
            }
        };

        let update_funcs_to_upgrade = update_funcs_to_upgrade(
            quote! {
                UpgradeStableState {
                    proxy: get_proxy(),
                    initialized: is_initialized(),
                    env: get_env(),
                    target_addr: get_target_addr(),
                    config: get_config(),
                    indexing_interval: get_indexing_interval(),
                }
            },
            quote! {
                set_initialized(state.initialized);
                set_proxy(state.proxy);
                set_env(state.env);
                setup(
                    state.target_addr,
                    state.config,
                ).expect("Failed to `setup` in post_upgrade");
                set_indexing_interval(state.indexing_interval);
            },
        );

        quote! {
            #state_struct
            #update_funcs_to_upgrade
        }
    };

    let method_name = input.method_name;
    quote! {
        use candid::CandidType;
        use chainsight_cdk::indexer::IndexingConfig;
        use chainsight_cdk_macros::{
            algorithm_indexer, chainsight_common, did_export, init_in, manage_single_state, setup_func,
            timer_task_func, prepare_stable_structure, algorithm_indexer_source, StableMemoryStorable, CborSerde
        };
        use ic_stable_structures::writer::Writer;
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        did_export!(#canister_name);  // NOTE: need to be declared before query, update
        chainsight_common!();
        init_in!();
        prepare_stable_structure!();
        manage_single_state!("target_addr", String, false);
        setup_func!({ target_addr: String, config: IndexingConfig });
        timer_task_func!("set_task", "index");
        use #canister_name_ident::*;

        algorithm_indexer_source!();
        algorithm_indexer!(#input_ty, #method_name);

        #quote_to_upgradable
    }
}

pub fn input_type_ident(
    struct_name: String,
    source_type: AlgorithmInputType,
) -> proc_macro2::TokenStream {
    let event_struct = format_ident!("{}", &struct_name);
    match source_type {
        AlgorithmInputType::EventIndexer => {
            // HashMap<u64, Vec<event_struct>>
            let source_ident = format_ident!("{}", &"HashMap".to_string());
            quote! {
                #source_ident<u64, Vec<#event_struct>>
            }
        }
        AlgorithmInputType::KeyValue => {
            // HashMap<String, event_struct>
            let source_ident = format_ident!("{}", &"HashMap".to_string());
            quote! {
                #source_ident<String, #event_struct>
            }
        }
        AlgorithmInputType::KeyValues => {
            // HashMap<String, Vec<event_struct>>
            let source_ident = format_ident!("{}", &"HashMap".to_string());
            quote! {
                #source_ident<String, Vec<#event_struct>>
            }
        }
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::{
        config::components::{AlgorithmIndexerInput, CommonConfig},
        indexer::IndexingConfig,
    };
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = AlgorithmIndexerConfig {
            common: CommonConfig {
                canister_name: "example_canister".to_string(),
            },
            indexing: IndexingConfig {
                start_from: 1222222,
                chunk_size: None,
            },
            input: AlgorithmIndexerInput {
                method_name: "get_list".to_string(),
                response_type: "String".to_string(),
                source_type: AlgorithmInputType::EventIndexer,
            },
        };
        let generated = algorithm_indexer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_indexer", formatted);
    }
}
