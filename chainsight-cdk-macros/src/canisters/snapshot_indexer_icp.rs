use chainsight_cdk::{
    config::components::{
        CommonConfig, LensParameter, SnapshotIndexerICPConfig, LENS_FUNCTION_ARGS_TYPE,
    },
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
    let common = common_code(&config.common, config.lens_parameter.is_some());
    let custom = custom_code(config);
    quote! {
        #common
        #custom
    }
}

fn common_code(config: &CommonConfig, with_lens: bool) -> proc_macro2::TokenStream {
    let id = &config.canister_name;

    let lens_targets_quote = if with_lens {
        quote! { lens_targets: Vec<String> }
    } else {
        quote! {}
    };

    quote! {
        use candid::{Decode, Encode};
        use chainsight_cdk_macros::{init_in, manage_single_state, setup_func, prepare_stable_structure, stable_memory_for_scalar, stable_memory_for_btree_map, StableMemoryStorable, timer_task_func, chainsight_common, did_export, CborSerde, snapshot_indexer_icp_source};
        use chainsight_cdk::rpc::{CallProvider, Caller, Message};
        use ic_stable_structures::writer::Writer;

        mod types;

        did_export!(#id); // NOTE: need to be declared before query, update

        init_in!(2);
        chainsight_common!();

        stable_memory_for_scalar!("target_canister", String, 3, false);
        setup_func!({
            target_canister: String,
            #lens_targets_quote
        }, 5);

        prepare_stable_structure!();
        stable_memory_for_btree_map!("snapshot", Snapshot, 1, true);
        timer_task_func!("set_task", "index", 6);
    }
}

fn custom_code(config: SnapshotIndexerICPConfig) -> proc_macro2::TokenStream {
    let SnapshotIndexerICPConfig {
        common: CommonConfig { canister_name },
        method_identifier: method_identifier_str,
        is_target_component,
        lens_parameter,
    } = config;

    let canister_name_ident = format_ident!("{}", &canister_name);

    let method_identifier =
        CanisterMethodIdentifier::new(&method_identifier_str).expect("invalid method identifier");
    let method_ident = method_identifier.identifier.to_string();

    let response_ty_def_ident = {
        let types_mod_ident = format_ident!("types");
        let response_ty_name_def_ident =
            format_ident!("{}", CanisterMethodIdentifier::RESPONSE_TYPE_NAME);
        quote! { #types_mod_ident::#response_ty_name_def_ident }
    };

    let (snapshot_idents, queries_expect_timestamp) = (
        quote! {
            #[derive(Clone, Debug, candid::CandidType, candid::Deserialize, serde::Serialize, StableMemoryStorable)]
            pub struct Snapshot {
                pub value: SnapshotValue,
                pub timestamp: u64,
            }
            pub type SnapshotValue = #response_ty_def_ident;
        },
        generate_queries_without_timestamp(format_ident!("SnapshotValue")),
    );

    let (call_args_ident, source_ident) = if let Some(LensParameter { with_args }) = lens_parameter
    {
        let call_args_ident = if with_args {
            let lens_args_ident = format_ident!("{}", LENS_FUNCTION_ARGS_TYPE);
            quote! {
                type CallCanisterArgs = #canister_name_ident::#lens_args_ident;
                pub fn call_args() -> CallCanisterArgs {
                    #canister_name_ident::#lens_args_ident {
                        targets: get_lens_targets().into(),
                        args: #canister_name_ident::call_args(),
                    }
                }
            }
        } else {
            quote! {
                type CallCanisterArgs = Vec<String>;
                #[ic_cdk::query]
                #[candid::candid_method(query)]
                pub fn call_args() -> CallCanisterArgs {
                    get_lens_targets().into()
                }
            }
        };
        (
            quote! {
                stable_memory_for_scalar!("lens_targets", chainsight_cdk::storage::StorableStrings, 4, false);
                #call_args_ident
            },
            quote! { snapshot_indexer_icp_source!(#method_ident, "get_lens_targets"); },
        )
    } else {
        (
            quote! {
                type CallCanisterArgs = #canister_name_ident::CallCanisterArgs;
                #[ic_cdk::query]
                #[candid::candid_method(query)]
                pub fn call_args() -> CallCanisterArgs {
                    #canister_name_ident::call_args()
                }
            },
            quote! { snapshot_indexer_icp_source!(#method_ident); },
        )
    };

    let quote_to_call_target =
        generate_quote_to_call_target(is_target_component, method_ident.clone());

    quote! {
        #snapshot_idents

        #queries_expect_timestamp

        #source_ident

        #call_args_ident
        type CallCanisterResponse = SnapshotValue;

        #quote_to_call_target

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn index() {
            if ic_cdk::caller() != proxy() {
                panic!("Not permitted")
            }

            let current_ts_sec = ic_cdk::api::time() / 1000000;
            let target_canister = candid::Principal::from_text(get_target_canister()).expect("invalid principal");
            let value = call_target_method_to_target_canister(target_canister, call_args()).await;
            let datum = Snapshot {
                value: value.clone(),
                timestamp: current_ts_sec,
            };
            add_snapshot(datum.clone());

            ic_cdk::println!("timestamp={}, value={:?}", datum.timestamp, datum.value);
        }
    }
}

// Generate token stream for `call_target_method_to_target_canister`
fn generate_quote_to_call_target(
    is_target_component: bool,
    method_identifier: String,
) -> proc_macro2::TokenStream {
    let logic = if is_target_component {
        // NOTE: Call via `proxy_` function when target is in Chainsight
        let proxy_method_identifier = "proxy_".to_string() + &method_identifier;
        quote! {
            let px = _get_target_proxy(target).await;
            let call_result = CallProvider::new()
                .call(
                    Message::new::<CallCanisterArgs>(
                        call_args,
                        px.clone(),
                        #proxy_method_identifier
                    ).expect("failed to create message"),
                ).await.expect("failed to call");

            call_result.reply::<CallCanisterResponse>().expect("failed to get reply")
        }
    } else {
        quote! {
            let out: ic_cdk::api::call::CallResult<(SnapshotValue,)> = ic_cdk::api::call::call(
                target,
                #method_identifier,
                call_args
            ).await;
            out.expect("failed to call").0
        }
    };

    quote! {
        async fn call_target_method_to_target_canister(target: candid::Principal, call_args: CallCanisterArgs) -> SnapshotValue {
            #logic
        }
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_display_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    fn config() -> SnapshotIndexerICPConfig {
        SnapshotIndexerICPConfig {
            common: CommonConfig {
                canister_name: "sample_snapshot_indexer_icp".to_string(),
            },
            method_identifier:
                "get_last_snapshot : () -> (record { value : text; timestamp : nat64 })".to_string(),
            is_target_component: true,
            lens_parameter: None,
        }
    }

    #[test]
    fn test_snapshot() {
        let generated = snapshot_indexer_icp(config());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!("snapshot__snapshot_indexer_icp", formatted);
    }

    #[test]
    fn test_snapshot_with_lens() {
        let mut config = config();
        config.lens_parameter = Some(LensParameter { with_args: false });

        let generated = snapshot_indexer_icp(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!("snapshot__snapshot_indexer_icp__with_lens", formatted);
    }

    #[test]
    fn test_snapshot_with_lens_with_args() {
        let mut config = config();
        config.lens_parameter = Some(LensParameter { with_args: true });

        let generated = snapshot_indexer_icp(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!(
            "snapshot__snapshot_indexer_icp__with_lens_with_args",
            formatted
        );
    }

    #[test]
    fn test_snapshot_target_is_not_component() {
        let config = SnapshotIndexerICPConfig {
            common: CommonConfig {
                canister_name: "sample_snapshot_indexer_icp".to_string(),
            },
            method_identifier: "icrc1_total_supply : () -> (nat)".to_string(),
            is_target_component: false,
            lens_parameter: None,
        };

        let generated = snapshot_indexer_icp(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_display_snapshot!(
            "snapshot__snapshot_indexer_icp__target_is_not_component",
            formatted
        );
    }
}
