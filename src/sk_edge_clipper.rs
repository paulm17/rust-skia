//! Port of `src/core/SkEdgeClipper.h` and `src/core/SkEdgeClipper.cpp`.
//!
//! `SkEdgeClipper::ClipPath` consumes `SkPathRaw`/`SkPathEdgeIter`, both owned by M07; see
//! `plan/handoffs/M06.md` for the recorded integration gate.

use crate::point::Point;
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_abs};
use crate::sk_geometry::{
    sk_chop_cubic_at, sk_chop_cubic_at_x_extrema, sk_chop_cubic_at_y_extrema,
    sk_chop_mono_cubic_at_x, sk_chop_mono_cubic_at_y, sk_chop_quad_at, sk_chop_quad_at_x_extrema,
    sk_chop_quad_at_y_extrema, sk_find_unit_quad_roots,
};
use crate::sk_line_clipper::{K_MAX_POINTS as K_LINE_MAX_POINTS, clip_line};
use crate::sk_path_edge_iter::{SkPathEdge, SkPathEdgeIter};
use crate::sk_path_raw::SkPathRaw;

/// `SkPathVerb` from `include/core/SkPathTypes.h`, declared here for the clipper's iterator API.
/// Path ownership (M07) adopts this enum unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SkPathVerb {
    Move = 0,
    Line = 1,
    Quad = 2,
    Conic = 3,
    Cubic = 4,
    Close = 5,
    #[default]
    Done = 6,
}

const K_MAX_VERBS: usize = 18;
const K_MAX_POINTS: usize = 54;

fn quick_reject(bounds: &Rect, clip: &Rect) -> bool {
    bounds.top >= clip.bottom || bounds.bottom <= clip.top
}

fn clamp_le(value: &mut Scalar, max: Scalar) {
    if *value > max {
        *value = max;
    }
}

fn clamp_ge(value: &mut Scalar, min: Scalar) {
    if *value < min {
        *value = min;
    }
}

/// `src[]` must be monotonic in Y. This routine copies `src` into `dst`, and sorts it to be
/// increasing in Y. If it had to reverse the order of the points, it returns true.
fn sort_increasing_y(dst: &mut [Point], src: &[Point], count: usize) -> bool {
    // we need the data to be monotonically increasing in Y
    if src[0].y > src[count - 1].y {
        for i in 0..count {
            dst[i] = src[count - i - 1];
        }
        true
    } else {
        dst[..count].copy_from_slice(&src[..count]);
        false
    }
}

fn chop_mono_quad_at(c0: Scalar, c1: Scalar, c2: Scalar, target: Scalar, t: &mut Scalar) -> bool {
    // Solve F(t) = y where F(t) := [0](1-t)^2 + 2[1]t(1-t) + [2]t^2
    let a = c0 - c1 - c1 + c2;
    let b = 2.0 * (c1 - c0);
    let c = c0 - target;

    let mut roots = [0.0; 2]; // we only expect one, but make room for 2 for safety
    let count = sk_find_unit_quad_roots(a, b, c, &mut roots);
    if count != 0 {
        *t = roots[0];
        true
    } else {
        false
    }
}

fn chop_mono_quad_at_y(pts: &[Point; 3], y: Scalar) -> Option<Scalar> {
    let mut t = 0.0;
    if chop_mono_quad_at(pts[0].y, pts[1].y, pts[2].y, y, &mut t) {
        Some(t)
    } else {
        None
    }
}

fn chop_mono_quad_at_x(pts: &[Point; 3], x: Scalar) -> Option<Scalar> {
    let mut t = 0.0;
    if chop_mono_quad_at(pts[0].x, pts[1].x, pts[2].x, x, &mut t) {
        Some(t)
    } else {
        None
    }
}

