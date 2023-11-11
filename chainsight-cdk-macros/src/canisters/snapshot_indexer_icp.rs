use candid::Principal;
use chainsight_cdk::{
    config::components::{CommonConfig, SnapshotIndexerICPConfig},
    convert::candid::CanisterMethodIdentifier,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::canisters::utils::generate_queries_without_timestamp;

pub fn def_snapshot_indexer_icp(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: SnapshotIndexerICPConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    snapshot_indexer_icp(config).into()
}

fn snapshot_indexer_icp(config: SnapshotIndexerICPConfig) -> proc_macro2::TokenStream {
    let common = common_code(&config.common);
    let custom = custom_code(config);
    quote! {
        #common
        #custom
    }
}

fn common_code(config: &CommonConfig) -> proc_macro2::TokenStream {
    let id = &config.canister_name;

    quote! {
        use candid::{Decode, Encode};
        use chainsight_cdk_macros::{init_in,manage_single_state, setup_func, prepare_stable_structure, stable_memory_for_vec, StableMemoryStorable, timer_task_func, chainsight_common, did_export, snapshot_indexer_icp_source};
        use chainsight_cdk::rpc::{CallProvider, Caller, Message};

        mod types;

        did_export!(#id); // NOTE: need to be declared before query, update

        init_in!();
        chainsight_common!();

        manage_single_state!("target_canister", String, false);
        setup_func!({ target_canister: String });

        prepare_stable_structure!();
        stable_memory_for_vec!("snapshot", Snapshot, 0, true);
        timer_task_func!("set_task", "index");
    }
}

fn custom_code(config: SnapshotIndexerICPConfig) -> proc_macro2::TokenStream {
    let SnapshotIndexerICPConfig {
        common: CommonConfig { canister_name },
        method_identifier: method_identifier_str,
        lens_targets,
    } = config;

    let bindings_crate_ident = format_ident!("{}", &canister_name);

    let method_identifier =
        CanisterMethodIdentifier::new(&method_identifier_str).expect("invalid method identifier");
    let method_ident = "proxy_".to_string() + &method_identifier.identifier;

    let response_ty_def_ident = {
        let types_mod_ident = format_ident!("types");
        let response_ty_name_def_ident =
            format_ident!("{}", CanisterMethodIdentifier::RESPONSE_TYPE_NAME);
        quote! { #types_mod_ident::#response_ty_name_def_ident }
    };

    let (snapshot_idents, queries_expect_timestamp) = (
        quote! {
            #[derive(Clone, Debug, candid::CandidType, candid::Deserialize, serde::Serialize, StableMemoryStorable)]
            #[stable_mem_storable_opts(max_size = 10000, is_fixed_size = false)] // temp: max_size
            pub struct Snapshot {
                pub value: SnapshotValue,
                pub timestamp: u64,
            }
            pub type SnapshotValue = #response_ty_def_ident;
        },
        generate_queries_without_timestamp(format_ident!("SnapshotValue")),
    );

    let call_canister_args_ident = if let Some(lens_targets) = lens_targets {
        let lens_target_principals: Vec<Principal> = lens_targets
            .identifiers
            .iter()
            .map(|p| {
                Principal::from_text(p)
                    .unwrap_or_else(|_| panic!("lens target must be principal '{}'", p))
            })
            .collect();

        let lens_targets_string_ident: Vec<_> =
            lens_target_principals.iter().map(|p| p.to_text()).collect();

        quote! {
            type CallCanisterArgs = Vec<String>;
            pub fn call_args() -> CallCanisterArgs {
                vec![
                    #(#lens_targets_string_ident.to_string()),*
                ]
            }
        }
    } else {
        quote! {
            type CallCanisterArgs = #bindings_crate_ident::CallCanisterArgs;
            pub fn call_args() -> CallCanisterArgs {
                #bindings_crate_ident::call_args()
            }
        }
    };

    quote! {
        #snapshot_idents

        #queries_expect_timestamp

        snapshot_indexer_icp_source!(#method_ident);

        #call_canister_args_ident
        type CallCanisterResponse = SnapshotValue;

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }

            let current_ts_sec = ic_cdk::api::time() / 1000000;
            let target_canister = candid::Principal::from_text(get_target_canister()).unwrap();
            let px = _get_target_proxy(target_canister).await;
            let call_result = CallProvider::new()
                .call(
                    Message::new::<CallCanisterArgs>(
                        call_args(),
                        px.clone(),
                        #method_ident
                    ).unwrap()
                ).await.unwrap();

            let value = call_result.reply::<CallCanisterResponse>().unwrap();
            let datum = Snapshot {
                value: value.clone(),
                timestamp: current_ts_sec,
            };
            let _ = add_snapshot(datum.clone());

            ic_cdk::println!("ts={}, value={:?}", datum.timestamp, datum.value);
        }
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::{CommonConfig, LensTargets};
    use insta::assert_display_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = SnapshotIndexerICPConfig {
            common: CommonConfig {
                canister_name: "sample_snapshot_indexer_icp".to_string(),
            },
            method_identifier:
                "get_last_snapshot : () -> (record { value : text; timestamp : nat64 })".to_string(),
            lens_targets: None,
        };
        let generated = snapshot_indexer_icp(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!("snapshot__snapshot_indexer_icp", formatted);
    }

    #[test]
    fn test_snapshot_with_lens_targets() {
        let config = SnapshotIndexerICPConfig {
            common: CommonConfig {
                canister_name: "sample_snapshot_indexer_icp".to_string(),
            },
            method_identifier:
                "get_last_snapshot : () -> (record { value : text; timestamp : nat64 })".to_string(),
            lens_targets: Some(LensTargets {
                identifiers: vec![
                    "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(), // NNS Ledger
                    "zfcdd-tqaaa-aaaaq-aaaga-cai".to_string(), // SNS-1
                    "mxzaz-hqaaa-aaaar-qaada-cai".to_string(), // ckBTC
                ],
            }),
        };
        let generated = snapshot_indexer_icp(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!(
            "snapshot__snapshot_indexer_icp__with_lens_targets",
            formatted
        );
    }
}
