//! Port of `src/core/SkStrokerPriv.h` and `src/core/SkStrokerPriv.cpp`.
//!
//! The reference stores the cap and join procedures as function pointers and identifies them by
//! pointer equality (`SkStrokerPriv::CapFactory(kButt_Cap) == fCapper`). The Rust port models the
//! same three-by-three factory set as enums whose variants stand in for the procedure addresses,
//! so the identity tests remain exact and the dispatch is still a real call. `JoinProc::call`
//! receives `&mut &mut SkPathBuilder` because the reference joiners swap the two builder pointers.

use crate::point::{Point, Vector};
use crate::point_priv::PointPriv;
use crate::scalar::{SCALAR_1, SCALAR_ROOT2_OVER2, Scalar, scalar_nearly_zero, scalar_sqrt};
use crate::sk_geometry::{SkConic, SkPathDirection};
use crate::sk_paint_types::{SkPaintCap, SkPaintJoin};
use crate::sk_path_builder::SkPathBuilder;

/// `CWX`/`CWY` pair: `SkPointPriv::RotateCW`.
#[inline]
pub fn rotate_cw(src: &Vector, dst: &mut Vector) {
    PointPriv::rotate_cw(src, dst);
}

/// `CUBIC_ARC_FACTOR` — `((SK_ScalarSqrt2 - SK_Scalar1) * 4 / 3)`.
pub const CUBIC_ARC_FACTOR: Scalar = (crate::scalar::SCALAR_SQRT2 - SCALAR_1) * 4.0 / 3.0;

/// One of the reference `CapProc` implementations selected by [`cap_factory`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapProc {
    Butt,
    Round,
    Square,
}

impl CapProc {
    /// Invoke the cap procedure.
    pub fn call(
        self,
        sink: &mut SkPathBuilder,
        pivot: Point,
        normal: Vector,
        stop: Point,
        extend_last_pt: bool,
    ) {
        match self {
            CapProc::Butt => butt_capper(sink, pivot, normal, stop, extend_last_pt),
            CapProc::Round => round_capper(sink, pivot, normal, stop, extend_last_pt),
            CapProc::Square => square_capper(sink, pivot, normal, stop, extend_last_pt),
        }
    }
}

/// One of the reference `JoinProc` implementations selected by [`join_factory`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinProc {
    Miter,
    Round,
    Blunt,
}

impl JoinProc {
    /// Invoke the join procedure. `outer` and `inner` are references so the procedure can swap
    /// which builder is the outer one, exactly like the reference's `SkPathBuilder*` swap.
    #[allow(clippy::too_many_arguments)]
    pub fn call<'a>(
        self,
        outer: &mut &'a mut SkPathBuilder,
        inner: &mut &'a mut SkPathBuilder,
        before_unit_normal: Vector,
        pivot: Point,
        after_unit_normal: Vector,
        radius: Scalar,
        inv_miter_limit: Scalar,
        prev_is_line: bool,
        curr_is_line: bool,
    ) {
        match self {
            JoinProc::Miter => miter_joiner(
                outer,
                inner,
                before_unit_normal,
                pivot,
                after_unit_normal,
                radius,
                inv_miter_limit,
                prev_is_line,
                curr_is_line,
            ),
            JoinProc::Round => round_joiner(
                outer,
                inner,
                before_unit_normal,
                pivot,
                after_unit_normal,
                radius,
                inv_miter_limit,
                prev_is_line,
                curr_is_line,
            ),
            JoinProc::Blunt => blunt_joiner(
                outer,
                inner,
                before_unit_normal,
                pivot,
                after_unit_normal,
                radius,
                inv_miter_limit,
                prev_is_line,
                curr_is_line,
            ),
        }
    }
}

/// `ButtCapper`.
pub fn butt_capper(
    sink: &mut SkPathBuilder,
    _pivot: Point,
    _normal: Vector,
    stop: Point,
    _extend_last_pt: bool,
) {
    sink.line_to_xy(stop.x, stop.y);
}

/// `RoundCapper`.
pub fn round_capper(
    sink: &mut SkPathBuilder,
    pivot: Point,
    normal: Vector,
    stop: Point,
    _extend_last_pt: bool,
) {
    let mut parallel = Vector::new(0.0, 0.0);
    PointPriv::rotate_cw(&normal, &mut parallel);

    let projected_center = pivot + parallel;

    sink.conic_to(
        projected_center + normal,
        projected_center,
        SCALAR_ROOT2_OVER2,
    );
    sink.conic_to(projected_center - normal, stop, SCALAR_ROOT2_OVER2);
}

