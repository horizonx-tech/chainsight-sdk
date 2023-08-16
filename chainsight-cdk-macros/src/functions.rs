use proc_macro::TokenStream;
use quote::{format_ident, quote};
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

struct LensArgs {
    func_output: Type,
    target_count: usize,
    func_arg: Option<Type>,
}

impl Parse for LensArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let func_output: Type = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let target_count: syn::LitInt = input.parse()?;

        let func_arg = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(LensArgs {
            func_output,
            target_count: target_count.base10_parse().unwrap(),
            func_arg,
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
                chainsight_cdk::initializer::InitConfig { env: env.clone() },
            );
            let init_result = initializer.initialize().await?;
            let proxy = init_result.proxy;
            INITIALIZED.with(|f| *f.borrow_mut() = true);
            PROXY.with(|f| *f.borrow_mut() = proxy);
            ENV.with(|f| *f.borrow_mut() = env);
            Ok(())
        }
        fn proxy() -> candid::Principal {
            PROXY.with(|proxy| proxy.borrow().clone())
        }
        fn get_env() -> chainsight_cdk::core::Env {
            ENV.with(|env| env.borrow().clone())
        }
        thread_local! {
            static INITIALIZED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
            static PROXY: std::cell::RefCell<candid::Principal> = std::cell::RefCell::new(candid::Principal::anonymous());
            static ENV: std::cell::RefCell<chainsight_cdk::core::Env> = std::cell::RefCell::new(chainsight_cdk::core::Env::default());
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
                update_last_executed();
            });
        }
    } else {
        quote! {
            #called_func_name();
            update_last_executed();
        }
    };

    let timer_state_name = format!("timer_task_{}", called_func_name);
    let set_timer_state_name = syn::Ident::new(
        &format!("set_timer_task_{}", called_func_name),
        called_func_name.span(),
    );

    let output = quote! {
        chainsight_cdk_macros::manage_single_state!(#timer_state_name, ic_cdk_timers::TimerId, false);
        thread_local!{
            static TIMER_DURATION: std::cell::RefCell<u32> = std::cell::RefCell::new(0);
        }
        fn set_timer_duration(duration: u32) {
            TIMER_DURATION.with(|f| *f.borrow_mut() = duration);
        }
        fn get_timer_duration() -> u32 {
            TIMER_DURATION.with(|f| *f.borrow())
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        pub fn #func_name(task_interval_secs: u32, delay_secs: u32) {
            let current_time_sec = (ic_cdk::api::time() / (1000 * 1000000)) as u32;
            let round_timestamp = |ts: u32, unit: u32| ts / unit * unit;
            let delay = round_timestamp(current_time_sec, task_interval_secs) + task_interval_secs + delay_secs - current_time_sec;
            set_timer_duration(task_interval_secs);
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

pub fn lens_method(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as LensArgs);

    let getter_name = format_ident!("{}", "get_result");
    let proxy_getter_name = format_ident!("{}", "proxy_get_result");

    let out = args.func_output;
    let target_count = args.target_count;

    let (getter_func, receiver_provider, inter_calc_func) = match args.func_arg {
        Some(arg_ty) => (
            quote! {
                #[ic_cdk::query]
                #[candid::candid_method(query)]
                async fn #getter_name(targets: Vec<String>, input: #arg_ty) -> #out {
                    if targets.len() != #target_count {
                        panic!("Expected {} targets", #target_count);
                    }
                    _calc(targets, input).await
                }
            },
            quote! {
                chainsight_cdk::rpc::AsyncReceiverProvider::<#arg_ty, #out>::new(
                    proxy(),
                    _calc,
                )
            },
            quote! {
                fn _calc(targets: Vec<String>, args: #arg_ty) -> BoxFuture<'static, #out> {
                    async move { app::calculate(targets, args).await }.boxed()
                }
            },
        ),
        None => (
            quote! {
                #[ic_cdk::query]
                #[candid::candid_method(query)]
                async fn #getter_name() -> #out {
                    _calc().await
                }
            },
            quote! {
                chainsight_cdk::rpc::AsyncReceiverProviderWithoutArgs::<#out>::new(
                    proxy(),
                    _calc,
                )
            },
            quote! {
                fn _calc() -> BoxFuture<'static, #out> {
                    async move { app::calculate().await }.boxed()
                }
            },
        ),
    };

    quote! {
        use app::{calculate};

        #getter_func

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn #proxy_getter_name(input:Vec<u8>) -> Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            let reciever_provider = #receiver_provider;
            reciever_provider.reply(input)
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<std::collections::HashMap<String, String>>> {
            vec![]
        }

        #inter_calc_func
    }
    .into()
}
