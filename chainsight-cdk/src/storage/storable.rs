use std::borrow::Cow;

use ic_stable_structures::{BoundedStorable, Storable};

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    PartialOrd,
    candid::CandidType,
    candid::Deserialize,
    serde::Serialize,
)]
pub struct StorableBool(pub bool);

impl Storable for StorableBool {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let num: u8 = if self.0 { 1 } else { 0 };
        Cow::Owned(num.to_be_bytes().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let num = u8::from_be_bytes(bytes.as_ref().try_into().unwrap());
        match num {
            0 => StorableBool(false),
            1 => StorableBool(true),
            _ => panic!("Invalid bool encoding: expected 0 or 1, found {}", num),
        }
    }
}
impl BoundedStorable for StorableBool {
    const MAX_SIZE: u32 = 1;
    const IS_FIXED_SIZE: bool = true;
}
impl From<StorableBool> for bool {
    fn from(wrapper: StorableBool) -> Self {
        wrapper.0
    }
}
impl From<bool> for StorableBool {
    fn from(unwrapped: bool) -> Self {
        StorableBool(unwrapped)
    }
}

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    PartialOrd,
    candid::CandidType,
    candid::Deserialize,
    serde::Serialize,
)]
pub struct StorableStrings(pub Vec<String>);

impl Storable for StorableStrings {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut result = Vec::new();

        for s in &self.0 {
            let s_bytes = s.as_bytes();
            let s_len = s_bytes.len() as u64;
            let s_len_bytes = s_len.to_be_bytes().to_vec();

            result.extend_from_slice(&s_len_bytes);
            result.extend_from_slice(s_bytes);
        }

        Cow::Owned(result)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let mut result = Vec::new();
        let mut bytes = bytes.as_ref();

        while !bytes.is_empty() {
            let s_len_bytes = &bytes[0..8];
            let s_len = u64::from_be_bytes(s_len_bytes.try_into().unwrap()) as usize;
            let s_bytes = &bytes[8..(8 + s_len)];
            let s = String::from_utf8(s_bytes.to_vec()).unwrap();

            result.push(s);

            bytes = &bytes[(8 + s_len)..];
        }

        Self(result)
    }
}

impl BoundedStorable for StorableStrings {
    const MAX_SIZE: u32 = 8192;
    const IS_FIXED_SIZE: bool = false;
}

impl From<StorableStrings> for Vec<String> {
    fn from(wrapper: StorableStrings) -> Self {
        wrapper.0
    }
}
impl From<Vec<String>> for StorableStrings {
    fn from(unwrapped: Vec<String>) -> Self {
        StorableStrings(unwrapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storable_strings() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let storable = StorableStrings(data.clone());
        let decoded = StorableStrings::from_bytes(storable.to_bytes());
        assert_eq!(data, decoded.0);

        let data = vec![
            "u3zgx-4yaaa-aaaal-achaa-cai".to_string(),
            "ua42s-gaaaa-aaaal-achcq-cai".to_string(),
            "uh54g-lyaaa-aaaal-achca-cai".to_string(),
            "7fpuj-hqaaa-aaaal-acg7q-cai".to_string(),
        ];
        let storable = StorableStrings(data.clone());
        let decoded = StorableStrings::from_bytes(storable.to_bytes());
        assert_eq!(data, decoded.0);
    }
}
