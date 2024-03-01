use candid::{self, Decode, Encode};
#[derive(
    Clone,
    Debug,
    candid :: CandidType,
    candid :: Deserialize,
    serde :: Serialize,
    chainsight_cdk_macros :: StableMemoryStorable,
)]
pub struct ResponseType {
    pub value: String,
    pub timestamp: u64,
}

pub type CallCanisterResponse = ResponseType;
pub type CallCanisterArgs = ();
pub fn call_args() -> CallCanisterArgs {
    ()
}
pub fn filter(_: &CallCanisterResponse, _: Vec<CallCanisterResponse>) -> bool {
    true
}
