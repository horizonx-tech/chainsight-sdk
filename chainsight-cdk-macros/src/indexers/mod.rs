use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, LitInt, Type,
};

use crate::internal::{attrs_query_func, attrs_update_func, gen_func_quote_to_call_proxy};

pub mod sources;

pub struct Web3EventIndexerInput {
    out_type: syn::Type,
    stable_memory_id: Option<LitInt>,
}
impl Parse for Web3EventIndexerInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let out_type: Type = input.parse()?;
        let stable_memory_id = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let parsed: LitInt = input.parse()?;
            Some(parsed)
        } else {
            None
        };
        Ok(Web3EventIndexerInput {
            out_type,
            stable_memory_id,
        })
    }
}
pub fn web3_event_indexer(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Web3EventIndexerInput);
    web3_event_indexer_internal(args).into()
}
fn web3_event_indexer_internal(args: Web3EventIndexerInput) -> proc_macro2::TokenStream {
    let Web3EventIndexerInput {
        out_type,
        stable_memory_id,
    } = args;
    let common = event_indexer_common(out_type.clone(), stable_memory_id);

    quote! {
        #common
        fn indexer() -> chainsight_cdk::web3::Web3Indexer<#out_type> {
            chainsight_cdk::web3::Web3Indexer::new(get_logs, None)
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }
            indexer().index(get_config()).await.unwrap();
        }

    }
}
fn event_indexer_common(
    out_type: syn::Type,
    stable_memory_id: Option<LitInt>,
) -> proc_macro2::TokenStream {
    let storage_quote = if let Some(memory_id) = stable_memory_id {
        quote! {
            stable_memory_for_scalar!("config", IndexingConfig, #memory_id, false);
        }
    } else {
        quote! {
            manage_single_state!("config", IndexingConfig, false);
        }
    };

    let attrs_query = attrs_query_func();
    let attrs_update = attrs_update_func();

    let _proxy_events_from_to_quote = gen_func_quote_to_call_proxy(
        "_proxy_events_from_to",
        parse_quote! { HashMap<u64, Vec<#out_type>> },
        Some(parse_quote! { (u64, u64) }),
        "_events_from_to",
    );
    let proxy_events_latest_n_quote = gen_func_quote_to_call_proxy(
        "proxy_events_latest_n",
        parse_quote! { HashMap<u64, Vec<#out_type>> },
        Some(parse_quote! { u64 }),
        "_events_latest_n",
    );
    let proxy_get_last_indexed_quote = gen_func_quote_to_call_proxy(
        "proxy_get_last_indexed",
        parse_quote! { u64 },
        None,
        "_get_last_indexed",
    );

    let output = quote! {
        #storage_quote

        #attrs_query
        pub fn events_from_to(from:u64, to: u64) -> HashMap<u64, Vec<#out_type>> {
            _events_from_to((from,to))
        }
        #attrs_update
        pub async fn proxy_events_from_to(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            _proxy_events_from_to(input).await
        }

        #_proxy_events_from_to_quote

        fn _events_from_to(input: (u64,  u64)) -> HashMap<u64, Vec<#out_type>> {
            indexer().between(input.0,input.1).unwrap()
        }

        #attrs_query
        pub fn events_latest_n(n: u64) -> HashMap<u64, Vec<#out_type>> {
            _events_latest_n(n)
        }

        #attrs_update
        pub #proxy_events_latest_n_quote

        fn _events_latest_n(n: u64) -> HashMap<u64, Vec<#out_type>> {
            let last_indexed = indexer().get_last_indexed().unwrap();
            _events_from_to((last_indexed - n, last_indexed + 1)) // note: +1 to include the last indexed
        }


        #attrs_query
        pub fn get_last_indexed() -> u64 {
            _get_last_indexed()
        }

        #attrs_update
        pub #proxy_get_last_indexed_quote

        fn _get_last_indexed() -> u64 {
            indexer().get_last_indexed().unwrap()
        }

        #attrs_update
        async fn proxy_call(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            _proxy_events_from_to(input).await
        }
    };

    output
}

