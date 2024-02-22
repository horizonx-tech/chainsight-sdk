use quote::quote;

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
