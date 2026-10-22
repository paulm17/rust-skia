//! Port of `include/core/SkPathBuilder.h`, `src/core/SkPathBuilder.cpp`,
//! `src/core/SkPath_interpolate.cpp` (builder-independent) and `src/core/SkPathDump.cpp`.

#![allow(
    clippy::too_many_arguments,
    non_camel_case_types,
    clippy::manual_range_contains,
    clippy::needless_return,
    clippy::let_and_return,
    clippy::single_match,
    clippy::redundant_clone,
    clippy::unnecessary_cast,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::excessive_precision
)]

use std::sync::Arc;

use crate::floating_point::is_finite;
use crate::matrix::Matrix;
use crate::point::Point;
use crate::rect::Rect;
use crate::rrect::RRect;
use crate::scalar::{
    SCALAR_HALF, SCALAR_PI, Scalar, degrees_to_radians, scalar_abs, scalar_atan2,
    scalar_ceil_to_int, scalar_copy_sign, scalar_cos, scalar_cos_snap_to_zero,
    scalar_floor_to_scalar, scalar_mod, scalar_nearly_equal, scalar_nearly_zero,
    scalar_round_to_scalar, scalar_sin, scalar_sin_snap_to_zero, scalar_sqrt, scalar_tan,
};
use crate::sk_geometry::{SkConic, sk_chop_cubic_at_half};
use crate::sk_path::{RangeIter, SkPath};
use crate::sk_path_data::{SkPathData, SkPathIsAData, SkPathIsAType};
use crate::sk_path_iter::{SkPathIter, SkPathIterRec};
use crate::sk_path_priv as priv_;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_raw_shapes::SkPathRawShapes;
use crate::sk_path_types::{
    K_CONIC_SK_PATH_SEGMENT_MASK, K_CUBIC_SK_PATH_SEGMENT_MASK, K_LINE_SK_PATH_SEGMENT_MASK,
    K_QUAD_SK_PATH_SEGMENT_MASK, SkPathConvexity, SkPathDirection, SkPathFillType, SkPathVerb,
    SkResolveConvexity, sk_path_convexity_is_convex, sk_path_direction_to_convexity,
};
use crate::string::{SkScalarAsStringType, SkString, sk_append_scalar};

/// `SkPathBuilder::Reserve`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reserve {
    Exact,
    Grow,
}

/// `SkPathBuilder::ArcSize`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArcSize {
    Small,
    Large,
}

/// `SkPathBuilder::DumpFormat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpFormat {
    Decimal,
    Hex,
}

/// Port of `class SkPathBuilder`.
#[derive(Clone)]
pub struct SkPathBuilder {
    f_pts: Vec<Point>,
    f_verbs: Vec<SkPathVerb>,
    f_conic_weights: Vec<f32>,
    f_fill_type: SkPathFillType,
    f_is_volatile: bool,
    f_convexity: SkPathConvexity,
    f_segment_mask: u32,
    f_last_move_index: i32,
    f_type: SkPathIsAType,
    f_is_a: SkPathIsAData,
}

impl PartialEq for SkPathBuilder {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
            || (self.f_segment_mask == other.f_segment_mask
                && self.f_fill_type == other.f_fill_type
                && self.f_verbs == other.f_verbs
                && self.f_pts == other.f_pts
                && self.f_conic_weights == other.f_conic_weights)
    }
}
impl Eq for SkPathBuilder {}

impl Default for SkPathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SkPathBuilder {
    pub fn new() -> Self {
        let mut b = SkPathBuilder {
            f_pts: Vec::new(),
            f_verbs: Vec::new(),
            f_conic_weights: Vec::new(),
            f_fill_type: SkPathFillType::Winding,
            f_is_volatile: false,
            f_convexity: SkPathConvexity::Unknown,
            f_segment_mask: 0,
            f_last_move_index: -1,
            f_type: SkPathIsAType::General,
            f_is_a: SkPathIsAData::default(),
        };
        b.reset();
        b
    }

    pub fn new_with_fill_type(ft: SkPathFillType) -> Self {
        let mut b = Self::new();
        b.f_fill_type = ft;
        b
    }

    pub fn from_path(src: &SkPath) -> Self {
        let mut b = Self::new();
        b.f_fill_type = src.get_fill_type();
        b.f_is_volatile = src.is_volatile();
        if src.is_empty() {
            return b;
        }
        if let Some(raw) = src.raw(SkResolveConvexity::Yes) {
            b.add_raw(&raw, Reserve::Exact);
        }
        b.f_last_move_index = priv_::find_last_move_to_index(&b.f_verbs, b.f_pts.len());
        if let Some(info) = src.oval_info() {
            b.f_type = SkPathIsAType::Oval;
            b.f_is_a = SkPathIsAData {
                f_start_index: info.f_start_index,
                f_direction: info.f_direction,
            };
        } else if let Some(info) = src.rrect_info() {
            b.f_type = SkPathIsAType::RRect;
            b.f_is_a = SkPathIsAData {
                f_start_index: info.f_start_index,
                f_direction: info.f_direction,
            };
        }
        b
    }

    pub fn reset(&mut self) -> &mut Self {
        self.f_pts.clear();
        self.f_verbs.clear();
        self.f_conic_weights.clear();
        self.f_fill_type = SkPathFillType::Winding;
        self.f_is_volatile = false;
        self.f_segment_mask = 0;
        self.f_last_move_index = -1;
        self.f_type = SkPathIsAType::General;
        self.f_convexity = SkPathConvexity::Unknown;
        self
    }

    pub fn set_fill_type(&mut self, ft: SkPathFillType) -> &mut Self {
        self.f_fill_type = ft;
        self
    }

    pub fn get_fill_type(&self) -> SkPathFillType {
        self.f_fill_type
    }

    pub fn set_is_volatile(&mut self, v: bool) -> &mut Self {
        self.f_is_volatile = v;
        self
    }

    pub fn is_volatile(&self) -> bool {
        self.f_is_volatile
    }

    pub fn toggle_inverse_fill_type(&mut self) -> &mut Self {
        self.f_fill_type = self.f_fill_type.toggle_inverse();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.f_verbs.is_empty()
    }

    pub fn is_finite(&self) -> bool {
        self.f_pts.iter().all(|p| p.is_finite())
    }

    pub fn count_points(&self) -> i32 {
        self.f_pts.len() as i32
    }

    pub fn is_inverse_fill_type(&self) -> bool {
        self.f_fill_type.is_inverse()
    }

    pub fn points(&self) -> &[Point] {
        &self.f_pts
    }

