// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// b81c69985fd14bb7bfbd356b7618633ce07dc1a4. HUGE_VALL is infinite on the target, represented by f64::INFINITY.
use super::*;
// tests/FloatingPointTest.cpp:20
#[test]
fn DoubleNearlyZero() {
    let mut r = Reporter::default();
    check!(r, double_nearly_zero(0.));
    check!(r, double_nearly_zero(-0.));
    check!(r, double_nearly_zero(f64::EPSILON));
    check!(r, double_nearly_zero(-f64::EPSILON));

    let nearly: f64 = 1. / 20000000000f64;
    check!(r, nearly != 0.);
    check!(r, double_nearly_zero(nearly));
    check!(r, double_nearly_zero(-nearly));

    check!(r, !double_nearly_zero(f32::EPSILON as f64));
    check!(r, !double_nearly_zero(-(f32::EPSILON as f64)));
    check!(r, !double_nearly_zero(1.));
    check!(r, !double_nearly_zero(-1.));
    check!(r, !double_nearly_zero(f64::INFINITY));
    check!(r, !double_nearly_zero(f64::INFINITY));
    check!(r, !double_nearly_zero(f64::INFINITY));
    check!(r, !double_nearly_zero(f64::INFINITY));
    check!(r, !double_nearly_zero(-f64::INFINITY));
    check!(r, !double_nearly_zero(-f64::INFINITY));
    check!(r, !double_nearly_zero(-f64::INFINITY));
    check!(r, !double_nearly_zero(-f64::INFINITY));
    check!(r, !double_nearly_zero(f64::NAN));
    check!(r, !double_nearly_zero(-f64::NAN));
    r.finish();
}

// tests/FloatingPointTest.cpp:47
#[test]
fn DoubleNearlyEqualUlps() {
    let mut r = Reporter::default();
    // Our tolerance is looser than f64::EPSILON
    check!(r, doubles_nearly_equal_ulps(1., 1.));
    check!(r, doubles_nearly_equal_ulps(1., 1. - f64::EPSILON));
    check!(r, doubles_nearly_equal_ulps(1., 1. + f64::EPSILON));
    check!(r, doubles_nearly_equal_ulps(100.5, 100.5));
    check!(r, doubles_nearly_equal_ulps(100.5, 100.5 - f64::EPSILON));
    check!(r, doubles_nearly_equal_ulps(100.5, 100.5 + f64::EPSILON));

    // Our tolerance is tighter than (f32::EPSILON as f64)
    check!(
        r,
        !doubles_nearly_equal_ulps(1., 1. - (f32::EPSILON as f64))
    );
    check!(
        r,
        !doubles_nearly_equal_ulps(1., 1. + (f32::EPSILON as f64))
    );
    check!(
        r,
        !doubles_nearly_equal_ulps(100.5, 100.5 - (f32::EPSILON as f64))
    );
    check!(
        r,
        !doubles_nearly_equal_ulps(100.5, 100.5 + f32::EPSILON as f64)
    );
    check!(r, !doubles_nearly_equal_ulps(0., 0.1));
    check!(r, !doubles_nearly_equal_ulps(f32::EPSILON as f64, 0.));

    check!(r, doubles_nearly_equal_ulps(f64::INFINITY, f64::INFINITY));
    check!(r, !doubles_nearly_equal_ulps(f64::INFINITY, 10.));
    check!(r, !doubles_nearly_equal_ulps(10., f64::INFINITY));
    check!(r, !doubles_nearly_equal_ulps(f64::NAN, f64::INFINITY));

    check!(r, !doubles_nearly_equal_ulps(f64::INFINITY, -f64::INFINITY));
    check!(r, !doubles_nearly_equal_ulps(-f64::INFINITY, f64::INFINITY));
    check!(r, doubles_nearly_equal_ulps(-f64::INFINITY, -f64::INFINITY));

    // Test values upto the edge of infinity.
    let biggest = f64::MAX;
    // nextafter(max, -infinity) moves down one bit at a time for these positive finite values.
    let almostBiggest = |n: u64| {
        let mut value = biggest;
        for _ in 0..n {
            value = value.next_down();
        }
        value
    };
    let nextBiggest = almostBiggest(1);
    check!(r, doubles_nearly_equal_ulps(biggest, nextBiggest));
    check!(r, doubles_nearly_equal_ulps(biggest, almostBiggest(16)));
    check!(r, !doubles_nearly_equal_ulps(biggest, almostBiggest(17)));

    // One ulp less would be infinity.
    let smallestNANPattern = 0x7ff0000000000001u64;
    let smallestNAN = f64::from_bits(smallestNANPattern);
    debug_assert!(smallestNAN.is_nan());
    debug_assert!(biggest != nextBiggest);

    // Sanity check.
    check!(r, !doubles_nearly_equal_ulps(smallestNAN, f64::NAN));

    // Make sure to return false along the edge of infinity.
    check!(r, !doubles_nearly_equal_ulps(f64::INFINITY, biggest));
    check!(r, !doubles_nearly_equal_ulps(smallestNAN, biggest));
    check!(r, !doubles_nearly_equal_ulps(smallestNAN, f64::INFINITY));

    let smallest = f64::from_bits(1);
    check!(r, !doubles_nearly_equal_ulps(f64::NAN, f64::NAN));
    check!(r, doubles_nearly_equal_ulps(smallest, 0.));
    check!(r, doubles_nearly_equal_ulps(smallest, -smallest));
    check!(r, doubles_nearly_equal_ulps(8. * smallest, -8. * smallest));
    check!(r, !doubles_nearly_equal_ulps(8. * smallest, -9. * smallest));
    r.finish();
}

