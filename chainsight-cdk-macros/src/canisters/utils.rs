use quote::quote;
use std::path::PathBuf;

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
    let query_derives = quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
    };
    let update_derives = quote! {
        #[ic_cdk::update]
        #[candid::candid_method(update)]
    };

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_contract_name_from_path() {
        let path = "__interfaces/Oracle.json";
        assert_eq!(extract_contract_name_from_path(path), "Oracle");
    }
}
