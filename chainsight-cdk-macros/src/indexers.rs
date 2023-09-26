use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
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

pub struct AlgorithmIndexerWithArgsInput {
    in_type: syn::Type,
    args: syn::Type,
    call_method: syn::LitStr,
}

pub struct AlgorithmLensFinderInput {
    id: syn::LitStr,
    call_method: syn::LitStr,
    return_ty: syn::Type,
    args_ty: Option<syn::Type>,
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
    }
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
    }
}

pub fn relayer_source(input: TokenStream) -> TokenStream {
    let from_lens: syn::LitBool = parse_macro_input!(input as syn::LitBool);
    if from_lens.value {
        return quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
            fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::RelayerWithLensSourceAttrs>> {
                vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::RelayerWithLensSourceAttrs>::new_relayer(
                get_target_canister(),
                get_timer_duration(),
                call_args()),
                ]
            }
        }.into();
    }

    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::RelayerWithLensSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::RelayerWithLensSourceAttrs>::new_relayer(
            get_target_canister(),
            get_timer_duration(),
            vec![])
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
                get_target_canister(),
                get_timer_duration(),
                #func_name.to_string(),
            )]
        }
    }.into()
}

pub fn snapshot_web3_source(input: TokenStream) -> TokenStream {
    let func_name: syn::LitStr = parse_macro_input!(input as syn::LitStr);
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3SnapshotIndexerSourceAttrs>> {
            vec![chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3SnapshotIndexerSourceAttrs>::new_web3_snapshot_indexer(
                get_target_addr(),
                get_timer_duration(),
                get_web3_ctx_param().chain_id,
                #func_name.to_string(),
            )]
        }
    }.into()
}

pub fn algorithm_indexer_with_args(input: TokenStream) -> TokenStream {
    let AlgorithmIndexerWithArgsInput {
        in_type,
        args,
        call_method,
    } = parse_macro_input!(input as AlgorithmIndexerWithArgsInput);
    let source = generate_algorithm_indexer_source();
    quote! {
        mod app;
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
        #source
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
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
    .into()
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
        async fn indexer() -> chainsight_cdk::algorithm::AlgorithmIndexer<#in_type> {
            chainsight_cdk::algorithm::AlgorithmIndexer::new_with_method(_get_target_proxy(get_target()).await, #call_method, persist)
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
    .into()
}

pub fn algorithm_lens_finder(input: TokenStream) -> TokenStream {
    let AlgorithmLensFinderInput {
        id,
        call_method,
        return_ty,
        args_ty,
    } = parse_macro_input!(input as AlgorithmLensFinderInput);

    let finder_method_name = format_ident!("finder_{}", id.value().to_lowercase());
    let get_method_name = format_ident!("get_{}", id.value().to_lowercase());
    let get_unwrap_method_name = format_ident!("get_{}_unwrap", id.value().to_lowercase());

    let call_functions = match args_ty {
        Some(args_ty) => {
            quote! {
                pub async fn #get_method_name(target_principal: String, args: #args_ty) -> std::result::Result<#return_ty, String> {
                    #finder_method_name(target_principal.clone()).find(args).await.map_err(|e| format!("{:?}", e))
                }

                pub async fn #get_unwrap_method_name(target_principal: String, args: #args_ty) -> #return_ty {
                    #finder_method_name(target_principal.clone()).find_unwrap(args).await
                }
            }
        }
        None => {
            quote! {
                pub async fn #get_method_name(target_principal: String) -> std::result::Result<#return_ty, String> {
                    #finder_method_name(target_principal.clone()).await.find(()).await.map_err(|e| format!("{:?}", e))
                }

                pub async fn #get_unwrap_method_name(target_principal: String) -> #return_ty {
                    #finder_method_name(target_principal.clone()).await.find_unwrap(()).await
                }
            }
        }
    };

    quote! {
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

        #call_functions
    }.into()
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
        async fn proxy_call(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            chainsight_cdk::rpc::ReceiverProvider::<(u64, u64), HashMap<u64, Vec<#out_type>>>::new(
                proxy(),
                _events_from_to.clone(),
            )
            .reply(input)
            .await
        }
    };

    output
}
