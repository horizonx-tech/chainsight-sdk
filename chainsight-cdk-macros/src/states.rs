use chainsight_cdk::storage::Data;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Expr, LitBool, LitStr, Type,
};

use crate::internal::{attrs_query_func, attrs_update_func, gen_func_quote_to_call_proxy};

pub trait Persist {
    fn untokenize(data: Data) -> Self;
    fn tokenize(&self) -> Data;
}

pub fn persist_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
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
    let mut untokenize_functions = Vec::new();
    // let mut tokenize_functions = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        field_names.push(field_name);
        field_types.push(field_type);
        let untokenize_function = match field_type.to_token_stream().to_string().as_str() {
            "String" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_string(),
            },
            "u128" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_u128().unwrap(),
            },
            "u64" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_u64().unwrap(),
            },
            "u32" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_u32().unwrap(),
            },
            "u16" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_u16().unwrap(),
            },
            "u8" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_u8().unwrap(),
            },
            "i128" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_i128().unwrap(),
            },
            "i64" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_i64().unwrap(),
            },
            "i32" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_i32().unwrap(),
            },
            "i16" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_i16().unwrap(),
            },
            "i8" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_i8().unwrap(),
            },
            "bool" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_bool().unwrap(),
            },
            "Vec<u8>" => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_vec().unwrap(),
            },
            "chainsight_cdk::core::U256" => quote! {
               #field_name: data.get(stringify!(#field_name)).unwrap().to_u256().unwrap(),
            },
            "U256" => quote! {
               #field_name: data.get(stringify!(#field_name)).unwrap().to_u256().unwrap(),
            },
            _ => quote! {
                #field_name: data.get(stringify!(#field_name)).unwrap().to_string(),
            },
        };
        untokenize_functions.push(untokenize_function);
    }
    let expanded = quote! {
        impl #name {
            pub fn _tokenize(&self) -> chainsight_cdk::storage::Data {
                let mut data: HashMap<std::string::String, chainsight_cdk::storage::Token> = HashMap::new();
                #(
                    data.insert(stringify!(#field_names).to_string(), chainsight_cdk::storage::Token::from(self.#field_names.clone()));
                )*
                Data::new(data)
            }
            pub fn _untokenize(data: Data) -> Self {
                Self {
                    #(#untokenize_functions)*
                }
            }
        }
        impl chainsight_cdk::storage::Persist for #name {
            fn untokenize(data: Data) -> Self {
                Self::_untokenize(data)
            }
            fn tokenize(&self) -> Data {
                self._tokenize()
            }
        }
    };

    expanded.into()
}

struct SingleStateInput {
    name: LitStr,
    ty: Type,
    is_expose_getter: LitBool,
    init: Option<Expr>,
}
impl Parse for SingleStateInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let is_expose_getter: LitBool = input.parse()?;
        let init = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(SingleStateInput {
            name,
            ty,
            is_expose_getter,
            init,
        })
    }
}
pub fn manage_single_state(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SingleStateInput);
    manage_single_state_internal(args).into()
}
fn manage_single_state_internal(args: SingleStateInput) -> proc_macro2::TokenStream {
    let SingleStateInput {
        name,
        ty,
        is_expose_getter,
        init,
    } = args;

    let var_ident = syn::Ident::new(&name.value().to_uppercase(), name.span());
    let get_ident = syn::Ident::new(&format!("get_{}", name.value()), name.span());
    let set_ident = syn::Ident::new(&format!("set_{}", name.value()), name.span());

    let init = match init {
        Some(init_value) => quote!(#init_value),
        None => quote!(std::default::Default::default()),
    };
    let getter_derives = if is_expose_getter.value {
        attrs_query_func()
    } else {
        quote! {}
    };

    quote! {
        thread_local! {
            static #var_ident: std::cell::RefCell<#ty> = std::cell::RefCell::new(#init);
        }

        #getter_derives
        pub fn #get_ident() -> #ty {
            #var_ident.with(|state| state.borrow().clone())
        }

        pub fn #set_ident(value: #ty) {
            #var_ident.with(|state| *state.borrow_mut() = value);
        }
    }
}

