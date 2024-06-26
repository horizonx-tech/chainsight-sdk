use chainsight_cdk::config::components::{
    AlgorithmIndexerConfig, AlgorithmInputType, AlgorithmOutputType,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

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
        output,
    } = config;
    let canister_name = common.canister_name.clone();
    let canister_name_ident = format_ident!("{}", common.canister_name);
    let input_ty = input_type_ident(input.response_type, input.source_type);
    let outputs_quote = output
        .types
        .iter()
        .map(|val| {
            let name = format_ident!("{}", val.name);
            match val.type_ {
                AlgorithmOutputType::KeyValue => {
                    quote! { generate_queries_for_key_value_store_struct!(#name) }
                }
                AlgorithmOutputType::KeyValues => {
                    quote! { generate_queries_for_key_values_store_struct!(#name) }
                }
            }
        })
        .collect::<Vec<_>>();

    let method_name = input.method_name;
    quote! {
        use candid::{CandidType, Decode, Encode};
        use chainsight_cdk::indexer::IndexingConfig;
        use chainsight_cdk_macros::{
            algorithm_indexer, chainsight_common, did_export, init_in, manage_single_state, setup_func,
            generate_queries_for_key_value_store_struct, generate_queries_for_key_values_store_struct,
            timer_task_func, prepare_stable_structure, stable_memory_for_scalar, algorithm_indexer_source, StableMemoryStorable, CborSerde
        };
        use ic_stable_structures::writer::Writer;
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        did_export!(#canister_name);  // NOTE: need to be declared before query, update
        chainsight_common!();

        // NOTE: The memory id in canister is used from a number that does not duplicate the memory id declared in the storage module of the cdk.
        // https://github.com/horizonx-tech/chainsight-sdk/blob/8aa1d1dd1cb8e3d0adde2fa9d27f374d430f663a/chainsight-cdk/src/storage/storage.rs#L97
        init_in!(11);
        prepare_stable_structure!();
        stable_memory_for_scalar!("target_addr", String, 12, false);
        setup_func!({ target_addr: String, config: IndexingConfig }, 13);
        timer_task_func!("set_task", "index", 14);
        use #canister_name_ident::*;

        algorithm_indexer_source!();
        algorithm_indexer!(#input_ty, #method_name, 15);

        #(#outputs_quote;)*
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
                #source_ident<u64, #event_struct>
            }
        }
        AlgorithmInputType::KeyValues => {
            // HashMap<String, Vec<event_struct>>
            let source_ident = format_ident!("{}", &"HashMap".to_string());
            quote! {
                #source_ident<u64, Vec<#event_struct>>
            }
        }
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::{
        config::components::{
            AlgorithmIndexerInput, AlgorithmIndexerOutput, AlgorithmIndexerOutputIdentifier,
            CommonConfig,
        },
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
            output: AlgorithmIndexerOutput {
                types: vec![
                    AlgorithmIndexerOutputIdentifier {
                        name: "OutputType1".to_string(),
                        type_: AlgorithmOutputType::KeyValue,
                    },
                    AlgorithmIndexerOutputIdentifier {
                        name: "OutputType2".to_string(),
                        type_: AlgorithmOutputType::KeyValues,
                    },
                ],
            },
        };
        let generated = algorithm_indexer_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_indexer", formatted);
    }
}
