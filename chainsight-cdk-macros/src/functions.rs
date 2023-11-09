use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, punctuated::Punctuated, Ident, Result, Token, Type};

pub fn init_in_env(_input: TokenStream) -> TokenStream {
    init_in_env_internal().into()
}
fn init_in_env_internal() -> proc_macro2::TokenStream {
    quote! {
        use chainsight_cdk::initializer::{CycleManagements, Initializer};
        use ic_cdk::api::management_canister::{provisional::{CanisterIdRecord, CanisterSettings}, main::{update_settings, UpdateSettingsArgument}};
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn init_in(env: chainsight_cdk::core::Env, cycles: CycleManagements) -> Result<(), chainsight_cdk::initializer::InitError> {
            assert!(!INITIALIZED.with(|f| *f.borrow()), "Already initialized");
            let initializer = chainsight_cdk::initializer::ChainsightInitializer::new(
                chainsight_cdk::initializer::InitConfig { env: env.clone() },
            );
            let deployer = ic_cdk::caller();
            let init_result = initializer.initialize(&deployer, &cycles).await?;
            let proxy = init_result.proxy;
            INITIALIZED.with(|f| *f.borrow_mut() = true);
            PROXY.with(|f| *f.borrow_mut() = proxy);
            ENV.with(|f| *f.borrow_mut() = env);

            let canister_id = ic_cdk::api::id();
            let vault = init_result.vault;
            let (status,) = ic_cdk::api::management_canister::main::canister_status(CanisterIdRecord {
                canister_id,
            })
            .await
            .unwrap();
            let mut controllers = status.settings.controllers.clone();
            controllers.push(vault.clone());
            update_settings(UpdateSettingsArgument {
                canister_id,
                settings: CanisterSettings {
                    controllers: Some(controllers),
                    compute_allocation: None,
                    freezing_threshold: None,
                    memory_allocation: None,
                },
            })
            .await.unwrap();
            Ok(())
        }
        fn proxy() -> candid::Principal {
            PROXY.with(|proxy| proxy.borrow().clone())
        }
        fn get_env() -> chainsight_cdk::core::Env {
            ENV.with(|env| env.borrow().clone())
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn get_proxy() -> candid::Principal {
            proxy()
        }

        thread_local! {
            static INITIALIZED: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
            static PROXY: std::cell::RefCell<candid::Principal> = std::cell::RefCell::new(candid::Principal::anonymous());
            static ENV: std::cell::RefCell<chainsight_cdk::core::Env> = std::cell::RefCell::new(chainsight_cdk::core::Env::default());
        }
    }
}

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
pub fn setup_func(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as SetupArgs);
    setup_func_internal(args).into()
}
fn setup_func_internal(input: SetupArgs) -> proc_macro2::TokenStream {
    let SetupArgs { fields } = input;

    let setters: Vec<_> = fields
        .iter()
        .map(|field| Ident::new(&format!("set_{}", field.name), field.name.span()))
        .collect();

    let names: Vec<_> = fields.iter().map(|field| &field.name).collect();
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    quote! {
        chainsight_cdk_macros::manage_single_state!("setup_flag", bool, false);

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn setup(#( #names: #types ),*) -> Result<(), String> {
            assert!(!get_setup_flag(), "Already setup");
            #( #setters(#names); )*
            set_setup_flag(true);
            Ok(())
        }
    }
}

