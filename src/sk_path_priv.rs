//! Port of `src/core/SkPathPriv.cpp` (and the pure helpers declared in
//! `src/core/SkPathPriv.h`). Functions requiring `SkPath`/`SkPathData`/`SkPathBuilder`
//! are implemented on the `SkPathPriv` struct where they can reference those modules.

use crate::matrix::Matrix;
use crate::point::Point;
use crate::point_priv::PointPriv;
use crate::rect::Rect;
use crate::rrect::{Corner, RRect};
use crate::scalar::{Scalar, scalar_nearly_equal, scalar_nearly_zero, scalar_sign_as_int};
use crate::sk_cubic_clipper::SkCubicClipper;
use crate::sk_geometry::{
    SkConic, sk_chop_cubic_at_y_extrema, sk_chop_quad_at_y_extrema, sk_eval_cubic_at,
    sk_eval_quad_at, sk_eval_quad_tangent_at, sk_find_cubic_extrema, sk_find_quad_extrema,
    sk_find_unit_quad_roots,
};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_edge_iter::{SkPathEdge, SkPathEdgeIter};
use crate::sk_path_iter::SkPathIter;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_types::{
    SkPathConvexity, SkPathDirection, SkPathFillType, SkPathFirstDirection, SkPathVerb,
    SkResolveConvexity, sk_path_convexity_is_convex, sk_path_convexity_opposite_convex_direction,
    sk_path_first_direction_to_convexity,
};

/// `SkPathPriv::RRectAsEnum`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RRectAsEnum {
    Rect,
    Oval,
    RRect,
}

/// `SkPathPriv::RectContour`.
#[derive(Clone, Copy, Debug)]
pub struct RectContour {
    pub f_rect: Rect,
    pub f_is_closed: bool,
    pub f_direction: SkPathDirection,
    pub f_points_consumed: usize,
    pub f_verbs_consumed: usize,
}

#[inline]
pub fn pts_in_verb(verb: SkPathVerb) -> i32 {
    const TABLE: [i32; 7] = [1, 1, 2, 2, 3, 0, 0];
    TABLE[verb as usize]
}

#[inline]
pub fn pts_in_iter(verb: SkPathVerb) -> i32 {
    const TABLE: [i32; 7] = [1, 2, 3, 3, 4, 0, 0];
    TABLE[verb as usize]
}

#[inline]
fn segment_mask_for_verb(verb: SkPathVerb) -> u8 {
    const TABLE: [u8; 6] = [
        0,
        crate::sk_path_types::K_LINE_SK_PATH_SEGMENT_MASK,
        crate::sk_path_types::K_QUAD_SK_PATH_SEGMENT_MASK,
        crate::sk_path_types::K_CONIC_SK_PATH_SEGMENT_MASK,
        crate::sk_path_types::K_CUBIC_SK_PATH_SEGMENT_MASK,
        0,
    ];
    TABLE[verb as usize]
}

/// `SkPathPriv::ComputeSegmentMask`.
pub fn compute_segment_mask(verbs: &[SkPathVerb]) -> u8 {
    let mut mask = 0u8;
    for &v in verbs {
        mask |= segment_mask_for_verb(v);
    }
    mask
}

/// `SkPathPriv::TrimmedBounds`. Returns `None` when non-finite.
pub fn trimmed_bounds(pts: &[Point], vbs: &[SkPathVerb]) -> Option<Rect> {
    let mut pts = pts;
    if vbs.len() > 1 && *vbs.last().unwrap() == SkPathVerb::Move {
        debug_assert!(!pts.is_empty());
        if !pts.last().unwrap().is_finite() {
            return None;
        }
        pts = &pts[..pts.len() - 1];
    }
    Rect::bounds(pts)
}

/// `SkPathPriv::IsAxisAligned`.
pub fn is_axis_aligned(pts: &[Point]) -> bool {
    for i in 1..pts.len() {
        if pts[i - 1].x != pts[i].x && pts[i - 1].y != pts[i].y {
            return false;
        }
    }
    true
}

/// Local helper mirroring `SkRect::set(a, b)` (min/max sorting of two points).
#[inline]
fn rect_set_points(a: Point, b: Point) -> Rect {
    Rect::new(a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y))
}

#[inline]
fn rect_make_dir(dx: Scalar, dy: Scalar) -> i32 {
    ((dx != 0.0) as i32) | (((dx > 0.0) || (dy > 0.0)) as i32) << 1
}

#[inline]
fn axis_aligned_orthogonal(a: &Point, b: &Point) -> i32 {
    (((a.x == 0.0) ^ (b.x == 0.0)) as i32) & (((a.y == 0.0) ^ (b.y == 0.0)) as i32)
}

fn trivial_rect(pts: &[Point], vbs: &[SkPathVerb]) -> Option<RectContour> {
    if pts.len() != 4 || vbs.len() != 5 {
        return None;
    }
    if vbs
        != [
            SkPathVerb::Move,
            SkPathVerb::Line,
            SkPathVerb::Line,
            SkPathVerb::Line,
            SkPathVerb::Close,
        ]
    {
        return None;
    }
    let v0 = pts[1] - pts[0];
    let v1 = pts[2] - pts[1];
    let v2 = pts[3] - pts[2];
    let v3 = pts[0] - pts[3];
    if !(((v0.x == 0.0) ^ (v0.y == 0.0))
        & (axis_aligned_orthogonal(&v0, &v1) != 0)
        & (axis_aligned_orthogonal(&v1, &v2) != 0)
        & (axis_aligned_orthogonal(&v2, &v3) != 0))
    {
        return None;
    }
    let rect = Rect::new(pts[0].x, pts[0].y, pts[2].x, pts[2].y).make_sorted();
    let dir = if Point::cross_product(&v0, &v1) > 0.0 {
        SkPathDirection::Cw
    } else {
        SkPathDirection::Ccw
    };
    Some(RectContour {
        f_rect: rect,
        f_is_closed: true,
        f_direction: dir,
        f_points_consumed: 4,
        f_verbs_consumed: 5,
    })
}