/// Modify `pts[]` in place so that it is clipped in Y to the clip rect.
fn chop_quad_in_y(pts: &mut [Point; 3], clip: &Rect) {
    let mut tmp = [Point::default(); 5]; // for SkChopQuadAt

    // are we partially above
    if pts[0].y < clip.top {
        if let Some(t) = chop_mono_quad_at_y(pts, clip.top) {
            // take the 2nd chopped quad
            let src = *pts;
            sk_chop_quad_at(&src, &mut tmp, t);
            // clamp to clean up imprecise numerics in the chop
            tmp[2].y = clip.top;
            clamp_ge(&mut tmp[3].y, clip.top);

            pts[0] = tmp[2];
            pts[1] = tmp[3];
        } else {
            // if chopMonoQuadAtY failed, then we may have hit inexact numerics
            // so we just clamp against the top
            for point in pts.iter_mut() {
                if point.y < clip.top {
                    point.y = clip.top;
                }
            }
        }
    }

    // are we partially below
    if pts[2].y > clip.bottom {
        if let Some(t) = chop_mono_quad_at_y(pts, clip.bottom) {
            let src = *pts;
            sk_chop_quad_at(&src, &mut tmp, t);
            // clamp to clean up imprecise numerics in the chop
            clamp_le(&mut tmp[1].y, clip.bottom);
            tmp[2].y = clip.bottom;

            pts[1] = tmp[1];
            pts[2] = tmp[2];
        } else {
            // if chopMonoQuadAtY failed, then we may have hit inexact numerics
            // so we just clamp against the bottom
            for point in pts.iter_mut() {
                if point.y > clip.bottom {
                    point.y = clip.bottom;
                }
            }
        }
    }
}

fn mono_cubic_closest_t(src: &[Scalar], x: Scalar) -> Scalar {
    let mut t = 0.5;
    let mut best_t = 0.0;
    let mut step = 0.25;
    let d = src[0];
    let a = src[6] + 3.0 * (src[2] - src[4]) - d;
    let b = 3.0 * (src[4] - src[2] - src[2] + d);
    let c = 3.0 * (src[2] - d);
    let x = x - d;
    let mut closest = Scalar::MAX;
    loop {
        let loc = ((a * t + b) * t + c) * t;
        let dist = scalar_abs(loc - x);
        if closest > dist {
            closest = dist;
            best_t = t;
        }
        let last_t = t;
        t += if loc < x { step } else { -step };
        step *= 0.5;
        if !(closest > 0.25 && last_t != t) {
            break;
        }
    }
    best_t
}

fn chop_mono_cubic_at_y(src: &[Point; 4], y: Scalar, dst: &mut [Point]) {
    if sk_chop_mono_cubic_at_y(src, y, dst) {
        return;
    }
    let ys = [src[0].y, src[1].y, src[2].y, src[3].y];
    sk_chop_cubic_at(src, dst, mono_cubic_closest_t(&ys, y));
}

/// Modify `pts[]` in place so that it is clipped in Y to the clip rect.
fn chop_cubic_in_y(pts: &mut [Point; 4], clip: &Rect) {
    // are we partially above
    if pts[0].y < clip.top {
        let mut tmp = [Point::default(); 7];
        chop_mono_cubic_at_y(pts, clip.top, &mut tmp);

        // For a large range in the points, we can do a poor job of chopping, such that the t
        // we computed resulted in the lower cubic still being partly above the clip.
        // If just the first or first 2 Y values are above the fTop, we can just smash them
        // down. If the first 3 Ys are above fTop, we can't smash all 3, as that can really
        // distort the cubic. In this case, we take the first output (tmp[3..6] and treat it as
        // a guess, and re-chop against fTop.
        if tmp[3].y < clip.top && tmp[4].y < clip.top && tmp[5].y < clip.top {
            let tmp2: [Point; 4] = [tmp[3], tmp[4], tmp[5], tmp[6]];
            chop_mono_cubic_at_y(&tmp2, clip.top, &mut tmp);
        }

        // tmp[3, 4].fY should all be to the below clip.fTop.
        // Since we can't trust the numerics of the chopper, we force those conditions now
        tmp[3].y = clip.top;
        clamp_ge(&mut tmp[4].y, clip.top);

        pts[0] = tmp[3];
        pts[1] = tmp[4];
        pts[2] = tmp[5];
    }

    // are we partially below
    if pts[3].y > clip.bottom {
        let mut tmp = [Point::default(); 7];
        chop_mono_cubic_at_y(pts, clip.bottom, &mut tmp);
        tmp[3].y = clip.bottom;
        clamp_le(&mut tmp[2].y, clip.bottom);

        pts[1] = tmp[1];
        pts[2] = tmp[2];
        pts[3] = tmp[3];
    }
}

