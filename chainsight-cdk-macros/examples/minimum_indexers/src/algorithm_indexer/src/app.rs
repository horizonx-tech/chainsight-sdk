use std::collections::HashMap;

use crate::{Account, TotalSupply, TransferEvent};

pub fn persist(elem: HashMap<u64, Vec<TransferEvent>>) {
    get_val();
    elem.iter().for_each(|(k, v)| {
        let accounts: Vec<Account> = v
            .iter()
            .map(|e| {
                let mut new_account = Account::default();
                new_account.value = e.to.clone();
                new_account
            })
            .collect();
        Account::put(k.to_string().as_str(), accounts);
    })
}

fn get_val() -> TotalSupply {
    let store = TotalSupply::get_store();
    match store.get("TotalSupply") {
        Some(val) => val,
        None => TotalSupply::default(),
    }
}
