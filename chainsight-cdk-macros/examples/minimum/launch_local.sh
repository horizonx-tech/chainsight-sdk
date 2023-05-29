dfx stop && dfx start --background --clean && dfx deploy

dfx canister call snapshot_chain setup "(
  \"bEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7\",
  record {
    url = \"<YOUR_RPC_URL_ETHEREUM_MAINNET>\";
    from = null;
    chain_id = 1;
    key = variant { LocalDevelopment };
  }
)"
dfx canister call snapshot_chain set_task '(15, 0)'

dfx canister call snapshot_icp setup "(\"$(dfx canister id snapshot_chain)\")"
dfx canister call snapshot_icp set_task '(30, 5)'

dfx canister call relayer setup "(
  \"$(dfx canister id snapshot_chain)\",
  \"E5f0DA5761B82e14E45021246EE657D07a9BBd23\",
  record {
    url = \"<YOUR_RPC_URL_POLYGON_MUMBAI>\";
    from = null;
    chain_id = 80001;
    key = variant { LocalDevelopment };
  }
)"
dfx canister call relayer get_ethereum_address "(variant { LocalDevelopment })"
# pre: need to transfer gas to this canister's address
dfx canister call relayer set_task '(30, 5)'

