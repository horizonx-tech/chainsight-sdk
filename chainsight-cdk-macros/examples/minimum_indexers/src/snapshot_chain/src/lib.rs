use chainsight_cdk_macros::{
    chainsight_common, define_transform_for_web3, define_web3_ctx, did_export, init_in,
    manage_single_state, manage_vec_state, setup_func, snapshot_web3_source, timer_task_func,
};
use ic_web3_rs::types::Address;
use serde::Serialize;
use std::str::FromStr;
chainsight_common!(60);
ic_solidity_bindgen::contract_abi!("./src/snapshot_chain/abi/StableSwap.json");
define_web3_ctx!();
define_transform_for_web3!();
manage_single_state!("target_addr", String, false);
setup_func!({
    target_addr: String,
    web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
});
init_in!();
// storage
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize, Serialize)]
pub struct VirtualPrice {
    pub value: String,
    pub timestamp: u64,
}
manage_vec_state!("price", VirtualPrice, true);
snapshot_web3_source!("get_virtual_price");
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
