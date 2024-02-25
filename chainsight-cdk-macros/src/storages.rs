use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, parse_quote, DeriveInput, Expr, LitBool, LitInt, LitStr, Type,
};

use crate::internal::{attrs_query_func, attrs_update_func, gen_func_quote_to_call_proxy};

pub fn prepare_stable_structure(_input: TokenStream) -> TokenStream {
    prepare_stable_structure_internal().into()
}
fn prepare_stable_structure_internal() -> proc_macro2::TokenStream {
    quote! {
        use ic_stable_structures::Memory;

        type MemoryType = ic_stable_structures::memory_manager::VirtualMemory<ic_stable_structures::DefaultMemoryImpl>;

        const MEMORY_ID_FOR_UPGRADE: ic_stable_structures::memory_manager::MemoryId = ic_stable_structures::memory_manager::MemoryId::new(0);

        thread_local! {
            static MEMORY_MANAGER: std::cell::RefCell<ic_stable_structures::memory_manager::MemoryManager<ic_stable_structures::DefaultMemoryImpl>> =
                std::cell::RefCell::new(
                    ic_stable_structures::memory_manager::MemoryManager::init(ic_stable_structures::DefaultMemoryImpl::default()
                )
            );
        }

        fn get_upgrades_memory() -> MemoryType {
            MEMORY_MANAGER.with(|m| m.borrow().get(MEMORY_ID_FOR_UPGRADE))
        }
    }
}

#[derive(FromDeriveInput, Default)]
#[darling(
    default,
    attributes(stable_mem_storable_opts),
    forward_attrs(allow, doc, cfg)
)]
struct StableMemoryStorableOpts {
    max_size: Option<u32>,
    is_fixed_size: Option<bool>,
}
pub fn derive_storable_in_stable_memory(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = StableMemoryStorableOpts::from_derive_input(&input).unwrap();

    let struct_name = &input.ident;

    let storable_impl = quote! {
        impl ic_stable_structures::Storable for #struct_name {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(Encode!(self).unwrap())
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                Decode!(bytes.as_ref(), Self).unwrap()
            }
        }
    };

    let max_size = opts.max_size.unwrap_or(100000);
    let is_fixed_size = opts.is_fixed_size.unwrap_or(false);

    let bounded_storable_impl = quote! {
        impl ic_stable_structures::BoundedStorable for #struct_name {
            const MAX_SIZE: u32 = #max_size;
            const IS_FIXED_SIZE: bool = #is_fixed_size;
        }
    };

    let output = quote! {
        #storable_impl
        #bounded_storable_impl
    };

    output.into()
}

struct StableMemoryForScalarInput {
    name: LitStr,
    ty: Type,
    memory_id: u8,
    is_expose_getter: LitBool,
    init: Option<Expr>,
}
impl Parse for StableMemoryForScalarInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let ty: Type = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let lit_memory_id: LitInt = input.parse()?;
        let memory_id = lit_memory_id.base10_parse::<u8>().unwrap();
        input.parse::<syn::Token![,]>()?;
        let is_expose_getter: LitBool = input.parse()?;
        let init = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(StableMemoryForScalarInput {
            name,
            ty,
            memory_id,
            is_expose_getter,
            init,
        })
    }
}
pub fn stable_memory_for_scalar(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as StableMemoryForScalarInput);
    stable_memory_for_scalar_internal(args).into()
}
fn stable_memory_for_scalar_internal(args: StableMemoryForScalarInput) -> proc_macro2::TokenStream {
    let StableMemoryForScalarInput {
        name,
        ty,
        memory_id,
        is_expose_getter,
        init,
    } = args;

    let var_ident = syn::Ident::new(&name.value().to_uppercase(), name.span());
    let get_ident = syn::Ident::new(&format!("get_{}", name.value()), name.span());
    let set_ident = syn::Ident::new(&format!("set_{}", name.value()), name.span());
    let set_internal_ident =
        syn::Ident::new(&format!("set_{}_internal", name.value()), name.span());

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
            static #var_ident: std::cell::RefCell<ic_stable_structures::StableCell<#ty, MemoryType>> = std::cell::RefCell::new(
                ic_stable_structures::StableCell::init(
                    MEMORY_MANAGER.with(|mm| mm.borrow().get(
                        ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                    )),
                    #init
                ).unwrap()
            );
        }

        #getter_derives
        pub fn #get_ident() -> #ty {
            #var_ident.with(|cell| cell.borrow().get().clone())
        }

        // NOTE: consistency with macro return value for heap (not return Result)
        pub fn #set_ident(value: #ty) {
            #set_internal_ident(value).unwrap()
        }

        pub fn #set_internal_ident(value: #ty) -> Result<(), String> {
            let res = #var_ident.with(|cell| cell.borrow_mut().set(value));
            res.map(|_| ()).map_err(|e| format!("{:?}", e))
        }
    }
}

