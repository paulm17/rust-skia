#![allow(clippy::approx_constant, clippy::excessive_precision)]

use crate::floating_point;

pub type Scalar = f32;

pub const SCALAR_1: Scalar = 1.0;
pub const SCALAR_HALF: Scalar = 0.5;
pub const SCALAR_SQRT2: Scalar = floating_point::SK_FLOAT_SQRT2;
pub const SCALAR_PI: Scalar = floating_point::SK_FLOAT_PI;
pub const SCALAR_TAN_PI_OVER_8: Scalar = 0.414213562;
pub const SCALAR_ROOT2_OVER2: Scalar = 0.707106781;
pub const SCALAR_MAX: Scalar = f32::MAX;
pub const SCALAR_MIN: Scalar = -SCALAR_MAX;
pub const SCALAR_INFINITY: Scalar = f32::INFINITY;
pub const SCALAR_NEGATIVE_INFINITY: Scalar = f32::NEG_INFINITY;
pub const SCALAR_NAN: Scalar = f32::NAN;
pub const SCALAR_NEARLY_ZERO: Scalar = 1.0 / 4096.0;
pub const SCALAR_SIN_COS_NEARLY_ZERO: Scalar = 1.0 / 65536.0;

pub fn scalar_floor_to_scalar(x: Scalar) -> Scalar {
    x.floor()
}

pub fn scalar_ceil_to_scalar(x: Scalar) -> Scalar {
    x.ceil()
}

pub fn scalar_round_to_scalar(x: Scalar) -> Scalar {
    floating_point::round(x)
}

pub fn scalar_trunc_to_scalar(x: Scalar) -> Scalar {
    x.trunc()
}

pub fn scalar_floor_to_int(x: Scalar) -> i32 {
    floating_point::floor_to_int(x)
}

pub fn scalar_ceil_to_int(x: Scalar) -> i32 {
    floating_point::ceil_to_int(x)
}

pub fn scalar_round_to_int(x: Scalar) -> i32 {
    floating_point::round_to_int(x)
}

pub fn scalar_trunc_to_int(x: Scalar) -> i32 {
    floating_point::float_saturate_to_int(x)
}

pub fn scalar_abs(x: Scalar) -> Scalar {
    x.abs()
}

pub fn scalar_pin(x: Scalar, lo: Scalar, hi: Scalar) -> Scalar {
    // C++ `SkTPin` is `std::max(lo, std::min(x, hi))`, which returns `lo`
    // when `x` is NaN.
    let min = if hi < x { hi } else { x };
    if lo < min { min } else { lo }
}

pub fn scalar_copy_sign(x: Scalar, y: Scalar) -> Scalar {
    x.copysign(y)
}

pub fn scalar_mod(x: Scalar, y: Scalar) -> Scalar {
    x % y
}

pub fn scalar_sqrt(x: Scalar) -> Scalar {
    x.sqrt()
}

pub fn scalar_pow(base: Scalar, exponent: Scalar) -> Scalar {
    base.powf(exponent)
}

pub fn scalar_sin(radians: Scalar) -> Scalar {
    radians.sin()
}

pub fn scalar_cos(radians: Scalar) -> Scalar {
    radians.cos()
}

pub fn scalar_tan(radians: Scalar) -> Scalar {
    radians.tan()
}

pub fn scalar_asin(value: Scalar) -> Scalar {
    value.asin()
}

pub fn scalar_acos(value: Scalar) -> Scalar {
    value.acos()
}

pub fn scalar_atan2(y: Scalar, x: Scalar) -> Scalar {
    y.atan2(x)
}

pub fn scalar_exp(x: Scalar) -> Scalar {
    x.exp()
}

pub fn scalar_log(x: Scalar) -> Scalar {
    x.ln()
}

pub fn scalar_log2(x: Scalar) -> Scalar {
    x.log2()
}

pub fn int_to_scalar(x: i32) -> Scalar {
    x as Scalar
}

pub fn int_to_float(x: i32) -> f32 {
    x as f32
}

pub fn scalar_to_float(x: Scalar) -> f32 {
    x
}

pub fn float_to_scalar(x: f32) -> Scalar {
    x as Scalar
}

pub fn scalar_to_double(x: Scalar) -> f64 {
    x as f64
}

pub fn double_to_scalar(x: f64) -> Scalar {
    floating_point::double_to_float(x)
}

pub fn scalar_invert(x: Scalar) -> Scalar {
    SCALAR_1 / x
}

pub fn scalar_fraction(x: Scalar) -> Scalar {
    x - scalar_trunc_to_scalar(x)
}

pub fn scalar_square(x: Scalar) -> Scalar {
    x * x
}

pub fn scalar_ave(a: Scalar, b: Scalar) -> Scalar {
    floating_point::float_midpoint(a, b)
}

pub fn degrees_to_radians(degrees: Scalar) -> Scalar {
    degrees * (SCALAR_PI / 180.0)
}

pub fn radians_to_degrees(radians: Scalar) -> Scalar {
    radians * (180.0 / SCALAR_PI)
}

pub fn scalar_is_int(x: Scalar) -> bool {
    x == scalar_floor_to_scalar(x)
}

