use chainsight_cdk_macros::{manage_single_state, setup_func, manage_vec_state, timer_task_func, cross_canister_call_func, monitoring_canister_metrics, did_export};

monitoring_canister_metrics!(60);

manage_single_state!("target_canister", String, false);
setup_func!({
    target_canister: String
});

// storage
#[derive(Clone, candid::CandidType, candid::Deserialize)]
pub struct Snapshot {
    pub value: VirtualPrice,
    pub timestamp: u64,
}
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize)]
pub struct VirtualPrice {
    pub value: String,
    pub timestamp: u64,
}
manage_vec_state!("snapshot", Snapshot, true);

// timer task
type CallCanisterArgs = ();
type CallCanisterResponse = VirtualPrice;
cross_canister_call_func!("get_last_price", CallCanisterArgs, CallCanisterResponse);

timer_task_func!("set_task", "save_snapshot", true);
async fn save_snapshot() {
    let current_ts_sec = ic_cdk::api::time() / 1000000;
    let target_canister = candid::Principal::from_text(get_target_canister()).unwrap();
    let price = call_get_last_price(
        target_canister,
        ()
    ).await;
    if let Err(err) = price {
        ic_cdk::println!("error: {:?}", err);
        return;
    }
    let datum = Snapshot {
        value: price.unwrap().clone(),
        timestamp: current_ts_sec,
    };
    add_snapshot(datum.clone());
    ic_cdk::println!("ts={}, value={:?}", datum.timestamp, datum.value);
}

did_export!("snapshot_icp");
