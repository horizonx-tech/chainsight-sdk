mod canisters;
mod functions;
mod indexers;
mod internal;
mod states;
mod storages;
mod utils;
mod web3;

use proc_macro::TokenStream;

#[proc_macro]
pub fn chainsight_common(input: TokenStream) -> TokenStream {
    utils::chainsight_common(input)
}

#[proc_macro]
pub fn web3_event_indexer(input: TokenStream) -> TokenStream {
    indexers::web3_event_indexer(input)
}

#[proc_macro]
pub fn algorithm_indexer(input: TokenStream) -> TokenStream {
    indexers::algorithm_indexer(input)
}
#[proc_macro]
pub fn algorithm_indexer_with_args(input: TokenStream) -> TokenStream {
    indexers::algorithm_indexer_with_args(input)
}

#[proc_macro]
pub fn algorithm_lens_finder(input: TokenStream) -> TokenStream {
    indexers::algorithm_lens_finder(input)
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
pub fn init_in(input: TokenStream) -> TokenStream {
    functions::init_in_env(input)
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
pub fn prepare_stable_structure(input: TokenStream) -> TokenStream {
    storages::prepare_stable_structure(input)
}

#[proc_macro_derive(StableMemoryStorable, attributes(stable_mem_storable_opts))]
pub fn derive_storable_in_stable_memory(input: TokenStream) -> TokenStream {
    storages::derive_storable_in_stable_memory(input)
}

#[proc_macro]
pub fn stable_memory_for_scalar(input: TokenStream) -> TokenStream {
    storages::stable_memory_for_scalar(input)
}

#[proc_macro_derive(CborSerde)]
pub fn derive_cbor_serde(input: TokenStream) -> TokenStream {
    utils::derive_cbor_serde(input)
}

#[proc_macro]
pub fn def_algorithm_indexer_canister(input: TokenStream) -> TokenStream {
    canisters::algorithm_indexer::def_algorithm_indexer_canister(input)
}
#[proc_macro]
pub fn def_algorithm_lens_canister(input: TokenStream) -> TokenStream {
    canisters::algorithm_lens::def_algorithm_lens_canister(input)
}

#[proc_macro]
pub fn def_relayer_canister(input: TokenStream) -> TokenStream {
    canisters::relayer::def_relayer_canister(input)
}

#[proc_macro]
pub fn def_event_indexer_canister(input: TokenStream) -> TokenStream {
    canisters::event_indexer::def_event_indexer_canister(input)
}
#[proc_macro]
pub fn def_snapshot_indexer_evm_canister(input: TokenStream) -> TokenStream {
    canisters::snapshot_indexer_evm::def_snapshot_indexer_evm(input)
}
#[proc_macro]
pub fn def_snapshot_indexer_https_canister(input: TokenStream) -> TokenStream {
    canisters::snapshot_indexer_https::def_snapshot_indexer_https(input)
}
#[proc_macro]
pub fn def_snapshot_indexer_icp_canister(input: TokenStream) -> TokenStream {
    canisters::snapshot_indexer_icp::def_snapshot_indexer_icp(input)
}
#[proc_macro]
pub fn stable_memory_for_vec(input: TokenStream) -> TokenStream {
    storages::stable_memory_for_vec(input)
}
#[proc_macro]
pub fn stable_memory_for_btree_map(input: TokenStream) -> TokenStream {
    storages::stable_memory_for_btree_map(input)
}

#[proc_macro]
pub fn define_transform_for_web3(input: TokenStream) -> TokenStream {
    web3::define_transform_for_web3(input)
}

#[proc_macro]
pub fn define_web3_ctx(input: TokenStream) -> TokenStream {
    web3::define_web3_ctx(input)
}

#[proc_macro]
pub fn define_relayer_web3_ctx(input: TokenStream) -> TokenStream {
    web3::define_relayer_web3_ctx(input)
}

#[proc_macro]
pub fn define_get_ethereum_address(input: TokenStream) -> TokenStream {
    web3::define_get_ethereum_address(input)
}

#[proc_macro]
pub fn define_withdraw_balance(input: TokenStream) -> TokenStream {
    web3::define_withdraw_balance(input)
}

#[proc_macro]
pub fn define_logger(input: TokenStream) -> TokenStream {
    utils::define_logger(input)
}

#[proc_macro_derive(ContractEvent)]
pub fn contract_event_derive(input: TokenStream) -> TokenStream {
    web3::contract_event_derive(input)
}

#[proc_macro_derive(Persist)]
pub fn persist_derive(input: TokenStream) -> TokenStream {
    states::persist_derive(input)
}

#[proc_macro_derive(KeyValueStore, attributes(memory_id))]
pub fn key_value_store_derive(input: TokenStream) -> TokenStream {
    storages::key_value_store_derive(input)
}
#[proc_macro]
pub fn generate_queries_for_key_value_store_struct(input: TokenStream) -> TokenStream {
    storages::generate_queries_for_key_value_store_struct(input)
}

#[proc_macro_derive(KeyValuesStore, attributes(memory_id))]
pub fn key_values_store_derive(input: TokenStream) -> TokenStream {
    storages::key_values_store_derive(input)
}
#[proc_macro]
pub fn generate_queries_for_key_values_store_struct(input: TokenStream) -> TokenStream {
    storages::generate_queries_for_key_values_store_struct(input)
}

#[proc_macro]
pub fn lens_method(input: TokenStream) -> TokenStream {
    functions::lens_method(input)
}

#[proc_macro]
pub fn algorithm_indexer_source(input: TokenStream) -> TokenStream {
    indexers::sources::algorithm_indexer_source(input)
}
#[proc_macro]
pub fn web3_event_indexer_source(input: TokenStream) -> TokenStream {
    indexers::sources::web3_event_indexer_source(input)
}
#[proc_macro]
pub fn snapshot_indexer_web3_source(input: TokenStream) -> TokenStream {
    indexers::sources::snapshot_indexer_web3_source(input)
}
#[proc_macro]
pub fn snapshot_indexer_https_source(input: TokenStream) -> TokenStream {
    indexers::sources::snapshot_indexer_https_source(input)
}
#[proc_macro]
pub fn snapshot_indexer_icp_source(input: TokenStream) -> TokenStream {
    indexers::sources::snapshot_indexer_icp_source(input)
}
#[proc_macro]
pub fn relayer_source(input: TokenStream) -> TokenStream {
    indexers::sources::relayer_source(input)
}

#[proc_macro_attribute]
pub fn only_controller(_attr: proc_macro::TokenStream, item: TokenStream) -> TokenStream {
    canisters::attributes::only_controller(_attr, item)
}

#[proc_macro_attribute]
pub fn only_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    canisters::attributes::only_proxy(_attr, item)
}

#[proc_macro_attribute]
pub fn metric(_attr: TokenStream, item: TokenStream) -> TokenStream {
    canisters::attributes::metric(_attr, item)
}