fn mem_id(input: DeriveInput) -> u8 {
    let memory_id = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("memory_id") {
                attr.parse_args::<LitInt>().ok()
            } else {
                None
            }
        })
        .map(|lit| lit.base10_parse::<u8>().unwrap())
        .expect("memory_id is required. e.g. #[memory_id(1)]");
    assert!(memory_id < 6, "memory_id must be less than 6");
    memory_id
}
pub fn key_values_store_derive(input: TokenStream) -> TokenStream {
    key_values_store_derive_internal(syn::parse_macro_input!(input as syn::DeriveInput)).into()
}
fn key_values_store_derive_internal(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = input.clone().ident;
    let memory_id = mem_id(input);
    let query = attrs_query_func();
    let update = attrs_update_func();
    let getter = syn::Ident::new(
        &format!("get_{}", name.to_string().to_lowercase()),
        name.span(),
    );
    let _getter_str = format!("_get_{}", name.to_string().to_lowercase());
    let _getter = syn::Ident::new(&_getter_str, name.span());
    let proxy_getter_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_{}", name.to_string().to_lowercase()),
        parse_quote! { Vec<#name> },
        Some(parse_quote! { String }),
        &_getter_str,
    );
    let between = syn::Ident::new(
        &format!("between_{}", name.to_string().to_lowercase()),
        name.span(),
    );
    let _between_str = format!("_between_{}", name.to_string().to_lowercase());
    let _between = syn::Ident::new(&_between_str, name.span());
    let proxy_between_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_between_{}", name.to_string().to_lowercase()),
        parse_quote! {  HashMap<String, Vec<#name>> },
        Some(parse_quote! { (String, String) }),
        &_between_str,
    );
    let last = syn::Ident::new(
        &format!("last_{}", name.to_string().to_lowercase()),
        name.span(),
    );
    let _last_str = format!("_last_{}", name.to_string().to_lowercase());
    let _last = syn::Ident::new(&_last_str, name.span());
    let proxy_last_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_last_{}", name.to_string().to_lowercase()),
        parse_quote! {  HashMap<String, Vec<#name>> },
        Some(parse_quote! { u64 }),
        &_last_str,
    );

    quote! {
        #query
        fn #getter(id: String) -> Vec<#name> {
            #_getter(id)
        }
        fn #_getter(id: String) -> Vec<#name> {
            #name::get(id.as_str())
        }
        #update
        #proxy_getter_quote

        #query
        fn #between(a: (String, String)) -> HashMap<String, Vec<#name>> {
            #_between(a)
        }
        fn #_between(a: (String, String)) -> HashMap<String, Vec<#name>> {
            #name::between(a.0.as_str(), a.1.as_str())
        }

        #update
        #proxy_between_quote
        #query
        fn #last(n: u64) -> HashMap<String, Vec<#name>> {
            #_last(n)
        }
        fn #_last(n: u64) -> HashMap<String, Vec<#name>> {
            #name::last(n)
        }
        #update
        #proxy_last_quote

        impl #name {

            pub fn get(id: &str) -> Vec<Self> {
                Self::get_store().get(id)
            }

            pub fn put(id: &str, e: Vec<Self>) {
                Self::get_store().set(id, e)
            }
            pub fn between(from:&str, to: &str) -> HashMap<String, Vec<Self>> {
                Self::get_store().between(from, to)
            }
            pub fn last(n: u64) -> HashMap<String, Vec<Self>> {
                Self::get_store().last_elems(n)
            }
            fn get_store() -> chainsight_cdk::storage::KeyValuesStore {
                chainsight_cdk::storage::KeyValuesStore::new(#memory_id)
            }
        }
    }
}

