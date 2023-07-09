use std::collections::HashMap;

use candid::CandidType;
use chainsight_cdk::storage::Data;
use chainsight_cdk_macros::{ContractEvent, Persist};
use serde::Serialize;

/// This is auto-generated from yaml
//#[derive(Debug, Clone, CandidType, Default, ContractEvent, Persist)]
#[derive(Debug, Clone, CandidType, Default, ContractEvent, Serialize, Persist)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}
