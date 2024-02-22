use quote::quote;
use std::path::PathBuf;

use crate::internal::{attrs_query_func, attrs_update_func};

/// Convert camelCase String to snake_case
pub fn camel_to_snake(val: &str) -> String {
    // NOTE: use Inflator in ic-solidity-bindgen
    // https://github.com/horizonx-tech/ic-solidity-bindgen/blob/0972bede5957927bcb8f675decd93878b849dc76/ic-solidity-bindgen-macros/src/abi_gen.rs#L192
    inflector::cases::snakecase::to_snake_case(val)
}

pub fn extract_contract_name_from_path(s: &str) -> String {
    let path = PathBuf::from(s);
    let name = path.file_stem().expect("file_stem failed");
    name.to_str().expect("to_str failed").to_string()
}

pub fn generate_queries_without_timestamp(
    return_type: proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let query_derives = attrs_query_func();
    let update_derives = attrs_update_func();

    quote! {
        fn _get_last_snapshot_value() -> #return_type {
            get_last_snapshot().value
        }

        fn _get_top_snapshot_values(n: u64) -> Vec<#return_type> {
            get_top_snapshots(n).iter().map(|s| s.value.clone()).collect()
        }

        fn _get_snapshot_value(idx: u64) -> #return_type {
            get_snapshot(idx).value
        }

        #query_derives
        pub fn get_last_snapshot_value() -> #return_type {
            _get_last_snapshot_value()
        }

        #query_derives
        pub fn get_top_snapshot_values(n: u64) -> Vec<#return_type> {
            _get_top_snapshot_values(n)
        }

        #query_derives
        pub fn get_snapshot_value(idx: u64) -> #return_type {
            _get_snapshot_value(idx)
        }

        #update_derives
        pub async fn proxy_get_last_snapshot_value(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            chainsight_cdk::rpc::ReceiverProviderWithoutArgs::<#return_type>::new(
                proxy(),
                _get_last_snapshot_value,
            )
            .reply(input)
            .await
        }

        #update_derives
        pub async fn proxy_get_top_snapshot_values(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            chainsight_cdk::rpc::ReceiverProvider::<u64, Vec<#return_type>>::new(
                proxy(),
                _get_top_snapshot_values,
            )
            .reply(input)
            .await
        }

        #update_derives
        pub async fn proxy_get_snapshot_value(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            chainsight_cdk::rpc::ReceiverProvider::<u64, #return_type>::new(
                proxy(),
                _get_snapshot_value,
            )
            .reply(input)
            .await
        }
    }
}

#[allow(dead_code)]
pub fn update_funcs_to_upgrade(
    generate_state: proc_macro2::TokenStream,
    recover_from_state: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let pre_upgrade = quote! {
        #[ic_cdk::pre_upgrade]
        fn pre_upgrade() {
            ic_cdk::println!("start: pre_upgrade");

            let state = #generate_state;
            let state_bytes = state.to_cbor();

            let len = state_bytes.len() as u32;
            let mut memory = get_upgrades_memory();
            let mut writer = Writer::new(&mut memory, 0);
            writer.write(&len.to_le_bytes()).unwrap();
            writer.write(&state_bytes).unwrap();

            ic_cdk::println!("finish: pre_upgrade");
        }
    };

    let post_upgrade = quote! {
        #[ic_cdk::post_upgrade]
        fn post_upgrade() {
            ic_cdk::println!("start: post_upgrade");

            let memory = get_upgrades_memory();

            // Read the length of the state bytes.
            let mut state_len_bytes = [0; 4];
            memory.read(0, &mut state_len_bytes);
            let state_len = u32::from_le_bytes(state_len_bytes) as usize;

            // Read the bytes
            let mut state_bytes = vec![0; state_len];
            memory.read(4, &mut state_bytes);

            // Restore
            let state = UpgradeStableState::from_cbor(&state_bytes);
            #recover_from_state

            ic_cdk::println!("finish: post_upgrade");
        }
    };

    quote! {
        #pre_upgrade
        #post_upgrade
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_contract_name_from_path() {
        let path = "__interfaces/Oracle.json";
        assert_eq!(extract_contract_name_from_path(path), "Oracle");
    }
}