pub fn scalar_sign_as_int(x: Scalar) -> i32 {
    if x < 0.0 { -1 } else { (x > 0.0) as i32 }
}

pub fn scalar_sign_as_scalar(x: Scalar) -> Scalar {
    if x < 0.0 {
        -SCALAR_1
    } else if x > 0.0 {
        SCALAR_1
    } else {
        0.0
    }
}

pub fn scalar_nearly_zero(x: Scalar) -> bool {
    scalar_nearly_zero_within(x, SCALAR_NEARLY_ZERO)
}

pub fn scalar_nearly_zero_within(x: Scalar, tolerance: Scalar) -> bool {
    debug_assert!(tolerance >= 0.0);
    x.abs() <= tolerance
}

pub fn scalar_nearly_equal(x: Scalar, y: Scalar) -> bool {
    scalar_nearly_equal_within(x, y, SCALAR_NEARLY_ZERO)
}

pub fn scalar_nearly_equal_within(x: Scalar, y: Scalar, tolerance: Scalar) -> bool {
    debug_assert!(tolerance >= 0.0);
    (x - y).abs() <= tolerance
}

pub fn scalar_sin_snap_to_zero(radians: Scalar) -> Scalar {
    let value = scalar_sin(radians);
    if scalar_nearly_zero_within(value, SCALAR_SIN_COS_NEARLY_ZERO) {
        0.0
    } else {
        value
    }
}

pub fn scalar_cos_snap_to_zero(radians: Scalar) -> Scalar {
    let value = scalar_cos(radians);
    if scalar_nearly_zero_within(value, SCALAR_SIN_COS_NEARLY_ZERO) {
        0.0
    } else {
        value
    }
}

pub fn scalar_interp(a: Scalar, b: Scalar, t: Scalar) -> Scalar {
    debug_assert!((0.0..=SCALAR_1).contains(&t));
    a + (b - a) * t
}

pub fn scalars_equal(a: &[Scalar], b: &[Scalar]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(left, right)| left == right)
}

pub fn scalars_equal_n(a: &[Scalar], b: &[Scalar], n: usize) -> bool {
    debug_assert!(n <= a.len());
    debug_assert!(n <= b.len());
    a[..n]
        .iter()
        .zip(&b[..n])
        .all(|(left, right)| left == right)
}

pub fn float_interp_func(search_key: Scalar, keys: &[Scalar], values: &[Scalar]) -> Scalar {
    debug_assert_eq!(keys.len(), values.len());
    debug_assert!(keys.windows(2).all(|pair| pair[0] <= pair[1]));

    let length = keys.len().min(values.len());
    if length == 0 {
        return 0.0;
    }

    let mut right = 0;
    while right < length && keys[right] < search_key {
        right += 1;
    }
    if right == length {
        return values[length - 1];
    }
    if right == 0 {
        return values[0];
    }

    let left_key = keys[right - 1];
    let right_key = keys[right];
    let fract = (search_key - left_key) / (right_key - left_key);
    scalar_interp(values[right - 1], values[right], fract)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_scalar_rounding_and_sign_helpers() {
        assert_eq!(scalar_floor_to_int(1.9), 1);
        assert_eq!(scalar_ceil_to_int(1.1), 2);
        assert_eq!(scalar_trunc_to_int(-1.9), -1);
        assert_eq!(scalar_round_to_scalar(0.49999997), 0.0);
        assert_eq!(scalar_fraction(1.25), 0.25);
        assert_eq!(scalar_sign_as_int(-3.0), -1);
        assert_eq!(scalar_sign_as_int(0.0), 0);
        assert_eq!(scalar_sign_as_int(3.0), 1);
        assert_eq!(scalar_sign_as_scalar(-3.0), -1.0);
        assert!(scalar_is_int(2.0));
        assert!(!scalar_is_int(2.5));
        assert_eq!(scalar_sin_snap_to_zero(0.0), 0.0);
        assert_eq!(scalar_cos_snap_to_zero(std::f32::consts::FRAC_PI_2), 0.0);
    }

    #[test]
    fn dev_float_interp_func_matches_table_walk() {
        let keys = [0.0, 1.0, 2.0];
        let values = [10.0, 20.0, 40.0];
        assert_eq!(float_interp_func(-1.0, &keys, &values), 10.0);
        assert_eq!(float_interp_func(0.0, &keys, &values), 10.0);
        assert_eq!(float_interp_func(0.5, &keys, &values), 15.0);
        assert_eq!(float_interp_func(1.0, &keys, &values), 20.0);
        assert_eq!(float_interp_func(1.5, &keys, &values), 30.0);
        assert_eq!(float_interp_func(3.0, &keys, &values), 40.0);
        assert_eq!(float_interp_func(0.0, &[], &[]), 0.0);
    }

    #[test]
    fn dev_scalars_equal_and_interp() {
        let a = [1.0, 2.0, 3.0];
        let b = [1.0, 2.0, 3.0];
        let c = [1.0, 2.0, 4.0];
        assert!(scalars_equal(&a, &b));
        assert!(!scalars_equal(&a, &c));
        assert!(scalars_equal_n(&a, &c, 2));
        assert_eq!(scalar_interp(1.0, 3.0, 0.5), 2.0);
    }
}