fn chop_mono_cubic_at_x(src: &[Point; 4], x: Scalar, dst: &mut [Point]) {
    if sk_chop_mono_cubic_at_x(src, x, dst) {
        return;
    }
    let xs = [src[0].x, src[1].x, src[2].x, src[3].x];
    sk_chop_cubic_at(src, dst, mono_cubic_closest_t(&xs, x));
}

fn compute_cubic_bounds(pts: &[Point; 4]) -> Rect {
    Rect::bounds_or_empty(pts)
}

fn too_big_for_reliable_float_math(r: &Rect) -> bool {
    // limit set as the largest float value for which we can still reliably compute things like
    // - chopping at XY extrema
    // - chopping at Y or X values for clipping
    let limit: Scalar = (1 << 22) as Scalar;
    r.left < -limit || r.top < -limit || r.right > limit || r.bottom > limit
}

/// This is basically an iterator. It is initialized with an edge and a clip,
/// and then `next()` is called until it returns `None`.
#[derive(Clone, Debug)]
pub struct SkEdgeClipper {
    can_cull_to_the_right: bool,
    points: [Point; K_MAX_POINTS],
    verbs: [SkPathVerb; K_MAX_VERBS],
    curr_point: usize,
    curr_verb: usize,
    curr_verb_stop: usize,
}

impl SkEdgeClipper {
    pub fn new(can_cull_to_the_right: bool) -> Self {
        Self {
            can_cull_to_the_right,
            points: [Point::default(); K_MAX_POINTS],
            verbs: [SkPathVerb::Done; K_MAX_VERBS],
            curr_point: 0,
            curr_verb: 0,
            curr_verb_stop: 0,
        }
    }

    pub fn can_cull_to_the_right(&self) -> bool {
        self.can_cull_to_the_right
    }

    pub fn clip_line(&mut self, p0: Point, p1: Point, clip: &Rect) -> bool {
        self.curr_point = 0;
        self.curr_verb = 0;

        let mut lines = [Point::default(); K_LINE_MAX_POINTS];
        let pts = [p0, p1];
        let line_count = clip_line(&pts, clip, &mut lines, self.can_cull_to_the_right);
        for i in 0..line_count as usize {
            self.append_line(lines[i], lines[i + 1]);
        }

        self.curr_verb_stop = self.curr_verb;
        self.curr_point = 0;
        self.curr_verb = 0;
        self.curr_verb_stop != self.curr_verb
    }