struct VecStateInput {
    name: LitStr,
    ty: Type,
    is_expose_getter: LitBool,
}
impl Parse for VecStateInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let is_expose_getter: LitBool = input.parse()?;
        Ok(VecStateInput {
            name,
            ty,
            is_expose_getter,
        })
    }
}
pub fn manage_vec_state(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as VecStateInput);
    manage_vec_state_internal(args).into()
}
fn manage_vec_state_internal(args: VecStateInput) -> proc_macro2::TokenStream {
    let VecStateInput {
        name,
        ty,
        is_expose_getter,
    } = args;

    let state_name = name.value();
    let state_upper_name = syn::Ident::new(&format!("{}S", state_name.to_uppercase()), name.span());
    let get_vec_func = syn::Ident::new(&format!("get_{}s", state_name), name.span());
    let _get_vec_func_str = format!("_get_{}s", state_name);
    let _get_vec_func = syn::Ident::new(&_get_vec_func_str, name.span());
    let proxy_get_vec_func_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_{}s", state_name),
        parse_quote! { Vec<#ty> },
        None,
        &_get_vec_func_str,
    );
    let get_len_func = syn::Ident::new(&format!("{}s_len", state_name), name.span());
    let _get_len_func_str = format!("_{}s_len", state_name);
    let _get_len_func = syn::Ident::new(&_get_len_func_str, name.span());
    let proxy_get_len_func_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_{}s_len", state_name),
        parse_quote! { usize },
        None,
        &_get_len_func_str,
    );
    let get_last_elem_func = syn::Ident::new(&format!("get_last_{}", state_name), name.span());
    let _get_last_elem_func_str = format!("_get_last_{}", state_name);
    let _get_last_elem_func = syn::Ident::new(&_get_last_elem_func_str, name.span());
    let proxy_get_last_elem_func_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_last_{}", state_name),
        parse_quote! { #ty },
        None,
        &_get_last_elem_func_str,
    );
    let get_top_elems_func = syn::Ident::new(&format!("get_top_{}s", state_name), name.span());
    let _get_top_elems_func_str = format!("_get_top_{}s", state_name);
    let _get_top_elems_func = syn::Ident::new(&_get_top_elems_func_str, name.span());
    let proxy_get_top_elems_func_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_top_{}s", state_name),
        parse_quote! { Vec<#ty> },
        Some(parse_quote! { usize }),
        &_get_top_elems_func_str,
    );
    let get_elem_func = syn::Ident::new(&format!("get_{}", state_name), name.span());
    let _get_elem_func_str = format!("_get_{}", state_name);
    let _get_elem_func = syn::Ident::new(&_get_elem_func_str, name.span());
    let proxy_get_elem_func_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_{}", state_name),
        parse_quote! { #ty },
        Some(parse_quote! { usize }),
        &_get_elem_func_str,
    );
    let add_elem_func = syn::Ident::new(&format!("add_{}", state_name), name.span());

    let getter_derives = if is_expose_getter.value {
        attrs_query_func()
    } else {
        quote! {}
    };
    let update_derive = attrs_update_func();

    quote! {
        thread_local! {
            static #state_upper_name: std::cell::RefCell<Vec<#ty>> = std::cell::RefCell::new(Vec::new());
        }

        #getter_derives
        fn #get_vec_func() -> Vec<#ty> {
            #_get_vec_func()
        }

        pub fn #_get_vec_func() -> Vec<#ty> {
            #state_upper_name.with(|state| state.borrow().clone())
        }

        #update_derive
        #proxy_get_vec_func_quote

        #getter_derives
        fn #get_len_func() -> usize {
            #_get_len_func()
        }

        pub fn #_get_len_func() -> usize {
            #state_upper_name.with(|state| state.borrow().len())
        }

        #update_derive
        #proxy_get_len_func_quote

        #getter_derives
        fn #get_last_elem_func() -> #ty {
            #_get_last_elem_func()
        }

        pub fn #_get_last_elem_func() -> #ty {
            #state_upper_name.with(|state| state.borrow().last().unwrap().clone())
        }

        #update_derive
        #proxy_get_last_elem_func_quote


        #getter_derives
        fn #get_top_elems_func(n: usize) -> Vec<#ty> {
            #_get_top_elems_func(n)
        }

        pub fn #_get_top_elems_func(n: usize) -> Vec<#ty> {
            #state_upper_name.with(|state| state.borrow().iter().rev().take(n).cloned().collect::<Vec<_>>())
        }

        #update_derive
        #proxy_get_top_elems_func_quote

        #getter_derives
        fn #get_elem_func(idx: usize) -> #ty {
            #_get_elem_func(idx)
        }

        pub fn #_get_elem_func(idx: usize) -> #ty {
            #state_upper_name.with(|state| state.borrow()[idx].clone())
        }

        #update_derive
        #proxy_get_elem_func_quote

        pub fn #add_elem_func(value: #ty) {
            #state_upper_name.with(|state| state.borrow_mut().push(value));
        }
    }
}

