use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr, Type, LitBool, Expr, parse::Parse, LitInt, DeriveInput};
use quote::quote;

pub fn prepare_stable_structure() -> TokenStream {
    let output = quote! {
        type Memory = ic_stable_structures::memory_manager::VirtualMemory<ic_stable_structures::DefaultMemoryImpl>;

        thread_local! {
            static MEMORY_MANAGER: std::cell::RefCell<ic_stable_structures::memory_manager::MemoryManager<ic_stable_structures::DefaultMemoryImpl>> =
                std::cell::RefCell::new(
                    ic_stable_structures::memory_manager::MemoryManager::init(ic_stable_structures::DefaultMemoryImpl::default()
                )
            );
        }
    };
    output.into()
}

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(stable_mem_storable_opts), forward_attrs(allow, doc, cfg))]
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
    let StableMemoryForScalarInput {
        name,
        ty,
        memory_id,
        is_expose_getter,
        init,
    } = parse_macro_input!(input as StableMemoryForScalarInput);

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
            static #var_ident: std::cell::RefCell<ic_stable_structures::StableCell<#ty, Memory>> = std::cell::RefCell::new(
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

        pub fn #set_ident(value: #ty) -> Result<(), String> {
            let res = #var_ident.with(|cell| cell.borrow_mut().set(value));
            res.map(|_| ()).map_err(|e| format!("{:?}", e))
        }
    };
    output.into()
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
    let StableMemoryForVecInput {
        name,
        ty,
        memory_id,
        is_expose_getter,
    } = parse_macro_input!(input as StableMemoryForVecInput);

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

    let output = quote! {
        thread_local! {
            static #state_upper_name: std::cell::RefCell<ic_stable_structures::StableVec<#ty, Memory>> = std::cell::RefCell::new(
                ic_stable_structures::StableVec::init(
                    MEMORY_MANAGER.with(|mm| mm.borrow().get(
                        ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                    ))
                ).unwrap()
            );
        }

        #getter_derives
        pub fn #get_vec_func() -> Vec<#ty> {
            #state_upper_name.with(|mem| mem.borrow().iter().collect())
        }

        #getter_derives
        pub fn #get_len_func() -> u64 {
            #state_upper_name.with(|mem| mem.borrow().len())
        }

        #getter_derives
        pub fn #get_last_elem_func() -> Option<#ty> {
            #state_upper_name.with(|mem| {
                let borrowed_mem = mem.borrow();
                let len = borrowed_mem.len();
                borrowed_mem.get(len - 1) // NOTE: Since StableVec does not have last()
            })
        }

        #getter_derives
        pub fn #get_top_elems_func(n: u64) -> Vec<#ty> {
            #state_upper_name.with(|mem| {
                let borrowed_mem = mem.borrow();
                let start_index = borrowed_mem.len().saturating_sub(n);
                let mut vec_var: Vec<#ty> = borrowed_mem.iter().collect(); // NOTE: Since StableVec does not have rev(), copy the entire StableVec to Vec
                vec_var.split_off(start_index as usize).iter().rev().cloned().collect()
            })
        }

        #getter_derives
        pub fn #get_elem_func(idx: u64) -> Option<#ty> {
            #state_upper_name.with(|mem| mem.borrow().get(idx))
        }

        pub fn #add_elem_func(value: #ty) -> Result<(), String> {
            let res = #state_upper_name.with(|vec| vec.borrow_mut().push(&value));
            res.map_err(|e| format!("{:?}", e))
        }
    };

    output.into()
}