pub struct AlgorithmIndexerInput {
    in_type: syn::Type,
    call_method: syn::LitStr,
    stable_memory_id: Option<LitInt>,
}
impl Parse for AlgorithmIndexerInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let in_ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let call_method_str: syn::LitStr = input.parse()?;
        let stable_memory_id = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let parsed: LitInt = input.parse()?;
            Some(parsed)
        } else {
            None
        };
        Ok(AlgorithmIndexerInput {
            in_type: in_ty,
            call_method: call_method_str,
            stable_memory_id,
        })
    }
}
pub fn algorithm_indexer(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as AlgorithmIndexerInput);
    algorithm_indexer_internal(args).into()
}
fn algorithm_indexer_internal(args: AlgorithmIndexerInput) -> proc_macro2::TokenStream {
    let AlgorithmIndexerInput {
        in_type,
        call_method,
        stable_memory_id,
    } = args;

    let storage_quote = if let Some(memory_id) = stable_memory_id {
        quote! {
            stable_memory_for_scalar!("config", IndexingConfig, #memory_id, false);
        }
    } else {
        quote! {
            manage_single_state!("config", IndexingConfig, false);
        }
    };

    quote! {
        #storage_quote

        use chainsight_cdk::indexer::Indexer;
        async fn indexer() -> chainsight_cdk::algorithm::AlgorithmIndexer<#in_type> {
            chainsight_cdk::algorithm::AlgorithmIndexer::new_with_method(_get_target_proxy(get_target()).await, #call_method, persist)
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }
            let mut config = get_config();
            let stored = chainsight_cdk::storage::get_last_key();
            ic_cdk::println!("stored: {:?}", stored);
            let stored_u64 = stored.parse::<u64>().unwrap_or(0);
            ic_cdk::println!("stored_u64: {:?}", stored_u64);
            if stored_u64 > config.start_from {
                config.start_from = stored_u64;
            }
            indexer().await.index(config).await.unwrap();
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
}

pub struct AlgorithmIndexerWithArgsInput {
    in_type: syn::Type,
    args: syn::Type,
    call_method: syn::LitStr,
}
impl Parse for AlgorithmIndexerWithArgsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let in_ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let args_ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let call_method_str: syn::LitStr = input.parse()?;
        Ok(AlgorithmIndexerWithArgsInput {
            in_type: in_ty,
            args: args_ty,
            call_method: call_method_str,
        })
    }
}
pub fn algorithm_indexer_with_args(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as AlgorithmIndexerWithArgsInput);
    algorithm_indexer_with_args_internal(args).into()
}
fn algorithm_indexer_with_args_internal(
    args: AlgorithmIndexerWithArgsInput,
) -> proc_macro2::TokenStream {
    let AlgorithmIndexerWithArgsInput {
        in_type,
        args,
        call_method,
    } = args;

    quote! {
        manage_single_state!("config", IndexingConfig, false);
        thread_local!{
            static ARGS: std::cell::RefCell<Option<#args>> = std::cell::RefCell::new(None);
        }
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn set_args(args: #args) {
            ARGS.with(|cell| {
                *cell.borrow_mut() = Some(args);
            });
        }
        fn get_args() -> #args {
            ARGS.with(|cell| {
                cell.borrow().clone().unwrap()
            })
        }

        use chainsight_cdk::indexer::Indexer;
        async fn indexer() -> chainsight_cdk::algorithm::AlgorithmIndexerWithArgs<#in_type, #args> {
            chainsight_cdk::algorithm::AlgorithmIndexerWithArgs::new_with_method(_get_target_proxy(get_target()).await, #call_method, persist, get_args())
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }
            indexer().await.index(chainsight_cdk::indexer::IndexingConfig::default()).await.unwrap()
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
}

