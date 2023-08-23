use crate::rpc::{caller::Caller, Error};
use async_trait::async_trait;
use candid::Principal;

use crate::rpc::{CallProvider, Message};

#[derive(serde::Serialize, Clone)]
pub struct LensTarget<Resp>
where
    Resp: serde::de::DeserializeOwned,
{
    _phantom: std::marker::PhantomData<Resp>,
    pub target: Principal,
    pub method: String,
}

impl<Resp> LensTarget<Resp>
where
    Resp: serde::de::DeserializeOwned,
{
    pub fn new(target: Principal, method: &str) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            target,
            method: method.to_string(),
        }
    }
}

#[async_trait]
pub trait LensFinder<Resp>
where
    Resp: serde::de::DeserializeOwned,
{
    async fn find<Args>(&self, args: Args) -> Result<Resp, Error>
    where
        Args: serde::Serialize + Send;

    async fn find_unwrap<Args>(&self, args: Args) -> Resp
    where
        Args: serde::Serialize + Send;
}

pub struct AlgorithmLensFinder<Resp>
where
    Resp: serde::de::DeserializeOwned,
{
    pub target: LensTarget<Resp>,
}

impl<Resp> AlgorithmLensFinder<Resp>
where
    Resp: serde::de::DeserializeOwned,
{
    pub fn new(target: LensTarget<Resp>) -> Self {
        Self { target }
    }
}

#[async_trait]
impl<Resp> LensFinder<Resp> for AlgorithmLensFinder<Resp>
where
    Resp: serde::de::DeserializeOwned + Send + Sync,
{
    async fn find<Args>(&self, args: Args) -> Result<Resp, Error>
    where
        Args: serde::Serialize + Send,
        Resp: serde::de::DeserializeOwned,
    {
        let call_result = CallProvider::new()
            .call(
                Message::new::<Args>(args, self.target.target, self.target.method.as_str())
                    .unwrap(),
            )
            .await;
        if let Err(err) = call_result {
            ic_cdk::println!("error: {:?}", err);
            return Err(Error::InvalidRequest(err.1));
        }
        let resp = call_result.unwrap().reply::<Resp>();
        if let Err(err) = resp {
            ic_cdk::println!("error: {:?}", err);
            return Err(Error::InvalidRequest(stringify!(err).to_string()));
        }
        Ok(resp.unwrap())
    }

    async fn find_unwrap<Args>(&self, args: Args) -> Resp
    where
        Args: serde::Serialize + Send,
        Resp: serde::de::DeserializeOwned,
    {
        self.find(args).await.unwrap()
    }
}
