use chainsight_cdk_macros::timer_task_func;

static HELLO: fn() -> () = || {};
timer_task_func!("set_task", "HELLO", false);

fn main() {}