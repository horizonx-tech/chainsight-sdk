use chainsight_cdk_macros::def_snapshot_indexer_https_canister;
mod example_canister; // NOTE: logics / Originally intended for a different crate

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
            \"Const\": {
                \"ids\": \"dai\",
                \"vs_currencies\": \"usd\"
            }
        }
    }"
);
