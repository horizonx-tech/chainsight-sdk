use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, LitBool, LitInt, Result,
};

pub fn chainsight_common(_input: TokenStream) -> TokenStream {
    chainsight_common_internal().into()
}
fn chainsight_common_internal() -> proc_macro2::TokenStream {
    quote! {
        async fn _get_target_proxy(target: candid::Principal) -> candid::Principal {
            let out: ic_cdk::api::call::CallResult<(candid::Principal,)> = ic_cdk::api::call::call(target, "get_proxy", ()).await;
            out.unwrap().0
        }
    }
}

#[derive(Default)]
struct DefineLoggerArgs {
    retention_days: Option<u8>,
    cleanup_interval_days: Option<u8>,
    disable_init_post_upgrade: bool,
}
impl Parse for DefineLoggerArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(DefineLoggerArgs::default());
        }
        let retention_days: Option<LitInt> = input.parse()?;
        let retention_days = retention_days.map(|x| x.base10_parse::<u8>().unwrap());
        if input.parse::<syn::Token![,]>().is_err() {
            return Ok(DefineLoggerArgs {
                retention_days,
                ..Default::default()
            });
        }
        let cleanup_interval_days: Option<LitInt> = input.parse()?;
        let cleanup_interval_days = cleanup_interval_days.map(|x| x.base10_parse::<u8>().unwrap());
        if input.parse::<syn::Token![,]>().is_err() {
            return Ok(DefineLoggerArgs {
                retention_days,
                cleanup_interval_days,
                ..Default::default()
            });
        }
        let disable_init_post_upgrade: Option<LitBool> = input.parse()?;
        Ok(DefineLoggerArgs {
            retention_days,
            cleanup_interval_days,
            disable_init_post_upgrade: disable_init_post_upgrade
                .map(|x| x.value)
                .unwrap_or_default(),
        })
    }
}
pub fn define_logger(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DefineLoggerArgs);
    define_logger_internal(args).into()
}

fn define_logger_internal(args: DefineLoggerArgs) -> proc_macro2::TokenStream {
    let retention_days = args.retention_days.unwrap_or(7);
    let cleanup_interval = args.cleanup_interval_days.unwrap_or(1) as u64 * 86400;
    let code = quote! {
        use chainsight_cdk::log::{Logger, LoggerImpl, TailRange, TailResponse};

        #[candid::candid_method(query)]
        #[ic_cdk::query]
        fn tail_logs(rows: usize, range: Option<TailRange>) -> TailResponse {
            _logger().tail(rows, range)
        }

        #[candid::candid_method(update)]
        #[ic_cdk::update]
        #[chainsight_cdk_macros::only_controller]
        fn drain_logs(rows: usize) -> Vec<String> {
            _logger().drain(rows)
        }

        fn schedule_cleanup() {
            ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(#cleanup_interval), || {
                ic_cdk::spawn(async {
                    _logger().sweep(#retention_days);
                })
            });
            _logger().info(format!("cleanup sheduled: interval = {} sec. retention days = {}", #cleanup_interval, #retention_days).as_str());
        }

        fn _init_logger() {
            schedule_cleanup();
        }
        fn _post_upgrade_logger() {
            schedule_cleanup();
        }
        fn _logger() -> LoggerImpl {
            LoggerImpl::new(Some("Logger"))
        }
    };
    if args.disable_init_post_upgrade {
        code
    } else {
        quote! {
            #code

            #[ic_cdk::init]
            fn init() {
                _init_logger();
            }
            #[ic_cdk::post_upgrade]
            fn post_upgrade() {
                _post_upgrade_logger();
            }
        }
    }
}

pub fn did_export(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as syn::LitStr);
    did_export_internal(args).into()
}
fn did_export_internal(args: syn::LitStr) -> proc_macro2::TokenStream {
    let file_name = args.value() + ".did";
    quote! {
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
    }
}

pub fn derive_cbor_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_cbor_serde_internal(&input.ident).into()
}
fn derive_cbor_serde_internal(struct_name: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl #struct_name {
            pub fn to_cbor(&self) -> Vec<u8> {
                let mut state_bytes = vec![];
                ciborium::ser::into_writer(self, &mut state_bytes).expect("Failed to serialize state");
                state_bytes
            }

            pub fn from_cbor(bytes: &[u8]) -> Self {
                ciborium::de::from_reader(bytes).expect("Failed to deserialize state")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_chainsight_common() {
        let generated = chainsight_common_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__chainsight_common", formatted);
    }

    #[test]
    fn test_snapshot_define_logger() {
        let generated = define_logger_internal(Default::default());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__define_logger", formatted);
    }

    #[test]
    fn test_snapshot_did_export() {
        let input = quote! {"sample_component"};
        let args: syn::Result<syn::LitStr> = syn::parse2(input);
        let generated = did_export_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__did_export", formatted);
    }

    #[test]
    fn test_snapshot_derive_cbor_serde() {
        let input = quote! {struct SampleComponent {}};
        let args: syn::Result<DeriveInput> = syn::parse2(input);
        let generated = derive_cbor_serde_internal(&args.unwrap().ident);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__derive_cbor_serde", formatted);
    }
}
