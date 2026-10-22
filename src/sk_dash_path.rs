//! Port of `src/utils/SkDashPathPriv.h` and `src/utils/SkDashPath.cpp`.
//!
//! The reference indexes `SkPoint::fX`/`fY` and `SkRect::fLeft/fTop/fRight/fBottom` through a
//! two-element pointer walk (`(&pts[0].fX)[xyOffset]`). The Rust port reproduces that with the
//! explicit `xy_component`/`rect_left_top`/`rect_right_bottom` accessors, which select the same
//! member. Double-precision accumulation for the dash walk is preserved.

use crate::floating_point::{ieee_float_divide, is_finite2};
use crate::point::{Point, Vector};
use crate::point_priv::PointPriv;
use crate::rect::Rect;
use crate::scalar::double_to_scalar;
use crate::scalar::{SCALAR_NEARLY_ZERO, Scalar, scalar_abs, scalar_ceil_to_int, scalar_mod};
use crate::sk_paint_types::SkPaintCap;
use crate::sk_path::{Iter, SkPath};
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::DashInfo;
use crate::sk_path_measure::SkPathMeasure;
use crate::sk_path_types::SkPathVerb;
use crate::sk_stroke_rec::{SkStrokeRec, SkStrokeRecStyle};

/// `SkDashPath::kMaxDashCount`.
pub const K_MAX_DASH_COUNT: Scalar = 1000000.0;

/// `is_even(int x)`.
#[inline]
fn is_even(x: i32) -> bool {
    (x & 1) == 0
}

#[inline]
fn xy_component(p: &Point, xy_offset: usize) -> Scalar {
    if xy_offset == 0 { p.x } else { p.y }
}

#[inline]
fn set_xy_component(p: &mut Point, xy_offset: usize, value: Scalar) {
    if xy_offset == 0 {
        p.x = value;
    } else {
        p.y = value;
    }
}

#[inline]
fn rect_left_top(r: &Rect, xy_offset: usize) -> Scalar {
    if xy_offset == 0 { r.left() } else { r.top() }
}

#[inline]
fn rect_right_bottom(r: &Rect, xy_offset: usize) -> Scalar {
    if xy_offset == 0 {
        r.right()
    } else {
        r.bottom()
    }
}

#[inline]
fn cpp_max(a: Scalar, b: Scalar) -> Scalar {
    if a < b { b } else { a }
}

/// `find_first_interval`.
fn find_first_interval(intervals: &[Scalar], mut phase: Scalar, index: &mut usize) -> Scalar {
    for (i, &gap) in intervals.iter().enumerate() {
        if phase > gap || (phase == gap && gap != 0.0) {
            phase -= gap;
        } else {
            *index = i;
            return gap - phase;
        }
    }
    // If we get here, phase "appears" to be larger than our length. This
    // shouldn't happen with perfect precision, but we can accumulate errors
    // during the initial length computation (rounding can make our sum be too
    // big or too small. In that event, we just have to eat the error here.
    *index = 0;
    intervals[0]
}

/// `SkDashPath::CalcDashParameters`.
///
/// Calculates `initial_dash_length`, `initial_dash_index` and `interval_length` from the phase and
/// intervals. When `adjusted_phase` is passed in, the phase is adjusted to be between 0 and
/// `interval_length` in place; otherwise the phase is assumed to be already in range.
pub fn calc_dash_parameters(
    mut phase: Scalar,
    intervals: &[Scalar],
    initial_dash_length: &mut Scalar,
    initial_dash_index: &mut usize,
    interval_length: &mut Scalar,
    adjusted_phase: Option<&mut Scalar>,
) {
    let mut len = 0.0;
    for &interval in intervals {
        len += interval;
    }
    *interval_length = len;
    // Adjust phase to be between 0 and len, "flipping" phase if negative.
    // e.g., if len is 100, then phase of -20 (or -120) is equivalent to 80
    if let Some(adjusted_phase) = adjusted_phase {
        if phase < 0.0 {
            phase = -phase;
            if phase > len {
                phase = scalar_mod(phase, len);
            }
            phase = len - phase;

            // Due to finite precision, it's possible that phase == len,
            // even after the subtract (if len >>> phase), so fix that here.
            // This fixes http://crbug.com/124652 .
            debug_assert!(phase <= len);
            if phase == len {
                phase = 0.0;
            }
        } else if phase >= len {
            phase = scalar_mod(phase, len);
        }
        *adjusted_phase = phase;
    }
    debug_assert!(phase >= 0.0 && phase < len);

    *initial_dash_length = find_first_interval(intervals, phase, initial_dash_index);

    debug_assert!(*initial_dash_length >= 0.0);
    debug_assert!(*initial_dash_index < intervals.len());
}

