use chainsight_cdk_macros::def_relayer_canister;
mod example_canister; // NOTE: logics

def_relayer_canister!(
    "{
        \"common\":{
            \"canister_name\":\"example_canister\"
        },
        \"destination\":\"0539a0EF8e5E60891fFf0958A059E049e43020d9\",
        \"method_identifier\":\"get_result : (LensArgs) -> (record { vec nat; vec text; vec nat; vec nat; vec nat })\",
        \"abi_file_path\":\"__interfaces/IProposalSynchronizer.json\",
        \"lens_parameter\":{
            \"with_args\":true
        },
        \"method_name\":\"batchSynchronize\"
    }"
);
