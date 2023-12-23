use chainsight_cdk_macros::def_snapshot_indexer_https_canister;
mod example_canister; // NOTE: logics / Originally intended for a different crate
use example_canister::get_query_parameters;

def_snapshot_indexer_https_canister!(
    "{
        \"common\":{
            \"canister_name\":\"example_canister\"
        },
        \"url\": \"https://api.coingecko.com/api/v3/simple/price\",
        \"headers\":{
            \"content-type\": \"application/json\"
        },
        \"queries\":{
            \"Func\": \"get_query_parameters\"
        }
    }"
);
