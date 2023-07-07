use std::collections::HashMap;

use candid::CandidType;
use chainsight_cdk::storage::{Data, Token};
use chainsight_cdk_macros::ContractEvent;
use serde::Serialize;

/// This is auto-generated from yaml
//#[derive(Debug, Clone, CandidType, Default, ContractEvent, Persist)]
#[derive(Debug, Clone, CandidType, Default, ContractEvent, Serialize)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}

impl TransferEvent {
    pub fn _tokenize(&self) -> chainsight_cdk::storage::Data {
        let mut data: HashMap<std::string::String, chainsight_cdk::storage::Token> = HashMap::new();
        data.insert("from".to_string(), Token::from(self.from.clone()));
        data.insert("to".to_string(), Token::from(self.to.clone()));
        data.insert("value".to_string(), Token::from(self.value.clone()));
        Data::new(data)
    }

    pub fn _untokenize(data: Data) -> Self {
        TransferEvent {
            from: data.get("from").unwrap().to_string(),
            to: data.get("to").unwrap().to_string(),
            value: data.get("value").unwrap().to_u128().unwrap(),
        }
    }
}
