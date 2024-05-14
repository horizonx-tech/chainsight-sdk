use chainsight_cdk_macros::def_algorithm_indexer_canister;
mod app;
mod example_canister; // NOTE: bindings
use app::{persist, OutputType1, OutputType2, Transfer}; // NOTE: logics

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
            \"response_type\": \"Transfer\",
            \"source_type\": \"event_indexer\"
        },
        \"output\": {
            \"types\": [
                {
                    \"name\": \"OutputType1\",
                    \"type_\": \"key_value\"
                },
                {
                    \"name\": \"OutputType2\",
                    \"type_\": \"key_values\"
                }
            ]
        }
    }"
);
