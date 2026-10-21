/*
 * Copyright 2011 Google Inc.
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE.skia file in the tests directory.
 */
#![allow(non_snake_case, clippy::excessive_precision)]

mod support;
use rust_skia::float_bits::float_to_bits;
use rust_skia::floating_point::{is_finite, is_nan};
use rust_skia::point::Point;
use rust_skia::rect::Rect;
use rust_skia::scalar::{SCALAR_INFINITY, SCALAR_NAN, scalar_round_to_int};
use support::{Reporter, reporter_assert};

fn test_roundtoint(reporter: &mut Reporter) {
    let mut x = 0.49999997_f32;
    let mut ix = scalar_round_to_int(x);
    let mut bad_ix = (x + 0.5_f32).floor() as i32;
    // x < 0.5, but addition in f32 rounds up before the naive floor.
    reporter_assert!(reporter, 0 == ix);
    reporter_assert!(reporter, 1 == bad_ix);

    // Between (2^23, 2^24], adding 0.5f can round up before floor().
    x = 8388609.0_f32;
    ix = scalar_round_to_int(x);
    bad_ix = (x + 0.5_f32).floor() as i32;
    reporter_assert!(reporter, 8388609 == ix);
    reporter_assert!(reporter, 8388610 == bad_ix);
}

fn test_isRectFinite(reporter: &mut Reporter) {
    let g_f0 = [Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
    let g_f1 = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(99.234, -42342.0),
    ];
    let g_i0 = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(99.234, -42342.0),
        Point::new(SCALAR_NAN, 3.0),
        Point::new(2.0, 3.0),
    ];
    let g_i1 = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(99.234, -42342.0),
        Point::new(3.0, SCALAR_NAN),
        Point::new(2.0, 3.0),
    ];
    let g_i2 = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(99.234, -42342.0),
        Point::new(SCALAR_INFINITY, 3.0),
        Point::new(2.0, 3.0),
    ];
    let g_i3 = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(99.234, -42342.0),
        Point::new(3.0, SCALAR_INFINITY),
        Point::new(2.0, 3.0),
    ];
    let g_sets: [(&[Point], bool); 6] = [
        (&g_f0, true),
        (&g_f1, true),
        (&g_i0, false),
        (&g_i1, false),
        (&g_i2, false),
        (&g_i3, false),
    ];
    for (points, expected_finite) in g_sets {
        let r = Rect::bounds_or_empty(points);
        let rect_is_finite = !r.is_empty();
        reporter_assert!(reporter, expected_finite == rect_is_finite);
    }
}

fn isFinite_int(x: f32) -> bool {
    let bits = float_to_bits(x);
    let exponent = bits << 1 >> 24;
    exponent != 0xFF
}

fn isFinite_float(x: f32) -> bool {
    is_finite(x)
}
fn isFinite_mulzero(x: f32) -> bool {
    let y = x * 0.0_f32;
    #[allow(clippy::eq_op)]
    {
        y == y
    }
}
type IsFiniteProc1 = fn(f32) -> bool;
fn isFinite2_and(x: f32, y: f32, proc: IsFiniteProc1) -> bool {
    proc(x) && proc(y)
}
fn isFinite2_mulzeroadd(x: f32, y: f32, proc: IsFiniteProc1) -> bool {
    proc(x * 0.0_f32 + y * 0.0_f32)
}
type IsFiniteProc2 = fn(f32, f32, IsFiniteProc1) -> bool;

#[derive(PartialEq)]
enum FloatClass {
    Finite,
    Infinite,
    NaN,
}
fn test_floatclass(reporter: &mut Reporter, value: f32, fc: FloatClass) {
    reporter_assert!(reporter, is_finite(value) == (fc == FloatClass::Finite));
    reporter_assert!(
        reporter,
        value.is_infinite() == (fc == FloatClass::Infinite)
    );
    reporter_assert!(reporter, is_nan(value) == (fc == FloatClass::NaN));
}

fn test_isfinite(reporter: &mut Reporter) {
    // Deliberate overflow and invalid multiplication, as in the C++ source.
    let max = 3.402823466e+38_f32;
    let inf = max * max;
    let nan = inf * 0.0_f32;
    test_floatclass(reporter, 0.0, FloatClass::Finite);
    test_floatclass(reporter, max, FloatClass::Finite);
    test_floatclass(reporter, -max, FloatClass::Finite);
    test_floatclass(reporter, inf, FloatClass::Infinite);
    test_floatclass(reporter, -inf, FloatClass::Infinite);
    test_floatclass(reporter, nan, FloatClass::NaN);
    test_floatclass(reporter, -nan, FloatClass::NaN);
    let data = [
        (0.0_f32, true),
        (1.0, true),
        (-1.0, true),
        (max * 0.75_f32, true),
        (max, true),
        (-max * 0.75_f32, true),
        (-max, true),
        (inf, false),
        (-inf, false),
        (nan, false),
    ];
    let g_proc1: [IsFiniteProc1; 3] = [isFinite_int, isFinite_float, isFinite_mulzero];
    let g_proc2: [IsFiniteProc2; 2] = [isFinite2_and, isFinite2_mulzeroadd];
    for rec in &data {
        for proc in g_proc1 {
            let finite = proc(rec.0);
            reporter_assert!(reporter, rec.1 == finite);
        }
    }
    for rec0 in &data {
        for rec1 in &data {
            for proc1 in g_proc1 {
                for proc2 in g_proc2 {
                    let finite = proc2(rec0.0, rec1.0, proc1);
                    let finite2 = rec0.1 && rec1.1;
                    reporter_assert!(reporter, finite2 == finite);
                }
            }
        }
    }
    test_isRectFinite(reporter);
}

#[test]
fn Scalar() {
    let mut reporter = Reporter::default();
    test_isfinite(&mut reporter);
    test_roundtoint(&mut reporter);
    reporter.finish();
}
