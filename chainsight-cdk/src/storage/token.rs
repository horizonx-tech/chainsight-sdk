use std::fmt::Display;

use candid::CandidType;
use primitive_types::U256;
use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize, CandidType)]
pub enum Token {
    String(String),
    Uint(Vec<u8>),
    Bool(bool),
    Array(Vec<Token>),
    Bytes(Vec<u8>),
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::String(s) => write!(f, "{}", s),
            Token::Uint(u) => write!(
                f,
                "{}",
                u.to_vec()
                    .into_iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
            ),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Array(a) => {
                write!(f, "[")?;
                for (i, t) in a.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, "]")
            }
            Token::Bytes(b) => write!(f, "{:?}", b),
        }
    }
}
impl From<String> for Token {
    fn from(s: String) -> Self {
        Token::String(s)
    }
}
impl From<&str> for Token {
    fn from(s: &str) -> Self {
        Token::String(s.to_string())
    }
}
impl From<U256> for Token {
    fn from(u: U256) -> Self {
        let mut vec: Vec<u8> = vec![];
        u.to_big_endian(&mut vec);
        Token::Uint(vec)
    }
}
impl From<u128> for Token {
    fn from(u: u128) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<u8> for Token {
    fn from(u: u8) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<u16> for Token {
    fn from(u: u16) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<u32> for Token {
    fn from(u: u32) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<u64> for Token {
    fn from(u: u64) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<usize> for Token {
    fn from(u: usize) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<i8> for Token {
    fn from(u: i8) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<i16> for Token {
    fn from(u: i16) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<i32> for Token {
    fn from(u: i32) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<i64> for Token {
    fn from(u: i64) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<i128> for Token {
    fn from(u: i128) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}
impl From<isize> for Token {
    fn from(u: isize) -> Self {
        Token::Uint(u.to_be_bytes().to_vec())
    }
}

impl From<bool> for Token {
    fn from(b: bool) -> Self {
        Token::Bool(b)
    }
}
impl From<Vec<Token>> for Token {
    fn from(a: Vec<Token>) -> Self {
        Token::Array(a)
    }
}

impl From<Vec<u8>> for Token {
    fn from(b: Vec<u8>) -> Self {
        Token::Bytes(b)
    }
}
impl From<&[u8]> for Token {
    fn from(b: &[u8]) -> Self {
        Token::Bytes(b.to_vec())
    }
}
impl From<&Vec<u8>> for Token {
    fn from(b: &Vec<u8>) -> Self {
        Token::Bytes(b.clone())
    }
}
impl From<&Vec<Token>> for Token {
    fn from(a: &Vec<Token>) -> Self {
        Token::Array(a.clone())
    }
}
impl Token {
    pub fn to_string(&self) -> String {
        match self {
            Token::String(s) => s.clone(),
            Token::Uint(u) => format!(
                "{}",
                u.to_vec()
                    .into_iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
            ),
            Token::Bool(b) => b.to_string(),
            Token::Array(a) => {
                let mut s = String::from("[");
                for (i, t) in a.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&t.to_string());
                }
                s.push_str("]");
                s
            }
            Token::Bytes(b) => format!("{:?}", b),
        }
    }
    pub fn to_usize(&self) -> Option<usize> {
        match self {
            Token::Uint(u) => Some(usize::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_u128(&self) -> Option<u128> {
        match self {
            Token::Uint(u) => Some(u128::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_u64(&self) -> Option<u64> {
        match self {
            Token::Uint(u) => Some(u64::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_u32(&self) -> Option<u32> {
        match self {
            Token::Uint(u) => Some(u32::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_u16(&self) -> Option<u16> {
        match self {
            Token::Uint(u) => Some(u16::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_u8(&self) -> Option<u8> {
        match self {
            Token::Uint(u) => Some(u8::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_i128(&self) -> Option<i128> {
        match self {
            Token::Uint(u) => Some(i128::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Token::Uint(u) => Some(i64::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Token::Uint(u) => Some(i32::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_i16(&self) -> Option<i16> {
        match self {
            Token::Uint(u) => Some(i16::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
    pub fn to_i8(&self) -> Option<i8> {
        match self {
            Token::Uint(u) => Some(i8::from_be_bytes(u.clone().try_into().unwrap())),
            _ => None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::Token;
    #[test]
    fn test_from_string() {
        let str: String = "hello".to_string();
        let token: Token = str.into();
        assert_eq!(token, Token::String("hello".to_string()));
    }
    #[test]
    fn test_from_str() {
        let str: &str = "hello";
        let token: Token = str.into();
        assert_eq!(token, Token::from("hello".to_string()));
    }
    #[test]
    fn test_from_u128() {
        let u: u128 = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_u128));
    }
    #[test]
    fn test_from_u64() {
        let u: u64 = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_u64));
    }
    #[test]
    fn test_from_u32() {
        let u: u32 = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_u32));
    }
    #[test]
    fn test_from_u16() {
        let u: u16 = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_u16));
    }
    #[test]
    fn test_from_u8() {
        let u: u8 = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_u8));
    }
    #[test]
    fn test_from_usize() {
        let u: usize = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_usize));
    }
    #[test]
    fn test_from_i128() {
        let u: i128 = 123;
        let token: Token = u.into();
        assert_eq!(token, Token::from(123_i128));
    }
    #[test]
    fn test_from_bool() {
        let b: bool = true;
        let token: Token = b.into();
        assert_eq!(token, Token::Bool(true));
    }
    #[test]
    fn test_from_vec_token() {
        let v: Vec<Token> = vec![Token::from(123_u128), Token::from(123_u128)];
        let token: Token = v.into();
        assert_eq!(
            token,
            Token::Array(vec![Token::from(123_u128), Token::from(123_u128)])
        );
    }
    #[test]
    fn test_from_vec_u8() {
        let v: Vec<u8> = vec![1, 2, 3];
        let token: Token = v.into();
        assert_eq!(token, Token::Bytes(vec![1, 2, 3]));
    }
    #[test]
    fn test_from_vec_u8_ref() {
        let v: Vec<u8> = vec![1, 2, 3];
        let token: Token = (&v).into();
        assert_eq!(token, Token::Bytes(vec![1, 2, 3]));
    }
}
