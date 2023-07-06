use chainsight_cdk::storage::Data;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, Expr, LitBool, LitStr, Type};

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
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        field_names.push(field_name);
        field_types.push(field_type.clone());
        // if field is String, use to_string() to convert
        let untokenize_function = match field_type.to_token_stream().to_string().as_str() {
            "String" => quote! {to_string() },
            "u128" => quote! { to_u128().unwrap() },
            "u64" => quote! { to_u64().unwrap() },
            "u32" => quote! { to_u32().unwrap() },
            "u16" => quote! { to_u16().unwrap() },
            "u8" => quote! { to_u8().unwrap() },
            "usize" => quote! { to_usize().unwrap() },
            "i128" => quote! { to_i128().unwrap() },
            "i64" => quote! { to_i64().unwrap() },
            "i16" => quote! { to_i16().unwrap() },
            "i8" => quote! { to_i8().unwrap() },
            _ => quote! { to_string() },
        };
        untokenize_functions.push(untokenize_function);
    }
    let expanded = quote! {
        impl #name {
            fn untokenize(data: Data) -> Self {
                #name {
                    #( #field_names: data.get(stringify!(#field_names)).unwrap().#untokenize_functions ),*
                }
            }

            fn tokenize(&self) -> Data {
                let mut data = HashMap<std::string::String, chainsight_cdk::storage::Token> = HashMap::new();
                #( data.insert(stringify!(#field_names).to_string(), chainsight_cdk::storage::Token::from(self.#field_names.clone())); )*
                data
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
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
    let SingleStateInput {
        name,
        ty,
        is_expose_getter,
        init,
    } = parse_macro_input!(input as SingleStateInput);

    let var_ident = syn::Ident::new(&name.value().to_uppercase(), name.span());
    let get_ident = syn::Ident::new(&format!("get_{}", name.value()), name.span());
    let set_ident = syn::Ident::new(&format!("set_{}", name.value()), name.span());

    let init = match init {
        Some(init_value) => quote!(#init_value),
        None => quote!(std::default::Default::default()),
    };
    let getter_derives = if is_expose_getter.value {
        quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
        }
    } else {
        quote! {}
    };

    let output = quote! {
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
    };
    output.into()
}

struct VecStateInput {
    name: LitStr,
    ty: Type,
    is_expose_getter: LitBool,
}
impl syn::parse::Parse for VecStateInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
    let VecStateInput {
        name,
        ty,
        is_expose_getter,
    } = parse_macro_input!(input as VecStateInput);

    let state_name = name.value();
    let state_upper_name = syn::Ident::new(&format!("{}S", state_name.to_uppercase()), name.span());
    let get_vec_func = syn::Ident::new(&format!("get_{}s", state_name), name.span());
    let get_len_func = syn::Ident::new(&format!("{}s_len", state_name), name.span());
    let get_last_elem_func = syn::Ident::new(&format!("get_last_{}", state_name), name.span());
    let get_top_elems_func = syn::Ident::new(&format!("get_top_{}s", state_name), name.span());
    let get_elem_func = syn::Ident::new(&format!("get_{}", state_name), name.span());
    let add_elem_func = syn::Ident::new(&format!("add_{}", state_name), name.span());

    let getter_derives = if is_expose_getter.value {
        quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        thread_local! {
            static #state_upper_name: std::cell::RefCell<Vec<#ty>> = std::cell::RefCell::new(Vec::new());
        }

        #getter_derives
        pub fn #get_vec_func() -> Vec<#ty> {
            #state_upper_name.with(|state| state.borrow().clone())
        }

        #getter_derives
        pub fn #get_len_func() -> usize {
            #state_upper_name.with(|state| state.borrow().len())
        }

        #getter_derives
        pub fn #get_last_elem_func() -> #ty {
            #state_upper_name.with(|state| state.borrow().last().unwrap().clone())
        }

        #getter_derives
        pub fn #get_top_elems_func(n: usize) -> Vec<#ty> {
            #state_upper_name.with(|state| state.borrow().iter().rev().take(n).cloned().collect::<Vec<_>>())
        }

        #getter_derives
        pub fn #get_elem_func(idx: usize) -> #ty {
            #state_upper_name.with(|state| state.borrow()[idx].clone())
        }

        pub fn #add_elem_func(value: #ty) {
            #state_upper_name.with(|state| state.borrow_mut().push(value));
        }
    };

    TokenStream::from(expanded)
}

struct MapStateInput {
    name: LitStr,
    key_ty: Type,
    val_ty: Type,
    is_expose_getter: LitBool,
}
impl syn::parse::Parse for MapStateInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
    let MapStateInput {
        name,
        key_ty,
        val_ty,
        is_expose_getter,
    } = parse_macro_input!(input as MapStateInput);

    let state_name = name.value();
    let state_upper_name = syn::Ident::new(&format!("{}S", state_name.to_uppercase()), name.span());
    let len_func = syn::Ident::new(&format!("{}s_len", state_name), name.span());
    let get_elem_func = syn::Ident::new(&format!("get_{}", state_name), name.span());
    let insert_elem_func = syn::Ident::new(&format!("insert_{}", state_name), name.span());

    let getter_derives = if is_expose_getter.value {
        quote! {
            #[ic_cdk::query]
            #[candid::candid_method(query)]
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
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
    };

    TokenStream::from(expanded)
}
