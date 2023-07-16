use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Type,
};
pub struct Web3EventIndexerInput {
    out_type: syn::Type,
}

pub struct AlgorithmIndexerInput {
    in_type: syn::Type,
    call_method: syn::LitStr,
}

impl Parse for AlgorithmIndexerInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let in_ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let call_method_str: syn::LitStr = input.parse()?;
        Ok(AlgorithmIndexerInput {
            in_type: in_ty,
            call_method: call_method_str,
        })
    }
}

impl Parse for Web3EventIndexerInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let out_type: Type = input.parse()?;
        Ok(Web3EventIndexerInput { out_type })
    }
}

pub fn web3_event_indexer(input: TokenStream) -> TokenStream {
    let Web3EventIndexerInput { out_type } = parse_macro_input!(input as Web3EventIndexerInput);
    let common = event_indexer_common(out_type.clone());
    let source = generate_event_indexer_source(out_type.clone());
    quote! {
        #source
        #common
        fn indexer() -> chainsight_cdk::web3::Web3Indexer<#out_type> {
            chainsight_cdk::web3::Web3Indexer::new(get_logs, None)
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            indexer().index(get_config()).await.unwrap();
        }

    }
    .into()
}

fn generate_event_indexer_source(tt: syn::Type) -> TokenStream2 {
    let type_str = stringify!(tt).to_string();
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3EventIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3EventIndexerSourceAttrs>::new_event_indexer(
            get_target_addr(),
            get_timer_duration(),
            chainsight_cdk::core::Web3EventIndexerSourceAttrs {
                chain_id: get_web3_ctx_param().chain_id,
                event_name: #type_str.to_string(),
            })
            ]
        }
    }.into()
}

fn generate_algorithm_indexer_source() -> TokenStream2 {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<std::collections::HashMap<String, String>>> {
            vec![chainsight_cdk::core::Sources::<std::collections::HashMap<String, String>>::new_algorithm_indexer(
            get_target_addr(),
            get_timer_duration())
            ]
        }
    }.into()
}

pub fn relayer_source() -> TokenStream {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<std::collections::HashMap<String, String>>> {
            vec![chainsight_cdk::core::Sources::<std::collections::HashMap<String, String>>::new_relayer(
            get_target_addr(),
            get_timer_duration())
            ]
        }
    }.into()
}
pub fn snapshot_icp_source(input: TokenStream) -> TokenStream {
    let func_name: syn::LitStr = parse_macro_input!(input as syn::LitStr);
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<std::collections::HashMap<String, String>>> {
            vec![chainsight_cdk::core::Sources::<std::collections::HashMap<String, String>>::new_snapshot_indexer(
            get_target_addr(),
            get_timer_duration(),
            #func_name)
            ]
        }
    }.into()
}

pub fn snapshot_web3_source(input: TokenStream) -> TokenStream {
    let func_name: syn::LitStr = parse_macro_input!(input as syn::LitStr);
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3AlgorithmIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3AlgorithmIndexerSourceAttrs>::new_web3_snapshot_indexer(
                get_target_addr(),
                get_timer_duration(),
                get_web3_ctx_param().chain_id,
                #func_name.to_string(),
            )]
        }
    }.into()
}

pub fn algorithm_indexer(input: TokenStream) -> TokenStream {
    let AlgorithmIndexerInput {
        in_type,
        call_method,
    } = parse_macro_input!(input as AlgorithmIndexerInput);
    let source = generate_algorithm_indexer_source();
    quote! {
        mod app;
        manage_single_state!("config", IndexingConfig, false);
        use chainsight_cdk::indexer::Indexer;
        fn indexer() -> chainsight_cdk::algorithm::AlgorithmIndexer<#in_type> {
            chainsight_cdk::algorithm::AlgorithmIndexer::new_with_method(proxy(), get_target(),#call_method, app::persist)
        }
        #source
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            let mut config = get_config();
            let stored = chainsight_cdk::storage::get_last_key();
            ic_cdk::println!("stored: {:?}", stored);
            let stored_u64 = stored.parse::<u64>().unwrap_or(0);
            ic_cdk::println!("stored_u64: {:?}", stored_u64);
            if stored_u64 > config.start_from {
                config.start_from = stored_u64;
            }
            indexer().index(config).await.unwrap();
        }
        fn get_target() -> candid::Principal {
            candid::Principal::from_text(get_target_addr()).unwrap()
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn event_source() -> candid::Principal {
            get_target()
        }

    }
    .into()
}

pub fn event_indexer_common(out_type: syn::Type) -> TokenStream2 {
    let output = quote! {
        manage_single_state!("config", IndexingConfig, false);
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn events_from_to(from:u64, to: u64) -> HashMap<u64, Vec<#out_type>> {
            _events_from_to((from,to))
        }
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn events_latest_n(n: u64) -> HashMap<u64, Vec<#out_type>> {
            let last_indexed = indexer().get_last_indexed().unwrap();
            _events_from_to((last_indexed - n, last_indexed))
        }
        fn _events_from_to(input: (u64,  u64)) -> HashMap<u64, Vec<#out_type>> {
            indexer().between(input.0,input.1).unwrap()
        }
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn get_last_indexed() -> u64 {
            indexer().get_last_indexed().unwrap()
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn proxy_call(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            chainsight_cdk::rpc::ReceiverProvider::<(u64, u64), HashMap<u64, Vec<#out_type>>>::new(
                proxy(),
                _events_from_to.clone(),
            )
            .reply(input)
        }
    };

    output
}
