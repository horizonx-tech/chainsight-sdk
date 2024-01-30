use chainsight_cdk::config::components::AlgorithmLensConfig;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::canisters::utils::update_funcs_to_upgrade;

pub fn def_algorithm_lens_canister(input: TokenStream) -> TokenStream {
    let input_json_string: String = parse_macro_input!(input as syn::LitStr).value();
    let config: AlgorithmLensConfig =
        serde_json::from_str(&input_json_string).expect("Failed to parse input_json_string");
    algorithm_lens_canister(config).into()
}

fn algorithm_lens_canister(config: AlgorithmLensConfig) -> proc_macro2::TokenStream {
    let canister_name = config.common.canister_name.clone();
    let canister_name_ident = format_ident!("{}", config.common.canister_name);
    let lens_size = config.target_count;
    let lens_method_quote = if let Some(args_type) = config.args_type {
        let args_type_ident = format_ident!("{}", args_type);
        quote! { lens_method!(#lens_size, #args_type_ident); }
    } else {
        quote! { lens_method!(#lens_size); }
    };

    let quote_to_upgradable = {
        let state_struct = quote! {
            #[derive(Clone, Debug, PartialEq, candid::CandidType, serde::Serialize, serde::Deserialize, CborSerde)]
            pub struct UpgradeStableState {
                pub initializing_state: InitializingState,
            }
        };

        let update_funcs_to_upgrade = update_funcs_to_upgrade(
            quote! {
                UpgradeStableState {
                    initializing_state: get_initializing_state()
                }
            },
            quote! {
                set_initializing_state(state.initializing_state);
            },
        );

        quote! {
            #state_struct
            #update_funcs_to_upgrade
        }
    };

    quote! {
        did_export!(#canister_name);
        use candid::{Decode, Encode};
        use chainsight_cdk_macros::{chainsight_common, did_export, init_in, lens_method, manage_single_state, stable_memory_for_scalar, prepare_stable_structure, StableMemoryStorable, CborSerde};
        use ic_stable_structures::writer::Writer;
        use ic_web3_rs::futures::{future::BoxFuture, FutureExt};
        chainsight_common!();
        init_in!();
        prepare_stable_structure!();
        use #canister_name_ident::*;
        #lens_method_quote
        #quote_to_upgradable
    }
}

#[cfg(test)]
mod test {
    use chainsight_cdk::config::components::CommonConfig;
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_without_args() {
        let config = AlgorithmLensConfig {
            common: CommonConfig {
                canister_name: "app".to_string(),
            },
            target_count: 10,
            args_type: None,
        };
        let generated = algorithm_lens_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_lens", formatted);
    }

    #[test]
    fn test_snapshot_with_args() {
        let config = AlgorithmLensConfig {
            common: CommonConfig {
                canister_name: "app".to_string(),
            },
            target_count: 10,
            args_type: Some("CalculateArgs".to_string()),
        };
        let generated = algorithm_lens_canister(config);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__algorithm_lens_with_args", formatted);
    }
}
