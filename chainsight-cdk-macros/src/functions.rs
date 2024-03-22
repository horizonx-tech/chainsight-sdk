use chainsight_cdk::config::components::{LENS_FUNCTION_ARGS_TYPE, LENS_FUNCTION_RESPONSE_TYPE};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::LitInt;
use syn::{braced, punctuated::Punctuated, Ident, Result, Token, Type};

struct InitInEnvArgs {
    stable_memory_id: Option<LitInt>,
}
impl Parse for InitInEnvArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let stable_memory_id = if !input.is_empty() {
            let parsed: LitInt = input.parse()?;
            Some(parsed)
        } else {
            None
        };
        Ok(InitInEnvArgs { stable_memory_id })
    }
}
pub fn init_in_env(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as InitInEnvArgs);
    init_in_env_internal(args).into()
}
fn init_in_env_internal(input: InitInEnvArgs) -> proc_macro2::TokenStream {
    let struct_quote = quote! {
        #[derive(Debug, Clone, Default, PartialEq, candid::CandidType, candid::Deserialize, serde::Serialize, StableMemoryStorable)]
        #[stable_mem_storable_opts(max_size = 10000, is_fixed_size = false)] // temp: max_size
        pub struct InitializingState {
            pub initialized: bool,
            pub proxy: String,
            pub env: chainsight_cdk::core::Env,
        }
    };
    let storage_quote = if let Some(memory_id) = input.stable_memory_id {
        quote! {
            stable_memory_for_scalar!("initializing_state", InitializingState, #memory_id, false);
        }
    } else {
        quote! {
            manage_single_state!("initializing_state", InitializingState, false);
        }
    };

    quote! {
        #struct_quote
        #storage_quote

        use chainsight_cdk::initializer::{CycleManagements, Initializer};
        use ic_cdk::api::management_canister::{provisional::{CanisterIdRecord, CanisterSettings}, main::{update_settings, UpdateSettingsArgument}};
        #[ic_cdk::update]
        #[candid::candid_method(update)]
        async fn init_in(env: chainsight_cdk::core::Env, cycles: CycleManagements) -> Result<(), chainsight_cdk::initializer::InitError> {
            assert!(!is_initialized(), "Already initialized");

            let initializer = chainsight_cdk::initializer::ChainsightInitializer::new(
                chainsight_cdk::initializer::InitConfig { env: env.clone() },
            );
            let deployer = ic_cdk::caller();
            let init_result = initializer.initialize(&deployer, &cycles).await?;
            let proxy = init_result.proxy;

            set_initializing_state(InitializingState {
                initialized: true,
                proxy: proxy.to_text(),
                env,
            }); // note: when using stable_memory, the returned result is not handled

            Ok(())
        }
        fn proxy() -> candid::Principal {
            candid::Principal::from_text(get_initializing_state().proxy).unwrap()
        }
        fn get_env() -> chainsight_cdk::core::Env {
            get_initializing_state().env
        }
        fn is_initialized() -> bool {
            get_initializing_state().initialized
        }

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn get_proxy() -> candid::Principal {
            proxy()
        }
    }
}

struct SetupArgs {
    fields: Punctuated<NamedField, Token![,]>,
    stable_memory_id: Option<LitInt>,
}
impl Parse for SetupArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let fields = Punctuated::parse_terminated(&content)?;
        let stable_memory_id = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let parsed: LitInt = input.parse()?;
            Some(parsed)
        } else {
            None
        };

        Ok(SetupArgs {
            fields,
            stable_memory_id,
        })
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
    let SetupArgs {
        fields,
        stable_memory_id,
    } = input;

    let setters: Vec<_> = fields
        .iter()
        .map(|field| Ident::new(&format!("set_{}", field.name), field.name.span()))
        .collect();

    let names: Vec<_> = fields.iter().map(|field| &field.name).collect();
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    // If stable_memory_id is specified, use Stable Memory, otherwise use Heap Memory
    let storage_quote = if let Some(memory_id) = stable_memory_id {
        quote! {
            stable_memory_for_scalar!("setup_flag", chainsight_cdk::storage::StorableBool, #memory_id, false);
        }
    } else {
        quote! {
            manage_single_state!("setup_flag", bool, false);
        }
    };

    quote! {
        #storage_quote

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        fn setup(#( #names: #types ),*) -> Result<(), String> {
            assert!(!bool::from(get_setup_flag()), "Already setup");
            #( #setters(#names.into()); )*
            set_setup_flag(true.into()); // note: when using stable_memory, the returned result is not handled
            Ok(())
        }
    }
}

