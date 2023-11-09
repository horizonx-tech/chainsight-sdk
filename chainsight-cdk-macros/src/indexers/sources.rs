use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input
};

pub fn generate_event_indexer_source(tt: syn::Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#tt).to_string();
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3EventIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3EventIndexerSourceAttrs>::new_event_indexer(
            get_target_addr(),
            get_indexing_interval(),
            chainsight_cdk::core::Web3EventIndexerSourceAttrs {
                chain_id: get_web3_ctx_param().chain_id,
                event_name: #type_str.to_string(),
            })
            ]
        }
    }
}
pub fn generate_algorithm_indexer_source() -> proc_macro2::TokenStream {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<std::collections::HashMap<String, String>>> {
            vec![
                chainsight_cdk::core::Sources::<std::collections::HashMap<String, String>>::new_algorithm_indexer(
                    get_target_addr(),
                    get_indexing_interval()
                )
            ]
        }
    }
}
pub fn snapshot_web3_source(input: TokenStream) -> TokenStream {
    let func_name: syn::LitStr = parse_macro_input!(input as syn::LitStr);
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3SnapshotIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3SnapshotIndexerSourceAttrs>::new_web3_snapshot_indexer(
                get_target_addr(),
                get_indexing_interval(),
                get_web3_ctx_param().chain_id,
                #func_name.to_string(),
            )]
        }
    }.into()
}
pub fn snapshot_https_source(_input: TokenStream) -> TokenStream {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::HttpsSnapshotIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::HttpsSnapshotIndexerSourceAttrs>::new_https_snapshot_indexer(
                URL.to_string(),
                get_indexing_interval(),
                get_attrs(),
            )]
        }
    }.into()
}
pub fn snapshot_icp_source(input: TokenStream) -> TokenStream {
    let func_name: syn::LitStr = parse_macro_input!(input as syn::LitStr);
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::ICSnapshotIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::ICSnapshotIndexerSourceAttrs>::new_snapshot_indexer(
                get_target_canister(),
                get_indexing_interval(),
                #func_name.to_string(),
            )]
        }
    }.into()
}

pub struct RelayerSourceInput {
    method_identifier: syn::LitStr,
    from_lens: syn::LitBool,
}
impl Parse for RelayerSourceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method_identifier: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let from_lens: syn::LitBool = input.parse()?;
        Ok(RelayerSourceInput {
            method_identifier,
            from_lens,
        })
    }
}
pub fn relayer_source(input: TokenStream) -> TokenStream {
    let RelayerSourceInput {
        from_lens,
        method_identifier,
    } = parse_macro_input!(input as RelayerSourceInput);
    if from_lens.value {
        return quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
            fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::RelayerWithLensSourceAttrs>> {
                vec![
                    chainsight_cdk::core::Sources::<chainsight_cdk::core::RelayerWithLensSourceAttrs>::new_relayer(
                        get_target_canister(),
                        get_indexing_interval(),
                        #method_identifier,
                        call_args()
                    ),
                ]
            }
        }.into();
    }

    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::RelayerWithLensSourceAttrs>> {
            vec![
                chainsight_cdk::core::Sources::<chainsight_cdk::core::RelayerWithLensSourceAttrs>::new_relayer(
                    get_target_canister(),
                    get_indexing_interval(),
                    #method_identifier,
                    vec![]
                )
            ]
        }
    }.into()
}
