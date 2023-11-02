use chainsight_cdk_macros::def_relayer_canister;
mod example_relayer; // NOTE: logics

def_relayer_canister!(
    "{
        \"common\":{
            \"monitor_duration\":60,
            \"canister_name\":\"example_relayer\"
        },
        \"destination\":\"0539a0EF8e5E60891fFf0958A059E049e43020d9\",
        \"oracle_type\":\"uint256\",
        \"method_identifier\":\"get_last_snapshot_value : () -> (text)\",
        \"abi_file_path\":\"__interfaces/Uint256Oracle.json\",
        \"lens_targets\":null
    }"
);