    // srcPts[] must be monotonic in X and Y
    fn clip_mono_quad(&mut self, src_pts: &[Point], clip: &Rect) {
        let mut pts = [Point::default(); 3];
        let mut reverse = sort_increasing_y(&mut pts, src_pts, 3);

        // are we completely above or below
        if pts[2].y <= clip.top || pts[0].y >= clip.bottom {
            return;
        }

        // Now chop so that pts is contained within clip in Y
        chop_quad_in_y(&mut pts, clip);

        if pts[0].x > pts[2].x {
            pts.swap(0, 2);
            reverse = !reverse;
        }
        debug_assert!(pts[0].x <= pts[1].x);
        debug_assert!(pts[1].x <= pts[2].x);

        // Now chop in X as needed, and record the segments
        if pts[2].x <= clip.left {
            // wholly to the left
            self.append_v_line(clip.left, pts[0].y, pts[2].y, reverse);
            return;
        }
        if pts[0].x >= clip.right {
            // wholly to the right
            if !self.can_cull_to_the_right() {
                self.append_v_line(clip.right, pts[0].y, pts[2].y, reverse);
            }
            return;
        }

        let mut tmp = [Point::default(); 5]; // for SkChopQuadAt

        // are we partially to the left
        if pts[0].x < clip.left {
            if let Some(t) = chop_mono_quad_at_x(&pts, clip.left) {
                let src = pts;
                sk_chop_quad_at(&src, &mut tmp, t);
                self.append_v_line(clip.left, tmp[0].y, tmp[2].y, reverse);
                // clamp to clean up imprecise numerics in the chop
                tmp[2].x = clip.left;
                clamp_ge(&mut tmp[3].x, clip.left);

                pts[0] = tmp[2];
                pts[1] = tmp[3];
            } else {
                // if chopMonoQuadAtX failed, then we may have hit inexact numerics
                // so we just clamp against the left
                self.append_v_line(clip.left, pts[0].y, pts[2].y, reverse);
                return;
            }
        }

        // are we partially to the right
        if pts[2].x > clip.right {
            if let Some(t) = chop_mono_quad_at_x(&pts, clip.right) {
                let src = pts;
                sk_chop_quad_at(&src, &mut tmp, t);
                // clamp to clean up imprecise numerics in the chop
                clamp_le(&mut tmp[1].x, clip.right);
                tmp[2].x = clip.right;

                self.append_quad(&[tmp[0], tmp[1], tmp[2]], reverse);
                self.append_v_line(clip.right, tmp[2].y, tmp[4].y, reverse);
            } else {
                // if chopMonoQuadAtX failed, then we may have hit inexact numerics
                // so we just clamp against the right
                pts[1].x = pts[1].x.min(clip.right);
                pts[2].x = pts[2].x.min(clip.right);
                self.append_quad(&pts, reverse);
            }
        } else {
            // wholly inside the clip
            self.append_quad(&pts, reverse);
        }
    }

    pub fn clip_quad(&mut self, src_pts: &[Point; 3], clip: &Rect) -> bool {
        self.curr_point = 0;
        self.curr_verb = 0;

        let bounds = Rect::bounds_or_empty(src_pts);

        if !quick_reject(&bounds, clip) {
            let mut mono_y = [Point::default(); 5];
            let count_y = sk_chop_quad_at_y_extrema(src_pts, &mut mono_y);
            for y in 0..=count_y as usize {
                let mut mono_x = [Point::default(); 5];
                let count_x = sk_chop_quad_at_x_extrema(&mono_y[y * 2..], &mut mono_x);
                for x in 0..=count_x as usize {
                    self.clip_mono_quad(&mono_x[x * 2..], clip);
                    debug_assert!(self.curr_verb < K_MAX_VERBS);
                    debug_assert!(self.curr_point <= K_MAX_POINTS);
                }
            }
        }

        self.curr_verb_stop = self.curr_verb;
        self.curr_point = 0;
        self.curr_verb = 0;
        self.curr_verb_stop != self.curr_verb
    }

