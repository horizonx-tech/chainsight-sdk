use chainsight_cdk_macros::def_relayer_canister;

def_relayer_canister!(
    "{
        \"common\": {
            \"monitor_duration\": 3600,
            \"canister_name\": \"example_canister\"
        },
        \"destination\": \"0x1234567678\",
        \"oracle_type\": \"uint256\",
        \"method_name\": \"example\",
        \"canister_method_value_type\": {
            
        },
        
    }"
);
