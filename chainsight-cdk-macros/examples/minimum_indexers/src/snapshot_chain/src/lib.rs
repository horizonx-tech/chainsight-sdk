use chainsight_cdk_macros::{
    define_transform_for_web3, define_web3_ctx, did_export, manage_single_state, manage_vec_state,
    monitoring_canister_metrics, setup_func, timer_task_func,
};
use ic_web3_rs::types::Address;
use std::str::FromStr;
monitoring_canister_metrics!(60);
ic_solidity_bindgen::contract_abi!("./src/snapshot_chain/abi/StableSwap.json");
define_web3_ctx!();
define_transform_for_web3!();
manage_single_state!("target_addr", String, false);
setup_func!({
    target_addr: String,
    web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
});

// storage
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize)]
pub struct VirtualPrice {
    pub value: String,
    pub timestamp: u64,
}
manage_vec_state!("price", VirtualPrice, true);

// timer task
timer_task_func!("set_task", "get_virtual_price", true);
async fn get_virtual_price() {
    let current_ts_sec = ic_cdk::api::time() / 1000000;
    let price = StableSwap::new(
        Address::from_str(&get_target_addr()).unwrap(),
        &web3_ctx().unwrap(),
    )
    .get_virtual_price(None)
    .await
    .unwrap();
    let datum = VirtualPrice {
        value: price.to_string(),
        timestamp: current_ts_sec,
    };
    add_price(datum.clone());
    ic_cdk::println!("ts={}, price={}", datum.timestamp, datum.value);
}

did_export!("snapshot_chain");