/// `outset_for_stroke`.
fn outset_for_stroke(rect: &mut Rect, rec: &SkStrokeRec) {
    let mut radius = rec.get_width() / 2.0;
    if 0.0 == radius {
        radius = crate::scalar::SCALAR_1; // hairlines
    }
    if crate::sk_paint_types::SkPaintJoin::Miter == rec.get_join() {
        radius *= rec.get_miter();
    }
    rect.outset(radius, radius);
}

/// If the line is zero-length, bump out the end by a tiny amount to draw endcaps. The bump factor
/// is sized so that `SkPoint::Distance()` computes a non-zero length. Offsets
/// `SK_ScalarNearlyZero` or smaller create empty paths when Iter measures length. Large values are
/// scaled by `SK_ScalarNearlyZero` so significant bits change.
fn adjust_zero_length_line(pts: &mut [Point; 2]) {
    debug_assert!(pts[0] == pts[1]);
    pts[1].x += cpp_max(1.001, pts[1].x) * SCALAR_NEARLY_ZERO;
}

/// `clip_line`.
fn clip_line(
    pts: &mut [Point; 2],
    bounds: &Rect,
    interval_length: Scalar,
    prior_phase: Scalar,
) -> bool {
    let dxy = pts[1] - pts[0];

    // only horizontal or vertical lines
    if dxy.x != 0.0 && dxy.y != 0.0 {
        return false;
    }
    let xy_offset = if dxy.y != 0.0 { 1 } else { 0 }; // 0 to adjust horizontal, 1 to adjust vertical

    let mut min_xy = xy_component(&pts[0], xy_offset);
    let mut max_xy = xy_component(&pts[1], xy_offset);
    let swapped = max_xy < min_xy;
    if swapped {
        std::mem::swap(&mut min_xy, &mut max_xy);
    }

    debug_assert!(min_xy <= max_xy);
    let left_top = rect_left_top(bounds, xy_offset);
    let right_bottom = rect_right_bottom(bounds, xy_offset);
    if max_xy < left_top || min_xy > right_bottom {
        return false;
    }

    // Now we actually perform the chop, removing the excess to the left/top and
    // right/bottom of the bounds (keeping our new line "in phase" with the dash,
    // hence the (mod intervalLength).
    if min_xy < left_top {
        min_xy = left_top - scalar_mod(left_top - min_xy, interval_length);
        if !swapped {
            min_xy -= prior_phase; // for rectangles, adjust by prior phase
        }
    }
    if max_xy > right_bottom {
        max_xy = right_bottom + scalar_mod(max_xy - right_bottom, interval_length);
        if swapped {
            max_xy += prior_phase; // for rectangles, adjust by prior phase
        }
    }

    debug_assert!(max_xy >= min_xy);
    if swapped {
        std::mem::swap(&mut min_xy, &mut max_xy);
    }
    set_xy_component(&mut pts[0], xy_offset, min_xy);
    set_xy_component(&mut pts[1], xy_offset, max_xy);

    if min_xy == max_xy {
        adjust_zero_length_line(pts);
    }
    true
}

