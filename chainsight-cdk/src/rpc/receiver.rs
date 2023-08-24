use async_trait::async_trait;
use candid::CandidType;
use serde::{de::DeserializeOwned, Serialize};

use super::message;

#[async_trait]
pub trait Receiver<In, Out>
where
    In: CandidType + DeserializeOwned + Send,
    Out: CandidType + Serialize + Sized + Send,
{
    async fn reply(&self, m: Vec<u8>) -> Vec<u8> {
        assert!(self.is_from_proxy());
        let parsed = message::deserialize::<In>(&m);
        match parsed {
            Ok(content) => {
                let result = self.handle(content).await;
                match message::serialize(result) {
                    Ok(reply) => reply,
                    Err(e) => {
                        ic_cdk::println!("Error: {:?}", e);
                        message::serialize("").unwrap()
                    }
                }
            }
            Err(e) => {
                ic_cdk::println!("Error: {:?}", e);
                message::serialize("").unwrap()
            }
        }
    }

    async fn handle(&self, content: In) -> Out;

    fn is_from_proxy(&self) -> bool;
}
