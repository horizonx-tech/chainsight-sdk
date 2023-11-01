use chainsight_cdk_macros::def_snapshot_indexer_icp_canister;
mod snapshot_indexer_icp; // NOTE: Originally intended for a different crate
def_snapshot_indexer_icp_canister!("{
    \"common\":{
        \"monitor_duration\": 60,
        \"canister_name\":\"snapshot_indexer_icp\"
    },
    \"method_identifier\": \"get_last_snapshot : () -> (record { value : text; timestamp : nat64 })\"
}");
