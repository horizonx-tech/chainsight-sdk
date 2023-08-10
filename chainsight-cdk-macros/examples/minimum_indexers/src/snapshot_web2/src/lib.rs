use chainsight_cdk::web2::{JsonRpcSnapshotParam, Web2JsonRpcSnapshotIndexer};
use chainsight_cdk_macros::{
    chainsight_common, did_export, init_in, manage_vec_state, timer_task_func,
};
init_in!();
chainsight_common!(60);
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize)]
pub struct Price {
    pub usd: f64,
    pub usd_24h_vol: f64,
}
#[derive(Debug, Clone, candid::CandidType, candid::Deserialize, serde::Serialize)]
pub struct PriceResult {
    pub oasys: Price,
}
manage_vec_state!("price_result", PriceResult, true);
timer_task_func!("set_task", "index", true);

async fn index() {
    let indexer = Web2JsonRpcSnapshotIndexer::new(
        "https://api.coingecko.com/api/v3/simple/price".to_string(),
    );

    let res = indexer
        .get::<String, PriceResult>(JsonRpcSnapshotParam {
            // TODO: pricvate api key
            headers: vec![].into_iter().collect(),
            queries: vec![
                ("ids".to_string(), "oasys".to_string()),
                ("vs_currencies".to_string(), "usd".to_string()),
                ("include_24hr_vol".to_string(), "true".to_string()),
                ("precision".to_string(), "18".to_string()),
            ]
            .into_iter()
            .collect(),
        })
        .await
        .unwrap();
    ic_cdk::println!("res={:?}", res.oasys.usd_24h_vol);
    ic_cdk::println!("res={:?}", res.oasys.usd);
}

did_export!("snapshot_web2");
