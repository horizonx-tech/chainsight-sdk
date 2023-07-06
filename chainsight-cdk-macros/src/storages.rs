use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr, Type, LitBool, Expr, parse::Parse, LitInt};
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
