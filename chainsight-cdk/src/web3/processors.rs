use async_trait::async_trait;
use ic_cdk::api::management_canister::http_request::{
    HttpResponse, TransformArgs, TransformContext, TransformFunc,
};
use ic_web3_rs::{
    api::Eth,
    contract::Options,
    transports::{
        ic_http_client::{CallOptions, CallOptionsBuilder},
        ICHttp,
    },
    types::U64,
    Transport, Web3,
};
use std::future::Future;

use serde_json::{json, Value};

const RPC_CALL_MAX_RETRY: u32 = 5;
#[async_trait]
pub trait TransactionOptionBuilder {
    async fn build(&self) -> anyhow::Result<Option<Options>>;
}

pub struct EVMTransactionOptionBuilder {
    pub url: String,
    pub chain_id: u64,
    pub key_name: String,
}

impl EVMTransactionOptionBuilder {
    pub fn new(url: String, chain_id: u64, key_name: String) -> Self {
        Self {
            url,
            chain_id,
            key_name,
        }
    }

    async fn with_retry<T, E, Fut, F: FnMut() -> Fut>(&self, mut f: F) -> Result<T, E>
    where
        Fut: Future<Output = Result<T, E>>,
    {
        let mut count = 0;
        loop {
            let result = f().await;

            if result.is_ok() {
                break result;
            } else {
                if count > RPC_CALL_MAX_RETRY {
                    break result;
                }
                count += 1;
            }
        }
    }
    async fn supports_eip1559(&self) -> anyhow::Result<bool> {
        let processor = EIP1559SupportProcessor;
        let http = ICHttp::new(&self.url, None)?;
        let include_txs = self.serialize(&false)?;
        let block_num = self.serialize(&ic_web3_rs::types::BlockNumber::Latest)?;
        let call_options = CallOptionsBuilder::default()
            .max_resp(None)
            .cycles(None)
            .transform(Some(processor.context()))
            .build()?;
        let execution = http
            .execute(
                "eth_getBlockByNumber",
                vec![block_num, include_txs],
                call_options,
            )
            .await?;
        // expected json: {"eip1559": true} or {"eip1559": false}
        ic_cdk::println!("eip1559_support: {:?}", execution);
        let result = execution.get("eip1559").unwrap().as_bool().unwrap();
        Ok(result)
    }

    fn serialize<T: serde::Serialize>(&self, t: &T) -> anyhow::Result<Value> {
        let v = serde_json::to_value(t);
        Ok(v?)
    }

    fn eth(&self) -> Eth<ICHttp> {
        let http = ICHttp::new(&self.url, None).unwrap();
        let web3 = Web3::new(http);
        web3.eth()
    }
}

#[async_trait]
impl TransactionOptionBuilder for EVMTransactionOptionBuilder {
    async fn build(&self) -> anyhow::Result<Option<Options>> {
        let supports_eip1559 = self.supports_eip1559().await?;
        if supports_eip1559 {
            return Ok(None);
        }
        let gas_price = self
            .with_retry(|| self.eth().gas_price(CallOptions::default()))
            .await?;
        let public_key = ic_web3_rs::ic::get_public_key(
            None,
            vec![ic_cdk::id().as_slice().to_vec()],
            self.key_name.clone(),
        )
        .await
        .unwrap();
        let ethereum_address = ic_web3_rs::ic::pubkey_to_address(&public_key).unwrap();
        let nonce = self
            .with_retry(|| {
                self.eth()
                    .transaction_count(ethereum_address, None, CallOptions::default())
            })
            .await?;
        Ok(Some(Options::with(|op| {
            op.gas_price = Some(gas_price);
            op.nonce = Some(nonce);
            op.transaction_type = Some(U64::from(0))
        })))
    }
}

pub trait TransformProcessor {
    fn transform(&self, raw: TransformArgs) -> HttpResponse {
        let mut res = HttpResponse {
            status: raw.response.status.clone(),
            ..Default::default()
        };
        if res.status == 200 {
            res.body = self.process_body(&raw.response.body);
        } else {
            ic_cdk::api::print(format!(
                "Received an error from blockchain: err = {:?}",
                raw
            ));
        }
        res
    }
    fn process_body(&self, body: &[u8]) -> Vec<u8>;
    fn context(&self) -> TransformContext;
}

