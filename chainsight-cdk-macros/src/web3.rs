use proc_macro::TokenStream;
use quote::quote;

pub fn define_transform_for_web3() -> TokenStream {
    let output = quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn transform(response: ic_cdk::api::management_canister::http_request::TransformArgs) -> ic_cdk::api::management_canister::http_request::HttpResponse {
            let res = response.response;
            ic_cdk::api::management_canister::http_request::HttpResponse {
                status: res.status,
                headers: Vec::default(),
                body: res.body,
            }
        }
    };
    output.into()
}

pub fn define_web3_ctx() -> TokenStream {
    let output = quote! {
        #[derive(Default, Clone, Debug, PartialEq, candid::CandidType, candid::Deserialize)]
        pub struct Web3CtxParam {
            pub url: String,
            pub from: Option<String>,
            pub chain_id: u64,
            pub key: chainsight_cdk::types::EcdsaKeyEnvs,
        }
        manage_single_state!("web3_ctx_param", Web3CtxParam, false);

        pub fn web3_ctx() -> Result<ic_solidity_bindgen::Web3Context, ic_web3::Error> {
            let param = get_web3_ctx_param();
            let from = match param.from {
                Some(from) => Address::from_str(&from).unwrap(),
                None => Address::from_low_u64_be(0),
            };
            ic_solidity_bindgen::Web3Context::new(
                &param.url,
                from,
                param.chain_id,
                param.key.to_key_name(),
            )
        }
    };
    output.into()
}

pub fn define_get_ethereum_address() -> TokenStream {
    let output = quote! {
        fn default_derivation_key() -> Vec<u8> {
            ic_cdk::id().as_slice().to_vec()
        }
        async fn public_key(key_name: String) -> Result<Vec<u8>, String> {
            ic_web3::ic::get_public_key(None, vec![default_derivation_key()], key_name).await
        }
        async fn ethereum_address(key_name: String) -> Result<Address, String> {
            let pub_key = public_key(key_name).await?;
            ic_web3::ic::pubkey_to_address(&pub_key)
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn get_ethereum_address(key: chainsight_cdk::types::EcdsaKeyEnvs) -> String {
            match ethereum_address(key.to_key_name()).await {
                Ok(v) => format!("0x{}", hex::encode(v)),
                Err(msg) => msg,
            }
        }
    };
    output.into()
}