struct TimerTaskArgs {
    func_name: syn::LitStr,
    called_func_name: syn::LitStr,
    stable_memory_id: Option<LitInt>,
}
impl Parse for TimerTaskArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let func_name: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let called_func_name: syn::LitStr = input.parse()?;
        let stable_memory_id = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let parsed: LitInt = input.parse()?;
            Some(parsed)
        } else {
            None
        };
        Ok(TimerTaskArgs {
            func_name,
            called_func_name,
            stable_memory_id,
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
        stable_memory_id,
    } = args;

    let func_name_ident = format_ident!("{}", func_name.value());
    let called_func_name_str = called_func_name.value().to_string();

    // If stable_memory_id is specified, use Stable Memory, otherwise use Heap Memory
    let storage_quote = if let Some(memory_id) = stable_memory_id {
        quote! {
            stable_memory_for_scalar!("indexing_interval", u32, #memory_id, false);
        }
    } else {
        quote! {
            manage_single_state!("indexing_interval", u32, false);
        }
    };

    quote! {
        #storage_quote

        #[ic_cdk::update]
        #[candid::candid_method(update)]
        pub async fn #func_name_ident(task_interval_secs: u32, delay_secs: u32, is_rounded_start_time: bool) {
            set_indexing_interval(task_interval_secs);
            let res = ic_cdk::api::call::call::<(u32, u32, bool, String, Vec<u8>), ()>(
                proxy(),
                "start_indexing_with_is_rounded",
                (task_interval_secs, delay_secs, is_rounded_start_time, #called_func_name_str.to_string(), Vec::<u8>::new()),
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
struct LensMethodArgs {
    target_count: usize,
    func_arg: Option<Type>,
}
impl Parse for LensMethodArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let target_count: syn::LitInt = input.parse()?;
        let func_arg = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(LensMethodArgs {
            target_count: target_count.base10_parse().unwrap(),
            func_arg,
        })
    }
}
pub fn lens_method(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as LensMethodArgs);
    lens_method_internal(args).into()
}
fn lens_method_internal(args: LensMethodArgs) -> proc_macro2::TokenStream {
    let getter_name = format_ident!("{}", "get_result");
    let proxy_getter_name = format_ident!("{}", "proxy_get_result");

    let args_ty = format_ident!("{}", LENS_FUNCTION_ARGS_TYPE);
    let value_ty = format_ident!("{}", LENS_FUNCTION_RESPONSE_TYPE);
    let target_count = args.target_count;

    let (lens_args_def, getter_func, receiver_provider, inter_calc_func) = match args.func_arg {
        Some(func_arg_ty) => (
            quote! {
                #[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]
                pub struct #args_ty {
                    pub targets: Vec<String>,
                    pub args: #func_arg_ty,
                }
            },
            quote! {
                #[ic_cdk::update]
                #[candid::candid_method(update)]
                async fn #getter_name(input: #args_ty) -> #value_ty {
                    if input.targets.len() != #target_count {
                        panic!("Expected {} targets", #target_count);
                    }
                    _calc(input).await
                }
            },
            quote! {
                chainsight_cdk::rpc::AsyncReceiverProvider::<#args_ty, #value_ty>::new(
                    proxy(),
                    _calc,
                )
            },
            quote! {
                fn _calc(input: #args_ty) -> BoxFuture<'static, #value_ty> {
                    async move { calculate(input.targets, input.args).await }.boxed()
                }
            },
        ),
        None => (
            quote! {
                type #args_ty = Vec<String>;
            },
            quote! {
                #[ic_cdk::update]
                #[candid::candid_method(update)]
                async fn #getter_name(targets: #args_ty) -> #value_ty {
                    if targets.len() != #target_count {
                        panic!("Expected {} targets", #target_count);
                    }
                    _calc(targets).await
                }
            },
            quote! {
                chainsight_cdk::rpc::AsyncReceiverProvider::<#args_ty, #value_ty>::new(
                    proxy(),
                    _calc,
                )
            },
            quote! {
                fn _calc(input: #args_ty) -> BoxFuture<'static, #value_ty> {
                    async move { calculate(input).await }.boxed()
                }
            },
        ),
    };

    quote! {
        #lens_args_def
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
        let input = quote! {};
        let args: syn::Result<InitInEnvArgs> = syn::parse2(input);
        let generated = init_in_env_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__init_in_env", formatted);
    }

    #[test]
    fn test_snapshot_init_in_env_with_stable_memory() {
        let input = quote! {1};
        let args: syn::Result<InitInEnvArgs> = syn::parse2(input);
        let generated = init_in_env_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__init_in_env_with_stable_memory", formatted);
    }

    #[test]
    fn test_snapshot_setup_func() {
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
    fn test_snapshot_setup_func_with_stable_memory() {
        let input = quote! {
            {
                target_canister: String,
                target_addr: String,
                web3_ctx_param: Web3CtxParam
            },
            0
        };
        let args: syn::Result<SetupArgs> = syn::parse2(input);
        let generated = setup_func_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__setup_func_with_stable_memory", formatted);
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
    fn test_snapshot_timer_task_func_with_stable_memory() {
        let input = quote! {"set_task","HELLO",1};
        let args: syn::Result<TimerTaskArgs> = syn::parse2(input);
        let generated = timer_task_func_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__timer_task_func_with_stable_memory", formatted);
    }

    #[test]
    fn test_snapshot_lens_method() {
        let input = quote! {10};
        let args: syn::Result<LensMethodArgs> = syn::parse2(input);
        let generated = lens_method_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__lens_method", formatted);
    }

    #[test]
    fn test_snapshot_lens_method_with_args() {
        let input = quote! {10, CalculateArgs};
        let args: syn::Result<LensMethodArgs> = syn::parse2(input);
        let generated = lens_method_internal(args.unwrap());
        let formatted = RustFmt::default()
            .format_str(generated.to_string())
            .expect("rustfmt failed");
        assert_snapshot!("snapshot__lens_method__with_args", formatted);
    }
}