    // srcPts[] must be monotonic in X and Y
    fn clip_mono_cubic(&mut self, src: &[Point], clip: &Rect) {
        let mut pts = [Point::default(); 4];
        let mut reverse = sort_increasing_y(&mut pts, src, 4);

        // are we completely above or below
        if pts[3].y <= clip.top || pts[0].y >= clip.bottom {
            return;
        }

        // Now chop so that pts is contained within clip in Y
        chop_cubic_in_y(&mut pts, clip);

        if pts[0].x > pts[3].x {
            pts.swap(0, 3);
            pts.swap(1, 2);
            reverse = !reverse;
        }

        // Now chop in X as needed, and record the segments
        if pts[3].x <= clip.left {
            // wholly to the left
            self.append_v_line(clip.left, pts[0].y, pts[3].y, reverse);
            return;
        }
        if pts[0].x >= clip.right {
            // wholly to the right
            if !self.can_cull_to_the_right() {
                self.append_v_line(clip.right, pts[0].y, pts[3].y, reverse);
            }
            return;
        }

        // are we partially to the left
        if pts[0].x < clip.left {
            let mut tmp = [Point::default(); 7];
            chop_mono_cubic_at_x(&pts, clip.left, &mut tmp);
            self.append_v_line(clip.left, tmp[0].y, tmp[3].y, reverse);

            // tmp[3, 4].fX should all be to the right of clip.fLeft.
            tmp[3].x = clip.left;
            clamp_ge(&mut tmp[4].x, clip.left);

            pts[0] = tmp[3];
            pts[1] = tmp[4];
            pts[2] = tmp[5];
        }

        // are we partially to the right
        if pts[3].x > clip.right {
            let mut tmp = [Point::default(); 7];
            chop_mono_cubic_at_x(&pts, clip.right, &mut tmp);
            tmp[3].x = clip.right;
            clamp_le(&mut tmp[2].x, clip.right);

            self.append_cubic(&[tmp[0], tmp[1], tmp[2], tmp[3]], reverse);
            self.append_v_line(clip.right, tmp[3].y, tmp[6].y, reverse);
        } else {
            // wholly inside the clip
            self.append_cubic(&pts, reverse);
        }
    }

    pub fn clip_cubic(&mut self, src_pts: &[Point; 4], clip: &Rect) -> bool {
        self.curr_point = 0;
        self.curr_verb = 0;

        let bounds = compute_cubic_bounds(src_pts);
        // check if we're clipped out vertically
        if bounds.bottom > clip.top && bounds.top < clip.bottom {
            if too_big_for_reliable_float_math(&bounds) {
                // can't safely clip the cubic, so we give up and draw a line (which we can
                // safely clip)
                return self.clip_line(src_pts[0], src_pts[3], clip);
            } else {
                let mut mono_y = [Point::default(); 10];
                let count_y = sk_chop_cubic_at_y_extrema(src_pts, &mut mono_y);
                for y in 0..=count_y as usize {
                    let mut mono_x = [Point::default(); 10];
                    let count_x = sk_chop_cubic_at_x_extrema(&mono_y[y * 3..], &mut mono_x);
                    for x in 0..=count_x as usize {
                        self.clip_mono_cubic(&mono_x[x * 3..], clip);
                        debug_assert!(self.curr_verb < K_MAX_VERBS);
                        debug_assert!(self.curr_point <= K_MAX_POINTS);
                    }
                }
            }
        }

        self.curr_verb_stop = self.curr_verb;
        self.curr_point = 0;
        self.curr_verb = 0;
        self.curr_verb_stop != self.curr_verb
    }

    fn append_line(&mut self, p0: Point, p1: Point) {
        assert!(self.curr_verb < K_MAX_VERBS);
        self.verbs[self.curr_verb] = SkPathVerb::Line;
        self.curr_verb += 1;
        assert!(self.curr_point + 2 <= K_MAX_POINTS);
        self.points[self.curr_point] = p0;
        self.points[self.curr_point + 1] = p1;
        self.curr_point += 2;
    }

    fn append_v_line(&mut self, x: Scalar, y0: Scalar, y1: Scalar, reverse: bool) {
        let (y0, y1) = if reverse { (y1, y0) } else { (y0, y1) };
        assert!(self.curr_verb < K_MAX_VERBS);
        self.verbs[self.curr_verb] = SkPathVerb::Line;
        self.curr_verb += 1;

        assert!(self.curr_point + 2 <= K_MAX_POINTS);
        self.points[self.curr_point] = Point::new(x, y0);
        self.points[self.curr_point + 1] = Point::new(x, y1);
        self.curr_point += 2;
    }

    fn append_quad(&mut self, pts: &[Point], reverse: bool) {
        assert!(self.curr_verb < K_MAX_VERBS);
        self.verbs[self.curr_verb] = SkPathVerb::Quad;
        self.curr_verb += 1;

        assert!(self.curr_point + 3 <= K_MAX_POINTS);
        if reverse {
            self.points[self.curr_point] = pts[2];
            self.points[self.curr_point + 2] = pts[0];
        } else {
            self.points[self.curr_point] = pts[0];
            self.points[self.curr_point + 2] = pts[2];
        }
        self.points[self.curr_point + 1] = pts[1];
        self.curr_point += 3;
    }

