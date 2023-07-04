use candid::CandidType;
use chainsight_cdk::{
    indexer::{Event, Indexer},
    storage::{Data, Token},
    web3::Web3Indexer,
};
use chainsight_cdk_macros::{
    define_get_ethereum_address, define_transform_for_web3, define_web3_ctx, did_export,
    manage_single_state, monitoring_canister_metrics, setup_func, ContractEvent, Persist,
};
use ic_solidity_bindgen::{contract_abis, types::EventLog};
use ic_web3_rs::{
    ethabi::Address,
    futures::{future::BoxFuture, FutureExt},
    transports::ic_http_client::CallOptions,
};
use std::{collections::HashMap, str::FromStr};
contract_abis!("src/indexer/abi");
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
/// This is auto-generated from yaml
//#[derive(Debug, Clone, CandidType, Default, ContractEvent, Persist)]
#[derive(Debug, Clone, CandidType, Default, ContractEvent)]
pub struct TransferEvent {
    from: String,
    to: String,
    value: u128,
}

impl TransferEvent {
    fn _tokenize(&self) -> chainsight_cdk::storage::Data {
        let mut data: HashMap<std::string::String, chainsight_cdk::storage::Token> = HashMap::new();
        data.insert("from".to_string(), Token::from(self.from.clone()));
        data.insert("to".to_string(), Token::from(self.to.clone()));
        data.insert("value".to_string(), Token::from(self.value.clone()));
        Data::new(data)
    }

    fn _untokenize(data: Data) -> Self {
        TransferEvent {
            from: data.get("from").unwrap().to_string(),
            to: data.get("to").unwrap().to_string(),
            value: data.get("value").unwrap().to_u128().unwrap(),
        }
    }
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

async fn this_is_timer_task_entry_point() {
    indexer().index::<TransferEvent>().await;
}

fn indexer() -> Web3Indexer {
    Web3Indexer::new(get_logs, None)
}
did_export!("indexer");
