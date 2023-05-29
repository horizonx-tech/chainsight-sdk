use proc_macro::TokenStream;
use syn::{parse::Parser, parse_macro_input};
use quote::{quote, format_ident};

pub fn cross_canister_call_func(input: TokenStream) -> TokenStream {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args = parser.parse(input).expect("Failed to parse input");
    if args.len() != 3 {
        panic!("Expected exactly 3 arguments");
    }

    let fn_name = match &args[0] {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Str(lit_str) = &lit.lit {
                lit_str.value()
            } else {
                panic!("Expected a string literal for the function name");
            }
        }
        _ => panic!("Expected a string literal for the function name"),
    };
    let call_fn_name = format_ident!("call_{}", fn_name);
    let args_type = &args[1];
    let result_type = &args[2];
    
    let output = quote! {
        async fn #call_fn_name(
            canister_id: candid::Principal,
            call_args: #args_type,
        ) -> Result<#result_type, String> {
            let res = ic_cdk::api::call::call::<_, (#result_type,)>(canister_id, #fn_name, call_args)
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
