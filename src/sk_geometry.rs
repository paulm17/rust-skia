//! Port of `src/core/SkGeometry.h` and `src/core/SkGeometry.cpp`.
//!
//! Quadratic/cubic/conic foundations for the rest of Skia: unit-quadratic roots, quad and
//! cubic evaluation/chopping, extrema, inflection and curvature splits, cubic classification,
//! conic evaluation/chopping/quad approximation and unit-arc construction.
//!
//! The reference implementation uses `skvx::float2`/`float4` for the vector math. Those are
//! plain per-component `f32` operations, so the port uses `Point` plus scalar arithmetic and
//! preserves the reference expression order. The two-lane `SkChopCubicAt(src, dst, t0, t1)`
//! SIMD body is evaluated lane by lane with the same `(b - a) * t + a` mixing.

use crate::floating_point::{double_to_float, ieee_float_divide, is_finite};
use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::point_priv::PointPriv;
use crate::point3::Point3;
use crate::rect::Rect;
use crate::scalar::{
    SCALAR_NEARLY_ZERO, SCALAR_PI, SCALAR_ROOT2_OVER2, Scalar, scalar_abs, scalar_acos, scalar_cos,
    scalar_invert, scalar_nearly_zero, scalar_pin, scalar_pow, scalar_sqrt,
};
use crate::sk_bezier_curves::SkBezierCubic;
use crate::sk_cubics::SkCubics;

/// `SkPathDirection` declaration from `include/core/SkPathTypes.h`, declared here because
/// `SkConic::BuildUnitArc` consumes it. Path ownership (M07) adopts this enum unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathDirection {
    Cw,
    Ccw,
}

/// `std::max` semantics (returns `a` when the comparison is unordered, e.g. for NaN).
#[inline]
fn cpp_max(a: Scalar, b: Scalar) -> Scalar {
    if a < b { b } else { a }
}

/// `std::min` semantics (returns `a` when the comparison is unordered, e.g. for NaN).
#[inline]
fn cpp_min(a: Scalar, b: Scalar) -> Scalar {
    if b < a { b } else { a }
}

#[inline]
fn cpp_max_f64(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

#[inline]
fn times_2(value: Point) -> Point {
    value + value
}

/// `SkIsNaN` counterpart for the scalar paths.
#[inline]
fn is_nan(x: Scalar) -> bool {
    x.is_nan()
}

// ---------------------------------------------------------------------------------------------
// Private helpers from the anonymous namespace
// ---------------------------------------------------------------------------------------------

fn is_not_monotonic(a: Scalar, b: Scalar, c: Scalar) -> bool {
    let ab = a - b;
    let mut bc = b - c;
    if ab < 0.0 {
        bc = -bc;
    }
    ab == 0.0 || bc < 0.0
}

fn valid_unit_divide(mut numer: Scalar, mut denom: Scalar, ratio: &mut Scalar) -> i32 {
    if numer < 0.0 {
        numer = -numer;
        denom = -denom;
    }

    if denom == 0.0 || numer == 0.0 || numer >= denom {
        return 0;
    }

    let r = numer / denom;
    if is_nan(r) {
        return 0;
    }
    debug_assert!((0.0..1.0).contains(&r));
    if r == 0.0 {
        return 0;
    }
    *ratio = r;
    1
}

fn return_check_zero(value: i32) -> i32 {
    if value == 0 { 0 } else { value }
}

// ---------------------------------------------------------------------------------------------
// SkFindUnitQuadRoots
// ---------------------------------------------------------------------------------------------

/// Given `A*x^2 + B*x + C = 0`, returns 0, 1 or 2 roots in `roots`.
///
/// From Numerical Recipes in C:
/// `Q = -1/2 (B + sign(B) sqrt[B*B - 4*A*C])`, `x1 = Q / A`, `x2 = C / Q`.
pub fn sk_find_unit_quad_roots(a: Scalar, b: Scalar, c: Scalar, roots: &mut [Scalar; 2]) -> i32 {
    if a == 0.0 {
        return return_check_zero(valid_unit_divide(-c, b, &mut roots[0]));
    }

    let mut index = 0usize;

    // use doubles so we don't overflow temporarily trying to compute R
    let mut dr = (b as f64) * (b as f64) - 4.0 * (a as f64) * (c as f64);
    if dr < 0.0 {
        return return_check_zero(0);
    }
    dr = dr.sqrt();
    let r = double_to_float(dr);
    if !is_finite(r) {
        return return_check_zero(0);
    }

    let q = if b < 0.0 {
        -(b - r) / 2.0
    } else {
        -(b + r) / 2.0
    };
    index += valid_unit_divide(q, a, &mut roots[index]) as usize;
    index += valid_unit_divide(c, q, &mut roots[index]) as usize;
    if index == 2 {
        if roots[0] > roots[1] {
            roots.swap(0, 1);
        } else if roots[0] == roots[1] {
            // nearly-equal?
            index -= 1; // skip the double root
        }
    }
    return_check_zero(index as i32)
}

// ---------------------------------------------------------------------------------------------
// Quadratics
// ---------------------------------------------------------------------------------------------

pub fn sk_eval_quad_at(src: &[Point], t: Scalar) -> Point {
    let coeff = QuadCoeff::new(src);
    coeff.eval(t)
}

pub fn sk_eval_quad_at_with(
    src: &[Point],
    t: Scalar,
    pt: Option<&mut Point>,
    tangent: Option<&mut Vector>,
) {
    debug_assert!(src.len() >= 3);
    debug_assert!((0.0..=1.0).contains(&t));

    if let Some(pt) = pt {
        *pt = sk_eval_quad_at(src, t);
    }
    if let Some(tangent) = tangent {
        *tangent = sk_eval_quad_tangent_at(src, t);
    }
}

pub fn sk_eval_quad_tangent_at(src: &[Point], t: Scalar) -> Vector {
    // The derivative equation is 2(b - a +(a - 2b +c)t). This returns a
    // zero tangent vector when t is 0 or 1, and the control point is equal
    // to the end point. In this case, use the quad end points to compute the tangent.
    if (t == 0.0 && src[0] == src[1]) || (t == 1.0 && src[1] == src[2]) {
        return src[2] - src[0];
    }
    debug_assert!((0.0..=1.0).contains(&t));

    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];

    let b = p1 - p0;
    let a = p2 - p1 - b;
    let tangent = a * t + b;

    tangent + tangent
}

fn interp(v0: Point, v1: Point, t: Scalar) -> Point {
    v0 + (v1 - v0) * t
}

pub fn sk_chop_quad_at(src: &[Point], dst: &mut [Point], t: Scalar) {
    debug_assert!(t > 0.0 && t < 1.0);
    debug_assert!(dst.len() >= 5);

    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];

    let p01 = interp(p0, p1, t);
    let p12 = interp(p1, p2, t);

    dst[0] = p0;
    dst[1] = p01;
    dst[2] = interp(p01, p12, t);
    dst[3] = p12;
    dst[4] = p2;
}

/// In-place form of `SkChopQuadAt(src, dst, t)` where `dst` aliases `src`.
pub fn sk_chop_quad_at_in_place(pts: &mut [Point], t: Scalar) {
    let src = [pts[0], pts[1], pts[2]];
    sk_chop_quad_at(&src, pts, t);
}

pub fn sk_chop_quad_at_half(src: &[Point], dst: &mut [Point]) {
    sk_chop_quad_at(src, dst, 0.5);
}

pub fn sk_measure_angle_between_vectors(a: Vector, b: Vector) -> Scalar {
    let mut cos_theta = ieee_float_divide(a.dot(&b), (a.dot(&a) * b.dot(&b)).sqrt());
    // Pin cosTheta such that if it is NaN (e.g., if a or b was 0), then we return acos(1) = 0.
    cos_theta = cpp_max(cpp_min(1.0, cos_theta), -1.0);
    scalar_acos(cos_theta)
}

pub fn sk_find_bisector(a: Vector, b: Vector) -> Vector {
    let v: [Vector; 2];
    if a.dot(&b) >= 0.0 {
        // a,b are within +/-90 degrees apart.
        v = [a, b];
    } else if a.cross(&b) >= 0.0 {
        // a,b are >90 degrees apart. Find the bisector of their interior normals instead.
        v = [Vector::new(-a.y, a.x), Vector::new(b.y, -b.x)];
    } else {
        // a,b are <-90 degrees apart. Find the bisector of their interior normals instead.
        v = [Vector::new(a.y, -a.x), Vector::new(-b.y, b.x)];
    }
    // Return "normalize(v[0]) + normalize(v[1])".
    let x0_x1 = [v[0].x, v[1].x];
    let y0_y1 = [v[0].y, v[1].y];
    let inv_lengths = 1.0 / (x0_x1[0] * x0_x1[0] + y0_y1[0] * y0_y1[0]).sqrt();
    let inv_lengths_1 = 1.0 / (x0_x1[1] * x0_x1[1] + y0_y1[1] * y0_y1[1]).sqrt();
    let x0 = x0_x1[0] * inv_lengths;
    let x1 = x0_x1[1] * inv_lengths_1;
    let y0 = y0_y1[0] * inv_lengths;
    let y1 = y0_y1[1] * inv_lengths_1;
    Vector::new(x0 + x1, y0 + y1)
}

