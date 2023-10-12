use chainsight_cdk_macros::def_algorithm_indexer_canister;
mod app;
mod example_canister;
use app::persist;
def_algorithm_indexer_canister!(
    "{
    \"common\": {
        \"monitor_duration\": 3600,
        \"canister_name\": \"example_canister\"
    },
    \"indexing\": {
        \"start_from\": 1222222,
        \"indexer_type\": \"algorithm\"
    },
    \"input\": {
        \"method_name\": \"get_list\",
        \"response_type\": \"String\",
        \"source_type\": \"EventIndexer\"
    }
}"
);