/// Handles only lines and rects. If `cull_path` returns true the builder is the new smaller path,
/// otherwise the builder may have been changed but you should ignore it.
fn cull_path(
    src_path: &SkPath,
    rec: &SkStrokeRec,
    cull_rect: Option<&Rect>,
    interval_length: Scalar,
    builder: &mut SkPathBuilder,
) -> bool {
    let cull_rect = match cull_rect {
        None => {
            let mut pts = [Point::new(0.0, 0.0); 2];
            if src_path.is_line(Some(&mut pts)) && pts[0] == pts[1] {
                adjust_zero_length_line(&mut pts);
                builder.move_to(pts[0]);
                builder.line_to(pts[1]);
                return true;
            }
            return false;
        }
        Some(cull_rect) => cull_rect,
    };

    let mut bounds = *cull_rect;
    outset_for_stroke(&mut bounds, rec);

    {
        let mut pts = [Point::new(0.0, 0.0); 2];
        if src_path.is_line(Some(&mut pts)) {
            if clip_line(&mut pts, &bounds, interval_length, 0.0) {
                builder.move_to(pts[0]);
                builder.line_to(pts[1]);
                return true;
            }
            return false;
        }
    }

    if src_path.is_rect(None, None, None) {
        // We'll break the rect into four lines, culling each separately.
        let mut iter = Iter::from_path(src_path, false);

        let first = iter.next_rec();
        debug_assert!(first.is_some() && first.unwrap().f_verb == SkPathVerb::Move);

        // Sum of unculled edge lengths to keep the phase correct.
        // Intentionally a double to minimize the risk of overflow and drift.
        let mut accum: f64 = 0.0;
        loop {
            let (v, p0, p1) = match iter.next_rec() {
                Some(rec) if rec.f_verb == SkPathVerb::Line => (
                    rec.f_points[1] - rec.f_points[0],
                    rec.f_points[0],
                    rec.f_points[1],
                ),
                _ => break,
            };

            // Notice this vector v and accum work with the original unclipped length.
            let mut pts = [p0, p1];
            if clip_line(
                &mut pts,
                &bounds,
                interval_length,
                (accum % interval_length as f64) as Scalar,
            ) {
                // pts[0] may have just been changed by clip_line().
                // If that's not where we ended the previous lineTo(), we need to moveTo() there.
                let maybe_last = builder.get_last_pt();
                if maybe_last.is_none() || maybe_last.unwrap() != pts[0] {
                    builder.move_to(pts[0]);
                }
                builder.line_to(pts[1]);
            }

            // We either just traveled v.fX horizontally or v.fY vertically.
            debug_assert!(v.x == 0.0 || v.y == 0.0);
            accum += scalar_abs(v.x + v.y) as f64;
        }
        return !builder.is_empty();
    }

    false
}

/// `SpecialLineRec`.
struct SpecialLineRec {
    f_pts: [Point; 2],
    f_tangent: Vector,
    f_normal: Vector,
    f_path_length: Scalar,
}

impl SpecialLineRec {
    fn new() -> Self {
        SpecialLineRec {
            f_pts: [Point::new(0.0, 0.0); 2],
            f_tangent: Vector::new(0.0, 0.0),
            f_normal: Vector::new(0.0, 0.0),
            f_path_length: 0.0,
        }
    }

    fn init(
        &mut self,
        src: &SkPath,
        dst: &mut SkPathBuilder,
        rec: &mut SkStrokeRec,
        interval_count: usize,
        interval_length: Scalar,
    ) -> bool {
        if rec.is_hairline_style() || !src.is_line(Some(&mut self.f_pts)) {
            return false;
        }

        // can relax this in the future, if we handle square and round caps
        if SkPaintCap::Butt != rec.get_cap() {
            return false;
        }

        let path_length = Point::distance(&self.f_pts[0], &self.f_pts[1]);

        self.f_tangent = self.f_pts[1] - self.f_pts[0];
        if self.f_tangent.is_zero() {
            return false;
        }

        self.f_path_length = path_length;
        self.f_tangent
            .scale_in_place(ieee_float_divide(1.0, path_length));
        if !is_finite2(self.f_tangent.x, self.f_tangent.y) {
            return false;
        }
        PointPriv::rotate_ccw(&self.f_tangent, &mut self.f_normal);
        self.f_normal.scale_in_place(rec.get_width() / 2.0);

        // now estimate how many quads will be added to the path
        //     resulting segments = pathLen * intervalCount / intervalLen
        //     resulting points = 4 * segments
        let mut pt_count = path_length * interval_count as Scalar / interval_length;
        pt_count = if pt_count < K_MAX_DASH_COUNT {
            pt_count
        } else {
            K_MAX_DASH_COUNT
        };
        if pt_count.is_nan() {
            return false;
        }
        let n = (scalar_ceil_to_int(pt_count) << 2).max(0) as usize;
        dst.inc_reserve(n, n, 0);

        // we will take care of the stroking
        rec.set_fill_style();
        true
    }

