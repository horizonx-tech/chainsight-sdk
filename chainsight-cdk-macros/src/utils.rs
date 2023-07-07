use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

struct CrossCanisterCallFuncInput {
    fn_name: syn::LitStr,
    args_type: syn::Type,
    result_type: syn::Type,
}
impl syn::parse::Parse for CrossCanisterCallFuncInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fn_name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let args_type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let result_type = input.parse()?;
        Ok(CrossCanisterCallFuncInput {
            fn_name,
            args_type,
            result_type,
        })
    }
}

pub fn cross_canister_call_func(input: TokenStream) -> TokenStream {
    let CrossCanisterCallFuncInput {
        fn_name,
        args_type,
        result_type,
    } = parse_macro_input!(input as CrossCanisterCallFuncInput);

    let call_fn_name = format_ident!("call_{}", &fn_name.value());

    let (args_type, args) = match args_type.clone() {
        syn::Type::Tuple(type_tuple) => {
            if type_tuple.elems.len() == 1 {
                // same as in the case of primitive type
                let single_arg_type = type_tuple.elems.first().unwrap().clone();
                (single_arg_type.clone(), quote! { (call_args,) })
            } else {
                (args_type, quote! { call_args })
            }
        }
        _ => (args_type, quote! { (call_args,) }),
    };

    let output = quote! {
        async fn #call_fn_name(
            canister_id: candid::Principal,
            call_args: #args_type,
        ) -> Result<#result_type, String> {
            let res = ic_cdk::api::call::call::<_, (#result_type,)>(canister_id, #fn_name, #args)
                .await
                .map_err(|e| format!("call error: {:?}", e))?;
            Ok(res.0)
        }
    };
    output.into()
}

pub fn monitoring_canister_metrics(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::LitInt);
    let output = quote! {
        #[derive(Clone, Debug, PartialEq, candid::CandidType, candid::Deserialize)]
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
                ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(#item), || {
                    let timestamp = ic_cdk::api::time();
                    let cycles = ic_cdk::api::canister_balance128();
                    let datum = CanisterMetricsSnapshot {
                        timestamp,
                        cycles,
                    };
                    ic_cdk::println!("monitoring: {:?}", datum.clone());
                    add_canister_metrics_snapshot(datum);
                });
            });
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