pub fn sk_find_quad_mid_tangent(src: &[Point]) -> Scalar {
    // Tangents point in the direction of increasing T, so tan0 and -tan1 both point toward the
    // midtangent. The bisector of tan0 and -tan1 is orthogonal to the midtangent.
    let tan0 = src[1] - src[0];
    let tan1 = src[2] - src[1];
    let bisector = sk_find_bisector(tan0, -tan1);

    let mut t = ieee_float_divide(tan0.dot(&bisector), (tan0 - tan1).dot(&bisector));
    if !(t > 0.0 && t < 1.0) {
        // Use "!(positive_logic)" so T=nan will take this branch.
        t = 0.5; // The quadratic was a line or near-line. Just chop at .5.
    }
    t
}

pub fn sk_chop_quad_at_mid_tangent(src: &[Point], dst: &mut [Point]) {
    sk_chop_quad_at(src, dst, sk_find_quad_mid_tangent(src));
}

pub fn sk_measure_quad_rotation(pts: &[Point]) -> Scalar {
    sk_measure_angle_between_vectors(pts[1] - pts[0], pts[2] - pts[1])
}

pub fn sk_find_quad_extrema(a: Scalar, b: Scalar, c: Scalar, t_value: &mut [Scalar; 1]) -> i32 {
    // Quad'(t) = At + B, where A = 2(a - 2b + c), B = 2(b - a).
    // Solve for t, only if it fits between 0 < t < 1.
    valid_unit_divide(a - b, a - b - b + c, &mut t_value[0])
}

pub fn sk_chop_quad_at_y_extrema(src: &[Point], dst: &mut [Point]) -> i32 {
    let a = src[0].y;
    let mut b = src[1].y;
    let c = src[2].y;

    if is_not_monotonic(a, b, c) {
        let mut t_value = 0.0;
        if valid_unit_divide(a - b, a - b - b + c, &mut t_value) != 0 {
            sk_chop_quad_at(src, dst, t_value);
            // flatten_double_quad_extrema(&dst[0].fY)
            dst[1].y = dst[2].y;
            dst[3].y = dst[2].y;
            return 1;
        }
        // if we get here, we need to force dst to be monotonic, even though
        // we couldn't compute a unit_divide value (probably underflow).
        b = if scalar_abs(a - b) < scalar_abs(b - c) {
            a
        } else {
            c
        };
    }
    dst[0] = Point::new(src[0].x, a);
    dst[1] = Point::new(src[1].x, b);
    dst[2] = Point::new(src[2].x, c);
    0
}

pub fn sk_chop_quad_at_x_extrema(src: &[Point], dst: &mut [Point]) -> i32 {
    let a = src[0].x;
    let mut b = src[1].x;
    let c = src[2].x;

    if is_not_monotonic(a, b, c) {
        let mut t_value = 0.0;
        if valid_unit_divide(a - b, a - b - b + c, &mut t_value) != 0 {
            sk_chop_quad_at(src, dst, t_value);
            // flatten_double_quad_extrema(&dst[0].fX)
            dst[1].x = dst[2].x;
            dst[3].x = dst[2].x;
            return 1;
        }
        b = if scalar_abs(a - b) < scalar_abs(b - c) {
            a
        } else {
            c
        };
    }
    dst[0] = Point::new(a, src[0].y);
    dst[1] = Point::new(b, src[1].y);
    dst[2] = Point::new(c, src[2].y);
    0
}

pub fn sk_find_quad_max_curvature(src: &[Point]) -> Scalar {
    let ax = src[1].x - src[0].x;
    let ay = src[1].y - src[0].y;
    let bx = src[0].x - src[1].x - src[1].x + src[2].x;
    let by = src[0].y - src[1].y - src[1].y + src[2].y;

    let mut numer = -(ax * bx + ay * by);
    let mut denom = bx * bx + by * by;
    if denom < 0.0 {
        numer = -numer;
        denom = -denom;
    }
    if numer <= 0.0 {
        return 0.0;
    }
    if numer >= denom {
        // Also catches denom=0.
        return 1.0;
    }
    let t = numer / denom;
    debug_assert!((0.0..1.0).contains(&t) || is_nan(t));
    t
}

pub fn sk_chop_quad_at_max_curvature(src: &[Point], dst: &mut [Point]) -> i32 {
    let t = sk_find_quad_max_curvature(src);
    if t > 0.0 && t < 1.0 {
        sk_chop_quad_at(src, dst, t);
        2
    } else {
        dst[..3].copy_from_slice(&src[..3]);
        1
    }
}

/// In-place form of `SkChopQuadAtMaxCurvature(src, dst)` where `dst` aliases `src`.
pub fn sk_chop_quad_at_max_curvature_in_place(pts: &mut [Point]) -> i32 {
    let src = [pts[0], pts[1], pts[2]];
    sk_chop_quad_at_max_curvature(&src, pts)
}

pub fn sk_convert_quad_to_cubic(src: &[Point], dst: &mut [Point]) {
    let scale = double_to_float(2.0 / 3.0);
    let s0 = src[0];
    let s1 = src[1];
    let s2 = src[2];

    dst[0] = s0;
    dst[1] = s0 + (s1 - s0) * scale;
    dst[2] = s2 + (s1 - s2) * scale;
    dst[3] = s2;
}

// ---------------------------------------------------------------------------------------------
// Cubics
// ---------------------------------------------------------------------------------------------

fn eval_cubic_derivative(src: &[Point], t: Scalar) -> Vector {
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];

    let a = p3 + (p1 - p2) * 3.0 - p0;
    let b = times_2(p2 - times_2(p1) + p0);
    let c = p1 - p0;
    (a * t + b) * t + c
}

fn eval_cubic_2nd_derivative(src: &[Point], t: Scalar) -> Vector {
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];

    let a = p3 + (p1 - p2) * 3.0 - p0;
    let b = p2 - times_2(p1) + p0;

    a * t + b
}

pub fn sk_eval_cubic_at(
    src: &[Point],
    t: Scalar,
    loc: Option<&mut Point>,
    tangent: Option<&mut Vector>,
    curvature: Option<&mut Vector>,
) {
    debug_assert!((0.0..=1.0).contains(&t));

    let cubic = SkCubicCoeff::new(src);

    if let Some(loc) = loc {
        *loc = cubic.eval(t);
    }
    if let Some(tangent) = tangent {
        // The derivative equation returns a zero tangent vector when t is 0 or 1, and the
        // adjacent control point is equal to the end point. In this case, use the
        // next control point or the end points to compute the tangent.
        if (t == 0.0 && src[0] == src[1]) || (t == 1.0 && src[2] == src[3]) {
            *tangent = if t == 0.0 {
                src[2] - src[0]
            } else {
                src[3] - src[1]
            };
            if tangent.x == 0.0 && tangent.y == 0.0 {
                *tangent = src[3] - src[0];
            }
        } else {
            *tangent = eval_cubic_derivative(src, t);
        }
    }
    if let Some(curvature) = curvature {
        *curvature = eval_cubic_2nd_derivative(src, t);
    }
}

pub fn sk_find_cubic_extrema(
    a: Scalar,
    b: Scalar,
    c: Scalar,
    d: Scalar,
    t_values: &mut [Scalar; 2],
) -> i32 {
    // Cubic'(t) = At^2 + Bt + C, where
    // A = 3(-a + 3(b - c) + d), B = 6(a - 2b + c), C = 3(b - a)
    // Solve for t, keeping only those that fit between 0 < t < 1.
    // We divide A,B,C by 3 to simplify.
    let coeff_a = d - a + 3.0 * (b - c);
    let coeff_b = 2.0 * (a - b - b + c);
    let coeff_c = b - a;

    sk_find_unit_quad_roots(coeff_a, coeff_b, coeff_c, t_values)
}

/// This does not return `b` when `t == 1`, but it otherwise gets better precision than
/// `a*(1 - t) + b*t` for things like chopping cubics on exact cusp points.
fn unchecked_mix(a: Point, b: Point, t: Scalar) -> Point {
    (b - a) * t + a
}

pub fn sk_chop_cubic_at(src: &[Point], dst: &mut [Point], t: Scalar) {
    debug_assert!((0.0..=1.0).contains(&t));
    debug_assert!(dst.len() >= 7);

    if t == 1.0 {
        dst[..4].copy_from_slice(&src[..4]);
        dst[4] = src[3];
        dst[5] = src[3];
        dst[6] = src[3];
        return;
    }

    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];

    let ab = unchecked_mix(p0, p1, t);
    let bc = unchecked_mix(p1, p2, t);
    let cd = unchecked_mix(p2, p3, t);
    let abc = unchecked_mix(ab, bc, t);
    let bcd = unchecked_mix(bc, cd, t);
    let abcd = unchecked_mix(abc, bcd, t);

    dst[0] = p0;
    dst[1] = ab;
    dst[2] = abc;
    dst[3] = abcd;
    dst[4] = bcd;
    dst[5] = cd;
    dst[6] = p3;
}

/// In-place form of `SkChopCubicAt(src, dst, t)` where `dst` aliases `src`.
pub fn sk_chop_cubic_at_in_place(pts: &mut [Point], t: Scalar) {
    let src = [pts[0], pts[1], pts[2], pts[3]];
    sk_chop_cubic_at(&src, pts, t);
}

