use chainsight_cdk_macros::{
    prepare_stable_structure,
    stable_memory_for_scalar,
    stable_memory_for_vec,
    did_export
};

prepare_stable_structure!();
stable_memory_for_scalar!("timestamp", u64, 0, true);
stable_memory_for_scalar!("price", u128, 1, true);
stable_memory_for_vec!("year", u16, 2, true);
stable_memory_for_vec!("score", u128, 3, true);

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_timestamp(value: u64) -> Result<(), String>{
    set_timestamp(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_price(value: u128) -> Result<(), String>{
    set_price(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn insert_year(value: u16) -> Result<(), String>{
    add_year(value)
}

#[ic_cdk::update]
#[candid::candid_method(update)]
fn insert_score(value: u128) -> Result<(), String>{
    add_score(value)
}

did_export!("example_stable_memory");
