//! Port of `src/core/SkStroke.h` and `src/core/SkStroke.cpp`.
//!
//! The reference implements path stroking as `SkPathStroker` (curve flattening into outer/inner
//! offsets with cap and join procedures) plus the public `SkStroke` driver for paths and rects.
//! Both live in one translation unit upstream and are kept together here.
//!
//! Adaptations (all documented in `plan/handoffs/M08.md`):
//! - `SkPaint` is owned by M13, so `SkStroke`'s paint constructors are expressed through
//!   [`SkStroke::from_params`] taking the four paint stroke fields explicitly.
//! - `SkPath::Iter` is not copyable through a shared reference in Rust; `SkPathIter::clone` is used
//!   for `has_valid_tangent`, which is the same snapshot the reference takes with `Iter copy = *iter`.
//! - `SkQuadConstruct` is passed by reference instead of pointer.
//! - Reduction-type queries report the tangent point by index instead of by pointer into `cubic`.

use crate::point::{Point, Vector};
use crate::point_priv::PointPriv;
use crate::rect::Rect;
use crate::rrect::RRect;
use crate::scalar::{
    SCALAR_1, SCALAR_HALF, SCALAR_NEARLY_ZERO, SCALAR_SQRT2, Scalar, scalar_abs, scalar_invert,
    scalar_nearly_zero,
};
use crate::sk_geometry::{
    SkConic, sk_chop_cubic_at, sk_eval_cubic_at, sk_eval_quad_at, sk_eval_quad_at_with,
    sk_find_cubic_cusp, sk_find_cubic_inflections, sk_find_cubic_max_curvature,
    sk_find_quad_max_curvature, sk_find_unit_quad_roots,
};
use crate::sk_paint_types::{
    SK_PAINT_DEFAULT_CAP, SK_PAINT_DEFAULT_JOIN, SK_PAINT_DEFAULTS_MITER_LIMIT, SkPaintCap,
    SkPaintJoin,
};
use crate::sk_path::{AddPathMode, Iter, SkPath};
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_priv;
use crate::sk_path_types::{
    K_LINE_SK_PATH_SEGMENT_MASK, SkPathDirection, SkPathFirstDirection, SkPathVerb,
    SkResolveConvexity,
};
use crate::sk_stroker_priv::{CapProc, JoinProc, cap_factory, join_factory};

/// `kTangent_RecursiveLimit`, `kCubic_RecursiveLimit`, `kConic_RecursiveLimit`,
/// `kQuad_RecursiveLimit`.
pub const K_TANGENT_RECURSIVE_LIMIT: usize = 0;
pub const K_CUBIC_RECURSIVE_LIMIT: usize = 1;
pub const K_CONIC_RECURSIVE_LIMIT: usize = 2;
pub const K_QUAD_RECURSIVE_LIMIT: usize = 3;

/// `kRecursiveLimits`. The limits are somewhat arbitrarily chosen: the largest depth that won't
/// time out the fuzzer. For cubics that's 24; for quads and conics that's 16.
pub const K_RECURSIVE_LIMITS: [i32; 4] = [5 * 3, 24, 16, 16];

const _: () = assert!(K_RECURSIVE_LIMITS[K_TANGENT_RECURSIVE_LIMIT] == 15);
const _: () = assert!(K_RECURSIVE_LIMITS[K_CUBIC_RECURSIVE_LIMIT] == 24);

const K_OUTER_STROKE_TYPE: i32 = 1;
const K_INNER_STROKE_TYPE: i32 = -1;

/// `degenerate_vector`.
#[inline]
fn degenerate_vector(v: &Vector) -> bool {
    !PointPriv::can_normalize(v.x, v.y)
}

/// `set_normal_unitnormal(const SkPoint& before, const SkPoint& after, ...)`.
fn set_normal_unitnormal_between(
    before: &Point,
    after: &Point,
    scale: Scalar,
    radius: Scalar,
    normal: &mut Vector,
    unit_normal: &mut Vector,
) -> bool {
    if !unit_normal.set_normalize((after.x - before.x) * scale, (after.y - before.y) * scale) {
        return false;
    }
    PointPriv::rotate_ccw_in_place(unit_normal);
    unit_normal.scale(radius, normal);
    true
}

/// `set_normal_unitnormal(const SkVector& vec, ...)`.
fn set_normal_unitnormal_vec(
    vec: &Vector,
    radius: Scalar,
    normal: &mut Vector,
    unit_normal: &mut Vector,
) -> bool {
    if !unit_normal.set_normalize(vec.x, vec.y) {
        return false;
    }
    PointPriv::rotate_ccw_in_place(unit_normal);
    unit_normal.scale(radius, normal);
    true
}

/// `SkQuadConstruct`: the state of the quad stroke under construction.
#[derive(Clone, Copy, Debug)]
pub struct SkQuadConstruct {
    /// the stroked quad parallel to the original curve
    pub f_quad: [Point; 3],
    /// tangent vector at `f_quad[0]`
    pub f_tangent_start: Vector,
    /// tangent vector at `f_quad[2]`
    pub f_tangent_end: Vector,
    /// the start of a segment of the original curve
    pub f_start_t: Scalar,
    /// the middle of the segment
    pub f_mid_t: Scalar,
    /// the end of the segment
    pub f_end_t: Scalar,
    /// state to share common points across structs
    pub f_start_set: bool,
    /// state to share common points across structs
    pub f_end_set: bool,
    /// set if coincident tangents have opposite directions
    pub f_opposite_tangents: bool,
}

impl Default for SkQuadConstruct {
    fn default() -> Self {
        SkQuadConstruct {
            f_quad: [Point::new(0.0, 0.0); 3],
            f_tangent_start: Vector::new(0.0, 0.0),
            f_tangent_end: Vector::new(0.0, 0.0),
            f_start_t: 0.0,
            f_mid_t: 0.0,
            f_end_t: 0.0,
            f_start_set: false,
            f_end_set: false,
            f_opposite_tangents: false,
        }
    }
}

impl SkQuadConstruct {
    /// Return false if start and end are too close to have a unique middle.
    pub fn init(&mut self, start: Scalar, end: Scalar) -> bool {
        self.f_start_t = start;
        self.f_mid_t = (start + end) * SCALAR_HALF;
        self.f_end_t = end;
        self.f_start_set = false;
        self.f_end_set = false;
        self.f_start_t < self.f_mid_t && self.f_mid_t < self.f_end_t
    }

    pub fn init_with_start(&mut self, parent: &SkQuadConstruct) -> bool {
        if !self.init(parent.f_start_t, parent.f_mid_t) {
            return false;
        }
        self.f_quad[0] = parent.f_quad[0];
        self.f_tangent_start = parent.f_tangent_start;
        self.f_start_set = true;
        true
    }

    pub fn init_with_end(&mut self, parent: &SkQuadConstruct) -> bool {
        if !self.init(parent.f_mid_t, parent.f_end_t) {
            return false;
        }
        self.f_quad[2] = parent.f_quad[2];
        self.f_tangent_end = parent.f_tangent_end;
        self.f_end_set = true;
        true
    }
}

/// `isZeroLengthSincePoint`.
fn is_zero_length_since_point(pts: &[Point], start_pt_index: usize) -> bool {
    if pts.len() <= start_pt_index {
        return true;
    }
    let count = pts.len() - start_pt_index;
    if count < 2 {
        return true;
    }
    let pts = &pts[start_pt_index..];
    let first = pts[0];
    for point in pts.iter().skip(1) {
        if first != *point {
            return false;
        }
    }
    true
}

/// `SkPathStroker::ResultType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeResultType {
    /// the caller should split the quad stroke in two
    Split,
    /// the caller should add a line
    Degenerate,
    /// the caller should (continue to try to) add a quad stroke
    Quad,
}

/// `SkPathStroker::ReductionType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReductionType {
    /// all curve points are practically identical
    Point,
    /// the control point is on the line between the ends
    Line,
    /// the control point is outside the line between the ends
    Quad,
    /// the control point is on the line but outside the ends
    Degenerate,
    /// two control points are on the line but outside ends (cubic)
    Degenerate2,
    /// three areas of max curvature found (for cubic)
    Degenerate3,
}

/// `SkPathStroker::IntersectRayType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntersectRayType {
    CtrlPt,
    ResultType,
}

/// `SkPathStroker` — the utility class that constructs paths by stroking geometries.
pub struct SkPathStroker {
    f_radius: Scalar,
    f_inv_miter_limit: Scalar,
    f_res_scale: Scalar,
    f_inv_res_scale: Scalar,
    f_inv_res_scale_squared: Scalar,

