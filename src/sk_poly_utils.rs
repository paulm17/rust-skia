//! Port of `src/utils/SkPolyUtils.h` and `src/utils/SkPolyUtils.cpp`.
//!
//! Polygon winding/convexity/simplicity classification, convex inset, simple-polygon offsetting
//! and ear-clipping triangulation. The reference guards this file behind
//! `!SK_ENABLE_OPTIMIZE_SIZE`; the normal macOS build includes it, so the full API is ported.
//!
//! The reference uses `SkTDArray` output parameters, intrusive linked lists (`OffsetEdge`,
//! `SkTInternalLList`), a red-black sweep-line tree (`ActiveEdge`) and `SkTDPQueue`. The port
//! keeps those algorithms but indexes nodes inside owned `Vec`s instead of raw pointers; the
//! processing order and all observable results are unchanged.

use crate::floating_point::{ieee_float_divide, is_finite};
use crate::point::{Point, Vector};
use crate::point_priv::PointPriv;
use crate::rect::Rect;
use crate::rect_priv::RectPriv;
use crate::scalar::{
    SCALAR_MAX, SCALAR_MIN, SCALAR_NEARLY_ZERO, Scalar, scalar_abs, scalar_atan2, scalar_cos,
    scalar_nearly_zero, scalar_nearly_zero_within, scalar_round_to_int, scalar_sin, scalar_sqrt,
};

/// `SK_ScalarNearlyZero * SK_ScalarNearlyZero`
pub const K_CROSS_TOLERANCE: Scalar = SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OffsetSegment {
    pub p0: Point,
    pub v: Vector,
}

/// Computes perpDot for point p compared to segment defined by origin p0 and vector v.
/// A positive value means the point is to the left of the segment, negative is to the right,
/// 0 is collinear.
fn compute_side(p0: Point, v: Vector, p: Point) -> i32 {
    let w = p - p0;
    let perp_dot = v.cross(&w);
    if !scalar_nearly_zero_within(perp_dot, K_CROSS_TOLERANCE) {
        return if perp_dot > 0.0 { 1 } else { -1 };
    }
    0
}

/// Returns 1 for cw, -1 for ccw and 0 if zero signed area (either degenerate or
/// self-intersecting).
pub fn sk_get_polygon_winding(polygon_verts: &[Point]) -> i32 {
    if polygon_verts.len() < 3 {
        return 0;
    }

    // compute area and use sign to determine winding
    let mut quad_area = 0.0;
    let mut v0 = polygon_verts[1] - polygon_verts[0];
    for point in polygon_verts.iter().skip(2) {
        let v1 = *point - polygon_verts[0];
        quad_area += v0.cross(&v1);
        v0 = v1;
    }
    if scalar_nearly_zero_within(quad_area, K_CROSS_TOLERANCE) {
        return 0;
    }
    // 1 == ccw, -1 == cw
    if quad_area > 0.0 { 1 } else { -1 }
}

/// Compute difference vector to offset `p0-p1` `offset` units in direction specified by `side`.
pub fn compute_offset_vector(p0: Point, p1: Point, offset: Scalar, side: i32) -> Option<Vector> {
    debug_assert!(side == -1 || side == 1);
    // if distances are equal, can just outset by the perpendicular
    let mut perp = Vector::new(p0.y - p1.y, p1.x - p0.x);
    if !perp.set_length(offset * side as Scalar) {
        return None;
    }
    Some(perp)
}

/// Check interval to see if intersection is in segment.
fn outside_interval(numer: Scalar, denom: Scalar, denom_positive: bool) -> bool {
    (denom_positive && (numer < 0.0 || numer > denom))
        || (!denom_positive && (numer > 0.0 || numer < denom))
}

/// Special zero-length test when we're using vdotv as a denominator.
fn zero_length(v: Vector, vdotv: Scalar) -> bool {
    !(v.is_finite() && vdotv != 0.0)
}

/// Compute the intersection between segments s0 and s1, if any. `s` is the parametric value for
/// the intersection along `s0` and `t` is the same for `s1`.
fn compute_intersection(s0: &OffsetSegment, s1: &OffsetSegment) -> Option<(Point, Scalar, Scalar)> {
    let v0 = s0.v;
    let v1 = s1.v;
    let w = s1.p0 - s0.p0;
    let mut denom = v0.cross(&v1);
    let denom_positive = denom > 0.0;
    let s_numer;
    let t_numer;
    if scalar_nearly_zero_within(denom, K_CROSS_TOLERANCE) {
        // segments are parallel, but not collinear
        if !scalar_nearly_zero_within(w.cross(&v0), K_CROSS_TOLERANCE)
            || !scalar_nearly_zero_within(w.cross(&v1), K_CROSS_TOLERANCE)
        {
            return None;
        }

        // Check for zero-length segments
        let v0dotv0 = v0.dot(&v0);
        if zero_length(v0, v0dotv0) {
            // Both are zero-length
            let v1dotv1 = v1.dot(&v1);
            if zero_length(v1, v1dotv1) {
                // Check if they're the same point
                if !PointPriv::can_normalize(w.x, w.y) {
                    return Some((s0.p0, 0.0, 0.0));
                } else {
                    // Intersection is indeterminate
                    return None;
                }
            }
            // Otherwise project segment0's origin onto segment1
            t_numer = v1.dot(&-w);
            denom = v1dotv1;
            if outside_interval(t_numer, denom, true) {
                return None;
            }
            s_numer = 0.0;
        } else {
            // Project segment1's endpoints onto segment0
            s_numer = v0.dot(&w);
            denom = v0dotv0;
            t_numer = 0.0;
            if outside_interval(s_numer, denom, true) {
                // The first endpoint doesn't lie on segment0
                // If segment1 is degenerate, then there's no collision
                let v1dotv1 = v1.dot(&v1);
                if zero_length(v1, v1dotv1) {
                    return None;
                }

                // Otherwise try the other one
                let old_s_numer = s_numer;
                let s_numer = v0.dot(&(w + v1));
                let t_numer = denom;
                if outside_interval(s_numer, denom, true) {
                    // it's possible that segment1's interval surrounds segment0
                    // this is false if params have the same signs, and in that case no collision
                    if s_numer * old_s_numer > 0.0 {
                        return None;
                    }
                    // otherwise project segment0's endpoint onto segment1 instead
                    let s_numer = 0.0;
                    let t_numer = v1.dot(&-w);
                    denom = v1dotv1;
                    let local_s = s_numer / denom;
                    let local_t = t_numer / denom;
                    return Some((s0.p0 + v0 * local_s, local_s, local_t));
                }
                let local_s = s_numer / denom;
                let local_t = t_numer / denom;
                return Some((s0.p0 + v0 * local_s, local_s, local_t));
            }
        }
    } else {
        s_numer = w.cross(&v1);
        if outside_interval(s_numer, denom, denom_positive) {
            return None;
        }
        t_numer = w.cross(&v0);
        if outside_interval(t_numer, denom, denom_positive) {
            return None;
        }
    }

    let local_s = s_numer / denom;
    let local_t = t_numer / denom;
    Some((s0.p0 + v0 * local_s, local_s, local_t))
}

pub fn sk_is_convex_polygon(polygon_verts: &[Point]) -> bool {
    if polygon_verts.len() < 3 {
        return false;
    }
    let polygon_size = polygon_verts.len();

    let mut last_perp_dot = 0.0;
    let mut x_sign_change_count = 0;
    let mut y_sign_change_count = 0;

    let mut prev_index = polygon_size - 1;
    let mut curr_index = 0;
    let mut next_index = 1;
    let mut v0 = polygon_verts[curr_index] - polygon_verts[prev_index];
    let mut last_vx = v0.x;
    let mut last_vy = v0.y;
    let mut v1 = polygon_verts[next_index] - polygon_verts[curr_index];
    for i in 0..polygon_size {
        if !polygon_verts[i].is_finite() {
            return false;
        }

        // Check that winding direction is always the same (otherwise we have a reflex vertex)
        let perp_dot = v0.cross(&v1);
        if last_perp_dot * perp_dot < 0.0 {
            return false;
        }
        if 0.0 != perp_dot {
            last_perp_dot = perp_dot;
        }

        // Check that the signs of the edge vectors don't change more than twice per coordinate
        if last_vx * v1.x < 0.0 {
            x_sign_change_count += 1;
        }
        if last_vy * v1.y < 0.0 {
            y_sign_change_count += 1;
        }
        if x_sign_change_count > 2 || y_sign_change_count > 2 {
            return false;
        }
        prev_index = curr_index;
        curr_index = next_index;
        next_index = (curr_index + 1) % polygon_size;
        if v1.x != 0.0 {
            last_vx = v1.x;
        }
        if v1.y != 0.0 {
            last_vy = v1.y;
        }
        v0 = v1;
        v1 = polygon_verts[next_index] - polygon_verts[curr_index];
        let _ = prev_index;
    }

    true
}

