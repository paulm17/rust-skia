// Supplemental contracts, separate from faithful upstream registrations.
// Source contracts: pinned b81c69985fd14bb7bfbd356b7618633ce07dc1a4 headers named below.
use super::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
// SkFixed.h, SkFDot6.h: representable endpoints, ties, signed shifts, fast/fallback division.
#[test]
fn supplemental_fixed_and_fdot6_boundaries() {
    let mut r = Reporter::default();
    check!(r, SK_FIXED_1 == 65536);
    check!(r, SK_FIXED_HALF == 32768);
    check!(r, SK_FIXED_QUARTER == 16384);
    check!(r, SK_FIXED_MAX == i32::MAX);
    check!(r, SK_FIXED_MIN == -i32::MAX);
    check!(r, SK_FIXED_PI == 0x3243f);
    check!(r, SK_FIXED_SQRT2 == 92682);
    check!(r, SK_FIXED_TAN_PI_OVER_8 == 0x6a0a);
    check!(r, SK_FIXED_ROOT2_OVER2 == 0xb505);
    check!(r, SK_FIXED_3232_MAX == i64::MAX);
    check!(r, SK_FIXED_3232_MIN == -i64::MAX);
    for n in [-32768, -32767, -1, 0, 1, 32767] {
        let expected = ((n as u32) << 16) as i32;
        check!(r, sk_int_to_fixed(n) == expected);
        check!(r, sk_fixed_to_double(expected) == n as f64);
        check!(r, sk_double_to_fixed(n as f64) == expected);
        check!(r, sk_fixed3232_to_int(sk_int_to_fixed3232(n)) == n);
    }
    for x in [
        -131073, -98305, -98304, -98303, -65537, -65536, -32769, -32768, -32767, -1, 0, 1, 32767,
        32768, 32769, 65535, 65536, 98303, 98304, 98305, 131073,
    ] {
        check!(
            r,
            sk_fixed_floor_to_int(x) == (x as f64 / 65536.).floor() as i32
        );
        check!(
            r,
            sk_fixed_ceil_to_int(x) == (x as f64 / 65536.).ceil() as i32
        );
        check!(
            r,
            sk_fixed_round_to_int(x) == (x as f64 / 65536. + 0.5).floor() as i32
        );
        check!(
            r,
            sk_fixed_floor_to_fixed(x) == sk_fixed_floor_to_int(x) * 65536
        );
        check!(
            r,
            sk_fixed_ceil_to_fixed(x) == sk_fixed_ceil_to_int(x) * 65536
        );
        check!(
            r,
            sk_fixed_round_to_fixed(x) == sk_fixed_round_to_int(x) * 65536
        );
        check!(r, sk_fixed3232_to_fixed(sk_fixed_to_fixed3232(x)) == x);
        check!(
            r,
            sk_fixed_to_scalar(x).to_bits() == sk_fixed_to_float(x).to_bits()
        );
    }
    for a in [
        i32::MIN,
        -2147483647,
        -65536,
        -32768,
        -1,
        0,
        1,
        32767,
        32768,
        65536,
        i32::MAX,
    ] {
        for b in [-2147483647, -65536, -1, 1, 65536, i32::MAX] {
            let expected = (((a as i64) << 16) / b as i64).clamp(-2147483647, 2147483647) as i32;
            check!(r, sk_fixed_div(a, b) == expected);
            // The i16 fast path is undefined for INT16_MIN divided by -1 after shifting.
            if !(a == -32768 && b == -1) {
                // SkFDot6Div's i16 fast path does not apply SkFixedDiv's sentinel clamp.
                let fdot6_expected = if i16::try_from(a).is_ok() {
                    (((a as i64) << 16) / b as i64) as i32
                } else {
                    expected
                };
                check!(r, sk_fdot6_div(a, b) == fdot6_expected);
            }
            check!(
                r,
                sk_fixed_mul(a, b) == ((a as i64 * b as i64) >> 16) as i32
            );
        }
    }
    // Do not assign behavior to signed C++ overflow in Ave/round/ceil; defined boundary inputs only.
    check!(r, sk_fixed_ave(-65536, 65536) == 0);
    check!(r, sk_fixed_ave(-3, 0) == -2);
    for (x, expected) in [(-1.5f32, -98304), (-0., 0), (0., 0), (1.5, 98304)] {
        check!(r, sk_float_to_fixed(x) == expected);
        check!(r, sk_float_to_fixed_check(x) == expected);
        check!(r, sk_scalar_to_fixed(x) == expected);
        check!(
            r,
            sk_float_to_fixed3232(x) == (x as f64 * 4294967296.) as i64
        );
        check!(
            r,
            sk_scalar_to_fixed3232(x) == (x as f64 * 4294967296.) as i64
        );
    }
    check!(
        r,
        sk_float_to_fixed(f32::INFINITY) == SK_MAX_S32_FITS_IN_FLOAT
    );
    check!(
        r,
        sk_float_to_fixed(f32::NEG_INFINITY) == SK_MIN_S32_FITS_IN_FLOAT
    );
    check!(r, sk_float_to_fixed(f32::NAN) == SK_MAX_S32_FITS_IN_FLOAT);
    check!(r, sk_fixed3232_to_float(1i64 << 32) == 1.);
    for shift in [0, 1, 4, 8] {
        for (numer, expected) in [(-5, -2), (-3, -2), (-1, 0), (1, 0), (3, 2), (5, 2)] {
            let x = numer as f32 / ((128u32) << shift) as f32;
            check!(r, sk_scalar_round_to_fdot6(x, shift) == expected);
        }
    }
    for x in [-65, -64, -33, -32, -31, -1, 0, 1, 31, 32, 33, 63, 64, 65] {
        check!(r, sk_fdot6_floor(x) == (x as f64 / 64.).floor() as i32);
        check!(r, sk_fdot6_ceil(x) == (x as f64 / 64.).ceil() as i32);
        check!(
            r,
            sk_fdot6_round(x) == (x as f64 / 64. + 0.5).floor() as i32
        );
        check!(r, sk_fixed_to_fdot6(sk_fdot6_to_fixed(x)) == x);
        check!(r, sk_float_to_fdot6(sk_fdot6_to_float(x)) == x);
        check!(r, sk_scalar_to_fdot6(sk_fdot6_to_scalar(x)) == x);
    }
    check!(r, sk_int_to_fdot6(i32::MIN >> 6) == i32::MIN);
    check!(r, sk_int_to_fdot6(i32::MAX >> 6) == 2147483584);
    #[cfg(debug_assertions)]
    {
        check!(
            r,
            std::panic::catch_unwind(|| sk_int_to_fixed(32768)).is_err()
        );
        check!(
            r,
            std::panic::catch_unwind(|| sk_float_to_fixed_check(32768.)).is_err()
        );
        check!(
            r,
            std::panic::catch_unwind(|| sk_fdot6_to_fixed(1 << 21)).is_err()
        );
    }
    r.finish();
}
// SkSafe32.h and SkMath.h: Skia minima exclude the sentinel INT_MIN.
#[test]
fn supplemental_integer_widths_overflow_and_bits() {
    let mut r = Reporter::default();
    check!(r, SK_MIN_S16 == -32767);
    check!(r, SK_MIN_S32 == -2147483647);
    check!(r, SK_NAN32 == i32::MIN);
    check!(r, SK_MIN_S64 == -i64::MAX);
    for a in [i32::MIN, -2147483647, -1, 0, 1, i32::MAX] {
        for b in [i32::MIN, -2147483647, -1, 0, 1, i32::MAX] {
            check!(
                r,
                sat_add(a, b) == (a as i64 + b as i64).clamp(-2147483647, 2147483647) as i32
            );
            check!(
                r,
                sat_sub(a, b) == (a as i64 - b as i64).clamp(-2147483647, 2147483647) as i32
            );
            check!(r, sk32_can_overflow_add(a, b) == a.wrapping_add(b));
            check!(r, sk32_can_overflow_sub(a, b) == a.wrapping_sub(b));
        }
    }
    for value in [
        i64::MIN,
        -2147483648,
        -2147483647,
        -1,
        0,
        1,
        2147483647,
        2147483648,
        i64::MAX,
    ] {
        check!(
            r,
            pin_to_s32(value) == value.clamp(-2147483647, 2147483647) as i32
        );
    }
    for value in [-2147483647, -1, 0, 1, i32::MAX] {
        check!(r, sk_abs32(value) == value.abs());
        check!(r, sk_extract_sign(value) == if value < 0 { -1 } else { 0 });
        check!(r, sk_apply_sign(value, 0) == value);
    }
    for shift in 0..32 {
        check!(r, left_shift_i32(-1, shift) == (u32::MAX << shift) as i32);
    }
    for shift in 0..64 {
        check!(r, left_shift_i64(-1, shift) == (u64::MAX << shift) as i64);
    }
    check!(r, sk_clz(0) == 32);
    check!(r, sk_ctz(0) == 32);
    check!(r, sk_is_pow2(0u32));
    for bit in 0..32 {
        let v = 1u32 << bit;
        check!(r, sk_clz(v) == 31 - bit);
        check!(r, sk_ctz(v) == bit);
        check!(r, sk_is_pow2(v));
        check!(r, sk_next_log2(v) == bit);
        check!(r, sk_prev_log2(v) == bit);
    }
    for v in [0, 1, 2, 3, 4, 15, 16, 17, 2147395600, i32::MAX] {
        let expected = (v as f64).sqrt().floor() as i32;
        check!(r, sk_sqrt32(v) == expected);
    }
    for v in [-1, 0, 1, i32::MAX] {
        check!(r, sk_clamp_pos(v) == v.max(0));
        check!(r, sk_negate_to_size_t(v) == (-(v as i64)) as usize);
    }
    check!(r, sk_clamp_u_max(u32::MAX, 1) == 1);
    check!(r, sk_clamp_u_max(0, 1) == 0);
    check!(
        r,
        sk_64_mul(i32::MAX as i64, i32::MAX as i64) == 4611686014132420609
    );
    for a in [0u16, 1, 255, 32767] {
        for b in [0u16, 1, 255, 32767] {
            for shift in 1..=8 {
                let prod = a as u32 * b as u32 + (1u32 << (shift - 1));
                let expected = (prod + (prod >> shift)) >> shift;
                check!(r, sk_mul16_shift_round(a, b, shift) == expected);
            }
        }
    }
    for prod in 0..=255 * 255 {
        check!(r, sk_div255_round(prod) == (prod + 127) / 255);
    }
    #[cfg(debug_assertions)]
    {
        check!(r, std::panic::catch_unwind(|| sk_abs32(i32::MIN)).is_err());
    }
    r.finish();
}
// SkSafeMath.h: sticky failure, truncated returns, alignment/lcm and invalid-cast result contracts.
#[test]
fn supplemental_safe_math_boundaries() {
    let mut r = Reporter::default();
    for a in [0usize, 1, usize::MAX / 2, usize::MAX / 2 + 1, usize::MAX] {
        for b in [0usize, 1, 2, usize::MAX] {
            let mut safe = SkSafeMath::new();
            let expected = a as u128 + b as u128;
            check!(r, safe.add(a, b) == expected as usize);
            check!(r, safe.ok() == (expected <= usize::MAX as u128));
            let mut safe = SkSafeMath::new();
            let expected = a as u128 * b as u128;
            check!(r, safe.mul(a, b) == expected as usize);
            check!(r, safe.ok() == (expected <= usize::MAX as u128));
            let mut safe = SkSafeMath::new();
            check!(r, safe.sub(a, b) == a.wrapping_sub(b));
            check!(r, safe.ok() == (a >= b));
            check!(r, SkSafeMath::add_saturating(a, b) == a.saturating_add(b));
            check!(r, SkSafeMath::mul_saturating(a, b) == a.saturating_mul(b));
        }
    }
    let mut safe = SkSafeMath::new();
    safe.add(usize::MAX, 1);
    check!(r, !safe.ok());
    check!(r, safe.add(1, 2) == 3);
    check!(r, !safe.ok());
    for alignment in [1usize, 2, 4, 8, 4096] {
        for x in [
            0usize,
            1,
            alignment - 1,
            alignment,
            alignment + 1,
            usize::MAX - alignment + 1,
            usize::MAX,
        ] {
            let mut safe = SkSafeMath::new();
            let wide = x as u128 + alignment as u128 - 1;
            let expected = (wide as usize) & !(alignment - 1);
            check!(r, safe.align_up(x, alignment) == expected);
            check!(r, safe.ok() == (wide <= usize::MAX as u128));
        }
    }
    for (x, a, expected) in [(0, 3, 0), (1, 3, 3), (3, 3, 3), (4, 3, 6)] {
        let mut safe = SkSafeMath::new();
        check!(r, safe.align_up_non_pow2(x, a) == expected);
        check!(r, safe.ok());
    }
    let mut safe = SkSafeMath::new();
    check!(r, safe.align_up_non_pow2(5, 0) == 0);
    check!(r, !safe.ok());
    for (a, b, want, ok) in [
        (3, 4, 12, true),
        (6, 8, 24, true),
        (0, 1, 0, false),
        (1, 0, 0, false),
        (usize::MAX, 2, usize::MAX - 1, false),
    ] {
        let mut safe = SkSafeMath::new();
        check!(r, safe.lcm(a, b) == want);
        check!(r, safe.ok() == ok);
    }
    let mut safe = SkSafeMath::new();
    check!(r, safe.cast_to_i32(2147483648u64) == i32::MIN);
    check!(r, !safe.ok());
    let mut safe = SkSafeMath::new();
    check!(r, safe.cast_to_usize(-1i64) == usize::MAX);
    check!(r, !safe.ok());
    for (a, b) in [(i32::MIN, -1), (i32::MAX, 0), (0, 0)] {
        let mut safe = SkSafeMath::new();
        check!(r, safe.div_int(a, b) == a);
        check!(r, !safe.ok());
        let mut safe = SkSafeMath::new();
        check!(r, safe.mod_int(a, b) == a);
        check!(r, !safe.ok());
    }
    r.finish();
}
// SkTo.h and SkTFitsIn.h. Debug precondition assertions become catchable Rust panics.
#[test]
fn supplemental_conversion_boundaries() {
    let mut r = Reporter::default();
    macro_rules! signed {
        ($t:ty,$checked:path,$fits:path,$to:path) => {{
            for n in [
                <$t>::MIN as i128 - 1,
                <$t>::MIN as i128,
                -1,
                0,
                1,
                <$t>::MAX as i128,
                <$t>::MAX as i128 + 1,
            ] {
                let fits = n >= <$t>::MIN as i128 && n <= <$t>::MAX as i128;
                check!(r, $fits(n) == fits);
                check!(r, $checked(n) == if fits { Some(n as $t) } else { None });
                let result = std::panic::catch_unwind(|| $to(n));
                if fits {
                    check!(r, result.ok() == Some(n as $t));
                } else if cfg!(debug_assertions) {
                    check!(r, result.is_err());
                } else {
                    check!(r, result.ok() == Some(n as $t));
                }
            }
        }};
    }
    macro_rules! unsigned {
        ($t:ty,$checked:path,$fits:path,$to:path) => {{
            for n in [-1i128, 0, 1, <$t>::MAX as i128, <$t>::MAX as i128 + 1] {
                let fits = n >= 0 && n <= <$t>::MAX as i128;
                check!(r, $fits(n) == fits);
                check!(r, $checked(n) == if fits { Some(n as $t) } else { None });
                let result = std::panic::catch_unwind(|| $to(n));
                if fits {
                    check!(r, result.ok() == Some(n as $t));
                } else if cfg!(debug_assertions) {
                    check!(r, result.is_err());
                } else {
                    check!(r, result.ok() == Some(n as $t));
                }
            }
        }};
    }
    signed!(i8, checked_to_i8, sk_tfits_in_i8, sk_to_s8);
    signed!(i16, checked_to_i16, sk_tfits_in_i16, sk_to_s16);
    signed!(i32, checked_to_i32, sk_tfits_in_i32, sk_to_s32);
    signed!(i64, checked_to_i64, sk_tfits_in_i64, sk_to_s64);
    unsigned!(u8, checked_to_u8, sk_tfits_in_u8, sk_to_u8);
    unsigned!(u16, checked_to_u16, sk_tfits_in_u16, sk_to_u16);
    unsigned!(u32, checked_to_u32, sk_tfits_in_u32, sk_to_u32);
    unsigned!(u64, checked_to_u64, sk_tfits_in_u64, sk_to_u64);
    unsigned!(usize, checked_to_usize, sk_tfits_in_usize, sk_to_size_t);
    check!(r, sk_to_bool(true));
    check!(r, !sk_to_bool(false));
    check!(r, sk_to_int(-1i64) == -1);
    check!(r, sk_to_uint(u32::MAX as u64) == u32::MAX);
    r.finish();
}
// An independent test oracle transcribed from the pinned SkRandom.h, not used by production.
#[derive(Clone)]
struct RandomOracle {
    k: u32,
    j: u32,
}
impl RandomOracle {
    fn new(seed: u32) -> Self {
        let lcg = |x: u32| x.wrapping_mul(1664525).wrapping_add(1013904223);
        let mut k = lcg(seed);
        if k == 0 {
            k = lcg(k);
        }
        let mut j = lcg(k);
        if j == 0 {
            j = lcg(j);
        }
        Self { k, j }
    }
    fn next(&mut self) -> u32 {
        self.k = 30345 * (self.k & 65535) + (self.k >> 16);
        self.j = 18000 * (self.j & 65535) + (self.j >> 16);
        self.k.rotate_left(16).wrapping_add(self.j)
    }
}
#[test]
fn supplemental_random_seed_advancement() {
    let mut r = Reporter::default();
    let zero_k = 0u32.wrapping_sub(1013904223).wrapping_mul(4276115653);
    let zero_j = zero_k.wrapping_sub(1013904223).wrapping_mul(4276115653);
    for seed in [0, 1, 1234, 123456, u32::MAX, zero_k, zero_j] {
        let mut rng = SkRandom::from_seed(seed);
        let mut oracle = RandomOracle::new(seed);
        for _ in 0..1024 {
            check!(r, rng.next_u() == oracle.next());
        }
        let mut clone = rng.clone();
        for _ in 0..64 {
            check!(r, clone.next_u() == rng.next_u());
        }
        rng.set_seed(seed);
        oracle = RandomOracle::new(seed);
        check!(r, rng.next_range_u(0, u32::MAX) == oracle.next());
        for bits in [1, 16, 31, 32] {
            check!(r, rng.next_bits(bits) == oracle.next() >> (32 - bits));
        }
        check!(r, rng.next_s() == oracle.next() as i32);
        check!(
            r,
            rng.next_f().to_bits()
                == (f32::from_bits(0x3f800000 | (oracle.next() >> 9)) - 1.).to_bits()
        );
        check!(
            r,
            rng.next_range_f(-3., 7.)
                == -3. + (f32::from_bits(0x3f800000 | (oracle.next() >> 9)) - 1.) * 10.
        );
        check!(r, rng.next_range_u(17, 272) == 17 + oracle.next() % 256);
        check!(r, rng.next_range_u(9, 9) == 9);
        let _ = oracle.next();
        check!(r, rng.next_u_less_than(1) == 0);
        let _ = oracle.next();
        check!(r, rng.next_u_fixed1() == (oracle.next() >> 16) as i32);
        check!(r, rng.next_s_fixed1() == (oracle.next() as i32) >> 15);
        check!(
            r,
            rng.next_u_scalar1() == (oracle.next() >> 16) as f32 / 65536.
        );
        check!(
            r,
            rng.next_s_scalar1() == ((oracle.next() as i32) >> 15) as f32 / 65536.
        );
        check!(
            r,
            rng.next_range_scalar(-3., 7.) == ((oracle.next() >> 16) as f32 / 65536.) * 10. - 3.
        );
        check!(r, rng.next_bool() == (oracle.next() >= 0x80000000));
        for fraction in [0., 0.5, 1.] {
            check!(
                r,
                rng.next_biased_bool(fraction)
                    == ((oracle.next() >> 16) as f32 / 65536. <= fraction)
            );
        }
        check!(r, rng.next_u() == oracle.next());
    }
    let mut a = SkRandom::default();
    let mut b = SkRandom::from_seed(0);
    for _ in 0..64 {
        check!(r, a.next_u() == b.next_u());
    }
    r.finish();
}
#[test]
fn supplemental_signed_zero_nan_half_and_ulps() {
    let mut r = Reporter::default();
    for (value, half) in [
        (0f32, 0u16),
        (-0., 0x8000),
        (1., 0x3c00),
        (-1., 0xbc00),
        (f32::INFINITY, 0x7c00),
        (f32::NEG_INFINITY, 0xfc00),
    ] {
        check!(r, sk_float_to_half(value) == half);
        check!(r, sk_half_to_float(half).to_bits() == value.to_bits());
    }
    for bits in [0x7f800001, 0x7fc00000, 0xff800001, 0xffffffff] {
        check!(r, sk_float_to_half(f32::from_bits(bits)) == SK_HALF_NaN);
    }
    for (value, expected) in [
        (1. + 1. / 2048., 0x3c00),
        (1. + 3. / 2048., 0x3c02),
        (65504., 0x7bff),
        (65520., 0x7c00),
        (2f32.powi(-25), 0),
        (3. * 2f32.powi(-25), 2),
    ] {
        check!(r, sk_float_to_half(value) == expected);
    }
    type F = SkFloatingPoint<f32, 4>;
    type D = SkFloatingPoint<f64, 4>;
    check!(r, F::new(0.).almost_equals(F::new(-0.)));
    check!(
        r,
        F::new(f32::from_bits(2)).almost_equals(F::new(-f32::from_bits(2)))
    );
    check!(
        r,
        !F::new(f32::from_bits(2)).almost_equals(F::new(-f32::from_bits(3)))
    );
    check!(
        r,
        F::new(1.).almost_equals(F::new(f32::from_bits(1f32.to_bits() + 4)))
    );
    check!(
        r,
        !F::new(1.).almost_equals(F::new(f32::from_bits(1f32.to_bits() + 5)))
    );
    check!(r, !F::new(f32::NAN).almost_equals(F::new(f32::NAN)));
    check!(r, F::new(f32::MAX).almost_equals(F::new(f32::INFINITY)));
    check!(r, D::new(0.).almost_equals(D::new(-0.)));
    check!(
        r,
        D::new(1.).almost_equals(D::new(f64::from_bits(1f64.to_bits() + 4)))
    );
    check!(
        r,
        !D::new(1.).almost_equals(D::new(f64::from_bits(1f64.to_bits() + 5)))
    );
    check!(r, !D::new(f64::NAN).almost_equals(D::new(0.)));
    r.finish();
}
#[derive(Clone, Copy)]
#[repr(u8)]
enum Flag {
    Low = 1,
    High = 128,
}
impl SkBitmaskEnum for Flag {
    type Repr = u8;
    fn to_repr(self) -> u8 {
        self as u8
    }
}
#[test]
fn supplemental_bitmask_and_fixed_capacity() {
    let mut r = Reporter::default();
    let low = SkBitmask::new(Flag::Low);
    let high = SkBitmask::new(Flag::High);
    let mut mask = low | high;
    check!(r, mask.repr() == 129);
    check!(r, mask.contains(Flag::Low));
    check!(r, mask.contains(Flag::High));
    check!(r, sk_bitmask_any(mask));
    check!(r, (low & high).repr() == 0);
    check!(r, (!mask).repr() == 126);
    mask ^= high;
    check!(r, mask.repr() == 1);
    mask |= high;
    mask &= high;
    check!(r, mask.repr() == 128);
    let mut a = FixedArray::<3, i32>::new();
    a.push(1);
    a.push(2);
    a.push(3);
    let p = a.data().as_ptr();
    check!(
        r,
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            a.push(4);
        }))
        .is_err()
    );
    check!(r, a.as_slice() == [1, 2, 3]);
    check!(r, a.data().as_ptr() == p);
    check!(
        r,
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.resize(4))).is_err()
    );
    check!(r, a.as_slice() == [1, 2, 3]);
    check!(
        r,
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            a.push_back_n(usize::MAX);
        }))
        .is_err()
    );
    check!(r, a.as_slice() == [1, 2, 3]);
    let clone = a.clone();
    a.pop_back();
    check!(r, clone.as_slice() == [1, 2, 3]);
    check!(r, a.as_slice() == [1, 2]);
    r.finish();
}
#[test]
fn supplemental_stream_byte_order_boundaries_and_lifetimes() {
    let mut r = Reporter::default();
    let mut w = SkDynamicMemoryWStream::new();
    check!(r, w.write_u16(0x1122));
    check!(r, w.write_u32(0x33445566));
    check!(r, w.write_u64(0x778899aabbccddee));
    check!(r, w.write_scalar(-0.));
    let expected = [
        0x22, 0x11, 0x66, 0x55, 0x44, 0x33, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0, 0,
        0, 0x80,
    ];
    check!(r, w.bytes() == expected);
    let data = w.detach_as_data();
    check!(r, w.bytes_written() == 0);
    w.write(b"replacement");
    drop(w);
    check!(r, data.bytes() == expected);
    let mut s = SkMemoryStream::new(data.clone());
    drop(data);
    check!(r, s.read_u16() == 0x1122);
    check!(r, s.read_u32() == 0x33445566);
    check!(r, s.read_u64() == 0x778899aabbccddee);
    check!(r, s.read_scalar().to_bits() == 0x80000000);
    check!(r, s.is_at_end());
    let mut w = SkDynamicMemoryWStream::new();
    for v in [0xfd, 0xfe, 0xffff, 0x10000, u32::MAX] {
        w.write_packed_u32(v);
    }
    let bytes = w.detach_as_vector();
    check!(
        r,
        bytes
            == [
                0xfd, 0xfe, 0xfe, 0, 0xfe, 0xff, 0xff, 0xff, 0, 0, 1, 0, 0xff, 0xff, 0xff, 0xff,
                0xff
            ]
    );
    w.write(b"new");
    drop(w);
    check!(r, bytes[0] == 0xfd);
    for bytes in [
        &[][..],
        &[0xfe][..],
        &[0xfe, 1][..],
        &[0xff][..],
        &[0xff, 1, 2, 3][..],
    ] {
        let mut s = SkMemoryStream::new(SkData::make_with_copy(bytes));
        check!(r, s.read_packed_u32().is_none());
    }
    let mut s = SkMemoryStream::new(SkData::make_with_copy(b"abc"));
    let mut peek = [0; 4];
    check!(r, s.peek(&mut peek) == 3);
    check!(r, s.position() == 0);
    check!(r, &peek[..3] == b"abc");
    check!(r, s.seek(usize::MAX));
    check!(r, s.position() == 3);
    check!(r, s.read(&mut peek) == 0);
    check!(r, s.rewind());
    check!(r, s.read(&mut []) == 0);
    check!(r, s.position() == 0);
    let mut w = SkDynamicMemoryWStream::new();
    w.write(b"abcd");
    let mut out = [0xa5; 5];
    check!(r, w.copy_to(&mut out));
    check!(r, out == [b'a', b'b', b'c', b'd', 0xa5]);
    let mut out = [0xa5; 1];
    check!(r, !w.read(usize::MAX, &mut out));
    check!(r, out == [0xa5]);
    r.finish();
}
struct Owned {
    live: Rc<Cell<usize>>,
    drops: Rc<Cell<usize>>,
    value: i32,
}
impl Owned {
    fn new(value: i32, live: &Rc<Cell<usize>>, drops: &Rc<Cell<usize>>) -> Self {
        live.set(live.get() + 1);
        Self {
            value,
            live: live.clone(),
            drops: drops.clone(),
        }
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        self.live.set(self.live.get() - 1);
        self.drops.set(self.drops.get() + 1);
    }
}
#[test]
fn supplemental_container_ownership_and_data_lifetime() {
    let mut r = Reporter::default();
    let live = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    {
        let mut a = SkTArray::new();
        for i in 0..100 {
            a.push_back(Owned::new(i, &live, &drops));
        }
        check!(r, live.get() == 100);
        check!(r, drops.get() == 0);
        a.reserve(1000);
        check!(r, live.get() == 100);
        check!(r, drops.get() == 0);
        let mut b = std::mem::take(&mut a);
        check!(r, a.count() == 0);
        check!(r, b.count() == 100);
        check!(r, b[99].value == 99);
        check!(r, drops.get() == 0);
        let removed = b.remove_shuffle(0);
        check!(r, removed.value == 0);
        check!(r, live.get() == 100);
        drop(removed);
        check!(r, live.get() == 99);
        check!(r, drops.get() == 1);
        b.clear();
        check!(r, live.get() == 0);
        check!(r, drops.get() == 100);
    }
    check!(r, live.get() == 0);
    check!(r, drops.get() == 100);
    let releases = Arc::new(AtomicUsize::new(0));
    let c = releases.clone();
    let data = SkData::make_with_proc(vec![1, 2, 3, 4], move || {
        c.fetch_add(1, Ordering::SeqCst);
    });
    let alias = data.clone();
    let subset = data.share_subset(1, 2).unwrap();
    drop(data);
    check!(r, releases.load(Ordering::SeqCst) == 0);
    drop(alias);
    check!(r, releases.load(Ordering::SeqCst) == 0);
    check!(r, subset.bytes() == [2, 3]);
    drop(subset);
    check!(r, releases.load(Ordering::SeqCst) == 1);
    r.finish();
}
// SkFloatToDecimal: exact bytes, NUL termination and untouched bytes after output.
#[test]
fn supplemental_decimal_and_yuv_contracts() {
    let mut r = Reporter::default();
    for (value, expected) in [
        (0f32, "0"),
        (-0., "0"),
        (f32::NAN, "0"),
        (1., "1"),
        (-1., "-1"),
        (0.5, ".5"),
        (-0.5, "-.5"),
        (1.25, "1.25"),
        (1000000., "1000000"),
    ] {
        let mut out = [0xa5; K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH];
        let n = sk_float_to_decimal(value, &mut out);
        check!(r, n == expected.len());
        check!(r, &out[..n] == expected.as_bytes());
        check!(r, out[n] == 0);
        check!(r, out[n + 1..].iter().all(|b| *b == 0xa5));
    }
    check!(
        r,
        sk_float_to_decimal_string(f32::INFINITY) == sk_float_to_decimal_string(f32::MAX)
    );
    check!(
        r,
        sk_float_to_decimal_string(f32::NEG_INFINITY) == sk_float_to_decimal_string(-f32::MAX)
    );
    for value in [
        f32::from_bits(1),
        -f32::from_bits(1),
        f32::MIN_POSITIVE,
        f32::MAX,
        -f32::MAX,
    ] {
        let mut out = [0xa5; K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH];
        let n = sk_float_to_decimal(value, &mut out);
        check!(r, n < K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH);
        check!(r, out[n] == 0);
        check!(r, !out[..n].contains(&b'e'));
        check!(r, !out[..n].contains(&b'E'));
    }
    let identity = [
        1f32, 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0.,
    ];
    let mut m = [f32::NAN; 20];
    sk_color_matrix_rgb_to_yuv(SkYUVColorSpace::Identity, &mut m);
    check!(r, m == identity);
    m.fill(f32::NAN);
    sk_color_matrix_yuv_to_rgb(SkYUVColorSpace::Identity, &mut m);
    check!(r, m == identity);
    // Literal portable #else JPEG coefficients from src/core/SkYUVMath.cpp; signed zeros retained.
    let jpeg = [
        0.299000f32,
        0.587000,
        0.114000,
        0.,
        0.,
        -0.168736,
        -0.331264,
        0.5,
        0.,
        0.501961,
        0.5,
        -0.418688,
        -0.081312,
        0.,
        0.501961,
        0.,
        0.,
        0.,
        1.,
        0.,
    ];
    sk_color_matrix_rgb_to_yuv(SkYUVColorSpace::from_i32(0).unwrap(), &mut m);
    for i in 0..20 {
        check!(r, m[i].to_bits() == jpeg[i].to_bits());
    }
    let inverse = [
        1.000000f32,
        -0.000000,
        1.402000,
        0.,
        -0.703749,
        1.000000,
        -0.344136,
        -0.714136,
        0.,
        0.531211,
        1.000000,
        1.772000,
        0.000000,
        0.,
        -0.889475,
        0.,
        0.,
        0.,
        1.,
        0.,
    ];
    sk_color_matrix_yuv_to_rgb(SkYUVColorSpace::from_i32(0).unwrap(), &mut m);
    for i in 0..20 {
        check!(r, m[i].to_bits() == inverse[i].to_bits());
    }
    check!(r, SkYUVColorSpace::from_i32(-1).is_none());
    check!(r, SkYUVColorSpace::from_i32(29).is_none());
    r.finish();
}
