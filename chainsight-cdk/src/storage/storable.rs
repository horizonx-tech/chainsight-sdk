use std::borrow::Cow;

use ic_stable_structures::{BoundedStorable, Storable};

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    PartialOrd,
    candid::CandidType,
    candid::Deserialize,
    serde::Serialize,
)]
pub struct StorableBool(pub bool);

impl Storable for StorableBool {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let num: u8 = if self.0 { 1 } else { 0 };
        Cow::Owned(num.to_be_bytes().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let num = u8::from_be_bytes(bytes.as_ref().try_into().unwrap());
        match num {
            0 => StorableBool(false),
            1 => StorableBool(true),
            _ => panic!("Invalid bool encoding: expected 0 or 1, found {}", num),
        }
    }
}
impl BoundedStorable for StorableBool {
    const MAX_SIZE: u32 = 1;
    const IS_FIXED_SIZE: bool = true;
}
impl From<StorableBool> for bool {
    fn from(storable_bool: StorableBool) -> Self {
        storable_bool.0
    }
}
impl From<bool> for StorableBool {
    fn from(bool: bool) -> Self {
        StorableBool(bool)
    }
}
