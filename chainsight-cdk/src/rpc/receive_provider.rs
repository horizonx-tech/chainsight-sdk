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

pub struct ReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    proxy: Principal,
    handle: fn() -> Out,
}
impl<Out> ReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    pub fn new(proxy: Principal, handle: fn() -> Out) -> Self {
        Self { proxy, handle }
    }
}
impl<Out> Receiver<(), Out> for ReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    fn handle(&self, _: ()) -> Out {
        (self.handle)()
    }

    fn is_from_proxy(&self) -> bool {
        ic_cdk::caller() == self.proxy
    }
}
