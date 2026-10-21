// Codex supplemental contract cases; not translations of upstream registrations.
// Matching independent C++ observations: tests/reference/contracts.cpp.
use rust_skia::*;
#[test]
fn supplemental_float_bit_encodings() {
    for bits in [
        0, 0x80000000, 0x3f800000, 0xbf800000, 0x7f800000, 0xff800000, 0x7fc01234, 1, 0x80000001,
    ] {
        let value = bits_to_float(bits);
        assert_eq!(float_to_bits(value), bits);
        let expected = if bits & 0x80000000 != 0 {
            -((bits & 0x7fffffff) as i32)
        } else {
            bits as i32
        };
        assert_eq!(float_as_2s_compliment(value), expected);
        assert_eq!(scalar_as_2s_compliment(value), expected);
        assert_eq!(
            float_to_bits(twos_compliment_as_float(expected)),
            if bits == 0x80000000 { 0 } else { bits }
        );
    }
}
#[test]
fn supplemental_saturation_rounding_and_midpoint() {
    for (value, expected) in [
        (f32::NAN, MAX_S32_FITS_IN_FLOAT),
        (f32::INFINITY, MAX_S32_FITS_IN_FLOAT),
        (f32::NEG_INFINITY, MIN_S32_FITS_IN_FLOAT),
        (-1.75, -1),
    ] {
        assert_eq!(float_saturate_to_int(value), expected);
    }
    assert_eq!(double_round(-0.5), 0.0);
    assert_eq!(double_round_to_int(-0.5), -1);
    assert_eq!(float_midpoint(f32::MAX, f32::MAX), f32::MAX);
    assert_eq!(float_midpoint(f32::MAX, -f32::MAX), 0.0);
}
#[test]
fn supplemental_interpolation_and_equality() {
    for (key, expected) in [
        (-1.0, 10.0),
        (0.0, 10.0),
        (0.5, 15.0),
        (1.0, 20.0),
        (1.5, 30.0),
        (2.0, 40.0),
        (3.0, 40.0),
    ] {
        assert_eq!(
            float_interp_func(key, &[0.0, 1.0, 2.0], &[10.0, 20.0, 40.0]),
            expected
        );
    }
    assert!(scalars_equal(&[0.0], &[-0.0]));
    assert!(!scalars_equal(&[f32::NAN], &[f32::NAN]));
}
#[test]
fn supplemental_ulp_boundaries() {
    assert!(doubles_nearly_equal_ulps_with(
        1.0,
        f64::from_bits(1.0_f64.to_bits() + 16),
        16
    ));
    assert!(!doubles_nearly_equal_ulps_with(
        1.0,
        f64::from_bits(1.0_f64.to_bits() + 17),
        16
    ));
    assert!(doubles_nearly_equal_ulps(f64::INFINITY, f64::INFINITY));
    assert!(!doubles_nearly_equal_ulps(f64::NAN, f64::NAN));
    assert!(double_nearly_zero(f64::from(f32::EPSILON) / 2.0));
    assert!(!double_nearly_zero(f64::from(f32::EPSILON)));
}
#[test]
fn supplemental_point_distances_and_saturation() {
    let p = Point::new(1.0, 3.0);
    let a = Point::new(0.0, 0.0);
    let b = Point::new(2.0, 0.0);
    let mut side = Side::On;
    assert_eq!(
        PointPriv::distance_to_line_between_sqd(&p, &a, &b, Some(&mut side)),
        9.0
    );
    assert_eq!(side, Side::Right);
    assert_eq!(
        PointPriv::distance_to_line_segment_between_sqd(&Point::new(4.0, 3.0), &a, &b),
        13.0
    );
    assert_eq!(
        PointPriv::distance_to_line_between_sqd(&p, &a, &a, None),
        10.0
    );
    assert_eq!(
        (IPoint::new(i32::MAX, i32::MIN) + IPoint::new(1, -1)),
        IPoint::new(i32::MAX, -i32::MAX)
    );
}
