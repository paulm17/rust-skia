//! Port of `include/effects/SkDashPathEffect.h`, `src/effects/SkDashPathEffect.cpp` and the
//! `SkDashImpl` class declared in `src/effects/SkDashImpl.h` (whose declaration is inventoried
//! under M13, but whose constructor and behavior live in the M08-owned translation unit).

use crate::matrix::Matrix;
use crate::point::Point;
use crate::rect::Rect;
use crate::scalar::{
    SCALAR_1, Scalar, scalar_floor_to_int, scalar_invert, scalar_is_int, scalar_mod,
    scalar_nearly_equal,
};
use crate::sk_dash_path::{
    K_MAX_DASH_COUNT, StrokeRecApplication, calc_dash_parameters, internal_filter, valid_dash_path,
};
use crate::sk_paint_types::{SkPaintCap, SkPaintJoin};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::{
    DashInfo, K_CIRCLES_POINT_FLAG, PointData, SkPathEffect, SkPathEffectBase,
};
use crate::sk_stroke_rec::SkStrokeRec;

/// `SkDashImpl`.
pub struct SkDashImpl {
    f_intervals: Vec<Scalar>,
    f_phase: Scalar,

    // computed from phase
    f_initial_dash_length: Scalar,
    f_interval_length: Scalar,
    f_initial_dash_index: usize,
}

impl SkDashImpl {
    pub fn new(intervals: &[Scalar], phase: Scalar) -> Self {
        debug_assert!(intervals.len() > 1 && (intervals.len() % 2) == 0);

        let mut impl_ = SkDashImpl {
            f_intervals: intervals.to_vec(),
            f_phase: 0.0,
            f_initial_dash_length: -1.0,
            f_interval_length: 0.0,
            f_initial_dash_index: 0,
        };

        // set the internal data members
        calc_dash_parameters(
            phase,
            &impl_.f_intervals,
            &mut impl_.f_initial_dash_length,
            &mut impl_.f_initial_dash_index,
            &mut impl_.f_interval_length,
            Some(&mut impl_.f_phase),
        );
        impl_
    }

    pub fn intervals(&self) -> &[Scalar] {
        &self.f_intervals
    }

    pub fn phase(&self) -> Scalar {
        self.f_phase
    }

    pub fn initial_dash_length(&self) -> Scalar {
        self.f_initial_dash_length
    }

    pub fn interval_length(&self) -> Scalar {
        self.f_interval_length
    }

    pub fn initial_dash_index(&self) -> usize {
        self.f_initial_dash_index
    }
}

fn outset_for_stroke(rect: &mut Rect, rec: &SkStrokeRec) {
    let mut radius = rec.get_width() / 2.0;
    if 0.0 == radius {
        radius = SCALAR_1; // hairlines
    }
    if SkPaintJoin::Miter == rec.get_join() {
        radius *= rec.get_miter();
    }
    rect.outset(radius, radius);
}

/// Attempt to trim the line to minimally cover the cull rect (currently only works for horizontal
/// and vertical lines). Return true if processing should continue; false otherwise.
fn cull_line(
    pts: &mut [Point; 2],
    rec: &SkStrokeRec,
    ctm: &Matrix,
    cull_rect: Option<&Rect>,
    interval_length: Scalar,
) -> bool {
    let cull_rect = match cull_rect {
        None => {
            debug_assert!(false); // Shouldn't ever occur in practice
            return false;
        }
        Some(cull_rect) => cull_rect,
    };

    let dx = pts[1].x() - pts[0].x();
    let dy = pts[1].y() - pts[0].y();

    if (dx != 0.0 && dy != 0.0) || (dx == 0.0 && dy == 0.0) {
        return false;
    }

    let mut bounds = *cull_rect;
    outset_for_stroke(&mut bounds, rec);

    // cullRect is in device space while pts are in the local coordinate system
    // defined by the ctm. We want our answer in the local coordinate system.

    debug_assert!(ctm.rect_stays_rect());
    let mut inv = Matrix::new();
    if !ctm.invert(Some(&mut inv)) {
        return false;
    }

    bounds = inv.map_rect(&bounds);

    if dx != 0.0 {
        debug_assert!(dx != 0.0 && dy == 0.0);
        let mut min_x = pts[0].x;
        let mut max_x = pts[1].x;

        if dx < 0.0 {
            std::mem::swap(&mut min_x, &mut max_x);
        }

        debug_assert!(min_x < max_x);
        if max_x <= bounds.left() || min_x >= bounds.right() {
            return false;
        }

        // Now we actually perform the chop, removing the excess to the left and
        // right of the bounds (keeping our new line "in phase" with the dash,
        // hence the (mod intervalLength).

        if min_x < bounds.left() {
            min_x = bounds.left() - scalar_mod(bounds.left() - min_x, interval_length);
        }
        if max_x > bounds.right() {
            max_x = bounds.right() + scalar_mod(max_x - bounds.right(), interval_length);
        }

        debug_assert!(max_x > min_x);
        if dx < 0.0 {
            std::mem::swap(&mut min_x, &mut max_x);
        }
        pts[0].x = min_x;
        pts[1].x = max_x;
    } else {
        debug_assert!(dy != 0.0 && dx == 0.0);
        let mut min_y = pts[0].y;
        let mut max_y = pts[1].y;

        if dy < 0.0 {
            std::mem::swap(&mut min_y, &mut max_y);
        }

        debug_assert!(min_y < max_y);
        if max_y <= bounds.top() || min_y >= bounds.bottom() {
            return false;
        }

        // Now we actually perform the chop, removing the excess to the top and
        // bottom of the bounds (keeping our new line "in phase" with the dash,
        // hence the (mod intervalLength).

        if min_y < bounds.top() {
            min_y = bounds.top() - scalar_mod(bounds.top() - min_y, interval_length);
        }
        if max_y > bounds.bottom() {
            max_y = bounds.bottom() + scalar_mod(max_y - bounds.bottom(), interval_length);
        }

        debug_assert!(max_y > min_y);
        if dy < 0.0 {
            std::mem::swap(&mut min_y, &mut max_y);
        }
        pts[0].y = min_y;
        pts[1].y = max_y;
    }

    true
}

