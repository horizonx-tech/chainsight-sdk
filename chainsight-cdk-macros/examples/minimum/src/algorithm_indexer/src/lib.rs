use candid::{CandidType, Principal};
use chainsight_cdk::{
    algorithm::{AlgorithmEventPersister, AlgorithmIndexer},
    indexer::Event,
    storage::{Data, Token},
};
use chainsight_cdk_macros::{
    define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export,
    manage_single_state, monitoring_canister_metrics, setup_func,
};
use ic_web3_rs::ethabi::Address;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};
monitoring_canister_metrics!(60);
define_web3_ctx!();
define_transform_for_web3!();
define_get_ethereum_address!();
manage_single_state!("target_canister", String, false);
manage_single_state!("target_addr", String, false);
manage_single_state!("proxy_canister", String, false);
setup_func!({
    target_canister: String,
    target_addr: String,
    web3_ctx_param: Web3CtxParam
});

/// This is auto-generated from yaml
//#[derive(Debug, Clone, CandidType, Default, ContractEvent, Persist)]
#[derive(Debug, Clone, CandidType, Default, Deserialize)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}

#[derive(Debug, Clone, CandidType, Default)]
pub struct SampleDest {
    from: String,
}

impl SampleDest {
    fn _tokenize(&self) -> chainsight_cdk::storage::Data {
        let mut data: HashMap<std::string::String, chainsight_cdk::storage::Token> = HashMap::new();
        data.insert("from".to_string(), Token::from(self.from.clone()));
        Data::new(data)
    }

    fn _untokenize(data: Data) -> Self {
        SampleDest {
            from: data.get("from").unwrap().to_string(),
        }
    }
}

/// This is auto-generated from yaml
impl Event<SampleDest> for SampleDest {
    fn from(event: SampleDest) -> Self
    where
        SampleDest: Into<Self>,
    {
        event.into()
    }

    fn tokenize(&self) -> chainsight_cdk::storage::Data {
        self._tokenize()
    }

    fn untokenize(data: Data) -> Self {
        SampleDest::_untokenize(data)
    }
}

async fn this_is_timer_task_entry_point() {
    indexer().index().await;
}

fn persist(input: HashMap<u64, Vec<TransferEvent>>) {}

fn indexer() -> AlgorithmIndexer<TransferEvent> {
    AlgorithmIndexer::new(
        Principal::anonymous(),
        Principal::anonymous(),
        AlgorithmEventPersister::new(persist),
    )
}
did_export!("algorithm_indexer");
