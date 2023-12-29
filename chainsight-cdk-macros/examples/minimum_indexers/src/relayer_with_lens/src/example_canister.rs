pub type CallCanisterResponse = String;
pub fn filter(_: &CallCanisterResponse) -> bool {
    true
}

pub type CallCanisterArgs = String;
pub fn call_args() -> CallCanisterArgs {
    "".to_string()
}
