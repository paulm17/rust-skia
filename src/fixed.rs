//! 16.16 fixed point and FDot6 helpers from `SkFixed.h` and `SkFDot6.h`.

use crate::floating_point::{float_saturate_to_int, float_saturate_to_int64};
use crate::safe32::pin_to_s32;

pub type SkFixed = i32;
pub type SkFDot6 = i32;
pub type SkFixed3232 = i64;

pub const SK_FIXED_1: SkFixed = 1 << 16;
pub const SK_FIXED_HALF: SkFixed = 1 << 15;
pub const SK_FIXED_QUARTER: SkFixed = 1 << 14;
pub const SK_FIXED_MAX: SkFixed = 0x7fff_ffff;
pub const SK_FIXED_MIN: SkFixed = -SK_FIXED_MAX;
pub const SK_FIXED_PI: SkFixed = 0x3_243f;
pub const SK_FIXED_SQRT2: SkFixed = 92682;
pub const SK_FIXED_TAN_PI_OVER_8: SkFixed = 0x6a0a;
pub const SK_FIXED_ROOT2_OVER2: SkFixed = 0xb505;

pub const SK_FDOT6_ONE: SkFDot6 = 64;
pub const SK_FDOT6_HALF: SkFDot6 = 32;

pub const SK_FIXED_3232_MAX: SkFixed3232 = i64::MAX;
pub const SK_FIXED_3232_MIN: SkFixed3232 = -SK_FIXED_3232_MAX;

pub const fn sk_fixed_to_float(x: SkFixed) -> f32 {
    x as f32 * (1.0f32 / 65536.0)
}

pub fn sk_float_to_fixed(x: f32) -> SkFixed {
    float_saturate_to_int(x * SK_FIXED_1 as f32)
}

pub fn sk_float_to_fixed_check(x: f32) -> SkFixed {
    if cfg!(debug_assertions) {
        let n64 = (x * SK_FIXED_1 as f32) as i64;
        let n32 = n64 as SkFixed;
        debug_assert_eq!(n64, n32 as i64);
        n32
    } else {
        sk_float_to_fixed(x)
    }
}

pub const fn sk_fixed_to_double(x: SkFixed) -> f64 {
    x as f64 * 1.52587890625e-5
}

pub const fn sk_double_to_fixed(x: f64) -> SkFixed {
    (x * SK_FIXED_1 as f64) as SkFixed
}

pub const fn sk_int_to_fixed(n: i32) -> SkFixed {
    debug_assert!(n >= -32768 && n <= 32767);
    ((n as u32) << 16) as SkFixed
}

pub const fn sk_fixed_round_to_int(x: SkFixed) -> i32 {
    (x.wrapping_add(SK_FIXED_HALF)) >> 16
}

pub const fn sk_fixed_ceil_to_int(x: SkFixed) -> i32 {
    (x.wrapping_add(SK_FIXED_1 - 1)) >> 16
}

pub const fn sk_fixed_floor_to_int(x: SkFixed) -> i32 {
    x >> 16
}

pub const fn sk_fixed_round_to_fixed(x: SkFixed) -> SkFixed {
    ((x.wrapping_add(SK_FIXED_HALF)) as u32 & 0xffff_0000) as SkFixed
}

pub const fn sk_fixed_ceil_to_fixed(x: SkFixed) -> SkFixed {
    ((x.wrapping_add(SK_FIXED_1 - 1)) as u32 & 0xffff_0000) as SkFixed
}

pub const fn sk_fixed_floor_to_fixed(x: SkFixed) -> SkFixed {
    (x as u32 & 0xffff_0000) as SkFixed
}

pub const fn sk_fixed_ave(a: SkFixed, b: SkFixed) -> SkFixed {
    a.wrapping_add(b) >> 1
}

pub fn sk_fixed_div(numer: SkFixed, denom: SkFixed) -> SkFixed {
    debug_assert!(denom != 0);
    pin_to_s32(((numer as i64) << 16) / denom as i64)
}

pub const fn sk_fixed_mul(a: SkFixed, b: SkFixed) -> SkFixed {
    ((a as i64 * b as i64) >> 16) as SkFixed
}

pub const fn sk_fixed_to_scalar(x: SkFixed) -> f32 {
    sk_fixed_to_float(x)
}
pub fn sk_scalar_to_fixed(x: f32) -> SkFixed {
    sk_float_to_fixed(x)
}

pub const fn sk_int_to_fixed3232(x: i32) -> SkFixed3232 {
    ((x as i64) << 32) as SkFixed3232
}
pub const fn sk_fixed3232_to_int(x: SkFixed3232) -> i32 {
    (x >> 32) as i32
}
pub const fn sk_fixed_to_fixed3232(x: SkFixed) -> SkFixed3232 {
    ((x as i64) << 16) as SkFixed3232
}
pub const fn sk_fixed3232_to_fixed(x: SkFixed3232) -> SkFixed {
    (x >> 16) as SkFixed
}
pub fn sk_float_to_fixed3232(x: f32) -> SkFixed3232 {
    float_saturate_to_int64(x * (65536.0f32 * 65536.0f32))
}
pub const fn sk_fixed3232_to_float(x: SkFixed3232) -> f32 {
    x as f32 * (1.0f32 / (65536.0f32 * 65536.0f32))
}
pub fn sk_scalar_to_fixed3232(x: f32) -> SkFixed3232 {
    sk_float_to_fixed3232(x)
}

pub fn sk_scalar_round_to_fdot6(x: f32, shift: i32) -> SkFDot6 {
    let fractional_bits = 6 + shift;
    let magic = ((1i64 << (52 - fractional_bits)) as f64) * 1.5;
    let bits = (x as f64 + magic).to_bits();
    (bits & 0xffff_ffff) as u32 as SkFDot6
}

pub const fn sk_int_to_fdot6(x: i32) -> SkFDot6 {
    debug_assert!(x >= (i32::MIN >> 6) && x <= (i32::MAX >> 6));
    x << 6
}
pub const fn sk_fdot6_floor(x: SkFDot6) -> i32 {
    x >> 6
}
pub const fn sk_fdot6_ceil(x: SkFDot6) -> i32 {
    (x + 63) >> 6
}
pub const fn sk_fdot6_round(x: SkFDot6) -> i32 {
    (x + SK_FDOT6_HALF) >> 6
}
pub const fn sk_fixed_to_fdot6(x: SkFixed) -> SkFDot6 {
    x >> 10
}
pub const fn sk_fdot6_to_fixed(x: SkFDot6) -> SkFixed {
    debug_assert!((x << 10 >> 10) == x);
    x << 10
}
pub const fn sk_float_to_fdot6(x: f32) -> SkFDot6 {
    (x * SK_FDOT6_ONE as f32) as SkFDot6
}
pub const fn sk_scalar_to_fdot6(x: f32) -> SkFDot6 {
    sk_float_to_fdot6(x)
}
pub const fn sk_fdot6_to_float(x: SkFDot6) -> f32 {
    x as f32 * 0.015625
}
pub const fn sk_fdot6_to_scalar(x: SkFDot6) -> f32 {
    sk_fdot6_to_float(x)
}

pub fn sk_fdot6_div(a: SkFDot6, b: SkFDot6) -> SkFixed {
    debug_assert!(b != 0);
    if (i16::MIN as i32..=i16::MAX as i32).contains(&a) {
        (a << 16) / b
    } else {
        sk_fixed_div(a, b)
    }
}