    pub fn verbs(&self) -> &[SkPathVerb] {
        &self.f_verbs
    }

    pub fn conic_weights(&self) -> &[f32] {
        &self.f_conic_weights
    }

    pub fn get_last_pt(&self) -> Option<Point> {
        self.f_pts.last().copied()
    }

    pub fn iter(&self) -> SkPathIter<'_> {
        SkPathIter::new(&self.f_pts, &self.f_verbs, &self.f_conic_weights)
    }

    // ---- private helpers ----

    fn ensure_move(&mut self) {
        self.f_type = SkPathIsAType::General;
        if self.f_verbs.is_empty() {
            self.move_to(Point::new(0.0, 0.0));
        } else if *self.f_verbs.last().unwrap() == SkPathVerb::Close {
            let p = self.f_pts[self.f_last_move_index as usize];
            self.move_to(p);
        }
    }

    fn is_empty_or_moves(&self) -> bool {
        self.f_verbs.is_empty() || (self.f_verbs.len() == 1 && self.f_verbs[0] == SkPathVerb::Move)
    }

    fn grow_for_verbs_in_path(&mut self, path: &SkPath) {
        self.f_segment_mask |= path.get_segment_masks();
        let verbs = path.verbs();
        if !verbs.is_empty() {
            self.f_type = SkPathIsAType::General;
            self.f_verbs.extend_from_slice(verbs);
        }
        self.f_pts.extend_from_slice(path.points());
        self.f_conic_weights.extend_from_slice(path.conic_weights());
    }

    // ---- construction verbs ----

    pub fn move_to(&mut self, pt: Point) -> &mut Self {
        if !self.f_verbs.is_empty() && *self.f_verbs.last().unwrap() == SkPathVerb::Move {
            *self.f_pts.last_mut().unwrap() = pt;
        } else {
            self.f_last_move_index = self.f_pts.len() as i32;
            self.f_pts.push(pt);
            self.f_verbs.push(SkPathVerb::Move);
            if self.f_type == SkPathIsAType::Oval || self.f_type == SkPathIsAType::RRect {
                self.f_type = SkPathIsAType::General;
            }
            self.f_convexity = SkPathConvexity::Unknown;
        }
        self
    }

    pub fn move_to_xy(&mut self, x: Scalar, y: Scalar) -> &mut Self {
        self.move_to(Point::new(x, y))
    }

    pub fn line_to(&mut self, pt: Point) -> &mut Self {
        self.ensure_move();
        self.f_pts.push(pt);
        self.f_verbs.push(SkPathVerb::Line);
        self.f_segment_mask |= K_LINE_SK_PATH_SEGMENT_MASK as u32;
        self
    }

    pub fn line_to_xy(&mut self, x: Scalar, y: Scalar) -> &mut Self {
        self.line_to(Point::new(x, y))
    }

    pub fn quad_to(&mut self, pt1: Point, pt2: Point) -> &mut Self {
        self.ensure_move();
        self.f_pts.push(pt1);
        self.f_pts.push(pt2);
        self.f_verbs.push(SkPathVerb::Quad);
        self.f_segment_mask |= K_QUAD_SK_PATH_SEGMENT_MASK as u32;
        self
    }

    pub fn quad_to_xy(&mut self, x1: Scalar, y1: Scalar, x2: Scalar, y2: Scalar) -> &mut Self {
        self.quad_to(Point::new(x1, y1), Point::new(x2, y2))
    }

    pub fn conic_to(&mut self, pt1: Point, pt2: Point, w: Scalar) -> &mut Self {
        self.ensure_move();
        if w <= 0.0 {
            return self.line_to(pt2);
        }
        self.f_pts.push(pt1);
        self.f_pts.push(pt2);
        if w == 1.0 {
            self.f_verbs.push(SkPathVerb::Quad);
            self.f_segment_mask |= K_QUAD_SK_PATH_SEGMENT_MASK as u32;
        } else if is_finite(w) {
            self.f_verbs.push(SkPathVerb::Conic);
            self.f_conic_weights.push(w);
            self.f_segment_mask |= K_CONIC_SK_PATH_SEGMENT_MASK as u32;
        } else {
            self.f_verbs.push(SkPathVerb::Line);
            self.f_verbs.push(SkPathVerb::Line);
            self.f_segment_mask |= K_LINE_SK_PATH_SEGMENT_MASK as u32;
        }
        self
    }

    pub fn conic_to_xy(
        &mut self,
        x1: Scalar,
        y1: Scalar,
        x2: Scalar,
        y2: Scalar,
        w: Scalar,
    ) -> &mut Self {
        self.conic_to(Point::new(x1, y1), Point::new(x2, y2), w)
    }

    pub fn cubic_to(&mut self, pt1: Point, pt2: Point, pt3: Point) -> &mut Self {
        self.ensure_move();
        self.f_pts.push(pt1);
        self.f_pts.push(pt2);
        self.f_pts.push(pt3);
        self.f_verbs.push(SkPathVerb::Cubic);
        self.f_segment_mask |= K_CUBIC_SK_PATH_SEGMENT_MASK as u32;
        self
    }

    pub fn cubic_to_xy(
        &mut self,
        x1: Scalar,
        y1: Scalar,
        x2: Scalar,
        y2: Scalar,
        x3: Scalar,
        y3: Scalar,
    ) -> &mut Self {
        self.cubic_to(Point::new(x1, y1), Point::new(x2, y2), Point::new(x3, y3))
    }

    pub fn close(&mut self) -> &mut Self {
        if !self.f_verbs.is_empty() && *self.f_verbs.last().unwrap() != SkPathVerb::Close {
            self.ensure_move();
            self.f_verbs.push(SkPathVerb::Close);
        }
        self
    }

    pub fn polyline_to(&mut self, pts: &[Point]) -> &mut Self {
        if !pts.is_empty() {
            self.ensure_move();
            self.f_pts.extend_from_slice(pts);
            for _ in 0..pts.len() {
                self.f_verbs.push(SkPathVerb::Line);
            }
            self.f_segment_mask |= K_LINE_SK_PATH_SEGMENT_MASK as u32;
        }
        self
    }

    // ---- relative verbs ----

    pub fn r_move_to(&mut self, pt: Point) -> &mut Self {
        let last_pt = if !self.f_pts.is_empty() {
            if *self.f_verbs.last().unwrap() == SkPathVerb::Close {
                self.f_pts[self.f_last_move_index as usize]
            } else {
                *self.f_pts.last().unwrap()
            }
        } else {
            Point::new(0.0, 0.0)
        };
        self.move_to(last_pt + pt)
    }

    pub fn r_move_to_xy(&mut self, dx: Scalar, dy: Scalar) -> &mut Self {
        self.r_move_to(Point::new(dx, dy))
    }

    pub fn r_line_to(&mut self, p1: Point) -> &mut Self {
        self.ensure_move();
        let last = *self.f_pts.last().unwrap();
        self.line_to(last + p1)
    }

    pub fn r_line_to_xy(&mut self, dx: Scalar, dy: Scalar) -> &mut Self {
        self.r_line_to(Point::new(dx, dy))
    }

    pub fn r_quad_to(&mut self, p1: Point, p2: Point) -> &mut Self {
        self.ensure_move();
        let base = *self.f_pts.last().unwrap();
        self.quad_to(base + p1, base + p2)
    }

    pub fn r_quad_to_xy(
        &mut self,
        dx1: Scalar,
        dy1: Scalar,
        dx2: Scalar,
        dy2: Scalar,
    ) -> &mut Self {
        self.r_quad_to(Point::new(dx1, dy1), Point::new(dx2, dy2))
    }

    pub fn r_conic_to(&mut self, p1: Point, p2: Point, w: Scalar) -> &mut Self {
        self.ensure_move();
        let base = *self.f_pts.last().unwrap();
        self.conic_to(base + p1, base + p2, w)
    }

    pub fn r_conic_to_xy(
        &mut self,
        dx1: Scalar,
        dy1: Scalar,
        dx2: Scalar,
        dy2: Scalar,
        w: Scalar,
    ) -> &mut Self {
        self.r_conic_to(Point::new(dx1, dy1), Point::new(dx2, dy2), w)
    }

    pub fn r_cubic_to(&mut self, p1: Point, p2: Point, p3: Point) -> &mut Self {
        self.ensure_move();
        let base = *self.f_pts.last().unwrap();
        self.cubic_to(base + p1, base + p2, base + p3)
    }

    pub fn r_cubic_to_xy(
        &mut self,
        dx1: Scalar,
        dy1: Scalar,
        dx2: Scalar,
        dy2: Scalar,
        dx3: Scalar,
        dy3: Scalar,
    ) -> &mut Self {
        self.r_cubic_to(
            Point::new(dx1, dy1),
            Point::new(dx2, dy2),
            Point::new(dx3, dy3),
        )
    }

    pub fn r_arc_to(
        &mut self,
        r: Point,
        x_axis_rotate: Scalar,
        large_arc: ArcSize,
        sweep: SkPathDirection,
        dxdy: Point,
    ) -> &mut Self {
        let current = self.get_last_pt().unwrap_or(Point::new(0.0, 0.0));
        self.arc_to_svg(r, x_axis_rotate, large_arc, sweep, current + dxdy)
    }

    // ---- arcs ----

    pub fn arc_to(
        &mut self,
        oval: &Rect,
        start_angle: Scalar,
        sweep_angle: Scalar,
        force_move_to: bool,
    ) -> &mut Self {
        if oval.width() < 0.0 || oval.height() < 0.0 {
            return self;
        }
        let start_angle = scalar_mod(start_angle, 360.0);
        let mut force_move_to = force_move_to;
        if self.f_verbs.is_empty() {
            force_move_to = true;
        }

        let mut lone_pt = Point::new(0.0, 0.0);
        if arc_is_lone_point(oval, start_angle, sweep_angle, &mut lone_pt) {
            return if force_move_to {
                self.move_to(lone_pt)
            } else {
                self.line_to(lone_pt)
            };
        }

        let mut start_v = Point::new(0.0, 0.0);
        let mut stop_v = Point::new(0.0, 0.0);
        let dir = angles_to_unit_vectors(start_angle, sweep_angle, &mut start_v, &mut stop_v);

        if start_v == stop_v {
            let end_angle = degrees_to_radians(start_angle + sweep_angle);
            let radius_x = oval.width() / 2.0;
            let radius_y = oval.height() / 2.0;
            let single_pt = Point::new(
                oval.center_x() + radius_x * scalar_cos(end_angle),
                oval.center_y() + radius_y * scalar_sin(end_angle),
            );
            return add_pt(self, force_move_to, single_pt);
        }

        let mut conics = [SkConic::default(); SkConic::K_MAX_CONICS_FOR_ARC];
        let mut single_pt = Point::new(0.0, 0.0);
        let count = build_arc_conics(oval, &start_v, &stop_v, dir, &mut conics, &mut single_pt);
        if count > 0 {
            self.inc_reserve((count * 2 + 1) as usize, (count * 2 + 1) as usize, 0);
            let pt = conics[0].pts[0];
            add_pt(self, force_move_to, pt);
            for i in 0..count as usize {
                self.conic_to(conics[i].pts[1], conics[i].pts[2], conics[i].w);
            }
        } else {
            add_pt(self, force_move_to, single_pt);
        }
        self
    }

    pub fn arc_to_pts(&mut self, p1: Point, p2: Point, radius: Scalar) -> &mut Self {
        self.ensure_move();
        if radius == 0.0 {
            return self.line_to(p1);
        }
        let start = *self.f_pts.last().unwrap();

        let befored = normalize_d2([(p1.x - start.x) as f64, (p1.y - start.y) as f64]);
        let afterd = normalize_d2([(p2.x - p1.x) as f64, (p2.y - p1.y) as f64]);
        let cosh = befored[0] * afterd[0] + befored[1] * afterd[1];
        let sinh = befored[0] * afterd[1] - befored[1] * afterd[0];

        if !is_finite2_f64(befored[0], befored[1])
            || !is_finite2_f64(afterd[0], afterd[1])
            || scalar_nearly_zero(sinh as Scalar)
        {
            return self.line_to(p1);
        }

        let dist = scalar_abs(((radius as f64) * (1.0 - cosh) / sinh) as Scalar);
        let xx = p1.x - dist * befored[0] as Scalar;
        let yy = p1.y - dist * befored[1] as Scalar;

        let mut after = Point::new(afterd[0] as Scalar, afterd[1] as Scalar);
        after.set_length(dist);
        self.line_to(Point::new(xx, yy));
        let weight = scalar_sqrt((SCALAR_HALF as f64 + cosh * 0.5) as Scalar);
        self.conic_to(p1, p1 + after, weight)
    }

    pub fn arc_to_svg(
        &mut self,
        rad: Point,
        angle: Scalar,
        arc_large: ArcSize,
        arc_sweep: SkPathDirection,
        end_pt: Point,
    ) -> &mut Self {
        self.ensure_move();

        let src_pts = [*self.f_pts.last().unwrap(), end_pt];

        if rad.x == 0.0 || rad.y == 0.0 {
            return self.line_to(end_pt);
        }
        if src_pts[0] == src_pts[1] {
            return self.line_to(end_pt);
        }
        let mut rx = scalar_abs(rad.x);
        let mut ry = scalar_abs(rad.y);
        let mut mid_point_distance = src_pts[0] - src_pts[1];
        mid_point_distance *= 0.5;

        let mut point_transform = Matrix::new();
        point_transform.set_rotate(-angle);

        let transformed_mid_point = point_transform.map_point(mid_point_distance);
        let square_rx = rx * rx;
        let square_ry = ry * ry;
        let square_x = transformed_mid_point.x * transformed_mid_point.x;
        let square_y = transformed_mid_point.y * transformed_mid_point.y;

        let mut radii_scale = square_x / square_rx + square_y / square_ry;
        if radii_scale > 1.0 {
            radii_scale = scalar_sqrt(radii_scale);
            rx *= radii_scale;
            ry *= radii_scale;
        }

        point_transform.set_scale(1.0 / rx, 1.0 / ry);
        point_transform.pre_rotate(-angle);

        let mut unit_pts = [Point::new(0.0, 0.0); 2];
        point_transform.map_points(&mut unit_pts, &src_pts);
        let mut delta = unit_pts[1] - unit_pts[0];

        let d = delta.x * delta.x + delta.y * delta.y;
        let scale_factor_squared = (1.0 / d - 0.25).max(0.0);

        let mut scale_factor = scalar_sqrt(scale_factor_squared);
        if (arc_sweep == SkPathDirection::Ccw) != (arc_large == ArcSize::Large) {
            scale_factor = -scale_factor;
        }
        delta *= scale_factor;
        let mut center_point = unit_pts[0] + unit_pts[1];
        center_point *= 0.5;
        center_point = Point::new(center_point.x - delta.y, center_point.y + delta.x);
        unit_pts[0] -= center_point;
        unit_pts[1] -= center_point;
        let theta1 = scalar_atan2(unit_pts[0].y, unit_pts[0].x);
        let theta2 = scalar_atan2(unit_pts[1].y, unit_pts[1].x);
        let mut theta_arc = theta2 - theta1;
        if theta_arc < 0.0 && arc_sweep == SkPathDirection::Cw {
            theta_arc += SCALAR_PI * 2.0;
        } else if theta_arc > 0.0 && arc_sweep != SkPathDirection::Cw {
            theta_arc -= SCALAR_PI * 2.0;
        }

        if scalar_abs(theta_arc) < (SCALAR_PI / (1000.0 * 1000.0)) {
            return self.line_to(end_pt);
        }

        point_transform.set_rotate(angle);
        point_transform.pre_scale(rx, ry);

        let segments = scalar_ceil_to_int(scalar_abs(theta_arc / (2.0 * SCALAR_PI / 3.0)));
        let theta_width = theta_arc / segments as Scalar;
        let t = scalar_tan(0.5 * theta_width);
        if !is_finite(t) {
            return self;
        }
        let mut start_theta = theta1;
        let w = scalar_sqrt(SCALAR_HALF + scalar_cos(theta_width) * SCALAR_HALF);
        let scalar_is_integer = |s: Scalar| s == scalar_floor_to_scalar(s);
        let expect_integers = scalar_nearly_zero(SCALAR_PI / 2.0 - scalar_abs(theta_width))
            && scalar_is_integer(rx)
            && scalar_is_integer(ry)
            && scalar_is_integer(end_pt.x)
            && scalar_is_integer(end_pt.y);

        for _ in 0..segments {
            let end_theta = start_theta + theta_width;
            let sin_end_theta = scalar_sin_snap_to_zero(end_theta);
            let cos_end_theta = scalar_cos_snap_to_zero(end_theta);

            unit_pts[1] = Point::new(cos_end_theta, sin_end_theta);
            unit_pts[1] += center_point;
            unit_pts[0] = unit_pts[1];
            unit_pts[0] = Point::new(
                unit_pts[0].x + t * sin_end_theta,
                unit_pts[0].y - t * cos_end_theta,
            );
            let mut mapped = [Point::new(0.0, 0.0); 2];
            point_transform.map_points(&mut mapped, &unit_pts);
            if expect_integers {
                for p in mapped.iter_mut() {
                    p.x = scalar_round_to_scalar(p.x);
                    p.y = scalar_round_to_scalar(p.y);
                }
            }
            self.conic_to(mapped[0], mapped[1], w);
            start_theta = end_theta;
        }

        *self.f_pts.last_mut().unwrap() = end_pt;
        self
    }

    pub fn add_arc(&mut self, oval: &Rect, start_angle: Scalar, sweep_angle: Scalar) -> &mut Self {
        if oval.is_empty() || 0.0 == sweep_angle {
            return self;
        }
        const K_FULL_CIRCLE_ANGLE: Scalar = 360.0;
        if sweep_angle >= K_FULL_CIRCLE_ANGLE || sweep_angle <= -K_FULL_CIRCLE_ANGLE {
            let start_over_90 = start_angle / 90.0;
            let start_over_90_i = scalar_round_to_scalar(start_over_90);
            let error = start_over_90 - start_over_90_i;
            if scalar_nearly_equal(error, 0.0) {
                let mut start_index = scalar_mod(start_over_90_i + 1.0, 4.0);
                if start_index < 0.0 {
                    start_index += 4.0;
                }
                return self.add_oval(
                    oval,
                    if sweep_angle > 0.0 {
                        SkPathDirection::Cw
                    } else {
                        SkPathDirection::Ccw
                    },
                    start_index as u32,
                );
            }
        }
        self.arc_to(oval, start_angle, sweep_angle, true)
    }

    // ---- shapes ----

    pub fn add_line(&mut self, a: Point, b: Point) -> &mut Self {
        self.move_to(a).line_to(b)
    }

    pub fn add_rect(&mut self, rect: &Rect, dir: SkPathDirection, index: u32) -> &mut Self {
        let was_empty = self.is_empty_or_moves();
        self.add_raw(
            &SkPathRawShapes::rect(rect, dir, index as usize),
            Reserve::Grow,
        );
        if was_empty {
            self.f_convexity = sk_path_direction_to_convexity(dir);
        }
        self
    }

    pub fn add_oval(&mut self, oval: &Rect, dir: SkPathDirection, index: u32) -> &mut Self {
        let was_empty = self.is_empty_or_moves();
        self.add_raw(
            &SkPathRawShapes::oval(oval, dir, index as usize),
            Reserve::Grow,
        );
        if was_empty {
            self.f_type = SkPathIsAType::Oval;
            self.f_is_a.f_direction = dir;
            self.f_is_a.f_start_index = (index % 4) as u8;
            self.f_convexity = sk_path_direction_to_convexity(dir);
        }
        self
    }

    pub fn add_rrect(&mut self, rrect: &RRect, dir: SkPathDirection, index: u32) -> &mut Self {
        let bounds = *rrect.get_bounds();
        let (as_type, new_index) = priv_::simplify_rrect(rrect, index);
        match as_type {
            priv_::RRectAsEnum::Rect => return self.add_rect(&bounds, dir, new_index),
            priv_::RRectAsEnum::Oval => return self.add_oval(&bounds, dir, new_index),
            priv_::RRectAsEnum::RRect => {}
        }

        let was_empty = self.is_empty_or_moves();
        self.add_raw(
            &SkPathRawShapes::rrect(rrect, dir, index as usize),
            Reserve::Grow,
        );
        if was_empty {
            self.f_type = SkPathIsAType::RRect;
            self.f_is_a.f_direction = dir;
            self.f_is_a.f_start_index = (index % 8) as u8;
            self.f_convexity = sk_path_direction_to_convexity(dir);
        }
        self
    }

    pub fn add_circle(&mut self, center: Point, radius: f32, dir: SkPathDirection) -> &mut Self {
        if radius >= 0.0 {
            self.add_oval(
                &Rect::new(
                    center.x - radius,
                    center.y - radius,
                    center.x + radius,
                    center.y + radius,
                ),
                dir,
                1,
            );
        }
        self
    }

    pub fn add_polygon(&mut self, pts: &[Point], is_closed: bool) -> &mut Self {
        if pts.is_empty() {
            return self;
        }
        self.move_to(pts[0]);
        self.polyline_to(&pts[1..]);
        if is_closed {
            self.close();
        }
        self
    }

    // ---- add path ----

    pub fn add_path(&mut self, path: &SkPath, mode: AddPathMode) -> &mut Self {
        self.add_path_matrix(path, &Matrix::new(), mode)
    }

    pub fn add_path_offset(
        &mut self,
        path: &SkPath,
        dx: Scalar,
        dy: Scalar,
        mode: AddPathMode,
    ) -> &mut Self {
        let matrix = Matrix::translate(dx, dy);
        self.add_path_matrix(path, &matrix, mode)
    }

    pub fn add_path_matrix(
        &mut self,
        src: &SkPath,
        matrix: &Matrix,
        mode: AddPathMode,
    ) -> &mut Self {
        if src.is_empty() {
            return self;
        }

        let can_replace_this =
            (mode == AddPathMode::Append && self.f_verbs.len() <= 1) || self.f_verbs.is_empty();
        if can_replace_this && matrix.is_identity() {
            let fill_type = self.f_fill_type;
            *self = SkPathBuilder::from_path(src);
            self.f_fill_type = fill_type;
            return self;
        }

        self.f_convexity = SkPathConvexity::Unknown;

        if mode == AddPathMode::Append && !matrix.has_perspective() {
            if !self.f_verbs.is_empty()
                && *self.f_verbs.last().unwrap() == SkPathVerb::Move
                && !src.is_empty()
            {
                self.f_verbs.pop();
                self.f_pts.pop();
            }

            let last_move_to_index =
                priv_::find_last_move_to_index(src.verbs(), src.points().len());
            self.f_last_move_index = last_move_to_index + self.count_points();

            let start = self.f_pts.len();
            self.grow_for_verbs_in_path(src);
            let count = src.points().len();
            let mut mapped = vec![Point::new(0.0, 0.0); count];
            matrix.map_points(&mut mapped, src.points());
            self.f_pts[start..start + count].copy_from_slice(&mapped);

            return self;
        }

        // `SkPathPriv::Iterate(path)`: the raw range iterator, which does not trim a trailing
        // moveTo, and yields nothing at all when the source path is not finite.
        let mut first_verb = true;
        if src.is_finite() {
            let mut iter = RangeIter::from_slices(src.verbs(), src.points(), src.conic_weights());
            while let Some((verb, pts, weight)) = iter.next() {
                match verb {
                    SkPathVerb::Move => {
                        let mapped = matrix.map_point(pts[0]);
                        if first_verb && mode == AddPathMode::Extend && !self.is_empty() {
                            self.ensure_move();
                            let last_pt = self.get_last_pt();
                            if last_pt.is_none() || last_pt.unwrap() != mapped {
                                self.line_to(mapped);
                            }
                        } else {
                            self.move_to(mapped);
                        }
                    }
                    SkPathVerb::Line => {
                        let mapped = matrix.map_point(pts[1]);
                        self.line_to(mapped);
                    }
                    SkPathVerb::Quad => {
                        let mut mapped = [Point::new(0.0, 0.0); 2];
                        matrix.map_points(&mut mapped, &pts[1..3]);
                        self.quad_to(mapped[0], mapped[1]);
                    }
                    SkPathVerb::Conic => {
                        let mut mapped = [Point::new(0.0, 0.0); 2];
                        matrix.map_points(&mut mapped, &pts[1..3]);
                        self.conic_to(mapped[0], mapped[1], weight);
                    }
                    SkPathVerb::Cubic => {
                        let mut mapped = [Point::new(0.0, 0.0); 3];
                        matrix.map_points(&mut mapped, &pts[1..4]);
                        self.cubic_to(mapped[0], mapped[1], mapped[2]);
                    }
                    SkPathVerb::Close => {
                        self.close();
                    }
                    SkPathVerb::Done => {}
                }
                first_verb = false;
            }
        }
        self
    }

    pub fn inc_reserve(&mut self, extra_pt: usize, extra_verb: usize, extra_conic: usize) {
        self.f_pts.reserve(extra_pt);
        self.f_verbs.reserve(extra_verb);
        self.f_conic_weights.reserve(extra_conic);
    }

    pub fn offset(&mut self, dx: Scalar, dy: Scalar) -> &mut Self {
        for p in &mut self.f_pts {
            p.offset(dx, dy);
        }
        self
    }

    pub fn transform(&mut self, matrix: &Matrix) -> &mut Self {
        if matrix.is_identity() || self.is_empty() {
            return self;
        }

        if matrix.has_perspective() {
            let src = self.detach(None);
            self.set_fill_type(src.get_fill_type());

            let mut clipped = SkPath::new();
            let src = if priv_::perspective_clip(&src, matrix, &mut clipped) {
                clipped
            } else {
                src
            };

            let mut iter = SkPathIter::new(src.points(), src.verbs(), src.conic_weights());
            while let Some(rec) = iter.next() {
                match rec.f_verb {
                    SkPathVerb::Move => {
                        self.move_to(rec.f_points[0]);
                    }
                    SkPathVerb::Line => {
                        self.line_to(rec.f_points[1]);
                    }
                    SkPathVerb::Quad => {
                        let w = SkConic::transform_w(rec.f_points, 1.0, matrix);
                        self.conic_to(rec.f_points[1], rec.f_points[2], w);
                    }
                    SkPathVerb::Conic => {
                        let w = SkConic::transform_w(rec.f_points, rec.f_conic_weight, matrix);
                        self.conic_to(rec.f_points[1], rec.f_points[2], w);
                    }
                    SkPathVerb::Cubic => {
                        let pts = [
                            rec.f_points[0],
                            rec.f_points[1],
                            rec.f_points[2],
                            rec.f_points[3],
                        ];
                        subdivide_cubic_to(self, &pts);
                    }
                    SkPathVerb::Close => {
                        self.close();
                    }
                    SkPathVerb::Done => {}
                }
            }
        } else {
            if !matrix.rect_stays_rect() || !priv_::is_axis_aligned(&self.f_pts) {
                self.f_type = SkPathIsAType::General;
                if sk_path_convexity_is_convex(self.f_convexity) {
                    self.f_convexity = SkPathConvexity::Unknown;
                }
            }

            if self.f_type == SkPathIsAType::Oval || self.f_type == SkPathIsAType::RRect {
                let (dir, start) = priv_::transform_dir_and_start(
                    matrix,
                    self.f_type == SkPathIsAType::RRect,
                    self.f_is_a.f_direction,
                    self.f_is_a.f_start_index as u32,
                );
                self.f_is_a.f_direction = dir;
                self.f_is_a.f_start_index = start as u8;
            }
        }

        matrix.map_points_in_place(&mut self.f_pts);
        self
    }

    pub fn set_point(&mut self, index: usize, p: Point) {
        if index < self.f_pts.len() {
            self.f_pts[index] = p;
            self.f_type = SkPathIsAType::General;
        }
    }

    pub fn set_last_point(&mut self, p: Point) {
        if !self.f_pts.is_empty() {
            self.set_point(self.f_pts.len() - 1, p);
        }
    }

    pub fn set_last_pt(&mut self, p: Point) {
        if self.f_pts.is_empty() {
            self.move_to(p);
        } else {
            self.set_point(self.f_pts.len() - 1, p);
        }
    }

    // ---- raw / snapshot / detach ----

    pub fn add_raw(&mut self, raw: &SkPathRaw, reserve: Reserve) -> &mut Self {
        if reserve == Reserve::Grow {
            self.inc_reserve(raw.points().len(), raw.verbs().len(), raw.conics().len());
        } else {
            self.f_pts.reserve_exact(raw.points().len());
            self.f_verbs.reserve_exact(raw.verbs().len());
            self.f_conic_weights.reserve_exact(raw.conics().len());
        }

        let mut iter = raw.iter();
        while let Some(rec) = iter.next() {
            let pts = rec.f_points;
            match rec.f_verb {
                SkPathVerb::Move => {
                    self.move_to(pts[0]);
                }
                SkPathVerb::Line => {
                    self.line_to(pts[1]);
                }
                SkPathVerb::Quad => {
                    self.quad_to(pts[1], pts[2]);
                }
                SkPathVerb::Conic => {
                    self.conic_to(pts[1], pts[2], rec.f_conic_weight);
                }
                SkPathVerb::Cubic => {
                    self.cubic_to(pts[1], pts[2], pts[3]);
                }
                SkPathVerb::Close => {
                    self.close();
                }
                SkPathVerb::Done => {}
            }
        }

        let has_trailing_move =
            |vbs: &[SkPathVerb]| !vbs.is_empty() && vbs[vbs.len() - 1] == SkPathVerb::Move;

        if has_trailing_move(raw.verbs()) && !has_trailing_move(&self.f_verbs) {
            let p = *raw.points().last().unwrap();
            self.move_to(p);
        }

        self
    }

    pub fn raw(&self, rc: SkResolveConvexity) -> Option<SkPathRaw> {
        let bounds = self.compute_finite_bounds()?;
        let mut convexity = self.f_convexity;
        if convexity == SkPathConvexity::Unknown && rc == SkResolveConvexity::Yes {
            convexity = priv_::compute_convexity(&self.f_pts, &self.f_verbs, &self.f_conic_weights);
        }
        Some(SkPathRaw {
            f_points: self.f_pts.clone(),
            f_verbs: self.f_verbs.clone(),
            f_conics: self.f_conic_weights.clone(),
            f_bounds: bounds,
            f_fill_type: self.f_fill_type,
            f_convexity: convexity,
            f_segment_mask: self.f_segment_mask as u8,
        })
    }

    pub fn compute_finite_bounds(&self) -> Option<Rect> {
        priv_::trimmed_bounds(&self.f_pts, &self.f_verbs)
    }

    pub fn compute_bounds(&self) -> Rect {
        self.compute_finite_bounds()
            .unwrap_or_else(Rect::make_empty)
    }

    pub fn compute_tight_bounds(&self) -> Option<Rect> {
        if !self.is_finite() {
            return None;
        }
        Some(priv_::compute_tight_bounds(
            &self.f_pts,
            &self.f_verbs,
            &self.f_conic_weights,
        ))
    }

    pub fn snapshot(&self, mx: Option<&Matrix>) -> SkPath {
        let identity;
        let mx = match mx {
            Some(m) => m,
            None => {
                identity = Matrix::new();
                &identity
            }
        };

        let pdata = match self.raw(SkResolveConvexity::No) {
            Some(raw) => SkPathData::make_transform(&raw, mx),
            None => None,
        };

        let pdata = if let Some(pdata) = pdata {
            if self.f_type != SkPathIsAType::General
                && mx.rect_stays_rect()
                && !pdata.bounds().is_empty()
            {
                let (dir, start) = priv_::transform_dir_and_start(
                    mx,
                    self.f_type == SkPathIsAType::RRect,
                    self.f_is_a.f_direction,
                    self.f_is_a.f_start_index as u32,
                );
                pdata.setup_is_a(self.f_type, dir, start as u8);
            }
            Some(pdata)
        } else {
            None
        };

        SkPath::make_null_check(pdata, self.f_fill_type, self.f_is_volatile)
    }

    pub fn detach(&mut self, mx: Option<&Matrix>) -> SkPath {
        let path = self.snapshot(mx);
        self.reset();
        path
    }

    pub fn snapshot_data(&self) -> Arc<SkPathData> {
        if self.f_verbs.len() <= 1 {
            return SkPathData::empty();
        }
        match self.f_type {
            SkPathIsAType::General => {}
            SkPathIsAType::Oval => {
                if let Some(r) = Rect::bounds(&self.f_pts) {
                    return SkPathData::oval(
                        &r,
                        self.f_is_a.f_direction,
                        self.f_is_a.f_start_index as u32,
                    )
                    .unwrap_or_else(SkPathData::peek_error_singleton);
                }
                return SkPathData::peek_error_singleton();
            }
            SkPathIsAType::RRect => {
                if let Some(r) = Rect::bounds(&self.f_pts) {
                    let rr = priv_::deduce_rrect_from_contour(&r, &self.f_pts, &self.f_verbs);
                    return SkPathData::rrect(
                        &rr,
                        self.f_is_a.f_direction,
                        self.f_is_a.f_start_index as u32,
                    )
                    .unwrap_or_else(SkPathData::peek_error_singleton);
                }
                return SkPathData::peek_error_singleton();
            }
        }
        SkPathData::make(&self.f_pts, &self.f_verbs, &self.f_conic_weights)
            .unwrap_or_else(SkPathData::peek_error_singleton)
    }

    pub fn detach_data(&mut self) -> Arc<SkPathData> {
        let data = self.snapshot_data();
        self.reset();
        data
    }

    pub fn contains(&self, p: Point) -> bool {
        match self.raw(SkResolveConvexity::No) {
            Some(raw) => priv_::contains(&raw, p),
            None => false,
        }
    }

    pub fn private_reverse_path_to(&mut self, path: &SkPath) -> &mut Self {
        if path.verbs().is_empty() {
            return self;
        }

        let mut vi = path.verbs().len();
        let mut pi: isize = path.points().len() as isize - 1;
        let mut ci = path.conic_weights().len();

        while vi > 0 {
            vi -= 1;
            let v = path.verbs()[vi];
            pi -= priv_::pts_in_verb(v) as isize;
            match v {
                SkPathVerb::Move => {
                    return self;
                }
                SkPathVerb::Line => {
                    let p = path.points()[pi as usize];
                    self.line_to(p);
                }
                SkPathVerb::Quad => {
                    let a = path.points()[pi as usize];
                    let b = path.points()[(pi + 1) as usize];
                    self.quad_to(b, a);
                }
                SkPathVerb::Conic => {
                    ci -= 1;
                    let a = path.points()[pi as usize];
                    let b = path.points()[(pi + 1) as usize];
                    self.conic_to(b, a, path.conic_weights()[ci]);
                }
                SkPathVerb::Cubic => {
                    let a = path.points()[pi as usize];
                    let b = path.points()[(pi + 1) as usize];
                    let c = path.points()[(pi + 2) as usize];
                    self.cubic_to(c, b, a);
                }
                SkPathVerb::Close => {}
                SkPathVerb::Done => {}
            }
        }
        self
    }

    pub fn private_reverse_add_path(&mut self, src: &SkPath) -> &mut Self {
        if src.verbs().is_empty() {
            return self;
        }

        let mut vi = src.verbs().len();
        let mut pi: isize = src.points().len() as isize;
        let mut ci = src.conic_weights().len();

        let mut need_move = true;
        let mut need_close = false;
        while vi > 0 {
            vi -= 1;
            let v = src.verbs()[vi];
            let n = priv_::pts_in_verb(v) as isize;

            if need_move {
                pi -= 1;
                let p = src.points()[pi as usize];
                self.move_to(Point::new(p.x, p.y));
                need_move = false;
            }
            pi -= n;
            match v {
                SkPathVerb::Move => {
                    if need_close {
                        self.close();
                        need_close = false;
                    }
                    need_move = true;
                    pi += 1;
                }
                SkPathVerb::Line => {
                    let p = src.points()[pi as usize];
                    self.line_to(p);
                }
                SkPathVerb::Quad => {
                    let a = src.points()[pi as usize];
                    let b = src.points()[(pi + 1) as usize];
                    self.quad_to(b, a);
                }
                SkPathVerb::Conic => {
                    ci -= 1;
                    let a = src.points()[pi as usize];
                    let b = src.points()[(pi + 1) as usize];
                    self.conic_to(b, a, src.conic_weights()[ci]);
                }
                SkPathVerb::Cubic => {
                    let a = src.points()[pi as usize];
                    let b = src.points()[(pi + 1) as usize];
                    let c = src.points()[(pi + 2) as usize];
                    self.cubic_to(c, b, a);
                }
                SkPathVerb::Close => {
                    need_close = true;
                }
                SkPathVerb::Done => {}
            }
        }
        self
    }

    // ---- dump ----

    pub fn dump_to_string(&self, format: DumpFormat) -> SkString {
        let as_type = if format == DumpFormat::Hex {
            SkScalarAsStringType::Hex
        } else {
            SkScalarAsStringType::Dec
        };

        let mut out = SkString::new();
        let ft = g_fill_type_strs(self.f_fill_type as usize);
        out.append_str(&format!("SkPathBuilder(SkPathFillType::k{ft})\n"));

        let mut iter = SkPathIter::new(&self.f_pts, &self.f_verbs, &self.f_conic_weights);
        while let Some(rec) = iter.next() {
            let mut cmd = String::new();
            let mut pts: &[Point] = &[];
            let mut cw = K_SENTINEL_CONIC_WEIGHT;
            match rec.f_verb {
                SkPathVerb::Move => {
                    cmd.push_str(".moveTo");
                    pts = &rec.f_points[0..1];
                }
                SkPathVerb::Line => {
                    cmd.push_str(".lineTo");
                    pts = &rec.f_points[1..2];
                }
                SkPathVerb::Quad => {
                    cmd.push_str(".quadTo");
                    pts = &rec.f_points[1..3];
                }
                SkPathVerb::Conic => {
                    cmd.push_str(".conicTo");
                    pts = &rec.f_points[1..3];
                    cw = rec.conic_weight();
                }
                SkPathVerb::Cubic => {
                    cmd.push_str(".cubicTo");
                    pts = &rec.f_points[1..4];
                }
                SkPathVerb::Close => {
                    cmd.push_str(".close()\n");
                    out.append_str(&cmd);
                }
                SkPathVerb::Done => {}
            }
            if !pts.is_empty() {
                append_params(&mut out, &cmd, pts, as_type, false, cw);
            }
        }
        out
    }

    pub fn dump(&self, format: DumpFormat) {
        let s = self.dump_to_string(format);
        print!("{}", s.as_str());
    }

    pub fn dump_default(&self) {
        self.dump(DumpFormat::Decimal);
    }
}

