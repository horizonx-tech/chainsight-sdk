use candid::{Decode, Encode};
use chainsight_cdk_macros::{
    prepare_stable_structure,
    stable_memory_for_scalar,
    stable_memory_for_vec,
    StableMemoryStorable,
    did_export
};

prepare_stable_structure!();
stable_memory_for_scalar!("timestamp", u64, 0, true);
stable_memory_for_scalar!("price", u128, 1, true);
stable_memory_for_vec!("year", u16, 2, true);
stable_memory_for_vec!("score", u128, 3, true);

#[derive(
    Clone,
    Debug,
    candid::CandidType,
    candid::Deserialize,
    serde :: Serialize,
    StableMemoryStorable
)]
#[stable_mem_storable_opts(max_size = 100, is_fixed_size = false)]
pub struct UserData {
    name: String,
    age: i32,
    is_student: bool
}
stable_memory_for_vec!("user", UserData, 4, true);

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_timestamp(value: u64) {
    set_timestamp(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_price(value: u128) {
    set_price(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn insert_year(value: u16) {
    add_year(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn insert_score(value: u128) {
    add_score(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn insert_user(value: UserData) {
    add_user(value)
}

// Function with dependencies
fn proxy() -> candid::Principal {
    candid::Principal::anonymous()
}

did_export!("interface");
