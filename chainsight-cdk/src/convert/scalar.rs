use ic_web3_rs::types::U256;

pub trait Convertible<To> {
    fn convert(
        &self,
        digit_to_scale: u32, // only support carry
    ) -> To;
}

macro_rules! convertible_num_for_string {
    ($uint_ty: ident) => {
        impl Convertible<$uint_ty> for String {
            fn convert(&self, digit_to_scale: u32) -> $uint_ty {
                let casted = self.parse::<$uint_ty>().unwrap();
                let scaled = casted * (10 as $uint_ty).pow(digit_to_scale);
                scaled
            }
        }
    };
}
macro_rules! convertible_num_for_str {
    ($uint_ty: ident) => {
        impl Convertible<$uint_ty> for &str {
            fn convert(&self, digit_to_scale: u32) -> $uint_ty {
                let casted = self.parse::<$uint_ty>().unwrap();
                let scaled = casted * (10 as $uint_ty).pow(digit_to_scale);
                scaled
            }
        }
    };
}

convertible_num_for_string!(u128);
convertible_num_for_string!(u64);
convertible_num_for_string!(u32);
convertible_num_for_string!(u16);
convertible_num_for_string!(u8);
convertible_num_for_string!(i128);
convertible_num_for_string!(i64);
convertible_num_for_string!(i32);
convertible_num_for_string!(i16);
convertible_num_for_string!(i8);

impl Convertible<U256> for String {
    fn convert(&self, digit_to_scale: u32) -> U256 {
        U256::from_dec_str(self).unwrap() * U256::from(10u128.pow(digit_to_scale))
    }
}

convertible_num_for_str!(u128);
convertible_num_for_str!(u64);
convertible_num_for_str!(u32);
convertible_num_for_str!(u16);
convertible_num_for_str!(u8);
convertible_num_for_str!(i128);
convertible_num_for_str!(i64);
convertible_num_for_str!(i32);
convertible_num_for_str!(i16);
convertible_num_for_str!(i8);

impl Convertible<U256> for &str {
    fn convert(&self, digit_to_scale: u32) -> U256 {
        U256::from_dec_str(self).unwrap() * U256::from(10u128.pow(digit_to_scale))
    }
}

macro_rules! convertible_u256_for_num {
    ($uint_ty: ident) => {
        impl Convertible<U256> for $uint_ty {
            fn convert(&self, digit_to_scale: u32) -> U256 {
                let scaled: u128 = self.convert(digit_to_scale);
                U256::from(scaled)
            }
        }
    };
}

macro_rules! convertible_uint_for_int {
    ($int_ty: ident, $uint_ty: ident) => {
        impl Convertible<$uint_ty> for $int_ty {
            fn convert(&self, digit_to_scale: u32) -> $uint_ty {
                let scaled = (*self as i128) * (10u128.pow(digit_to_scale) as i128);
                scaled as $uint_ty
            }
        }
    };
}

convertible_uint_for_int!(i128, u128);
convertible_u256_for_num!(i128);
convertible_uint_for_int!(i64, u128);
convertible_u256_for_num!(i64);
convertible_uint_for_int!(i32, u128);
convertible_u256_for_num!(i32);
convertible_uint_for_int!(i16, u128);
convertible_u256_for_num!(i16);
convertible_uint_for_int!(i8, u128);
convertible_u256_for_num!(i8);

macro_rules! convertible_num_for_float {
    ($int_ty: ident, $uint_ty: ident) => {
        impl Convertible<$uint_ty> for $int_ty {
            fn convert(&self, digit_to_scale: u32) -> $uint_ty {
                let scaled = (*self as f64) * (10u128.pow(digit_to_scale) as f64);
                scaled as $uint_ty
            }
        }
    };
}

convertible_num_for_float!(f64, u128);
convertible_num_for_float!(f64, i128);
convertible_u256_for_num!(f64);

convertible_num_for_float!(f32, u128);
convertible_num_for_float!(f32, i128);
convertible_u256_for_num!(f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convertible_string() {
        let s = "123".to_string();
        let val: u128 = s.convert(0);
        assert_eq!(val, 123u128);
        let val: u64 = s.convert(0);
        assert_eq!(val, 123u64);
        let val: u32 = s.convert(0);
        assert_eq!(val, 123u32);
        let val: u16 = s.convert(0);
        assert_eq!(val, 123u16);
        let val: u8 = s.convert(0);
        assert_eq!(val, 123u8);
        let val: i128 = s.convert(0);
        assert_eq!(val, 123i128);
        let val: i64 = s.convert(0);
        assert_eq!(val, 123i64);
        let val: i32 = s.convert(0);
        assert_eq!(val, 123i32);
        let val: i16 = s.convert(0);
        assert_eq!(val, 123i16);
        let val: i8 = s.convert(0);
        assert_eq!(val, 123i8);

        let val: U256 = s.convert(0);
        assert_eq!(val, U256::from(123u128));
    }

    #[test]
    fn test_convertible_str() {
        let s = "123";

        let val: u128 = s.convert(0);
        assert_eq!(val, 123u128);
        let val: u64 = s.convert(0);
        assert_eq!(val, 123u64);
        let val: u32 = s.convert(0);
        assert_eq!(val, 123u32);
        let val: u16 = s.convert(0);
        assert_eq!(val, 123u16);
        let val: u8 = s.convert(0);
        assert_eq!(val, 123u8);
        let val: i128 = s.convert(0);
        assert_eq!(val, 123i128);
        let val: i64 = s.convert(0);
        assert_eq!(val, 123i64);
        let val: i32 = s.convert(0);
        assert_eq!(val, 123i32);
        let val: i16 = s.convert(0);
        assert_eq!(val, 123i16);
        let val: i8 = s.convert(0);
        assert_eq!(val, 123i8);

        let val: U256 = s.convert(0);
        assert_eq!(val, U256::from(123u128));
    }

    fn assert_convertible_num<T>(num: T, expected: u128)
    where
        T: Convertible<u128> + Convertible<U256>,
    {
        let expected_u256 = U256::from(expected);

        let val_from_i128: u128 = num.convert(0);
        let val_from_u256: U256 = num.convert(0);

        assert_eq!(val_from_i128, expected);
        assert_eq!(val_from_u256, expected_u256);
    }

    #[test]
    fn test_convertible_int() {
        let expected = 123u128;
        assert_convertible_num::<i128>(123, expected);
        assert_convertible_num::<i64>(123, expected);
        assert_convertible_num::<i32>(123, expected);
        assert_convertible_num::<i16>(123, expected);
        assert_convertible_num::<i8>(123, expected);
    }

    #[test]
    fn test_convertible_float() {
        let expected = 123u128;
        assert_convertible_num::<f64>(123.0, expected);
        assert_convertible_num::<f32>(123.0, expected);
    }
}