impl SkPathBuilder {
    pub fn iter_recs(&self) -> SkPathIter<'_> {
        self.iter()
    }
}

/// `SkPathBuilder::AddPathMode` re-exported at this module level.
pub use crate::sk_path::AddPathMode;

fn subdivide_cubic_to(path: &mut SkPathBuilder, pts: &[Point; 4]) {
    subdivide_cubic_to_impl(path, pts, 2);
}

fn subdivide_cubic_to_impl(path: &mut SkPathBuilder, pts: &[Point; 4], level: i32) {
    let level = level - 1;
    if level >= 0 {
        let mut tmp = [Point::new(0.0, 0.0); 7];
        sk_chop_cubic_at_half(pts, &mut tmp);
        let first = [tmp[0], tmp[1], tmp[2], tmp[3]];
        subdivide_cubic_to_impl(path, &first, level);
        let second = [tmp[3], tmp[4], tmp[5], tmp[6]];
        subdivide_cubic_to_impl(path, &second, level);
    } else {
        path.cubic_to(pts[1], pts[2], pts[3]);
    }
}

fn add_pt(builder: &mut SkPathBuilder, force_move_to: bool, pt: Point) -> &mut SkPathBuilder {
    if force_move_to {
        builder.move_to(pt)
    } else if !nearly_equal(*builder.f_pts.last().unwrap(), pt) {
        builder.line_to(pt)
    } else {
        builder
    }
}

