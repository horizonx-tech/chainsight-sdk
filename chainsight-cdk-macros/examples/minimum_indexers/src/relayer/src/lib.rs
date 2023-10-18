use chainsight_cdk_macros::def_relayer_canister;
mod relayer;
def_relayer_canister ! ("{\"common\":{\"monitor_duration\":60,\"canister_name\":\"relayer\"},\"destination\":\"0539a0EF8e5E60891fFf0958A059E049e43020d9\",\"oracle_type\":\"uint256\",\"method_name\":\"get_last_snapshot_value\",\"canister_method_value_type\":{\"Scalar\":[\"String\",true]},\"abi_file_path\":\"__interfaces/Uint256Oracle.json\",\"lens_targets\":null}") ;
