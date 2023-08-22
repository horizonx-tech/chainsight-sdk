use chainsight_cdk::rpc::{CallProvider, Caller, Message};
use chainsight_cdk_macros::{
    chainsight_common, define_get_ethereum_address, define_transform_for_web3, define_web3_ctx,
    did_export, init_in, manage_single_state, relayer_source, setup_func, timer_task_func,
};
use ic_web3_rs::types::{Address, U256};
use std::str::FromStr;
chainsight_common!(60);
ic_solidity_bindgen::contract_abi!("./src/relayer/abi/Oracle.json");
define_web3_ctx!();
define_transform_for_web3!();
define_get_ethereum_address!();
manage_single_state!("target_canister", String, false);
manage_single_state!("target_addr", String, false);
setup_func!({
    target_canister: String,
    target_addr: String,
    web3_ctx_param: chainsight_cdk::web3::Web3CtxParam
});
init_in!();
relayer_source!();

#[derive(Debug, Clone, candid::CandidType, candid::Deserialize)]
pub struct VirtualPrice {
    pub value: String,
    pub timestamp: u64,
}

type CallCanisterArgs = ();
type CallCanisterResponse = VirtualPrice;
timer_task_func!("set_task", "sync", true);
async fn sync() {
    let target_canister = candid::Principal::from_text(get_target_canister()).unwrap();
    let call_result = CallProvider::new(_get_target_proxy(target_canister).await)
        .call(
            Message::new::<CallCanisterArgs>((), target_canister.clone(), "get_last_price")
                .unwrap(),
        )
        .await;
    if let Err(err) = call_result {
        ic_cdk::println!("error: {:?}", err);
        return;
    }

    let price = call_result.unwrap().reply::<CallCanisterResponse>();
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