/// Two chops performed in parallel in the reference using 4-lane SIMD. Each lane is an
/// independent `unchecked_mix` chain, and `dst[4..6]` mixes with the shuffled parameters.
pub fn sk_chop_cubic_at_two(src: &[Point], dst: &mut [Point], t0: Scalar, t1: Scalar) {
    debug_assert!(0.0 <= t0 && t0 <= t1 && t1 <= 1.0);
    debug_assert!(dst.len() >= 10);

    if t1 == 1.0 {
        sk_chop_cubic_at(src, dst, t0);
        dst[7] = src[3];
        dst[8] = src[3];
        dst[9] = src[3];
        return;
    }

    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];

    // lane 0 uses t0
    let ab0 = unchecked_mix(p0, p1, t0);
    let bc0 = unchecked_mix(p1, p2, t0);
    let cd0 = unchecked_mix(p2, p3, t0);
    let abc0 = unchecked_mix(ab0, bc0, t0);
    let bcd0 = unchecked_mix(bc0, cd0, t0);
    let abcd0 = unchecked_mix(abc0, bcd0, t0);

    // lane 1 uses t1
    let ab1 = unchecked_mix(p0, p1, t1);
    let bc1 = unchecked_mix(p1, p2, t1);
    let cd1 = unchecked_mix(p2, p3, t1);
    let abc1 = unchecked_mix(ab1, bc1, t1);
    let bcd1 = unchecked_mix(bc1, cd1, t1);
    let abcd1 = unchecked_mix(abc1, bcd1, t1);

    // middle = unchecked_mix(abc, bcd, shuffle<2,3,0,1>(T)): lo lane mixes with t1, hi with t0.
    let middle_lo = unchecked_mix(abc0, bcd0, t1);
    let middle_hi = unchecked_mix(abc1, bcd1, t0);

    dst[0] = p0;
    dst[1] = ab0;
    dst[2] = abc0;
    dst[3] = abcd0;
    dst[4] = middle_lo;
    dst[5] = middle_hi;
    dst[6] = abcd1;
    dst[7] = bcd1;
    dst[8] = cd1;
    dst[9] = p3;
}

/// `SkChopCubicAt(src, dst, tValues, tCount)`: chops at `t_count` sorted parameter values and
/// writes `t_count * 3 + 4` points into `dst`.
pub fn sk_chop_cubic_at_ts(src: &[Point], dst: &mut [Point], t_values: &[Scalar]) {
    let t_count = t_values.len();
    if t_count == 0 {
        // nothing to chop
        dst[..4].copy_from_slice(&src[..4]);
        return;
    }

    let mut current = [src[0], src[1], src[2], src[3]];
    let mut dst_offset = 0usize;
    let mut i = 0usize;
    while i + 1 < t_count {
        // Do two chops at once.
        let mut tt0 = t_values[i];
        let mut tt1 = t_values[i + 1];
        if i != 0 {
            let last_t = t_values[i - 1];
            let denom = 1.0 - last_t;
            tt0 = scalar_pin((tt0 - last_t) / denom, 0.0, 1.0);
            tt1 = scalar_pin((tt1 - last_t) / denom, 0.0, 1.0);
        }
        sk_chop_cubic_at_two(&current, &mut dst[dst_offset..], tt0, tt1);
        current = [
            dst[dst_offset + 6],
            dst[dst_offset + 7],
            dst[dst_offset + 8],
            dst[dst_offset + 9],
        ];
        dst_offset += 6;
        i += 2;
    }
    if i < t_count {
        // Chop the final cubic if there was an odd number of chops.
        debug_assert!(i + 1 == t_count);
        let mut t = t_values[i];
        if i != 0 {
            let last_t = t_values[i - 1];
            t = scalar_pin(ieee_float_divide(t - last_t, 1.0 - last_t), 0.0, 1.0);
        }
        sk_chop_cubic_at(&current, &mut dst[dst_offset..], t);
    }
}

pub fn sk_chop_cubic_at_half(src: &[Point], dst: &mut [Point]) {
    sk_chop_cubic_at(src, dst, 0.5);
}

pub fn sk_measure_non_inflect_cubic_rotation(pts: &[Point]) -> Scalar {
    let a = pts[1] - pts[0];
    let b = pts[2] - pts[1];
    let c = pts[3] - pts[2];
    if a.is_zero() {
        return sk_measure_angle_between_vectors(b, c);
    }
    if b.is_zero() {
        return sk_measure_angle_between_vectors(a, c);
    }
    if c.is_zero() {
        return sk_measure_angle_between_vectors(a, b);
    }
    // Postulate: When no points are colocated and there are no inflection points in T=0..1, the
    // rotation is: 360 degrees, minus the angle [p0,p1,p2], minus the angle [p1,p2,p3].
    2.0 * SCALAR_PI
        - sk_measure_angle_between_vectors(a, -b)
        - sk_measure_angle_between_vectors(b, -c)
}

/// Finds the root nearest 0.5. Returns 0.5 if the roots are undefined or outside 0..1.
fn solve_quadratic_equation_for_midtangent_4(
    a: Scalar,
    b: Scalar,
    c: Scalar,
    discr: Scalar,
) -> Scalar {
    // Quadratic formula from Numerical Recipes in C:
    let q = -0.5 * (b + scalar_sqrt(discr).copysign(b));
    // The roots are q/a and c/q. Pick the midtangent closer to T=.5.
    let _5qa = -0.5 * q * a;
    let mut t = if scalar_abs(q * q + _5qa) < scalar_abs(a * c + _5qa) {
        ieee_float_divide(q, a)
    } else {
        ieee_float_divide(c, q)
    };
    if !(t > 0.0 && t < 1.0) {
        // Use "!(positive_logic)" so T=NaN will take this branch.
        // Either the curve is a flat line with no rotation or FP precision failed us. Chop at .5.
        t = 0.5;
    }
    t
}

fn solve_quadratic_equation_for_midtangent(a: Scalar, b: Scalar, c: Scalar) -> Scalar {
    solve_quadratic_equation_for_midtangent_4(a, b, c, b * b - 4.0 * a * c)
}

/// `skvx::fma` over four lanes.
#[inline]
fn fma4(f: [Scalar; 4], m: Scalar, a: [Scalar; 4]) -> [Scalar; 4] {
    [
        f[0].mul_add(m, a[0]),
        f[1].mul_add(m, a[1]),
        f[2].mul_add(m, a[2]),
        f[3].mul_add(m, a[3]),
    ]
}

pub fn sk_find_cubic_mid_tangent(src: &[Point]) -> Scalar {
    // Tangents point in the direction of increasing T, so tan0 and -tan1 both point toward the
    // midtangent. The bisector of tan0 and -tan1 is orthogonal to the midtangent.
    let tan0 = if src[0] == src[1] {
        src[2] - src[0]
    } else {
        src[1] - src[0]
    };
    let tan1 = if src[2] == src[3] {
        src[3] - src[1]
    } else {
        src[3] - src[2]
    };
    let bisector = sk_find_bisector(tan0, -tan1);

    // Find the T value at the midtangent. This is a simple quadratic equation.
    const K_M: [[Scalar; 4]; 3] = [
        [-1.0, 2.0, -1.0, 0.0],
        [3.0, -4.0, 1.0, 0.0],
        [-3.0, 2.0, 0.0, 0.0],
    ];
    let c_x = fma4(
        K_M[0],
        src[0].x,
        fma4(
            K_M[1],
            src[1].x,
            fma4(K_M[2], src[2].x, [src[3].x, 0.0, 0.0, 0.0]),
        ),
    );
    let c_y = fma4(
        K_M[0],
        src[0].y,
        fma4(
            K_M[1],
            src[1].y,
            fma4(K_M[2], src[2].y, [src[3].y, 0.0, 0.0, 0.0]),
        ),
    );
    let mut coeffs = [
        c_x[0] * bisector.x + c_y[0] * bisector.y,
        c_x[1] * bisector.x + c_y[1] * bisector.y,
        c_x[2] * bisector.x + c_y[2] * bisector.y,
        c_x[3] * bisector.x + c_y[3] * bisector.y,
    ];

    // Now solve the quadratic for T.
    let mut t = 0.0;
    let a = coeffs[0];
    let b = coeffs[1];
    let c = coeffs[2];
    let discr = b * b - 4.0 * a * c;
    if discr > 0.0 {
        // This will only be false if the curve is a line.
        return solve_quadratic_equation_for_midtangent_4(a, b, c, discr);
    }

    // This is a 0- or 360-degree flat line. It doesn't have single points of midtangent.
    // Chop in between both cusps instead, if any.
    coeffs = [
        c_x[0] * tan0.x + c_y[0] * tan0.y,
        c_x[1] * tan0.x + c_y[1] * tan0.y,
        c_x[2] * tan0.x + c_y[2] * tan0.y,
        c_x[3] * tan0.x + c_y[3] * tan0.y,
    ];
    let a = coeffs[0];
    let b = coeffs[1];
    if a != 0.0 {
        // The midpoint of the two cusps is -b / (2*a).
        t = -b / (2.0 * a);
    }
    if !(t > 0.0 && t < 1.0) {
        // Use "!(positive_logic)" so T=NaN will take this branch.
        t = 0.5;
    }
    t
}

pub fn sk_chop_cubic_at_mid_tangent(src: &[Point], dst: &mut [Point]) {
    sk_chop_cubic_at(src, dst, sk_find_cubic_mid_tangent(src));
}

fn flatten_double_cubic_extrema(pts: &mut [Point]) {
    // coords[4] = coords[8] = coords[6] on the interleaved fY (or fX) array.
    pts[2].y = pts[3].y;
    pts[4].y = pts[3].y;
}

