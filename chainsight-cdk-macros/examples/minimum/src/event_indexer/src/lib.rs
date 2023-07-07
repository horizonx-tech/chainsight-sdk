use app::TransferEvent;
use chainsight_cdk::{
    indexer::{Event, Indexer, IndexingConfig},
    storage::Data,
    web3::{Web3CtxParam, Web3Indexer},
};
use chainsight_cdk_macros::{
    define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export,
    indexer_exports, manage_single_state, monitoring_canister_metrics, setup_func,
};
use ic_solidity_bindgen::{contract_abis, types::EventLog};
use ic_web3_rs::{
    ethabi::Address,
    futures::{future::BoxFuture, FutureExt},
    transports::ic_http_client::CallOptions,
};
use std::{collections::HashMap, str::FromStr};
mod app;
contract_abis!("src/event_indexer/abi");
monitoring_canister_metrics!(60);
define_web3_ctx!();
define_transform_for_web3!();
define_get_ethereum_address!();
manage_single_state!("target_addr", String, false);
setup_func!({
    proxy_canister: String,
    target_addr: String,
    web3_ctx_param: Web3CtxParam,
    config: IndexingConfig,
});
indexer_exports!(EventLog, TransferEvent, Web3Indexer);

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