    fn append_cubic(&mut self, pts: &[Point], reverse: bool) {
        assert!(self.curr_verb < K_MAX_VERBS);
        self.verbs[self.curr_verb] = SkPathVerb::Cubic;
        self.curr_verb += 1;

        assert!(self.curr_point + 4 <= K_MAX_POINTS);
        if reverse {
            for i in 0..4 {
                self.points[self.curr_point + i] = pts[3 - i];
            }
        } else {
            self.points[self.curr_point..self.curr_point + 4].copy_from_slice(&pts[..4]);
        }
        self.curr_point += 4;
    }

    /// `SkEdgeClipper::ClipPath`.
    pub fn clip_path<F: FnMut(&mut SkEdgeClipper, bool)>(
        raw: &SkPathRaw,
        clip: &Rect,
        can_cull_to_the_right: bool,
        mut consume: F,
    ) {
        let mut quadder = crate::sk_geometry::SkAutoConicToQuads::new();
        const K_CONIC_TOL: f32 = 0.25;

        let mut iter = SkPathEdgeIter::from_raw(raw);
        let mut clipper = SkEdgeClipper::new(can_cull_to_the_right);

        loop {
            let e = iter.next();
            if e.is_done() {
                break;
            }
            match e.f_edge {
                SkPathEdge::Line => {
                    if clipper.clip_line(e.f_pts[0], e.f_pts[1], clip) {
                        consume(&mut clipper, e.f_is_new_contour);
                    }
                }
                SkPathEdge::Quad => {
                    if clipper.clip_quad(&[e.f_pts[0], e.f_pts[1], e.f_pts[2]], clip) {
                        consume(&mut clipper, e.f_is_new_contour);
                    }
                }
                SkPathEdge::Conic => {
                    let quad_pts = quadder
                        .compute_quads_pts(&e.f_pts[..3], iter.conic_weight(), K_CONIC_TOL)
                        .to_vec();
                    let count = quadder.count_quads() as usize;
                    for i in 0..count {
                        let base = i * 2;
                        if clipper.clip_quad(
                            &[quad_pts[base], quad_pts[base + 1], quad_pts[base + 2]],
                            clip,
                        ) {
                            consume(&mut clipper, e.f_is_new_contour);
                        }
                    }
                }
                SkPathEdge::Cubic => {
                    if clipper.clip_cubic(&[e.f_pts[0], e.f_pts[1], e.f_pts[2], e.f_pts[3]], clip) {
                        consume(&mut clipper, e.f_is_new_contour);
                    }
                }
            }
        }
    }

    pub fn next(&mut self, pts: &mut [Point]) -> Option<SkPathVerb> {
        debug_assert!(self.curr_verb <= self.curr_verb_stop);
        if self.curr_verb >= self.curr_verb_stop {
            return None;
        }

        let verb = self.verbs[self.curr_verb];
        self.curr_verb += 1;
        match verb {
            SkPathVerb::Line => {
                assert!(self.curr_point + 2 <= K_MAX_POINTS);
                pts[..2].copy_from_slice(&self.points[self.curr_point..self.curr_point + 2]);
                self.curr_point += 2;
            }
            SkPathVerb::Quad => {
                assert!(self.curr_point + 3 <= K_MAX_POINTS);
                pts[..3].copy_from_slice(&self.points[self.curr_point..self.curr_point + 3]);
                self.curr_point += 3;
            }
            SkPathVerb::Cubic => {
                assert!(self.curr_point + 4 <= K_MAX_POINTS);
                pts[..4].copy_from_slice(&self.points[self.curr_point..self.curr_point + 4]);
                self.curr_point += 4;
            }
            _ => {
                debug_assert!(false, "unexpected verb in quadclipper2 iter");
            }
        }
        Some(verb)
    }
}