/// `SkPathPriv::IsRectContour`.
pub fn is_rect_contour(
    pt_span: &[Point],
    vb_span: &[SkPathVerb],
    segment_mask: u32,
    allow_partial: bool,
) -> Option<RectContour> {
    if segment_mask != crate::sk_path_types::K_LINE_SK_PATH_SEGMENT_MASK as u32
        || pt_span.len() < 4
        || vb_span.len() < 4
    {
        return None;
    }
    if let Some(rc) = trivial_rect(pt_span, vb_span) {
        return Some(rc);
    }

    let pts = pt_span;
    let vbs = vb_span;
    let verb_count = vbs.len();

    let mut curr_verb = 0usize;
    let mut directions = [-1i32; 5];
    let mut corners = 0i32;
    let mut closed_or_moved = false;
    let mut auto_close = false;
    let mut insert_close = false;
    let mut pi = 0usize;
    let mut save_pts: Option<usize> = None;
    let mut first_pt: Option<usize> = None;
    let mut last_pt: Option<usize> = None;
    let mut first_corner = Point::new(0.0, 0.0);
    let mut third_corner = Point::new(0.0, 0.0);
    let mut line_start = Point::new(0.0, 0.0);

    while curr_verb < verb_count && (!allow_partial || !auto_close) {
        let is_close = insert_close;
        let verb = if is_close {
            SkPathVerb::Close
        } else {
            vbs[curr_verb]
        };
        if is_close {
            insert_close = false;
        }

        if verb == SkPathVerb::Close {
            save_pts = Some(pi);
            auto_close = true;
        }

        if verb == SkPathVerb::Close || verb == SkPathVerb::Line {
            if verb != SkPathVerb::Close {
                last_pt = Some(pi);
            }
            let line_end = if verb == SkPathVerb::Close {
                pts[first_pt.unwrap()]
            } else {
                let e = pts[pi];
                pi += 1;
                e
            };
            let delta = line_end - line_start;
            if delta.x != 0.0 && delta.y != 0.0 {
                return None;
            }
            if !delta.is_finite() {
                return None;
            }
            if line_start == line_end {
                curr_verb += 1;
                continue;
            }
            let next = rect_make_dir(delta.x, delta.y);
            if corners == 0 {
                directions[0] = next;
                corners = 1;
                closed_or_moved = false;
                line_start = line_end;
                curr_verb += 1;
                continue;
            }
            if closed_or_moved {
                return None;
            }
            if auto_close && next == directions[0] {
                curr_verb += 1;
                continue;
            }
            closed_or_moved = auto_close;
            if directions[(corners - 1) as usize] == next {
                if corners == 3 && verb == SkPathVerb::Line {
                    third_corner = line_end;
                }
                line_start = line_end;
                curr_verb += 1;
                continue;
            }
            directions[corners as usize] = next;
            corners += 1;
            match corners {
                2 => first_corner = line_start,
                3 => {
                    if (directions[0] ^ directions[2]) != 2 {
                        return None;
                    }
                    third_corner = line_end;
                }
                4 => {
                    if (directions[1] ^ directions[3]) != 2 {
                        return None;
                    }
                }
                _ => return None,
            }
            line_start = line_end;
            curr_verb += 1;
            continue;
        }

        match verb {
            SkPathVerb::Quad | SkPathVerb::Conic | SkPathVerb::Cubic => return None,
            SkPathVerb::Move => {
                if allow_partial && !auto_close && directions[0] >= 0 {
                    insert_close = true;
                    curr_verb -= 1;
                    continue;
                }
                if corners == 0 {
                    first_pt = Some(pi);
                } else {
                    let close_xy = pts[first_pt.unwrap()] - pts[last_pt.unwrap()];
                    if close_xy.x != 0.0 && close_xy.y != 0.0 {
                        return None;
                    }
                }
                line_start = pts[pi];
                pi += 1;
                closed_or_moved = true;
            }
            _ => {}
        }
        curr_verb += 1;
    }

    if !(3..=4).contains(&corners) {
        return None;
    }
    let close_xy = pts[first_pt.unwrap()] - pts[last_pt.unwrap()];
    if close_xy.x != 0.0 && close_xy.y != 0.0 {
        return None;
    }
    let frect = rect_set_points(first_corner, third_corner);
    let dir = if directions[0] == ((directions[1] + 1) & 3) {
        SkPathDirection::Cw
    } else {
        SkPathDirection::Ccw
    };
    Some(RectContour {
        f_rect: frect,
        f_is_closed: auto_close,
        f_direction: dir,
        f_points_consumed: save_pts.unwrap_or(0),
        f_verbs_consumed: curr_verb,
    })
}