    fn add_segment(&self, d0: Scalar, d1: Scalar, path: &mut SkPathBuilder) {
        let mut d1 = d1;
        debug_assert!(d0 <= self.f_path_length);
        // clamp the segment to our length
        if d1 > self.f_path_length {
            d1 = self.f_path_length;
        }

        let x0 = self.f_pts[0].x + self.f_tangent.x * d0;
        let x1 = self.f_pts[0].x + self.f_tangent.x * d1;
        let y0 = self.f_pts[0].y + self.f_tangent.y * d0;
        let y1 = self.f_pts[0].y + self.f_tangent.y * d1;

        let pts = [
            Point::new(x0 + self.f_normal.x, y0 + self.f_normal.y), // moveTo
            Point::new(x1 + self.f_normal.x, y1 + self.f_normal.y), // lineTo
            Point::new(x1 - self.f_normal.x, y1 - self.f_normal.y), // lineTo
            Point::new(x0 - self.f_normal.x, y0 - self.f_normal.y), // lineTo
        ];

        path.add_polygon(&pts, false);
    }
}

/// `SkDashPath::StrokeRecApplication`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeRecApplication {
    Disallow,
    Allow,
}

/// `SkDashPath::InternalFilter`.
///
/// The caller should have already used [`valid_dash_path`] to exclude invalid data. Typically this
/// leaves the stroke rec unmodified. However, for some simple shapes (e.g. a line) it may directly
/// evaluate the dash and stroke to produce a stroked output path with a fill stroke rec. Passing
/// [`StrokeRecApplication::Disallow`] turns this behavior off.
#[allow(clippy::too_many_arguments)]
pub fn internal_filter(
    dst: &mut SkPathBuilder,
    src: &SkPath,
    rec: &mut SkStrokeRec,
    cull_rect: Option<&Rect>,
    a_intervals: &[Scalar],
    initial_dash_length: Scalar,
    initial_dash_index: i32,
    interval_length: Scalar,
    start_phase: Scalar,
    stroke_rec_application: StrokeRecApplication,
) -> bool {
    let src_pts = src.points();
    if src_pts.is_empty() {
        return true;
    }
    let count = a_intervals.len();
    // we must always have an even number of intervals
    debug_assert!(is_even(count as i32));

    // we do nothing if the src wants to be filled
    let style = rec.get_style();
    if SkStrokeRecStyle::Fill == style || SkStrokeRecStyle::StrokeAndFill == style {
        return false;
    }

    let intervals = a_intervals;
    let mut dash_count: Scalar = 0.0;

    let mut builder = SkPathBuilder::new();
    let cull_path_storage: SkPath;
    let mut src_ptr: &SkPath = src;
    if cull_path(src, rec, cull_rect, interval_length, &mut builder) {
        // if rect is closed, starts in a dash, and ends in a dash, add the initial join
        // potentially a better fix is described here: skbug.com/40038693
        if src.is_rect(None, None, None)
            && src.is_last_contour_closed()
            && is_even(initial_dash_index)
        {
            let path_length =
                SkPathMeasure::from_path(src, false, rec.get_res_scale()).get_length();
            let mut end_phase = scalar_mod(path_length + start_phase, interval_length);
            let mut index = 0usize;
            while end_phase > intervals[index] {
                end_phase -= intervals[index];
                index += 1;
                debug_assert!(index <= count);
                if index == count {
                    // We have run out of intervals. endPhase "should" never get to this point,
                    // but it could if the subtracts underflowed. Hence we will pin it as if it
                    // perfectly ran through the intervals.
                    // See crbug.com/875494 (and skbug.com/40039544)
                    end_phase = 0.0;
                    break;
                }
            }
            // if dash ends inside "on", or ends at beginning of "off"
            if is_even(index as i32) == (end_phase > 0.0) {
                let mid_point = src_pts[0];
                // get vector at end of rect
                let mut last = src.count_points() - 1;
                while mid_point == src_pts[last as usize] {
                    last -= 1;
                    debug_assert!(last >= 0);
                }
                // get vector at start of rect
                let mut next = 1usize;
                while mid_point == src_pts[next] {
                    next += 1;
                    debug_assert!(next < last as usize);
                }
                let mut v = mid_point - src_pts[last as usize];
                let k_tiny_offset = SCALAR_NEARLY_ZERO;
                // scale vector to make start of tiny right angle
                v *= k_tiny_offset;
                builder.move_to(mid_point - v);
                builder.line_to(mid_point);
                v = mid_point - src_pts[next];
                // scale vector to make end of tiny right angle
                v *= k_tiny_offset;
                builder.line_to(mid_point - v);
            }
        }

        // If PathMeasure took a SkPathRaw, we could pass it the raw from src or the builder,
        // and not need to first 'detach' a path from the builder.
        cull_path_storage = builder.detach(None);
        src_ptr = &cull_path_storage;
    }

    let mut line_rec = SpecialLineRec::new();
    let special_line = (StrokeRecApplication::Allow == stroke_rec_application)
        && line_rec.init(src_ptr, dst, rec, count >> 1, interval_length);

    let mut meas = SkPathMeasure::from_path(src_ptr, false, rec.get_res_scale());

    loop {
        let mut skip_first_segment = meas.is_closed();
        let mut added_segment = false;
        let length = meas.get_length();
        let mut index = initial_dash_index;

        // Since the path length / dash length ratio may be arbitrarily large, we can exert
        // significant memory pressure while attempting to build the filtered path. To avoid this,
        // we simply give up dashing beyond a certain threshold.
        //
        // The original bug report (http://crbug.com/165432) is based on a path yielding more than
        // 90 million dash segments and crashing the memory allocator. A limit of 1 million
        // segments seems reasonable: at 2 verbs per segment * 9 bytes per verb, this caps the
        // maximum dash memory overhead at roughly 17MB per path.
        dash_count += length * (count >> 1) as Scalar / interval_length;
        if dash_count > K_MAX_DASH_COUNT {
            dst.reset();
            return false;
        }

        // Using double precision to avoid looping indefinitely due to single precision rounding
        // (for extreme path_length/dash_length ratios). See test_infinite_dash() unittest.
        let mut distance: f64 = 0.0;
        let mut dlen: f64 = initial_dash_length as f64;

        while distance < length as f64 {
            debug_assert!(dlen >= 0.0);
            added_segment = false;
            if is_even(index) && !skip_first_segment {
                added_segment = true;

                if special_line {
                    line_rec.add_segment(
                        double_to_scalar(distance),
                        double_to_scalar(distance + dlen),
                        dst,
                    );
                } else {
                    meas.get_segment(
                        double_to_scalar(distance),
                        double_to_scalar(distance + dlen),
                        dst,
                        true,
                    );
                }
            }
            distance += dlen;

            // clear this so we only respect it the first time around
            skip_first_segment = false;

            // wrap around our intervals array if necessary
            index += 1;
            debug_assert!(index as usize <= count);
            if index as usize == count {
                index = 0;
            }

            // fetch our next dlen
            dlen = intervals[index as usize] as f64;
        }

        // extend if we ended on a segment and we need to join up with the (skipped) initial segment
        if meas.is_closed() && is_even(initial_dash_index) && initial_dash_length >= 0.0 {
            meas.get_segment(0.0, initial_dash_length, dst, !added_segment);
        }

        if !meas.next_contour() {
            break;
        }
    }

    true
}

