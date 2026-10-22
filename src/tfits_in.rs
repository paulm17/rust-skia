//! Integer fit checks corresponding to `include/private/SkTFitsIn.h`.

pub fn sk_tfits_in_i8<S: TryInto<i8>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_u8<S: TryInto<u8>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_i16<S: TryInto<i16>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_u16<S: TryInto<u16>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_i32<S: TryInto<i32>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_u32<S: TryInto<u32>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_i64<S: TryInto<i64>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_u64<S: TryInto<u64>>(value: S) -> bool {
    value.try_into().is_ok()
}
pub fn sk_tfits_in_usize<S: TryInto<usize>>(value: S) -> bool {
    value.try_into().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sk_tfits_in_matches_checked_conversion() {
        assert!(sk_tfits_in_u32(0u64));
        assert!(sk_tfits_in_u32(u32::MAX as u64));
        assert!(!sk_tfits_in_u32(u32::MAX as u64 + 1));
        assert!(!sk_tfits_in_usize(-1i64));
    }
}
