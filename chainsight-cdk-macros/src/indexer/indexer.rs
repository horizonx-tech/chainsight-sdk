use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Result, Token,
};

pub struct Args {
    contract_name: String,
    event_name: String,
    args: Punctuated<NamedField, Token![,]>,
    address: String,
}
struct NamedField {
    name: Ident,
    _colon_token: Token![:],
    ty: syn::Type,
}
impl Parse for NamedField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(NamedField {
            name: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}
impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let contract_name: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let event_name: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let args: Punctuated<NamedField, Token![,]> = Punctuated::parse_terminated(input)?;
        input.parse::<syn::Token![,]>()?;
        let address: syn::LitStr = input.parse()?;
        Ok(Args {
            contract_name: contract_name.value(),
            event_name: event_name.value(),
            args,
            address: address.value(),
        })
    }
}

pub fn gen(input: TokenStream) -> TokenStream {
    let Args {
        contract_name,
        event_name,
        args,
        address,
    } = syn::parse_macro_input!(input as Args);
    let struct_name = Ident::new(&format!("{}Event", event_name), event_name.span());

    quote! {}.into()
}
