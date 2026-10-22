//! Port of `include/core/SkContourMeasure.h` and `src/core/SkContourMeasure.cpp`
//! (plus `SkPathMeasure_segTo` from `src/core/SkPathMeasurePriv.h`).

use std::sync::Arc;

use crate::floating_point::{ieee_float_divide, is_finite, is_nan};
use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::scalar::Scalar;
use crate::sk_geometry::{
    SkConic, sk_chop_cubic_at, sk_chop_cubic_at_half, sk_chop_quad_at, sk_chop_quad_at_half,
    sk_eval_cubic_at, sk_eval_quad_at_with,
};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_types::SkPathVerb;

pub const K_MAX_T_VALUE: u32 = 0x3FFF_FFFF;

#[inline]
pub fn t_value_to_scalar(t: u32) -> Scalar {
    // 1/kMaxTValue can't be represented as a float, but it's close and the limits work fine.
    const K_MAX_T_RECIPROCAL: Scalar = 1.0 / K_MAX_T_VALUE as Scalar;
    t as Scalar * K_MAX_T_RECIPROCAL
}

/// `enum SkSegType` from `SkPathMeasurePriv.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkSegType {
    Line = 0,
    Quad = 1,
    Cubic = 2,
    Conic = 3,
}

/// `SkContourMeasure::Segment`.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub f_distance: Scalar,
    pub f_pt_index: u32,
    pub f_t_value: u32,
    pub f_type: SkSegType,
}

impl Segment {
    pub fn get_scalar_t(&self) -> Scalar {
        t_value_to_scalar(self.f_t_value)
    }

    /// `Segment::Next` — index of the first segment that follows the current verb.
    pub fn next_index(segs: &[Segment], index: usize) -> usize {
        let pt_index = segs[index].f_pt_index;
        let mut i = index;
        loop {
            i += 1;
            if i >= segs.len() || segs[i].f_pt_index != pt_index {
                break;
            }
        }
        i
    }
}

#[inline]
fn scalar_interp(a: Scalar, b: Scalar, t: Scalar) -> Scalar {
    a + (b - a) * t
}