#[derive(Clone, Debug)]
struct OffsetEdge {
    prev: usize,
    next: usize,
    offset: OffsetSegment,
    intersection: Point,
    t_value: Scalar,
    index: u16,
    end: u16,
}

impl Default for OffsetEdge {
    fn default() -> Self {
        Self {
            prev: usize::MAX,
            next: usize::MAX,
            offset: OffsetSegment::default(),
            intersection: Point::default(),
            t_value: SCALAR_MIN,
            index: 0,
            end: 0,
        }
    }
}

impl OffsetEdge {
    fn init(&mut self, start: u16, end: u16) {
        self.intersection = self.offset.p0;
        self.t_value = SCALAR_MIN;
        self.index = start;
        self.end = end;
    }

    /// Special intersection check that looks for endpoint intersection.
    fn check_intersection(&self, that: &OffsetEdge) -> Option<(Point, Scalar, Scalar)> {
        if self.end == that.index {
            let p1 = self.offset.p0 + self.offset.v;
            if PointPriv::equals_within_tolerance(&p1, &that.offset.p0) {
                return Some((p1, 1.0, 0.0));
            }
        }

        compute_intersection(&self.offset, &that.offset)
    }

    /// Computes the line intersection and then the "distance" from that to this. This is really
    /// a signed squared distance, where negative means that the intersection lies inside
    /// `this.offset`.
    fn compute_crossing_distance(&self, that: &OffsetEdge) -> Scalar {
        let s0 = &self.offset;
        let s1 = &that.offset;
        let v0 = s0.v;
        let v1 = s1.v;

        let denom = v0.cross(&v1);
        if scalar_nearly_zero_within(denom, K_CROSS_TOLERANCE) {
            // segments are parallel
            return SCALAR_MAX;
        }

        let w = s1.p0 - s0.p0;
        let mut local_s = w.cross(&v1) / denom;
        if local_s < 0.0 {
            local_s = -local_s;
        } else {
            local_s -= 1.0;
        }

        local_s *= scalar_abs(local_s);
        local_s *= v0.dot(&v0);

        local_s
    }
}

fn remove_node(nodes: &mut [OffsetEdge], node: usize, head: &mut Option<usize>) {
    // remove from linked list
    let prev = nodes[node].prev;
    let next = nodes[node].next;
    nodes[prev].next = next;
    nodes[next].prev = prev;
    if Some(node) == *head {
        *head = if next == node { None } else { Some(next) };
    }
}

/// Generates a polygon that is inset a constant from the boundary of a given convex polygon.
pub fn sk_inset_convex_polygon(
    input_polygon_verts: &[Point],
    inset: Scalar,
    inset_polygon: &mut Vec<Point>,
) -> bool {
    let input_polygon_size = input_polygon_verts.len();
    if input_polygon_size < 3 {
        return false;
    }

    // restrict this to match other routines
    if input_polygon_size > u16::MAX as usize {
        return false;
    }

    // can't inset by a negative or non-finite amount
    if inset < -SCALAR_NEARLY_ZERO || !is_finite(inset) {
        return false;
    }

    // insetting close to zero just returns the original poly
    if inset <= SCALAR_NEARLY_ZERO {
        inset_polygon.clear();
        inset_polygon.extend_from_slice(input_polygon_verts);
        return true;
    }

    // get winding direction
    let winding = sk_get_polygon_winding(input_polygon_verts);
    if 0 == winding {
        return false;
    }

    // set up
    let mut edge_data = vec![OffsetEdge::default(); input_polygon_size];
    let mut prev = input_polygon_size - 1;
    for curr in 0..input_polygon_size {
        let next = (curr + 1) % input_polygon_size;
        if !input_polygon_verts[curr].is_finite() {
            return false;
        }
        // check for convexity just to be sure
        if compute_side(
            input_polygon_verts[prev],
            input_polygon_verts[curr] - input_polygon_verts[prev],
            input_polygon_verts[next],
        ) * winding
            < 0
        {
            return false;
        }
        let v = input_polygon_verts[next] - input_polygon_verts[curr];
        let mut perp = Vector::new(-v.y, v.x);
        perp.set_length(inset * winding as Scalar);
        edge_data[curr].prev = prev;
        edge_data[curr].next = next;
        edge_data[curr].offset.p0 = input_polygon_verts[curr] + perp;
        edge_data[curr].offset.v = v;
        edge_data[curr].init(0, 0);
        prev = curr;
    }

    let mut head = Some(0usize);
    let mut curr_edge = 0usize;
    let mut prev_edge = edge_data[curr_edge].prev;
    let mut inset_vertex_count = input_polygon_size as i32;
    let mut iterations: u32 = 0;
    let max_iterations = (input_polygon_size * input_polygon_size) as u32;
    while head.is_some() && prev_edge != curr_edge {
        iterations += 1;
        // we should check each edge against each other edge at most once
        if iterations > max_iterations {
            return false;
        }

        if let Some((intersection, s, t)) =
            edge_data[prev_edge].check_intersection(&edge_data[curr_edge])
        {
            // if new intersection is further back on previous inset from the prior intersection
            if s < edge_data[prev_edge].t_value {
                // no point in considering this one again
                remove_node(&mut edge_data, prev_edge, &mut head);
                inset_vertex_count -= 1;
                // go back one segment
                prev_edge = edge_data[prev_edge].prev;
            // we've already considered this intersection, we're done
            } else if edge_data[curr_edge].t_value > SCALAR_MIN
                && PointPriv::equals_within_tolerance_within(
                    &intersection,
                    &edge_data[curr_edge].intersection,
                    1.0e-6,
                )
            {
                break;
            } else {
                // add intersection
                edge_data[curr_edge].intersection = intersection;
                edge_data[curr_edge].t_value = t;

                // go to next segment
                prev_edge = curr_edge;
                curr_edge = edge_data[curr_edge].next;
            }
        } else {
            // if prev to right side of curr
            let side = winding
                * compute_side(
                    edge_data[curr_edge].offset.p0,
                    edge_data[curr_edge].offset.v,
                    edge_data[prev_edge].offset.p0,
                );
            if side < 0
                && side
                    == winding
                        * compute_side(
                            edge_data[curr_edge].offset.p0,
                            edge_data[curr_edge].offset.v,
                            edge_data[prev_edge].offset.p0 + edge_data[prev_edge].offset.v,
                        )
            {
                // no point in considering this one again
                remove_node(&mut edge_data, prev_edge, &mut head);
                inset_vertex_count -= 1;
                // go back one segment
                prev_edge = edge_data[prev_edge].prev;
            } else {
                // move to next segment
                remove_node(&mut edge_data, curr_edge, &mut head);
                inset_vertex_count -= 1;
                curr_edge = edge_data[curr_edge].next;
            }
        }
    }

    // store all the valid intersections that aren't nearly coincident
    inset_polygon.clear();
    let Some(head_index) = head else {
        return false;
    };

    const K_CLEANUP_TOLERANCE: Scalar = 0.01;
    if inset_vertex_count >= 0 {
        inset_polygon.reserve(inset_vertex_count as usize);
    }
    let mut curr_index = 0usize;
    inset_polygon.push(edge_data[head_index].intersection);
    curr_edge = edge_data[head_index].next;
    while curr_edge != head_index {
        if !PointPriv::equals_within_tolerance_within(
            &edge_data[curr_edge].intersection,
            &inset_polygon[curr_index],
            K_CLEANUP_TOLERANCE,
        ) {
            inset_polygon.push(edge_data[curr_edge].intersection);
            curr_index += 1;
        }
        curr_edge = edge_data[curr_edge].next;
    }
    // make sure the first and last points aren't coincident
    if curr_index >= 1
        && PointPriv::equals_within_tolerance_within(
            &inset_polygon[0],
            &inset_polygon[curr_index],
            K_CLEANUP_TOLERANCE,
        )
    {
        inset_polygon.pop();
    }

    sk_is_convex_polygon(inset_polygon)
}

