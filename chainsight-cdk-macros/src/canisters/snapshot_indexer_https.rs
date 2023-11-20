use chainsight_cdk::config::components::SnapshotIndexerHTTPSConfig;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::canisters::utils::generate_queries_without_timestamp;

pub fn def_snapshot_indexer_https(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: SnapshotIndexerHTTPSConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    snapshot_indexer_https(config).into()
}

fn snapshot_indexer_https(config: SnapshotIndexerHTTPSConfig) -> proc_macro2::TokenStream {
    let use_idents = generate_use_idents(&config.common.canister_name);
    let custom = custom_code(config);
    quote! {
        #use_idents
        #custom
    }
}

fn generate_use_idents(id: &str) -> proc_macro2::TokenStream {
    let id_ident = format_ident!("{}", id);

    quote! {
        use std::collections::HashMap;

        use chainsight_cdk::core::HttpsSnapshotIndexerSourceAttrs;
        use chainsight_cdk::web2::{HttpsSnapshotParam, Web2HttpsSnapshotIndexer};
        use chainsight_cdk_macros::{
            chainsight_common, did_export, init_in, prepare_stable_structure, stable_memory_for_vec,
            timer_task_func, snapshot_indexer_https_source, StableMemoryStorable,
        };
        use candid::{Decode, Encode};
        use #id_ident::*;
    }
}

fn custom_code(config: SnapshotIndexerHTTPSConfig) -> proc_macro2::TokenStream {
    let SnapshotIndexerHTTPSConfig {
        common,
        url,
        headers,
        queries,
    } = config;

    let id = &common.canister_name;
    let header_keys: Vec<String> = headers.keys().cloned().collect();
    let header_values: Vec<String> = headers.values().cloned().collect();
    let query_keys: Vec<String> = queries.keys().cloned().collect();
    let query_values: Vec<String> = queries.values().cloned().collect();
    let queries = generate_queries_without_timestamp(format_ident!("SnapshotValue"));

    quote! {
        did_export!(#id); // NOTE: need to be declared before query, update
        init_in!();
        chainsight_common!();
        snapshot_indexer_https_source!();

        #[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize, StableMemoryStorable)]
        #[stable_mem_storable_opts(max_size = 10000, is_fixed_size = false)] // temp: max_size
        pub struct Snapshot {
            pub value: SnapshotValue,
            pub timestamp: u64,
        }
        prepare_stable_structure!();
        stable_memory_for_vec!("snapshot", Snapshot, 0, true);
        timer_task_func!("set_task", "index");

        const URL : &str = #url;
        fn get_attrs() -> HttpsSnapshotIndexerSourceAttrs {
            HttpsSnapshotIndexerSourceAttrs {
                queries: HashMap::from([
                    #(
                        (#query_keys.to_string(), #query_values.to_string()),
                    )*
                ]),
            }
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn transform_https_response(response: ic_cdk::api::management_canister::http_request::TransformArgs) -> ic_cdk::api::management_canister::http_request::HttpResponse {
            use chainsight_cdk::web3::TransformProcessor;
            let processor = chainsight_cdk::web2::processors::HTTPSResponseTransformProcessor::<SnapshotValue>::new();
            processor.transform(response)
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }
            let indexer = Web2HttpsSnapshotIndexer::new(
                URL.to_string(),
            );
            let res = indexer.get::<String, SnapshotValue>(
                HttpsSnapshotParam {
                    headers: vec![
                        #(
                            (#header_keys.to_string(), #header_values.to_string()),
                        )*
                    ].into_iter().collect(),
                    queries: vec![
                        #(
                            (#query_keys.to_string(), #query_values.to_string()),
                        )*
                    ].into_iter().collect(),
                }
            ).await.expect("Failed to get by indexer");
            let snapshot = Snapshot {
                value: res,
                timestamp: ic_cdk::api::time() / 1000000,
            };
            let _ = add_snapshot(snapshot.clone());
        }
        #queries
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_display_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = SnapshotIndexerHTTPSConfig {
            common: CommonConfig {
                canister_name: "sample_snapshot_indexer_https".to_string(),
            },
            url: "https://api.coingecko.com/api/v3/simple/price".to_string(),
            headers: BTreeMap::from([("content-type".to_string(), "application/json".to_string())]),
            queries: BTreeMap::from([
                ("ids".to_string(), "dai".to_string()),
                ("vs_currencies".to_string(), "usd".to_string()),
            ]),
        };
        let generated = snapshot_indexer_https(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!("snapshot__snapshot_indexer_https", formatted);
    }
}
