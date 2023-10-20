use chainsight_cdk::config::components::AlgorithmLensConfig;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

pub fn def_algorithm_lens_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: AlgorithmLensConfig = serde_json::from_str(&input_json_string).unwrap();
    algorithm_lens_canister(config)
}

fn algorithm_lens_canister(config: AlgorithmLensConfig) -> TokenStream {
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
    .into()
}
