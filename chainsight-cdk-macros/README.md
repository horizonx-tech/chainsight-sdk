# chainsight-cdk-macros

A set of macros to develop canisters for Chainsight Platform.

Provides entry point macros for building canisters according to Chainsight's Component Type and macros that are parts of those canisters.

- Entrypoint
  - `def_{component_type}!( ... )`
- Storage
  - General-purpose single or variable-length storage using Stable memory
- Initialization
  - Integration with Management Canister to be on the Chainsight Platform
- Periodic execution using Canister Timers
- Others
  - Generate .did interface
- etc...


## Example

```rust
use chainsight_cdk_macros::def_snapshot_indexer_evm_canister;

def_snapshot_indexer_evm_canister!(
    "{
        \"common\":{
            \"canister_name\":\"example_canister\"
        },
        \"method_identifier\":\"totalSupply():(uint256)\",
        \"method_args\":[],
        \"abi_file_path\":\"src/snapshot_indexer_evm/abi/ERC20.json\"
    }"
);
```
