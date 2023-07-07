use candid::{CandidType, Principal};
use serde::{de::DeserializeOwned, Serialize};

use super::receiver::Receiver;

pub struct ReceiverProvider<In, Out>
where
    In: CandidType + DeserializeOwned,
    Out: CandidType + Serialize,
{
    proxy: Principal,
    handle: fn(In) -> Out,
}

impl<In, Out> ReceiverProvider<In, Out>
where
    In: CandidType + DeserializeOwned,
    Out: CandidType + Serialize,
{
    pub fn new(proxy: Principal, handle: fn(In) -> Out) -> Self {
        Self { proxy, handle }
    }
}

impl<In, Out> Receiver<In, Out> for ReceiverProvider<In, Out>
where
    In: CandidType + DeserializeOwned,
    Out: CandidType + Serialize,
{
    fn handle(&self, content: In) -> Out {
        (self.handle)(content)
    }

    fn is_from_proxy(&self) -> bool {
        ic_cdk::caller() == self.proxy
    }
}