impl SkPathEffectBase for SkDashImpl {
    fn on_filter_path(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cull_rect: Option<&Rect>,
        _ctm: &Matrix,
    ) -> bool {
        internal_filter(
            builder,
            src,
            rec,
            cull_rect,
            &self.f_intervals,
            self.f_initial_dash_length,
            self.f_initial_dash_index as i32,
            self.f_interval_length,
            self.f_phase,
            StrokeRecApplication::Allow,
        )
    }

    fn on_as_points(
        &self,
        results: &mut PointData,
        src: &SkPath,
        rec: &SkStrokeRec,
        matrix: &Matrix,
        cull_rect: Option<&Rect>,
    ) -> bool {
        // width < 0 -> fill && width == 0 -> hairline so requiring width > 0 rules both out
        if 0.0 >= rec.get_width() {
            return false;
        }

        // TODO: this next test could be eased up. We could allow any number of
        // intervals as long as all the ons match and all the offs match.
        // Additionally, they do not necessarily need to be integers.
        // We cannot allow arbitrary intervals since we want the returned points
        // to be uniformly sized.
        if self.f_intervals.len() != 2
            || !scalar_nearly_equal(self.f_intervals[0], self.f_intervals[1])
            || !scalar_is_int(self.f_intervals[0])
            || !scalar_is_int(self.f_intervals[1])
        {
            return false;
        }

        let mut pts = [Point::new(0.0, 0.0); 2];

        if !src.is_line(Some(&mut pts)) {
            return false;
        }

        // TODO: this test could be eased up to allow circles
        if SkPaintCap::Butt != rec.get_cap() {
            return false;
        }

        // TODO: this test could be eased up for circles. Rotations could be allowed.
        if !matrix.rect_stays_rect() {
            return false;
        }

        // See if the line can be limited to something plausible.
        if !cull_line(&mut pts, rec, matrix, cull_rect, self.f_interval_length) {
            return false;
        }

        let length = Point::distance(&pts[1], &pts[0]);

        let mut tangent = pts[1] - pts[0];
        if tangent.is_zero() {
            return false;
        }

        tangent.scale_in_place(scalar_invert(length));

        // TODO: make this test for horizontal & vertical lines more robust
        let mut is_x_axis = true;
        if scalar_nearly_equal(SCALAR_1, tangent.x) || scalar_nearly_equal(-SCALAR_1, tangent.x) {
            results
                .f_size
                .set(self.f_intervals[0] / 2.0, rec.get_width() / 2.0);
        } else if scalar_nearly_equal(SCALAR_1, tangent.y)
            || scalar_nearly_equal(-SCALAR_1, tangent.y)
        {
            results
                .f_size
                .set(rec.get_width() / 2.0, self.f_intervals[0] / 2.0);
            is_x_axis = false;
        } else if SkPaintCap::Round != rec.get_cap() {
            // Angled lines don't have axis-aligned boxes.
            return false;
        }

        {
            results.f_flags = 0;
            let clamped_initial_dash_length = cpp_min(length, self.f_initial_dash_length);

            if SkPaintCap::Round == rec.get_cap() {
                results.f_flags |= K_CIRCLES_POINT_FLAG;
            }

            results.f_num_points = 0;
            let mut len2 = length;
            if clamped_initial_dash_length > 0.0 || 0 == self.f_initial_dash_index {
                debug_assert!(len2 >= clamped_initial_dash_length);
                if 0 == self.f_initial_dash_index {
                    if clamped_initial_dash_length > 0.0 {
                        if clamped_initial_dash_length >= self.f_intervals[0] {
                            results.f_num_points += 1; // partial first dash
                        }
                        len2 -= clamped_initial_dash_length;
                    }
                    len2 -= self.f_intervals[1]; // also skip first space
                    if len2 < 0.0 {
                        len2 = 0.0;
                    }
                } else {
                    len2 -= clamped_initial_dash_length; // skip initial partial empty
                }
            }
            // Too many midpoints can cause results->fNumPoints to overflow or
            // otherwise cause the results->fPoints allocation below to OOM.
            // Cap it to a sane value.
            let num_intervals = len2 / self.f_interval_length;
            if !num_intervals.is_finite() || num_intervals > K_MAX_DASH_COUNT {
                return false;
            }
            let mut num_mid_points = scalar_floor_to_int(num_intervals);
            results.f_num_points += num_mid_points;
            len2 -= num_mid_points as Scalar * self.f_interval_length;
            let mut partial_last = false;
            if len2 > 0.0 {
                if len2 < self.f_intervals[0] {
                    partial_last = true;
                } else {
                    num_mid_points += 1;
                    results.f_num_points += 1;
                }
            }

            results.f_points = vec![Point::new(0.0, 0.0); results.f_num_points.max(0) as usize];

            let mut distance: Scalar = 0.0;
            let mut cur_pt = 0usize;

            if clamped_initial_dash_length > 0.0 || 0 == self.f_initial_dash_index {
                debug_assert!(clamped_initial_dash_length <= length);

                if 0 == self.f_initial_dash_index {
                    if clamped_initial_dash_length > 0.0 {
                        // partial first block
                        debug_assert!(SkPaintCap::Round != rec.get_cap()); // can't handle partial circles
                        let x = pts[0].x + tangent.x * (clamped_initial_dash_length / 2.0);
                        let y = pts[0].y + tangent.y * (clamped_initial_dash_length / 2.0);
                        let (half_width, half_height) = if is_x_axis {
                            (clamped_initial_dash_length / 2.0, rec.get_width() / 2.0)
                        } else {
                            (rec.get_width() / 2.0, clamped_initial_dash_length / 2.0)
                        };
                        if clamped_initial_dash_length < self.f_intervals[0] {
                            // This one will not be like the others
                            results.f_first = SkPath::rect_default(&Rect::new(
                                x - half_width,
                                y - half_height,
                                x + half_width,
                                y + half_height,
                            ));
                        } else {
                            debug_assert!(cur_pt < results.f_num_points as usize);
                            results.f_points[cur_pt].set(x, y);
                            cur_pt += 1;
                        }

                        distance += clamped_initial_dash_length;
                    }

                    distance += self.f_intervals[1]; // skip over the next blank block too
                } else {
                    distance += clamped_initial_dash_length;
                }
            }

            if 0 != num_mid_points {
                distance += self.f_intervals[0] / 2.0;

                for _ in 0..num_mid_points {
                    let x = pts[0].x + tangent.x * distance;
                    let y = pts[0].y + tangent.y * distance;

                    debug_assert!(cur_pt < results.f_num_points as usize);
                    results.f_points[cur_pt].set(x, y);
                    cur_pt += 1;

                    distance += self.f_interval_length;
                }

                distance -= self.f_intervals[0] / 2.0;
            }

            if partial_last {
                // partial final block
                debug_assert!(SkPaintCap::Round != rec.get_cap()); // can't handle partial circles
                let temp = length - distance;
                debug_assert!(temp < self.f_intervals[0]);
                let x = pts[0].x + tangent.x * (distance + (temp / 2.0));
                let y = pts[0].y + tangent.y * (distance + (temp / 2.0));
                let (half_width, half_height) = if is_x_axis {
                    (temp / 2.0, rec.get_width() / 2.0)
                } else {
                    (rec.get_width() / 2.0, temp / 2.0)
                };
                results.f_last = SkPath::rect_default(&Rect::new(
                    x - half_width,
                    y - half_height,
                    x + half_width,
                    y + half_height,
                ));
            }

            debug_assert!(cur_pt == results.f_num_points.max(0) as usize);
        }

        true
    }

