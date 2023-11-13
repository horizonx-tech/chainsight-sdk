use chainsight_cdk_macros::def_algorithm_lens_canister;
mod example_canister; // NOTE: logics
use example_canister::CalculateArgs;

def_algorithm_lens_canister!(
    "{
        \"common\": {
            \"canister_name\": \"example_canister\"
        },
        \"target_count\": 10,
        \"args_type\": \"CalculateArgs\"
    }"
);
