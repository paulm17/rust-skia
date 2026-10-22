// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/Float16Test.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
const F32_SIGN: u32 = 1 << 31;
const F32_EXP: u32 = 255 << 23;
const F32_MANT: u32 = !(F32_SIGN | F32_EXP);
const F32_BIAS: i32 = 127;
const F16_SIGN: u32 = 1 << 15;
const F16_EXP: u32 = 31 << 10;
const F16_MANT: u32 = !(F16_SIGN | F16_EXP);
const F16_BIAS: i32 = 15;
// tests/Float16Test.cpp:29. The original #if 0 exhaustive 2^32 loop remains disabled.
#[test]
fn FloatToHalf() {
    let mut r = Reporter::default();
    let mut rand = SkRandom::new();
    const COUNT: u32 = 2 * 255 * 1024 * 8;
    for i in 0..COUNT {
        let sign = (i & 1) << 31;
        let exp = ((i >> 1) & 255) << 23;
        let man = ((i >> 9) & 1023) << 13;
        let frac = (i >> 19) & 7;
        let bits = sign
            | exp
            | man
            | if frac == 0 {
                0
            } else if frac == 1 {
                (1 << 13) - 1
            } else {
                rand.next_bits(13)
            };
        let f = bits_to_float(bits);
        if is_nan(f) {
            #[cfg(not(debug_assertions))]
            {
                let actual2 = to_half(Float2::splat(f))[0];
                let actual4 = to_half(Float4::splat(f))[0];
                check!(r, (actual2 as u32 & F16_EXP) == F16_EXP);
                check!(r, (actual4 as u32 & F16_EXP) == F16_EXP);
            }
            continue;
        }
        let s32 = bits & F32_SIGN;
        let e32 = bits & F32_EXP;
        let mut m32 = bits & F32_MANT;
        let e = (e32 >> 23) as i32 - F32_BIAS;
        let s16 = s32 >> 16;
        let mut e16;
        let m16;
        if e < -F16_BIAS - 10 || (e == -F16_BIAS - 10 && m32 <= 0) {
            e16 = 0;
            m16 = 0;
        } else if (e32 | m32) < 0x38fe0000 {
            e16 = 0;
            m16 = 0xffff & (0.5f32 + bits_to_float(e32 | m32)).to_bits();
        } else if (e32 | m32) < 0x38800000 {
            e16 = 1;
            m16 = 0;
        } else if e > F16_BIAS {
            e16 = F16_EXP;
            m16 = 0;
        } else {
            e16 = ((e + F16_BIAS) as u32) << 10;
            debug_assert!((e16 & !F16_EXP) == 0);
            m32 += 0xfff + ((m32 >> 13) & 1);
            if m32 > F32_MANT {
                e16 += 1 << 10;
                m16 = 0;
            } else {
                m16 = m32 >> 13;
            }
        }
        let expected = (s16 | e16 | m16) as u16;
        let actual2 = to_half(Float2::splat(f))[0];
        let actual4 = to_half(Float4::splat(f))[0];
        check!(r, expected == actual2);
        check!(r, expected == actual4);
    }
    r.finish();
}
// tests/Float16Test.cpp:119
#[test]
fn FloatToHalf_Constants() {
    let mut r = Reporter::default();
    let to_half = |f| rust_skia::to_half(Float4::splat(f))[0];
    check!(r, 0 == to_half(0.));
    check!(r, F16_SIGN == to_half(-0.) as u32);
    check!(r, SK_HALF_ONE == to_half(1.));
    check!(r, (F16_SIGN | SK_HALF_ONE as u32) == to_half(-1.) as u32);
    check!(r, SK_HALF_MAX == to_half(65504.));
    check!(r, SK_HALF_MIN == to_half(1. / (1 << 14) as f32));
    r.finish();
}
// tests/Float16Test.cpp:129
#[test]
fn HalfToFloat() {
    let mut r = Reporter::default();
    for bits in 0u32..=0xffff {
        let s16 = bits & F16_SIGN;
        let e16 = bits & F16_EXP;
        let m16 = bits & F16_MANT;
        let actual2: f32 = from_half(Half2::splat(bits as u16))[0];
        let actual4: f32 = from_half(Half4::splat(bits as u16))[0];
        if e16 == 0 {
            let mut expected = (1f32 / (1 << 14) as f32) * (1f32 / (1 << 10) as f32) * m16 as f32;
            if s16 != 0 {
                expected *= -1.;
            }
            check!(r, actual2 == expected);
            check!(r, actual4 == expected);
        } else if e16 == F16_EXP {
            if m16 != 0 {
                check!(r, is_nan(actual2));
                check!(r, is_nan(actual4));
            } else if s16 != 0 {
                check!(r, actual2 == SCALAR_NEGATIVE_INFINITY);
                check!(r, actual4 == SCALAR_NEGATIVE_INFINITY);
            } else {
                check!(r, actual2 == SCALAR_INFINITY);
                check!(r, actual4 == SCALAR_INFINITY);
            }
        } else {
            let s32 = s16 << 16;
            let e32 = ((e16 >> 10) + F32_BIAS as u32 - F16_BIAS as u32) << 23;
            let m32 = m16 << 13;
            let expected = bits_to_float(s32 | e32 | m32);
            check!(r, actual2 == expected);
            check!(r, actual4 == expected);
        }
    }
    r.finish();
}
