use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn chainsight_common(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::LitInt);
    let output = quote! {
        #[derive(Clone, Debug, PartialEq, candid::CandidType, candid::Deserialize, serde::Serialize)]
        pub struct CanisterMetricsSnapshot {
            pub timestamp: u64,
            pub cycles: u128,
        }
        chainsight_cdk_macros::manage_vec_state!("canister_metrics_snapshot", CanisterMetricsSnapshot, false);

        fn setup_monitoring_canister_metrics() {
            let round_timestamp = |ts: u32, unit: u32| ts / unit * unit;
            let current_time_sec = (ic_cdk::api::time() / (1000 * 1000000)) as u32;
            let delay = round_timestamp(current_time_sec, #item) + #item - current_time_sec;

            ic_cdk_timers::set_timer(std::time::Duration::from_secs(delay as u64), move || {
                ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(#item as u64), || {
                    monitor_canister_metrics();
                });
            });
            // in first time, immediate execution
            monitor_canister_metrics();
        }

        fn monitor_canister_metrics() {
            let timestamp = ic_cdk::api::time();
            let cycles = ic_cdk::api::canister_balance128();
            let datum = CanisterMetricsSnapshot {
                timestamp,
                cycles,
            };
            ic_cdk::println!("monitoring: {:?}", datum.clone());
            add_canister_metrics_snapshot(datum);
        }

        #[ic_cdk_macros::init]
        fn init() {
            setup_monitoring_canister_metrics()
        }

        #[ic_cdk_macros::post_upgrade]
        fn post_upgrade() {
            setup_monitoring_canister_metrics()
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn metric() -> CanisterMetricsSnapshot {
            get_last_canister_metrics_snapshot()
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        pub fn metrics(n: usize) -> Vec<CanisterMetricsSnapshot> {
            get_top_canister_metrics_snapshots(n)
        }

        async fn _get_target_proxy(target: candid::Principal) -> candid::Principal {
            let out: ic_cdk::api::call::CallResult<(candid::Principal,)> = ic_cdk::api::call::call(target, "get_proxy", ()).await;
            out.unwrap().0
        }
    };
    output.into()
}

pub fn did_export(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::LitStr);
    let file_name = item.value() + ".did";
    TokenStream::from(quote! {
        candid::export_service!();
        #[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
        #[candid::candid_method(query, rename = "__get_candid_interface_tmp_hack")]
        fn __export_did_tmp_() -> String {
            __export_service()
        }
        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn gen_candid() {
                std::fs::write(#file_name, __export_service()).unwrap();
            }
        }
    })
}
