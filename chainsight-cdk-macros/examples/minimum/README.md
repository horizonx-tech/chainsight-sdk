# minimum

## indexers

```bash
./launch_local.sh
```

- canisters
  - snapshot_chain: call Curve's StableSwap get_virtual_price
  - snapshot_icp: call snapshot_chain & save virtual_price to storage
  - relayer: call snapshot_chain & save virtual_price to oracle (in other chain)
- relations

```txt
                         /- snapshot_icp
contract - snapshot_chain
                         \- relayer - oracle
```

## example_canister

```bash
# pre: modify dfx.json
dfx stop && dfx start --background --clean && dfx deploy example_canister
dfx canister call example_canister setup '(record { a = 5; b = 3 })'
dfx canister call example_canister set_task '(30, 5)'
```