pub struct AlgorithmLensFinderInput {
    id: syn::LitStr,
    call_method: syn::LitStr,
    return_ty: syn::Type,
    args_ty: Option<syn::Type>,
}
impl Parse for AlgorithmLensFinderInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id_str: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let call_method_str: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let return_ty: Type = input.parse()?;
        let args_ty = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(AlgorithmLensFinderInput {
            id: id_str,
            call_method: call_method_str,
            return_ty,
            args_ty,
        })
    }
}
pub fn algorithm_lens_finder(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as AlgorithmLensFinderInput);
    algorithm_lens_finder_internal(args).into()
}
fn algorithm_lens_finder_internal(args: AlgorithmLensFinderInput) -> proc_macro2::TokenStream {
    let AlgorithmLensFinderInput {
        id,
        call_method,
        return_ty,
        args_ty,
    } = args;

    let finder_method_name = format_ident!("finder_{}", id.value().to_lowercase());
    let get_method_name = format_ident!("get_{}", id.value().to_lowercase());
    let get_unwrap_method_name = format_ident!("get_{}_unwrap", id.value().to_lowercase());

    match args_ty {
        Some(args_ty) => {
            quote! {
                pub async fn #get_method_name(target_principal: String, args: #args_ty) -> std::result::Result<#return_ty, String> {
                    let method = #finder_method_name(target_principal.clone());
                    method.await.find(args).await.map_err(|e| format!("{:?}", e))
                }

                pub async fn #get_unwrap_method_name(target_principal: String, args: #args_ty) -> #return_ty {
                    #finder_method_name(target_principal.clone()).await.find(args).await.unwrap()
                }


                async fn #finder_method_name(target_principal: String) -> chainsight_cdk::lens::AlgorithmLensFinder<#return_ty> {
                    use chainsight_cdk::lens::LensFinder;

                    let recipient = candid::Principal::from_text(target_principal).unwrap();
                    chainsight_cdk::lens::AlgorithmLensFinder::new(
                        chainsight_cdk::lens::LensTarget::<#return_ty>::new(
                            _get_target_proxy(recipient).await,
                            #call_method,
                        )
                    )
                }
            }
        }
        None => {
            quote! {
                pub async fn #get_method_name(target_principal: String) -> std::result::Result<#return_ty, String> {
                    #finder_method_name(target_principal.clone()).await.find(()).await.map_err(|e| format!("{:?}", e))
                }

                pub async fn #get_unwrap_method_name(target_principal: String) -> #return_ty {
                    #finder_method_name(target_principal.clone()).await.find(()).await.unwrap()
                }

                async fn #finder_method_name(target_principal: String) -> chainsight_cdk::lens::AlgorithmLensFinder<#return_ty> {
                    use chainsight_cdk::lens::LensFinder;

                    let recipient = candid::Principal::from_text(target_principal).unwrap();
                    chainsight_cdk::lens::AlgorithmLensFinder::new(
                        chainsight_cdk::lens::LensTarget::<#return_ty>::new(
                            _get_target_proxy(recipient).await,
                            #call_method,
                        )
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_web3_event_indexer() {
        let input = quote! {Transfer};
        let args: syn::Result<Web3EventIndexerInput> = syn::parse2(input);
        let generated = web3_event_indexer_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__web3_event_indexer", formatted);
    }

    #[test]
    fn test_snapshot_algorithm_indexer() {
        let input = quote! {HashMap<u64, Vec<String>>, "get_list"};
        let args: syn::Result<AlgorithmIndexerInput> = syn::parse2(input);
        let generated = algorithm_indexer_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_indexer", formatted);
    }

    #[test]
    fn test_snapshot_algorithm_indexer_with_args() {
        let input = quote! {TransferEvent, (Principal, String, String), "get_transfers"};
        let args: syn::Result<AlgorithmIndexerWithArgsInput> = syn::parse2(input);
        let generated = algorithm_indexer_with_args_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_indexer_witg_args", formatted);
    }

    #[test]
    fn test_snapshot_algorithm_lens_finder() {
        let input = quote! {"user", "get_user", User, u64};
        let args: syn::Result<AlgorithmLensFinderInput> = syn::parse2(input);
        let generated = algorithm_lens_finder_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_lens_finder", formatted);
    }
}
