use chainsight_cdk_macros::def_algorithm_indexer_canister;
mod app;
mod example_canister; // NOTE: bindings
use app::persist; // NOTE: logics

def_algorithm_indexer_canister!(
    "{
        \"common\": {
            \"canister_name\": \"example_canister\"
        },
        \"indexing\": {
            \"start_from\": 1222222,
            \"chunk_size\": null
        },
        \"input\": {
            \"method_name\": \"get_list\",
            \"response_type\": \"String\",
            \"source_type\": \"event_indexer\"
        },
        \"output\": {
            \"types\": []
        }
    }"
);
