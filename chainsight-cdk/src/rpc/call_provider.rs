use async_trait::async_trait;
use ic_cdk::api::call::{self, CallResult};

use crate::{
    rpc::caller::Caller,
    rpc::message::{Message, MessageCallResult, MessageResult},
};

pub struct CallProvider {}

impl CallProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Caller for CallProvider {
    async fn call(&self, m: Message) -> CallResult<MessageResult> {
        let result: MessageCallResult =
            call::call(m.recipient, "proxy_call", (m.method_name, m.content)).await;
        match result {
            Ok(result) => match result.0 {
                Ok(result) => Ok(MessageResult::new(result.0)),
                Err(err) => {
                    ic_cdk::println!("Error: {:?}", err);
                    Err(err)
                }
            },
            Err(err) => Err(err),
        }
    }
}