/// `SkPathPriv::IsNestedFillRects`.
pub fn is_nested_fill_rects(
    raw: &SkPathRaw,
    rect: Option<&mut [Rect; 2]>,
    dirs: Option<&mut [SkPathDirection; 2]>,
) -> bool {
    let pts = &raw.f_points;
    let vbs = &raw.f_verbs;
    let mask = raw.f_segment_mask as u32;

    let mut test_dirs = [SkPathDirection::Cw; 2];
    let mut test_rects = [Rect::make_empty(); 2];

    let rc = match is_rect_contour(pts, vbs, mask, true) {
        Some(rc) => rc,
        None => return false,
    };
    test_dirs[0] = rc.f_direction;
    test_rects[0] = rc.f_rect;

    let sub_pts = &pts[rc.f_points_consumed..];
    let sub_vbs = &vbs[rc.f_verbs_consumed..];
    if let Some(rc2) = is_rect_contour(sub_pts, sub_vbs, mask, false) {
        test_dirs[1] = rc2.f_direction;
        test_rects[1] = rc2.f_rect;
        if test_rects[0].contains_rect(&test_rects[1]) {
            if let Some(rect) = rect {
                rect[0] = test_rects[0];
                rect[1] = test_rects[1];
            }
            if let Some(dirs) = dirs {
                dirs[0] = test_dirs[0];
                dirs[1] = test_dirs[1];
            }
            return true;
        }
        if test_rects[1].contains_rect(&test_rects[0]) {
            if let Some(rect) = rect {
                rect[0] = test_rects[1];
                rect[1] = test_rects[0];
            }
            if let Some(dirs) = dirs {
                dirs[0] = test_dirs[1];
                dirs[1] = test_dirs[0];
            }
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// Convexity
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum DirChange {
    Unknown,
    Left,
    Right,
    Straight,
    Backwards,
    Invalid,
}

struct Convexicator {
    f_first_pt: Point,
    f_first_vec: Point,
    f_last_pt: Point,
    f_last_vec: Point,
    f_expected_dir: DirChange,
    f_first_direction: SkPathFirstDirection,
    f_reversals: i32,
    f_is_finite: bool,
}

impl Default for Convexicator {
    fn default() -> Self {
        Convexicator {
            f_first_pt: Point::new(0.0, 0.0),
            f_first_vec: Point::new(0.0, 0.0),
            f_last_pt: Point::new(0.0, 0.0),
            f_last_vec: Point::new(0.0, 0.0),
            f_expected_dir: DirChange::Invalid,
            f_first_direction: SkPathFirstDirection::Unknown,
            f_reversals: 0,
            f_is_finite: true,
        }
    }
}

impl Convexicator {
    fn set_move_pt(&mut self, pt: Point) {
        self.f_first_pt = pt;
        self.f_last_pt = pt;
        self.f_expected_dir = DirChange::Invalid;
    }

    fn add_pt(&mut self, pt: Point) -> bool {
        if self.f_last_pt == pt {
            return true;
        }
        if self.f_first_pt == self.f_last_pt
            && self.f_expected_dir == DirChange::Invalid
            && self.f_last_vec == Point::new(0.0, 0.0)
        {
            self.f_last_vec = pt - self.f_last_pt;
            self.f_first_vec = self.f_last_vec;
        } else if !self.add_vec(pt - self.f_last_pt) {
            return false;
        }
        self.f_last_pt = pt;
        true
    }

    fn close(&mut self) -> bool {
        self.add_pt(self.f_first_pt) && self.add_vec(self.f_first_vec)
    }

    fn direction_change(&self, cur: Point) -> DirChange {
        let cross = Point::cross_product(&self.f_last_vec, &cur);
        if !cross.is_finite() {
            return DirChange::Unknown;
        }
        if cross == 0.0 {
            if self.f_last_vec.dot(&cur) < 0.0 {
                DirChange::Backwards
            } else {
                DirChange::Straight
            }
        } else if scalar_sign_as_int(cross) == 1 {
            DirChange::Right
        } else {
            DirChange::Left
        }
    }

    fn add_vec(&mut self, cur: Point) -> bool {
        match self.direction_change(cur) {
            DirChange::Left | DirChange::Right => {
                let dir = self.direction_change(cur);
                if self.f_expected_dir == DirChange::Invalid {
                    self.f_expected_dir = dir;
                    self.f_first_direction = if dir == DirChange::Right {
                        SkPathFirstDirection::Cw
                    } else {
                        SkPathFirstDirection::Ccw
                    };
                } else if dir != self.f_expected_dir {
                    self.f_first_direction = SkPathFirstDirection::Unknown;
                    return false;
                }
                self.f_last_vec = cur;
            }
            DirChange::Straight => {}
            DirChange::Backwards => {
                self.f_last_vec = cur;
                self.f_reversals += 1;
                return self.f_reversals < 3;
            }
            DirChange::Unknown => {
                self.f_is_finite = false;
                return false;
            }
            DirChange::Invalid => unreachable!(),
        }
        true
    }
}

fn is_concave_by_sign(points: &[Point]) -> bool {
    if points.len() <= 3 {
        return false;
    }
    let sign = |x: Scalar| -> i32 { (x < 0.0) as i32 };

    let mut curr = points[0];
    let mut dxes = 0;
    let mut dyes = 0;
    let mut last_sx = 2;
    let mut last_sy = 2;

    for outer in 0..2 {
        // The second pass rewinds to the first point but keeps `curr` at the last point, so the
        // wrap-around vector is measured too.
        let mut i = if outer == 0 { 1 } else { 0 };
        while i < points.len() {
            let vec = points[i] - curr;
            if !vec.is_zero() {
                if !vec.is_finite() {
                    return true;
                }
                let sx = sign(vec.x);
                let sy = sign(vec.y);
                if sx != last_sx {
                    dxes += 1;
                }
                if sy != last_sy {
                    dyes += 1;
                }
                if dxes > 3 || dyes > 3 {
                    return true;
                }
                last_sx = sx;
                last_sy = sy;
            }
            curr = points[i];
            i += 1;
            if outer == 1 {
                break;
            }
        }
    }
    false
}

fn trim_trailing_moves(pts: &mut &[Point], vbs: &mut &[SkPathVerb]) {
    let mut n_pts = pts.len();
    let mut n_vbs = vbs.len();
    while n_vbs > 0 && vbs[n_vbs - 1] == SkPathVerb::Move {
        n_vbs -= 1;
        n_pts = n_pts.saturating_sub(1);
    }
    *pts = &pts[..n_pts];
    *vbs = &vbs[..n_vbs];
}

/// `SkPathPriv::ComputeConvexity`.
pub fn compute_convexity(
    points: &[Point],
    verbs: &[SkPathVerb],
    conics: &[f32],
) -> SkPathConvexity {
    let mut points = points;
    let mut verbs = verbs;

    trim_trailing_moves(&mut points, &mut verbs);

    if verbs.is_empty() {
        return SkPathConvexity::ConvexDegenerate;
    }
    if is_concave_by_sign(points) {
        return SkPathConvexity::Concave;
    }

    let mut contour_count = 0;
    let mut needs_close = false;
    let mut state = Convexicator::default();
    let mut iter = SkPathIter::new(points, verbs, conics);
    while let Some(rec) = iter.next() {
        let verb = rec.f_verb;
        let rec_pts = rec.f_points;
        if contour_count == 0 {
            if verb == SkPathVerb::Move {
                state.set_move_pt(rec_pts[0]);
            } else {
                contour_count += 1;
                needs_close = true;
            }
        }
        if contour_count == 1 {
            if verb == SkPathVerb::Close || verb == SkPathVerb::Move {
                if !state.close() {
                    return SkPathConvexity::Concave;
                }
                needs_close = false;
                contour_count += 1;
            } else {
                let count = pts_in_verb(verb) as usize;
                for i in 1..=count {
                    if !state.add_pt(rec_pts[i]) {
                        return SkPathConvexity::Concave;
                    }
                }
            }
        } else if verb != SkPathVerb::Move {
            return SkPathConvexity::Concave;
        }
    }
    if needs_close && !state.close() {
        return SkPathConvexity::Concave;
    }
    let first_dir = state.f_first_direction;
    if first_dir == SkPathFirstDirection::Unknown && state.f_reversals >= 3 {
        return SkPathConvexity::Concave;
    }
    sk_path_first_direction_to_convexity(first_dir)
}

// ---------------------------------------------------------------------------------------------
// First direction
// ---------------------------------------------------------------------------------------------

fn cross_prod(p0: Point, p1: Point, p2: Point) -> Scalar {
    let c = Point::cross_product(&(p1 - p0), &(p2 - p0));
    if c != 0.0 {
        return c;
    }
    ((p1.x as f64 - p0.x as f64) * (p2.y as f64 - p0.y as f64)
        - (p1.y as f64 - p0.y as f64) * (p2.x as f64 - p0.x as f64)) as Scalar
}

fn find_max_y(pts: &[Point]) -> usize {
    let mut index = 0;
    for i in 1..pts.len() {
        if pts[index].y < pts[i].y {
            index = i;
        }
    }
    index
}

fn find_diff_pt(pts: &[Point], index: usize, n: usize, inc: i32) -> usize {
    let mut i = index;
    loop {
        i = ((i as i32 + inc).rem_euclid(n as i32)) as usize;
        if i == index {
            break;
        }
        if pts[index] != pts[i] {
            break;
        }
    }
    i
}

fn find_min_max_x_at_y(pts: &[Point], index: usize, n: usize) -> (usize, usize) {
    let y = pts[index].y;
    let mut min_index = index;
    let mut max_index = index;
    let mut min = pts[index].x;
    let mut max = pts[index].x;
    for i in index + 1..n {
        if pts[i].y != y {
            break;
        }
        let x = pts[i].x;
        if x < min {
            min = x;
            min_index = i;
        } else if x > max {
            max = x;
            max_index = i;
        }
    }
    (min_index, max_index)
}

fn cross_to_dir(c: Scalar) -> SkPathFirstDirection {
    if c > 0.0 {
        SkPathFirstDirection::Cw
    } else {
        SkPathFirstDirection::Ccw
    }
}

/// `SkPathPriv::ComputeFirstDirection`.
pub fn compute_first_direction(raw: &SkPathRaw) -> SkPathFirstDirection {
    let pts_all = &raw.f_points;
    let vbs = &raw.f_verbs;
    let mut ymax = raw.f_bounds.top();
    let mut ymax_cross: Scalar = 0.0;

    let mut verb = 0usize;
    while verb < vbs.len() {
        // Determine contour [verb, end)
        let base = verb;
        let mut count = 1usize; // move point
        let start = if vbs[verb] == SkPathVerb::Move {
            verb + 1
        } else {
            verb
        };
        let mut j = start;
        while j < vbs.len() {
            match vbs[j] {
                SkPathVerb::Move => break,
                SkPathVerb::Line => count += 1,
                SkPathVerb::Conic => count += 2,
                SkPathVerb::Quad => count += 2,
                SkPathVerb::Cubic => count += 3,
                SkPathVerb::Close => {}
                SkPathVerb::Done => break,
            }
            j += 1;
        }
        // point base index
        let pt_base: usize = vbs[..base].iter().map(|&v| pts_in_verb(v) as usize).sum();
        verb = j;

        let n = count;
        if n < 3 {
            continue;
        }
        let pts = &pts_all[pt_base..pt_base + n];
        let mut cross: Scalar;

        let index = find_max_y(pts);
        if pts[index].y < ymax {
            continue;
        }

        if pts[(index + 1) % n].y == pts[index].y {
            let (min_i, max_i) = find_min_max_x_at_y(pts, index, n);
            if min_i != max_i {
                cross = (min_i as Scalar) - (max_i as Scalar);
            } else {
                let prev = find_diff_pt(pts, index, n, -1);
                if prev == index {
                    continue;
                }
                let next = find_diff_pt(pts, index, n, 1);
                cross = cross_prod(pts[prev], pts[index], pts[next]);
                if cross == 0.0 && pts[prev].y == pts[index].y && pts[next].y == pts[index].y {
                    cross = pts[index].x - pts[next].x;
                }
            }
        } else {
            let prev = find_diff_pt(pts, index, n, -1);
            if prev == index {
                continue;
            }
            let next = find_diff_pt(pts, index, n, 1);
            cross = cross_prod(pts[prev], pts[index], pts[next]);
            if cross == 0.0 && pts[prev].y == pts[index].y && pts[next].y == pts[index].y {
                cross = pts[index].x - pts[next].x;
            }
        }

        if cross != 0.0 {
            ymax = pts[index].y;
            ymax_cross = cross;
        }
    }

    if ymax_cross != 0.0 {
        cross_to_dir(ymax_cross)
    } else {
        SkPathFirstDirection::Unknown
    }
}

// ---------------------------------------------------------------------------------------------
// Contains / winding
// ---------------------------------------------------------------------------------------------

#[inline]
fn poly_eval3(a: Scalar, b: Scalar, c: Scalar, t: Scalar) -> Scalar {
    (a * t + b) * t + c
}

#[inline]
fn poly_eval4(a: Scalar, b: Scalar, c: Scalar, d: Scalar, t: Scalar) -> Scalar {
    ((a * t + b) * t + c) * t + d
}

#[inline]
fn between(a: Scalar, b: Scalar, c: Scalar) -> bool {
    (a - b) * (c - b) <= 0.0
}

fn eval_cubic_pts(c0: Scalar, c1: Scalar, c2: Scalar, c3: Scalar, t: Scalar) -> Scalar {
    let a = c3 + 3.0 * (c1 - c2) - c0;
    let b = 3.0 * (c2 - c1 - c1 + c0);
    let c = 3.0 * (c1 - c0);
    let d = c0;
    poly_eval4(a, b, c, d, t)
}

#[inline]
fn check_on_curve(x: Scalar, y: Scalar, start: Point, end: Point) -> bool {
    if start.y == end.y {
        between(start.x, x, end.x) && x != end.x
    } else {
        x == start.x && y == start.y
    }
}

fn winding_mono_cubic(pts: &[Point], x: Scalar, y: Scalar, on_curve: &mut i32) -> i32 {
    let (mut y0, y1, mut y3) = (pts[0].y, pts[1].y, pts[3].y);
    let mut dir = 1;
    if y0 > y3 {
        std::mem::swap(&mut y0, &mut y3);
        dir = -1;
    }
    let y2 = y1;
    let _ = y2;
    if y < y0 || y > y3 {
        return 0;
    }
    if check_on_curve(x, y, pts[0], pts[3]) {
        *on_curve += 1;
        return 0;
    }
    if y == y3 {
        return 0;
    }

    let (mut min, mut max) = (pts[0].x, pts[0].x);
    for p in &pts[..4] {
        min = min.min(p.x);
        max = max.max(p.x);
    }
    if x < min {
        return 0;
    }
    if x > max {
        return dir;
    }

    let t = match SkCubicClipper::chop_mono_at_y(&[pts[0], pts[1], pts[2], pts[3]], y) {
        Some(t) => t,
        None => return 0,
    };
    let xt = eval_cubic_pts(pts[0].x, pts[1].x, pts[2].x, pts[3].x, t);
    if scalar_nearly_equal(xt, x) {
        if x != pts[3].x || y != pts[3].y {
            *on_curve += 1;
        }
        return 0;
    }
    if xt < x { dir } else { 0 }
}

fn winding_cubic(pts: &[Point], x: Scalar, y: Scalar, on_curve: &mut i32) -> i32 {
    let mut dst = [Point::new(0.0, 0.0); 10];
    let n = sk_chop_cubic_at_y_extrema(pts, &mut dst);
    let mut w = 0;
    for i in 0..=n as usize {
        w += winding_mono_cubic(&dst[i * 3..], x, y, on_curve);
    }
    w
}

#[inline]
fn is_mono_quad(y0: Scalar, y1: Scalar, y2: Scalar) -> bool {
    if y0 == y1 {
        return true;
    }
    if y0 < y1 { y1 <= y2 } else { y1 >= y2 }
}

fn conic_eval_numerator(src: &[Scalar], w: Scalar, t: Scalar) -> f64 {
    let src2w = (src[2] as f64) * (w as f64);
    let c = src[0] as f64;
    let a = src[4] as f64 - 2.0 * src2w + c;
    let b = 2.0 * (src2w - c);
    (a * (t as f64) + b) * (t as f64) + c
}

fn conic_eval_denominator(w: Scalar, t: Scalar) -> f64 {
    let b = 2.0 * (w as f64 - 1.0);
    let c = 1.0;
    let a = -b;
    (a * (t as f64) + b) * (t as f64) + c
}

fn winding_mono_conic(conic: &SkConic, x: Scalar, y: Scalar, on_curve: &mut i32) -> i32 {
    let pts = &conic.pts;
    let (mut y0, _y1, mut y2) = (pts[0].y, pts[1].y, pts[2].y);
    let mut dir = 1;
    if y0 > y2 {
        std::mem::swap(&mut y0, &mut y2);
        dir = -1;
    }
    if y < y0 || y > y2 {
        return 0;
    }
    if check_on_curve(x, y, pts[0], pts[2]) {
        *on_curve += 1;
        return 0;
    }
    if y == y2 {
        return 0;
    }

    let mut a = pts[2].y;
    let mut b = pts[1].y * conic.w - y * conic.w + y;
    let mut c = pts[0].y;
    a += c - 2.0 * b;
    b -= c;
    c -= y;

    let mut roots = [0.0; 2];
    let n = sk_find_unit_quad_roots(a, 2.0 * b, c, &mut roots);
    let xt = if n == 0 {
        pts[(1 - dir) as usize].x
    } else {
        let t = roots[0];
        let src = [pts[0].x, pts[1].x, pts[1].x, pts[2].x, pts[2].x];
        (conic_eval_numerator(&src, conic.w, t) / conic_eval_denominator(conic.w, t)) as Scalar
    };
    if scalar_nearly_equal(xt, x) {
        if x != pts[2].x || y != pts[2].y {
            *on_curve += 1;
        }
        return 0;
    }
    if xt < x { dir } else { 0 }
}

fn winding_conic(pts: &[Point], x: Scalar, y: Scalar, w: Scalar, on_curve: &mut i32) -> i32 {
    let conic = SkConic::from_pts(pts, w);
    let mut chopped = [SkConic::new(Point::default(), Point::default(), Point::default(), 0.0); 2];
    let is_mono =
        is_mono_quad(pts[0].y, pts[1].y, pts[2].y) || !conic.chop_at_y_extrema(&mut chopped);
    let mut ww = winding_mono_conic(if is_mono { &conic } else { &chopped[0] }, x, y, on_curve);
    if !is_mono {
        ww += winding_mono_conic(&chopped[1], x, y, on_curve);
    }
    ww
}

fn winding_mono_quad(pts: &[Point], x: Scalar, y: Scalar, on_curve: &mut i32) -> i32 {
    let (mut y0, y1, mut y2) = (pts[0].y, pts[1].y, pts[2].y);
    let mut dir = 1;
    if y0 > y2 {
        std::mem::swap(&mut y0, &mut y2);
        dir = -1;
    }
    if y < y0 || y > y2 {
        return 0;
    }
    if check_on_curve(x, y, pts[0], pts[2]) {
        *on_curve += 1;
        return 0;
    }
    if y == y2 {
        return 0;
    }

    let mut roots = [0.0; 2];
    let n = sk_find_unit_quad_roots(y0 - 2.0 * y1 + y2, 2.0 * (y1 - y0), y0 - y, &mut roots);
    let xt = if n == 0 {
        pts[(1 - dir) as usize].x
    } else {
        let t = roots[0];
        poly_eval3(
            pts[2].x - 2.0 * pts[1].x + pts[0].x,
            2.0 * (pts[1].x - pts[0].x),
            pts[0].x,
            t,
        )
    };
    if scalar_nearly_equal(xt, x) {
        if x != pts[2].x || y != pts[2].y {
            *on_curve += 1;
        }
        return 0;
    }
    if xt < x { dir } else { 0 }
}

fn winding_quad(pts: &[Point], x: Scalar, y: Scalar, on_curve: &mut i32) -> i32 {
    let mut storage = [Point::new(0.0, 0.0); 5];
    let mut w = 0;
    let mut n = 0;
    if !is_mono_quad(pts[0].y, pts[1].y, pts[2].y) {
        n = sk_chop_quad_at_y_extrema(pts, &mut storage);
    }
    if n > 0 {
        w += winding_mono_quad(&storage[..3], x, y, on_curve);
        w += winding_mono_quad(&storage[2..], x, y, on_curve);
    } else {
        w += winding_mono_quad(pts, x, y, on_curve);
    }
    w
}

fn winding_line(pts: &[Point], x: Scalar, y: Scalar, on_curve: &mut i32) -> i32 {
    let (x0, mut y0) = (pts[0].x, pts[0].y);
    let (x1, mut y1) = (pts[1].x, pts[1].y);
    let dy = y1 - y0;
    let mut dir = 1;
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
        dir = -1;
    }
    if y < y0 || y > y1 {
        return 0;
    }
    if check_on_curve(x, y, pts[0], pts[1]) {
        *on_curve += 1;
        return 0;
    }
    if y == y1 {
        return 0;
    }
    let cross = (x1 - x0) * (y - pts[0].y) - dy * (x - x0);
    if cross == 0.0 {
        if x != x1 || y != pts[1].y {
            *on_curve += 1;
        }
        dir = 0;
    } else if scalar_sign_as_int(cross) == dir {
        dir = 0;
    }
    dir
}

fn tangent_line(pts: &[Point], x: Scalar, y: Scalar, tangents: &mut Vec<Point>) {
    if !between(pts[0].y, y, pts[1].y) {
        return;
    }
    if !between(pts[0].x, x, pts[1].x) {
        return;
    }
    let dx = pts[1].x - pts[0].x;
    let dy = pts[1].y - pts[0].y;
    if !scalar_nearly_equal((x - pts[0].x) * dy, dx * (y - pts[0].y)) {
        return;
    }
    tangents.push(Point::new(dx, dy));
}

fn tangent_quad(pts: &[Point], x: Scalar, y: Scalar, tangents: &mut Vec<Point>) {
    if !between(pts[0].y, y, pts[2].y) {
        return;
    }
    if !between(pts[0].x, x, pts[2].x) {
        return;
    }
    let mut roots = [0.0; 2];
    let n = sk_find_unit_quad_roots(
        pts[0].y - 2.0 * pts[1].y + pts[2].y,
        2.0 * (pts[1].y - pts[0].y),
        pts[0].y - y,
        &mut roots,
    );
    for i in 0..n as usize {
        let t = roots[i];
        let xt = poly_eval3(
            pts[2].x - 2.0 * pts[1].x + pts[0].x,
            2.0 * (pts[1].x - pts[0].x),
            pts[0].x,
            t,
        );
        if scalar_nearly_equal(x, xt) {
            tangents.push(sk_eval_quad_tangent_at(pts, t));
        }
    }
}

fn tangent_conic(pts: &[Point], x: Scalar, y: Scalar, w: Scalar, tangents: &mut Vec<Point>) {
    if !between(pts[0].y, y, pts[2].y) {
        return;
    }
    if !between(pts[0].x, x, pts[2].x) {
        return;
    }
    let mut a = pts[2].y;
    let mut b = pts[1].y * w - y * w + y;
    let mut c = pts[0].y;
    a += c - 2.0 * b;
    b -= c;
    c -= y;
    let mut roots = [0.0; 2];
    let n = sk_find_unit_quad_roots(a, 2.0 * b, c, &mut roots);
    let conic = SkConic::from_pts(pts, w);
    for i in 0..n as usize {
        let t = roots[i];
        let src = [pts[0].x, pts[1].x, pts[1].x, pts[2].x, pts[2].x];
        let xt = (conic_eval_numerator(&src, w, t) / conic_eval_denominator(w, t)) as Scalar;
        if scalar_nearly_equal(x, xt) {
            tangents.push(conic.eval_tangent_at(t));
        }
    }
}

fn tangent_cubic(pts: &[Point], x: Scalar, y: Scalar, tangents: &mut Vec<Point>) {
    if !between(pts[0].y, y, pts[3].y) {
        return;
    }
    if !between(pts[0].x, x, pts[3].x) {
        return;
    }
    let mut dst = [Point::new(0.0, 0.0); 10];
    let n = sk_chop_cubic_at_y_extrema(pts, &mut dst);
    for i in 0..=n as usize {
        let c = &dst[i * 3..];
        let t = match SkCubicClipper::chop_mono_at_y(&[c[0], c[1], c[2], c[3]], y) {
            Some(t) => t,
            None => continue,
        };
        let xt = eval_cubic_pts(c[0].x, c[1].x, c[2].x, c[3].x, t);
        if !scalar_nearly_equal(x, xt) {
            continue;
        }
        let mut tangent = Point::new(0.0, 0.0);
        sk_eval_cubic_at(&[c[0], c[1], c[2], c[3]], t, None, Some(&mut tangent), None);
        tangents.push(tangent);
    }
}

#[inline]
fn contains_inclusive(r: &Rect, p: Point) -> bool {
    p.x >= r.left() && p.x <= r.right() && p.y >= r.top() && p.y <= r.bottom()
}

/// `SkPathPriv::Contains`.
pub fn contains(raw: &SkPathRaw, p: Point) -> bool {
    let ft = raw.f_fill_type;
    let is_inverse = ft.is_inverse();

    if raw.empty() {
        return is_inverse;
    }
    if !contains_inclusive(&raw.f_bounds, p) {
        return is_inverse;
    }

    let mut w: i32 = 0;
    let mut on_curve_count: i32 = 0;
    let mut iter = SkPathEdgeIter::from_raw(raw);
    loop {
        let res = iter.next();
        if res.is_done() {
            break;
        }
        let pts = res.pts();
        match res.f_edge {
            SkPathEdge::Line => w += winding_line(pts, p.x, p.y, &mut on_curve_count),
            SkPathEdge::Quad => w += winding_quad(pts, p.x, p.y, &mut on_curve_count),
            SkPathEdge::Conic => {
                w += winding_conic(pts, p.x, p.y, iter.conic_weight(), &mut on_curve_count)
            }
            SkPathEdge::Cubic => w += winding_cubic(pts, p.x, p.y, &mut on_curve_count),
        }
    }

    let even_odd_fill = ft == SkPathFillType::EvenOdd || ft == SkPathFillType::InverseEvenOdd;
    if even_odd_fill {
        w &= 1;
    }
    if w != 0 {
        return !is_inverse;
    }
    if on_curve_count <= 1 {
        return ((on_curve_count != 0) as i32 ^ (is_inverse as i32)) != 0;
    }
    if (on_curve_count & 1) != 0 || even_odd_fill {
        return (((on_curve_count & 1) != 0) as i32 ^ (is_inverse as i32)) != 0;
    }

    let mut tangents: Vec<Point> = Vec::new();
    let mut iter = SkPathEdgeIter::from_raw(raw);
    loop {
        let res = iter.next();
        if res.is_done() {
            break;
        }
        let pts = res.pts();
        let old_len = tangents.len();
        match res.f_edge {
            SkPathEdge::Line => tangent_line(pts, p.x, p.y, &mut tangents),
            SkPathEdge::Quad => tangent_quad(pts, p.x, p.y, &mut tangents),
            SkPathEdge::Conic => tangent_conic(pts, p.x, p.y, iter.conic_weight(), &mut tangents),
            SkPathEdge::Cubic => tangent_cubic(pts, p.x, p.y, &mut tangents),
        }
        if tangents.len() > old_len {
            let last = tangents.len() - 1;
            let tangent = tangents[last];
            let sq_len = PointPriv::length_sqd(&tangent);
            if scalar_nearly_zero(sq_len) {
                tangents.remove(last);
            } else {
                for index in 0..last {
                    let test = tangents[index];
                    if scalar_nearly_zero(test.cross(&tangent))
                        && scalar_sign_as_int(tangent.x * test.x) <= 0
                        && scalar_sign_as_int(tangent.y * test.y) <= 0
                    {
                        let removed = tangents.swap_remove(index);
                        let _ = removed;
                        break;
                    }
                }
            }
        }
    }
    ((!tangents.is_empty()) as i32 ^ (is_inverse as i32)) != 0
}

// ---------------------------------------------------------------------------------------------
// Transform convexity / tight bounds
// ---------------------------------------------------------------------------------------------

/// `SkPathPriv::TransformConvexity`.
pub fn transform_convexity(
    mx: &Matrix,
    pts: &[Point],
    convexity: SkPathConvexity,
) -> SkPathConvexity {
    if mx.is_identity() || pts.is_empty() {
        return convexity;
    }
    if sk_path_convexity_is_convex(convexity) {
        if !mx.is_scale_translate() || !is_axis_aligned(pts) {
            return SkPathConvexity::Unknown;
        }
        let det = mx.getScaleX() * mx.getScaleY() - mx.getSkewX() * mx.getSkewY();
        if det < 0.0 {
            return sk_path_convexity_opposite_convex_direction(convexity);
        }
        if det == 0.0 {
            return SkPathConvexity::ConvexDegenerate;
        }
    }
    convexity
}

fn compute_quad_extremas(src: &[Point], extremas: &mut [Point; 5]) -> i32 {
    let mut ts = [0.0; 2];
    let mut n = sk_find_quad_extrema(
        src[0].x,
        src[1].x,
        src[2].x,
        (&mut ts[0..1]).try_into().unwrap(),
    );
    n += sk_find_quad_extrema(
        src[0].y,
        src[1].y,
        src[2].y,
        (&mut ts[n as usize..n as usize + 1]).try_into().unwrap(),
    );
    for i in 0..n as usize {
        extremas[i] = sk_eval_quad_at(src, ts[i]);
    }
    extremas[n as usize] = src[2];
    n + 1
}

fn compute_conic_extremas(src: &[Point], w: Scalar, extremas: &mut [Point; 5]) -> i32 {
    let conic = SkConic::from_pts(src, w);
    let mut ts = [0.0; 2];
    let mut n = 0;
    if let Some(t) = conic.find_x_extrema() {
        ts[n] = t;
        n += 1;
    }
    if let Some(t) = conic.find_y_extrema() {
        ts[n] = t;
        n += 1;
    }
    for i in 0..n {
        extremas[i] = conic.eval_at(ts[i]);
    }
    extremas[n] = src[2];
    (n + 1) as i32
}

fn compute_cubic_extremas(src: &[Point], extremas: &mut [Point; 5]) -> i32 {
    let mut ts = [0.0; 2];
    let mut n = sk_find_cubic_extrema(src[0].x, src[1].x, src[2].x, src[3].x, &mut ts);
    {
        let mut ts2 = [0.0; 2];
        let n2 = sk_find_cubic_extrema(src[0].y, src[1].y, src[2].y, src[3].y, &mut ts2);
        for i in 0..n2 as usize {
            ts[n as usize + i] = ts2[i];
        }
        n += n2;
    }
    for i in 0..n as usize {
        let mut pt = Point::new(0.0, 0.0);
        sk_eval_cubic_at(src, ts[i], Some(&mut pt), None, None);
        extremas[i] = pt;
    }
    extremas[n as usize] = src[3];
    n + 1
}

/// `SkPathPriv::ComputeTightBounds`.
pub fn compute_tight_bounds(points: &[Point], verbs: &[SkPathVerb], conics: &[f32]) -> Rect {
    if verbs.is_empty() {
        return Rect::make_empty();
    }
    if points.is_empty() {
        return Rect::make_empty();
    }
    let mut l = points[0].x;
    let mut t = points[0].y;
    let mut r = points[0].x;
    let mut b = points[0].y;

    let mut iter = SkPathIter::new(points, verbs, conics);
    while let Some(rec) = iter.next() {
        let mut extremas = [Point::new(0.0, 0.0); 5];
        let count: i32 = match rec.f_verb {
            SkPathVerb::Move => {
                extremas[0] = rec.f_points[0];
                1
            }
            SkPathVerb::Line => {
                extremas[0] = rec.f_points[1];
                1
            }
            SkPathVerb::Quad => compute_quad_extremas(rec.f_points, &mut extremas),
            SkPathVerb::Conic => {
                compute_conic_extremas(rec.f_points, rec.f_conic_weight, &mut extremas)
            }
            SkPathVerb::Cubic => compute_cubic_extremas(rec.f_points, &mut extremas),
            SkPathVerb::Close => 0,
            SkPathVerb::Done => 0,
        };
        for i in 0..count as usize {
            l = l.min(extremas[i].x);
            t = t.min(extremas[i].y);
            r = r.max(extremas[i].x);
            b = b.max(extremas[i].y);
        }
    }
    Rect::new(l, t, r, b)
}

// ---------------------------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------------------------

/// `SkPathPriv::FindLastMoveToIndex`.
pub fn find_last_move_to_index(verbs: &[SkPathVerb], pt_count: usize) -> i32 {
    if verbs.is_empty() {
        return -1;
    }
    let mut pt_index = pt_count as i32 - 1;
    for &v in verbs.iter().rev() {
        if v == SkPathVerb::Move {
            break;
        }
        pt_index -= pts_in_verb(v);
    }
    pt_index
}

/// `SkPathPriv::SimplifyRRect`.
pub fn simplify_rrect(rr: &RRect, start_index: u32) -> (RRectAsEnum, u32) {
    if rr.is_rect() || rr.is_empty() {
        return (RRectAsEnum::Rect, (start_index + 1) / 2);
    }
    if rr.is_oval() {
        return (RRectAsEnum::Oval, start_index / 2);
    }
    (RRectAsEnum::RRect, start_index)
}

/// `SkPathPriv::TransformDirAndStart`.
pub fn transform_dir_and_start(
    mx: &Matrix,
    is_rrect: bool,
    dir: SkPathDirection,
    start: u32,
) -> (SkPathDirection, u32) {
    if mx.is_identity() {
        return (dir, start);
    }
    let mut in_start = start;
    let mut is_ccw = dir == SkPathDirection::Ccw;
    let mut rm = 0u32;
    if is_rrect {
        rm = in_start & 1;
        in_start /= 2;
    }

    let scale_x = mx.getScaleX();
    let skew_x = mx.getSkewX();
    let skew_y = mx.getSkewY();
    let scale_y = mx.getScaleY();

    let anti_diag: u32;
    let top_neg: u32;
    let same_sign: u32;
    if scale_x != 0.0 {
        anti_diag = 0;
        if scale_x > 0.0 {
            top_neg = 0;
            same_sign = if scale_y > 0.0 { 1 } else { 0 };
        } else {
            top_neg = 2;
            same_sign = if scale_y > 0.0 { 0 } else { 1 };
        }
    } else {
        anti_diag = 1;
        if skew_x > 0.0 {
            top_neg = 0;
            same_sign = if skew_y > 0.0 { 1 } else { 0 };
        } else {
            top_neg = 2;
            same_sign = if skew_y > 0.0 { 0 } else { 1 };
        }
    }

    let mut out_start: u32;
    if same_sign != anti_diag {
        out_start = (in_start + 4 - (top_neg | anti_diag)) % 4;
        if is_rrect {
            out_start = 2 * out_start + rm;
        }
    } else {
        is_ccw = !is_ccw;
        out_start = (6 + (top_neg | anti_diag) - in_start) % 4;
        if is_rrect {
            out_start = 2 * out_start + if rm == 1 { 0 } else { 1 };
        }
    }
    (
        if is_ccw {
            SkPathDirection::Ccw
        } else {
            SkPathDirection::Cw
        },
        out_start,
    )
}

/// `SkPathPriv::DeduceRRectFromContour`.
pub fn deduce_rrect_from_contour(bounds: &Rect, pts: &[Point], verbs: &[SkPathVerb]) -> RRect {
    if bounds.is_empty() {
        return RRect::make_empty();
    }
    let mut radii = [Point::new(0.0, 0.0); 4];
    let mut pt_index = 0usize;
    for &verb in verbs {
        match verb {
            SkPathVerb::Move => pt_index += 1,
            SkPathVerb::Line => pt_index += 1,
            SkPathVerb::Quad | SkPathVerb::Cubic => {
                debug_assert!(false, "unexpected verb in rrect contour");
            }
            SkPathVerb::Conic => {
                let v1_0 = pts[pt_index] - pts[pt_index - 1];
                let v2_1 = pts[pt_index + 1] - pts[pt_index];
                let dxdy = if v1_0.x != 0.0 {
                    Point::new(v1_0.x.abs(), v2_1.y.abs())
                } else if v1_0.y == 0.0 {
                    Point::new(v2_1.x.abs(), v2_1.y.abs())
                } else {
                    Point::new(v2_1.x.abs(), v1_0.y.abs())
                };

                let corner = if pts[pt_index].x > bounds.center_x() {
                    if pts[pt_index].y > bounds.center_y() {
                        Corner::LowerRight
                    } else {
                        Corner::UpperRight
                    }
                } else if pts[pt_index].y > bounds.center_y() {
                    Corner::LowerLeft
                } else {
                    Corner::UpperLeft
                };
                radii[corner.index()] = dxdy;
                pt_index += 2;
            }
            SkPathVerb::Close => {}
            SkPathVerb::Done => {}
        }
    }
    let mut rrect = RRect::default();
    rrect.set_rect_radii(bounds, &radii);
    rrect
}

/// `SkPathPriv::AsFirstDirection`.
#[inline]
pub fn as_first_direction(dir: SkPathDirection) -> SkPathFirstDirection {
    match dir {
        SkPathDirection::Cw => SkPathFirstDirection::Cw,
        SkPathDirection::Ccw => SkPathFirstDirection::Ccw,
    }
}

/// `SkPathPriv::kW0PlaneDistance`.
pub const KW0_PLANE_DISTANCE: Scalar = 1.0 / (1u32 << 14) as Scalar;

/// The `SkPathPriv` namespace. Functions that need `SkPath`/`SkPathData`/`SkPathBuilder`
/// live here.
pub struct SkPathPriv;

impl SkPathPriv {
    #[inline]
    pub fn compute_segment_mask(verbs: &[SkPathVerb]) -> u8 {
        compute_segment_mask(verbs)
    }

    #[inline]
    pub fn compute_convexity(
        pts: &[Point],
        verbs: &[SkPathVerb],
        conics: &[f32],
    ) -> SkPathConvexity {
        compute_convexity(pts, verbs, conics)
    }

    #[inline]
    pub fn transform_convexity(
        mx: &Matrix,
        pts: &[Point],
        convexity: SkPathConvexity,
    ) -> SkPathConvexity {
        transform_convexity(mx, pts, convexity)
    }

    #[inline]
    pub fn compute_first_direction(raw: &SkPathRaw) -> SkPathFirstDirection {
        compute_first_direction(raw)
    }

    #[inline]
    pub fn contains(raw: &SkPathRaw, p: Point) -> bool {
        contains(raw, p)
    }

    #[inline]
    pub fn compute_tight_bounds(points: &[Point], verbs: &[SkPathVerb], conics: &[f32]) -> Rect {
        compute_tight_bounds(points, verbs, conics)
    }

    #[inline]
    pub fn find_last_move_to_index(verbs: &[SkPathVerb], pt_count: usize) -> i32 {
        find_last_move_to_index(verbs, pt_count)
    }

    #[inline]
    pub fn is_axis_aligned(pts: &[Point]) -> bool {
        is_axis_aligned(pts)
    }

    #[inline]
    pub fn pts_in_verb(verb: SkPathVerb) -> i32 {
        pts_in_verb(verb)
    }

    #[inline]
    pub fn pts_in_iter(verb: SkPathVerb) -> i32 {
        pts_in_iter(verb)
    }

    #[inline]
    pub fn all_points_eq(pts: &[Point]) -> bool {
        for i in 1..pts.len() {
            if pts[0] != pts[i] {
                return false;
            }
        }
        true
    }

    #[inline]
    pub fn is_closed_single_contour_verbs(verbs: &[SkPathVerb]) -> bool {
        if verbs.is_empty() {
            return false;
        }
        let mut move_count = 0;
        for (i, &verb) in verbs.iter().enumerate() {
            match verb {
                SkPathVerb::Move => {
                    move_count += 1;
                    if move_count > 1 {
                        return false;
                    }
                }
                SkPathVerb::Close => {
                    return i == verbs.len() - 1;
                }
                _ => {}
            }
        }
        false
    }

    #[inline]
    pub fn simplify_rrect(rr: &RRect, start_index: u32) -> (RRectAsEnum, u32) {
        simplify_rrect(rr, start_index)
    }

    #[inline]
    pub fn transform_dir_and_start(
        mx: &Matrix,
        is_rrect: bool,
        dir: SkPathDirection,
        start: u32,
    ) -> (SkPathDirection, u32) {
        transform_dir_and_start(mx, is_rrect, dir, start)
    }

    #[inline]
    pub fn deduce_rrect_from_contour(bounds: &Rect, pts: &[Point], verbs: &[SkPathVerb]) -> RRect {
        deduce_rrect_from_contour(bounds, pts, verbs)
    }

    #[inline]
    pub fn is_rect_contour(
        pts: &[Point],
        verbs: &[SkPathVerb],
        segment_mask: u32,
        allow_partial: bool,
    ) -> Option<RectContour> {
        is_rect_contour(pts, verbs, segment_mask, allow_partial)
    }

    #[inline]
    pub fn as_first_direction(dir: SkPathDirection) -> SkPathFirstDirection {
        as_first_direction(dir)
    }

    #[inline]
    pub fn raw(path: &SkPath, rc: SkResolveConvexity) -> Option<SkPathRaw> {
        path.raw(rc)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////
// `struct SkHalfPlane` and `SkPathPriv::PerspectiveClip` from `src/core/SkPath.cpp`.

/// `SkHalfPlane`: `fA*x + fB*y + fC`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkHalfPlane {
    pub f_a: Scalar,
    pub f_b: Scalar,
    pub f_c: Scalar,
}

/// `SkHalfPlane::Result`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkHalfPlaneResult {
    AllNegative,
    AllPositive,
    Mixed,
}

impl SkHalfPlane {
    pub fn new(f_a: Scalar, f_b: Scalar, f_c: Scalar) -> Self {
        SkHalfPlane { f_a, f_b, f_c }
    }

    #[inline]
    pub fn eval(&self, x: Scalar, y: Scalar) -> Scalar {
        self.f_a * x + self.f_b * y + self.f_c
    }

    pub fn normalize(&mut self) -> bool {
        let mut a = self.f_a as f64;
        let mut b = self.f_b as f64;
        let mut c = self.f_c as f64;
        let dmag = (a * a + b * b).sqrt();
        // length of initial plane normal is zero
        if dmag == 0.0 {
            self.f_a = 0.0;
            self.f_b = 0.0;
            self.f_c = 1.0;
            return true;
        }
        let dscale = crate::floating_point::ieee_double_divide(1.0, dmag);
        a *= dscale;
        b *= dscale;
        c *= dscale;
        // check if we're not finite, or normal is zero-length
        if !(a.is_finite() && b.is_finite() && c.is_finite()) || (a == 0.0 && b == 0.0) {
            self.f_a = 0.0;
            self.f_b = 0.0;
            self.f_c = 1.0;
            return false;
        }
        self.f_a = a as Scalar;
        self.f_b = b as Scalar;
        self.f_c = c as Scalar;
        true
    }

    pub fn test(&self, bounds: &Rect) -> SkHalfPlaneResult {
        // check whether the diagonal aligned with the normal crosses the plane
        let (diag_min_x, diag_max_x) = if self.f_a >= 0.0 {
            (bounds.left(), bounds.right())
        } else {
            (bounds.right(), bounds.left())
        };
        let (diag_min_y, diag_max_y) = if self.f_b >= 0.0 {
            (bounds.top(), bounds.bottom())
        } else {
            (bounds.bottom(), bounds.top())
        };
        let test = self.eval(diag_min_x, diag_min_y);
        let sign = test * self.eval(diag_max_x, diag_max_y);
        if sign > 0.0 {
            // the path is either all on one side of the half-plane or the other
            if test < 0.0 {
                SkHalfPlaneResult::AllNegative
            } else {
                SkHalfPlaneResult::AllPositive
            }
        } else {
            SkHalfPlaneResult::Mixed
        }
    }
}

/// `static std::optional<SkPath> clip(const SkPath&, const SkHalfPlane&)`.
/// Assumes the plane is pre-normalized.
pub fn clip_path_half_plane(path: &SkPath, plane: &SkHalfPlane) -> Option<SkPath> {
    use crate::sk_edge_clipper::SkEdgeClipper;

    let p0 = Point::new(-plane.f_a * plane.f_c, -plane.f_b * plane.f_c);
    let mut mx = Matrix::new();
    mx.set_all(
        plane.f_b, plane.f_a, p0.x, -plane.f_a, plane.f_b, p0.y, 0.0, 0.0, 1.0,
    );
    let mut inv = Matrix::new();
    if !mx.invert(Some(&mut inv)) {
        return None;
    }

    let rotated = path.try_make_transform(&inv)?;
    let raw = SkPathPriv::raw(&rotated, SkResolveConvexity::No)?;

    let big = Scalar::MAX;
    let clip = Rect::new(-big, 0.0, big, big);

    struct Rec {
        f_result: SkPathBuilder,
        f_prev: Point,
    }
    let mut rec = Rec {
        f_result: SkPathBuilder::new(),
        f_prev: Point::new(0.0, 0.0),
    };

    SkEdgeClipper::clip_path(&raw, &clip, false, |clipper, mut new_ctr| {
        let mut add_line_to = false;
        let mut pts = [Point::new(0.0, 0.0); 4];
        while let Some(verb) = clipper.next(&mut pts) {
            if new_ctr {
                rec.f_result.move_to(pts[0]);
                rec.f_prev = pts[0];
                new_ctr = false;
            }

            if add_line_to || pts[0] != rec.f_prev {
                rec.f_result.line_to(pts[0]);
            }

            match verb {
                SkPathVerb::Line => {
                    rec.f_result.line_to(pts[1]);
                    rec.f_prev = pts[1];
                }
                SkPathVerb::Quad => {
                    rec.f_result.quad_to(pts[1], pts[2]);
                    rec.f_prev = pts[2];
                }
                SkPathVerb::Cubic => {
                    rec.f_result.cubic_to(pts[1], pts[2], pts[3]);
                    rec.f_prev = pts[3];
                }
                _ => {}
            }
            add_line_to = true;
        }
    });

    rec.f_result.set_fill_type(path.get_fill_type());
    let result = rec.f_result.detach(Some(&mx));
    if !result.is_finite() {
        return None;
    }
    Some(result)
}

/// `SkPathPriv::PerspectiveClip`.
pub fn perspective_clip(path: &SkPath, matrix: &Matrix, clipped_path: &mut SkPath) -> bool {
    if !matrix.has_perspective() {
        return false;
    }

    let mut plane = SkHalfPlane::new(
        matrix.persp_x(),
        matrix.persp_y(),
        matrix.persp_z() - KW0_PLANE_DISTANCE,
    );
    if plane.normalize() {
        match plane.test(path.get_bounds()) {
            SkHalfPlaneResult::AllPositive => return false,
            SkHalfPlaneResult::Mixed => {
                match clip_path_half_plane(path, &plane) {
                    Some(result) => *clipped_path = result,
                    // clipped out (or failed)
                    None => *clipped_path = SkPath::new(),
                }
                return true;
            }
            _ => {} // handled outside of the match
        }
    }
    // clipped out (or failed)
    *clipped_path = SkPath::new();
    true
}
