use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{braced, punctuated::Punctuated, Ident, Result, Token, Type};

struct SetupArgs {
    fields: Punctuated<NamedField, Token![,]>,
}
impl Parse for SetupArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let fields = Punctuated::parse_terminated(&content)?;
        Ok(SetupArgs { fields })
    }
}
struct NamedField {
    name: Ident,
    _colon_token: Token![:],
    ty: Type,
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
pub fn init_in_env(_input: TokenStream) -> TokenStream {
    quote! {
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn init_in(env: chainsight_cdk::core::Env) -> Result<(), chainsight_cdk::initializer::InitError> {
            assert!(!INITIALIZED.with(|f| *f.borrow()), "Already initialized");
            use chainsight_cdk::initializer::Initializer;
            let initializer = chainsight_cdk::initializer::ChainsightInitializer::new(
                chainsight_cdk::initializer::InitConfig { env },
            );
            let init_result = initializer.initialize().await?;
            let proxy = init_result.proxy;
            INITIALIZED.with(|f| *f.borrow_mut() = true);
            PROXY.with(|f| *f.borrow_mut() = proxy);
            Ok(())
        }
        fn proxy() -> candid::Principal {
            PROXY.with(|proxy| proxy.borrow().clone())
        }
        thread_local! {
            static INITIALIZED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
            static PROXY: std::cell::RefCell<candid::Principal> = std::cell::RefCell::new(candid::Principal::anonymous());
        }
    }
    .into()
}

pub fn setup_func(input: TokenStream) -> TokenStream {
    let SetupArgs { fields } = syn::parse_macro_input!(input as SetupArgs);

    let setters: Vec<_> = fields
        .iter()
        .map(|field| Ident::new(&format!("set_{}", field.name), field.name.span()))
        .collect();

    let names: Vec<_> = fields.iter().map(|field| &field.name).collect();
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let expanded = quote! {
        chainsight_cdk_macros::manage_single_state!("setup_flag", bool, false);

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn setup(#( #names: #types ),*) -> Result<(), String> {
            if (get_setup_flag()) {
                return Err(String::from("Already setup"));
            }
            #( #setters(#names); )*
            set_setup_flag(true);
            Ok(())
        }
    };
    TokenStream::from(expanded)
}

pub fn timer_task_func(input: TokenStream) -> TokenStream {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args = parser.parse(input).expect("Failed to parse input");
    if args.len() != 3 {
        panic!("Expected 3 arguments");
    }

    let func_name = match &args[0] {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Str(lit_str) = &lit.lit {
                syn::Ident::new(&format!("{}", lit_str.value()), lit_str.span())
            } else {
                panic!("Expected a string literal for the variable name");
            }
        }
        _ => panic!("Expected a string literal for the variable name"),
    };
    let called_func_name = match &args[1] {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Str(lit_str) = &lit.lit {
                syn::Ident::new(&format!("{}", lit_str.value()), lit_str.span())
            } else {
                panic!("Expected a string literal for the variable name");
            }
        }
        _ => panic!("Expected a string literal for the variable name"),
    };
    let is_async = match &args[2] {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Bool(lit_bool) = &lit.lit {
                lit_bool.value
            } else {
                panic!("Expected a boolean literal for the variable name");
            }
        }
        _ => panic!("Expected a boolean literal for the variable name"),
    };
    let called_closure = if is_async {
        quote! {
            ic_cdk::spawn(async move {
                #called_func_name().await;
            });
        }
    } else {
        quote! {
            #called_func_name();
        }
    };

    let timer_state_name = format!("timer_task_{}", called_func_name);
    let set_timer_state_name = syn::Ident::new(
        &format!("set_timer_task_{}", called_func_name),
        called_func_name.span(),
    );

    let output = quote! {
        chainsight_cdk_macros::manage_single_state!(#timer_state_name, ic_cdk_timers::TimerId, false);

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        pub fn #func_name(task_interval_secs: u32, delay_secs: u32) {
            let current_time_sec = (ic_cdk::api::time() / (1000 * 1000000)) as u32;
            let round_timestamp = |ts: u32, unit: u32| ts / unit * unit;
            let delay = round_timestamp(current_time_sec, task_interval_secs) + task_interval_secs + delay_secs - current_time_sec;
            ic_cdk_timers::set_timer(std::time::Duration::from_secs(delay as u64), move || {
                let timer_id = ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(task_interval_secs as u64), || {
                    #called_closure;
                });
                #set_timer_state_name(timer_id);
            });
        }
    };

    output.into()
}