// tests/FloatingPointTest.cpp:111
#[test]
fn BitCastDoubleRoundTrip() {
    let mut r = Reporter::default();
    let testCases = [0.0f64, 1.0, -13.0, 1.234567890123456, -543210.987654321];

    for i in 0..testCases.len() {
        let input = testCases[i];
        let bits: u64 = sk_bit_cast(&input);
        let output: f64 = sk_bit_cast(&bits);
        check!(r, input == output);
    }

    {
        let bits: u64 = sk_bit_cast(&f64::NAN);
        let output: f64 = sk_bit_cast(&bits);
        check!(r, output.is_nan());
    }
    {
        let bits: u64 = sk_bit_cast(&f64::INFINITY);
        let output: f64 = sk_bit_cast(&bits);
        check!(r, !output.is_finite());
    }
    r.finish();
}

// tests/FloatingPointTest.cpp:133
#[test]
fn FMA() {
    let mut r = Reporter::default();
    // 0b0'01111111111'00'0000000000'0000000000'0000000010'0000000000'0000000000
    let over1 = 1f64 + 4.656612873e-10;

    // 0b0'01111111110'11'1111111111'1111111111'1111111100'0000000000'0000000000
    let under1 = 1f64 - 4.656612873e-10;

    // Precision loss
    //                         -------------- becomes 1.; extra bits are rounded off.
    let x = 1f64.mul_add(-1., over1 * under1);

    // Precision maintained
    //                  ------------- becomes 1 - 2^-62; extra bits are maintained
    let y = over1.mul_add(under1, -1.);

    check!(r, x == 0.);
    check!(r, y == -2f64.powi(-62));
    r.finish();
}

// tests/FloatingPointTest.cpp:152
#[test]
fn Midpoint() {
    let mut r = Reporter::default();
    let smallest = f32::from_bits(1);
    check!(r, float_midpoint(smallest, smallest) == smallest);
    check!(r, float_midpoint(smallest, -smallest) == 0.);

    let biggest = f32::MAX;
    check!(r, float_midpoint(biggest, biggest) == biggest);
    check!(r, float_midpoint(biggest, -biggest) == 0.);

    r.finish();
}
