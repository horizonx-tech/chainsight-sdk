use proc_macro::TokenStream;
use syn::{parse::Parse, Type, LitStr, parse_macro_input, Expr, LitBool};
use quote::{quote};

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
        Ok(SingleStateInput { name, ty, is_expose_getter, init })
    }
}
pub fn manage_single_state(input: TokenStream) -> TokenStream {
    let SingleStateInput { name, ty, is_expose_getter, init } = parse_macro_input!(input as SingleStateInput);

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
        Ok(VecStateInput { name, ty, is_expose_getter })
    }
}
pub fn manage_vec_state(input: TokenStream) -> TokenStream {
    let VecStateInput { name, ty, is_expose_getter } = parse_macro_input!(input as VecStateInput);
    
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
        Ok(MapStateInput { name, key_ty, val_ty, is_expose_getter })
    }
}
pub fn manage_map_state(input: TokenStream) -> TokenStream {
    let MapStateInput { name, key_ty, val_ty, is_expose_getter } = parse_macro_input!(input as MapStateInput);

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