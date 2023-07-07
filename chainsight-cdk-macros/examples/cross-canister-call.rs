#[allow(unused_parens)]
use chainsight_cdk_macros::cross_canister_call_func;

// cross_canister_call_func!("arg_primitive", String, String);
// cross_canister_call_func!("arg_primitive_2", (String), String);
// cross_canister_call_func!("arg_tuple_with_single", (String,), String);
// cross_canister_call_func!("arg_tuple_with_double", (String, u8), String);
// cross_canister_call_func!("arg_tuple_with_triple", (String, u8, f32), String);

type SnapshotValue = (String, i32, u16, u16, u16, u16, bool);
type CallCanisterArgs = ();
type CallCanisterResponse = SnapshotValue;
cross_canister_call_func!("get_last_snapshot", CallCanisterArgs, CallCanisterResponse);

fn main() {}
