use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Type,
};

pub struct IndexerInput {
    in_type: r#syn::Type,
    out_type: syn::Type,
    indexer_impl: syn::Type,
}
impl Parse for IndexerInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let in_ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let out_type: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let indexer_impl: Type = input.parse()?;
        Ok(IndexerInput {
            in_type: in_ty,
            out_type,
            indexer_impl,
        })
    }
}

pub fn candid_exports(input: TokenStream) -> TokenStream {
    let IndexerInput {
        in_type: _type,
        out_type,
        indexer_impl,
    } = parse_macro_input!(input as IndexerInput);
    let output = quote! {
        manage_single_state!("proxy_canister", String, false);
        manage_single_state!("config", IndexingConfig, false);
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn events_from_to(from:u64, to: u64) -> HashMap<u64, Vec<#out_type>> {
            _events_from_to((from,to))
        }
        fn _events_from_to(input: (u64,  u64))-> HashMap<u64, Vec<#out_type>> {
            indexer().between::<#out_type>(input.0,input.1).unwrap()
        }
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn get_last_indexed() -> u64 {
            indexer().get_last_indexed().unwrap()
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            indexer().index::<#out_type>(get_config()).await.unwrap();
        }
        fn indexer() -> #indexer_impl {
            #indexer_impl::new(get_logs, None)
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn proxy_call(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            let proxy_canister = candid::Principal::from_text(get_proxy_canister()).unwrap();
            chainsight_cdk::rpc::ReceiverProvider::<(u64, u64), HashMap<u64, Vec<#out_type>>>::new(
                proxy_canister,
                _events_from_to.clone(),
            )
            .reply(input)
        }
    };
    output.into()
}
