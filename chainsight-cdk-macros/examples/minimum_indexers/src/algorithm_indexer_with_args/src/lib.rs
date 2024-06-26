use candid::{CandidType, Decode, Encode, Principal};
use chainsight_cdk::{indexer::IndexingConfig, storage::Data};
use chainsight_cdk_macros::{
    algorithm_indexer_source, algorithm_indexer_with_args, chainsight_common, did_export,
    generate_queries_for_key_value_store_struct, generate_queries_for_key_values_store_struct,
    init_in, manage_single_state, setup_func, timer_task_func, KeyValueStore, KeyValuesStore,
    Persist, StableMemoryStorable,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod app;
use app::persist;

chainsight_common!();
init_in!();
manage_single_state!("target_addr", String, false);

setup_func!({
    target_addr: String,
    config: IndexingConfig
});
algorithm_indexer_source!();
algorithm_indexer_with_args!(TransferEvent, (Principal, String, String), "proxy_call");
timer_task_func!("set_task", "index");

/// This is auto-generated from yaml
#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}

#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize, Persist, KeyValueStore)]
#[memory_id(1)]
pub struct TotalSupply {
    value: String,
}
generate_queries_for_key_value_store_struct!(TotalSupply);

#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize, Persist, KeyValueStore)]
#[memory_id(2)]
pub struct Balance {
    value: String,
}
generate_queries_for_key_value_store_struct!(Balance);

#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize, Persist, KeyValuesStore)]
#[memory_id(1)]
pub struct Account {
    value: String,
}
generate_queries_for_key_values_store_struct!(Account);

did_export!("example_canister");