/// `SquareCapper`.
pub fn square_capper(
    sink: &mut SkPathBuilder,
    pivot: Point,
    normal: Vector,
    stop: Point,
    extend_last_pt: bool,
) {
    let mut parallel = Vector::new(0.0, 0.0);
    PointPriv::rotate_cw(&normal, &mut parallel);

    if extend_last_pt {
        sink.set_last_point(pivot + normal + parallel);
        sink.line_to(pivot - normal + parallel);
    } else {
        sink.line_to(pivot + normal + parallel);
        sink.line_to(pivot - normal + parallel);
        sink.line_to(stop);
    }
}

#[inline]
fn is_clockwise(before: &Vector, after: &Vector) -> bool {
    before.x * after.y > before.y * after.x
}

/// `AngleType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AngleType {
    Nearly180,
    Sharp,
    Shallow,
    NearlyLine,
}

/// `Dot2AngleType`.
fn dot2_angle_type(dot: Scalar) -> AngleType {
    // need more precise fixed normalization
    if dot >= 0.0 {
        // shallow or line
        if scalar_nearly_zero(SCALAR_1 - dot) {
            AngleType::NearlyLine
        } else {
            AngleType::Shallow
        }
    } else {
        // sharp or 180
        if scalar_nearly_zero(SCALAR_1 + dot) {
            AngleType::Nearly180
        } else {
            AngleType::Sharp
        }
    }
}

/// `HandleInnerJoin`.
fn handle_inner_join(inner: &mut SkPathBuilder, pivot: Point, after: Vector) {
    /*  In the degenerate case that the stroke radius is larger than our segments
        just connecting the two inner segments may "show through" as a funny
        diagonal. To pseudo-fix this, we go through the pivot point. This adds
        an extra point/edge, but I can't see a cheap way to know when this is
        not needed :(
    */
    inner.line_to_xy(pivot.x, pivot.y);

    inner.line_to_xy(pivot.x - after.x, pivot.y - after.y);
}

/// `BluntJoiner`.
#[allow(clippy::too_many_arguments)]
pub fn blunt_joiner<'a>(
    outer: &mut &'a mut SkPathBuilder,
    inner: &mut &'a mut SkPathBuilder,
    before_unit_normal: Vector,
    pivot: Point,
    after_unit_normal: Vector,
    radius: Scalar,
    _inv_miter_limit: Scalar,
    _prev_is_line: bool,
    _curr_is_line: bool,
) {
    let mut after = Vector::new(0.0, 0.0);
    after_unit_normal.scale(radius, &mut after);

    if !is_clockwise(&before_unit_normal, &after_unit_normal) {
        std::mem::swap(outer, inner);
        after.negate();
    }

    outer.line_to_xy(pivot.x + after.x, pivot.y + after.y);
    handle_inner_join(inner, pivot, after);
}

/// `RoundJoiner`.
#[allow(clippy::too_many_arguments)]
pub fn round_joiner<'a>(
    outer: &mut &'a mut SkPathBuilder,
    inner: &mut &'a mut SkPathBuilder,
    before_unit_normal: Vector,
    pivot: Point,
    after_unit_normal: Vector,
    radius: Scalar,
    _inv_miter_limit: Scalar,
    _prev_is_line: bool,
    _curr_is_line: bool,
) {
    let dot_prod = Point::dot_product(&before_unit_normal, &after_unit_normal);
    let angle_type = dot2_angle_type(dot_prod);

    if angle_type == AngleType::NearlyLine {
        return;
    }

    let mut before = before_unit_normal;
    let mut after = after_unit_normal;
    let mut dir = SkPathDirection::Cw;

    if !is_clockwise(&before, &after) {
        std::mem::swap(outer, inner);
        before.negate();
        after.negate();
        dir = SkPathDirection::Ccw;
    }

    let mut matrix = crate::matrix::Matrix::new();
    matrix.set_scale(radius, radius);
    matrix.post_translate(pivot.x, pivot.y);
    let mut conics = [SkConic::default(); SkConic::K_MAX_CONICS_FOR_ARC];
    let count = SkConic::build_unit_arc(before, after, dir, Some(&matrix), &mut conics);
    if count > 0 {
        for conic in conics.iter().take(count as usize) {
            outer.conic_to(conic.pts[1], conic.pts[2], conic.w);
        }
        after.scale_in_place(radius);
        handle_inner_join(inner, pivot, after);
    }
}

