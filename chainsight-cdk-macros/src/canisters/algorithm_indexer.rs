use chainsight_cdk::config::components::{AlgorithmIndexerConfig, AlgorithmInputType};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

pub fn def_algorithm_indexer_canister(input: TokenStream) -> TokenStream {
    let input_json_string = parse_macro_input!(input as syn::LitStr).value();
    let config: AlgorithmIndexerConfig = serde_json::from_str(&input_json_string).unwrap();
    algorithm_indexer_canister(config)
}

fn algorithm_indexer_canister(config: AlgorithmIndexerConfig) -> TokenStream {
    let monitor_duration = config.common.monitor_duration;
    let canister_name = config.common.canister_name.clone();
    let canister_name_ident = format_ident!("{}", config.common.canister_name);
    let input_ty = input_type_ident(config.input.response_type, config.input.source_type);

    let method_name = config.input.method_name;
    quote! {
        use candid::CandidType;
        use chainsight_cdk::core::*;
        use chainsight_cdk::{indexer::IndexingConfig, storage::Data};
        use chainsight_cdk_macros::*;
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        chainsight_common!(#monitor_duration);
        init_in!();
        manage_single_state!("target_addr", String, false);
        setup_func ! ({ target_addr : String , config : IndexingConfig });
        timer_task_func!("set_task", "index", true);
        use #canister_name_ident::*;

        algorithm_indexer!(#input_ty, #method_name);
        did_export!(#canister_name);
    }
    .into()
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