pub fn sk_chop_cubic_at_y_extrema(src: &[Point], dst: &mut [Point]) -> i32 {
    let mut t_values = [0.0; 2];
    let roots = sk_find_cubic_extrema(src[0].y, src[1].y, src[2].y, src[3].y, &mut t_values);

    sk_chop_cubic_at_ts(src, dst, &t_values[..roots as usize]);
    if roots > 0 {
        // we do some cleanup to ensure our Y extrema are flat
        flatten_double_cubic_extrema(dst);
        if roots == 2 {
            flatten_double_cubic_extrema(&mut dst[3..]);
        }
    }
    roots
}

pub fn sk_chop_cubic_at_x_extrema(src: &[Point], dst: &mut [Point]) -> i32 {
    let mut t_values = [0.0; 2];
    let roots = sk_find_cubic_extrema(src[0].x, src[1].x, src[2].x, src[3].x, &mut t_values);

    sk_chop_cubic_at_ts(src, dst, &t_values[..roots as usize]);
    if roots > 0 {
        // we do some cleanup to ensure our X extrema are flat
        flatten_double_cubic_extrema_x(dst);
        if roots == 2 {
            flatten_double_cubic_extrema_x(&mut dst[3..]);
        }
    }
    roots
}

fn flatten_double_cubic_extrema_x(pts: &mut [Point]) {
    pts[2].x = pts[3].x;
    pts[4].x = pts[3].x;
}

pub fn sk_find_cubic_inflections(src: &[Point], t_values: &mut [Scalar; 2]) -> i32 {
    let ax = src[1].x - src[0].x;
    let ay = src[1].y - src[0].y;
    let bx = src[2].x - 2.0 * src[1].x + src[0].x;
    let by = src[2].y - 2.0 * src[1].y + src[0].y;
    let cx = src[3].x + 3.0 * (src[1].x - src[2].x) - src[0].x;
    let cy = src[3].y + 3.0 * (src[1].y - src[2].y) - src[0].y;

    sk_find_unit_quad_roots(
        bx * cy - by * cx,
        ax * cy - ay * cx,
        ax * by - ay * bx,
        t_values,
    )
}

pub fn sk_chop_cubic_at_inflections(src: &[Point], dst: &mut [Point]) -> i32 {
    let mut t_values = [0.0; 2];
    let count = sk_find_cubic_inflections(src, &mut t_values);

    if count == 0 {
        dst[..4].copy_from_slice(&src[..4]);
    } else {
        sk_chop_cubic_at_ts(src, dst, &t_values[..count as usize]);
    }
    count + 1
}

// Assumes the third component of points is 1.
// Calcs p0 . (p1 x p2)
fn calc_dot_cross_cubic(p0: Point, p1: Point, p2: Point) -> f64 {
    let x_comp = (p0.x as f64) * ((p1.y as f64) - (p2.y as f64));
    let y_comp = (p0.y as f64) * ((p2.x as f64) - (p1.x as f64));
    let w_comp = (p1.x as f64) * (p2.y as f64) - (p1.y as f64) * (p2.x as f64);
    x_comp + y_comp + w_comp
}

/// Returns a positive power of 2 that, when multiplied by n, shifts the exponent of n to yield
/// a magnitude somewhere inside [1..2). Returns 2^1023 if `abs(n) < 2^-1022` (including 0).
/// Returns NaN if n is Inf or NaN.
fn previous_inverse_pow2(n: f64) -> f64 {
    let bits = n.to_bits();
    // exp=-exp
    let bits = ((1023u64 * 2) << 52)
        .wrapping_add((1u64 << 52) - 1)
        .wrapping_sub(bits);
    // mantissa=1.0, sign=0
    let bits = bits & (0x7ffu64 << 52);
    f64::from_bits(bits)
}

fn write_cubic_inflection_roots(
    t0: f64,
    s0: f64,
    t1: f64,
    s1: f64,
    t: &mut [f64; 2],
    s: &mut [f64; 2],
) {
    t[0] = t0;
    s[0] = s0;

    // This copysign/abs business orients the implicit function so positive values are always on
    // the "left" side of the curve.
    t[1] = -(t1.copysign(t1 * s1));
    s[1] = -s1.abs();

    // Ensure t[0]/s[0] <= t[1]/s[1] (s[1] is negative from above).
    if s[1].copysign(s[0]) * t[0] > -s[0].abs() * t[1] {
        t.swap(0, 1);
        s.swap(0, 1);
    }
}

/// Cubic classification types from `SkCubicType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkCubicType {
    Serpentine,
    Loop,
    LocalCusp,
    /// Cusp at a non-infinite parameter value with an inflection at t=infinity.
    CuspAtInfinity,
    Quadratic,
    LineOrPoint,
}

pub fn sk_cubic_is_degenerate(cubic_type: SkCubicType) -> bool {
    match cubic_type {
        SkCubicType::Serpentine
        | SkCubicType::Loop
        | SkCubicType::LocalCusp
        | SkCubicType::CuspAtInfinity => false,
        SkCubicType::Quadratic | SkCubicType::LineOrPoint => true,
    }
}

pub fn sk_cubic_type_name(cubic_type: SkCubicType) -> &'static str {
    match cubic_type {
        SkCubicType::Serpentine => "kSerpentine",
        SkCubicType::Loop => "kLoop",
        SkCubicType::LocalCusp => "kLocalCusp",
        SkCubicType::CuspAtInfinity => "kCuspAtInfinity",
        SkCubicType::Quadratic => "kQuadratic",
        SkCubicType::LineOrPoint => "kLineOrPoint",
    }
}

/// Classifies a cubic, optionally reporting the inflection parameters (`t`, `s`) and the
/// inflection-function coefficients (`d`).
pub fn sk_classify_cubic(
    p: &[Point],
    t: Option<&mut [f64; 2]>,
    s: Option<&mut [f64; 2]>,
    d: Option<&mut [f64; 4]>,
) -> SkCubicType {
    // Find the cubic's inflection function, I = [T^3  -3T^2  3T  -1] dot D.
    let a1 = calc_dot_cross_cubic(p[0], p[3], p[2]);
    let a2 = calc_dot_cross_cubic(p[1], p[0], p[3]);
    let a3 = calc_dot_cross_cubic(p[2], p[1], p[0]);

    let d3 = 3.0 * a3;
    let d2 = d3 - a2;
    let d1 = d2 - a2 + a1;

    // Shift the exponents in D so the largest magnitude falls somewhere in 1..2.
    let dmax = cpp_max_f64(cpp_max_f64(d1.abs(), d2.abs()), d3.abs());
    let norm = previous_inverse_pow2(dmax);
    let d1 = d1 * norm;
    let d2 = d2 * norm;
    let d3 = d3 * norm;

    if let Some(d) = d {
        d[3] = d3;
        d[2] = d2;
        d[1] = d1;
        d[0] = 0.0;
    }

    if 0.0 != d1 {
        let discr = 3.0 * d2 * d2 - 4.0 * d1 * d3;
        if discr > 0.0 {
            // Serpentine.
            if let (Some(t), Some(s)) = (t, s) {
                let q = 3.0 * d2 + (3.0 * discr).sqrt().copysign(d2);
                write_cubic_inflection_roots(q, 6.0 * d1, 2.0 * d3, q, t, s);
            }
            SkCubicType::Serpentine
        } else if discr < 0.0 {
            // Loop.
            if let (Some(t), Some(s)) = (t, s) {
                let q = d2 + (-discr).sqrt().copysign(d2);
                write_cubic_inflection_roots(q, 2.0 * d1, 2.0 * (d2 * d2 - d3 * d1), d1 * q, t, s);
            }
            SkCubicType::Loop
        } else {
            // Cusp.
            if let (Some(t), Some(s)) = (t, s) {
                write_cubic_inflection_roots(d2, 2.0 * d1, d2, 2.0 * d1, t, s);
            }
            SkCubicType::LocalCusp
        }
    } else if 0.0 != d2 {
        // Cusp at T=infinity.
        if let (Some(t), Some(s)) = (t, s) {
            write_cubic_inflection_roots(d3, 3.0 * d2, 1.0, 0.0, t, s); // T1=infinity.
        }
        SkCubicType::CuspAtInfinity
    } else {
        // Degenerate.
        if let (Some(t), Some(s)) = (t, s) {
            write_cubic_inflection_roots(1.0, 0.0, 1.0, 0.0, t, s); // T0=T1=infinity.
        }
        if 0.0 != d3 {
            SkCubicType::Quadratic
        } else {
            SkCubicType::LineOrPoint
        }
    }
}

/// Convenience form of `sk_classify_cubic` used when only the type is needed.
pub fn classify_cubic(p: &[Point]) -> SkCubicType {
    sk_classify_cubic(p, None, None, None)
}

fn bubble_sort(array: &mut [Scalar], count: usize) {
    for i in (1..count).rev() {
        for j in (1..=i).rev() {
            if array[j] < array[j - 1] {
                array.swap(j, j - 1);
            }
        }
    }
}

/// Given an array and count, remove all pair-wise duplicates from the array, keeping the
/// existing sorting, and return the new count.
fn collaps_duplicates(array: &mut [Scalar], count: usize) -> usize {
    let mut offset = 0usize;
    let mut n = count;
    let mut count = count;
    while n > 1 {
        if array[offset] == array[offset + 1] {
            for i in 1..n {
                array[offset + i - 1] = array[offset + i];
            }
            count -= 1;
        } else {
            offset += 1;
        }
        n -= 1;
    }
    count
}

