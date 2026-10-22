//! Checked integer conversions corresponding to `include/private/SkTo.h`.
//!
//! The C++ helpers assert in debug builds and then static-cast. The Rust
//! `sk_to_*` functions mirror that debug/release split; internal callers use
//! the `checked_*` forms when they need a recoverable failure.

pub fn checked_to_i8<S: TryInto<i8>>(value: S) -> Option<i8> {
    value.try_into().ok()
}
pub fn checked_to_u8<S: TryInto<u8>>(value: S) -> Option<u8> {
    value.try_into().ok()
}
pub fn checked_to_i16<S: TryInto<i16>>(value: S) -> Option<i16> {
    value.try_into().ok()
}
pub fn checked_to_u16<S: TryInto<u16>>(value: S) -> Option<u16> {
    value.try_into().ok()
}
pub fn checked_to_i32<S: TryInto<i32>>(value: S) -> Option<i32> {
    value.try_into().ok()
}
pub fn checked_to_u32<S: TryInto<u32>>(value: S) -> Option<u32> {
    value.try_into().ok()
}
pub fn checked_to_i64<S: TryInto<i64>>(value: S) -> Option<i64> {
    value.try_into().ok()
}
pub fn checked_to_u64<S: TryInto<u64>>(value: S) -> Option<u64> {
    value.try_into().ok()
}
pub fn checked_to_usize<S: TryInto<usize>>(value: S) -> Option<usize> {
    value.try_into().ok()
}

/// C++ `SkTo`: assert the conversion fits in debug builds, then perform a
/// static cast. Release builds intentionally preserve the cast result.
pub trait SkToCast<Dst> {
    fn sk_to(self) -> Dst;
}

macro_rules! impl_sk_to_cast {
    ($src:ty => $($dst:ty),+ $(,)?) => {
        $(
            impl SkToCast<$dst> for $src {
                fn sk_to(self) -> $dst {
                    #[cfg(debug_assertions)]
                    {
                        assert!(
                            <$src as TryInto<$dst>>::try_into(self).is_ok(),
                            "SkTo conversion does not fit"
                        );
                    }
                    self as $dst
                }
            }
        )+
    };
}

impl_sk_to_cast!(i8 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(i16 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(i32 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(i64 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(i128 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(isize => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(u8 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(u16 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(u32 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(u64 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(u128 => i8, u8, i16, u16, i32, u32, i64, u64, usize);
impl_sk_to_cast!(usize => i8, u8, i16, u16, i32, u32, i64, u64, usize);

macro_rules! sk_to_fn {
    ($name:ident, $dst:ty) => {
        pub fn $name<S: SkToCast<$dst>>(value: S) -> $dst {
            value.sk_to()
        }
    };
}

sk_to_fn!(sk_to_s8, i8);
sk_to_fn!(sk_to_u8, u8);
sk_to_fn!(sk_to_s16, i16);
sk_to_fn!(sk_to_u16, u16);
sk_to_fn!(sk_to_s32, i32);
sk_to_fn!(sk_to_u32, u32);
sk_to_fn!(sk_to_s64, i64);
sk_to_fn!(sk_to_u64, u64);
sk_to_fn!(sk_to_size_t, usize);

pub fn sk_to_int<S: SkToCast<i32>>(value: S) -> i32 {
    sk_to_s32(value)
}

pub fn sk_to_uint<S: SkToCast<u32>>(value: S) -> u32 {
    sk_to_u32(value)
}

pub fn sk_to_bool<T: Into<bool>>(value: T) -> bool {
    value.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sk_to_checks_boundaries() {
        assert_eq!(sk_to_s32(42usize), 42);
        assert_eq!(checked_to_i32(u32::MAX), None);
        assert_eq!(checked_to_i32(i32::MAX as u32), Some(i32::MAX));
        assert_eq!(checked_to_usize(-1i32), None);
    }
}
