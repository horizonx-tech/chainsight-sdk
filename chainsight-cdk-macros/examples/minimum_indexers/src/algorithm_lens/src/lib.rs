use chainsight_cdk_macros::def_algorithm_lens_canister;
mod app;

def_algorithm_lens_canister!(
    "{
    \"common\": {
        \"canister_name\": \"app\",
        \"monitor_duration\": 1000
    },
    \"target_count\": 10
}"
);
