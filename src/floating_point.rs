#![allow(clippy::approx_constant, clippy::eq_op, clippy::excessive_precision)]

use crate::math::{SK_MAX_S32, SK_MIN_S32};

pub const SK_FLOAT_SQRT2: f32 = 1.41421356;
pub const SK_FLOAT_PI: f32 = 3.14159265;
pub const SK_DOUBLE_PI: f64 = 3.14159265358979323846264338327950288;

pub const SK_FLOAT_NAN: f32 = f32::NAN;
pub const SK_FLOAT_INFINITY: f32 = f32::INFINITY;
pub const SK_FLOAT_NEGATIVE_INFINITY: f32 = f32::NEG_INFINITY;
pub const SK_DOUBLE_NAN: f64 = f64::NAN;

pub const MAX_S32_FITS_IN_FLOAT: i32 = 2147483520;
pub const MIN_S32_FITS_IN_FLOAT: i32 = -MAX_S32_FITS_IN_FLOAT;
pub const MAX_S64_FITS_IN_FLOAT: i64 = i64::MAX >> (63 - 24) << (63 - 24);
pub const MIN_S64_FITS_IN_FLOAT: i64 = -MAX_S64_FITS_IN_FLOAT;

pub const SK_MAX_S32_FITS_IN_FLOAT: i32 = MAX_S32_FITS_IN_FLOAT;
pub const SK_MIN_S32_FITS_IN_FLOAT: i32 = MIN_S32_FITS_IN_FLOAT;
pub const SK_MAX_S64_FITS_IN_FLOAT: i64 = MAX_S64_FITS_IN_FLOAT;
pub const SK_MIN_S64_FITS_IN_FLOAT: i64 = MIN_S64_FITS_IN_FLOAT;

pub fn float_sgn(x: f32) -> i32 {
    (0.0 < x) as i32 - (x < 0.0) as i32
}

pub fn float_degrees_to_radians(degrees: f32) -> f32 {
    degrees * (SK_FLOAT_PI / 180.0)
}

pub fn float_radians_to_degrees(radians: f32) -> f32 {
    radians * (180.0 / SK_FLOAT_PI)
}

pub fn double_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

pub fn is_nan(x: f32) -> bool {
    x != x
}

pub fn is_finite(x: f32) -> bool {
    let prod = x - x;
    prod == prod
}

pub fn is_finite2(x: f32, y: f32) -> bool {
    let prod = (x - x) * y;
    prod == prod
}

pub fn is_finite3(x: f32, y: f32, z: f32) -> bool {
    let prod = (x - x) * y * z;
    prod == prod
}

pub fn is_finite4(a: f32, b: f32, c: f32, d: f32) -> bool {
    let prod = (a - a) * b * c * d;
    prod == prod
}

pub fn is_finite_slice(values: &[f32]) -> bool {
    let Some((&first, rest)) = values.split_first() else {
        return true;
    };
    let mut prod = first - first;
    for &value in rest {
        prod *= value;
    }
    prod == prod
}

pub fn round(x: f32) -> f32 {
    double_round(x as f64) as f32
}

pub fn float_saturate_to_int(x: f32) -> i32 {
    let max = MAX_S32_FITS_IN_FLOAT as f32;
    let min = MIN_S32_FITS_IN_FLOAT as f32;
    let x = if x < max { x } else { max };
    let x = if x > min { x } else { min };
    x as i32
}

pub fn saturate_to_int(x: f32) -> i32 {
    float_saturate_to_int(x)
}

pub fn double_saturate_to_int(x: f64) -> i32 {
    let max = SK_MAX_S32 as f64;
    let min = SK_MIN_S32 as f64;
    let x = if x < max { x } else { max };
    let x = if x > min { x } else { min };
    x as i32
}

pub fn float_saturate_to_int64(x: f32) -> i64 {
    let max = MAX_S64_FITS_IN_FLOAT as f32;
    let min = MIN_S64_FITS_IN_FLOAT as f32;
    let x = if x < max { x } else { max };
    let x = if x > min { x } else { min };
    x as i64
}

