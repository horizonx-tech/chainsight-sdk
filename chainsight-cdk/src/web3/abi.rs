use crate::core::U256 as ChainsightU256;
use ethabi::{Bytes, Token};
use primitive_types::U256;

trait Encoder<T> {
    fn encode(&self, val: T) -> Bytes;
}

struct EthAbiEncoder;

impl Encoder<U256> for EthAbiEncoder {
    fn encode(&self, val: U256) -> Bytes {
        let token = Token::Uint(val);
        ethabi::encode(&[token])
    }
}
impl Encoder<ChainsightU256> for EthAbiEncoder {
    fn encode(&self, val: ChainsightU256) -> Bytes {
        let token = Token::Uint(val.value());
        ethabi::encode(&[token])
    }
}
impl Encoder<u128> for EthAbiEncoder {
    fn encode(&self, val: u128) -> Bytes {
        let token = Token::Uint(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<u64> for EthAbiEncoder {
    fn encode(&self, val: u64) -> Bytes {
        let token = Token::Uint(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<u32> for EthAbiEncoder {
    fn encode(&self, val: u32) -> Bytes {
        let token = Token::Uint(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<u16> for EthAbiEncoder {
    fn encode(&self, val: u16) -> Bytes {
        let token = Token::Uint(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<u8> for EthAbiEncoder {
    fn encode(&self, val: u8) -> Bytes {
        let token = Token::Uint(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<i128> for EthAbiEncoder {
    fn encode(&self, val: i128) -> Bytes {
        let token = Token::Int(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<i64> for EthAbiEncoder {
    fn encode(&self, val: i64) -> Bytes {
        let token = Token::Int(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<i32> for EthAbiEncoder {
    fn encode(&self, val: i32) -> Bytes {
        let token = Token::Int(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<i16> for EthAbiEncoder {
    fn encode(&self, val: i16) -> Bytes {
        let token = Token::Int(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<i8> for EthAbiEncoder {
    fn encode(&self, val: i8) -> Bytes {
        let token = Token::Int(val.into());
        ethabi::encode(&[token])
    }
}
impl Encoder<String> for EthAbiEncoder {
    fn encode(&self, val: String) -> Bytes {
        let token = Token::String(val);
        ethabi::encode(&[token])
    }
}
impl Encoder<&str> for EthAbiEncoder {
    fn encode(&self, val: &str) -> Bytes {
        let token = Token::String(val.to_string());
        ethabi::encode(&[token])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode_u128() {
        let encoder = EthAbiEncoder;
        let expected: u128 = 12345678901234567890;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(128)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap()
            .as_u128();
        assert_eq!(decoded, expected);
    }
    #[test]
    fn test_encode_u64() {
        let encoder = EthAbiEncoder;
        let expected: u64 = 12345678901234567890;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(64)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap()
            .as_u64();
        assert_eq!(decoded, expected);
    }
    #[test]
    fn test_encode_u32() {
        let encoder = EthAbiEncoder;
        let expected: u32 = 1234567890;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(32)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap()
            .as_u32();
        assert_eq!(decoded, expected);
    }
    #[test]
    fn test_encode_u16() {
        let encoder = EthAbiEncoder;
        let expected: u16 = 12345;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(16)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap()
            .as_u128();
        assert_eq!((decoded as u16), expected);
    }
    #[test]
    fn test_encode_u8() {
        let encoder = EthAbiEncoder;
        let expected: u8 = 123;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(8)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap()
            .as_u128();
        assert_eq!((decoded as u8), expected);
    }
    #[test]
    fn test_encode_i128() {
        let encoder = EthAbiEncoder;
        let expected: i128 = 12345678901234567890;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Int(128)], &encoded).unwrap()[0]
            .clone()
            .into_int()
            .unwrap()
            .as_u128();
        assert_eq!(decoded, (expected as u128));
    }
    #[test]
    fn test_encode_i64() {
        let encoder = EthAbiEncoder;
        let expected: i64 = 1234567890123456789;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Int(64)], &encoded).unwrap()[0]
            .clone()
            .into_int()
            .unwrap()
            .as_u128();
        assert_eq!(decoded, (expected as u128));
    }
    #[test]
    fn test_encode_i32() {
        let encoder = EthAbiEncoder;
        let expected: i32 = 1234567890;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Int(32)], &encoded).unwrap()[0]
            .clone()
            .into_int()
            .unwrap()
            .as_u128();
        assert_eq!(decoded, (expected as u128));
    }
    #[test]
    fn test_encode_i16() {
        let encoder = EthAbiEncoder;
        let expected: i16 = 12345;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Int(16)], &encoded).unwrap()[0]
            .clone()
            .into_int()
            .unwrap()
            .as_u128();
        assert_eq!(decoded, (expected as u128));
    }
    #[test]
    fn test_encode_i8() {
        let encoder = EthAbiEncoder;
        let expected: i8 = 123;
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Int(8)], &encoded).unwrap()[0]
            .clone()
            .into_int()
            .unwrap()
            .as_u128();
        assert_eq!((decoded), (expected as u128));
    }
    #[test]
    fn test_encode_u256() {
        let encoder = EthAbiEncoder;
        let expected: U256 = U256::from(12345678901234567890u128);
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(256)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap();
        assert_eq!(decoded, expected);
    }
    #[test]
    fn test_encode_string() {
        let encoder = EthAbiEncoder;
        let expected: String = "hello world".to_string();
        let encoded = encoder.encode(expected.clone());
        let decoded = ethabi::decode(&[ethabi::ParamType::String], &encoded).unwrap()[0]
            .clone()
            .into_string()
            .unwrap();
        assert_eq!(decoded, expected);
    }
    #[test]
    fn test_encode_str() {
        let encoder = EthAbiEncoder;
        let expected: &str = "hello world";
        let encoded = encoder.encode(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::String], &encoded).unwrap()[0]
            .clone()
            .into_string()
            .unwrap();
        assert_eq!(decoded, expected);
    }
    #[test]
    fn test_encode_chainsight_u256() {
        let encoder = EthAbiEncoder;
        let expected: ChainsightU256 = ChainsightU256::from(U256::from(12345678901234567890u128));
        let encoded = encoder.encode(expected.clone());
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(256)], &encoded).unwrap()[0]
            .clone()
            .into_uint()
            .unwrap();
        assert_eq!(decoded, expected.value());
    }
}