/// Compute the number of points needed for a circular join when offsetting a reflex vertex.
/// Returns `(rot_sin, rot_cos, n)`.
pub fn sk_compute_radial_steps(
    offset0: Vector,
    offset1: Vector,
    offset: Scalar,
) -> Option<(Scalar, Scalar, i32)> {
    const K_RECIP_PIXELS_PER_ARC_SEGMENT: Scalar = 0.25;

    let r_cos = offset0.dot(&offset1);
    if !is_finite(r_cos) {
        return None;
    }
    let r_sin = offset0.cross(&offset1);
    if !is_finite(r_sin) {
        return None;
    }
    let theta = scalar_atan2(r_sin, r_cos);

    let float_steps = scalar_abs(offset * theta * K_RECIP_PIXELS_PER_ARC_SEGMENT);
    // limit the number of steps to at most max uint16_t (that's all we can index)
    // knock one value off the top to account for rounding
    if float_steps >= u16::MAX as Scalar {
        return None;
    }
    let steps = scalar_round_to_int(float_steps);

    let d_theta = if steps > 0 {
        theta / steps as Scalar
    } else {
        0.0
    };
    let rot_sin = scalar_sin(d_theta);
    let rot_cos = scalar_cos(d_theta);
    // Our offset may be so large that we end up with a tiny dTheta, in which case we
    // lose precision when computing rotSin and rotCos.
    if steps > 0 && (rot_sin == 0.0 || rot_cos == 1.0) {
        return None;
    }
    Some((rot_sin, rot_cos, steps))
}

// ---------------------------------------------------------------------------------------------
// Sweep-line simplicity test
// ---------------------------------------------------------------------------------------------

// a point is "left" to another if its x-coord is less, or if equal, its y-coord is greater.
// The reference deliberately uses `!(a > b)` rather than `a <= b`, which differs only for NaN;
// the inputs are checked finite before these predicates are used.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn left(p0: Point, p1: Point) -> bool {
    p0.x < p1.x || (!(p0.x > p1.x) && p0.y > p1.y)
}

// a point is "right" to another if its x-coord is greater, or if equal, its y-coord is less
#[allow(dead_code)] // reached only by the debug-only ordering checks, as in SkASSERT
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn right(p0: Point, p1: Point) -> bool {
    p0.x > p1.x || (!(p0.x < p1.x) && p0.y < p1.y)
}

#[derive(Clone, Copy, Debug, Default)]
struct Vertex {
    position: Point,
    index: u16,
    prev_index: u16,
    next_index: u16,
    flags: u16,
}

const K_PREV_LEFT_VERTEX_FLAG: u16 = 0x1;
const K_NEXT_LEFT_VERTEX_FLAG: u16 = 0x2;

/// Faithful port of `SkTDPQueue<Vertex, Vertex::Left>`: a binary heap whose `LESS` is
/// `left(a.position, b.position)`.
#[derive(Clone, Debug, Default)]
struct VertexQueue {
    heap: Vec<Vertex>,
}

