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
    out_type: syn::Type,
}

impl Parse for AlgorithmIndexerInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let in_ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let out_type: Type = input.parse()?;
        Ok(AlgorithmIndexerInput {
            in_type: in_ty,
            out_type,
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
    let common = indexer_common(out_type.clone());
    quote! {
        #common
        fn indexer() -> chainsight_cdk::web3::Web3Indexer {
            chainsight_cdk::web3::Web3Indexer::new(get_logs, None)
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            indexer().index::<#out_type>(get_config()).await.unwrap();
        }
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        async fn canister_type() -> String {
            "event_indexer".to_string()
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn event_source() -> String {
            get_target_addr()
        }
    }
    .into()
}

pub fn algorithm_indexer(input: TokenStream) -> TokenStream {
    let AlgorithmIndexerInput { in_type, out_type } =
        parse_macro_input!(input as AlgorithmIndexerInput);
    let common = indexer_common(out_type.clone());
    quote! {
        #common
        mod app;
        fn indexer() -> chainsight_cdk::algorithm::AlgorithmIndexer<#in_type> {
            chainsight_cdk::algorithm::AlgorithmIndexer::new(proxy(), get_target(), app::persist)
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            indexer().index(get_config()).await.unwrap();
        }
        fn get_target() -> candid::Principal {
            candid::Principal::from_text(get_target_addr()).unwrap()
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn event_source() -> candid::Principal {
            get_target()
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        async fn canister_type() -> String {
            "algorithm_indexer".to_string()
        }
    }
    .into()
}

pub fn indexer_common(out_type: syn::Type) -> TokenStream2 {
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
            indexer().between::<#out_type>(input.0,input.1).unwrap()
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
