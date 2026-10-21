/*
 * Copyright 2011 Google Inc.
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE.skia file in the tests directory.
 */
#![allow(non_snake_case)]

mod support;
use rust_skia::floating_point::is_finite;
use rust_skia::point::Point;
use rust_skia::point_priv::PointPriv;
use rust_skia::rect::Rect;
use rust_skia::scalar::{
    SCALAR_1, SCALAR_INFINITY, SCALAR_NAN, Scalar, int_to_scalar, scalar_invert,
    scalar_nearly_equal,
};
use support::{Reporter, reporter_assert};

fn test_casts(reporter: &mut Reporter) {
    let p = Point::new(0.0, 0.0);
    let r = Rect::new(0.0, 0.0, 0.0, 0.0);
    let p_ptr = (&p as *const Point).cast::<Scalar>();
    let r_ptr = (&r as *const Rect).cast::<Scalar>();
    reporter_assert!(reporter, PointPriv::as_scalars(&p) == p_ptr);
    reporter_assert!(reporter, r.as_scalars() == r_ptr);
}

fn test_Normalize(reporter: &mut Reporter, x: Scalar, y: Scalar) {
    let mut point = Point::new(0.0, 0.0);
    point.set(x, y);
    let old_length = point.length();
    let returned = point.normalize_returning_length();
    let new_length = point.length();
    reporter_assert!(reporter, scalar_nearly_equal(returned, old_length));
    reporter_assert!(reporter, scalar_nearly_equal(new_length, SCALAR_1));
}

fn test_normalize_cannormalize_consistent(reporter: &mut Reporter) {
    let values = [
        1.0_f32,
        1e18_f32,
        1e20_f32,
        1e38_f32,
        SCALAR_INFINITY,
        SCALAR_NAN,
    ];
    for val in values {
        let variants = [val, -val, scalar_invert(val), -scalar_invert(val)];
        for v in variants {
            let pts = [
                Point::new(0.0, v),
                Point::new(v, 0.0),
                Point::new(1.0, v),
                Point::new(v, 1.0),
                Point::new(v, v),
            ];
            for mut p in pts {
                let can = PointPriv::can_normalize(p.x, p.y);
                let nor = p.normalize();
                reporter_assert!(reporter, can == nor);
            }
        }
    }
}

fn test_length(reporter: &mut Reporter, x: Scalar, y: Scalar, expected_length: Scalar) {
    let mut point = Point::new(0.0, 0.0);
    point.set(x, y);
    let s1 = point.length();
    let s2 = Point::length_xy(x, y);
    // The original allows approximate equality for historical compiler precision differences.
    reporter_assert!(reporter, scalar_nearly_equal(s1, s2));
    reporter_assert!(reporter, scalar_nearly_equal(s1, expected_length));
    test_Normalize(reporter, x, y);
}

// C++ uses the non-null reporter to prevent compile-time overflow diagnostics.
// A Rust reference is always non-null; retain its runtime opacity with black_box.
fn get_value<T>(reporter: &Reporter, value: T) -> T {
    std::hint::black_box(reporter);
    std::hint::black_box(value)
}

// C++ memcpy forces a float-sized storage round-trip (historically needed on x87).
// Rust uses native bit conversions, independent of the library under test.
fn force_as_float(reporter: &Reporter, value: f32) -> f32 {
    std::hint::black_box(reporter);
    let storage = std::hint::black_box(value.to_bits());
    f32::from_bits(storage)
}

fn test_overflow(reporter: &mut Reporter) {
    let big_float = get_value(reporter, 3.4e38_f32);
    let mut pt = Point::new(big_float, big_float);
    let mut length = pt.length();
    length = force_as_float(reporter, length);
    // Length must overflow, while normalization must still succeed.
    if is_finite(length) {
        eprintln!("length({}, {}) == {}", pt.x, pt.y, length);
        reporter_assert!(reporter, !is_finite(length));
    }
    reporter_assert!(reporter, pt.set_length(SCALAR_1));
    length = pt.length();
    reporter_assert!(reporter, scalar_nearly_equal(length, SCALAR_1));
}

#[test]
fn Point() {
    let mut reporter = Reporter::default();
    test_casts(&mut reporter);
    let g_rec = [
        (int_to_scalar(3), int_to_scalar(4), int_to_scalar(5)),
        (0.6_f32, 0.8_f32, SCALAR_1),
    ];
    for rec in g_rec {
        test_length(&mut reporter, rec.0, rec.1, rec.2);
    }
    test_overflow(&mut reporter);
    test_normalize_cannormalize_consistent(&mut reporter);
    reporter.finish();
}

#[test]
fn Point_setLengthFast() {
    let mut reporter = Reporter::default();
    // Slow and fast scaling of (1,1) must agree within the original strict 0.1% bounds.
    let tests = [
        1.0_f32,
        0.0_f32,
        1.0e-37_f32,
        3.4e38_f32,
        42.0_f32,
        0.00012_f32,
    ];
    let k_one = Point::new(1.0, 1.0);
    for length in tests {
        let mut slow = Point::new(k_one.x, k_one.y);
        let mut fast = Point::new(k_one.x, k_one.y);
        slow.set_length(length);
        PointPriv::set_length_fast(&mut fast, length);
        if slow.length() < f32::MIN_POSITIVE && fast.length() < f32::MIN_POSITIVE {
            continue;
        }
        let ratio = slow.length() / fast.length();
        reporter_assert!(reporter, ratio > 0.999_f32);
        reporter_assert!(reporter, ratio < 1.001_f32);
    }
    reporter.finish();
}
