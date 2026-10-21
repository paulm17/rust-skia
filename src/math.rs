// Early subset of SkMath.h required by SkPoint/SkFloatBits.
// The complete SkMath.h ownership remains with milestone M03.

pub const SK_MAX_S32: i32 = i32::MAX;
pub const SK_MIN_S32: i32 = -SK_MAX_S32;
pub const SK_MAX_S64: i64 = i64::MAX;
pub const SK_MIN_S64: i64 = -SK_MAX_S64;

pub const fn left_shift_i32(value: i32, shift: i32) -> i32 {
    ((value as u32) << shift) as i32
}

pub const fn left_shift_i64(value: i64, shift: i32) -> i64 {
    ((value as u64) << shift) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sk_math_limits_and_left_shift() {
        assert_eq!(SK_MAX_S32, 2_147_483_647);
        assert_eq!(SK_MIN_S32, -2_147_483_647);
        assert_eq!(SK_MAX_S64, 9_223_372_036_854_775_807);
        assert_eq!(SK_MIN_S64, -9_223_372_036_854_775_807);
        assert_eq!(left_shift_i32(-1, 31), i32::MIN);
        assert_eq!(left_shift_i64(-1, 63), i64::MIN);
    }
}