struct TimerTaskArgs {
    func_name: syn::LitStr,
    called_func_name: syn::LitStr,
}
impl Parse for TimerTaskArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let func_name: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let called_func_name: syn::LitStr = input.parse()?;
        Ok(TimerTaskArgs {
            func_name,
            called_func_name,
        })
    }
}
pub fn timer_task_func(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as TimerTaskArgs);
    timer_task_func_internal(args).into()
}
fn timer_task_func_internal(args: TimerTaskArgs) -> proc_macro2::TokenStream {
    let TimerTaskArgs {
        func_name,
        called_func_name,
    } = args;

    let func_name_ident = format_ident!("{}", func_name.value());
    let called_func_name_str = called_func_name.value().to_string();

    quote! {
        thread_local! {
            static INDEXING_INTERVAL: std::cell::RefCell<u32> = std::cell::RefCell::new(0);
        }

        fn get_indexing_interval() -> u32 {
            INDEXING_INTERVAL.with(|f| f.borrow().clone())
        }
        fn set_indexing_interval(interval: u32) {
            INDEXING_INTERVAL.with(|f| *f.borrow_mut() = interval);
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        pub async fn #func_name_ident(task_interval_secs: u32, delay_secs: u32) {
            set_indexing_interval(task_interval_secs);
            let res = ic_cdk::api::call::call::<(u32, u32, String, Vec<u8>), ()>(
                proxy(),
                "start_indexing",
                (task_interval_secs, delay_secs, #called_func_name_str.to_string(), Vec::<u8>::new()),
            )
            .await;
            match res {
                Ok(_) => {}
                Err(e) => { panic!("Failed to start indexing: {:?}", e) }
            };
        }
    }
}

#[derive(Debug)]
struct LensArgs {
    target_count: usize,
    func_arg: Option<Type>,
}
impl Parse for LensArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let target_count: syn::LitInt = input.parse()?;
        let func_arg = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(LensArgs {
            target_count: target_count.base10_parse().unwrap(),
            func_arg,
        })
    }
}
pub fn lens_method(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as LensArgs);
    lens_method_internal(args).into()
}
fn lens_method_internal(args: LensArgs) -> proc_macro2::TokenStream {
    let getter_name = format_ident!("{}", "get_result");
    let proxy_getter_name = format_ident!("{}", "proxy_get_result");

    let value_ty = format_ident!("{}", "LensValue");
    let target_count = args.target_count;

    let (getter_func, receiver_provider, inter_calc_func) = match args.func_arg {
        Some(arg_ty) => (
            quote! {
                #[ic_cdk::update]
                #[candid::candid_method(update)]
                async fn #getter_name(targets: Vec<String>, input: #arg_ty) -> #value_ty {
                    if targets.len() != #target_count {
                        panic!("Expected {} targets", #target_count);
                    }
                    _calc((targets, input)).await
                }
            },
            quote! {
                chainsight_cdk::rpc::AsyncReceiverProvider::<(Vec<String>, #arg_ty), #value_ty>::new(
                    proxy(),
                    _calc,
                )
            },
            quote! {
                fn _calc(args: (Vec<String>, #arg_ty)) -> BoxFuture<'static, #value_ty> {
                    async move { calculate(args.0, args.1).await }.boxed()
                }
            },
        ),
        None => (
            quote! {
                #[ic_cdk::update]
                #[candid::candid_method(update)]
                async fn #getter_name(targets: Vec<String>) -> #value_ty {
                    if targets.len() != #target_count {
                        panic!("Expected {} targets", #target_count);
                    }
                    _calc(targets).await
                }
            },
            quote! {
                chainsight_cdk::rpc::AsyncReceiverProvider::<Vec<String>, #value_ty>::new(
                    proxy(),
                    _calc,
                )
            },
            quote! {
                fn _calc(targets: Vec<String>) -> BoxFuture<'static, #value_ty> {
                    async move { calculate(targets).await }.boxed()
                }
            },
        ),
    };

    quote! {
        #getter_func

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn #proxy_getter_name(input:Vec<u8>) -> Vec<u8> {
            use chainsight_cdk::rpc::Receiver;
            let reciever_provider = #receiver_provider;
            reciever_provider.reply(input).await
        }

        #[ic_cdk::query]
        #[candid::candid_method(query)]
        fn get_sources() -> Vec<chainsight_cdk::core::Sources<std::collections::HashMap<String, String>>> {
            vec![]
        }

        #inter_calc_func
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use rust_format::{Formatter, RustFmt};

    use super::*;

    #[test]
    fn test_snapshot_init_in_env() {
        let generated = init_in_env_internal();
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__init_in_env", formatted);
    }

    #[test]
    fn test_snapshot_setup_fumc() {
        let input = quote! {
            {
                target_canister: String,
                target_addr: String,
                web3_ctx_param: Web3CtxParam
            }
        };
        let args: syn::Result<SetupArgs> = syn::parse2(input);
        let generated = setup_func_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__setup_func", formatted);
    }

    #[test]
    fn test_snapshot_timer_task_func() {
        let input = quote! {"set_task","HELLO"};
        let args: syn::Result<TimerTaskArgs> = syn::parse2(input);
        let generated = timer_task_func_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__timer_task_func", formatted);
    }

    #[test]
    fn test_snapshot_lens_method() {
        let input = quote! {10};
        let args: syn::Result<LensArgs> = syn::parse2(input);
        let generated = lens_method_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__lens_method", formatted);
    }

    #[test]
    fn test_snapshot_lens_method_with_args() {
        let input = quote! {10, CalculateArgs};
        let args: syn::Result<LensArgs> = syn::parse2(input);
        let generated = lens_method_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__lens_method__with_args", formatted);
    }
}