pub struct EIP1559SupportProcessor;
impl TransformProcessor for EIP1559SupportProcessor {
    fn process_body(&self, body: &[u8]) -> Vec<u8> {
        let mut body: Value = serde_json::from_slice(body).unwrap();
        let elements = body.get_mut("result").unwrap().as_object_mut().unwrap();
        let contains_base_fee_per_gas = elements.get_mut("baseFeePerGas").is_some();
        let result: Value = json!({"eip1559": contains_base_fee_per_gas});
        serde_json::to_vec(&result).unwrap()
    }
    fn context(&self) -> TransformContext {
        TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform_eip1559_support".to_string(),
            }),
            context: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_eip_1559_support_processor_true() {
        let processor = EIP1559SupportProcessor;
        let raw = TransformArgs {
            response: HttpResponse {
                status: candid::Nat::from(200),
                headers: vec![],
                body: r#"{
                    "jsonrpc": "2.0",
                    "id": 0,
                    "result": {
                      "number": "0x1b4",
                      "difficulty": "0x4ea3f27bc",
                      "extraData": "0x476574682f4c5649562f76312e302e302f6c696e75782f676f312e342e32",
                      "gasLimit": "0x1388",
                      "gasUsed": "0x0",
                      "hash": "0xdc0818cf78f21a8e70579cb46a43643f78291264dda342ae31049421c82d21ae",
                      "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                      "miner": "0xbb7b8287f3f0a933474a79eae42cbca977791171",
                      "mixHash": "0x4fffe9ae21f1c9e15207b1f472d5bbdd68c9595d461666602f2be20daf5e7843",
                      "nonce": "0x689056015818adbe",
                      "parentHash": "0xe99e022112df268087ea7eafaf4790497fd21dbeeb6bd7a1721df161a6657a54",
                      "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                      "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                      "size": "0x220",
                      "stateRoot": "0xddc8b0234c2e0cad087c8b389aa7ef01f7d79b2570bccb77ce48648aa61c904d",
                      "timestamp": "0x55ba467c",
                      "totalDifficulty": "0x78ed983323d",
                      "transactions": [],
                      "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                      "uncles": [],
                      "baseFeePerGas": "0x19aefc00"
                    }
                  }"#.into(),
            },
            context: vec![],
        };
        let res = processor.transform(raw);
        assert_eq!(
            res.body,
            serde_json::to_vec(&json!({"eip1559": true})).unwrap()
        );
    }
    #[test]
    fn test_eip_1559_support_processor_false() {
        let processor = EIP1559SupportProcessor;
        let raw = TransformArgs {
            response: HttpResponse {
                status: candid::Nat::from(200),
                headers: vec![],
                body: r#"{
                    "jsonrpc": "2.0",
                    "id": 0,
                    "result": {
                    "number": "0x1b4",
                    "difficulty": "0x4ea3f27bc",
                    "extraData": "0x476574682f4c5649562f76312e302e302f6c696e75782f676f312e342e32",
                    "gasLimit": "0x1388",
                    "gasUsed": "0x0",
                    "hash": "0xdc0818cf78f21a8e70579cb46a43643f78291264dda342ae31049421c82d21ae",
                    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                    "miner": "0xbb7b8287f3f0a933474a79eae42cbca977791171",
                    "mixHash": "0x4fffe9ae21f1c9e15207b1f472d5bbdd68c9595d461666602f2be20daf5e7843",
                    "nonce": "0x689056015818adbe",
                    "parentHash": "0xe99e022112df268087ea7eafaf4790497fd21dbeeb6bd7a1721df161a6657a54",
                    "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                    "size": "0x220",
                    "stateRoot": "0xddc8b0234c2e0cad087c8b389aa7ef01f7d79b2570bccb77ce48648aa61c904d",
                    "timestamp": "0x55ba467c",
                    "totalDifficulty": "0x78ed983323d",
                    "transactions": [],
                    "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                    "uncles": []
                    }
                }"#.into(),
            },
            context: vec![],
        };
        let res = processor.transform(raw);
        assert_eq!(
            res.body,
            serde_json::to_vec(&json!({"eip1559": false})).unwrap()
        );
    }
}
