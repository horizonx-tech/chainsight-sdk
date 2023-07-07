use chainsight_cdk::web3::Web3CtxParam;
use chainsight_cdk_macros::{
    cross_canister_call_func, define_get_ethereum_address, define_transform_for_web3,
    define_web3_ctx, did_export, manage_single_state, monitoring_canister_metrics, setup_func,
    timer_task_func,
};
use ic_web3_rs::types::{Address, U256};
use std::str::FromStr;
monitoring_canister_metrics!(60);
ic_solidity_bindgen::contract_abi!("./src/relayer/abi/Oracle.json");
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

#[derive(Debug, Clone, candid::CandidType, candid::Deserialize)]
pub struct VirtualPrice {
    pub value: String,
    pub timestamp: u64,
}

// timer task
type CallCanisterArgs = ();
type CallCanisterResponse = VirtualPrice;
cross_canister_call_func!("get_last_price", CallCanisterArgs, CallCanisterResponse);

timer_task_func!("set_task", "sync", true);
async fn sync() {
    let target_canister = candid::Principal::from_text(get_target_canister()).unwrap();
    let price = call_get_last_price(target_canister, ()).await;
    if let Err(err) = price {
        ic_cdk::println!("error: {:?}", err);
        return;
    }
    let datum = price.unwrap();

    // temp: set gas_price, nonce
    ic_cdk::println!(
        "before sync: ts={}, value={:?}",
        &datum.timestamp,
        &datum.value
    );
    Oracle::new(
        Address::from_str(&get_target_addr()).unwrap(),
        &web3_ctx().unwrap(),
    )
    .update_state(U256::from_str(&datum.value).unwrap(), None)
    .await
    .unwrap();
    ic_cdk::println!(
        "after sync: ts={}, value={:?}",
        &datum.timestamp,
        &datum.value
    );
}

did_export!("relayer");
