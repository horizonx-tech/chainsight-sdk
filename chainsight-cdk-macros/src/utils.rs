use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn chainsight_common() -> TokenStream {
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
    fn test_snapshot_did_export() {
        let input = quote! {"sample_component"};
        let args: syn::Result<syn::LitStr> = syn::parse2(input);
        let generated = did_export_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__did_export", formatted);
    }
}