fn nearly_equal(a: Point, b: Point) -> bool {
    scalar_nearly_equal(a.x, b.x) && scalar_nearly_equal(a.y, b.y)
}

fn arc_is_lone_point(
    oval: &Rect,
    start_angle: Scalar,
    sweep_angle: Scalar,
    pt: &mut Point,
) -> bool {
    if 0.0 == sweep_angle && (0.0 == start_angle || 360.0 == start_angle) {
        *pt = Point::new(oval.right, oval.center_y());
        true
    } else if 0.0 == oval.width() && 0.0 == oval.height() {
        *pt = Point::new(oval.right, oval.top);
        true
    } else {
        false
    }
}

fn angles_to_unit_vectors(
    start_angle: Scalar,
    sweep_angle: Scalar,
    start_v: &mut Point,
    stop_v: &mut Point,
) -> SkPathDirection {
    let start_rad = degrees_to_radians(start_angle);
    let mut stop_rad = degrees_to_radians(start_angle + sweep_angle);

    start_v.y = scalar_sin_snap_to_zero(start_rad);
    start_v.x = scalar_cos_snap_to_zero(start_rad);
    stop_v.y = scalar_sin_snap_to_zero(stop_rad);
    stop_v.x = scalar_cos_snap_to_zero(stop_rad);

    if *start_v == *stop_v {
        let sw = scalar_abs(sweep_angle);
        if sw < 360.0 && sw > 359.0 {
            let delta_rad = scalar_copy_sign(1.0 / 512.0, sweep_angle);
            loop {
                stop_rad -= delta_rad;
                stop_v.y = scalar_sin_snap_to_zero(stop_rad);
                stop_v.x = scalar_cos_snap_to_zero(stop_rad);
                if *start_v != *stop_v {
                    break;
                }
            }
        }
    }
    if sweep_angle > 0.0 {
        SkPathDirection::Cw
    } else {
        SkPathDirection::Ccw
    }
}

