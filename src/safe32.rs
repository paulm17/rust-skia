// Early subset of SkSafe32.h required by SkIPoint arithmetic.
// The complete SkSafe32.h ownership remains with milestone M03.

use crate::math::{SK_MAX_S32, SK_MIN_S32};

pub const fn pin_to_s32(value: i64) -> i32 {
    if value < SK_MIN_S32 as i64 {
        SK_MIN_S32
    } else if value > SK_MAX_S32 as i64 {
        SK_MAX_S32
    } else {
        value as i32
    }
}

pub const fn sat_add(a: i32, b: i32) -> i32 {
    pin_to_s32(a as i64 + b as i64)
}

pub const fn sat_sub(a: i32, b: i32) -> i32 {
    pin_to_s32(a as i64 - b as i64)
}

/// C++ `Sk32_can_overflow_add` returns the wrapped sum; despite its name it is
/// not a predicate. Used by `SkRect::width()`.
pub const fn sk32_can_overflow_add(a: i32, b: i32) -> i32 {
    (a as u32).wrapping_add(b as u32) as i32
}

/// C++ `Sk32_can_overflow_sub` returns the wrapped difference.
pub const fn sk32_can_overflow_sub(a: i32, b: i32) -> i32 {
    (a as u32).wrapping_sub(b as u32) as i32
}

pub const fn sk_abs32(value: i32) -> i32 {
    debug_assert!(value != crate::math::SK_NAN32);
    if value < 0 {
        value.wrapping_neg()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_safe32_saturates_at_sk_limits() {
        assert_eq!(sat_add(SK_MAX_S32, 1), SK_MAX_S32);
        assert_eq!(sat_add(SK_MIN_S32, -1), SK_MIN_S32);
        assert_eq!(sat_sub(SK_MIN_S32, 1), SK_MIN_S32);
        assert_eq!(sat_sub(SK_MAX_S32, -1), SK_MAX_S32);
        assert_eq!(sat_add(2, 3), 5);
        assert_eq!(sat_sub(2, 3), -1);
    }
}