/// `SkContourMeasure_segTo`.
pub fn sk_contour_measure_seg_to(
    pts: &[Point],
    seg_type: SkSegType,
    start_t: Scalar,
    stop_t: Scalar,
    dst: &mut SkPathBuilder,
) {
    debug_assert!((0.0..=1.0).contains(&start_t));
    debug_assert!((0.0..=1.0).contains(&stop_t));
    debug_assert!(start_t <= stop_t);

    if start_t == stop_t {
        if !dst.is_empty() {
            // If the dash has a zero-length on segment, add a corresponding zero-length line.
            // The stroke code will add end caps to zero length lines as appropriate.
            if let Some(last_pt) = dst.get_last_pt() {
                dst.line_to(last_pt);
            }
        }
        return;
    }

    match seg_type {
        SkSegType::Line => {
            if stop_t == 1.0 {
                dst.line_to(pts[1]);
            } else {
                dst.line_to_xy(
                    scalar_interp(pts[0].x, pts[1].x, stop_t),
                    scalar_interp(pts[0].y, pts[1].y, stop_t),
                );
            }
        }
        SkSegType::Quad => {
            let mut tmp0 = [Point::default(); 5];
            let mut tmp1 = [Point::default(); 5];
            if start_t == 0.0 {
                if stop_t == 1.0 {
                    dst.quad_to(pts[1], pts[2]);
                } else {
                    sk_chop_quad_at(pts, &mut tmp0, stop_t);
                    dst.quad_to(tmp0[1], tmp0[2]);
                }
            } else {
                sk_chop_quad_at(pts, &mut tmp0, start_t);
                if stop_t == 1.0 {
                    dst.quad_to(tmp0[3], tmp0[4]);
                } else {
                    sk_chop_quad_at(&tmp0[2..], &mut tmp1, (stop_t - start_t) / (1.0 - start_t));
                    dst.quad_to(tmp1[1], tmp1[2]);
                }
            }
        }
        SkSegType::Conic => {
            let conic = SkConic::new(pts[0], pts[2], pts[3], pts[1].x);

            if start_t == 0.0 {
                if stop_t == 1.0 {
                    dst.conic_to(conic.pts[1], conic.pts[2], conic.w);
                } else {
                    let mut tmp = [SkConic::default(); 2];
                    if conic.chop_at(stop_t, &mut tmp) {
                        dst.conic_to(tmp[0].pts[1], tmp[0].pts[2], tmp[0].w);
                    }
                }
            } else if stop_t == 1.0 {
                let mut tmp = [SkConic::default(); 2];
                if conic.chop_at(start_t, &mut tmp) {
                    dst.conic_to(tmp[1].pts[1], tmp[1].pts[2], tmp[1].w);
                }
            } else {
                let mut tmp = SkConic::default();
                conic.chop_at_two(start_t, stop_t, &mut tmp);
                dst.conic_to(tmp.pts[1], tmp.pts[2], tmp.w);
            }
        }
        SkSegType::Cubic => {
            let mut tmp0 = [Point::default(); 7];
            let mut tmp1 = [Point::default(); 7];
            if start_t == 0.0 {
                if stop_t == 1.0 {
                    dst.cubic_to(pts[1], pts[2], pts[3]);
                } else {
                    sk_chop_cubic_at(pts, &mut tmp0, stop_t);
                    dst.cubic_to(tmp0[1], tmp0[2], tmp0[3]);
                }
            } else {
                sk_chop_cubic_at(pts, &mut tmp0, start_t);
                if stop_t == 1.0 {
                    dst.cubic_to(tmp0[4], tmp0[5], tmp0[6]);
                } else {
                    sk_chop_cubic_at(&tmp0[3..], &mut tmp1, (stop_t - start_t) / (1.0 - start_t));
                    dst.cubic_to(tmp1[1], tmp1[2], tmp1[3]);
                }
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[inline]
fn tspan_big_enough(tspan: i32) -> bool {
    debug_assert!(tspan >= 0);
    tspan >> 10 != 0
}

/// `CHEAP_DIST_LIMIT`.
const CHEAP_DIST_LIMIT: Scalar = 0.5;

fn quad_too_curvy(pts: &[Point], tolerance: Scalar) -> bool {
    // diff = (a/4 + b/2 + c/4) - (a/2 + c/2)
    // diff = -a/4 + b/2 - c/4
    let dx = (pts[1].x / 2.0) - (((pts[0].x + pts[2].x) / 2.0) / 2.0);
    let dy = (pts[1].y / 2.0) - (((pts[0].y + pts[2].y) / 2.0) / 2.0);

    let dist = dx.abs().max(dy.abs());
    dist > tolerance
}

fn conic_too_curvy(first_pt: Point, mid_t_pt: Point, last_pt: Point, tolerance: Scalar) -> bool {
    let mut mid_ends = first_pt + last_pt;
    mid_ends *= 0.5;
    let dxy = mid_t_pt - mid_ends;
    let dist = dxy.x.abs().max(dxy.y.abs());
    dist > tolerance
}

fn cheap_dist_exceeds_limit(pt: Point, x: Scalar, y: Scalar, tolerance: Scalar) -> bool {
    let dist = (x - pt.x).abs().max((y - pt.y).abs());
    // just made up the 1/2
    dist > tolerance
}

fn cubic_too_curvy(pts: &[Point], tolerance: Scalar) -> bool {
    cheap_dist_exceeds_limit(
        pts[1],
        scalar_interp(pts[0].x, pts[3].x, 1.0 / 3.0),
        scalar_interp(pts[0].y, pts[3].y, 1.0 / 3.0),
        tolerance,
    ) || cheap_dist_exceeds_limit(
        pts[2],
        scalar_interp(pts[0].x, pts[3].x, 1.0 * 2.0 / 3.0),
        scalar_interp(pts[0].y, pts[3].y, 1.0 * 2.0 / 3.0),
        tolerance,
    )
}

// puts a cap on the total size of our output, since the client can pass in
// arbitrarily large values for resScale.
const K_MAX_RECURSION_DEPTH: i32 = 8;

/// `SkContourMeasure::MatrixFlags`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatrixFlags(pub u32);

impl MatrixFlags {
    pub const GET_POSITION: MatrixFlags = MatrixFlags(0x01);
    pub const GET_TANGENT: MatrixFlags = MatrixFlags(0x02);
    pub const GET_POS_AND_TAN: MatrixFlags = MatrixFlags(0x03);

    #[inline]
    pub fn contains(self, other: MatrixFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

/// `SkContourMeasure::VerbMeasure`.
#[derive(Clone, Copy, Debug)]
pub struct VerbMeasure<'a> {
    pub f_distance: Scalar,
    pub f_verb: SkPathVerb,
    pub f_pts: &'a [Point],
}

const SEG_PT_COUNT: [usize; 4] = [
    2, // kLine  (current_pt, 1 line pt)
    3, // kQuad  (current_pt, 2 quad pts)
    4, // kCubic (current_pt, 3 cubic pts)
    4, // kConic (current_pt, {weight, 0}, 2 conic pts)
];

const SEG_VERB: [SkPathVerb; 4] = [
    SkPathVerb::Line,
    SkPathVerb::Quad,
    SkPathVerb::Cubic,
    SkPathVerb::Conic,
];

/// `SkContourMeasure`.
#[derive(Clone, Debug)]
pub struct SkContourMeasure {
    pub f_segments: Vec<Segment>,
    pub f_pts: Vec<Point>, // Points used to define the segments
    pub f_length: Scalar,
    pub f_is_closed: bool,
}

impl SkContourMeasure {
    pub fn new(segs: Vec<Segment>, pts: Vec<Point>, length: Scalar, is_closed: bool) -> Self {
        SkContourMeasure {
            f_segments: segs,
            f_pts: pts,
            f_length: length,
            f_is_closed: is_closed,
        }
    }

    #[inline]
    pub fn length(&self) -> Scalar {
        self.f_length
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.f_is_closed
    }

    /// `SkContourMeasure::distanceToSegment`.
    pub fn distance_to_segment(&self, distance: Scalar, t: &mut Scalar) -> usize {
        debug_assert!(distance >= 0.0 && distance <= self.length());

        let seg = &self.f_segments;
        let count = seg.len();

        let mut index = sk_tk_search(seg, count, distance);
        // don't care if we hit an exact match or not, so we negate index if it is negative
        if index < 0 {
            index = !index;
        }
        let index = index as usize;

        // now interpolate t-values with the prev segment (if possible)
        let mut start_t = 0.0;
        let mut start_d = 0.0;
        // check if the prev segment is legal, and references the same set of points
        if index > 0 {
            start_d = seg[index - 1].f_distance;
            if seg[index - 1].f_pt_index == seg[index].f_pt_index {
                debug_assert!(seg[index - 1].f_type == seg[index].f_type);
                start_t = seg[index - 1].get_scalar_t();
            }
        }

        debug_assert!(seg[index].get_scalar_t() > start_t);
        debug_assert!(distance >= start_d);
        debug_assert!(seg[index].f_distance > start_d);

        *t = start_t
            + (seg[index].get_scalar_t() - start_t) * (distance - start_d)
                / (seg[index].f_distance - start_d);
        index
    }

    pub fn get_pos_tan(
        &self,
        distance: Scalar,
        position: Option<&mut Point>,
        tangent: Option<&mut Vector>,
    ) -> bool {
        if is_nan(distance) {
            return false;
        }

        let length = self.length();
        debug_assert!(length > 0.0 && !self.f_segments.is_empty());

        // pin the distance to a legal range
        let distance = scalar_pin(distance, 0.0, length);

        let mut t = 0.0;
        let seg_index = self.distance_to_segment(distance, &mut t);
        if is_nan(t) {
            return false;
        }

        debug_assert!((self.f_segments[seg_index].f_pt_index as usize) < self.f_pts.len());
        let pts = &self.f_pts[self.f_segments[seg_index].f_pt_index as usize..];
        compute_pos_tan(pts, self.f_segments[seg_index].f_type, t, position, tangent);
        true
    }

    pub fn get_matrix(
        &self,
        distance: Scalar,
        matrix: Option<&mut Matrix>,
        flags: MatrixFlags,
    ) -> bool {
        let mut position = Point::default();
        let mut tangent = Vector::default();

        if self.get_pos_tan(distance, Some(&mut position), Some(&mut tangent)) {
            if let Some(matrix) = matrix {
                if flags.contains(MatrixFlags::GET_TANGENT) {
                    matrix.set_sin_cos_pivot(tangent.y, tangent.x, 0.0, 0.0);
                } else {
                    matrix.reset();
                }
                if flags.contains(MatrixFlags::GET_POSITION) {
                    matrix.post_translate(position.x, position.y);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn get_segment(
        &self,
        start_d: Scalar,
        stop_d: Scalar,
        dst: &mut SkPathBuilder,
        start_with_move_to: bool,
    ) -> bool {
        let length = self.length();

        let start_d = if start_d < 0.0 { 0.0 } else { start_d };
        let stop_d = if stop_d > length { length } else { stop_d };
        if start_d.is_nan() || stop_d.is_nan() || start_d > stop_d {
            // catch NaN values as well
            return false;
        }
        if self.f_segments.is_empty() {
            return false;
        }

        let mut start_t = 0.0;
        let mut seg = self.distance_to_segment(start_d, &mut start_t);
        if !is_finite(start_t) {
            return false;
        }
        let mut stop_t = 0.0;
        let stop_seg = self.distance_to_segment(stop_d, &mut stop_t);
        if !is_finite(stop_t) {
            return false;
        }
        debug_assert!(seg <= stop_seg);

        if start_with_move_to {
            let mut p = Point::default();
            let pts = &self.f_pts[self.f_segments[seg].f_pt_index as usize..];
            compute_pos_tan(
                pts,
                self.f_segments[seg].f_type,
                start_t,
                Some(&mut p),
                None,
            );
            dst.move_to(p);
        }

        if self.f_segments[seg].f_pt_index == self.f_segments[stop_seg].f_pt_index {
            let pts = &self.f_pts[self.f_segments[seg].f_pt_index as usize..];
            sk_contour_measure_seg_to(pts, self.f_segments[seg].f_type, start_t, stop_t, dst);
        } else {
            let mut start = start_t;
            loop {
                let pts = &self.f_pts[self.f_segments[seg].f_pt_index as usize..];
                sk_contour_measure_seg_to(pts, self.f_segments[seg].f_type, start, 1.0, dst);
                seg = Segment::next_index(&self.f_segments, seg);
                start = 0.0;
                if self.f_segments[seg].f_pt_index >= self.f_segments[stop_seg].f_pt_index {
                    break;
                }
            }
            let pts = &self.f_pts[self.f_segments[seg].f_pt_index as usize..];
            sk_contour_measure_seg_to(pts, self.f_segments[seg].f_type, 0.0, stop_t, dst);
        }

        true
    }

    /// Iterator over the contour's verbs (`SkContourMeasure::begin`/`end`).
    pub fn iter(&self) -> ForwardVerbIterator<'_> {
        ForwardVerbIterator::new(&self.f_segments, &self.f_pts)
    }

    /// `ForwardVerbIterator::operator*`.
    pub fn verb_measure_at(&self, seg_index: usize) -> VerbMeasure<'_> {
        let seg = &self.f_segments[seg_index];
        let ty = seg.f_type as usize;
        VerbMeasure {
            f_distance: seg.f_distance,
            f_verb: SEG_VERB[ty],
            f_pts: &self.f_pts[seg.f_pt_index as usize..seg.f_pt_index as usize + SEG_PT_COUNT[ty]],
        }
    }
}

#[inline]
fn scalar_pin(v: Scalar, lo: Scalar, hi: Scalar) -> Scalar {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// `SkTKSearch` — index of `key`, or `!insertion_point` if not found.
fn sk_tk_search(base: &[Segment], count: usize, key: Scalar) -> i32 {
    debug_assert!(count as i32 >= 0);
    if count == 0 {
        return !0;
    }

    let mut lo: u32 = 0;
    let mut hi: u32 = count as u32 - 1;

    while lo < hi {
        let mid = (hi + lo) >> 1;
        if base[mid as usize].f_distance < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    if base[hi as usize].f_distance < key {
        hi += 1;
        !(hi as i32)
    } else if key < base[hi as usize].f_distance {
        !(hi as i32)
    } else {
        hi as i32
    }
}

fn normalize_tangent(tangent: &mut Option<&mut Vector>) {
    if let Some(tangent) = tangent.as_deref_mut() {
        tangent.normalize();
    }
}

fn compute_pos_tan(
    pts: &[Point],
    seg_type: SkSegType,
    t: Scalar,
    mut pos: Option<&mut Point>,
    mut tangent: Option<&mut Vector>,
) {
    match seg_type {
        SkSegType::Line => {
            if let Some(pos) = pos.as_deref_mut() {
                pos.set(
                    scalar_interp(pts[0].x, pts[1].x, t),
                    scalar_interp(pts[0].y, pts[1].y, t),
                );
            }
            if let Some(tangent) = tangent.as_deref_mut() {
                tangent.set_normalize(pts[1].x - pts[0].x, pts[1].y - pts[0].y);
            }
        }
        SkSegType::Quad => {
            let (pt, tan) = (pos.as_deref_mut(), tangent.as_deref_mut());
            sk_eval_quad_at_with(pts, t, pt, tan);
            normalize_tangent(&mut tangent);
        }
        SkSegType::Conic => {
            let conic = SkConic::new(pts[0], pts[2], pts[3], pts[1].x);
            let (pt, tan) = (pos.as_deref_mut(), tangent.as_deref_mut());
            conic.eval_at_with(t, pt, tan);
            normalize_tangent(&mut tangent);
        }
        SkSegType::Cubic => {
            let tan = tangent.as_deref_mut();
            sk_eval_cubic_at(pts, t, pos, tan, None);
            normalize_tangent(&mut tangent);
        }
    }
}

/// `SkContourMeasure::ForwardVerbIterator`.
pub struct ForwardVerbIterator<'a> {
    f_segments: &'a [Segment],
    f_pts: &'a [Point],
    index: usize,
    end: usize,
}

impl<'a> ForwardVerbIterator<'a> {
    fn new(segs: &'a [Segment], pts: &'a [Point]) -> Self {
        let index = Self::last_seg_for_current_verb(segs, 0);
        ForwardVerbIterator {
            f_segments: segs,
            f_pts: pts,
            index,
            end: segs.len(),
        }
    }

    fn last_seg_for_current_verb(segs: &[Segment], start: usize) -> usize {
        let mut i = start + 1;
        while i < segs.len() && segs[start].f_pt_index == segs[i].f_pt_index {
            i += 1;
        }
        i - 1
    }
}

impl<'a> Iterator for ForwardVerbIterator<'a> {
    type Item = VerbMeasure<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            return None;
        }
        let seg = &self.f_segments[self.index];
        let ty = seg.f_type as usize;
        let item = VerbMeasure {
            f_distance: seg.f_distance,
            f_verb: SEG_VERB[ty],
            f_pts: &self.f_pts[seg.f_pt_index as usize..seg.f_pt_index as usize + SEG_PT_COUNT[ty]],
        };
        // `operator++`: drop the current verb's segments, then land on the last
        // segment of the next verb.
        let start = self.index + 1;
        self.index = if start < self.end {
            Self::last_seg_for_current_verb(self.f_segments, start)
        } else {
            self.end
        };
        Some(item)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `SkContourMeasureIter`.
pub struct SkContourMeasureIter {
    f_verbs: Vec<SkPathVerb>,
    f_pts: Vec<Point>,
    f_weights: Vec<f32>,
    f_v_index: usize,
    f_p_index: usize,
    f_w_index: usize,
    f_tolerance: Scalar,
    f_force_closed: bool,
    f_impl: bool,

    // temporary (reused between contours)
    segments: Vec<Segment>,
    out_pts: Vec<Point>,
}

impl Default for SkContourMeasureIter {
    fn default() -> Self {
        Self::new()
    }
}

impl SkContourMeasureIter {
    pub fn new() -> Self {
        SkContourMeasureIter {
            f_verbs: Vec::new(),
            f_pts: Vec::new(),
            f_weights: Vec::new(),
            f_v_index: 0,
            f_p_index: 0,
            f_w_index: 0,
            f_tolerance: 0.0,
            f_force_closed: false,
            f_impl: false,
            segments: Vec::new(),
            out_pts: Vec::new(),
        }
    }

    pub fn from_path(path: &SkPath, force_closed: bool, res_scale: Scalar) -> Self {
        let mut iter = Self::new();
        iter.reset(path, force_closed, res_scale);
        iter
    }

    /// Assign a new path, or an empty path to have none.
    pub fn reset(&mut self, path: &SkPath, force_closed: bool, res_scale: Scalar) {
        if path.is_finite() {
            self.f_verbs = path.verbs().to_vec();
            self.f_pts = path.points().to_vec();
            self.f_weights = path.conic_weights().to_vec();
            self.f_v_index = 0;
            self.f_p_index = 0;
            self.f_w_index = 0;
            self.f_tolerance = CHEAP_DIST_LIMIT * ieee_float_divide(1.0, res_scale);
            self.f_force_closed = force_closed;
            self.f_impl = true;
        } else {
            self.f_verbs.clear();
            self.f_pts.clear();
            self.f_weights.clear();
            self.f_v_index = 0;
            self.f_p_index = 0;
            self.f_w_index = 0;
            self.f_impl = false;
        }
    }

    fn has_next_segments(&self) -> bool {
        self.f_v_index < self.f_verbs.len()
    }

    /// `SkPathPriv::RangeIter::operator*` equivalent for the current position.
    /// The points (including the verb's backset point) are copied out so that the
    /// caller can keep mutating `self`.
    fn peek(&self) -> Option<(SkPathVerb, [Point; 4], usize, f32)> {
        if self.f_v_index >= self.f_verbs.len() {
            return None;
        }
        let verb = self.f_verbs[self.f_v_index];
        let backset = if verb == SkPathVerb::Move { 0 } else { 1 };
        let start = self.f_p_index.saturating_sub(backset);
        let want = g_pts_per_verb(verb) + 1;
        let available = self.f_pts.len().saturating_sub(start);
        let len = want.min(available);
        let mut pts = [Point::default(); 4];
        pts[..len].copy_from_slice(&self.f_pts[start..start + len]);
        let weight = if verb == SkPathVerb::Conic {
            self.f_weights.get(self.f_w_index).copied().unwrap_or(1.0)
        } else {
            1.0
        };
        Some((verb, pts, len, weight))
    }

    fn advance(&mut self) {
        let verb = self.f_verbs[self.f_v_index];
        self.f_p_index += pts_advance_after_verb(verb);
        if verb == SkPathVerb::Conic {
            self.f_w_index += 1;
        }
        self.f_v_index += 1;
    }

    pub fn next(&mut self) -> Option<Arc<SkContourMeasure>> {
        if !self.f_impl {
            return None;
        }
        while self.has_next_segments() {
            if let Some(cm) = self.build_segments() {
                return Some(Arc::new(cm));
            }
        }
        None
    }

    fn compute_line_seg(
        &mut self,
        p0: Point,
        p1: Point,
        mut distance: Scalar,
        pt_index: u32,
    ) -> Scalar {
        let d = Point::distance(&p0, &p1);
        debug_assert!(d >= 0.0);
        let prev_d = distance;
        distance += d;
        if distance > prev_d {
            debug_assert!((pt_index as usize) < self.out_pts.len());
            self.segments.push(Segment {
                f_distance: distance,
                f_pt_index: pt_index,
                f_t_value: K_MAX_T_VALUE,
                f_type: SkSegType::Line,
            });
        }
        distance
    }

    fn compute_quad_segs(
        &mut self,
        pts: &[Point],
        mut distance: Scalar,
        mint: i32,
        maxt: i32,
        pt_index: u32,
        recursion_depth: i32,
    ) -> Scalar {
        if recursion_depth < K_MAX_RECURSION_DEPTH
            && tspan_big_enough(maxt - mint)
            && quad_too_curvy(pts, self.f_tolerance)
        {
            let mut tmp = [Point::default(); 5];
            let halft = (mint + maxt) >> 1;

            sk_chop_quad_at_half(pts, &mut tmp);
            let depth = recursion_depth + 1;
            distance = self.compute_quad_segs(&tmp[0..3], distance, mint, halft, pt_index, depth);
            distance = self.compute_quad_segs(&tmp[2..5], distance, halft, maxt, pt_index, depth);
        } else {
            let d = Point::distance(&pts[0], &pts[2]);
            let prev_d = distance;
            distance += d;
            if distance > prev_d {
                debug_assert!((pt_index as usize) < self.out_pts.len());
                self.segments.push(Segment {
                    f_distance: distance,
                    f_pt_index: pt_index,
                    f_t_value: maxt as u32,
                    f_type: SkSegType::Quad,
                });
            }
        }
        distance
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_conic_segs(
        &mut self,
        conic: &SkConic,
        mut distance: Scalar,
        mint: i32,
        min_pt: Point,
        maxt: i32,
        max_pt: Point,
        pt_index: u32,
        recursion_depth: i32,
    ) -> Scalar {
        let halft = (mint + maxt) >> 1;
        let half_pt = conic.eval_at(t_value_to_scalar(halft as u32));
        if !half_pt.is_finite() {
            return distance;
        }
        if recursion_depth < K_MAX_RECURSION_DEPTH
            && tspan_big_enough(maxt - mint)
            && conic_too_curvy(min_pt, half_pt, max_pt, self.f_tolerance)
        {
            let depth = recursion_depth + 1;
            distance = self.compute_conic_segs(
                conic, distance, mint, min_pt, halft, half_pt, pt_index, depth,
            );
            distance = self.compute_conic_segs(
                conic, distance, halft, half_pt, maxt, max_pt, pt_index, depth,
            );
        } else {
            let d = Point::distance(&min_pt, &max_pt);
            let prev_d = distance;
            distance += d;
            if distance > prev_d {
                debug_assert!((pt_index as usize) < self.out_pts.len());
                self.segments.push(Segment {
                    f_distance: distance,
                    f_pt_index: pt_index,
                    f_t_value: maxt as u32,
                    f_type: SkSegType::Conic,
                });
            }
        }
        distance
    }

    fn compute_cubic_segs(
        &mut self,
        pts: &[Point],
        mut distance: Scalar,
        mint: i32,
        maxt: i32,
        pt_index: u32,
        recursion_depth: i32,
    ) -> Scalar {
        if recursion_depth < K_MAX_RECURSION_DEPTH
            && tspan_big_enough(maxt - mint)
            && cubic_too_curvy(pts, self.f_tolerance)
        {
            let mut tmp = [Point::default(); 7];
            let halft = (mint + maxt) >> 1;

            sk_chop_cubic_at_half(pts, &mut tmp);
            let depth = recursion_depth + 1;
            distance = self.compute_cubic_segs(&tmp[0..4], distance, mint, halft, pt_index, depth);
            distance = self.compute_cubic_segs(&tmp[3..7], distance, halft, maxt, pt_index, depth);
        } else {
            let d = Point::distance(&pts[0], &pts[3]);
            let prev_d = distance;
            distance += d;
            if distance > prev_d {
                debug_assert!((pt_index as usize) < self.out_pts.len());
                self.segments.push(Segment {
                    f_distance: distance,
                    f_pt_index: pt_index,
                    f_t_value: maxt as u32,
                    f_type: SkSegType::Cubic,
                });
            }
        }
        distance
    }

    fn build_segments(&mut self) -> Option<SkContourMeasure> {
        let mut pt_index: i32 = -1;
        let mut distance: Scalar = 0.0;
        let mut have_seen_close = self.f_force_closed;
        let mut have_seen_move_to = false;

        self.segments.clear();
        self.out_pts.clear();

        while self.has_next_segments() {
            let (verb, pts, n_pts, w) = self.peek().unwrap();
            let pts = &pts[..n_pts];
            if have_seen_move_to && verb == SkPathVerb::Move {
                break;
            }
            match verb {
                SkPathVerb::Move => {
                    pt_index += 1;
                    self.out_pts.push(pts[0]);
                    debug_assert!(!have_seen_move_to);
                    have_seen_move_to = true;
                }
                SkPathVerb::Line => {
                    debug_assert!(have_seen_move_to);
                    let prev_d = distance;
                    distance = self.compute_line_seg(pts[0], pts[1], distance, pt_index as u32);
                    if distance > prev_d {
                        self.out_pts.push(pts[1]);
                        pt_index += 1;
                    }
                }
                SkPathVerb::Quad => {
                    debug_assert!(have_seen_move_to);
                    let prev_d = distance;
                    distance = self.compute_quad_segs(
                        pts,
                        distance,
                        0,
                        K_MAX_T_VALUE as i32,
                        pt_index as u32,
                        0,
                    );
                    if distance > prev_d {
                        self.out_pts.push(pts[1]);
                        self.out_pts.push(pts[2]);
                        pt_index += 2;
                    }
                }
                SkPathVerb::Conic => {
                    debug_assert!(have_seen_move_to);
                    let conic = SkConic::from_pts(pts, w);
                    let prev_d = distance;
                    distance = self.compute_conic_segs(
                        &conic,
                        distance,
                        0,
                        conic.pts[0],
                        K_MAX_T_VALUE as i32,
                        conic.pts[2],
                        pt_index as u32,
                        0,
                    );
                    if distance > prev_d {
                        // we store the conic weight in our next point, followed by the last 2 pts
                        // thus to reconstitute a conic, you'd need to say
                        // SkConic(pts[0], pts[2], pts[3], weight = pts[1].x)
                        self.out_pts.push(Point::new(conic.w, 0.0));
                        self.out_pts.push(pts[1]);
                        self.out_pts.push(pts[2]);
                        pt_index += 3;
                    }
                }
                SkPathVerb::Cubic => {
                    debug_assert!(have_seen_move_to);
                    let prev_d = distance;
                    distance = self.compute_cubic_segs(
                        pts,
                        distance,
                        0,
                        K_MAX_T_VALUE as i32,
                        pt_index as u32,
                        0,
                    );
                    if distance > prev_d {
                        self.out_pts.push(pts[1]);
                        self.out_pts.push(pts[2]);
                        self.out_pts.push(pts[3]);
                        pt_index += 3;
                    }
                }
                SkPathVerb::Close => {
                    have_seen_close = true;
                }
                SkPathVerb::Done => {}
            }
            self.advance();
        }

        if !is_finite(distance) {
            return None;
        }
        if self.segments.is_empty() {
            return None;
        }

        if have_seen_close {
            let prev_d = distance;
            let first_pt = self.out_pts[0];
            distance = self.compute_line_seg(
                self.out_pts[pt_index as usize],
                first_pt,
                distance,
                pt_index as u32,
            );
            if distance > prev_d {
                self.out_pts.push(first_pt);
            }
        }

        Some(SkContourMeasure::new(
            self.segments.clone(),
            self.out_pts.clone(),
            distance,
            have_seen_close,
        ))
    }
}

fn g_pts_per_verb(v: SkPathVerb) -> usize {
    match v {
        SkPathVerb::Move => 1,
        SkPathVerb::Line => 1,
        SkPathVerb::Quad => 2,
        SkPathVerb::Conic => 2,
        SkPathVerb::Cubic => 3,
        SkPathVerb::Close => 0,
        SkPathVerb::Done => 0,
    }
}

fn pts_advance_after_verb(v: SkPathVerb) -> usize {
    g_pts_per_verb(v)
}

/// `SkPathMeasurePriv`.
pub struct SkPathMeasurePriv;

impl SkPathMeasurePriv {
    pub fn count_segments(meas: &crate::sk_path_measure::SkPathMeasure) -> usize {
        match meas.current_measure() {
            Some(cm) => cm.f_segments.len(),
            None => 0,
        }
    }
}