pub fn key_value_store_derive(input: TokenStream) -> TokenStream {
    key_value_store_derive_internal(syn::parse_macro_input!(input as syn::DeriveInput)).into()
}
pub fn key_value_store_derive_internal(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = input.clone().ident;
    let memory_id = mem_id(input);
    let query = attrs_query_func();
    let update = attrs_update_func();
    let getter = syn::Ident::new(
        &format!("get_{}", name.to_string().to_lowercase()),
        name.span(),
    );
    let _getter_str = format!("_get_{}", name.to_string().to_lowercase());
    let _getter = syn::Ident::new(&_getter_str, name.span());
    let proxy_getter_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_{}", name.to_string().to_lowercase()),
        parse_quote! { Option<#name> },
        Some(parse_quote! { String }),
        &_getter_str,
    );
    let between = syn::Ident::new(
        &format!("between_{}", name.to_string().to_lowercase()),
        name.span(),
    );
    let _between_str = format!("_between_{}", name.to_string().to_lowercase());
    let _between = syn::Ident::new(&_between_str, name.span());
    let proxy_between_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_between_{}", name.to_string().to_lowercase()),
        parse_quote! {  Vec<(String, #name)> },
        Some(parse_quote! { (String, String) }),
        &_between_str,
    );
    let last = syn::Ident::new(
        &format!("last_{}", name.to_string().to_lowercase()),
        name.span(),
    );
    let _last_str = format!("_last_{}", name.to_string().to_lowercase());
    let _last = syn::Ident::new(&_last_str, name.span());
    let proxy_last_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_last_{}", name.to_string().to_lowercase()),
        parse_quote! {  Vec<(String, #name)> },
        Some(parse_quote! { u64 }),
        &_last_str,
    );

    quote! {

        #query
        fn #getter(id: String) -> Option<#name> {
            #_getter(id)
        }
        fn #_getter(id: String) -> Option<#name> {
            #name::get(id.as_str())
        }
        #update
        #proxy_getter_quote

        #query
        fn #between(a:(String, String)) -> Vec<(String, #name)> {
            #_between(a)
        }
        fn #_between(a:(String, String)) -> Vec<(String, #name)> {
            #name::between(a.0.as_str(), a.1.as_str())
        }
        #update
        #proxy_between_quote

        #query
        fn #last(n: u64) -> Vec<(String, #name)> {
            #_last(n)
        }
        fn #_last(n: u64) -> Vec<(String, #name)> {
            #name::last(n)
        }
        #update
        #proxy_last_quote

        impl #name {
            pub fn get(id: &str) -> Option<Self> {
                Self::get_store().get(id)
            }
            pub fn put(&self, id: &str) {
                Self::get_store().set(id, self.clone())
            }
            pub fn between(from:&str, to: &str) -> Vec<(String, Self)> {
                Self::get_store().between(from, to)
            }
            pub fn last(n: u64) -> Vec<(String, Self)> {
                Self::get_store().last(n)
            }
            fn get_store() -> chainsight_cdk::storage::KeyValueStore {
                chainsight_cdk::storage::KeyValueStore::new(#memory_id)
            }
        }
    }
}