fn sk_scalar_cube_root(x: Scalar) -> Scalar {
    scalar_pow(x, 0.3333333)
}

/// Solve `coeff(t) == 0`, returning the number of roots that lie within 0 < t < 1.
/// Eliminates repeated roots (so that all tValues are distinct and always in increasing order).
fn solve_cubic_poly(coeff: &[Scalar; 4], t_values: &mut [Scalar; 3]) -> i32 {
    if scalar_nearly_zero(coeff[0]) {
        // we're just a quadratic
        let mut quad = [t_values[0], t_values[1]];
        let count = sk_find_unit_quad_roots(coeff[1], coeff[2], coeff[3], &mut quad);
        t_values[0] = quad[0];
        t_values[1] = quad[1];
        return count;
    }

    let inva = scalar_invert(coeff[0]);
    let a = coeff[1] * inva;
    let b = coeff[2] * inva;
    let c = coeff[3] * inva;

    let q = (a * a - b * 3.0) / 9.0;
    let r = (2.0 * a * a * a - 9.0 * a * b + 27.0 * c) / 54.0;

    let q3 = q * q * q;
    let r2_minus_q3 = r * r - q3;
    let adiv3 = a / 3.0;

    if r2_minus_q3 < 0.0 {
        // we have 3 real roots
        // the divide/root can, due to finite precisions, be slightly outside of -1...1
        let theta = scalar_acos(scalar_pin(r / scalar_sqrt(q3), -1.0, 1.0));
        let neg2_root_q = -2.0 * scalar_sqrt(q);

        t_values[0] = scalar_pin(neg2_root_q * scalar_cos(theta / 3.0) - adiv3, 0.0, 1.0);
        t_values[1] = scalar_pin(
            neg2_root_q * scalar_cos((theta + 2.0 * SCALAR_PI) / 3.0) - adiv3,
            0.0,
            1.0,
        );
        t_values[2] = scalar_pin(
            neg2_root_q * scalar_cos((theta - 2.0 * SCALAR_PI) / 3.0) - adiv3,
            0.0,
            1.0,
        );

        // now sort the roots
        bubble_sort(t_values, 3);
        collaps_duplicates(t_values, 3) as i32
    } else {
        // we have 1 real root
        let mut a = scalar_abs(r) + scalar_sqrt(r2_minus_q3);
        a = sk_scalar_cube_root(a);
        if r > 0.0 {
            a = -a;
        }
        if a != 0.0 {
            a += q / a;
        }
        t_values[0] = scalar_pin(a - adiv3, 0.0, 1.0);
        1
    }
}

/// Looking for F' dot F'' == 0.
fn formulate_f1_dot_f2(src: &[Scalar], coeff: &mut [Scalar; 4]) {
    let a = src[2] - src[0];
    let b = src[4] - 2.0 * src[2] + src[0];
    let c = src[6] + 3.0 * (src[2] - src[4]) - src[0];

    coeff[0] = c * c;
    coeff[1] = 3.0 * b * c;
    coeff[2] = 2.0 * b * b + c * a;
    coeff[3] = a * b;
}

pub fn sk_find_cubic_max_curvature(src: &[Point], t_values: &mut [Scalar; 3]) -> i32 {
    let mut coeff_x = [0.0; 4];
    let mut coeff_y = [0.0; 4];

    // formulate_F1DotF2 reads an interleaved coordinate array.
    let xs = [src[0].x, src[1].x, src[2].x, src[3].x];
    let ys = [src[0].y, src[1].y, src[2].y, src[3].y];
    formulate_f1_dot_f2(&[xs[0], 0.0, xs[1], 0.0, xs[2], 0.0, xs[3]], &mut coeff_x);
    formulate_f1_dot_f2(&[ys[0], 0.0, ys[1], 0.0, ys[2], 0.0, ys[3]], &mut coeff_y);

    for i in 0..4 {
        coeff_x[i] += coeff_y[i];
    }

    solve_cubic_poly(&coeff_x, t_values)
}

pub fn sk_chop_cubic_at_max_curvature(
    src: &[Point],
    dst: Option<&mut [Point]>,
    t_values: Option<&mut [Scalar; 3]>,
) -> i32 {
    let mut t_storage = [0.0; 3];
    let t_values = match t_values {
        Some(values) => values,
        None => &mut t_storage,
    };

    let mut roots = [0.0; 3];
    let root_count = sk_find_cubic_max_curvature(src, &mut roots);

    // Throw out values not inside 0..1.
    let mut count = 0usize;
    for root in roots.iter().take(root_count as usize) {
        if 0.0 < *root && *root < 1.0 {
            t_values[count] = *root;
            count += 1;
        }
    }

    if let Some(dst) = dst {
        if count == 0 {
            dst[..4].copy_from_slice(&src[..4]);
        } else {
            sk_chop_cubic_at_ts(src, dst, &t_values[..count]);
        }
    }
    count as i32 + 1
}

fn calc_cubic_precision(src: &[Point]) -> Scalar {
    (PointPriv::distance_to_sqd(&src[1], &src[0])
        + PointPriv::distance_to_sqd(&src[2], &src[1])
        + PointPriv::distance_to_sqd(&src[3], &src[2]))
        * 1e-8
}

/// Returns true if both points `src[test_index]`, `src[test_index+1]` are in the same half plane
/// defined by the line segment `src[line_index]`, `src[line_index+1]`.
fn on_same_side(src: &[Point], test_index: usize, line_index: usize) -> bool {
    let origin = src[line_index];
    let line = src[line_index + 1] - origin;
    let mut crosses = [0.0; 2];
    for index in 0..2 {
        let test_line = src[test_index + index] - origin;
        crosses[index] = line.cross(&test_line);
    }
    crosses[0] * crosses[1] >= 0.0
}

/// Return location (in t) of cubic cusp, if there is one.
pub fn sk_find_cubic_cusp(src: &[Point]) -> Scalar {
    // When the adjacent control point matches the end point, it behaves as if
    // the cubic has a cusp: there's a point of max curvature where the derivative
    // goes to zero. Ideally, this would be where t is zero or one, but math
    // error makes not so.
    if src[0] == src[1] {
        return -1.0;
    }
    if src[2] == src[3] {
        return -1.0;
    }
    // Cubics only have a cusp if the line segments formed by the control and end points cross.
    if on_same_side(src, 0, 2) || on_same_side(src, 2, 0) {
        return -1.0;
    }
    let mut max_curvature = [0.0; 3];
    let roots = sk_find_cubic_max_curvature(src, &mut max_curvature);
    for test_t in max_curvature.iter().take(roots as usize) {
        let test_t = *test_t;
        if 0.0 >= test_t || test_t >= 1.0 {
            // no need to consider max curvature on the end
            continue;
        }
        // A cusp is at the max curvature, and also has a derivative close to zero.
        let d_pt = eval_cubic_derivative(src, test_t);
        let d_pt_magnitude = PointPriv::length_sqd(&d_pt);
        let precision = calc_cubic_precision(src);
        if d_pt_magnitude < precision {
            return test_t;
        }
    }
    -1.0
}

fn close_enough_to_zero(x: f64) -> bool {
    x.abs() < 0.00001
}

fn first_axis_intersection(
    coefficients: &[f64; 8],
    y_direction: bool,
    axis_intercept: f64,
    solution: &mut f64,
) -> bool {
    let [a, b, c, mut d] = SkBezierCubic::convert_to_polynomial(coefficients, y_direction);
    d -= axis_intercept;
    let mut roots = [0.0; 3];
    let mut count = SkCubics::roots_valid_t(a, b, c, d, &mut roots);
    if count == 0 {
        return false;
    }
    // Verify that at least one of the roots is accurate.
    for root in roots.iter().take(count) {
        if close_enough_to_zero(SkCubics::eval_at(a, b, c, d, *root)) {
            *solution = *root;
            return true;
        }
    }
    // None of the roots returned by our normal cubic solver were correct enough
    // (e.g. https://bugs.chromium.org/p/oss-fuzz/issues/detail?id=55732)
    // So we need to fallback to a more accurate solution.
    count = SkCubics::binary_search_roots_valid_t(a, b, c, d, &mut roots);
    if count == 0 {
        return false;
    }
    for root in roots.iter().take(count) {
        if close_enough_to_zero(SkCubics::eval_at(a, b, c, d, *root)) {
            *solution = *root;
            return true;
        }
    }
    false
}

pub fn sk_chop_mono_cubic_at_y(src: &[Point], y: Scalar, dst: &mut [Point]) -> bool {
    let coefficients = [
        src[0].x as f64,
        src[0].y as f64,
        src[1].x as f64,
        src[1].y as f64,
        src[2].x as f64,
        src[2].y as f64,
        src[3].x as f64,
        src[3].y as f64,
    ];
    let mut solution = 0.0;
    if first_axis_intersection(&coefficients, true, y as f64, &mut solution) {
        let mut cubic_pair = [0.0; 14];
        SkBezierCubic::subdivide(&coefficients, solution, &mut cubic_pair);
        for i in 0..7 {
            dst[i] = Point::new(
                double_to_float(cubic_pair[i * 2]),
                double_to_float(cubic_pair[i * 2 + 1]),
            );
        }
        true
    } else {
        false
    }
}

