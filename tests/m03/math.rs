// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/MathTest.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
fn check_length(r: &mut Reporter, p: Point, target: f32) {
    let x = p.x();
    let y = p.y();
    let mut len = (x * x + y * y).sqrt();
    len /= target;
    check!(r, len > 0.999 && len < 1.001);
}
// Instantiated separately for the two original floating-point widths.
macro_rules! unittest_isfinite {
    ($r:expr,$t:ty) => {{
        let zero: $t = 0.;
        let plain: $t = 123.;
        let inf = <$t>::INFINITY;
        let big = <$t>::MAX;
        let nan = inf * zero;
        check!($r, !inf.is_nan());
        check!($r, !(-inf).is_nan());
        check!($r, !inf.is_finite());
        check!($r, !(-inf).is_finite());
        check!($r, nan.is_nan());
        check!($r, !big.is_nan());
        check!($r, !(-big).is_nan());
        check!($r, !zero.is_nan());
        check!($r, !nan.is_finite());
        check!($r, big.is_finite());
        check!($r, (-big).is_finite());
        check!($r, zero.is_finite());
        // Multi-argument SkIsFinite is translated to the public width-specific helpers.
        check!($r, !finite2(inf, plain));
        check!($r, !finite2(plain, -inf));
        check!($r, !finite2(nan, plain));
        check!($r, finite2(plain, big));
        check!($r, finite2(-big, plain));
        check!($r, finite2(plain, zero));
        check!($r, !finite3(inf, plain, plain));
        check!($r, !finite3(plain, -inf, plain));
        check!($r, !finite3(plain, plain, nan));
        check!($r, finite3(big, plain, plain));
        check!($r, finite3(plain, -big, plain));
        check!($r, finite3(plain, plain, zero));
    }};
}
trait FiniteTest: Copy {
    fn finite2(a: Self, b: Self) -> bool;
    fn finite3(a: Self, b: Self, c: Self) -> bool;
}
impl FiniteTest for f32 {
    fn finite2(a: Self, b: Self) -> bool {
        is_finite2(a, b)
    }
    fn finite3(a: Self, b: Self, c: Self) -> bool {
        is_finite3(a, b, c)
    }
}
impl FiniteTest for f64 {
    fn finite2(a: Self, b: Self) -> bool {
        rust_skia::floating_point::is_finite2_f64(a, b)
    }
    fn finite3(a: Self, b: Self, c: Self) -> bool {
        rust_skia::floating_point::is_finite3_f64(a, b, c)
    }
}
fn finite2<T: FiniteTest>(a: T, b: T) -> bool {
    T::finite2(a, b)
}
fn finite3<T: FiniteTest>(a: T, b: T, c: T) -> bool {
    T::finite3(a, b, c)
}
fn unittest_half(r: &mut Reporter) {
    let floats = [
        0f32,
        1.,
        0.5,
        0.499999,
        0.5000001,
        1. / 3.,
        -0.,
        -1.,
        -0.5,
        -0.499999,
        -0.5000001,
        -1. / 3.,
    ];
    for value in floats {
        let h = sk_float_to_half(value);
        let f = sk_half_to_float(h);
        check!(r, scalar_nearly_equal(f, value));
    }
    let largest_positive = f32::from_bits((142 << 23) | (1023 << 13));
    let mut h = sk_float_to_half(largest_positive);
    let mut f = sk_half_to_float(h);
    check!(r, scalar_nearly_equal(f, largest_positive));
    let largest_negative = f32::from_bits((1 << 31) | (142 << 23) | (1023 << 13));
    h = sk_float_to_half(largest_negative);
    f = sk_half_to_float(h);
    check!(r, scalar_nearly_equal(f, largest_negative));
    let smallest_positive = f32::from_bits(102 << 23);
    h = sk_float_to_half(smallest_positive);
    f = sk_half_to_float(h);
    check!(r, scalar_nearly_equal(f, smallest_positive));
    let overflow = f32::from_bits((143 << 23) | (1023 << 13));
    h = sk_float_to_half(overflow);
    f = sk_half_to_float(h);
    check!(r, !is_finite(f));
    let underflow = f32::from_bits(101 << 23);
    h = sk_float_to_half(underflow);
    f = sk_half_to_float(h);
    check!(r, f == 0.);
    let inf = f32::from_bits(255 << 23);
    h = sk_float_to_half(inf);
    f = sk_half_to_float(h);
    check!(r, !is_finite(f));
    let nan = f32::from_bits((255 << 23) | 1);
    h = sk_float_to_half(nan);
    f = sk_half_to_float(h);
    check!(r, is_nan(f));
}
fn test_rsqrt(r: &mut Reporter, rsqrt: impl Fn(f32) -> f32) {
    let max_relative_error = 6.50196699e-4f32;
    let mut input = 0.000001f32;
    for _ in 0..1000 {
        let exact = 1. / input.sqrt();
        let estimate = rsqrt(input);
        let relative_error = (exact - estimate).abs() / exact;
        check!(r, relative_error <= max_relative_error);
        input += 0.001;
    }
    input = 1.;
    for _ in 0..1000 {
        let exact = 1. / input.sqrt();
        let estimate = rsqrt(input);
        let relative_error = (exact - estimate).abs() / exact;
        check!(r, relative_error <= max_relative_error);
        input += 0.01;
    }
    input = 1000000.;
    for _ in 0..100 {
        let exact = 1. / input.sqrt();
        let estimate = rsqrt(input);
        let relative_error = (exact - estimate).abs() / exact;
        check!(r, relative_error <= max_relative_error);
        input += 754326.;
    }
}
fn test_muldiv255(r: &mut Reporter) {
    for a in 0..=255 {
        for b in 0..=255 {
            let ab = a * b;
            let s = ab as f32 / 255.;
            let round = (s + 0.5).floor() as i32;
            let trunc = s.floor() as i32;
            let iround = sk_mul_div255_round(a as u16, b as u16) as i32;
            let itrunc = sk_mul_div255_trunc(a as u8, b as u8) as i32;
            check!(r, iround == round);
            check!(r, itrunc == trunc);
            check!(r, itrunc <= iround);
            check!(r, iround <= a);
            check!(r, iround <= b);
        }
    }
}
fn test_muldiv255ceiling(r: &mut Reporter) {
    for c in 0..=255 {
        for a in 0..=255 {
            let product = c * a + 255;
            let expected = (product + (product >> 8)) >> 8;
            let webkit = (c * a + 254) / 255;
            check!(r, expected == webkit);
            let skia = sk_mul_div255_ceiling(c as u8, a as u8) as i32;
            check!(r, skia == webkit);
        }
    }
}
fn test_copysign(r: &mut Reporter) {
    let triples = [
        0, 0, 0, 0, 1, 0, 0, -1, 0, 1, 0, 1, 1, 1, 1, 1, -1, -1, -1, 0, 1, -1, 1, 1, -1, -1, -1,
    ];
    for i in (0..triples.len()).step_by(3) {
        check!(
            r,
            sk_copy_sign32(triples[i], triples[i + 1]) == triples[i + 2]
        );
        let x = triples[i] as f32;
        let y = triples[i + 1] as f32;
        let expected = triples[i + 2] as f32;
        check!(r, x.copysign(y) == expected);
    }
    let mut rand = SkRandom::new();
    for _ in 0..1000 {
        let ix = rand.next_s();
        check!(r, sk_copy_sign32(ix, ix) == ix);
        check!(r, sk_copy_sign32(ix, -ix) == -ix);
        check!(r, sk_copy_sign32(-ix, ix) == ix);
        check!(r, sk_copy_sign32(-ix, -ix) == -ix);
        let sx = rand.next_s_scalar1();
        check!(r, scalar_copy_sign(sx, sx) == sx);
        check!(r, scalar_copy_sign(sx, -sx) == -sx);
        check!(r, scalar_copy_sign(-sx, sx) == sx);
        check!(r, scalar_copy_sign(-sx, -sx) == -sx);
    }
}
fn huge_vector_normalize(r: &mut Reporter) {
    let fail = [
        Point::new(0., 0.),
        Point::new(SCALAR_INFINITY, 0.),
        Point::new(0., SCALAR_INFINITY),
        Point::new(0., SCALAR_NAN),
        Point::new(SCALAR_NAN, 0.),
    ];
    for mut v in fail {
        let mut v2 = v;
        if v2.set_length(1.) {
            check!(r, !v.set_length(1.));
        }
    }
}
// tests/MathTest.cpp:369
#[test]
fn PopCount() {
    let mut r = Reporter::default();
    {
        let test_val = 0;
        check!(r, sk_pop_count(test_val) == 0);
    }
    for i in 0..32 {
        let mut test_val = 1u32 << i;
        check!(r, sk_pop_count(test_val) == 1);
        test_val ^= 0xffffffff;
        check!(r, sk_pop_count(test_val) == 31);
    }
    {
        let test_val = 0xffffffff;
        check!(r, sk_pop_count(test_val) == 32);
    }
    let mut rand = SkRandom::new();
    for _ in 0..100 {
        let mut expected = 0;
        let mut value = 0u32;
        let tries = rand.next_u_less_than(33);
        for _ in 0..tries {
            let bit = rand.next_range_u(0, 31);
            if value & (1 << bit) != 0 {
                continue;
            }
            expected += 1;
            value |= 1 << bit;
        }
        check!(r, sk_pop_count(value) == expected);
    }
    r.finish();
}
fn test_clz(r: &mut Reporter) {
    let mut expect = |value, count| {
        check!(r, sk_clz(value) == count);
    };
    expect(0, 32);
    for i in 0..32 {
        let value = 0xffffffffu32;
        expect(value >> i, i);
    }
}
// Original test_floor and test_blend31 are reached only through literal-false guards.
fn sk_fsel(pred: f32, ge: f32, lt: f32) -> f32 {
    if pred >= 0. { ge } else { lt }
}
fn fast_floor(x: f32) -> f32 {
    let big = sk_fsel(x, (1 << 23) as f32, -((1 << 23) as f32));
    (x + big) - big
}
fn test_floor(r: &mut Reporter) {
    let vals = [
        0., 1., 1.1, 1.01, 1.001, 1.0001, 1.00001, 1.000001, 1.0000001,
    ];
    for value in vals {
        let fast = fast_floor(value);
        let std = value.floor();
        if std != fast {
            check!(r, false);
        }
    }
    // Original negative-value call is commented out.
}
fn blend31(src: i32, dst: i32, a: i32) -> i32 {
    dst + (((src - dst) * a * 2114) >> 16)
}
fn blend31_slow(src: i32, dst: i32, a: i32) -> i32 {
    let prod = src * a + (31 - a) * dst + 16;
    (prod + (prod >> 5)) >> 5
}
fn blend31_round(src: i32, dst: i32, a: i32) -> i32 {
    let prod = (src - dst) * a + 16;
    dst + ((prod + (prod >> 5)) >> 5)
}
fn blend31_old(src: i32, dst: i32, mut a: i32) -> i32 {
    a += a >> 4;
    dst + (((src - dst) * a) >> 5)
}
fn test_blend31() {
    let mut failed = 0;
    let mut death = 0;
    let blend_functions = [blend31, blend31_slow, blend31_round, blend31_old];
    if false {
        failed = blend_functions[0](0, 0, 0);
    }
    for src in 0..=255 {
        for dst in 0..=255 {
            for a in 0..=31 {
                let r0 = blend31_slow(src, dst, a);
                let f = dst as f32 + (src - dst) as f32 * (a as f32 / 31.);
                let r1 = f as i32;
                let r2 = scalar_round_to_int(f);
                if r0 != r1 && r0 != r2 {
                    eprintln!("src:{src} dst:{dst} a:{a} result:{r0} float:{f}");
                    failed += 1;
                }
                if r0 > 255 {
                    death += 1;
                    eprintln!("death src:{src} dst:{dst} a:{a} result:{r0} float:{f}");
                }
            }
        }
    }
    eprintln!("---- failed {failed} death {death}");
}
// tests/MathTest.cpp:421
#[test]
fn Math() {
    let mut r = Reporter::default();
    let mut rand = SkRandom::new();
    // Original #if 0 invalid SkTo assertion probes remain disabled (supplemental probes are separate).
    test_muldiv255(&mut r);
    test_muldiv255ceiling(&mut r);
    test_copysign(&mut r);
    {
        let x = SCALAR_NAN;
        check!(r, is_nan(x));
    }
    for _ in 0..10000 {
        let mut p = Point::default();
        p.set_length_xy(rand.next_s() as f32, rand.next_s() as f32, SCALAR_1);
        check_length(&mut r, p, SCALAR_1);
        p.set_length_xy(
            (rand.next_s() >> 13) as f32,
            (rand.next_s() >> 13) as f32,
            SCALAR_1,
        );
        check_length(&mut r, p, SCALAR_1);
    }
    {
        let mut result = sk_fixed_div(100, 100);
        check!(r, result == SK_FIXED_1);
        result = sk_fixed_div(1, SK_FIXED_1);
        check!(r, result == 1);
        result = sk_fixed_div(10 - 1, SK_FIXED_1 * 3);
        check!(r, result == 3);
    }
    {
        check!(
            r,
            (sk_fixed_round_to_fixed(-SK_FIXED_1 * 10) >> 1) == -SK_FIXED_1 * 5
        );
        check!(
            r,
            (sk_fixed_floor_to_fixed(-SK_FIXED_1 * 10) >> 1) == -SK_FIXED_1 * 5
        );
        check!(
            r,
            (sk_fixed_ceil_to_fixed(-SK_FIXED_1 * 10) >> 1) == -SK_FIXED_1 * 5
        );
    }
    huge_vector_normalize(&mut r);
    unittest_isfinite!(&mut r, f32);
    unittest_isfinite!(&mut r, f64);
    unittest_half(&mut r);
    test_rsqrt(&mut r, float_rsqrt);
    for _ in 0..10000 {
        let numer = rand.next_s();
        let denom = rand.next_s();
        let result = sk_fixed_div(numer, denom);
        let mut expected = left_shift_i64(numer as i64, 16) / denom as i64;
        let _ = sk_clz(numer as u32);
        let _ = sk_clz(denom as u32);
        check!(r, result != SK_NAN32);
        if expected > SK_MAX_S32 as i64 {
            expected = SK_MAX_S32 as i64;
        } else if expected < -SK_MAX_S32 as i64 {
            expected = SK_MIN_S32 as i64;
        }
        if result != expected as i32 {
            check!(r, false);
        }
        check!(r, result == expected as i32);
    }
    if false {
        test_floor(&mut r);
    }
    if false {
        test_blend31();
    }
    test_clz(&mut r);
    r.finish();
}
// tests/MathTest.cpp:525. Const expressions retain the template-swap compile-time checks.
#[test]
fn TestEndian() {
    let mut r = Reporter::default();
    const SWAP16: u16 = rust_skia::codec::sk_bswap16(0x2211);
    const SWAP32: u32 = rust_skia::codec::sk_bswap32(0x44332211);
    const SWAP64: u64 = rust_skia::codec::sk_bswap64(0x8877665544332211);
    check!(r, 0x1122 == SWAP16);
    check!(r, 0x11223344 == SWAP32);
    check!(r, 0x1122334455667788 == SWAP64);
    for (yin, yang) in [(0u16, 0), (0xffff, 0xffff), (0x1122, 0x2211)] {
        check!(r, yang == sk_bswap16(yin));
    }
    for (yin, yang) in [
        (0u32, 0),
        (0xffffffff, 0xffffffff),
        (0x11223344, 0x44332211),
    ] {
        check!(r, yang == rust_skia::codec::sk_bswap32(yin));
    }
    for (yin, yang) in [
        (0u64, 0),
        (0xffffffffffffffff, 0xffffffffffffffff),
        (0x1122334455667788, 0x8877665544332211),
    ] {
        check!(r, yang == sk_bswap64(yin));
    }
    r.finish();
}
// tests/MathTest.cpp:558 helper, instantiated by registrations at 598..626.
macro_rules! divmod_test {
    ($name:ident,$t:ty,$function:path) => {
        #[test]
        fn $name() {
            let mut r = Reporter::default();
            let edge = [
                (17i32, 17i32),
                (17, 4),
                (0, 17),
                (-17, -17),
                (-17, 4),
                (17, -4),
                (-17, -4),
            ];
            for (n, d) in edge {
                let numer = n as $t;
                let denom = d as $t;
                let (div, modulo) = $function(numer, denom);
                check!(r, numer / denom == div);
                check!(r, numer % denom == modulo);
            }
            let mut rand = SkRandom::new();
            for _ in 0..10000 {
                let numer = rand.next_s() as $t;
                let mut denom = 0 as $t;
                while denom == 0 {
                    denom = rand.next_s() as $t;
                }
                let (div, modulo) = $function(numer, denom);
                check!(r, numer / denom == div);
                check!(r, numer % denom == modulo);
            }
            r.finish();
        }
    };
}
// Missing width-specific production APIs must be supplied by Deepseek, never emulated here.
divmod_test!(divmod_u8, u8, rust_skia::math_priv::sk_tdivmod_u8);
divmod_test!(divmod_u16, u16, rust_skia::math_priv::sk_tdivmod_u16);
divmod_test!(divmod_u32, u32, rust_skia::math_priv::sk_tdivmod_u32);
divmod_test!(divmod_u64, u64, rust_skia::math_priv::sk_tdivmod_u64);
divmod_test!(divmod_s8, i8, rust_skia::math_priv::sk_tdivmod_i8);
divmod_test!(divmod_s16, i16, rust_skia::math_priv::sk_tdivmod_i16);
divmod_test!(divmod_s32, i32, sk_tdivmod_i32);
divmod_test!(divmod_s64, i64, sk_tdivmod_i64);
fn test_nextsizepow2(r: &mut Reporter, test: usize, expected: usize) {
    let ans = sk_next_size_pow2(test);
    check!(r, ans == expected);
}
// tests/MathTest.cpp:637
#[test]
fn SkNextSizePow2() {
    let mut r = Reporter::default();
    let mut test = 0usize;
    let mut expected = 1usize;
    test_nextsizepow2(&mut r, test, expected);
    test = 1;
    expected = 1;
    for _ in 1..usize::BITS {
        test_nextsizepow2(&mut r, test, expected);
        test += 1;
        expected <<= 1;
        test_nextsizepow2(&mut r, test, expected);
        test = expected;
    }
    test = 1;
    test <<= usize::BITS - 1;
    test_nextsizepow2(&mut r, test, test);
    test += 1;
    test_nextsizepow2(&mut r, test, test);
    test_nextsizepow2(&mut r, usize::MAX, usize::MAX);
    r.finish();
}
// tests/MathTest.cpp:668
#[test]
fn FloatSaturate32() {
    let mut r = Reporter::default();
    let recs = [
        (0f32, 0),
        (100.5, 100),
        (SK_MAX_S32 as f32, SK_MAX_S32_FITS_IN_FLOAT),
        (SK_MIN_S32 as f32, SK_MIN_S32_FITS_IN_FLOAT),
        (SK_MAX_S32 as f32 * 100., SK_MAX_S32_FITS_IN_FLOAT),
        (SK_MIN_S32 as f32 * 100., SK_MIN_S32_FITS_IN_FLOAT),
        (SCALAR_INFINITY, SK_MAX_S32_FITS_IN_FLOAT),
        (SCALAR_NEGATIVE_INFINITY, SK_MIN_S32_FITS_IN_FLOAT),
        (SCALAR_NAN, SK_MAX_S32_FITS_IN_FLOAT),
    ];
    for (f, expected) in recs {
        let i = float_saturate_to_int(f);
        check!(r, expected == i);
        let p = rust_skia::scalar::scalar_pin(f, 0., 100.);
        check!(r, p >= 0. && p <= 100.);
    }
    r.finish();
}
// tests/MathTest.cpp:694
#[test]
fn FloatSaturate64() {
    let mut r = Reporter::default();
    let recs = [
        (0f32, 0),
        (100.5, 100),
        (SK_MAX_S64 as f32, SK_MAX_S64_FITS_IN_FLOAT),
        (SK_MIN_S64 as f32, SK_MIN_S64_FITS_IN_FLOAT),
        (SK_MAX_S64 as f32 * 100., SK_MAX_S64_FITS_IN_FLOAT),
        (SK_MIN_S64 as f32 * 100., SK_MIN_S64_FITS_IN_FLOAT),
        (SCALAR_INFINITY, SK_MAX_S64_FITS_IN_FLOAT),
        (SCALAR_NEGATIVE_INFINITY, SK_MIN_S64_FITS_IN_FLOAT),
        (SCALAR_NAN, SK_MAX_S64_FITS_IN_FLOAT),
    ];
    for (f, expected) in recs {
        let i = float_saturate_to_int64(f);
        check!(r, expected == i);
    }
    r.finish();
}
// tests/MathTest.cpp:716
#[test]
fn SkNextPow2() {
    let mut r = Reporter::default();
    for (input, expected) in [
        (1, 1),
        (2, 2),
        (3, 4),
        (4, 4),
        (5, 8),
        (1073741822, 1073741824),
        (1073741823, 1073741824),
        (1073741824, 1073741824),
    ] {
        let actual = sk_next_pow2(input);
        check!(r, expected == actual);
        check!(r, actual == sk_next_pow2(input));
    }
    for i in 6..63356 {
        let actual = sk_next_pow2(i);
        let expected = 2f32.powf(((i as f32).ln() / 2f32.ln()).ceil()) as i32;
        check!(r, expected == actual);
        check!(r, actual == sk_next_pow2(i));
    }
    r.finish();
}
// tests/MathTest.cpp:750
#[test]
fn DoubleSaturate32() {
    let mut r = Reporter::default();
    for (f, expected) in [
        (0f64, 0),
        (100.5, 100),
        (SK_MAX_S32 as f64, SK_MAX_S32),
        (SK_MIN_S32 as f64, SK_MIN_S32),
        ((SK_MAX_S32 - 1) as f64, SK_MAX_S32 - 1),
        ((SK_MIN_S32 + 1) as f64, SK_MIN_S32 + 1),
        (SK_MAX_S32 as f64 * 100., SK_MAX_S32),
        (SK_MIN_S32 as f64 * 100., SK_MIN_S32),
        (SCALAR_INFINITY as f64, SK_MAX_S32),
        (SCALAR_NEGATIVE_INFINITY as f64, SK_MIN_S32),
        (SCALAR_NAN as f64, SK_MAX_S32),
    ] {
        let i = double_saturate_to_int(f);
        check!(r, expected == i);
    }
    r.finish();
}
// tests/MathTest.cpp:777; original __ARM_NEON guard. Native intrinsic test, not a scalar replacement.
#[cfg(target_arch = "aarch64")]
#[test]
fn NeonU16Div255() {
    use std::arch::aarch64::*;
    let mut r = Reporter::default();
    for v in 0..=255 * 255 {
        let want = (v + 127) / 255;
        let got = unsafe {
            let value = vdupq_n_u16(v as u16);
            vgetq_lane_u16::<0>(vrshrq_n_u16::<8>(vrsraq_n_u16::<8>(value, value)))
        } as i32;
        if got != want {
            eprintln!("{v} -> {got}, want {want}");
        }
        check!(r, got == want);
    }
    r.finish();
}