struct StableMemoryForVecInput {
    name: LitStr,
    ty: Type,
    memory_id: u8,
    is_expose_getter: LitBool,
}
impl syn::parse::Parse for StableMemoryForVecInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let lit_memory_id: LitInt = input.parse()?;
        let memory_id = lit_memory_id.base10_parse::<u8>().unwrap();
        input.parse::<syn::Token![,]>()?;
        let is_expose_getter: LitBool = input.parse()?;
        Ok(StableMemoryForVecInput {
            name,
            ty,
            memory_id,
            is_expose_getter,
        })
    }
}
pub fn stable_memory_for_vec(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as StableMemoryForVecInput);
    stable_memory_for_vec_internal(args).into()
}
fn stable_memory_for_vec_internal(args: StableMemoryForVecInput) -> proc_macro2::TokenStream {
    let StableMemoryForVecInput {
        name,
        ty,
        memory_id,
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
        parse_quote! { u64 },
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
        Some(parse_quote! { u64 }),
        &_get_top_elems_func_str,
    );
    let get_elem_func = syn::Ident::new(&format!("get_{}", state_name), name.span());
    let _get_elem_func_str = format!("_get_{}", state_name);
    let _get_elem_func = syn::Ident::new(&_get_elem_func_str, name.span());
    let proxy_get_elem_func_quote = gen_func_quote_to_call_proxy(
        &format!("proxy_get_{}", state_name),
        parse_quote! { #ty },
        Some(parse_quote! { u64 }),
        &_get_elem_func_str,
    );
    let add_elem_func = syn::Ident::new(&format!("add_{}", state_name), name.span());
    let add_elem_func_internal =
        syn::Ident::new(&format!("add_{}_internal", state_name), name.span());

    let getter_derives = if is_expose_getter.value {
        attrs_query_func()
    } else {
        quote! {}
    };
    let update_derives = attrs_update_func();

    quote! {
        thread_local! {
            static #state_upper_name: std::cell::RefCell<ic_stable_structures::StableVec<#ty, MemoryType>> = std::cell::RefCell::new(
                ic_stable_structures::StableVec::init(
                    MEMORY_MANAGER.with(|mm| mm.borrow().get(
                        ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                    ))
                ).unwrap()
            );
        }

        #getter_derives
        fn #get_vec_func() -> Vec<#ty> {
            #_get_vec_func()
        }

        pub fn #_get_vec_func() -> Vec<#ty> {
            #state_upper_name.with(|mem| mem.borrow().iter().collect())
        }

        #update_derives
        #proxy_get_vec_func_quote

        #getter_derives
        fn #get_len_func() -> u64 {
            #_get_len_func()
        }

        pub fn #_get_len_func() -> u64 {
            #state_upper_name.with(|mem| mem.borrow().len())
        }

        #update_derives
        #proxy_get_len_func_quote

        #getter_derives
        fn #get_last_elem_func() -> #ty {
           #_get_last_elem_func()
        }

        pub fn #_get_last_elem_func() -> #ty {
            #state_upper_name.with(|mem| {
                let borrowed_mem = mem.borrow();
                let len = borrowed_mem.len();
                borrowed_mem.get(len - 1) // NOTE: Since StableVec does not have last()
            }).unwrap() // temp: unwrap to not return opt
        }

        #update_derives
        #proxy_get_last_elem_func_quote

        #getter_derives
        pub fn #get_top_elems_func(n: u64) -> Vec<#ty> {
            #_get_top_elems_func(n)
        }

        pub fn #_get_top_elems_func(n: u64) -> Vec<#ty> {
            #state_upper_name.with(|mem| {
                let borrowed_mem = mem.borrow();
                let len = borrowed_mem.len();
                let mut res = Vec::new();
                for i in 0..n {
                    if i >= len {
                        break;
                    }
                    res.push(borrowed_mem.get(len - i - 1).unwrap());
                }
                res
            })
        }

        #update_derives
        #proxy_get_top_elems_func_quote

        #getter_derives
        fn #get_elem_func(idx: u64) -> #ty {
            #_get_elem_func(idx)
        }

        pub fn #_get_elem_func(idx: u64) -> #ty {
            #state_upper_name.with(|mem| mem.borrow().get(idx)).unwrap() // temp: unwrap to not return opt
        }

        #update_derives
        #proxy_get_elem_func_quote

        // NOTE: consistency with macro return value for heap (not return Result)
        pub fn #add_elem_func(value: #ty) {
            #add_elem_func_internal(value).unwrap()
        }

        pub fn #add_elem_func_internal(value: #ty) -> Result<(), String> {
            let res = #state_upper_name.with(|vec| vec.borrow_mut().push(&value));
            res.map_err(|e| format!("{:?}", e))
        }
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_prepare_stable_structure() {
        let generated = prepare_stable_structure_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__prepare_stable_structure", formatted);
    }

    #[test]
    fn test_snapshot_stable_memory_for_scalar() {
        let input = quote! {"timestamp", u64, 0, true};
        let args: syn::Result<StableMemoryForScalarInput> = syn::parse2(input);
        let generated = stable_memory_for_scalar_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__stable_memory_for_scalar", formatted);
    }

    #[test]
    fn test_snapshot_stable_memory_for_vec() {
        let input = quote! {"timestamp", u64, 0, true};
        let args: syn::Result<StableMemoryForVecInput> = syn::parse2(input);
        let generated = stable_memory_for_vec_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__stable_memory_for_vec", formatted);
    }

    #[test]
    fn test_snapshot_key_value_store_derive() {
        let input = quote! {
            #[memory_id(1)]
            struct Account {
                pub id: String,
                pub token: String,
                pub balance: u64,
            }
        };
        let input: syn::DeriveInput = syn::parse2(input).unwrap();
        let generated = key_value_store_derive_internal(input);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__key_value_store_derive", formatted);
    }

    #[test]
    fn test_snapshot_key_values_store_derive() {
        let input = quote! {
            #[memory_id(1)]
            struct Account {
                pub id: String,
                pub token: String,
                pub balance: u64,
            }
        };
        let input: syn::DeriveInput = syn::parse2(input).unwrap();
        let generated = key_values_store_derive_internal(input);
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__key_values_store_derive", formatted);
    }
}