pub fn floor_to_int(x: f32) -> i32 {
    float_saturate_to_int(x.floor())
}

pub fn round_to_int(x: f32) -> i32 {
    float_saturate_to_int(round(x))
}

pub fn ceil_to_int(x: f32) -> i32 {
    float_saturate_to_int(x.ceil())
}

pub fn floor_to_int_no_saturate(x: f32) -> i32 {
    x.floor() as i32
}

pub fn round_to_int_no_saturate(x: f32) -> i32 {
    round(x) as i32
}

pub fn ceil_to_int_no_saturate(x: f32) -> i32 {
    x.ceil() as i32
}

pub fn double_floor_to_int(x: f64) -> i32 {
    x.floor() as i32
}

pub fn double_round_to_int(x: f64) -> i32 {
    x.round() as i32
}

pub fn double_ceil_to_int(x: f64) -> i32 {
    x.ceil() as i32
}

pub fn double_to_float(x: f64) -> f32 {
    x as f32
}

pub fn float_midpoint(a: f32, b: f32) -> f32 {
    (0.5 * (a as f64 + b as f64)) as f32
}

pub fn float_rsqrt(x: f32) -> f32 {
    1.0 / x.sqrt()
}

pub fn ieee_float_divide(numer: f32, denom: f32) -> f32 {
    numer / denom
}

pub fn ieee_double_divide(numer: f64, denom: f64) -> f64 {
    numer / denom
}

fn magnitude(a: f64) -> f64 {
    const EXTRACT_MAGNITUDE: u64 = 0x7FF0_0000_0000_0000;
    f64::from_bits(a.to_bits() & EXTRACT_MAGNITUDE)
}

pub fn double_nearly_zero(a: f64) -> bool {
    a == 0.0 || a.abs() < f32::EPSILON as f64
}

pub fn doubles_nearly_equal_ulps(a: f64, b: f64) -> bool {
    doubles_nearly_equal_ulps_with(a, b, 16)
}

pub fn doubles_nearly_equal_ulps_with(a: f64, b: f64, max_ulps_diff: u8) -> bool {
    let min_magnitude = f64::MIN_POSITIVE;
    let max_magnitude = magnitude(a).max(min_magnitude).max(magnitude(b));
    let ulp_factor = f64::EPSILON;
    let tolerance = max_magnitude * (ulp_factor * (f64::from(max_ulps_diff) + 1.0));
    a == b || (b - a).abs() < tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_floating_point_rounding_and_saturation() {
        assert_eq!(round(0.49999997), 0.0);
        assert_eq!(round_to_int(0.49999997), 0);
        assert_eq!(round_to_int(8388609.0), 8388609);
        assert_eq!(float_saturate_to_int(f32::NAN), MAX_S32_FITS_IN_FLOAT);
        assert_eq!(float_saturate_to_int(f32::INFINITY), MAX_S32_FITS_IN_FLOAT);
        assert_eq!(
            float_saturate_to_int(f32::NEG_INFINITY),
            MIN_S32_FITS_IN_FLOAT
        );
        assert_eq!(float_saturate_to_int64(f32::NAN), MAX_S64_FITS_IN_FLOAT);
        assert_eq!(double_saturate_to_int(f64::NAN), SK_MAX_S32);
    }

    #[test]
    fn dev_floating_point_classification_and_ulps() {
        assert!(is_finite(1.0));
        assert!(!is_finite3(1.0, f32::INFINITY, 1.0));
        assert!(is_finite_slice(&[]));
        assert!(doubles_nearly_equal_ulps(1.0, 1.0 + f64::EPSILON));
        assert!(!doubles_nearly_equal_ulps(1.0, 1.1));
        assert!(doubles_nearly_equal_ulps(f64::INFINITY, f64::INFINITY));
        assert!(!doubles_nearly_equal_ulps(f64::NAN, f64::NAN));
        assert!(double_nearly_zero(0.0));
        assert!(double_nearly_zero(1.0e-10));
        assert!(!double_nearly_zero(1.0));
    }
}
