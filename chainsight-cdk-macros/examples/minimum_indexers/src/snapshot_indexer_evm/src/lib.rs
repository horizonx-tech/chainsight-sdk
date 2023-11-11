use chainsight_cdk_macros::def_snapshot_indexer_evm_canister;

def_snapshot_indexer_evm_canister!(
    "{
        \"common\":{
            \"canister_name\":\"snapshot_indexer_evm\"
        },
        \"method_identifier\":\"totalSupply():(uint256)\",
        \"method_args\":[],
        \"abi_file_path\":\"src/snapshot_indexer_evm/abi/ERC20.json\"
    }"
);
