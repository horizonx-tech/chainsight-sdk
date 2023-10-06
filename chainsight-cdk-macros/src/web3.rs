use proc_macro::TokenStream;
use quote::{quote, ToTokens};

pub trait ContractEvent {
    fn from(item: ic_solidity_bindgen::types::EventLog) -> Self;
}

pub fn contract_event_derive(input: TokenStream) -> TokenStream {
    // get struct body
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    // get struct name
    let name = input.ident;
    // get struct fields
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => panic!("Only support struct"),
    };
    // get field name and type
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut token_to_field_functions = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        field_names.push(field_name);
        field_types.push(field_type.clone());
        // if field is String, use to_string() to convert
        let func = match field_type.to_token_stream().to_string().as_str() {
            "String" => quote! { to_string() },
            "Vec<u8>" => quote! { into_arary().unwrap() },
            "u128" => quote! { into_uint().unwrap().as_u128() },
            "u64" => quote! { into_uint().unwrap().as_u64() },
            "u32" => quote! { into_uint().unwrap().as_u32() },
            "u16" => quote! { into_uint().unwrap().as_u16() },
            "u8" => quote! { into_uint().unwrap().as_u8() },
            "bool" => quote! { into_bool().unwrap() },
            "U256" => quote! { into_uint().unwrap().into() },
            _ => quote! {},
        };

        token_to_field_functions.push(func);
    }

    let gen = quote! {
    impl From<ic_solidity_bindgen::types::EventLog> for #name {
            fn from(item: ic_solidity_bindgen::types::EventLog) -> Self {
                let mut event = #name::default();
                let mut params = item.event.params.iter();
                #(
                    let token = params.clone().find(|p| p.name == stringify!(#field_names)).unwrap().clone().value;
                    // match type of field_name
                    event.#field_names = token.#token_to_field_functions;

                 )*
                event
            }
        }
    };
    gen.into()
}

pub fn define_transform_for_web3() -> TokenStream {
    let output = quote! {
        use ic_web3_rs::transforms::transform::TransformProcessor;
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

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn transform_send_transaction(response: ic_cdk::api::management_canister::http_request::TransformArgs) -> ic_cdk::api::management_canister::http_request::HttpResponse {
            ic_web3_rs::transforms::processors::send_transaction_processor().transform(response)
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn transform_get_filter_changes(response: ic_cdk::api::management_canister::http_request::TransformArgs) -> ic_cdk::api::management_canister::http_request::HttpResponse {
            ic_web3_rs::transforms::processors::get_filter_changes_processor().transform(response)
        }
    };
    output.into()
}

pub fn define_web3_ctx() -> TokenStream {
    let output = quote! {

        manage_single_state!("web3_ctx_param", chainsight_cdk::web3::Web3CtxParam, false);
            use ic_web3_rs::{
                futures::executor::block_on,
            };
            pub fn web3_ctx() -> Result<ic_solidity_bindgen::Web3Context, ic_web3_rs::Error> {
            let param = get_web3_ctx_param();
            let from = match param.from {
                Some(from) => Address::from_str(&from).unwrap(),
                None => {
                    block_on(ethereum_address(get_env().ecdsa_key_name())).unwrap()
                },
            };
            ic_solidity_bindgen::Web3Context::new(
                &param.url,
                from,
                param.chain_id,
                param.env.ecdsa_key_name(),
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
            ic_web3_rs::ic::get_public_key(None, vec![default_derivation_key()], key_name).await
        }
        async fn ethereum_address(key_name: String) -> Result<Address, String> {
            let pub_key = public_key(key_name).await?;
            ic_web3_rs::ic::pubkey_to_address(&pub_key)
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn get_ethereum_address() -> String {
            match ethereum_address(get_env().ecdsa_key_name()).await {
                Ok(v) => format!("0x{}", hex::encode(v)),
                Err(msg) => msg,
            }
        }
    };
    output.into()
}