pub fn sk_chop_mono_cubic_at_x(src: &[Point], x: Scalar, dst: &mut [Point]) -> bool {
    let coefficients = [
        src[0].x as f64,
        src[0].y as f64,
        src[1].x as f64,
        src[1].y as f64,
        src[2].x as f64,
        src[2].y as f64,
        src[3].x as f64,
        src[3].y as f64,
    ];
    let mut solution = 0.0;
    if first_axis_intersection(&coefficients, false, x as f64, &mut solution) {
        let mut cubic_pair = [0.0; 14];
        SkBezierCubic::subdivide(&coefficients, solution, &mut cubic_pair);
        for i in 0..7 {
            dst[i] = Point::new(
                double_to_float(cubic_pair[i * 2]),
                double_to_float(cubic_pair[i * 2 + 1]),
            );
        }
        true
    } else {
        false
    }
}

// ---------------------------------------------------------------------------------------------
// NURB / conic representation
// ---------------------------------------------------------------------------------------------

/// `f(t) == A * t^2 + B * t + C` for one coordinate pair.
#[derive(Clone, Copy, Debug, Default)]
struct QuadCoeff {
    a: Point,
    b: Point,
    c: Point,
}

impl QuadCoeff {
    fn new(src: &[Point]) -> Self {
        let c = src[0];
        let p1 = src[1];
        let p2 = src[2];
        let b = times_2(p1 - c);
        let a = p2 - times_2(p1) + c;
        Self { a, b, c }
    }

    fn from_coeffs(a: Point, b: Point, c: Point) -> Self {
        Self { a, b, c }
    }

    fn eval(&self, t: Scalar) -> Point {
        (self.a * t + self.b) * t + self.c
    }
}

struct ConicCoeff {
    numer: QuadCoeff,
    denom: QuadCoeff,
}

impl ConicCoeff {
    fn new(conic: &SkConic) -> Self {
        let p0 = conic.pts[0];
        let p1 = conic.pts[1];
        let p2 = conic.pts[2];
        let ww = conic.w;

        let p1w = p1 * ww;
        let numer = QuadCoeff::from_coeffs(p2 - times_2(p1w) + p0, times_2(p1w - p0), p0);

        let denom_c = Point::new(1.0, 1.0);
        let denom_b = times_2(Point::new(ww, ww) - denom_c);
        let denom = QuadCoeff::from_coeffs(Point::new(0.0, 0.0) - denom_b, denom_b, denom_c);

        Self { numer, denom }
    }

    fn eval(&self, t: Scalar) -> Point {
        let numer = self.numer.eval(t);
        let denom = self.denom.eval(t);
        Point::new(numer.x / denom.x, numer.y / denom.y)
    }
}

/// `SkCubicCoeff`: power-basis form used by the cubic evaluator and by the upstream
/// `CubicMapTest` reference solver. The reference exposes it from `SkGeometry.h`; its
/// `fA`/`fB`/`fC`/`fD` are `[f32; 2]` lanes here represented by `Point` components.
#[derive(Clone, Copy, Debug, Default)]
pub struct SkCubicCoeff {
    pub a: Point,
    pub b: Point,
    pub c: Point,
    pub d: Point,
}

pub type CubicCoeff = SkCubicCoeff;

impl SkCubicCoeff {
    pub fn new(src: &[Point]) -> Self {
        let p0 = src[0];
        let p1 = src[1];
        let p2 = src[2];
        let p3 = src[3];
        let three = 3.0;
        Self {
            a: p3 + (p1 - p2) * three - p0,
            b: (p2 - times_2(p1) + p0) * three,
            c: (p1 - p0) * three,
            d: p0,
        }
    }

    pub fn eval(&self, t: Scalar) -> Point {
        ((self.a * t + self.b) * t + self.c) * t + self.d
    }
}

fn conic_deriv_coeff(src: &[Scalar], w: Scalar, coeff: &mut [Scalar; 3]) {
    let p20 = src[4] - src[0];
    let p10 = src[2] - src[0];
    let w_p10 = w * p10;
    coeff[0] = w * p20 - p20;
    coeff[1] = p20 - 2.0 * w_p10;
    coeff[2] = w_p10;
}

fn conic_find_extrema(src: &[Scalar], w: Scalar, t: &mut Scalar) -> bool {
    let mut coeff = [0.0; 3];
    conic_deriv_coeff(src, w, &mut coeff);

    let mut t_values = [0.0; 2];
    let roots = sk_find_unit_quad_roots(coeff[0], coeff[1], coeff[2], &mut t_values);
    debug_assert!(0 == roots || 1 == roots);

    if 1 == roots {
        *t = t_values[0];
        true
    } else {
        false
    }
}

/// We only interpolate one dimension at a time (the first, at +0, +3, +6).
fn p3d_interp(values: [Scalar; 3], t: Scalar) -> [Scalar; 3] {
    let ab = values[0] + (values[1] - values[0]) * t;
    let bc = values[1] + (values[2] - values[1]) * t;
    [ab, ab + (bc - ab) * t, bc]
}

fn ratquad_map_to_3d(src: &[Point], w: Scalar, dst: &mut [Point3; 3]) {
    dst[0] = Point3::new(src[0].x, src[0].y, 1.0);
    dst[1] = Point3::new(src[1].x * w, src[1].y * w, w);
    dst[2] = Point3::new(src[2].x, src[2].y, 1.0);
}

fn project_down(src: Point3) -> Point {
    Point::new(src.x / src.z, src.y / src.z)
}

fn subdivide_w_value(w: Scalar) -> Scalar {
    scalar_sqrt(0.5 + w * 0.5)
}

/// `SkConic`: a rational quadratic Bézier (`fPts`, `fW`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkConic {
    pub pts: [Point; 3],
    pub w: Scalar,
}

pub type Conic = SkConic;

impl Default for SkConic {
    fn default() -> Self {
        Self {
            pts: [Point::default(); 3],
            w: 1.0,
        }
    }
}

impl SkConic {
    pub fn new(p0: Point, p1: Point, p2: Point, w: Scalar) -> Self {
        let mut conic = Self {
            pts: [p0, p1, p2],
            w: 1.0,
        };
        conic.set_w(w);
        conic
    }

    pub fn from_pts(pts: &[Point], w: Scalar) -> Self {
        let mut conic = Self {
            pts: [pts[0], pts[1], pts[2]],
            w: 1.0,
        };
        conic.set_w(w);
        conic
    }

    pub fn set(&mut self, pts: &[Point], w: Scalar) {
        self.pts = [pts[0], pts[1], pts[2]];
        self.set_w(w);
    }

    pub fn set_pts_w(&mut self, p0: Point, p1: Point, p2: Point, w: Scalar) {
        self.pts = [p0, p1, p2];
        self.set_w(w);
    }

    /// Guard against bad weights by forcing them to 1.
    pub fn set_w(&mut self, w: Scalar) {
        self.w = if w > 0.0 && is_finite(w) { w } else { 1.0 };
    }

    pub fn eval_at(&self, t: Scalar) -> Point {
        ConicCoeff::new(self).eval(t)
    }

    pub fn eval_at_with(&self, t: Scalar, pt: Option<&mut Point>, tangent: Option<&mut Vector>) {
        debug_assert!((0.0..=1.0).contains(&t));

        if let Some(pt) = pt {
            *pt = self.eval_at(t);
        }
        if let Some(tangent) = tangent {
            *tangent = self.eval_tangent_at(t);
        }
    }

    pub fn eval_tangent_at(&self, t: Scalar) -> Vector {
        // The derivative equation returns a zero tangent vector when t is 0 or 1,
        // and the control point is equal to the end point.
        // In this case, use the conic endpoints to compute the tangent.
        if (t == 0.0 && self.pts[0] == self.pts[1]) || (t == 1.0 && self.pts[1] == self.pts[2]) {
            return self.pts[2] - self.pts[0];
        }
        let p0 = self.pts[0];
        let p1 = self.pts[1];
        let p2 = self.pts[2];
        let ww = self.w;

        let p20 = p2 - p0;
        let p10 = p1 - p0;

        let c = p10 * ww;
        let a = p20 * ww - p20;
        let b = p20 - c - c;

        QuadCoeff::from_coeffs(a, b, c).eval(t)
    }

    /// Return false if infinity or NaN is generated; caller must check.
    pub fn chop_at(&self, t: Scalar, dst: &mut [SkConic; 2]) -> bool {
        let mut tmp = [Point3::default(); 3];
        let mut tmp2 = [Point3::default(); 3];

        ratquad_map_to_3d(&self.pts, self.w, &mut tmp);

        let xs = p3d_interp([tmp[0].x, tmp[1].x, tmp[2].x], t);
        let ys = p3d_interp([tmp[0].y, tmp[1].y, tmp[2].y], t);
        let zs = p3d_interp([tmp[0].z, tmp[1].z, tmp[2].z], t);
        for i in 0..3 {
            tmp2[i] = Point3::new(xs[i], ys[i], zs[i]);
        }

        dst[0].pts[0] = self.pts[0];
        dst[0].pts[1] = project_down(tmp2[0]);
        dst[0].pts[2] = project_down(tmp2[1]);
        dst[1].pts[0] = dst[0].pts[2];
        dst[1].pts[1] = project_down(tmp2[2]);
        dst[1].pts[2] = self.pts[2];

        // to put in "standard form", where w0 and w2 are both 1, we compute the
        // new w1 as sqrt(w1*w1/w0*w2) or w1 /= sqrt(w0*w2).
        // However, in our case we know that for dst[0]: w0 == 1, and for dst[1], w2 == 1.
        let root = scalar_sqrt(tmp2[1].z);
        dst[0].w = tmp2[0].z / root;
        dst[1].w = tmp2[2].z / root;

        let mut finite = true;
        for conic in dst.iter() {
            finite &= conic.pts.iter().all(|p| p.is_finite()) && conic.w.is_finite();
        }
        finite
    }

