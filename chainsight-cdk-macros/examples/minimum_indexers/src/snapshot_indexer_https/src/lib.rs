use chainsight_cdk_macros::def_snapshot_indexer_https_canister;
mod snapshot_indexer_https; // NOTE: Originally intended for a different crate
def_snapshot_indexer_https_canister!("{
    \"common\":{
        \"monitor_duration\": 60,
        \"canister_name\":\"snapshot_indexer_https\"
    },
    \"url\": \"https://api.coingecko.com/api/v3/simple/price\",
    \"headers\":{
        \"content-type\": \"application/json\"
    },
    \"queries\":{
        \"ids\": \"dai\",
        \"vs_currencies\": \"usd\"
    }
}");