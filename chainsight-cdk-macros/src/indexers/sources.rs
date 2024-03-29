use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
};

pub fn web3_event_indexer_source(input: TokenStream) -> TokenStream {
    let tt = parse_macro_input!(input as syn::Type);
    web3_event_indexer_source_internal(tt).into()
}
fn web3_event_indexer_source_internal(tt: syn::Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#tt).to_string();
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3EventIndexerSourceAttrs>> {
            vec![
                chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3EventIndexerSourceAttrs>::new_event_indexer(
                    get_target_addr(),
                    get_indexing_interval(),
                    chainsight_cdk::core::Web3EventIndexerSourceAttrs {
                        chain_id: get_web3_ctx_param().chain_id,
                        event_name: #type_str.to_string(),
                    }
                )
            ]
        }
    }
}

pub fn algorithm_indexer_source(_input: TokenStream) -> TokenStream {
    algorithm_indexer_source_internal().into()
}
fn algorithm_indexer_source_internal() -> proc_macro2::TokenStream {
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

pub fn snapshot_indexer_web3_source(input: TokenStream) -> TokenStream {
    let func_name: syn::LitStr = parse_macro_input!(input as syn::LitStr);
    snapshot_indexer_web3_source_internal(func_name).into()
}
fn snapshot_indexer_web3_source_internal(func_name: syn::LitStr) -> proc_macro2::TokenStream {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::Web3SnapshotIndexerSourceAttrs>> {
            vec![
                chainsight_cdk::core::Sources::<chainsight_cdk::core::Web3SnapshotIndexerSourceAttrs>::new_web3_snapshot_indexer(
                    get_target_addr(),
                    get_indexing_interval(),
                    get_web3_ctx_param().chain_id,
                    #func_name.to_string(),
                )
            ]
        }
    }
}

pub fn snapshot_indexer_https_source(_input: TokenStream) -> TokenStream {
    snapshot_indexer_https_source_internal().into()
}
fn snapshot_indexer_https_source_internal() -> proc_macro2::TokenStream {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::HttpsSnapshotIndexerSourceAttrs>> {
            vec![
                chainsight_cdk::core::Sources::<chainsight_cdk::core::HttpsSnapshotIndexerSourceAttrs>::new_https_snapshot_indexer(
                    URL.to_string(),
                    get_indexing_interval(),
                    get_attrs(),
                )
            ]
        }
    }
}

pub struct SnapshotIndexerICPSourceInput {
    method_identifier: syn::LitStr,
    getter_ids_func_name_for_lens: Option<syn::LitStr>,
}
impl Parse for SnapshotIndexerICPSourceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method_identifier: syn::LitStr = input.parse()?;
        let getter_ids_func_name_for_lens = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(SnapshotIndexerICPSourceInput {
            method_identifier,
            getter_ids_func_name_for_lens,
        })
    }
}
pub fn snapshot_indexer_icp_source(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SnapshotIndexerICPSourceInput);
    snapshot_indexer_icp_source_internal(args).into()
}
pub fn snapshot_indexer_icp_source_internal(
    args: SnapshotIndexerICPSourceInput,
) -> proc_macro2::TokenStream {
    let SnapshotIndexerICPSourceInput {
        method_identifier,
        getter_ids_func_name_for_lens,
    } = args;

    if let Some(func_name) = getter_ids_func_name_for_lens {
        let func_ident = format_ident!("{}", func_name.value());
        return quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
            fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::ICSnapshotIndexerSourceAttrs>> {
                vec![
                    chainsight_cdk::core::Sources::<chainsight_cdk::core::ICSnapshotIndexerSourceAttrs>::new_snapshot_indexer(
                        get_target_canister(),
                        get_indexing_interval(),
                        #method_identifier,
                        #func_ident().into()
                    )
                ]
            }
        };
    }
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::ICSnapshotIndexerSourceAttrs>> {
            vec![
                chainsight_cdk::core::Sources::<chainsight_cdk::core::ICSnapshotIndexerSourceAttrs>::new_snapshot_indexer(
                    get_target_canister(),
                    get_indexing_interval(),
                    #method_identifier,
                    vec![]
                )
            ]
        }
    }
}

pub struct RelayerSourceInput {
    method_identifier: syn::LitStr,
    getter_ids_func_name_for_lens: Option<syn::LitStr>,
}
impl Parse for RelayerSourceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method_identifier: syn::LitStr = input.parse()?;
        let getter_ids_func_name_for_lens = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(RelayerSourceInput {
            method_identifier,
            getter_ids_func_name_for_lens,
        })
    }
}
pub fn relayer_source(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as RelayerSourceInput);
    relayer_source_internal(args).into()
}
fn relayer_source_internal(args: RelayerSourceInput) -> proc_macro2::TokenStream {
    let RelayerSourceInput {
        method_identifier,
        getter_ids_func_name_for_lens,
    } = args;

    if let Some(func_name) = getter_ids_func_name_for_lens {
        let func_ident = format_ident!("{}", func_name.value());

        return quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
            fn get_sources() -> Vec<chainsight_cdk::core::Sources<chainsight_cdk::core::RelayerWithLensSourceAttrs>> {
                vec![
                    chainsight_cdk::core::Sources::<chainsight_cdk::core::RelayerWithLensSourceAttrs>::new_relayer(
                        get_target_canister(),
                        get_indexing_interval(),
                        #method_identifier,
                        #func_ident().into()
                    ),
                ]
            }
        };
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
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_algorithm_indexer_source() {
        let generated = algorithm_indexer_source_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_indexer_source", formatted);
    }

    #[test]
    fn test_snapshot_web3_event_indexer_source() {
        let input = quote! {Transfer};
        let args: syn::Result<syn::Type> = syn::parse2(input);
        let generated = web3_event_indexer_source_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__web3_event_indexer_source", formatted);
    }

    #[test]
    fn test_snapshot_snapshot_indexer_web3_source() {
        let input = quote! {"total_supply"};
        let args: syn::Result<syn::LitStr> = syn::parse2(input);
        let generated = snapshot_indexer_web3_source_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__snapshot_indexer_web3_source", formatted);
    }

    #[test]
    fn test_snapshot_snapshot_indexer_https_source() {
        let generated = snapshot_indexer_https_source_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__snapshot_indexer_https_source", formatted);
    }

    #[test]
    fn test_snapshot_snapshot_indexer_icp_source() {
        let input = quote! {"icrc1_balance_of"};
        let args: syn::Result<SnapshotIndexerICPSourceInput> = syn::parse2(input);
        let generated = snapshot_indexer_icp_source_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__snapshot_indexer_icp_source", formatted);
    }

    #[test]
    fn test_snapshot_snapshot_indexer_icp_source_from_lens() {
        let input = quote! {"calculate", "get_lens_targets"};
        let args: syn::Result<SnapshotIndexerICPSourceInput> = syn::parse2(input);
        let generated = snapshot_indexer_icp_source_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!(
            "snapshot__snapshot_indexer_icp_source__from_lens",
            formatted
        );
    }

    #[test]
    fn test_snapshot_relayer_source() {
        let input = quote! {"icrc1_balance_of"};
        let args: syn::Result<RelayerSourceInput> = syn::parse2(input);
        let generated = relayer_source_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__relayer_source", formatted);
    }

    #[test]
    fn test_snapshot_relayer_source_from_lens() {
        let input = quote! {"calculate", "get_lens_targets"};
        let args: syn::Result<RelayerSourceInput> = syn::parse2(input);
        let generated = relayer_source_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__relayer_source__from_lens", formatted);
    }
}