    pub fn chop_at_two(&self, t1: Scalar, t2: Scalar, dst: &mut SkConic) {
        if 0.0 == t1 || 1.0 == t2 {
            if 0.0 == t1 && 1.0 == t2 {
                *dst = *self;
                return;
            } else {
                let mut pair = [SkConic::default(); 2];
                let chop_t = if t1 != 0.0 { t1 } else { t2 };
                if self.chop_at(chop_t, &mut pair) {
                    *dst = pair[if t1 != 0.0 { 1 } else { 0 }];
                    return;
                }
            }
        }
        let coeff = ConicCoeff::new(self);
        let a_xy = coeff.numer.eval(t1);
        let a_zz = coeff.denom.eval(t1);
        let mid_t = (t1 + t2) / 2.0;
        let d_xy = coeff.numer.eval(mid_t);
        let d_zz = coeff.denom.eval(mid_t);
        let c_xy = coeff.numer.eval(t2);
        let c_zz = coeff.denom.eval(t2);
        let b_xy = times_2(d_xy) - (a_xy + c_xy) * 0.5;
        let b_zz = times_2(d_zz) - (a_zz + c_zz) * 0.5;
        dst.pts[0] = Point::new(a_xy.x / a_zz.x, a_xy.y / a_zz.y);
        dst.pts[1] = Point::new(b_xy.x / b_zz.x, b_xy.y / b_zz.y);
        dst.pts[2] = Point::new(c_xy.x / c_zz.x, c_xy.y / c_zz.y);
        let ww_x = b_zz.x / (a_zz.x * c_zz.x).sqrt();
        let _ww_y = b_zz.y / (a_zz.y * c_zz.y).sqrt();
        dst.w = ww_x;
    }

    pub fn chop(&self, dst: &mut [SkConic; 2]) {
        // Observe that scale will always be smaller than 1 because fW > 0.
        let scale = 1.0 / (1.0 + self.w);

        // The subdivided control points below are the sums of the following three terms.
        let t0 = self.pts[0] * scale;
        let t1 = self.pts[1] * (self.w * scale);
        let t2 = self.pts[2] * scale;

        let p1 = t0 + t1;
        let p3 = t1 + t2;
        // p2 = (t0 + 2*t1 + t2) / 2.
        let p2 = t0 * 0.5 + t1 + t2 * 0.5;

        debug_assert!(p1.is_finite() && p2.is_finite() && p3.is_finite());

        dst[0].pts[0] = self.pts[0];
        dst[0].pts[1] = p1;
        dst[0].pts[2] = p2;
        dst[1].pts[0] = p2;
        dst[1].pts[1] = p3;
        dst[1].pts[2] = self.pts[2];

        let new_w = subdivide_w_value(self.w);
        dst[0].w = new_w;
        dst[1].w = new_w;
    }

    fn as_quad_error_setup(&self) -> (Scalar, Scalar) {
        let a = self.w - 1.0;
        let k = a / (4.0 * (2.0 + a));
        let x = k * (self.pts[0].x - 2.0 * self.pts[1].x + self.pts[2].x);
        let y = k * (self.pts[0].y - 2.0 * self.pts[1].y + self.pts[2].y);
        (x, y)
    }

    pub fn compute_as_quad_error(&self, err: &mut Vector) {
        // "High order approximation of conic sections by quadratic splines", Michael Floater.
        let (x, y) = self.as_quad_error_setup();
        *err = Vector::new(x, y);
    }

    pub fn as_quad_tol(&self, tol: Scalar) -> bool {
        let (x, y) = self.as_quad_error_setup();
        (x * x + y * y) <= tol * tol
    }

    /// Return the power-of-2 number of quads needed to approximate this conic with a sequence
    /// of quads. Will be >= 0.
    pub fn compute_quad_pow2(&self, tol: Scalar) -> i32 {
        const K_MAX_CONIC_TO_QUAD_POW2: i32 = 5;
        if tol < 0.0 || !is_finite(tol) || !PointPriv::are_finite(&self.pts) || bad_conic_w(self.w)
        {
            return 0;
        }

        let (x, y) = self.as_quad_error_setup();

        let mut error = scalar_sqrt(x * x + y * y);
        let mut pow2 = 0;
        while pow2 < K_MAX_CONIC_TO_QUAD_POW2 {
            if error <= tol {
                break;
            }
            error *= 0.25;
            pow2 += 1;
        }
        pow2
    }

    /// Chop this conic into N quads, stored contiguously in `pts`, where `N = 1 << pow2`.
    /// The amount of storage needed is `1 + 2 * N`.
    pub fn chop_into_quads_pow2(&self, pts: &mut [Point], pow2: i32) -> i32 {
        debug_assert!((0..=5).contains(&pow2));
        let mut pow2 = pow2;

        if bad_conic_w(self.w) {
            pow2 = 0;
        }

        pts[0] = self.pts[0];
        let end_pts: usize;
        if pow2 == 5 {
            // If an extreme weight generates many quads ...
            let mut dst = [SkConic::default(); 2];
            self.chop(&mut dst);
            // check to see if the first chop generates a pair of lines
            if PointPriv::equals_within_tolerance(&dst[0].pts[1], &dst[0].pts[2])
                && PointPriv::equals_within_tolerance(&dst[1].pts[0], &dst[1].pts[1])
            {
                // set ctrl == end to make lines
                pts[1] = dst[0].pts[1];
                pts[2] = dst[0].pts[1];
                pts[3] = dst[0].pts[1];
                pts[4] = dst[1].pts[2];
                pow2 = 1;
                end_pts = 5;
                return self.finish_chop_into_quads(pts, pow2, end_pts);
            }
        }
        end_pts = subdivide(self, pts, 1, pow2);
        self.finish_chop_into_quads(pts, pow2, end_pts)
    }

    fn finish_chop_into_quads(&self, pts: &mut [Point], pow2: i32, end_pts: usize) -> i32 {
        let quad_count = 1usize << pow2;
        let pt_count = 2 * quad_count + 1;
        debug_assert!(end_pts == pt_count);
        if !PointPriv::are_finite(&pts[..pt_count]) {
            // if we generated a non-finite, pin ourselves to the middle of the hull,
            // as our first and last are already on the first/last pts of the hull.
            for point in pts.iter_mut().take(pt_count - 1).skip(1) {
                *point = self.pts[1];
            }
        }
        1 << pow2
    }

    pub fn find_mid_tangent(&self) -> Scalar {
        // Tangents point in the direction of increasing T, so tan0 and -tan1 both point toward
        // the midtangent.
        let tan0 = self.pts[1] - self.pts[0];
        let tan1 = self.pts[2] - self.pts[1];
        let bisector = sk_find_bisector(tan0, -tan1);

        // Start by finding the tangent function's power basis coefficients.
        let a = (self.pts[2] - self.pts[0]) * (self.w - 1.0);
        let b = (self.pts[2] - self.pts[0]) - (self.pts[1] - self.pts[0]) * (self.w * 2.0);
        let c = (self.pts[1] - self.pts[0]) * self.w;

        // Now solve for "bisector dot midtangent = 0":
        let a = bisector.dot(&a);
        let b = bisector.dot(&b);
        let c = bisector.dot(&c);
        solve_quadratic_equation_for_midtangent(a, b, c)
    }

    pub fn find_x_extrema(&self) -> Option<Scalar> {
        let mut t = 0.0;
        if conic_find_extrema(&self.xs(), self.w, &mut t) {
            Some(t)
        } else {
            None
        }
    }

    pub fn find_y_extrema(&self) -> Option<Scalar> {
        let mut t = 0.0;
        if conic_find_extrema(&self.ys(), self.w, &mut t) {
            Some(t)
        } else {
            None
        }
    }

    fn xs(&self) -> [Scalar; 7] {
        [
            self.pts[0].x,
            0.0,
            self.pts[1].x,
            0.0,
            self.pts[2].x,
            0.0,
            0.0,
        ]
    }

    fn ys(&self) -> [Scalar; 7] {
        [
            self.pts[0].y,
            0.0,
            self.pts[1].y,
            0.0,
            self.pts[2].y,
            0.0,
            0.0,
        ]
    }

    pub fn chop_at_x_extrema(&self, dst: &mut [SkConic; 2]) -> bool {
        if let Some(t) = self.find_x_extrema() {
            if !self.chop_at(t, dst) {
                // if chop can't return finite values, don't chop
                return false;
            }
            // now clean-up the middle, since we know t was meant to be at an X-extrema
            let value = dst[0].pts[2].x;
            dst[0].pts[1].x = value;
            dst[1].pts[0].x = value;
            dst[1].pts[1].x = value;
            true
        } else {
            false
        }
    }

