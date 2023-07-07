# minimum_indexers

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