struct MapStateInput {
    name: LitStr,
    key_ty: Type,
    val_ty: Type,
    is_expose_getter: LitBool,
}
impl Parse for MapStateInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let key_ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let val_ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let is_expose_getter: LitBool = input.parse()?;
        Ok(MapStateInput {
            name,
            key_ty,
            val_ty,
            is_expose_getter,
        })
    }
}
pub fn manage_map_state(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MapStateInput);
    manage_map_state_internal(args).into()
}
// todo: support functions to call proxy
fn manage_map_state_internal(args: MapStateInput) -> proc_macro2::TokenStream {
    let MapStateInput {
        name,
        key_ty,
        val_ty,
        is_expose_getter,
    } = args;

    let state_name = name.value();
    let state_upper_name = syn::Ident::new(&format!("{}S", state_name.to_uppercase()), name.span());
    let len_func = syn::Ident::new(&format!("{}s_len", state_name), name.span());
    let get_elem_func = syn::Ident::new(&format!("get_{}", state_name), name.span());
    let insert_elem_func = syn::Ident::new(&format!("insert_{}", state_name), name.span());

    let getter_derives = if is_expose_getter.value {
        attrs_query_func()
    } else {
        quote! {}
    };

    quote! {
        thread_local! {
            static #state_upper_name: std::cell::RefCell<std::collections::HashMap<#key_ty, #val_ty>> = std::cell::RefCell::new(std::collections::HashMap::new());
        }

        #getter_derives
        pub fn #len_func() -> usize {
            #state_upper_name.with(|state| state.borrow().len())
        }

        #getter_derives
        pub fn #get_elem_func(key: #key_ty) -> #val_ty {
            #state_upper_name.with(|state| state.borrow().get(&key).cloned().expect("key not found"))
        }

        pub fn #insert_elem_func(key: #key_ty, value: #val_ty) {
            #state_upper_name.with(|state| state.borrow_mut().insert(key, value));
        }
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_manage_single_state() {
        let input = quote! {"timer_id", TimerId, true};
        let args: syn::Result<SingleStateInput> = syn::parse2(input);
        let generated = manage_single_state_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__manage_single_state", formatted);
    }

    #[test]
    fn test_snapshot_manage_vec_state() {
        let input = quote! {"snapshots", Snapshot, true};
        let args: syn::Result<VecStateInput> = syn::parse2(input);
        let generated = manage_vec_state_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__manage_vec_state", formatted);
    }

    #[test]
    fn test_snapshot_manage_map_state() {
        let input = quote! {"time_differences", String, u64, true};
        let args: syn::Result<MapStateInput> = syn::parse2(input);
        let generated = manage_map_state_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__manage_map_state", formatted);
    }
}
