use ethabi::Bytes;
use primitive_types::U256;

trait Encoder {
    fn encode_str(val: &str) -> Bytes;
    fn encode_u64(val: u64) -> Bytes;
    fn encode_u128(val: u128) -> Bytes;
    fn encode_u256(val: U256) -> Bytes;
}

struct EthAbiEncoder;

impl Encoder for EthAbiEncoder {
    fn encode_str(val: &str) -> Bytes {
        ethabi::encode(&[ethabi::Token::String(val.to_string())])
    }

    fn encode_u64(val: u64) -> Bytes {
        ethabi::encode(&[ethabi::Token::Uint(val.into())])
    }

    fn encode_u128(val: u128) -> Bytes {
        ethabi::encode(&[ethabi::Token::Uint(val.into())])
    }

    fn encode_u256(val: U256) -> Bytes {
        ethabi::encode(&[ethabi::Token::Uint(val)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_str() {
        let expected = "test";
        let encoded = EthAbiEncoder::encode_str(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::String], &encoded).unwrap();
        assert_eq!(decoded[0].to_string(), expected);
    }

    #[test]
    fn test_encode_u64() {
        let expected = 1234567890;
        let encoded = EthAbiEncoder::encode_u64(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(64)], &encoded).unwrap();
        assert_eq!(decoded[0].to_string(), expected.to_string());
    }

    #[test]
    fn test_encode_u128() {
        let expected = 12345678901234567890;
        let encoded = EthAbiEncoder::encode_u128(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(128)], &encoded).unwrap();
        assert_eq!(decoded[0].to_string(), expected.to_string());
    }

    #[test]
    fn test_encode_u256() {
        let expected = U256::from_dec_str("1234567890123456789012345678901234567890").unwrap();
        let encoded = EthAbiEncoder::encode_u256(expected);
        let decoded = ethabi::decode(&[ethabi::ParamType::Uint(256)], &encoded).unwrap();
        assert_eq!(decoded[0].to_string(), expected.to_string());
    }
}
