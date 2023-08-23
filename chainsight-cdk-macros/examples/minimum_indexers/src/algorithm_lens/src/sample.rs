// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
use candid::{self, CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize)]
pub struct Body {
    data: Box<BodyData>,
}

#[derive(CandidType, Deserialize)]
pub struct BodyData {
    data: Box<CurrencyData>,
}

#[derive(CandidType, Deserialize)]
pub struct CanisterMetricsSnapshot {
    cycles: candid::Nat,
    timestamp: u64,
}

#[derive(CandidType, Deserialize)]
pub struct CurrencyData {
    quote: Box<Quote>,
}

#[derive(CandidType, Deserialize)]
pub struct Data {
    body: Body,
}

#[derive(CandidType, Deserialize)]
pub enum Env {
    Production,
    Test,
    LocalDevelopment,
}

#[derive(CandidType, Deserialize)]
pub enum InitError {
    InvalidDestination(String),
    InvalidPrincipal(Principal),
    InvalidContent(String),
    InvalidRequest(String),
}

#[derive(CandidType, Deserialize)]
pub struct Quote {
    usd: Box<QuoteData>,
}

#[derive(CandidType, Deserialize)]
pub struct QuoteData {
    volume_24h: f64,
    price: f64,
}

#[derive(CandidType, Deserialize)]
 enum Result {
    Ok,
    Err(InitError),
}

#[derive(CandidType, Deserialize)]
pub struct Snapshot {
    value: Box<SnapshotValue>,
    timestamp: u64,
}

#[derive(CandidType, Deserialize)]
pub struct SnapshotValue {
    data: Data,
}