/// `SkDashPath::FilterDashPath`.
pub fn filter_dash_path(
    dst: &mut SkPathBuilder,
    src: &SkPath,
    rec: &mut SkStrokeRec,
    cull_rect: Option<&Rect>,
    info: &DashInfo,
) -> bool {
    if !valid_dash_path(info.f_phase, &info.f_intervals) {
        return false;
    }
    let mut initial_dash_length = 0.0;
    let mut initial_dash_index = 0usize;
    let mut interval_length = 0.0;
    calc_dash_parameters(
        info.f_phase,
        &info.f_intervals,
        &mut initial_dash_length,
        &mut initial_dash_index,
        &mut interval_length,
        None,
    );
    internal_filter(
        dst,
        src,
        rec,
        cull_rect,
        &info.f_intervals,
        initial_dash_length,
        initial_dash_index as i32,
        interval_length,
        info.f_phase,
        StrokeRecApplication::Allow,
    )
}

/// `SkDashPath::ValidDashPath`.
pub fn valid_dash_path(phase: Scalar, intervals: &[Scalar]) -> bool {
    if intervals.len() < 2 || (intervals.len() % 2) != 0 {
        return false;
    }
    let mut length = 0.0;
    for &interval in intervals {
        if interval < 0.0 {
            return false;
        }
        length += interval;
    }
    // watch out for values that might make us go out of bounds
    length > 0.0 && is_finite2(phase, length)
}
