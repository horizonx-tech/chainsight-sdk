use chainsight_cdk_macros::{chainsight_common, init_in, timer_task_func};

#[allow(dead_code)]
static HELLO: fn() -> () = || {};
init_in!();
chainsight_common!(0);
timer_task_func!("set_task", "HELLO");

fn main() {}
