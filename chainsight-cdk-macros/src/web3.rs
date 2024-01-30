use chainsight_cdk::web3::ContractFunction;
use ethabi::Param;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    LitInt, Result,
};

pub trait ContractEvent {
    fn from(item: ic_solidity_bindgen::types::EventLog) -> Self;
}

#[derive(Clone)]
pub struct ContractCall {
    contract_function: ContractFunction,
}

impl ContractCall {
    pub fn new(contract_function: ContractFunction) -> Self {
        Self { contract_function }
    }

    pub fn function(&self) -> &ContractFunction {
        &self.contract_function
    }

    pub fn field_names(&self) -> Vec<String> {
        self.call_args().into_iter().map(|arg| arg.name).collect()
    }

    pub fn call_args(&self) -> Vec<Param> {
        self.contract_function.call_args()
    }
}

pub fn contract_event_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
    define_transform_for_web3_internal().into()
}
fn define_transform_for_web3_internal() -> proc_macro2::TokenStream {
    quote! {
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

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn transform_eip1559_support(response: ic_cdk::api::management_canister::http_request::TransformArgs) -> ic_cdk::api::management_canister::http_request::HttpResponse {
            use chainsight_cdk::web3::TransformProcessor;
            let processor = chainsight_cdk::web3::processors::EIP1559SupportProcessor;
            processor.transform(response)
        }

    }
}

struct DefineWeb3CtxArgs {
    stable_memory_id: Option<LitInt>,
}
impl Parse for DefineWeb3CtxArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let stable_memory_id = if !input.is_empty() {
            let parsed: LitInt = input.parse()?;
            Some(parsed)
        } else {
            None
        };
        Ok(DefineWeb3CtxArgs { stable_memory_id })
    }
}
pub fn define_web3_ctx(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as DefineWeb3CtxArgs);
    define_web3_ctx_internal(args).into()
}
fn define_web3_ctx_internal(input: DefineWeb3CtxArgs) -> proc_macro2::TokenStream {
    let storage_quote = if let Some(memory_id) = input.stable_memory_id {
        quote! {
            stable_memory_for_scalar!("web3_ctx_param", chainsight_cdk::web3::Web3CtxParam, #memory_id, false);
        }
    } else {
        quote! {
            manage_single_state!("web3_ctx_param", chainsight_cdk::web3::Web3CtxParam, false);
        }
    };

    quote! {
        #storage_quote

        pub fn web3_ctx() -> Result<ic_solidity_bindgen::Web3Context, ic_web3_rs::Error> {
            let param = get_web3_ctx_param();
            let from = match param.from {
                Some(from) => Address::from_str(&from).unwrap(),
                None => Address::from_low_u64_be(0),
            };
            ic_solidity_bindgen::Web3Context::new(
                &param.url,
                from,
                param.chain_id,
                param.env.ecdsa_key_name(),
            )
        }
    }
}

pub fn define_get_ethereum_address() -> TokenStream {
    define_get_ethereum_address_internal().into()
}
fn define_get_ethereum_address_internal() -> proc_macro2::TokenStream {
    quote! {
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
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_define_transform_for_web3() {
        let generated = define_transform_for_web3_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__define_transform_for_web3", formatted);
    }

    #[test]
    fn test_snapshot_define_web3_ctx() {
        let input = quote! {};
        let args: syn::Result<DefineWeb3CtxArgs> = syn::parse2(input);
        let generated = define_web3_ctx_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__define_web3_ctx", formatted);
    }

    #[test]
    fn test_snapshot_define_web3_ctx_with_stable_memory() {
        let input = quote! {1};
        let args: syn::Result<DefineWeb3CtxArgs> = syn::parse2(input);
        let generated = define_web3_ctx_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__define_web3_ctx__with_stable_memory", formatted);
    }

    #[test]
    fn define_get_ethereum_address() {
        let generated = define_get_ethereum_address_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__define_get_ethereum_address", formatted);
    }
}