fn build_arc_conics(
    oval: &Rect,
    start: &Point,
    stop: &Point,
    dir: SkPathDirection,
    conics: &mut [SkConic],
    single_pt: &mut Point,
) -> i32 {
    let mut matrix = Matrix::new();
    matrix.set_scale(oval.width() / 2.0, oval.height() / 2.0);
    matrix.post_translate(oval.center_x(), oval.center_y());

    let count = SkConic::build_unit_arc(*start, *stop, dir, Some(&matrix), conics);
    if 0 == count {
        *single_pt = matrix.map_point(*stop);
    }
    count
}

fn normalize_d2(v: [f64; 2]) -> [f64; 2] {
    let mag = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if mag > 0.0 && mag.is_finite() {
        [v[0] / mag, v[1] / mag]
    } else {
        [f64::NAN, f64::NAN]
    }
}

fn is_finite2_f64(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite()
}

fn g_fill_type_strs(index: usize) -> &'static str {
    const STRS: [&str; 4] = ["Winding", "EvenOdd", "InverseWinding", "InverseEvenOdd"];
    STRS[index & 3]
}

const K_SENTINEL_CONIC_WEIGHT: f32 = -12345.0;

fn append_params(
    str: &mut SkString,
    label: &str,
    pts: &[Point],
    str_type: SkScalarAsStringType,
    use_semicolon: bool,
    conic_weight: f32,
) {
    str.append_str(label);
    str.append_str("(");

    let mut values: Vec<Scalar> = Vec::with_capacity(pts.len() * 2);
    for p in pts {
        values.push(p.x);
        values.push(p.y);
    }
    let count = values.len();

    for (i, value) in values.iter().enumerate() {
        sk_append_scalar(str, *value, str_type);
        if i < count - 1 {
            str.append_str(", ");
        }
    }
    if conic_weight != K_SENTINEL_CONIC_WEIGHT {
        str.append_str(", ");
        sk_append_scalar(str, conic_weight, str_type);
    }
    str.append_str(if use_semicolon { ");" } else { ")" });
    if str_type == SkScalarAsStringType::Hex {
        str.append_str("  // ");
        for (i, value) in values.iter().enumerate() {
            str.append_str(&format!("{value}"));
            if i < count - 1 {
                str.append_str(", ");
            }
        }
        if conic_weight >= 0.0 {
            str.append_str(", ");
            str.append_str(&format!("{conic_weight}"));
        }
    }
    str.append_str("\n");
}

#[allow(dead_code)]
fn _unused_rec(_rec: Option<SkPathIterRec<'_>>) {}
