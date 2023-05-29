use chainsight_cdk_macros::cross_canister_call_func;
use candid::Principal;

type _CallCanisterArgs = (String, String);
type _CallCanisterResponse = String;

cross_canister_call_func!("greet", _CallCanisterArgs, _CallCanisterResponse);

fn main() {}