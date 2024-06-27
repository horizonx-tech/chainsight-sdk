use candid::{Decode, Encode};
use chainsight_cdk_macros::{did_export, prepare_stable_structure, stable_memory_for_btree_map};

prepare_stable_structure!();

#[derive(Clone, Debug, candid::CandidType, candid::Deserialize, serde :: Serialize)]
pub struct Snapshot {
    value: String,
    timestamp: u64,
}
impl ic_stable_structures::Storable for Snapshot {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

stable_memory_for_btree_map!("snapshot", Snapshot, 0, true);

// Function with dependencies
fn proxy() -> candid::Principal {
    candid::Principal::anonymous()
}

did_export!("interface");
