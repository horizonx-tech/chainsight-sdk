use std::collections::HashMap;

use crate::TransferEvent;

pub fn persist(elem: HashMap<u64, Vec<TransferEvent>>) {
    ic_cdk::println!("persisting");
    elem.iter()
        .for_each(|(k, _)| ic_cdk::println!("key: {}", k));
}
