use chainsight_cdk_macros::def_relayer_canister;
mod example_canister; // NOTE: logics

def_relayer_canister!(
    "{
        \"common\":{
            \"canister_name\":\"example_canister\"
        },
        \"destination\":\"0539a0EF8e5E60891fFf0958A059E049e43020d9\",
        \"method_identifier\":\"get_last_snapshot : () -> (record { value : text; timestamp : nat64; })\",
        \"conversion_parameter\":{
            \"extracted_field\": \"value\",
            \"destination_type_to_convert\": \"U256\",
            \"exponent_of_power10\": 18
        },
        \"abi_file_path\":\"__interfaces/Uint256Oracle.json\",
        \"lens_targets\":null,
        \"method_name\":\"update_state\"
    }"
);
