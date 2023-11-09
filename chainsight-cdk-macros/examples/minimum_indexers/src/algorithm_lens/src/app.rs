#[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]
pub struct CalculateArgs {
    pub index: u64,
    pub target: String
}

#[derive(Clone, Debug, Default, candid::CandidType, serde::Deserialize, serde::Serialize)]
pub struct LensValue {
    pub dummy: u64,
}
pub async fn calculate(_targets: Vec<String>, _args: CalculateArgs) -> LensValue {
    //    let _result = get_last_snapshot_value(targets.get(0).unwrap().clone()).await;
    todo!()
}