/// `kOneOverSqrt2` — `(0.707106781f)`, the same literal as `SK_ScalarRoot2Over2`.
const K_ONE_OVER_SQRT2: Scalar = SCALAR_ROOT2_OVER2;

/// `MiterJoiner`.
#[allow(clippy::too_many_arguments)]
pub fn miter_joiner<'a>(
    outer: &mut &'a mut SkPathBuilder,
    inner: &mut &'a mut SkPathBuilder,
    before_unit_normal: Vector,
    pivot: Point,
    after_unit_normal: Vector,
    radius: Scalar,
    inv_miter_limit: Scalar,
    prev_is_line: bool,
    curr_is_line: bool,
) {
    // negate the dot since we're using normals instead of tangents
    let dot_prod = Point::dot_product(&before_unit_normal, &after_unit_normal);
    let angle_type = dot2_angle_type(dot_prod);
    let mut before = before_unit_normal;
    let mut after = after_unit_normal;
    let mut mid = Vector::new(0.0, 0.0);
    let mut curr_is_line = curr_is_line;
    let ccw: bool;
    let mut do_miter = false;

    if angle_type == AngleType::NearlyLine {
        return;
    }

    if angle_type != AngleType::Nearly180 {
        ccw = !is_clockwise(&before, &after);
        if ccw {
            std::mem::swap(outer, inner);
            before.negate();
            after.negate();
        }

        /*  Before we enter the world of square-roots and divides,
            check if we're trying to join an upright right angle
            (common case for stroking rectangles). If so, special case
            that (for speed an accuracy).
            Note: we only need to check one normal if dot==0
        */
        if 0.0 == dot_prod && inv_miter_limit <= K_ONE_OVER_SQRT2 {
            mid = (before + after) * radius;
            do_miter = true;
        } else {
            /*  midLength = radius / sinHalfAngle
                if (midLength > miterLimit * radius) abort
                if (radius / sinHalf > miterLimit * radius) abort
                if (1 / sinHalf > miterLimit) abort
                if (1 / miterLimit > sinHalf) abort
                My dotProd is opposite sign, since it is built from normals and not tangents
                hence 1 + dot instead of 1 - dot in the formula
            */
            let sin_half_angle = scalar_sqrt((SCALAR_1 + dot_prod) / 2.0);
            if sin_half_angle < inv_miter_limit {
                curr_is_line = false;
            } else {
                // choose the most accurate way to form the initial mid-vector
                if angle_type == AngleType::Sharp {
                    mid.set(after.y - before.y, before.x - after.x);
                    if ccw {
                        mid.negate();
                    }
                } else {
                    mid = before + after;
                }

                mid.set_length(radius / sin_half_angle);
                do_miter = true;
            }
        }
    } else {
        // kNearly180_AngleType: DO_BLUNT with currIsLine cleared.
        curr_is_line = false;
    }

    // DO_MITER
    if do_miter {
        if prev_is_line {
            outer.set_last_point(pivot + mid);
        } else {
            outer.line_to(pivot + mid);
        }
    }

    // DO_BLUNT (also reached by falling through from DO_MITER)
    after.scale_in_place(radius);
    if !curr_is_line {
        outer.line_to_xy(pivot.x + after.x, pivot.y + after.y);
    }
    handle_inner_join(inner, pivot, after);
}

/// `SkStrokerPriv::CapFactory`.
pub fn cap_factory(cap: SkPaintCap) -> CapProc {
    match cap {
        SkPaintCap::Butt => CapProc::Butt,
        SkPaintCap::Round => CapProc::Round,
        SkPaintCap::Square => CapProc::Square,
    }
}

/// `SkStrokerPriv::JoinFactory`.
pub fn join_factory(join: SkPaintJoin) -> JoinProc {
    match join {
        SkPaintJoin::Miter => JoinProc::Miter,
        SkPaintJoin::Round => JoinProc::Round,
        SkPaintJoin::Bevel => JoinProc::Blunt,
    }
}