impl VertexQueue {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: Vec::with_capacity(capacity),
        }
    }

    fn count(&self) -> usize {
        self.heap.len()
    }

    fn peek(&self) -> &Vertex {
        &self.heap[0]
    }

    fn pop(&mut self) {
        if 1 == self.heap.len() {
            self.heap.pop();
            return;
        }
        let last = self.heap.pop().unwrap();
        self.heap[0] = last;
        self.percolate_down_if_necessary(0);
    }

    fn insert(&mut self, entry: Vertex) {
        self.heap.push(entry);
        let index = self.heap.len() - 1;
        self.percolate_up_if_necessary(index);
    }

    fn percolate_up_if_necessary(&mut self, mut index: usize) -> bool {
        let mut percolated = false;
        loop {
            if 0 == index {
                return percolated;
            }
            let p = (index - 1) >> 1;
            if left(self.heap[index].position, self.heap[p].position) {
                self.heap.swap(index, p);
                index = p;
                percolated = true;
            } else {
                return percolated;
            }
        }
    }

    fn percolate_down_if_necessary(&mut self, mut index: usize) {
        loop {
            let mut child = 2 * index + 1;
            if child >= self.heap.len() {
                return;
            }
            if child + 1 >= self.heap.len() {
                if left(self.heap[child].position, self.heap[index].position) {
                    self.heap.swap(child, index);
                    return;
                }
            } else if left(self.heap[child + 1].position, self.heap[child].position) {
                child += 1;
            }

            if left(self.heap[child].position, self.heap[index].position) {
                self.heap.swap(child, index);
                index = child;
            } else {
                return;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ActiveEdge {
    segment: OffsetSegment,
    index0: u16,
    index1: u16,
    child: [Option<usize>; 2],
    above: Option<usize>,
    below: Option<usize>,
    red: bool,
}

impl Default for ActiveEdge {
    fn default() -> Self {
        Self {
            segment: OffsetSegment::default(),
            index0: 0,
            index1: 0,
            child: [None, None],
            above: None,
            below: None,
            red: false,
        }
    }
}

impl ActiveEdge {
    fn new(p0: Point, v: Vector, index0: u16, index1: u16) -> Self {
        Self {
            segment: OffsetSegment { p0, v },
            index0,
            index1,
            child: [None, None],
            above: None,
            below: None,
            red: true,
        }
    }

    /// Returns true if "this" is above "that", assuming this.p0 is to the left of that.p0.
    ///
    /// `above_if_left`, `above` and `less_than` reproduce the reference's debug-only ordering
    /// verification (`VerifyTree`/`lessThan`) and are only reached from `debug_assert!`s.
    #[allow(dead_code)]
    fn above_if_left(&self, that: &ActiveEdge) -> bool {
        let p0 = self.segment.p0;
        let q0 = that.segment.p0;
        debug_assert!(p0.x <= q0.x);
        let mut d = q0 - p0;
        let v = self.segment.v;
        let w = that.segment.v;
        if self.index0 != that.index0 {
            let cross = d.cross(&v);
            if cross > K_CROSS_TOLERANCE {
                return true;
            } else if cross < -K_CROSS_TOLERANCE {
                return false;
            }
        } else if self.index1 == that.index1 {
            return false;
        }
        // d = that.P1 - this.P0
        d += w;
        let cross = d.cross(&v);
        if cross > K_CROSS_TOLERANCE {
            return true;
        } else if cross < -K_CROSS_TOLERANCE {
            return false;
        }
        // If the previous check fails, the two segments are nearly collinear
        if p0.x < q0.x {
            p0.y >= q0.y
        } else if p0.y > q0.y {
            true
        } else if p0.y < q0.y {
            false
        } else {
            // The first endpoints are the same, so check the other endpoint
            let p1 = p0 + v;
            let q1 = q0 + w;
            if p1.x < q1.x {
                p1.y >= q1.y
            } else {
                p1.y > q1.y
            }
        }
    }

    /// Same as `above_if_left`, but generalized.
    #[allow(dead_code)]
    fn above(&self, that: &ActiveEdge) -> bool {
        let p0 = self.segment.p0;
        let q0 = that.segment.p0;
        if right(p0, q0) {
            !that.above_if_left(self)
        } else {
            self.above_if_left(that)
        }
    }

    fn intersect(&self, q0: Point, w: Vector, index0: u16, index1: u16) -> bool {
        // check first to see if these edges are neighbors in the polygon
        if self.index0 == index0
            || self.index1 == index0
            || self.index0 == index1
            || self.index1 == index1
        {
            return false;
        }

        let p0 = self.segment.p0;
        let v = self.segment.v;
        let p1 = p0 + v;
        let q1 = q0 + w;

        debug_assert!(q0.x <= p1.x + SCALAR_NEARLY_ZERO);

        // if each segment straddles the other (i.e., the endpoints have different sides),
        // then they intersect
        if p0.x < q0.x {
            if q1.x < p1.x {
                compute_side(p0, v, q0) * compute_side(p0, v, q1) < 0
            } else {
                compute_side(p0, v, q0) * compute_side(q0, w, p1) > 0
            }
        } else if p1.x < q1.x {
            compute_side(q0, w, p0) * compute_side(q0, w, p1) < 0
        } else {
            compute_side(q0, w, p0) * compute_side(p0, v, q1) > 0
        }
    }

    fn intersect_edge(&self, edge: &ActiveEdge) -> bool {
        self.intersect(edge.segment.p0, edge.segment.v, edge.index0, edge.index1)
    }

    #[allow(dead_code)]
    fn less_than(&self, that: &ActiveEdge) -> bool {
        debug_assert!(!self.above(self));
        debug_assert!(!that.above(that));
        debug_assert!(!(self.above(that) && that.above(self)));
        self.above(that)
    }

    fn equals(&self, index0: u16, index1: u16) -> bool {
        self.index0 == index0 && self.index1 == index1
    }
}

#[derive(Clone, Debug)]
struct ActiveEdgeList {
    /// `nodes[0]` is the tree head sentinel, mirroring `fTreeHead`.
    nodes: Vec<ActiveEdge>,
    curr_free: usize,
    max_free: usize,
}

impl ActiveEdgeList {
    fn new(max_edges: usize) -> Self {
        Self {
            nodes: vec![ActiveEdge::default()],
            curr_free: 0,
            max_free: max_edges,
        }
    }

    fn allocate(&mut self, p0: Point, v: Vector, index0: u16, index1: u16) -> Option<usize> {
        if self.curr_free >= self.max_free {
            return None;
        }
        let node = ActiveEdge::new(p0, v, index0, index1);
        self.nodes.push(node);
        self.curr_free += 1;
        Some(self.nodes.len() - 1)
    }

    fn is_red(&self, node: Option<usize>) -> bool {
        match node {
            Some(node) => self.nodes[node].red,
            None => false,
        }
    }

    fn single_rotation(&mut self, node: usize, dir: usize) -> usize {
        let tmp = self.nodes[node].child[1 - dir].unwrap();

        self.nodes[node].child[1 - dir] = self.nodes[tmp].child[dir];
        self.nodes[tmp].child[dir] = Some(node);

        self.nodes[node].red = true;
        self.nodes[tmp].red = false;

        tmp
    }

    fn double_rotation(&mut self, node: usize, dir: usize) -> usize {
        let child = self.nodes[node].child[1 - dir].unwrap();
        let rotated = self.single_rotation(child, 1 - dir);
        self.nodes[node].child[1 - dir] = Some(rotated);

        self.single_rotation(node, dir)
    }

    fn insert(&mut self, p0: Point, p1: Point, index0: u16, index1: u16) -> bool {
        let v = p1 - p0;
        if !v.is_finite() {
            return false;
        }
        // empty tree case -- easy
        if self.nodes[0].child[1].is_none() {
            let Some(root) = self.allocate(p0, v, index0, index1) else {
                return false;
            };
            self.nodes[0].child[1] = Some(root);
            self.nodes[root].red = false;
            return true;
        }

        // set up helpers
        let mut top = 0usize;
        let mut grandparent: Option<usize> = None;
        let mut parent: Option<usize> = None;
        let mut curr = self.nodes[top].child[1];
        let mut dir = 0usize;
        let mut last = 0usize;
        // predecessor and successor, for intersection check
        let mut pred: Option<usize> = None;
        let mut succ: Option<usize> = None;

        // search down the tree
        loop {
            let Some(current) = curr else {
                // check for intersection with predecessor and successor
                if let Some(pred) = pred {
                    if self.nodes[pred].intersect(p0, v, index0, index1) {
                        return false;
                    }
                }
                if let Some(succ) = succ {
                    if self.nodes[succ].intersect(p0, v, index0, index1) {
                        return false;
                    }
                }
                // insert new node at bottom
                let parent_index = parent.unwrap();
                let Some(new_node) = self.allocate(p0, v, index0, index1) else {
                    return false;
                };
                self.nodes[parent_index].child[dir] = Some(new_node);
                self.nodes[new_node].above = pred;
                self.nodes[new_node].below = succ;
                if let Some(pred) = pred {
                    if self.nodes[pred].segment.p0 == self.nodes[new_node].segment.p0
                        && self.nodes[pred].segment.v == self.nodes[new_node].segment.v
                    {
                        return false;
                    }
                    self.nodes[pred].below = Some(new_node);
                }
                if let Some(succ) = succ {
                    if self.nodes[succ].segment.p0 == self.nodes[new_node].segment.p0
                        && self.nodes[succ].segment.v == self.nodes[new_node].segment.v
                    {
                        return false;
                    }
                    self.nodes[succ].above = Some(new_node);
                }
                if self.is_red(parent) {
                    let gp = grandparent.unwrap();
                    let dir2 = usize::from(self.nodes[top].child[1] == Some(gp));
                    let parent_index = parent.unwrap();
                    if Some(new_node) == self.nodes[parent_index].child[last] {
                        let rot = self.single_rotation(gp, 1 - last);
                        self.nodes[top].child[dir2] = Some(rot);
                    } else {
                        let rot = self.double_rotation(gp, 1 - last);
                        self.nodes[top].child[dir2] = Some(rot);
                    }
                }
                break;
            };

            if self.is_red(self.nodes[current].child[0])
                && self.is_red(self.nodes[current].child[1])
            {
                // color flip
                self.nodes[current].red = true;
                let c0 = self.nodes[current].child[0].unwrap();
                let c1 = self.nodes[current].child[1].unwrap();
                self.nodes[c0].red = false;
                self.nodes[c1].red = false;
                if self.is_red(parent) {
                    let gp = grandparent.unwrap();
                    let dir2 = usize::from(self.nodes[top].child[1] == Some(gp));
                    let parent_index = parent.unwrap();
                    if Some(current) == self.nodes[parent_index].child[last] {
                        let rot = self.single_rotation(gp, 1 - last);
                        self.nodes[top].child[dir2] = Some(rot);
                    } else {
                        let rot = self.double_rotation(gp, 1 - last);
                        self.nodes[top].child[dir2] = Some(rot);
                    }
                }
            }

            last = dir;
            // check to see if segment is above or below
            let side = if self.nodes[current].index0 == index0 {
                compute_side(
                    self.nodes[current].segment.p0,
                    self.nodes[current].segment.v,
                    p1,
                )
            } else {
                compute_side(
                    self.nodes[current].segment.p0,
                    self.nodes[current].segment.v,
                    p0,
                )
            };
            if 0 == side {
                return false;
            }
            dir = usize::from(side < 0);

            if 0 == dir {
                succ = Some(current);
            } else {
                pred = Some(current);
            }

            // update helpers
            if let Some(gp) = grandparent {
                top = gp;
            }
            grandparent = parent;
            parent = Some(current);
            curr = self.nodes[current].child[dir];
        }

        // update root and make it black
        if let Some(root) = self.nodes[0].child[1] {
            self.nodes[root].red = false;
        }
        true
    }

    /// Replaces edge p0p1 with p1p2.
    fn replace(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        index0: u16,
        index1: u16,
        index2: u16,
    ) -> bool {
        if self.nodes[0].child[1].is_none() {
            return false;
        }

        let v = p2 - p1;
        let mut curr = 0usize;
        let mut found: Option<usize> = None;
        let mut dir = 1usize;

        // search
        while let Some(child) = self.nodes[curr].child[dir] {
            // update helpers
            curr = child;
            // save found node
            if self.nodes[curr].equals(index0, index1) {
                found = Some(curr);
                break;
            } else {
                // check to see if segment is above or below
                let side = if self.nodes[curr].index1 == index1 {
                    compute_side(self.nodes[curr].segment.p0, self.nodes[curr].segment.v, p0)
                } else {
                    compute_side(self.nodes[curr].segment.p0, self.nodes[curr].segment.v, p1)
                };
                if 0 == side {
                    return false;
                }
                dir = usize::from(side < 0);
            }
        }

        let Some(found) = found else {
            return false;
        };

        // replace if found
        let pred = self.nodes[found].above;
        let succ = self.nodes[found].below;
        // check deletion and insert intersection cases
        if let Some(pred) = pred {
            if self.nodes[pred].intersect_edge(&self.nodes[found])
                || self.nodes[pred].intersect(p1, v, index1, index2)
            {
                return false;
            }
        }
        if let Some(succ) = succ {
            if self.nodes[succ].intersect_edge(&self.nodes[found])
                || self.nodes[succ].intersect(p1, v, index1, index2)
            {
                return false;
            }
        }
        self.nodes[found].segment.p0 = p1;
        self.nodes[found].segment.v = v;
        self.nodes[found].index0 = index1;
        self.nodes[found].index1 = index2;
        // above and below should stay the same
        true
    }

    fn remove(&mut self, p0: Point, p1: Point, index0: u16, index1: u16) -> bool {
        if self.nodes[0].child[1].is_none() {
            return false;
        }

        let mut curr = 0usize;
        let mut parent: Option<usize> = None;
        let mut grandparent: Option<usize>;
        let mut found: Option<usize> = None;
        let mut dir = 1usize;

        // search and push a red node down
        while self.nodes[curr].child[dir].is_some() {
            let last = dir;

            // update helpers
            grandparent = parent;
            parent = Some(curr);
            curr = self.nodes[curr].child[dir].unwrap();
            // save found node
            if self.nodes[curr].equals(index0, index1) {
                found = Some(curr);
                dir = 0;
            } else {
                // check to see if segment is above or below
                let side = if self.nodes[curr].index1 == index1 {
                    compute_side(self.nodes[curr].segment.p0, self.nodes[curr].segment.v, p0)
                } else {
                    compute_side(self.nodes[curr].segment.p0, self.nodes[curr].segment.v, p1)
                };
                if 0 == side {
                    return false;
                }
                dir = usize::from(side < 0);
            }

            // push the red node down
            if !self.is_red(Some(curr)) && !self.is_red(self.nodes[curr].child[dir]) {
                if self.is_red(self.nodes[curr].child[1 - dir]) {
                    let p = parent.unwrap();
                    let rotated = self.single_rotation(curr, dir);
                    self.nodes[p].child[last] = Some(rotated);
                    parent = Some(rotated);
                } else {
                    let p = parent.unwrap();
                    let s = self.nodes[p].child[1 - last];

                    if let Some(s) = s {
                        if !self.is_red(self.nodes[s].child[1 - last])
                            && !self.is_red(self.nodes[s].child[last])
                        {
                            // color flip
                            self.nodes[p].red = false;
                            self.nodes[s].red = true;
                            self.nodes[curr].red = true;
                        } else {
                            let gp = grandparent.unwrap();
                            let dir2 = usize::from(self.nodes[gp].child[1] == Some(p));

                            if self.is_red(self.nodes[s].child[last]) {
                                let rotated = self.double_rotation(p, last);
                                self.nodes[gp].child[dir2] = Some(rotated);
                            } else if self.is_red(self.nodes[s].child[1 - last]) {
                                let rotated = self.single_rotation(p, last);
                                self.nodes[gp].child[dir2] = Some(rotated);
                            }

                            // ensure correct coloring
                            let subtree = self.nodes[gp].child[dir2].unwrap();
                            self.nodes[curr].red = true;
                            self.nodes[subtree].red = true;
                            let c0 = self.nodes[subtree].child[0].unwrap();
                            let c1 = self.nodes[subtree].child[1].unwrap();
                            self.nodes[c0].red = false;
                            self.nodes[c1].red = false;
                        }
                    }
                }
            }
        }

        // replace and remove if found
        if let Some(found) = found {
            let pred = self.nodes[found].above;
            let succ = self.nodes[found].below;
            if let Some(pred) = pred {
                if self.nodes[pred].intersect_edge(&self.nodes[found]) {
                    return false;
                }
            }
            if let Some(succ) = succ {
                if self.nodes[succ].intersect_edge(&self.nodes[found]) {
                    return false;
                }
            }
            let pred = if found != curr {
                self.nodes[found].segment = self.nodes[curr].segment;
                self.nodes[found].index0 = self.nodes[curr].index0;
                self.nodes[found].index1 = self.nodes[curr].index1;
                self.nodes[found].above = self.nodes[curr].above;
                self.nodes[found].above
            } else {
                if let Some(succ) = succ {
                    self.nodes[succ].above = pred;
                }
                pred
            };
            if let Some(pred) = pred {
                self.nodes[pred].below = self.nodes[curr].below;
            }
            let p = parent.unwrap();
            let child_index = usize::from(self.nodes[p].child[1] == Some(curr));
            let replacement =
                self.nodes[curr].child[usize::from(self.nodes[curr].child[0].is_none())];
            self.nodes[p].child[child_index] = replacement;

            // `curr` is no longer reachable; the reference tags it with 0xdeadbeef here.
        }

        // update root and make it black
        if let Some(root) = self.nodes[0].child[1] {
            self.nodes[root].red = false;
        }
        true
    }
}

/// Here we implement a sweep line algorithm to determine whether the provided points represent
/// a simple polygon, i.e., the polygon is non-self-intersecting.
pub fn sk_is_simple_polygon(polygon: &[Point]) -> bool {
    let polygon_size = polygon.len();
    if polygon_size < 3 {
        return false;
    }

    // If it's convex, it's simple
    if sk_is_convex_polygon(polygon) {
        return true;
    }

    // practically speaking, it takes too long to process large polygons
    if polygon_size > 2048 {
        return false;
    }

    let mut vertex_queue = VertexQueue::with_capacity(polygon_size);
    for i in 0..polygon_size {
        if !polygon[i].is_finite() {
            return false;
        }
        let mut new_vertex = Vertex {
            position: polygon[i],
            index: i as u16,
            prev_index: ((i + polygon_size - 1) % polygon_size) as u16,
            next_index: ((i + 1) % polygon_size) as u16,
            flags: 0,
        };
        // The two edges adjacent to this vertex are the same, so polygon is not simple
        if polygon[new_vertex.prev_index as usize] == polygon[new_vertex.next_index as usize] {
            return false;
        }
        if left(polygon[new_vertex.prev_index as usize], polygon[i]) {
            new_vertex.flags |= K_PREV_LEFT_VERTEX_FLAG;
        }
        if left(polygon[new_vertex.next_index as usize], polygon[i]) {
            new_vertex.flags |= K_NEXT_LEFT_VERTEX_FLAG;
        }
        vertex_queue.insert(new_vertex);
    }

    // pop each vertex from the queue and generate events depending on
    // where it lies relative to its neighboring edges
    let mut sweep_line = ActiveEdgeList::new(polygon_size);
    while vertex_queue.count() > 0 {
        let v = *vertex_queue.peek();

        // both to the right -- insert both
        if v.flags == 0 {
            if !sweep_line.insert(
                v.position,
                polygon[v.prev_index as usize],
                v.index,
                v.prev_index,
            ) {
                break;
            }
            if !sweep_line.insert(
                v.position,
                polygon[v.next_index as usize],
                v.index,
                v.next_index,
            ) {
                break;
            }
        // both to the left -- remove both
        } else if v.flags == (K_PREV_LEFT_VERTEX_FLAG | K_NEXT_LEFT_VERTEX_FLAG) {
            if !sweep_line.remove(
                polygon[v.prev_index as usize],
                v.position,
                v.prev_index,
                v.index,
            ) {
                break;
            }
            if !sweep_line.remove(
                polygon[v.next_index as usize],
                v.position,
                v.next_index,
                v.index,
            ) {
                break;
            }
        // one to left and right -- replace one with another
        } else if v.flags & K_PREV_LEFT_VERTEX_FLAG != 0 {
            if !sweep_line.replace(
                polygon[v.prev_index as usize],
                v.position,
                polygon[v.next_index as usize],
                v.prev_index,
                v.index,
                v.next_index,
            ) {
                break;
            }
        } else {
            debug_assert!(v.flags & K_NEXT_LEFT_VERTEX_FLAG != 0);
            if !sweep_line.replace(
                polygon[v.next_index as usize],
                v.position,
                polygon[v.prev_index as usize],
                v.next_index,
                v.index,
                v.prev_index,
            ) {
                break;
            }
        }

        vertex_queue.pop();
    }

    vertex_queue.count() == 0
}

// ---------------------------------------------------------------------------------------------
// Simple-polygon offsetting
// ---------------------------------------------------------------------------------------------

fn setup_offset_edge(
    curr_edge: &mut OffsetEdge,
    endpoint0: Point,
    endpoint1: Point,
    start_index: u16,
    end_index: u16,
) {
    curr_edge.offset.p0 = endpoint0;
    curr_edge.offset.v = endpoint1 - endpoint0;
    curr_edge.init(start_index, end_index);
}

fn is_reflex_vertex(
    input_polygon_verts: &[Point],
    winding: i32,
    offset: Scalar,
    prev_index: u16,
    curr_index: u16,
    next_index: u16,
) -> bool {
    let side = compute_side(
        input_polygon_verts[prev_index as usize],
        input_polygon_verts[curr_index as usize] - input_polygon_verts[prev_index as usize],
        input_polygon_verts[next_index as usize],
    );
    // if reflex point, we need to add extra edges
    (side as Scalar) * (winding as Scalar) * offset < 0.0
}

/// Generates a simple polygon (if possible) that is offset a constant distance from the boundary
/// of a given simple polygon.
pub fn sk_offset_simple_polygon(
    input_polygon_verts: &[Point],
    bounds: &Rect,
    offset: Scalar,
    offset_polygon: &mut Vec<Point>,
    mut polygon_indices: Option<&mut Vec<i32>>,
) -> bool {
    let input_polygon_size = input_polygon_verts.len();
    if input_polygon_size < 3 {
        return false;
    }

    // need to be able to represent all the vertices in the 16-bit indices
    if input_polygon_size >= u16::MAX as usize {
        return false;
    }

    if !is_finite(offset) {
        return false;
    }

    // can't inset more than the half bounds of the polygon
    if offset
        > scalar_abs(RectPriv::half_width(bounds)).min(scalar_abs(RectPriv::half_height(bounds)))
    {
        return false;
    }

    // offsetting close to zero just returns the original poly
    if scalar_nearly_zero(offset) {
        offset_polygon.clear();
        if let Some(indices) = polygon_indices.as_deref_mut() {
            indices.clear();
        }
        for (i, point) in input_polygon_verts.iter().enumerate() {
            offset_polygon.push(*point);
            if let Some(indices) = polygon_indices.as_deref_mut() {
                indices.push(i as i32);
            }
        }
        return true;
    }

    // get winding direction
    let winding = sk_get_polygon_winding(input_polygon_verts);
    if 0 == winding {
        return false;
    }

    // build normals
    let mut normals = vec![Vector::default(); input_polygon_size];
    let mut num_edges: u32 = 0;
    let mut prev_index = input_polygon_size - 1;
    for curr_index in 0..input_polygon_size {
        if !input_polygon_verts[curr_index].is_finite() {
            return false;
        }
        let next_index = (curr_index + 1) % input_polygon_size;
        let Some(normal) = compute_offset_vector(
            input_polygon_verts[curr_index],
            input_polygon_verts[next_index],
            offset,
            winding,
        ) else {
            return false;
        };
        normals[curr_index] = normal;
        if curr_index > 0 {
            // if reflex point, we need to add extra edges
            if is_reflex_vertex(
                input_polygon_verts,
                winding,
                offset,
                prev_index as u16,
                curr_index as u16,
                next_index as u16,
            ) {
                let Some((_rot_sin, _rot_cos, num_steps)) =
                    sk_compute_radial_steps(normals[prev_index], normals[curr_index], offset)
                else {
                    return false;
                };
                num_edges += num_steps.max(1) as u32;
            }
        }
        num_edges += 1;
        prev_index = curr_index;
    }
    // finish up the edge counting
    if is_reflex_vertex(
        input_polygon_verts,
        winding,
        offset,
        (input_polygon_size - 1) as u16,
        0,
        1,
    ) {
        let Some((_rot_sin, _rot_cos, num_steps)) =
            sk_compute_radial_steps(normals[input_polygon_size - 1], normals[0], offset)
        else {
            return false;
        };
        num_edges += num_steps.max(1) as u32;
    }

    // Make sure we don't overflow the max array count.
    if num_edges > i32::MAX as u32 {
        return false;
    }

    // build initial offset edge list
    let mut edge_data: Vec<OffsetEdge> = Vec::with_capacity(num_edges as usize);
    let mut prev_edge: Option<usize> = None;
    let mut prev_index = input_polygon_size - 1;
    for curr_index in 0..input_polygon_size {
        let next_index = (curr_index + 1) % input_polygon_size;
        // if reflex point, fill in curve
        if is_reflex_vertex(
            input_polygon_verts,
            winding,
            offset,
            prev_index as u16,
            curr_index as u16,
            next_index as u16,
        ) {
            let mut prev_normal = normals[prev_index];
            let Some((rot_sin, rot_cos, num_steps)) =
                sk_compute_radial_steps(prev_normal, normals[curr_index], offset)
            else {
                return false;
            };
            for _ in 0..num_steps - 1 {
                let curr_normal = Vector::new(
                    prev_normal.x * rot_cos - prev_normal.y * rot_sin,
                    prev_normal.y * rot_cos + prev_normal.x * rot_sin,
                );
                let edge = edge_data.len();
                edge_data.push(OffsetEdge::default());
                setup_offset_edge(
                    &mut edge_data[edge],
                    input_polygon_verts[curr_index] + prev_normal,
                    input_polygon_verts[curr_index] + curr_normal,
                    curr_index as u16,
                    curr_index as u16,
                );
                prev_normal = curr_normal;
                edge_data[edge].prev = prev_edge.unwrap_or(edge);
                if let Some(prev) = prev_edge {
                    edge_data[prev].next = edge;
                }
                prev_edge = Some(edge);
            }
            let edge = edge_data.len();
            edge_data.push(OffsetEdge::default());
            setup_offset_edge(
                &mut edge_data[edge],
                input_polygon_verts[curr_index] + prev_normal,
                input_polygon_verts[curr_index] + normals[curr_index],
                curr_index as u16,
                curr_index as u16,
            );
            edge_data[edge].prev = prev_edge.unwrap_or(edge);
            if let Some(prev) = prev_edge {
                edge_data[prev].next = edge;
            }
            prev_edge = Some(edge);
        }

        // Add the edge
        let edge = edge_data.len();
        edge_data.push(OffsetEdge::default());
        setup_offset_edge(
            &mut edge_data[edge],
            input_polygon_verts[curr_index] + normals[curr_index],
            input_polygon_verts[next_index] + normals[curr_index],
            curr_index as u16,
            next_index as u16,
        );
        edge_data[edge].prev = prev_edge.unwrap_or(edge);
        if let Some(prev) = prev_edge {
            edge_data[prev].next = edge;
        }
        prev_edge = Some(edge);
        prev_index = curr_index;
    }
    // close up the linked list
    let last_edge = prev_edge.unwrap();
    edge_data[last_edge].next = 0;
    edge_data[0].prev = last_edge;

    // now clip edges
    debug_assert!(edge_data.len() == num_edges as usize);
    let mut head = Some(0usize);
    let mut curr_edge = 0usize;
    let mut prev_edge = last_edge;
    let mut offset_vertex_count = num_edges;
    let mut iterations: u64 = 0;
    let max_iterations = (num_edges as u64) * (num_edges as u64);
    while head.is_some() && prev_edge != curr_edge && offset_vertex_count > 0 {
        iterations += 1;
        // we should check each edge against each other edge at most once
        if iterations > max_iterations {
            return false;
        }

        if let Some((intersection, s, t)) =
            edge_data[prev_edge].check_intersection(&edge_data[curr_edge])
        {
            // if new intersection is further back on previous inset from the prior intersection
            if s < edge_data[prev_edge].t_value {
                // no point in considering this one again
                remove_node(&mut edge_data, prev_edge, &mut head);
                offset_vertex_count -= 1;
                // go back one segment
                prev_edge = edge_data[prev_edge].prev;
                // we've already considered this intersection, we're done
            } else if edge_data[curr_edge].t_value > SCALAR_MIN
                && PointPriv::equals_within_tolerance_within(
                    &intersection,
                    &edge_data[curr_edge].intersection,
                    1.0e-6,
                )
            {
                break;
            } else {
                // add intersection
                edge_data[curr_edge].intersection = intersection;
                edge_data[curr_edge].t_value = t;
                edge_data[curr_edge].index = edge_data[prev_edge].end;

                // go to next segment
                prev_edge = curr_edge;
                curr_edge = edge_data[curr_edge].next;
            }
        } else {
            // If there is no intersection, we want to minimize the distance between
            // the point where the segment lines cross and the segments themselves.
            let prev_prev_edge = edge_data[prev_edge].prev;
            let curr_next_edge = edge_data[curr_edge].next;
            let dist0 = edge_data[curr_edge].compute_crossing_distance(&edge_data[prev_prev_edge]);
            let dist1 = edge_data[prev_edge].compute_crossing_distance(&edge_data[curr_next_edge]);
            // if both lead to direct collision
            if dist0 < 0.0 && dist1 < 0.0 {
                // check first to see if either represent parts of one contour
                let p1 = edge_data[prev_prev_edge].offset.p0 + edge_data[prev_prev_edge].offset.v;
                let prev_same_contour =
                    PointPriv::equals_within_tolerance(&p1, &edge_data[prev_edge].offset.p0);
                let p1 = edge_data[curr_edge].offset.p0 + edge_data[curr_edge].offset.v;
                let curr_same_contour =
                    PointPriv::equals_within_tolerance(&p1, &edge_data[curr_next_edge].offset.p0);

                // want to step along contour to find intersections rather than jump to new one
                if curr_same_contour && !prev_same_contour {
                    remove_node(&mut edge_data, curr_edge, &mut head);
                    curr_edge = curr_next_edge;
                    offset_vertex_count -= 1;
                    continue;
                } else if prev_same_contour && !curr_same_contour {
                    remove_node(&mut edge_data, prev_edge, &mut head);
                    prev_edge = prev_prev_edge;
                    offset_vertex_count -= 1;
                    continue;
                }
            }

            // otherwise minimize collision distance along segment
            if dist0 < dist1 {
                remove_node(&mut edge_data, prev_edge, &mut head);
                prev_edge = prev_prev_edge;
            } else {
                remove_node(&mut edge_data, curr_edge, &mut head);
                curr_edge = curr_next_edge;
            }
            offset_vertex_count -= 1;
        }
    }

    // store all the valid intersections that aren't nearly coincident
    offset_polygon.clear();
    let Some(head_index) = head else {
        return false;
    };
    if offset_vertex_count == 0 || offset_vertex_count >= u16::MAX as u32 {
        return false;
    }
    if let Some(indices) = polygon_indices.as_deref_mut() {
        indices.clear();
    }

    const K_CLEANUP_TOLERANCE: Scalar = 0.01;
    offset_polygon.reserve(offset_vertex_count as usize);
    let mut curr_index = 0usize;
    offset_polygon.push(edge_data[head_index].intersection);
    if let Some(indices) = polygon_indices.as_deref_mut() {
        indices.push(edge_data[head_index].index as i32);
    }
    curr_edge = edge_data[head_index].next;
    while curr_edge != head_index {
        if !PointPriv::equals_within_tolerance_within(
            &edge_data[curr_edge].intersection,
            &offset_polygon[curr_index],
            K_CLEANUP_TOLERANCE,
        ) {
            offset_polygon.push(edge_data[curr_edge].intersection);
            if let Some(indices) = polygon_indices.as_deref_mut() {
                indices.push(edge_data[curr_edge].index as i32);
            }
            curr_index += 1;
        }
        curr_edge = edge_data[curr_edge].next;
    }
    // make sure the first and last points aren't coincident
    if curr_index >= 1
        && PointPriv::equals_within_tolerance_within(
            &offset_polygon[0],
            &offset_polygon[curr_index],
            K_CLEANUP_TOLERANCE,
        )
    {
        offset_polygon.pop();
        if let Some(indices) = polygon_indices {
            indices.pop();
        }
    }

    // check winding of offset polygon (it should be same as the original polygon)
    let offset_winding = sk_get_polygon_winding(offset_polygon);

    winding * offset_winding > 0 && sk_is_simple_polygon(offset_polygon)
}

// ---------------------------------------------------------------------------------------------
// Triangulation
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum VertexType {
    #[default]
    Convex,
    Reflex,
}

#[derive(Clone, Debug, Default)]
struct TriangulationVertex {
    position: Point,
    vertex_type: VertexType,
    index: u16,
    prev_index: u16,
    next_index: u16,
    /// Intrusive link used by the convex list; `None` when not linked.
    link_prev: Option<usize>,
    link_next: Option<usize>,
}

/// Intrusive doubly linked list over `TriangulationVertex` indices.
#[derive(Clone, Copy, Debug, Default)]
struct ConvexList {
    head: Option<usize>,
    tail: Option<usize>,
}

impl ConvexList {
    fn add_to_tail(&mut self, nodes: &mut [TriangulationVertex], value: usize) {
        nodes[value].link_next = None;
        nodes[value].link_prev = self.tail;
        if let Some(tail) = self.tail {
            nodes[tail].link_next = Some(value);
        } else {
            self.head = Some(value);
        }
        self.tail = Some(value);
    }

    fn add_to_head(&mut self, nodes: &mut [TriangulationVertex], value: usize) {
        nodes[value].link_prev = None;
        nodes[value].link_next = self.head;
        if let Some(head) = self.head {
            nodes[head].link_prev = Some(value);
        } else {
            self.tail = Some(value);
        }
        self.head = Some(value);
    }

    fn remove(&mut self, nodes: &mut [TriangulationVertex], value: usize) {
        let prev = nodes[value].link_prev;
        let next = nodes[value].link_next;
        if let Some(prev) = prev {
            nodes[prev].link_next = next;
        } else {
            self.head = next;
        }
        if let Some(next) = next {
            nodes[next].link_prev = prev;
        } else {
            self.tail = prev;
        }
        nodes[value].link_prev = None;
        nodes[value].link_next = None;
    }
}

fn compute_triangle_bounds(p0: Point, p1: Point, p2: Point) -> Rect {
    Rect::new(
        p0.x.min(p1.x).min(p2.x),
        p0.y.min(p1.y).min(p2.y),
        p0.x.max(p1.x).max(p2.x),
        p0.y.max(p1.y).max(p2.y),
    )
}

/// Test to see if point p is in triangle p0p1p2. For now assuming strictly inside -- if on the
/// edge it's outside.
fn point_in_triangle(p0: Point, p1: Point, p2: Point, p: Point) -> bool {
    let v0 = p1 - p0;
    let v1 = p2 - p1;
    let n = v0.cross(&v1);

    let w0 = p - p0;
    if n * v0.cross(&w0) < SCALAR_NEARLY_ZERO {
        return false;
    }

    let w1 = p - p1;
    if n * v1.cross(&w1) < SCALAR_NEARLY_ZERO {
        return false;
    }

    let v2 = p0 - p2;
    let w2 = p - p2;
    if n * v2.cross(&w2) < SCALAR_NEARLY_ZERO {
        return false;
    }

    true
}

/// Data structure to track reflex vertices and check whether any are inside a given triangle.
#[derive(Debug, Default)]
struct ReflexHash {
    bounds: Rect,
    h_count: i32,
    v_count: i32,
    num_verts: i32,
    // converts distance from the origin to a grid location (when cast to int)
    grid_conversion: Vector,
    grid: Vec<Vec<usize>>,
}

impl ReflexHash {
    fn init(&mut self, bounds: Rect, vertex_count: i32) -> bool {
        self.bounds = bounds;
        self.num_verts = 0;
        let width = bounds.width();
        let height = bounds.height();
        if !crate::floating_point::is_finite2(width, height) {
            return false;
        }

        // We want vertexCount grid cells, roughly distributed to match the bounds ratio
        let h_count = scalar_sqrt(ieee_float_divide((vertex_count as Scalar) * width, height));
        if !is_finite(h_count) {
            return false;
        }
        self.h_count = cpp_max(cpp_min(scalar_round_to_int(h_count), vertex_count), 1);
        self.v_count = vertex_count / self.h_count;
        self.grid_conversion = Vector::new(
            ieee_float_divide(self.h_count as Scalar - 0.001, width),
            ieee_float_divide(self.v_count as Scalar - 0.001, height),
        );
        if !self.grid_conversion.is_finite() {
            return false;
        }

        self.grid = vec![Vec::new(); (self.h_count * self.v_count) as usize];
        true
    }

    fn hash(&self, vert: &TriangulationVertex) -> usize {
        let h = ((vert.position.x - self.bounds.left) * self.grid_conversion.x) as i32;
        let v = ((vert.position.y - self.bounds.top) * self.grid_conversion.y) as i32;
        debug_assert!(v * self.h_count + h >= 0);
        (v * self.h_count + h) as usize
    }

    fn add(&mut self, nodes: &[TriangulationVertex], index: usize) {
        let cell = self.hash(&nodes[index]);
        self.grid[cell].push(index);
        self.num_verts += 1;
    }

    fn remove(&mut self, nodes: &[TriangulationVertex], index: usize) {
        let cell = self.hash(&nodes[index]);
        if let Some(pos) = self.grid[cell].iter().position(|&v| v == index) {
            self.grid[cell].remove(pos);
        }
        self.num_verts -= 1;
    }

    fn check_triangle(
        &self,
        nodes: &[TriangulationVertex],
        p0: Point,
        p1: Point,
        p2: Point,
        ignore_index0: u16,
        ignore_index1: u16,
    ) -> bool {
        if 0 == self.num_verts {
            return false;
        }

        let tri_bounds = compute_triangle_bounds(p0, p1, p2);
        let h0 = ((tri_bounds.left - self.bounds.left) * self.grid_conversion.x) as i32;
        let h1 = ((tri_bounds.right - self.bounds.left) * self.grid_conversion.x) as i32;
        let v0 = ((tri_bounds.top - self.bounds.top) * self.grid_conversion.y) as i32;
        let v1 = ((tri_bounds.bottom - self.bounds.top) * self.grid_conversion.y) as i32;

        for v in v0..=v1 {
            for h in h0..=h1 {
                let i = (v * self.h_count + h) as usize;
                let Some(cell) = self.grid.get(i) else {
                    continue;
                };
                for &reflex_index in cell {
                    let reflex_vertex = &nodes[reflex_index];
                    if reflex_vertex.index != ignore_index0
                        && reflex_vertex.index != ignore_index1
                        && point_in_triangle(p0, p1, p2, reflex_vertex.position)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[inline]
fn cpp_max(a: i32, b: i32) -> i32 {
    if a < b { b } else { a }
}

#[inline]
fn cpp_min(a: i32, b: i32) -> i32 {
    if b < a { b } else { a }
}

/// Check to see if a reflex vertex has become a convex vertex after clipping an ear.
fn reclassify_vertex(
    nodes: &mut [TriangulationVertex],
    p: usize,
    polygon_verts: &[Point],
    winding: i32,
    reflex_hash: &mut ReflexHash,
    convex_list: &mut ConvexList,
) {
    if VertexType::Reflex == nodes[p].vertex_type {
        let v0 = nodes[p].position - polygon_verts[nodes[p].prev_index as usize];
        let v1 = polygon_verts[nodes[p].next_index as usize] - nodes[p].position;
        if (winding as Scalar) * v0.cross(&v1) > SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO {
            nodes[p].vertex_type = VertexType::Convex;
            reflex_hash.remove(nodes, p);
            nodes[p].link_prev = None;
            nodes[p].link_next = None;
            convex_list.add_to_tail(nodes, p);
        }
    }
}

/// Compute indices to triangulate the given polygon.
pub fn sk_triangulate_simple_polygon(
    polygon_verts: &[Point],
    index_map: &[u16],
    triangle_indices: &mut Vec<u16>,
) -> bool {
    let polygon_size = polygon_verts.len();
    if polygon_size < 3 {
        return false;
    }
    // need to be able to represent all the vertices in the 16-bit indices
    if polygon_size >= u16::MAX as usize {
        return false;
    }

    // get bounds
    let mut bounds = Rect::default();
    if !bounds.set_bounds_check(polygon_verts) {
        return false;
    }
    // get winding direction
    let winding = sk_get_polygon_winding(polygon_verts);
    if 0 == winding {
        return false;
    }

    // Set up vertices
    let mut triangulation_vertices = vec![TriangulationVertex::default(); polygon_size];
    let mut prev_index = polygon_size - 1;
    let mut v0 = polygon_verts[0] - polygon_verts[prev_index];
    for curr_index in 0..polygon_size {
        let next_index = (curr_index + 1) % polygon_size;

        triangulation_vertices[curr_index] = TriangulationVertex::default();
        triangulation_vertices[curr_index].position = polygon_verts[curr_index];
        triangulation_vertices[curr_index].index = curr_index as u16;
        triangulation_vertices[curr_index].prev_index = prev_index as u16;
        triangulation_vertices[curr_index].next_index = next_index as u16;
        let v1 = polygon_verts[next_index] - polygon_verts[curr_index];
        if (winding as Scalar) * v0.cross(&v1) > SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO {
            triangulation_vertices[curr_index].vertex_type = VertexType::Convex;
        } else {
            triangulation_vertices[curr_index].vertex_type = VertexType::Reflex;
        }

        prev_index = curr_index;
        v0 = v1;
    }

    // Classify initial vertices into a list of convex vertices and a hash of reflex vertices
    let mut convex_list = ConvexList::default();
    let mut reflex_hash = ReflexHash::default();
    if !reflex_hash.init(bounds, polygon_size as i32) {
        return false;
    }
    let mut prev_index = polygon_size - 1;
    for curr_index in 0..polygon_size {
        let curr_type = triangulation_vertices[curr_index].vertex_type;
        if VertexType::Convex == curr_type {
            let next_index = (curr_index + 1) % polygon_size;
            let prev_type = triangulation_vertices[prev_index].vertex_type;
            let next_type = triangulation_vertices[next_index].vertex_type;
            // We prioritize clipping vertices with neighboring reflex vertices.
            if VertexType::Reflex == prev_type || VertexType::Reflex == next_type {
                convex_list.add_to_head(&mut triangulation_vertices, curr_index);
            } else {
                convex_list.add_to_tail(&mut triangulation_vertices, curr_index);
            }
        } else {
            // We treat near collinear vertices as reflex
            reflex_hash.add(&triangulation_vertices, curr_index);
        }
        prev_index = curr_index;
    }

    // The general concept: We are trying to find three neighboring vertices where
    // no other vertex lies inside the triangle (an "ear"). If we find one, we clip
    // that ear off, and then repeat on the new polygon. Once we get down to three vertices
    // we have triangulated the entire polygon.
    triangle_indices.reserve(3 * (polygon_size - 2));
    let mut vertex_count = polygon_size;
    while vertex_count > 3 {
        let mut success = false;
        let mut ear_vertex = 0usize;
        let mut p0 = 0usize;
        let mut p2 = 0usize;
        // find a convex vertex to clip
        let mut convex_iter = convex_list.head;
        while let Some(vertex) = convex_iter {
            ear_vertex = vertex;
            debug_assert!(VertexType::Reflex != triangulation_vertices[ear_vertex].vertex_type);

            p0 = triangulation_vertices[ear_vertex].prev_index as usize;
            p2 = triangulation_vertices[ear_vertex].next_index as usize;

            // see if any reflex vertices are inside the ear
            let failed = reflex_hash.check_triangle(
                &triangulation_vertices,
                triangulation_vertices[p0].position,
                triangulation_vertices[ear_vertex].position,
                triangulation_vertices[p2].position,
                triangulation_vertices[p0].index,
                triangulation_vertices[p2].index,
            );
            if failed {
                convex_iter = triangulation_vertices[vertex].link_next;
                continue;
            }

            // found one we can clip
            success = true;
            break;
        }
        // If we can't find any ears to clip, this probably isn't a simple polygon
        if !success {
            return false;
        }

        // add indices
        triangle_indices.push(index_map[triangulation_vertices[p0].index as usize]);
        triangle_indices.push(index_map[triangulation_vertices[ear_vertex].index as usize]);
        triangle_indices.push(index_map[triangulation_vertices[p2].index as usize]);

        // clip the ear
        convex_list.remove(&mut triangulation_vertices, ear_vertex);
        vertex_count -= 1;

        // reclassify reflex verts
        triangulation_vertices[p0].next_index = triangulation_vertices[ear_vertex].next_index;
        reclassify_vertex(
            &mut triangulation_vertices,
            p0,
            polygon_verts,
            winding,
            &mut reflex_hash,
            &mut convex_list,
        );

        triangulation_vertices[p2].prev_index = triangulation_vertices[ear_vertex].prev_index;
        reclassify_vertex(
            &mut triangulation_vertices,
            p2,
            polygon_verts,
            winding,
            &mut reflex_hash,
            &mut convex_list,
        );
    }

    // output indices
    let mut vertex_iter = convex_list.head;
    while let Some(vertex) = vertex_iter {
        triangle_indices.push(index_map[triangulation_vertices[vertex].index as usize]);
        vertex_iter = triangulation_vertices[vertex].link_next;
    }

    true
}
