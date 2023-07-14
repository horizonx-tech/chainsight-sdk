use candid::CandidType;
use chainsight_cdk::{indexer::IndexingConfig, storage::Data};
use chainsight_cdk_macros::{
    algorithm_indexer, chainsight_common, did_export, init_in, manage_single_state, setup_func,
    timer_task_func, KeyValueStore, KeyValuesStore, Persist,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
chainsight_common!(60);
init_in!();
manage_single_state!("target_addr", String, false);

setup_func!({
    target_addr: String,
    config: IndexingConfig
});
algorithm_indexer!(HashMap<u64, Vec<TransferEvent>>,"proxy_call");
timer_task_func!("set_task", "index", true);

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

#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize, Persist, KeyValueStore)]
#[memory_id(2)]
pub struct Balance {
    value: String,
}

#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize, Persist, KeyValuesStore)]
#[memory_id(1)]
pub struct Account {
    value: String,
}

did_export!("algorithm_indexer");
