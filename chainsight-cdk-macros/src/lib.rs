mod web3;
mod functions;
mod states;
mod utils;

use proc_macro::TokenStream;

#[proc_macro]
pub fn cross_canister_call_func(input: TokenStream) -> TokenStream {
    utils::cross_canister_call_func(input)
}

#[proc_macro]
pub fn monitoring_canister_metrics(input: TokenStream) -> TokenStream {
    utils::monitoring_canister_metrics(input)
}

#[proc_macro]
pub fn did_export(input: TokenStream) -> TokenStream {
    utils::did_export(input)
}

#[proc_macro]
pub fn setup_func(input: TokenStream) -> TokenStream {
    functions::setup_func(input)
}

#[proc_macro]
pub fn timer_task_func(input: TokenStream) -> TokenStream {
    functions::timer_task_func(input)
}

#[proc_macro]
pub fn manage_single_state(input: TokenStream) -> TokenStream {
    states::manage_single_state(input)
}

#[proc_macro]
pub fn manage_vec_state(input: TokenStream) -> TokenStream {
    states::manage_vec_state(input)
}

#[proc_macro]
pub fn manage_map_state(input: TokenStream) -> TokenStream {
    states::manage_map_state(input)
}

#[proc_macro]
pub fn define_transform_for_web3(_input: TokenStream) -> TokenStream {
    web3::define_transform_for_web3()
}

#[proc_macro]
pub fn define_web3_ctx(_input: TokenStream) -> TokenStream {
    web3::define_web3_ctx()
}

#[proc_macro]
pub fn define_get_ethereum_address(_input: TokenStream) -> TokenStream {
    web3::define_get_ethereum_address()
}
