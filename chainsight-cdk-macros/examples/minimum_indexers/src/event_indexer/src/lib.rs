use chainsight_cdk_macros::def_event_indexer_canister;

def_event_indexer_canister!(
    "{
    \"common\": {
        \"canister_name\": \"app\",
        \"monitor_duration\": 1000
    },
    \"def\": {
        \"identifier\": \"Transfer\",
        \"abi_file_path\":\"src/event_indexer/abi/ERC20.json\"
    }
}"
);
