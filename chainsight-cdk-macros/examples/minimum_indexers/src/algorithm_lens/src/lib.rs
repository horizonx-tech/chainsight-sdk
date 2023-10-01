use candid::CandidType;
use chainsight_cdk::lens::LensFinder;
use chainsight_cdk_macros::{
    algorithm_lens_finder, chainsight_common, did_export, init_in, lens_method,
};
use ic_web3_rs::futures::{future::BoxFuture, FutureExt};
mod app;
use app::*;
init_in!();
chainsight_common!(60);
#[derive(Clone, Debug, Default, CandidType, serde::Deserialize, serde::Serialize)]
pub struct Account {
    pub address: String,
}
mod sample;
use sample::*;
algorithm_lens_finder!(
    "last_snapshot_value",
    "proxy_get_last_snapshot_value",
    String
);
async fn calculate(_targets: Vec<String>) -> LensValue {
    todo!()
}
lens_method!(1usize);
did_export!("sample_algorithm_lens");
