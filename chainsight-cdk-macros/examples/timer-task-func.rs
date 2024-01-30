use candid::{Decode, Encode};
use chainsight_cdk_macros::{
    chainsight_common, init_in, manage_single_state, timer_task_func, StableMemoryStorable,
};

#[allow(dead_code)]
static HELLO: fn() -> () = || {};
init_in!();
chainsight_common!();
timer_task_func!("set_task", "HELLO");

fn main() {}