    fn as_a_dash(&self) -> Option<DashInfo> {
        Some(DashInfo {
            f_intervals: self.f_intervals.clone(),
            f_phase: self.f_phase,
        })
    }

    fn compute_fast_bounds(&self, _bounds: Option<&mut Rect>) -> bool {
        // Dashing a path returns a subset of the input path so just return true and leave
        // bounds unmodified
        true
    }
}

#[inline]
fn cpp_min(a: Scalar, b: Scalar) -> Scalar {
    if b < a { b } else { a }
}

/// `SkDashPathEffect::Make`.
pub fn make_dash_path_effect(intervals: &[Scalar], phase: Scalar) -> Option<SkPathEffect> {
    if !valid_dash_path(phase, intervals) {
        return None;
    }
    Some(SkPathEffect::from_base(SkDashImpl::new(intervals, phase)))
}

/// `SkDashPathEffect`.
pub struct SkDashPathEffect;

impl SkDashPathEffect {
    /// `SkDashPathEffect::Make`. `intervals` must hold an even number (>= 2) of entries with the
    /// even indices specifying "on" intervals and the odd indices "off" intervals.
    pub fn make(intervals: &[Scalar], phase: Scalar) -> Option<SkPathEffect> {
        make_dash_path_effect(intervals, phase)
    }
}