    f_first_normal: Vector,
    f_prev_normal: Vector,
    f_first_unit_normal: Vector,
    f_prev_unit_normal: Vector,
    f_first_pt: Point,
    f_prev_pt: Point,
    f_first_outer_pt: Point,
    f_first_outer_pt_index_in_contour: usize,
    f_segment_count: i32,
    f_prev_is_line: bool,
    f_can_ignore_center: bool,

    f_capper: CapProc,
    f_joiner: JoinProc,

    f_inner: SkPathBuilder,
    f_outer: SkPathBuilder,
    f_cusper: SkPathBuilder,

    f_stroke_type: i32,

    f_recursion_depth: i32,
    f_found_tangents: bool,
    f_join_completed: bool,
}

impl SkPathStroker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        src: &SkPath,
        radius: Scalar,
        miter_limit: Scalar,
        cap: SkPaintCap,
        join: SkPaintJoin,
        res_scale: Scalar,
        can_ignore_center: bool,
    ) -> Self {
        let mut stroker = SkPathStroker {
            f_radius: radius,
            f_inv_miter_limit: 0.0,
            f_res_scale: res_scale,
            f_inv_res_scale: 0.0,
            f_inv_res_scale_squared: 0.0,
            f_first_normal: Vector::new(0.0, 0.0),
            f_prev_normal: Vector::new(0.0, 0.0),
            f_first_unit_normal: Vector::new(0.0, 0.0),
            f_prev_unit_normal: Vector::new(0.0, 0.0),
            f_first_pt: Point::new(0.0, 0.0),
            f_prev_pt: Point::new(0.0, 0.0),
            f_first_outer_pt: Point::new(0.0, 0.0),
            f_first_outer_pt_index_in_contour: 0,
            f_segment_count: -1,
            f_prev_is_line: false,
            f_can_ignore_center: can_ignore_center,
            f_capper: CapProc::Butt,
            f_joiner: JoinProc::Miter,
            f_inner: SkPathBuilder::new(),
            f_outer: SkPathBuilder::new(),
            f_cusper: SkPathBuilder::new(),
            f_stroke_type: K_INNER_STROKE_TYPE,
            f_recursion_depth: 0,
            f_found_tangents: false,
            f_join_completed: false,
        };

        /*  This is only used when join is miter_join, but we initialize it here
            so that it is always defined, to fix sanitizer warnings.
        */
        stroker.f_inv_miter_limit = 0.0;

        let mut join = join;
        if join == SkPaintJoin::Miter {
            if miter_limit <= SCALAR_1 {
                join = SkPaintJoin::Bevel;
            } else {
                stroker.f_inv_miter_limit = scalar_invert(miter_limit);
            }
        }
        stroker.f_capper = cap_factory(cap);
        stroker.f_joiner = join_factory(join);
        stroker.f_segment_count = -1;
        stroker.f_first_outer_pt_index_in_contour = 0;
        stroker.f_prev_is_line = false;

        // Need some estimate of how large our final result (fOuter) and our per-contour temp
        // (fInner) will be, so we don't spend extra time repeatedly growing these arrays.
        //
        // 3x for result == inner + outer + join (swag)
        // 1x for inner == 'wag' (worst contour length would be better guess)
        let count_points = src.count_points().max(0) as usize;
        stroker
            .f_outer
            .inc_reserve(count_points * 3, count_points * 3, 0);
        stroker.f_outer.set_is_volatile(true);
        stroker.f_inner.inc_reserve(count_points, count_points, 0);
        stroker.f_inner.set_is_volatile(true);
        // TODO : write a common error function used by stroking and filling
        // The '4' below matches the fill scan converter's error term
        stroker.f_inv_res_scale = scalar_invert(res_scale * 4.0);
        stroker.f_inv_res_scale_squared = stroker.f_inv_res_scale * stroker.f_inv_res_scale;
        stroker.f_recursion_depth = 0;
        stroker
    }

    pub fn has_only_move_to(&self) -> bool {
        0 == self.f_segment_count
    }

    pub fn move_to_pt(&self) -> Point {
        self.f_first_pt
    }

    pub fn get_res_scale(&self) -> Scalar {
        self.f_res_scale
    }

    pub fn is_current_contour_empty(&self) -> bool {
        is_zero_length_since_point(self.f_inner.points(), 0)
            && is_zero_length_since_point(
                self.f_outer.points(),
                self.f_first_outer_pt_index_in_contour,
            )
    }

    pub fn move_to(&mut self, pt: Point) {
        if self.f_segment_count > 0 {
            self.finish_contour(false, false);
        }
        self.f_segment_count = 0;
        self.f_first_pt = pt;
        self.f_prev_pt = pt;
        self.f_join_completed = false;
    }

    fn line_to(&mut self, curr_pt: Point, normal: &Vector) {
        self.f_outer.line_to(curr_pt + *normal);
        self.f_inner.line_to(curr_pt - *normal);
    }

    fn pre_join_to(
        &mut self,
        curr_pt: Point,
        normal: &mut Vector,
        unit_normal: &mut Vector,
        curr_is_line: bool,
    ) -> bool {
        debug_assert!(self.f_segment_count >= 0);

        if !set_normal_unitnormal_between(
            &self.f_prev_pt,
            &curr_pt,
            self.f_res_scale,
            self.f_radius,
            normal,
            unit_normal,
        ) {
            if self.f_capper == CapProc::Butt {
                return false;
            }
            /* Square caps and round caps draw even if the segment length is zero.
            Since the zero length segment has no direction, set the orientation
            to upright as the default orientation */
            normal.set(self.f_radius, 0.0);
            unit_normal.set(1.0, 0.0);
        }

        if self.f_segment_count == 0 {
            self.f_first_normal = *normal;
            self.f_first_unit_normal = *unit_normal;
            self.f_first_outer_pt = self.f_prev_pt + *normal;

            self.f_outer.move_to(self.f_first_outer_pt);
            self.f_inner.move_to(self.f_prev_pt - *normal);
        } else {
            // we have a previous segment
            let joiner = self.f_joiner;
            let mut outer = &mut self.f_outer;
            let mut inner = &mut self.f_inner;
            joiner.call(
                &mut outer,
                &mut inner,
                self.f_prev_unit_normal,
                self.f_prev_pt,
                *unit_normal,
                self.f_radius,
                self.f_inv_miter_limit,
                self.f_prev_is_line,
                curr_is_line,
            );
        }
        self.f_prev_is_line = curr_is_line;
        true
    }

    fn post_join_to(&mut self, curr_pt: Point, normal: &Vector, unit_normal: &Vector) {
        self.f_join_completed = true;
        self.f_prev_pt = curr_pt;
        self.f_prev_unit_normal = *unit_normal;
        self.f_prev_normal = *normal;
        self.f_segment_count += 1;
    }

    fn finish_contour(&mut self, close: bool, curr_is_line: bool) {
        if self.f_segment_count > 0 {
            if close {
                let joiner = self.f_joiner;
                let mut outer = &mut self.f_outer;
                let mut inner = &mut self.f_inner;
                joiner.call(
                    &mut outer,
                    &mut inner,
                    self.f_prev_unit_normal,
                    self.f_prev_pt,
                    self.f_first_unit_normal,
                    self.f_radius,
                    self.f_inv_miter_limit,
                    self.f_prev_is_line,
                    curr_is_line,
                );
                self.f_outer.close();

                if self.f_can_ignore_center {
                    // If we can ignore the center just make sure the larger of the two paths
                    // is preserved and don't add the smaller one.
                    if self
                        .f_inner
                        .compute_bounds()
                        .contains_rect(&self.f_outer.compute_bounds())
                    {
                        self.f_outer = self.f_inner.clone();
                    }
                } else {
                    // now add fInner as its own contour
                    if let Some(pt) = self.f_inner.get_last_pt() {
                        self.f_outer.move_to(pt);
                        let inner_path = self.f_inner.detach(None);
                        self.f_outer.private_reverse_path_to(&inner_path);
                        self.f_outer.close();
                    }
                }
            } else {
                // add caps to start and end
                // cap the end
                if let Some(pt) = self.f_inner.get_last_pt() {
                    self.f_capper.call(
                        &mut self.f_outer,
                        self.f_prev_pt,
                        self.f_prev_normal,
                        pt,
                        curr_is_line,
                    );
                    let inner_path = self.f_inner.detach(None);
                    self.f_outer.private_reverse_path_to(&inner_path);
                    // cap the start
                    self.f_capper.call(
                        &mut self.f_outer,
                        self.f_first_pt,
                        -self.f_first_normal,
                        self.f_first_outer_pt,
                        self.f_prev_is_line,
                    );
                    self.f_outer.close();
                }
            }
            if !self.f_cusper.is_empty() {
                let cusper = self.f_cusper.detach(None);
                self.f_outer.add_path(&cusper, AddPathMode::Append);
            }
        }
        self.f_inner.reset();
        self.f_segment_count = -1;
        self.f_first_outer_pt_index_in_contour = self.f_outer.count_points().max(0) as usize;
    }

    pub fn close(&mut self, is_line: bool) {
        self.finish_contour(true, is_line);
    }

    pub fn done(&mut self, dst: &mut SkPathBuilder, is_line: bool) {
        self.finish_contour(false, is_line);
        // Reference: `*dst = fOuter; fOuter.reset();`
        *dst = std::mem::take(&mut self.f_outer);
        self.f_outer.reset();
    }

    pub fn line_to_iter(&mut self, curr_pt: Point, iter: Option<&Iter<'_>>) {
        let teeny_line = PointPriv::equals_within_tolerance_within(
            &self.f_prev_pt,
            &curr_pt,
            SCALAR_NEARLY_ZERO * self.f_inv_res_scale,
        );
        if self.f_capper == CapProc::Butt && teeny_line {
            return;
        }
        if teeny_line
            && (self.f_join_completed || (iter.is_some() && has_valid_tangent(iter.unwrap())))
        {
            return;
        }
        let mut normal = Vector::new(0.0, 0.0);
        let mut unit_normal = Vector::new(0.0, 0.0);

        if !self.pre_join_to(curr_pt, &mut normal, &mut unit_normal, true) {
            return;
        }
        self.line_to(curr_pt, &normal);
        self.post_join_to(curr_pt, &normal, &unit_normal);
    }

    /// `SkPathStroker::lineTo(const SkPoint&)`.
    pub fn line_to_pt(&mut self, curr_pt: Point) {
        self.line_to_iter(curr_pt, None);
    }

    fn set_quad_end_normal(
        &self,
        quad: &[Point; 3],
        normal_ab: &Vector,
        unit_normal_ab: &Vector,
        normal_bc: &mut Vector,
        unit_normal_bc: &mut Vector,
    ) {
        if !set_normal_unitnormal_between(
            &quad[1],
            &quad[2],
            self.f_res_scale,
            self.f_radius,
            normal_bc,
            unit_normal_bc,
        ) {
            *normal_bc = *normal_ab;
            *unit_normal_bc = *unit_normal_ab;
        }
    }

    fn set_conic_end_normal(
        &self,
        conic: &SkConic,
        normal_ab: &Vector,
        unit_normal_ab: &Vector,
        normal_bc: &mut Vector,
        unit_normal_bc: &mut Vector,
    ) {
        self.set_quad_end_normal(
            &conic.pts,
            normal_ab,
            unit_normal_ab,
            normal_bc,
            unit_normal_bc,
        );
    }

    fn set_cubic_end_normal(
        &self,
        cubic: &[Point; 4],
        normal_ab: &Vector,
        unit_normal_ab: &Vector,
        normal_cd: &mut Vector,
        unit_normal_cd: &mut Vector,
    ) {
        let mut ab = cubic[1] - cubic[0];
        let mut cd = cubic[3] - cubic[2];

        let mut degenerate_ab = degenerate_vector(&ab);
        let mut degenerate_cd = degenerate_vector(&cd);

        if degenerate_ab && degenerate_cd {
            *normal_cd = *normal_ab;
            *unit_normal_cd = *unit_normal_ab;
            return;
        }

        if degenerate_ab {
            ab = cubic[2] - cubic[0];
            degenerate_ab = degenerate_vector(&ab);
        }
        if degenerate_cd {
            cd = cubic[3] - cubic[1];
            degenerate_cd = degenerate_vector(&cd);
        }
        if degenerate_ab || degenerate_cd {
            *normal_cd = *normal_ab;
            *unit_normal_cd = *unit_normal_ab;
            return;
        }
        let _ = set_normal_unitnormal_vec(&cd, self.f_radius, normal_cd, unit_normal_cd);
    }

    fn init(
        &mut self,
        stroke_type: i32,
        quad_pts: &mut SkQuadConstruct,
        t_start: Scalar,
        t_end: Scalar,
    ) {
        self.f_stroke_type = stroke_type;
        self.f_found_tangents = false;
        self.f_recursion_depth = 0;
        quad_pts.init(t_start, t_end);
    }

    fn conic_perp_ray(
        &self,
        conic: &SkConic,
        t: Scalar,
        t_pt: &mut Point,
        on_pt: &mut Point,
        tangent: Option<&mut Vector>,
    ) {
        let mut dxy = Vector::new(0.0, 0.0);
        conic.eval_at_with(t, Some(t_pt), Some(&mut dxy));
        if dxy.is_zero() {
            dxy = conic.pts[2] - conic.pts[0];
        }
        self.set_ray_pts(*t_pt, &mut dxy, on_pt, tangent);
    }

    fn conic_quad_ends(&self, conic: &SkConic, quad_pts: &mut SkQuadConstruct) {
        if !quad_pts.f_start_set {
            let mut conic_start_pt = Point::new(0.0, 0.0);
            self.conic_perp_ray(
                conic,
                quad_pts.f_start_t,
                &mut conic_start_pt,
                &mut quad_pts.f_quad[0],
                Some(&mut quad_pts.f_tangent_start),
            );
            quad_pts.f_start_set = true;
        }
        if !quad_pts.f_end_set {
            let mut conic_end_pt = Point::new(0.0, 0.0);
            self.conic_perp_ray(
                conic,
                quad_pts.f_end_t,
                &mut conic_end_pt,
                &mut quad_pts.f_quad[2],
                Some(&mut quad_pts.f_tangent_end),
            );
            quad_pts.f_end_set = true;
        }
    }

    fn cubic_perp_ray(
        &self,
        cubic: &[Point; 4],
        t: Scalar,
        t_pt: &mut Point,
        on_pt: &mut Point,
        tangent: Option<&mut Vector>,
    ) {
        let mut dxy = Vector::new(0.0, 0.0);
        let mut chopped = [Point::new(0.0, 0.0); 7];
        sk_eval_cubic_at(cubic, t, Some(t_pt), Some(&mut dxy), None);
        if dxy.is_zero() {
            let mut c_pts: &[Point] = cubic;
            if scalar_nearly_zero(t) {
                dxy = cubic[2] - cubic[0];
            } else if scalar_nearly_zero(1.0 - t) {
                dxy = cubic[3] - cubic[1];
            } else {
                // If the cubic inflection falls on the cusp, subdivide the cubic
                // to find the tangent at that point.
                sk_chop_cubic_at(cubic, &mut chopped, t);
                dxy = chopped[3] - chopped[2];
                if dxy.is_zero() {
                    dxy = chopped[3] - chopped[1];
                    c_pts = &chopped;
                }
            }
            if dxy.is_zero() {
                dxy = c_pts[3] - c_pts[0];
            }
        }
        self.set_ray_pts(*t_pt, &mut dxy, on_pt, tangent);
    }

    fn cubic_quad_ends(&self, cubic: &[Point; 4], quad_pts: &mut SkQuadConstruct) {
        if !quad_pts.f_start_set {
            let mut cubic_start_pt = Point::new(0.0, 0.0);
            self.cubic_perp_ray(
                cubic,
                quad_pts.f_start_t,
                &mut cubic_start_pt,
                &mut quad_pts.f_quad[0],
                Some(&mut quad_pts.f_tangent_start),
            );
            quad_pts.f_start_set = true;
        }
        if !quad_pts.f_end_set {
            let mut cubic_end_pt = Point::new(0.0, 0.0);
            self.cubic_perp_ray(
                cubic,
                quad_pts.f_end_t,
                &mut cubic_end_pt,
                &mut quad_pts.f_quad[2],
                Some(&mut quad_pts.f_tangent_end),
            );
            quad_pts.f_end_set = true;
        }
    }

    fn cubic_quad_mid(&self, cubic: &[Point; 4], quad_pts: &SkQuadConstruct, mid: &mut Point) {
        let mut cubic_mid_pt = Point::new(0.0, 0.0);
        self.cubic_perp_ray(cubic, quad_pts.f_mid_t, &mut cubic_mid_pt, mid, None);
    }

    fn quad_perp_ray(
        &self,
        quad: &[Point; 3],
        t: Scalar,
        t_pt: &mut Point,
        on_pt: &mut Point,
        tangent: Option<&mut Vector>,
    ) {
        let mut dxy = Vector::new(0.0, 0.0);
        sk_eval_quad_at_with(quad, t, Some(t_pt), Some(&mut dxy));
        if dxy.is_zero() {
            dxy = quad[2] - quad[0];
        }
        self.set_ray_pts(*t_pt, &mut dxy, on_pt, tangent);
    }

    /// `setRayPts`: scale the derivative by the radius, and compute the perpendicular point and
    /// its tangent.
    fn set_ray_pts(
        &self,
        t_pt: Point,
        dxy: &mut Vector,
        on_pt: &mut Point,
        tangent: Option<&mut Vector>,
    ) {
        if !dxy.set_length(self.f_radius) {
            dxy.set(self.f_radius, 0.0);
        }
        let axis_flip = self.f_stroke_type as Scalar; // go opposite ways for outer, inner
        on_pt.x = t_pt.x + axis_flip * dxy.y;
        on_pt.y = t_pt.y - axis_flip * dxy.x;
        if let Some(tangent) = tangent {
            *tangent = *dxy;
        }
    }

    fn intersect_ray(
        &self,
        quad_pts: &mut SkQuadConstruct,
        intersect_ray_type: IntersectRayType,
    ) -> StrokeResultType {
        let start = quad_pts.f_quad[0];
        let end = quad_pts.f_quad[2];
        let a_len = quad_pts.f_tangent_start;
        let b_len = quad_pts.f_tangent_end;
        /* Slopes match when denom goes to zero:
                          axLen / ayLen ==                   bxLen / byLen
        (ayLen * byLen) * axLen / ayLen == (ayLen * byLen) * bxLen / byLen
                 byLen  * axLen         ==  ayLen          * bxLen
                 byLen  * axLen         -   ayLen          * bxLen         ( == denom )
         */
        let denom = a_len.cross(&b_len);
        if denom == 0.0 || !denom.is_finite() {
            quad_pts.f_opposite_tangents = a_len.dot(&b_len) < 0.0;
            return StrokeResultType::Degenerate;
        }
        quad_pts.f_opposite_tangents = false;
        let ab0 = start - end;
        let mut numer_a = b_len.cross(&ab0);
        let numer_b = a_len.cross(&ab0);
        if (numer_a >= 0.0) == (numer_b >= 0.0) {
            // if the control point is outside the quad ends
            // if the perpendicular distances from the quad points to the opposite tangent line
            // are small, a straight line is good enough
            let dist1 = pt_to_tangent_line(&start, &end, &quad_pts.f_tangent_end);
            let dist2 = pt_to_tangent_line(&end, &start, &quad_pts.f_tangent_start);
            if cpp_max(dist1, dist2) <= self.f_inv_res_scale_squared {
                return StrokeResultType::Degenerate;
            }
            return StrokeResultType::Split;
        }
        // check to see if the denominator is teeny relative to the numerator
        // if the offset by one will be lost, the ratio is too large
        numer_a /= denom;
        let valid_divide = numer_a > numer_a - 1.0;
        if valid_divide {
            if intersect_ray_type == IntersectRayType::CtrlPt {
                // the intersection of the tangents need not be on the tangent segment
                // so 0 <= numerA <= 1 is not necessarily true
                quad_pts.f_quad[1] = start + quad_pts.f_tangent_start * numer_a;
            }
            return StrokeResultType::Quad;
        }
        quad_pts.f_opposite_tangents = a_len.dot(&b_len) < 0.0;
        // if the lines are parallel, straight line is good enough
        StrokeResultType::Degenerate
    }

    fn tangents_meet(
        &mut self,
        cubic: &[Point; 4],
        quad_pts: &mut SkQuadConstruct,
    ) -> StrokeResultType {
        self.cubic_quad_ends(cubic, quad_pts);
        self.intersect_ray(quad_pts, IntersectRayType::ResultType)
    }

    fn pt_in_quad_bounds(&self, quad: &[Point; 3], pt: &Point) -> bool {
        let x_min = cpp_min3(quad[0].x, quad[1].x, quad[2].x);
        if pt.x + self.f_inv_res_scale < x_min {
            return false;
        }
        let x_max = cpp_max3(quad[0].x, quad[1].x, quad[2].x);
        if pt.x - self.f_inv_res_scale > x_max {
            return false;
        }
        let y_min = cpp_min3(quad[0].y, quad[1].y, quad[2].y);
        if pt.y + self.f_inv_res_scale < y_min {
            return false;
        }
        let y_max = cpp_max3(quad[0].y, quad[1].y, quad[2].y);
        if pt.y - self.f_inv_res_scale > y_max {
            return false;
        }
        true
    }

    fn stroke_close_enough(
        &self,
        stroke: &[Point; 3],
        ray: &[Point; 2],
        quad_pts: &SkQuadConstruct,
    ) -> StrokeResultType {
        let stroke_mid = sk_eval_quad_at(stroke, SCALAR_HALF);
        // measure the distance from the curve to the quad-stroke midpoint, compare to radius
        if points_within_dist(&ray[0], &stroke_mid, self.f_inv_res_scale) {
            // if the difference is small
            if sharp_angle(&quad_pts.f_quad) {
                return StrokeResultType::Split;
            }
            return StrokeResultType::Quad;
        }
        // measure the distance to quad's bounds (quick reject)
        // an alternative : look for point in triangle
        if !self.pt_in_quad_bounds(stroke, &ray[0]) {
            // if far, subdivide
            return StrokeResultType::Split;
        }
        // measure the curve ray distance to the quad-stroke
        let mut roots = [0.0; 2];
        let root_count = intersect_quad_ray(ray, stroke, &mut roots);
        if root_count != 1 {
            return StrokeResultType::Split;
        }
        let quad_pt = sk_eval_quad_at(stroke, roots[0]);
        let error = self.f_inv_res_scale * (SCALAR_1 - scalar_abs(roots[0] - 0.5) * 2.0);
        if points_within_dist(&ray[0], &quad_pt, error) {
            // if the difference is small, we're done
            if sharp_angle(&quad_pts.f_quad) {
                return StrokeResultType::Split;
            }
            return StrokeResultType::Quad;
        }
        // otherwise, subdivide
        StrokeResultType::Split
    }

    fn compare_quad_cubic(
        &self,
        cubic: &[Point; 4],
        quad_pts: &mut SkQuadConstruct,
    ) -> StrokeResultType {
        // get the quadratic approximation of the stroke
        self.cubic_quad_ends(cubic, quad_pts);
        let result_type = self.intersect_ray(quad_pts, IntersectRayType::CtrlPt);
        if result_type != StrokeResultType::Quad {
            return result_type;
        }
        // project a ray from the curve to the stroke
        let mut ray = [Point::new(0.0, 0.0); 2]; // points near midpoint on quad, midpoint on cubic
        {
            let mid_t = quad_pts.f_mid_t;
            let mut on_curve = Point::new(0.0, 0.0);
            let mut on_stroke = Point::new(0.0, 0.0);
            self.cubic_perp_ray(cubic, mid_t, &mut on_curve, &mut on_stroke, None);
            ray[1] = on_curve;
            ray[0] = on_stroke;
        }
        self.stroke_close_enough(&quad_pts.f_quad, &ray, quad_pts)
    }

    fn compare_quad_conic(
        &self,
        conic: &SkConic,
        quad_pts: &mut SkQuadConstruct,
    ) -> StrokeResultType {
        // get the quadratic approximation of the stroke
        self.conic_quad_ends(conic, quad_pts);
        let result_type = self.intersect_ray(quad_pts, IntersectRayType::CtrlPt);
        if result_type != StrokeResultType::Quad {
            return result_type;
        }
        // project a ray from the curve to the stroke
        let mut ray = [Point::new(0.0, 0.0); 2]; // points near midpoint on quad, midpoint on conic
        {
            let mid_t = quad_pts.f_mid_t;
            let mut on_curve = Point::new(0.0, 0.0);
            let mut on_stroke = Point::new(0.0, 0.0);
            self.conic_perp_ray(conic, mid_t, &mut on_curve, &mut on_stroke, None);
            ray[1] = on_curve;
            ray[0] = on_stroke;
        }
        self.stroke_close_enough(&quad_pts.f_quad, &ray, quad_pts)
    }

    fn compare_quad_quad(
        &self,
        quad: &[Point; 3],
        quad_pts: &mut SkQuadConstruct,
    ) -> StrokeResultType {
        // get the quadratic approximation of the stroke
        if !quad_pts.f_start_set {
            let mut quad_start_pt = Point::new(0.0, 0.0);
            self.quad_perp_ray(
                quad,
                quad_pts.f_start_t,
                &mut quad_start_pt,
                &mut quad_pts.f_quad[0],
                Some(&mut quad_pts.f_tangent_start),
            );
            quad_pts.f_start_set = true;
        }
        if !quad_pts.f_end_set {
            let mut quad_end_pt = Point::new(0.0, 0.0);
            self.quad_perp_ray(
                quad,
                quad_pts.f_end_t,
                &mut quad_end_pt,
                &mut quad_pts.f_quad[2],
                Some(&mut quad_pts.f_tangent_end),
            );
            quad_pts.f_end_set = true;
        }
        let result_type = self.intersect_ray(quad_pts, IntersectRayType::CtrlPt);
        if result_type != StrokeResultType::Quad {
            return result_type;
        }
        // project a ray from the curve to the stroke
        let mut ray = [Point::new(0.0, 0.0); 2];
        {
            let mid_t = quad_pts.f_mid_t;
            let mut on_curve = Point::new(0.0, 0.0);
            let mut on_stroke = Point::new(0.0, 0.0);
            self.quad_perp_ray(quad, mid_t, &mut on_curve, &mut on_stroke, None);
            ray[1] = on_curve;
            ray[0] = on_stroke;
        }
        self.stroke_close_enough(&quad_pts.f_quad, &ray, quad_pts)
    }

    fn add_degenerate_line(&mut self, quad_pts: &SkQuadConstruct) {
        let quad = quad_pts.f_quad;
        let sink = if self.f_stroke_type == K_OUTER_STROKE_TYPE {
            &mut self.f_outer
        } else {
            &mut self.f_inner
        };
        sink.line_to(quad[2]);
    }

    fn cubic_mid_on_line(&self, cubic: &[Point; 4], quad_pts: &SkQuadConstruct) -> bool {
        let mut stroke_mid = Point::new(0.0, 0.0);
        self.cubic_quad_mid(cubic, quad_pts, &mut stroke_mid);
        let dist = pt_to_line(&stroke_mid, &quad_pts.f_quad[0], &quad_pts.f_quad[2]);
        dist < self.f_inv_res_scale_squared
    }

    fn cubic_stroke(&mut self, cubic: &[Point; 4], quad_pts: &mut SkQuadConstruct) -> bool {
        if !self.f_found_tangents {
            let result_type = self.tangents_meet(cubic, quad_pts);
            if StrokeResultType::Quad != result_type {
                if (StrokeResultType::Degenerate == result_type
                    || points_within_dist(
                        &quad_pts.f_quad[0],
                        &quad_pts.f_quad[2],
                        self.f_inv_res_scale,
                    ))
                    && self.cubic_mid_on_line(cubic, quad_pts)
                {
                    self.add_degenerate_line(quad_pts);
                    return true;
                }
            } else {
                self.f_found_tangents = true;
            }
        }
        if self.f_found_tangents {
            let result_type = self.compare_quad_cubic(cubic, quad_pts);
            if StrokeResultType::Quad == result_type {
                let stroke = quad_pts.f_quad;
                let sink = if self.f_stroke_type == K_OUTER_STROKE_TYPE {
                    &mut self.f_outer
                } else {
                    &mut self.f_inner
                };
                sink.quad_to(stroke[1], stroke[2]);
                return true;
            }
            if StrokeResultType::Degenerate == result_type && !quad_pts.f_opposite_tangents {
                self.add_degenerate_line(quad_pts);
                return true;
            }
        }
        if !quad_pts.f_quad[2].is_finite() {
            return false; // just abort if projected quad isn't representable
        }
        self.f_recursion_depth += 1;
        if self.f_recursion_depth > K_RECURSIVE_LIMITS[usize::from(self.f_found_tangents)] {
            // If we stop making progress, just emit a line and move on
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        let mut half = SkQuadConstruct::default();
        if !half.init_with_start(quad_pts) {
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        if !self.cubic_stroke(cubic, &mut half) {
            return false;
        }
        if !half.init_with_end(quad_pts) {
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        if !self.cubic_stroke(cubic, &mut half) {
            return false;
        }
        self.f_recursion_depth -= 1;
        true
    }

    fn conic_stroke(&mut self, conic: &SkConic, quad_pts: &mut SkQuadConstruct) -> bool {
        let result_type = self.compare_quad_conic(conic, quad_pts);
        if StrokeResultType::Quad == result_type {
            let stroke = quad_pts.f_quad;
            let sink = if self.f_stroke_type == K_OUTER_STROKE_TYPE {
                &mut self.f_outer
            } else {
                &mut self.f_inner
            };
            sink.quad_to(stroke[1], stroke[2]);
            return true;
        }
        if StrokeResultType::Degenerate == result_type {
            self.add_degenerate_line(quad_pts);
            return true;
        }
        if !quad_pts.f_quad[2].is_finite() {
            return false; // just abort if projected quad isn't representable
        }
        self.f_recursion_depth += 1;
        if self.f_recursion_depth > K_RECURSIVE_LIMITS[K_CONIC_RECURSIVE_LIMIT] {
            // If we stop making progress, just emit a line and move on
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        let mut half = SkQuadConstruct::default();
        if !half.init_with_start(quad_pts) {
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        if !self.conic_stroke(conic, &mut half) {
            return false;
        }
        if !half.init_with_end(quad_pts) {
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        if !self.conic_stroke(conic, &mut half) {
            return false;
        }
        self.f_recursion_depth -= 1;
        true
    }

    fn quad_stroke(&mut self, quad: &[Point; 3], quad_pts: &mut SkQuadConstruct) -> bool {
        let result_type = self.compare_quad_quad(quad, quad_pts);
        if StrokeResultType::Quad == result_type {
            let stroke = quad_pts.f_quad;
            let sink = if self.f_stroke_type == K_OUTER_STROKE_TYPE {
                &mut self.f_outer
            } else {
                &mut self.f_inner
            };
            sink.quad_to(stroke[1], stroke[2]);
            return true;
        }
        if StrokeResultType::Degenerate == result_type {
            self.add_degenerate_line(quad_pts);
            return true;
        }
        if !quad_pts.f_quad[2].is_finite() {
            return false; // just abort if projected quad isn't representable
        }
        self.f_recursion_depth += 1;
        if self.f_recursion_depth > K_RECURSIVE_LIMITS[K_QUAD_RECURSIVE_LIMIT] {
            // If we stop making progress, just emit a line and move on
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        let mut half = SkQuadConstruct::default();
        if !half.init_with_start(quad_pts) {
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        if !self.quad_stroke(quad, &mut half) {
            return false;
        }
        if !half.init_with_end(quad_pts) {
            self.add_degenerate_line(quad_pts);
            self.f_recursion_depth -= 1;
            return true;
        }
        if !self.quad_stroke(quad, &mut half) {
            return false;
        }
        self.f_recursion_depth -= 1;
        true
    }

    pub fn quad_to(&mut self, pt1: Point, pt2: Point) {
        let quad = [self.f_prev_pt, pt1, pt2];
        let mut reduction = Point::new(0.0, 0.0);
        let reduction_type = check_quad_linear(&quad, &mut reduction);
        if ReductionType::Point == reduction_type {
            /* If the stroke consists of a moveTo followed by a degenerate curve, treat it
            as if it were followed by a zero-length line. Lines without length
            can have square and round end caps. */
            self.line_to_pt(pt2);
            return;
        }
        if ReductionType::Line == reduction_type {
            self.line_to_pt(pt2);
            return;
        }
        if ReductionType::Degenerate == reduction_type {
            self.line_to_pt(reduction);
            let save_joiner = self.f_joiner;
            self.f_joiner = join_factory(SkPaintJoin::Round);
            self.line_to_pt(pt2);
            self.f_joiner = save_joiner;
            return;
        }
        debug_assert!(ReductionType::Quad == reduction_type);
        let mut normal_ab = Vector::new(0.0, 0.0);
        let mut unit_ab = Vector::new(0.0, 0.0);
        let mut normal_bc = Vector::new(0.0, 0.0);
        let mut unit_bc = Vector::new(0.0, 0.0);
        if !self.pre_join_to(pt1, &mut normal_ab, &mut unit_ab, false) {
            self.line_to_pt(pt2);
            return;
        }
        let mut quad_pts = SkQuadConstruct::default();
        self.init(K_OUTER_STROKE_TYPE, &mut quad_pts, 0.0, 1.0);
        let _ = self.quad_stroke(&quad, &mut quad_pts);
        self.init(K_INNER_STROKE_TYPE, &mut quad_pts, 0.0, 1.0);
        let _ = self.quad_stroke(&quad, &mut quad_pts);
        self.set_quad_end_normal(&quad, &normal_ab, &unit_ab, &mut normal_bc, &mut unit_bc);

        self.post_join_to(pt2, &normal_bc, &unit_bc);
    }

    pub fn conic_to(&mut self, pt1: Point, pt2: Point, weight: Scalar) {
        let conic = SkConic::new(self.f_prev_pt, pt1, pt2, weight);
        let mut reduction = Point::new(0.0, 0.0);
        let reduction_type = check_conic_linear(&conic, &mut reduction);
        if ReductionType::Point == reduction_type {
            /* If the stroke consists of a moveTo followed by a degenerate curve, treat it
            as if it were followed by a zero-length line. Lines without length
            can have square and round end caps. */
            self.line_to_pt(pt2);
            return;
        }
        if ReductionType::Line == reduction_type {
            self.line_to_pt(pt2);
            return;
        }
        if ReductionType::Degenerate == reduction_type {
            self.line_to_pt(reduction);
            let save_joiner = self.f_joiner;
            self.f_joiner = join_factory(SkPaintJoin::Round);
            self.line_to_pt(pt2);
            self.f_joiner = save_joiner;
            return;
        }
        debug_assert!(ReductionType::Quad == reduction_type);
        let mut normal_ab = Vector::new(0.0, 0.0);
        let mut unit_ab = Vector::new(0.0, 0.0);
        let mut normal_bc = Vector::new(0.0, 0.0);
        let mut unit_bc = Vector::new(0.0, 0.0);
        if !self.pre_join_to(pt1, &mut normal_ab, &mut unit_ab, false) {
            self.line_to_pt(pt2);
            return;
        }
        let mut quad_pts = SkQuadConstruct::default();
        self.init(K_OUTER_STROKE_TYPE, &mut quad_pts, 0.0, 1.0);
        let _ = self.conic_stroke(&conic, &mut quad_pts);
        self.init(K_INNER_STROKE_TYPE, &mut quad_pts, 0.0, 1.0);
        let _ = self.conic_stroke(&conic, &mut quad_pts);
        self.set_conic_end_normal(&conic, &normal_ab, &unit_ab, &mut normal_bc, &mut unit_bc);
        self.post_join_to(pt2, &normal_bc, &unit_bc);
    }

    pub fn cubic_to(&mut self, pt1: Point, pt2: Point, pt3: Point) {
        let cubic = [self.f_prev_pt, pt1, pt2, pt3];
        let mut reduction = [Point::new(0.0, 0.0); 3];
        let mut tangent_pt = 0usize;
        let reduction_type = check_cubic_linear(&cubic, &mut reduction, &mut tangent_pt);
        if ReductionType::Point == reduction_type {
            /* If the stroke consists of a moveTo followed by a degenerate curve, treat it
            as if it were followed by a zero-length line. Lines without length
            can have square and round end caps. */
            self.line_to_pt(pt3);
            return;
        }
        if ReductionType::Line == reduction_type {
            self.line_to_pt(pt3);
            return;
        }
        if ReductionType::Degenerate == reduction_type
            || ReductionType::Degenerate2 == reduction_type
            || ReductionType::Degenerate3 == reduction_type
        {
            self.line_to_pt(reduction[0]);
            let save_joiner = self.f_joiner;
            self.f_joiner = join_factory(SkPaintJoin::Round);
            if ReductionType::Degenerate2 == reduction_type
                || ReductionType::Degenerate3 == reduction_type
            {
                self.line_to_pt(reduction[1]);
            }
            if ReductionType::Degenerate3 == reduction_type {
                self.line_to_pt(reduction[2]);
            }
            self.line_to_pt(pt3);
            self.f_joiner = save_joiner;
            return;
        }
        debug_assert!(ReductionType::Quad == reduction_type);
        let mut normal_ab = Vector::new(0.0, 0.0);
        let mut unit_ab = Vector::new(0.0, 0.0);
        let mut normal_cd = Vector::new(0.0, 0.0);
        let mut unit_cd = Vector::new(0.0, 0.0);
        if !self.pre_join_to(cubic[tangent_pt], &mut normal_ab, &mut unit_ab, false) {
            self.line_to_pt(pt3);
            return;
        }
        let mut t_values = [0.0; 2];
        let count = sk_find_cubic_inflections(&cubic, &mut t_values);
        let mut last_t = 0.0;
        for index in 0..=count {
            let next_t = if index < count {
                t_values[index as usize]
            } else {
                1.0
            };
            let mut quad_pts = SkQuadConstruct::default();
            self.init(K_OUTER_STROKE_TYPE, &mut quad_pts, last_t, next_t);
            let _ = self.cubic_stroke(&cubic, &mut quad_pts);
            self.init(K_INNER_STROKE_TYPE, &mut quad_pts, last_t, next_t);
            let _ = self.cubic_stroke(&cubic, &mut quad_pts);
            last_t = next_t;
        }
        let cusp = sk_find_cubic_cusp(&cubic);
        if cusp > 0.0 {
            let mut cusp_loc = Point::new(0.0, 0.0);
            sk_eval_cubic_at(&cubic, cusp, Some(&mut cusp_loc), None, None);
            self.f_cusper
                .add_circle(cusp_loc, self.f_radius, SkPathDirection::Cw);
        }
        // emit the join even if one stroke succeeded but the last one failed
        // this avoids reversing an inner stroke with a partial path followed by another moveto
        self.set_cubic_end_normal(&cubic, &normal_ab, &unit_ab, &mut normal_cd, &mut unit_cd);

        self.post_join_to(pt3, &normal_cd, &unit_cd);
    }
}

/// `has_valid_tangent`.
fn has_valid_tangent(iter: &Iter<'_>) -> bool {
    let mut copy = iter.clone();
    while let Some(rec) = copy.next_rec() {
        let pts = rec.f_points;
        match rec.f_verb {
            SkPathVerb::Move => return false,
            SkPathVerb::Line => {
                if pts[0] == pts[1] {
                    continue;
                }
                return true;
            }
            SkPathVerb::Quad | SkPathVerb::Conic => {
                if pts[0] == pts[1] && pts[0] == pts[2] {
                    continue;
                }
                return true;
            }
            SkPathVerb::Cubic => {
                if pts[0] == pts[1] && pts[0] == pts[2] && pts[0] == pts[3] {
                    continue;
                }
                return true;
            }
            SkPathVerb::Close | SkPathVerb::Done => return false,
        }
    }
    false
}

/// `std::max` semantics.
#[inline]
fn cpp_max(a: Scalar, b: Scalar) -> Scalar {
    if a < b { b } else { a }
}

#[inline]
fn cpp_min(a: Scalar, b: Scalar) -> Scalar {
    if b < a { b } else { a }
}

#[inline]
fn cpp_min3(a: Scalar, b: Scalar, c: Scalar) -> Scalar {
    cpp_min(cpp_min(a, b), c)
}

#[inline]
fn cpp_max3(a: Scalar, b: Scalar, c: Scalar) -> Scalar {
    cpp_max(cpp_max(a, b), c)
}

/// Returns the distance squared from the point to the line.
fn pt_to_line(pt: &Point, line_start: &Point, line_end: &Point) -> Scalar {
    let dxy = *line_end - *line_start;
    let ab0 = *pt - *line_start;
    let numer = dxy.dot(&ab0);
    let denom = dxy.dot(&dxy);
    let t = crate::floating_point::ieee_float_divide(numer, denom);
    if (0.0..=1.0).contains(&t) {
        let hit = *line_start * (1.0 - t) + *line_end * t;
        PointPriv::distance_to_sqd(&hit, pt)
    } else {
        PointPriv::distance_to_sqd(pt, line_start)
    }
}

/// Returns the distance squared from the point to the (tangent) line.
fn pt_to_tangent_line(pt: &Point, line_start: &Point, tangent: &Vector) -> Scalar {
    let dxy = *tangent;
    let ab0 = *pt - *line_start;
    let numer = dxy.dot(&ab0);
    let denom = dxy.dot(&dxy);
    let t = crate::floating_point::ieee_float_divide(numer, denom);
    if (0.0..=1.0).contains(&t) {
        let hit = *line_start + *tangent * t;
        PointPriv::distance_to_sqd(&hit, pt)
    } else {
        PointPriv::distance_to_sqd(pt, line_start)
    }
}

/// Given a cubic, determine if all four points are in a line.
///
/// Return true if the inner points are close to a line connecting the outermost points.
fn cubic_in_line(cubic: &[Point; 4]) -> bool {
    let mut pt_max = -1.0;
    let mut outer1 = 0usize;
    let mut outer2 = 0usize;
    for index in 0..3 {
        for inner in (index + 1)..4 {
            let test_diff = cubic[inner] - cubic[index];
            let test_max = cpp_max(scalar_abs(test_diff.x), scalar_abs(test_diff.y));
            if pt_max < test_max {
                outer1 = index;
                outer2 = inner;
                pt_max = test_max;
            }
        }
    }
    debug_assert!(outer1 <= 2);
    debug_assert!((1..=3).contains(&outer2));
    debug_assert!(outer1 < outer2);
    let mid1 = (1 + (2 >> outer2)) >> outer1;
    debug_assert!(mid1 <= 2);
    let mid2 = outer1 ^ outer2 ^ mid1;
    let line_slop = pt_max * pt_max * 0.00001; // this multiplier is pulled out of the air
    pt_to_line(&cubic[mid1], &cubic[outer1], &cubic[outer2]) <= line_slop
        && pt_to_line(&cubic[mid2], &cubic[outer1], &cubic[outer2]) <= line_slop
}

/// Given a quad, see if all three points are in a line.
fn quad_in_line(quad: &[Point; 3]) -> bool {
    let mut pt_max = -1.0;
    let mut outer1 = 0usize;
    let mut outer2 = 0usize;
    for index in 0..2 {
        for inner in (index + 1)..3 {
            let test_diff = quad[inner] - quad[index];
            let test_max = cpp_max(scalar_abs(test_diff.x), scalar_abs(test_diff.y));
            if pt_max < test_max {
                outer1 = index;
                outer2 = inner;
                pt_max = test_max;
            }
        }
    }
    debug_assert!(outer1 <= 1);
    debug_assert!((1..=2).contains(&outer2));
    debug_assert!(outer1 < outer2);
    let mid = outer1 ^ outer2 ^ 3;
    const K_CURVATURE_SLOP: Scalar = 0.000005; // this multiplier is pulled out of the air
    let line_slop = pt_max * pt_max * K_CURVATURE_SLOP;
    pt_to_line(&quad[mid], &quad[outer1], &quad[outer2]) <= line_slop
}

fn conic_in_line(conic: &SkConic) -> bool {
    quad_in_line(&conic.pts)
}

/// `SkPathStroker::CheckCubicLinear`. `tangent_pt` receives the index into `cubic` used as the
/// initial tangent point (the reference returns a pointer).
fn check_cubic_linear(
    cubic: &[Point; 4],
    reduction: &mut [Point; 3],
    tangent_pt: &mut usize,
) -> ReductionType {
    let degenerate_ab = degenerate_vector(&(cubic[1] - cubic[0]));
    let degenerate_bc = degenerate_vector(&(cubic[2] - cubic[1]));
    let degenerate_cd = degenerate_vector(&(cubic[3] - cubic[2]));
    if degenerate_ab & degenerate_bc & degenerate_cd {
        return ReductionType::Point;
    }
    if (degenerate_ab as i32) + (degenerate_bc as i32) + (degenerate_cd as i32) == 2 {
        return ReductionType::Line;
    }
    if !cubic_in_line(cubic) {
        *tangent_pt = if degenerate_ab { 2 } else { 1 };
        return ReductionType::Quad;
    }
    let mut t_values = [0.0; 3];
    let count = sk_find_cubic_max_curvature(cubic, &mut t_values);
    let mut r_count = 0;
    // Now loop over the t-values, and reject any that evaluate to either end-point
    for index in 0..count.max(0) as usize {
        let t = t_values[index];
        if 0.0 >= t || t >= 1.0 {
            continue;
        }
        let mut pt = Point::new(0.0, 0.0);
        sk_eval_cubic_at(cubic, t, Some(&mut pt), None, None);
        if pt != cubic[0] && pt != cubic[3] {
            reduction[r_count] = pt;
            r_count += 1;
        }
    }
    if r_count == 0 {
        return ReductionType::Line;
    }
    match r_count {
        1 => ReductionType::Degenerate,
        2 => ReductionType::Degenerate2,
        _ => ReductionType::Degenerate3,
    }
}

/// `SkPathStroker::CheckConicLinear`.
fn check_conic_linear(conic: &SkConic, reduction: &mut Point) -> ReductionType {
    let degenerate_ab = degenerate_vector(&(conic.pts[1] - conic.pts[0]));
    let degenerate_bc = degenerate_vector(&(conic.pts[2] - conic.pts[1]));
    if degenerate_ab & degenerate_bc {
        return ReductionType::Point;
    }
    if degenerate_ab | degenerate_bc {
        return ReductionType::Line;
    }
    if !conic_in_line(conic) {
        return ReductionType::Quad;
    }
    // SkFindConicMaxCurvature would be a better solution, once we know how to
    // implement it. Quad curvature is a reasonable substitute
    let t = sk_find_quad_max_curvature(&conic.pts);
    if 0.0 == t || t.is_nan() {
        return ReductionType::Line;
    }
    conic.eval_at_with(t, Some(reduction), None);
    ReductionType::Degenerate
}

/// `SkPathStroker::CheckQuadLinear`.
fn check_quad_linear(quad: &[Point; 3], reduction: &mut Point) -> ReductionType {
    let degenerate_ab = degenerate_vector(&(quad[1] - quad[0]));
    let degenerate_bc = degenerate_vector(&(quad[2] - quad[1]));
    if degenerate_ab & degenerate_bc {
        return ReductionType::Point;
    }
    if degenerate_ab | degenerate_bc {
        return ReductionType::Line;
    }
    if !quad_in_line(quad) {
        return ReductionType::Quad;
    }
    let t = sk_find_quad_max_curvature(quad);
    if 0.0 == t || 1.0 == t {
        return ReductionType::Line;
    }
    *reduction = sk_eval_quad_at(quad, t);
    ReductionType::Degenerate
}

/// Intersect the line with the quad and return the t values on the quad where the line crosses.
fn intersect_quad_ray(line: &[Point; 2], quad: &[Point; 3], roots: &mut [Scalar; 2]) -> i32 {
    let vec = line[1] - line[0];
    let mut r = [0.0; 3];
    for n in 0..3 {
        r[n] = vec.cross(&(quad[n] - line[0]));
    }
    let mut a = r[2];
    let mut b = r[1];
    let c = r[0];
    a += c - 2.0 * b; // A = a - 2*b + c
    b -= c; // B = -(b - c)
    sk_find_unit_quad_roots(a, 2.0 * b, c, roots)
}

fn points_within_dist(near_pt: &Point, far_pt: &Point, limit: Scalar) -> bool {
    PointPriv::distance_to_sqd(near_pt, far_pt) <= limit * limit
}

fn sharp_angle(quad: &[Point; 3]) -> bool {
    let mut smaller = quad[1] - quad[0];
    let mut larger = quad[1] - quad[2];
    let smaller_len = PointPriv::length_sqd(&smaller);
    let mut larger_len = PointPriv::length_sqd(&larger);
    if smaller_len > larger_len {
        std::mem::swap(&mut smaller, &mut larger);
        larger_len = smaller_len;
    }
    if !smaller.set_length(larger_len) {
        return false;
    }
    let dot = smaller.dot(&larger);
    dot > 0.0
}

/// `SkStroke`.
pub struct SkStroke {
    f_width: Scalar,
    f_miter_limit: Scalar,
    f_res_scale: Scalar,
    f_cap: SkPaintCap,
    f_join: SkPaintJoin,
    f_do_fill: bool,
}

impl Default for SkStroke {
    fn default() -> Self {
        Self::new()
    }
}

impl SkStroke {
    pub fn new() -> Self {
        SkStroke {
            f_width: SCALAR_1,
            f_miter_limit: SK_PAINT_DEFAULTS_MITER_LIMIT,
            f_res_scale: 1.0,
            f_cap: SK_PAINT_DEFAULT_CAP,
            f_join: SK_PAINT_DEFAULT_JOIN,
            f_do_fill: false,
        }
    }

    /// `SkStroke(const SkPaint&)` without `SkPaint`.
    pub fn from_params(
        stroke_width: Scalar,
        stroke_miter: Scalar,
        stroke_cap: SkPaintCap,
        stroke_join: SkPaintJoin,
        do_fill: bool,
    ) -> Self {
        SkStroke {
            f_width: stroke_width,
            f_miter_limit: stroke_miter,
            f_res_scale: 1.0,
            f_cap: stroke_cap,
            f_join: stroke_join,
            f_do_fill: do_fill,
        }
    }

    /// `SkStroke(const SkPaint&, SkScalar width)` without `SkPaint`.
    pub fn from_params_and_width(
        width: Scalar,
        stroke_miter: Scalar,
        stroke_cap: SkPaintCap,
        stroke_join: SkPaintJoin,
        do_fill: bool,
    ) -> Self {
        SkStroke {
            f_width: width,
            f_miter_limit: stroke_miter,
            f_res_scale: 1.0,
            f_cap: stroke_cap,
            f_join: stroke_join,
            f_do_fill: do_fill,
        }
    }

    #[inline]
    pub fn get_cap(&self) -> SkPaintCap {
        self.f_cap
    }

    pub fn set_cap(&mut self, cap: SkPaintCap) {
        debug_assert!((cap as usize) < crate::sk_paint_types::SK_PAINT_CAP_COUNT);
        self.f_cap = cap;
    }

    #[inline]
    pub fn get_join(&self) -> SkPaintJoin {
        self.f_join
    }

    pub fn set_join(&mut self, join: SkPaintJoin) {
        debug_assert!((join as usize) < crate::sk_paint_types::SK_PAINT_JOIN_COUNT);
        self.f_join = join;
    }

    pub fn set_miter_limit(&mut self, miter_limit: Scalar) {
        debug_assert!(miter_limit >= 0.0);
        self.f_miter_limit = miter_limit;
    }

    pub fn set_width(&mut self, width: Scalar) {
        debug_assert!(width >= 0.0);
        self.f_width = width;
    }

    #[inline]
    pub fn get_do_fill(&self) -> bool {
        self.f_do_fill
    }

    pub fn set_do_fill(&mut self, do_fill: bool) {
        self.f_do_fill = do_fill;
    }

    #[inline]
    pub fn get_res_scale(&self) -> Scalar {
        self.f_res_scale
    }

    pub fn set_res_scale(&mut self, rs: Scalar) {
        debug_assert!(rs > 0.0 && rs.is_finite());
        self.f_res_scale = rs;
    }

    /// Stroke the specified path.
    pub fn stroke_path(&self, src: &SkPath, dst: &mut SkPathBuilder) {
        let radius = self.f_width / 2.0;

        if radius <= 0.0 {
            return;
        }

        let raw = match sk_path_priv::SkPathPriv::raw(src, SkResolveConvexity::No) {
            Some(raw) => raw,
            None => return,
        };

        // If src is really a rect, call our specialty stroke_rect() method
        {
            let mut rect = Rect::new(0.0, 0.0, 0.0, 0.0);
            let mut is_closed = false;
            let mut dir = SkPathDirection::Cw;
            if src.is_rect(Some(&mut rect), Some(&mut is_closed), Some(&mut dir)) && is_closed {
                self.stroke_rect(&rect, dst, dir);
                // our answer should preserve the inverseness of the src
                if src.is_inverse_fill_type() {
                    debug_assert!(!dst.is_inverse_fill_type());
                    dst.toggle_inverse_fill_type();
                }
                return;
            }
        }

        // We can always ignore centers for stroke and fill convex line-only paths
        // TODO: remove the line-only restriction
        let ignore_center = self.f_do_fill
            && (src.get_segment_masks() == K_LINE_SK_PATH_SEGMENT_MASK as u32)
            && src.is_last_contour_closed()
            && src.is_convex();

        let mut stroker = SkPathStroker::new(
            src,
            radius,
            self.f_miter_limit,
            self.get_cap(),
            self.get_join(),
            self.f_res_scale,
            ignore_center,
        );

        let mut iter = Iter::from_path(src, false);
        let mut last_segment = SkPathVerb::Move;
        while let Some(rec) = iter.next_rec() {
            let mut pts = [Point::new(0.0, 0.0); 4];
            let n = rec.f_points.len();
            pts[..n].copy_from_slice(rec.f_points);
            let verb = rec.f_verb;
            let conic_weight = rec.f_conic_weight;
            match verb {
                SkPathVerb::Move => {
                    stroker.move_to(pts[0]);
                }
                SkPathVerb::Line => {
                    stroker.line_to_iter(pts[1], Some(&iter));
                    last_segment = SkPathVerb::Line;
                }
                SkPathVerb::Quad => {
                    stroker.quad_to(pts[1], pts[2]);
                    last_segment = SkPathVerb::Quad;
                }
                SkPathVerb::Conic => {
                    stroker.conic_to(pts[1], pts[2], conic_weight);
                    last_segment = SkPathVerb::Conic;
                }
                SkPathVerb::Cubic => {
                    stroker.cubic_to(pts[1], pts[2], pts[3]);
                    last_segment = SkPathVerb::Cubic;
                }
                SkPathVerb::Close => {
                    if SkPaintCap::Butt != self.get_cap() {
                        /* If the stroke consists of a moveTo followed by a close, treat it
                        as if it were followed by a zero-length line. Lines without length
                        can have square and round end caps. */
                        if stroker.has_only_move_to() {
                            let pt = stroker.move_to_pt();
                            stroker.line_to_pt(pt);
                            last_segment = SkPathVerb::Line;
                        } else if stroker.is_current_contour_empty() {
                            /* If the stroke consists of a moveTo followed by one or more
                            zero-length verbs, then followed by a close, treat is as if it
                            were followed by a zero-length line. Lines without length can
                            have square & round end caps. */
                            last_segment = SkPathVerb::Line;
                        } else {
                            stroker.close(last_segment == SkPathVerb::Line);
                        }
                    } else {
                        stroker.close(last_segment == SkPathVerb::Line);
                    }
                }
                SkPathVerb::Done => {}
            }
        }
        stroker.done(dst, last_segment == SkPathVerb::Line);

        if self.f_do_fill && !ignore_center {
            let d = sk_path_priv::SkPathPriv::compute_first_direction(&raw);
            if d == SkPathFirstDirection::Ccw {
                dst.private_reverse_add_path(src);
            } else {
                dst.add_path(src, AddPathMode::Append);
            }
        }

        // our answer should preserve the inverseness of the src
        if src.is_inverse_fill_type() {
            debug_assert!(!dst.is_inverse_fill_type());
            dst.toggle_inverse_fill_type();
        }
    }

    /// Stroke the specified rect, winding it in the specified direction.
    pub fn stroke_rect(&self, orig_rect: &Rect, dst: &mut SkPathBuilder, dir: SkPathDirection) {
        dst.reset();

        let radius = self.f_width / 2.0;
        if radius <= 0.0 {
            return;
        }

        let mut rw = orig_rect.width();
        let mut rh = orig_rect.height();
        let mut dir = dir;
        if (rw < 0.0) ^ (rh < 0.0) {
            dir = reverse_direction(dir);
        }
        let mut rect = *orig_rect;
        rect.sort();
        // reassign these, now that we know they'll be >= 0
        rw = rect.width();
        rh = rect.height();

        let mut r = rect;
        r.outset(radius, radius);

        let mut join = self.f_join;
        if SkPaintJoin::Miter == join && self.f_miter_limit < SCALAR_SQRT2 {
            join = SkPaintJoin::Bevel;
        }

        match join {
            SkPaintJoin::Miter => {
                dst.add_rect(&r, dir, 0);
            }
            SkPaintJoin::Bevel => {
                add_bevel(dst, &rect, &r, dir);
            }
            SkPaintJoin::Round => {
                // `addRRect(rrect, dir)` uses the legacy start indices 6 (CW) and 7 (CCW).
                dst.add_rrect(
                    &RRect::make_rect_xy(&r, radius, radius),
                    dir,
                    if dir == SkPathDirection::Cw { 6 } else { 7 },
                );
            }
        }

        if self.f_width < cpp_min(rw, rh) && !self.f_do_fill {
            r = rect;
            r.inset(radius, radius);
            dst.add_rect(&r, reverse_direction(dir), 0);
        }
    }
}

fn reverse_direction(dir: SkPathDirection) -> SkPathDirection {
    match dir {
        SkPathDirection::Ccw => SkPathDirection::Cw,
        SkPathDirection::Cw => SkPathDirection::Ccw,
    }
}

fn add_bevel(path: &mut SkPathBuilder, r: &Rect, outer: &Rect, dir: SkPathDirection) {
    let mut pts = [Point::new(0.0, 0.0); 8];

    if SkPathDirection::Cw == dir {
        pts[0].set(r.left(), outer.top());
        pts[1].set(r.right(), outer.top());
        pts[2].set(outer.right(), r.top());
        pts[3].set(outer.right(), r.bottom());
        pts[4].set(r.right(), outer.bottom());
        pts[5].set(r.left(), outer.bottom());
        pts[6].set(outer.left(), r.bottom());
        pts[7].set(outer.left(), r.top());
    } else {
        pts[7].set(r.left(), outer.top());
        pts[6].set(r.right(), outer.top());
        pts[5].set(outer.right(), r.top());
        pts[4].set(outer.right(), r.bottom());
        pts[3].set(r.right(), outer.bottom());
        pts[2].set(r.left(), outer.bottom());
        pts[1].set(outer.left(), r.bottom());
        pts[0].set(outer.left(), r.top());
    }
    path.add_polygon(&pts, true);
}
