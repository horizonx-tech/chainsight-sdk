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

pub trait Scalable {
    fn scale(&self, digit: u32) -> Self;
}

macro_rules! scale_num {
    ($scalar_ty: ident, $intermediate_ty: ident) => {
        impl Scalable for $scalar_ty {
            fn scale(&self, digit_to_scale: u32) -> Self {
                let scaled =
                    *self as $intermediate_ty * (10u128.pow(digit_to_scale) as $intermediate_ty);
                scaled as $scalar_ty
            }
        }
    };
}

scale_num!(u128, u128);
scale_num!(u64, u128);
scale_num!(u32, u128);
scale_num!(u16, u128);
scale_num!(u8, u128);
scale_num!(i128, i128);
scale_num!(i64, i128);
scale_num!(i32, i128);
scale_num!(i16, i128);
scale_num!(i8, i128);
scale_num!(f64, f64);
scale_num!(f32, f64);

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
    fn test_convertible_string_with_scale() {
        let s = "123".to_string();
        let val: u128 = s.convert(3);
        assert_eq!(val, 123000u128);
        let val: u64 = s.convert(3);
        assert_eq!(val, 123000u64);
        let val: u32 = s.convert(2);
        assert_eq!(val, 12300u32);
        let val: u16 = s.convert(1);
        assert_eq!(val, 1230u16);
        let val: u8 = s.convert(0);
        assert_eq!(val, 123u8);
        let val: i128 = s.convert(3);
        assert_eq!(val, 123000i128);
        let val: i64 = s.convert(3);
        assert_eq!(val, 123000i64);
        let val: i32 = s.convert(2);
        assert_eq!(val, 12300i32);
        let val: i16 = s.convert(1);
        assert_eq!(val, 1230i16);
        let val: i8 = s.convert(0);
        assert_eq!(val, 123i8);

        let val: U256 = s.convert(3);
        assert_eq!(val, U256::from(123000u128));
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

    #[test]
    fn test_convertible_str_with_scale() {
        let s = "123";
        let val: u128 = s.convert(3);
        assert_eq!(val, 123000u128);
        let val: u64 = s.convert(3);
        assert_eq!(val, 123000u64);
        let val: u32 = s.convert(2);
        assert_eq!(val, 12300u32);
        let val: u16 = s.convert(1);
        assert_eq!(val, 1230u16);
        let val: u8 = s.convert(0);
        assert_eq!(val, 123u8);
        let val: i128 = s.convert(3);
        assert_eq!(val, 123000i128);
        let val: i64 = s.convert(3);
        assert_eq!(val, 123000i64);
        let val: i32 = s.convert(2);
        assert_eq!(val, 12300i32);
        let val: i16 = s.convert(1);
        assert_eq!(val, 1230i16);
        let val: i8 = s.convert(0);
        assert_eq!(val, 123i8);

        let val: U256 = s.convert(3);
        assert_eq!(val, U256::from(123000u128));
    }

    fn assert_convertible_num<T>(num: T, digit: u32, expected: u128)
    where
        T: Convertible<u128> + Convertible<U256>,
    {
        let expected_u256 = U256::from(expected);

        let val_from_i128: u128 = num.convert(digit);
        let val_from_u256: U256 = num.convert(digit);

        assert_eq!(val_from_i128, expected);
        assert_eq!(val_from_u256, expected_u256);
    }

    #[test]
    fn test_convertible_int() {
        let expected = 123u128;
        assert_convertible_num::<i128>(123, 0, expected);
        assert_convertible_num::<i64>(123, 0, expected);
        assert_convertible_num::<i32>(123, 0, expected);
        assert_convertible_num::<i16>(123, 0, expected);
        assert_convertible_num::<i8>(123, 0, expected);
    }

    #[test]
    fn test_converible_int_with_scale() {
        assert_convertible_num::<i128>(123, 3, 123000);
        assert_convertible_num::<i64>(123, 3, 123000);
        assert_convertible_num::<i32>(123, 2, 12300);
        assert_convertible_num::<i16>(123, 1, 1230);
        assert_convertible_num::<i8>(123, 0, 123);
    }

    #[test]
    fn test_convertible_float() {
        let expected = 123u128;
        assert_convertible_num::<f64>(123.0, 0, expected);
        assert_convertible_num::<f32>(123.0, 0, expected);
    }

    #[test]
    fn test_convertible_float_with_scale() {
        assert_convertible_num::<f64>(123.0, 3, 123000);
        assert_convertible_num::<f32>(123.0, 3, 123000);
    }

    fn assert_scale<T>(num: T, digit: u32, expected: T)
    where
        T: Scalable + std::fmt::Debug + PartialEq,
    {
        let scaled = num.scale(digit);
        assert_eq!(scaled, expected);
    }

    #[test]
    fn test_scale() {
        assert_scale(12345u128, 5, 1234500000);
        assert_scale(1234u64, 4, 12340000);
        assert_scale(123u32, 3, 123000);
        assert_scale(12u16, 2, 1200);
        assert_scale(1u8, 1, 10);
        assert_scale(1u8, 0, 1);

        assert_scale(-12345i128, 5, -1234500000);
        assert_scale(-1234i64, 4, -12340000);
        assert_scale(-123i32, 3, -123000);
        assert_scale(-12i16, 2, -1200);
        assert_scale(-1i8, 1, -10);
        assert_scale(-1i8, 0, -1);

        assert_scale(654.321f64, 3, 654321.0);
        assert_scale(654.321f64, 1, 6543.21);
        assert_scale(43.21f32, 3, 43210.0);
        assert_scale(43.21f32, 1, 432.09998); // 432.1
    }
}
