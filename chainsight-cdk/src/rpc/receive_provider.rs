use candid::{CandidType, Principal};
use futures::future::BoxFuture;
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

pub struct AsyncReceiverProvider<In, Out>
where
    In: CandidType + DeserializeOwned,
    Out: CandidType + Serialize,
{
    proxy: Principal,
    handle: fn(In) -> BoxFuture<'static, Out>,
}

pub struct ReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    proxy: Principal,
    handle: fn() -> Out,
}

pub struct AsyncReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    proxy: Principal,
    handle: fn() -> BoxFuture<'static, Out>,
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

impl<In, Out> AsyncReceiverProvider<In, Out>
where
    In: CandidType + DeserializeOwned,
    Out: CandidType + Serialize,
{
    pub fn new(proxy: Principal, handle: fn(In) -> BoxFuture<'static, Out>) -> Self {
        Self { proxy, handle }
    }
}
impl<In, Out> Receiver<In, Out> for AsyncReceiverProvider<In, Out>
where
    In: CandidType + DeserializeOwned,
    Out: CandidType + Serialize,
{
    fn handle(&self, content: In) -> Out {
        futures::executor::block_on((self.handle)(content))
    }

    fn is_from_proxy(&self) -> bool {
        ic_cdk::caller() == self.proxy
    }
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

impl<Out> AsyncReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    pub fn new(proxy: Principal, handle: fn() -> BoxFuture<'static, Out>) -> Self {
        Self { proxy, handle }
    }
}
impl<Out> Receiver<(), Out> for AsyncReceiverProviderWithoutArgs<Out>
where
    Out: CandidType + Serialize,
{
    fn handle(&self, _: ()) -> Out {
        futures::executor::block_on((self.handle)())
    }

    fn is_from_proxy(&self) -> bool {
        ic_cdk::caller() == self.proxy
    }
}
