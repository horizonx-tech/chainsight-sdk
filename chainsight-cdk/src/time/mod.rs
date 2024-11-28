pub struct TimeStamper;

impl TimeStamper {
    /// returns current time nano seconds
    #[cfg(not(target_arch = "wasm32"))]
    fn _now() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64
    }
    #[cfg(target_arch = "wasm32")]
    fn _now() -> u64 {
        ic_cdk::api::time()
    }

    pub fn now_nanosec() -> u64 {
        Self::_now()
    }
    pub fn now_millisec() -> u64 {
        Self::_now() / 1_000_000
    }
    pub fn now_microsec() -> u64 {
        Self::_now() / 1_000
    }
    pub fn now_sec() -> u64 {
        Self::_now() / 1_000_000_000
    }
}
