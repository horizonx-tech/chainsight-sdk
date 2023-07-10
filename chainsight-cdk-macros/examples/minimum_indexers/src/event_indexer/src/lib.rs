use candid::CandidType;
use chainsight_cdk::{
    indexer::{Event, Indexer, IndexingConfig},
    storage::Data,
    web3::Web3CtxParam,
};
use chainsight_cdk_macros::{
    define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export, init_in,
    manage_single_state, monitoring_canister_metrics, setup_func, timer_task_func,
    web3_event_indexer, ContractEvent, Persist,
};
use ic_solidity_bindgen::{contract_abis, types::EventLog};
use ic_web3_rs::{
    ethabi::Address,
    futures::{future::BoxFuture, FutureExt},
    transports::ic_http_client::CallOptions,
};
use serde::Serialize;
use std::{collections::HashMap, str::FromStr};
contract_abis!("src/event_indexer/abi");
monitoring_canister_metrics!(60);
define_web3_ctx!();
define_transform_for_web3!();
define_get_ethereum_address!();
manage_single_state!("target_addr", String, false);
setup_func!({
    target_addr: String,
    web3_ctx_param: Web3CtxParam,
    config: IndexingConfig,
});
init_in!();
timer_task_func!("set_task", "index", true);

web3_event_indexer!(TransferEvent);

fn get_logs(
    from: u64,
    to: u64,
    call_options: CallOptions,
) -> BoxFuture<'static, Result<HashMap<u64, Vec<EventLog>>, chainsight_cdk::indexer::Error>> {
    async move {
        let address = Address::from_str(get_target_addr().as_str()).unwrap();
        let instance = ERC20::new(address, &web3_ctx().unwrap());
        let result = instance.event_transfer(from, to, call_options);
        match result.await {
            Ok(logs) => Ok(logs),
            Err(e) => Err(chainsight_cdk::indexer::Error::OtherError(e.to_string())),
        }
    }
    .boxed()
}

/// This is auto-generated from yaml
#[derive(Debug, Clone, CandidType, Default, ContractEvent, Serialize, Persist)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}

/// This is auto-generated from yaml
impl Event<EventLog> for TransferEvent {
    fn from(event: EventLog) -> Self
    where
        EventLog: Into<Self>,
    {
        event.into()
    }

    fn tokenize(&self) -> chainsight_cdk::storage::Data {
        self._tokenize()
    }

    fn untokenize(data: Data) -> Self {
        TransferEvent::_untokenize(data)
    }
}
did_export!("event_indexer");
