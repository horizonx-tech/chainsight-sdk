use candid::CandidType;
use chainsight_cdk::{
    indexer::{Event, Indexer},
    web3::Web3Indexer,
};
use chainsight_cdk_macros::{
    define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export,
    manage_single_state, monitoring_canister_metrics, setup_func,
};
use ic_solidity_bindgen::{contract_abis, types::EventLog};
use ic_web3_rs::{
    ethabi::Address,
    futures::{future::BoxFuture, FutureExt},
    transports::ic_http_client::CallOptions,
};
use std::{collections::HashMap, str::FromStr};

monitoring_canister_metrics!(60);
contract_abis!("src/indexer/abi");
define_web3_ctx!();
define_transform_for_web3!();
define_get_ethereum_address!();
manage_single_state!("target_canister", String, false);
manage_single_state!("target_addr", String, false);
setup_func!({
    target_canister: String,
    target_addr: String,
    web3_ctx_param: Web3CtxParam
});

/// This is generated from yaml
#[derive(Debug, Clone, CandidType)]
struct TransferEvent {
    from: String,
    to: String,
    value: u64,
}

impl Event for TransferEvent {
    fn from<EventLog>(log: EventLog) -> Self {
        // This is where users must implement.
        todo!()
    }
}

/// This is auto-generated from yaml
/// inputs:
/// - address(get_target_addr())
/// - contract name(ERC20)
/// - event name(event_transfer)
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

async fn this_is_timer_task_entry_point() {
    indexer().index::<TransferEvent>().await;
}

fn indexer() -> Web3Indexer {
    Web3Indexer::new(get_logs, CallOptions::default())
}
did_export!("indexer");
