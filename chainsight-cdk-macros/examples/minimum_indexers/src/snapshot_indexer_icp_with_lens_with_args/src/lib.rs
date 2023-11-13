use chainsight_cdk_macros::def_snapshot_indexer_icp_canister;
mod example_canister; // NOTE: bindings / Originally intended for a different crate

def_snapshot_indexer_icp_canister!(
    "{
        \"common\":{
            \"canister_name\":\"example_canister\"
        },
        \"method_identifier\": \"get_last_snapshot : () -> (record { value : text; timestamp : nat64 })\",
        \"lens_parameter\":{
            \"with_args\":true
        }
    }"
);