    pub fn chop_at_y_extrema(&self, dst: &mut [SkConic; 2]) -> bool {
        if let Some(t) = self.find_y_extrema() {
            if !self.chop_at(t, dst) {
                // if chop can't return finite values, don't chop
                return false;
            }
            // now clean-up the middle, since we know t was meant to be at a Y-extrema
            let value = dst[0].pts[2].y;
            dst[0].pts[1].y = value;
            dst[1].pts[0].y = value;
            dst[1].pts[1].y = value;
            true
        } else {
            false
        }
    }

    pub fn compute_tight_bounds(&self) -> Rect {
        let mut pts = [Point::default(); 4];
        pts[0] = self.pts[0];
        pts[1] = self.pts[2];
        let mut count = 2;

        if let Some(t) = self.find_x_extrema() {
            pts[count] = self.eval_at(t);
            count += 1;
        }
        if let Some(t) = self.find_y_extrema() {
            pts[count] = self.eval_at(t);
            count += 1;
        }
        Rect::bounds_or_empty(&pts[..count])
    }

    pub fn compute_fast_bounds(&self) -> Rect {
        Rect::bounds_or_empty(&self.pts)
    }

    pub fn transform_w(pts: &[Point], w: Scalar, matrix: &Matrix) -> Scalar {
        if !matrix.has_perspective() {
            return w;
        }

        let mut src = [Point3::default(); 3];
        let mut dst = [Point3::default(); 3];

        ratquad_map_to_3d(pts, w, &mut src);
        matrix.map_homogeneous_points(&mut dst, &src);

        // w' = sqrt(w1*w1/w0*w2)
        // use doubles temporarily, to handle small numer/denom
        let w0 = dst[0].z as f64;
        let w1 = dst[1].z as f64;
        let w2 = dst[2].z as f64;
        double_to_float((crate::floating_point::ieee_double_divide(w1 * w1, w0 * w2)).sqrt())
    }

    /// Build a conic for a unit arc. `out` must hold at least `kMaxConicsForArc` entries.
    pub fn build_unit_arc(
        u_start: Vector,
        u_stop: Vector,
        dir: SkPathDirection,
        user_matrix: Option<&Matrix>,
        out: &mut [SkConic],
    ) -> i32 {
        // rotate by x,y so that uStart is (1.0)
        let x = Point::dot_product(&u_start, &u_stop);
        let y = Point::cross_product(&u_start, &u_stop);

        let abs_y = scalar_abs(y);

        // check for (effectively) coincident vectors
        // this can happen if our angle is nearly 0 or nearly 180 (y == 0)
        if abs_y <= SCALAR_NEARLY_ZERO
            && x > 0.0
            && ((y >= 0.0 && SkPathDirection::Cw == dir)
                || (y <= 0.0 && SkPathDirection::Ccw == dir))
        {
            return 0;
        }

        let y = if SkPathDirection::Ccw == dir { -y } else { y };

        // We decide to use 1-conic per quadrant of a circle. What quadrant does [xy] lie in?
        let mut quadrant = 0;
        if 0.0 == y {
            quadrant = 2; // 180
        } else if 0.0 == x {
            quadrant = if y > 0.0 { 1 } else { 3 }; // 90 : 270
        } else {
            if y < 0.0 {
                quadrant += 2;
            }
            if (x < 0.0) != (y < 0.0) {
                quadrant += 1;
            }
        }

        const QUADRANT_PTS: [Point; 8] = [
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            Point::new(-1.0, 1.0),
            Point::new(-1.0, 0.0),
            Point::new(-1.0, -1.0),
            Point::new(0.0, -1.0),
            Point::new(1.0, -1.0),
        ];
        let quadrant_weight = SCALAR_ROOT2_OVER2;

        let mut conic_count = quadrant;
        for i in 0..conic_count as usize {
            out[i].set(&QUADRANT_PTS[i * 2..i * 2 + 3], quadrant_weight);
        }

        // Now compute any remaining (sub-90-degree) arc for the last conic.
        let final_p = Point::new(x, y);
        let last_q = QUADRANT_PTS[quadrant as usize * 2];
        let dot = Point::dot_product(&last_q, &final_p);
        if is_nan(dot) {
            return 0;
        }
        debug_assert!((0.0..=1.0 + SCALAR_NEARLY_ZERO).contains(&dot));

        if dot < 1.0 {
            let mut off_curve = Point::new(last_q.x + x, last_q.y + y);
            // compute the bisector vector, and then rescale to be the off-curve point.
            // we compute its length from cos(theta/2) = length / 1, using half-angle identity we
            // get length = sqrt(2 / (1 + cos(theta)).
            let cos_theta_over_2 = scalar_sqrt((1.0 + dot) / 2.0);
            off_curve.set_length(scalar_invert(cos_theta_over_2));
            if !PointPriv::equals_within_tolerance(&last_q, &off_curve) {
                out[conic_count as usize] =
                    SkConic::new(last_q, off_curve, final_p, cos_theta_over_2);
                conic_count += 1;
            }
        }

        // now handle counter-clockwise and the initial unitStart rotation
        let mut matrix = Matrix::new();
        matrix.set_sin_cos(u_start.y, u_start.x);
        if SkPathDirection::Ccw == dir {
            matrix.pre_scale(1.0, -1.0);
        }
        if let Some(user_matrix) = user_matrix {
            matrix.post_concat(user_matrix);
        }
        for conic in out.iter_mut().take(conic_count as usize) {
            matrix.map_points_in_place(&mut conic.pts);
        }
        conic_count
    }

    pub const K_MAX_CONICS_FOR_ARC: usize = 5;
}

fn bad_conic_w(w: Scalar) -> bool {
    w < 0.0 || !is_finite(w)
}

/// Returns true if `(a <= b <= c) || (a >= b >= c)`.
fn between(a: Scalar, b: Scalar, c: Scalar) -> bool {
    (a - b) * (c - b) <= 0.0
}

fn subdivide(src: &SkConic, pts: &mut [Point], offset: usize, level: i32) -> usize {
    debug_assert!(level >= 0);

    if 0 == level {
        pts[offset] = src.pts[1];
        pts[offset + 1] = src.pts[2];
        offset + 2
    } else {
        let mut dst = [SkConic::default(); 2];
        src.chop(&mut dst);
        let start_y = src.pts[0].y;
        let end_y = src.pts[2].y;
        if between(start_y, src.pts[1].y, end_y) {
            // If the input is monotonic and the output is not, the scan converter hangs.
            // Ensure that the chopped conics maintain their y-order.
            let mid_y = dst[0].pts[2].y;
            if !between(start_y, mid_y, end_y) {
                // If the computed midpoint is outside the ends, move it to the closer one.
                let closer_y = if scalar_abs(mid_y - start_y) < scalar_abs(mid_y - end_y) {
                    start_y
                } else {
                    end_y
                };
                dst[0].pts[2].y = closer_y;
                dst[1].pts[0].y = closer_y;
            }
            if !between(start_y, dst[0].pts[1].y, dst[0].pts[2].y) {
                // If the 1st control is not between the start and end, put it at the start.
                dst[0].pts[1].y = start_y;
            }
            if !between(dst[1].pts[0].y, dst[1].pts[1].y, end_y) {
                // If the 2nd control is not between the start and end, put it at the end.
                dst[1].pts[1].y = end_y;
            }
        }
        let level = level - 1;
        let offset = subdivide(&dst[0], pts, offset, level);
        subdivide(&dst[1], pts, offset, level)
    }
}

/// Help class to allocate storage for approximating a conic with N quads.
#[derive(Clone, Debug, Default)]
pub struct SkAutoConicToQuads {
    storage: Vec<Point>,
    quad_count: i32,
}

pub type AutoConicToQuads = SkAutoConicToQuads;

impl SkAutoConicToQuads {
    pub fn new() -> Self {
        Self::default()
    }

    /// Given a conic and a tolerance, return the points for the approximating quads.
    /// Call `count_quads()` to know the number of quads represented in these points.
    ///
    /// The quads are allocated to share end-points. e.g. if there are 4 quads,
    /// there will be 9 points allocated as follows: quad[0] == pts[0..2], quad[1] == pts[2..4],
    /// quad[2] == pts[4..6], quad[3] == pts[6..8].
    pub fn compute_quads(&mut self, conic: &SkConic, tol: Scalar) -> &[Point] {
        let pow2 = conic.compute_quad_pow2(tol);
        self.quad_count = 1 << pow2;
        let count = 1 + 2 * self.quad_count as usize;
        self.storage = vec![Point::default(); count];
        self.quad_count = conic.chop_into_quads_pow2(&mut self.storage, pow2);
        &self.storage[..1 + 2 * self.quad_count as usize]
    }

    pub fn compute_quads_pts(&mut self, pts: &[Point], weight: Scalar, tol: Scalar) -> &[Point] {
        if weight <= 0.0 {
            // The underlying conic.chopIntoQuads... is based on an algorithm that
            // requires a positive, non-zero weight. So for w = 0 (or any invalid w < 0),
            // we just make this a quad that is a line between the two points.
            self.quad_count = 1;
            self.storage = vec![Point::default(); 3];
            self.storage[0] = pts[0];
            self.storage[1] = Point::new((pts[0].x + pts[2].x) * 0.5, (pts[0].y + pts[2].y) * 0.5);
            self.storage[2] = pts[2];
            return &self.storage;
        }
        let mut conic = SkConic::default();
        conic.set(pts, weight);
        self.compute_quads(&conic, tol)
    }

    pub fn count_quads(&self) -> i32 {
        self.quad_count
    }
}
