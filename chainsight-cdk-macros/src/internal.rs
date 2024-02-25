use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Type};

pub fn attrs_query_func() -> proc_macro2::TokenStream {
    quote! {
        #[ic_cdk::query]
        #[candid::candid_method(query)]
    }
}

pub fn attrs_update_func() -> proc_macro2::TokenStream {
    quote! {
        #[ic_cdk::update]
        #[candid::candid_method(update)]
    }
}

pub fn gen_func_quote_to_call_proxy(
    func_name: &str,
    response_type: Type,
    args_type: Option<Type>,
    func_name_to_call: &str,
) -> proc_macro2::TokenStream {
    let func_name_ident = Ident::new(func_name, Span::call_site());
    let func_name_to_call_ident = Ident::new(func_name_to_call, Span::call_site());

    let receiver_provider_quote = if let Some(args_type) = args_type {
        quote! { ReceiverProvider::<#args_type, #response_type> }
    } else {
        quote! { ReceiverProviderWithoutArgs::<#response_type> }
    };

    quote! {
        async fn #func_name_ident(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            chainsight_cdk::rpc::#receiver_provider_quote::new(
                proxy(),
                #func_name_to_call_ident
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
    fn test_gen_func_quote_to_call_proxy() {
        let actual = gen_func_quote_to_call_proxy(
            "proxy_get_last_timestamp",
            syn::parse_quote! { u64 },
            None,
            "_get_last_timestamp",
        );
        let expected = quote! {
            async fn proxy_get_last_timestamp(input: std::vec::Vec<u8>) -> std::vec::Vec<u8> {
                use chainsight_cdk::rpc::Receiver;
                chainsight_cdk::rpc::ReceiverProviderWithoutArgs::<u64>::new(proxy(), _get_last_timestamp)
                    .reply(input)
                    .await
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
