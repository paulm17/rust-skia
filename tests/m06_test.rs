// Translated from Skia b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
// BSD license: tests/LICENSE.skia.
// Registration correspondence (all CPU, no external fixtures):
// BezierCurveTest.cpp:52,107,121,201 -> bezier::{BezierCubicEvalAt,
//   BezierCubicToPolynomials,QuadRoots_CheckTRange,SkBezierCubic_CheckTRange}.
// CubicMapTest.cpp:63 -> cubic_map::CubicMap.
// GeometryTest.cpp:632,978 -> geometry::{Geometry,ConicsWithCrazyW}.
// GeometryTest.cpp:692,818 -> mono_y::{GeometryChopMonoCubicAtY_Successful,
//   GeometryChopMonoCubicAtY_OutOfRangeReturnFalse}.
// GeometryTest.cpp:851,968 -> mono_x::{GeometryChopMonoCubicAtX_Successful,
//   GeometryChopMonoCubicAtX_OutOfRangeReturnFalse}.
// InsetConvexPolyTest.cpp:14 -> insetconvexpoly::InsetConvexPoly.
// OffsetSimplePolyTest.cpp:15 -> offsetsimplepoly::OffsetSimplePoly.
// PolyUtilsTest.cpp:17 -> polyutils::PolyUtils.
// Polygon registrations retain the normal !SK_ENABLE_OPTIMIZE_SIZE configuration.
// Supplemental checks are confined to supplemental.rs. No execution performed.
#![allow(
    non_snake_case,
    unused_mut,
    unused_assignments,
    dead_code,
    clippy::all,
    unused_imports,
    unused_parens,
    non_upper_case_globals
)]
mod support;
use rust_skia::random::SkRandom;
use rust_skia::sk_geometry::*;
use rust_skia::sk_poly_utils::*;
use rust_skia::*;
use support::{Reporter, reporter_assert as check};
const NZ: f32 = 1.0 / 4096.0;
const fn p(x: f32, y: f32) -> Point {
    Point::new(x, y)
}
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= NZ
}
fn point_near(a: Point, b: Point) -> bool {
    near(a.x, b.x) && near(a.y, b.y)
}
fn mono(r: &mut Reporter, name: &str, input: &[Point], t: f32, expected: &[Point], x: bool) {
    eprintln!("{name}");
    if x {
        check!(r, input.len() == 4);
        check!(r, expected.len() == 7);
    }
    check!(r, near(if x { expected[3].x } else { expected[3].y }, t));
    let mut output = [Point::default(); 7];
    check!(
        r,
        if x {
            sk_chop_mono_cubic_at_x(input, t, &mut output)
        } else {
            sk_chop_mono_cubic_at_y(input, t, &mut output)
        }
    );
    for i in 0..7 {
        check!(r, point_near(expected[i], output[i]));
    }
}
#[path = "m06/bezier.rs"]
mod bezier;
#[path = "m06/cubic_map.rs"]
mod cubic_map;
#[path = "m06/geometry.rs"]
mod geometry;
#[path = "m06/insetconvexpoly.rs"]
mod insetconvexpoly;
#[path = "m06/mono_x.rs"]
mod mono_x;
#[path = "m06/mono_y.rs"]
mod mono_y;
#[path = "m06/offsetsimplepoly.rs"]
mod offsetsimplepoly;
#[path = "m06/polyutils.rs"]
mod polyutils;
#[path = "m06/supplemental.rs"]
mod supplemental;
