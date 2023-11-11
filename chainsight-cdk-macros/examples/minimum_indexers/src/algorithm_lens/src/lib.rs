use chainsight_cdk_macros::def_algorithm_lens_canister;
mod app; // NOTE: logics
use app::CalculateArgs;

def_algorithm_lens_canister!(
    "{
        \"common\": {
            \"canister_name\": \"app\"
        },
        \"target_count\": 10,
        \"args_type\": \"CalculateArgs\"
    }"
);
