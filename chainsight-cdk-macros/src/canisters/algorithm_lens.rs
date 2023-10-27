use chainsight_cdk::config::components::AlgorithmLensConfig;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

pub fn def_algorithm_lens_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: AlgorithmLensConfig = serde_json::from_str(&input_json_string).unwrap();
    algorithm_lens_canister(config).into()
}

fn algorithm_lens_canister(config: AlgorithmLensConfig) -> proc_macro2::TokenStream {
    let monitor_duration = config.common.monitor_duration;
    let canister_name = config.common.canister_name.clone();
    let canister_name_ident = format_ident!("{}", config.common.canister_name);
    let lens_size = config.target_count;
    quote! {
        did_export!(#canister_name);
        use chainsight_cdk_macros::{chainsight_common, did_export, init_in, lens_method};
        use ic_web3_rs::futures::{future::BoxFuture, FutureExt};
        chainsight_common!(#monitor_duration);
        init_in!();
        use #canister_name_ident::*;
        lens_method!(#lens_size);

    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_snapshot;
    use rust_format::{RustFmt, Formatter};

    use super::*;

    #[test]
    fn test_snapshot() {
        let config = AlgorithmLensConfig {
            common: CommonConfig {
                monitor_duration: 1000,
                canister_name: "app".to_string()
            },
            target_count: 10,
        };
        let generated = algorithm_lens_canister(config);
        let formatted = RustFmt::default().format_str(generated.to_string()).expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_lens", formatted);
    }
}
