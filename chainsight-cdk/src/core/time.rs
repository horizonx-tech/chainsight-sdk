use candid::CandidType;
use serde::Deserialize;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum TimeUnit {
    Minute = 60,
    Hour = 3600,
    Day = 86400,
    Week = 604800,
}
