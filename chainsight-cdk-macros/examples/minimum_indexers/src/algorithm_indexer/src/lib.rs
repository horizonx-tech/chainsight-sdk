use candid::CandidType;
use chainsight_cdk::{
    indexer::{Event, IndexingConfig},
    storage::Data,
};
use chainsight_cdk_macros::{
    algorithm_indexer, did_export, init_in, manage_single_state, monitoring_canister_metrics,
    setup_func, Persist,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
monitoring_canister_metrics!(60);
init_in!();
manage_single_state!("target_addr", String, false);

setup_func!({
    target_addr: String,
    config: IndexingConfig
});
use chainsight_cdk::indexer::Indexer;
algorithm_indexer!(TransferEvent, SampleDest);

/// This is auto-generated from yaml
//#[derive(Debug, Clone, CandidType, Default, ContractEvent, Persist)]
#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}

#[derive(Debug, Clone, CandidType, Default, Deserialize, Serialize, Persist)]
pub struct SampleDest {
    from: String,
}

impl Event<TransferEvent> for SampleDest {
    fn from(_event: TransferEvent) -> Self {
        Self::default()
    }

    fn tokenize(&self) -> chainsight_cdk::storage::Data {
        self._tokenize()
    }

    fn untokenize(data: Data) -> Self {
        SampleDest::_untokenize(data)
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

did_export!("algorithm_indexer");
