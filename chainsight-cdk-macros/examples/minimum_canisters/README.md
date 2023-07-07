# minimum_canisters

```bash
cargo make did \
  && dfx stop \
  && dfx start --background --clean \
  && dfx deploy
dfx canister call example_state setup '(record { a = 5; b = 3 })'
dfx canister call example_state set_task '(30, 5)'
```