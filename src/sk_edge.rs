//! Port of `src/core/SkEdge.h` and `src/core/SkEdge.cpp`.
//!
//! `SkEdge` approximates monotonic curves by one or more Y-monotonic line segments, using FDot6
//! coordinates and forward differencing. The C++ class hierarchy is modelled as an enum here so
//! that the fixed-point scan converter can keep its edges in a plain `Vec`; the fields, per-segment
//! update order and the virtual `nextSegment()` behavior are preserved.

use crate::fixed::{
    SK_FDOT6_HALF, SkFDot6, SkFixed, sk_fdot6_div, sk_fdot6_round, sk_fixed_mul, sk_fixed_to_fdot6,
    sk_float_to_fdot6,
};
use crate::math_priv::sk_clz;
use crate::point::Point;
use crate::rect::IRect;
use crate::safe32::sk_abs32;

/// `SkEdge::Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkEdgeType {
    Line,
    Quad,
    Cubic,
}

/// `SkEdge::Winding`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum SkEdgeWinding {
    Cw = 1,
    Ccw = -1,
}

/// `SkEdge_Compute_DY`.
#[inline]
fn sk_edge_compute_dy(top: i32, y0: i32) -> SkFDot6 {
    ((top as u32) << 6) as i32 + SK_FDOT6_HALF - y0
}

#[inline]
fn sk_fdot6_to_fixed(value: SkFDot6) -> SkFixed {
    ((value as u32) << 10) as i32
}

#[inline]
fn sk_fdot6_to_fixed_div2(value: SkFDot6) -> SkFixed {
    ((value as u32) << (16 - 6 - 1)) as i32
}

const MAX_COEFF_SHIFT: i32 = 6;

/// The fields shared by every edge type.
#[derive(Clone, Debug)]
pub struct SkEdgeBase {
    /// Current line segment starts at `(f_x, f_first_y + 0.5)`, slope `f_dxdy`.
    pub f_x: SkFixed,
    pub f_dxdy: SkFixed,
    pub f_first_y: i32,
    pub f_last_y: i32,
    pub f_edge_type: SkEdgeType,
    pub f_winding: SkEdgeWinding,
    /// Only non-zero for quads and cubics.
    pub f_segment_count: u8,
    /// How much to shift the derivatives when doing forward differencing.
    pub f_curve_shift: u8,
}

impl SkEdgeBase {
    fn new() -> SkEdgeBase {
        SkEdgeBase {
            f_x: 0,
            f_dxdy: 0,
            f_first_y: 0,
            f_last_y: 0,
            f_edge_type: SkEdgeType::Line,
            f_winding: SkEdgeWinding::Cw,
            f_segment_count: 0,
            f_curve_shift: 0,
        }
    }

    #[inline]
    pub fn has_next_segment(&self) -> bool {
        self.f_segment_count != 0
    }

    #[inline]
    pub fn segments_left(&self) -> u8 {
        self.f_segment_count
    }

    /// `SkEdge::setLine`.
    pub fn set_line(&mut self, p0: Point, p1: Point, clip: Option<&IRect>) -> bool {
        let mut x0 = sk_float_to_fdot6(p0.x);
        let mut y0 = sk_float_to_fdot6(p0.y);
        let mut x1 = sk_float_to_fdot6(p1.x);
        let mut y1 = sk_float_to_fdot6(p1.y);

        let mut winding = SkEdgeWinding::Cw;
        if y0 > y1 {
            core::mem::swap(&mut x0, &mut x1);
            core::mem::swap(&mut y0, &mut y1);
            winding = SkEdgeWinding::Ccw;
        }

        let top = sk_fdot6_round(y0);
        let bot = sk_fdot6_round(y1);

        // are we a zero-height line?
        if top == bot {
            return false;
        }
        // are we completely above or below the clip?
        if let Some(clip) = clip {
            if top >= clip.bottom || bot <= clip.top {
                return false;
            }
        }

        let slope = sk_fdot6_div(x1 - x0, y1 - y0);
        let dy = sk_edge_compute_dy(top, y0);

        // Note that SkFixedMul(SkFixed, SkFDot6) produces results in SkFDot6.
        self.f_x = sk_fdot6_to_fixed(x0.wrapping_add(sk_fixed_mul(slope, dy)));
        self.f_dxdy = slope;
        self.f_first_y = top;
        self.f_last_y = bot - 1;
        self.f_edge_type = SkEdgeType::Line;
        self.f_segment_count = 0;
        self.f_winding = winding;
        self.f_curve_shift = 0;

        if let Some(clip) = clip {
            self.chop_line_with_clip(clip);
        }
        true
    }

    fn update_line(
        &mut self,
        x_start: SkFixed,
        y_start: SkFixed,
        x_end: SkFixed,
        y_end: SkFixed,
    ) -> bool {
        debug_assert!(self.f_segment_count != 0);

        let y0 = sk_fixed_to_fdot6(y_start);
        let y1 = sk_fixed_to_fdot6(y_end);
        debug_assert!(y0 <= y1);

        let top = sk_fdot6_round(y0);
        let bot = sk_fdot6_round(y1);

        // are we a zero-height line?
        if top == bot {
            return false;
        }

        let x0 = sk_fixed_to_fdot6(x_start);
        let x1 = sk_fixed_to_fdot6(x_end);

        let slope = sk_fdot6_div(x1 - x0, y1 - y0);
        let dy = sk_edge_compute_dy(top, y0);

        self.f_x = sk_fdot6_to_fixed(x0.wrapping_add(sk_fixed_mul(slope, dy)));
        self.f_dxdy = slope;
        self.f_first_y = top;
        self.f_last_y = bot - 1;
        true
    }

    fn chop_line_with_clip(&mut self, clip: &IRect) {
        let top = self.f_first_y;
        debug_assert!(top < clip.bottom);
        if top < clip.top {
            debug_assert!(self.f_last_y >= clip.top);
            self.f_x = self
                .f_x
                .wrapping_add(self.f_dxdy.wrapping_mul(clip.top - top));
            self.f_first_y = clip.top;
        }
    }
}

/// Approximate the distance from `(0,0)` to `(dx, dy)`.
fn cheap_distance(dx: SkFDot6, dy: SkFDot6) -> SkFDot6 {
    let dx = sk_abs32(dx);
    let dy = sk_abs32(dy);
    if dx > dy {
        dx + (dy / 2)
    } else {
        dy + (dx / 2)
    }
}

fn diff_to_steps(dx: SkFDot6, dy: SkFDot6, accuracy: i32) -> i32 {
    let mut dist = cheap_distance(dx, dy);
    dist = (dist + (1 << (2 + accuracy))) >> (3 + accuracy);
    (32 - sk_clz(dist as u32)) >> 1
}

/// `SkQuadraticEdge`.
#[derive(Clone, Debug)]
pub struct SkQuadraticEdge {
    pub base: SkEdgeBase,
    f_qx: SkFixed,
    f_qy: SkFixed,
    f_qdxdt: SkFixed,
    f_qdydt: SkFixed,
    f_qd2xdt2: SkFixed,
    f_qd2ydt2: SkFixed,
    f_qlastx: SkFixed,
    f_qlasty: SkFixed,
}

impl SkQuadraticEdge {
    fn new() -> SkQuadraticEdge {
        SkQuadraticEdge {
            base: SkEdgeBase::new(),
            f_qx: 0,
            f_qy: 0,
            f_qdxdt: 0,
            f_qdydt: 0,
            f_qd2xdt2: 0,
            f_qd2ydt2: 0,
            f_qlastx: 0,
            f_qlasty: 0,
        }
    }

    /// `SkQuadraticEdge::setQuadratic`.
    pub fn set_quadratic(&mut self, pts: &[Point]) -> bool {
        let mut x0 = sk_float_to_fdot6(pts[0].x);
        let mut y0 = sk_float_to_fdot6(pts[0].y);
        let x1 = sk_float_to_fdot6(pts[1].x);
        let y1 = sk_float_to_fdot6(pts[1].y);
        let mut x2 = sk_float_to_fdot6(pts[2].x);
        let mut y2 = sk_float_to_fdot6(pts[2].y);

        let mut winding = SkEdgeWinding::Cw;
        if y0 > y2 {
            core::mem::swap(&mut x0, &mut x2);
            core::mem::swap(&mut y0, &mut y2);
            winding = SkEdgeWinding::Ccw;
        }
        debug_assert!(y0 <= y1 && y1 <= y2, "curve must be monotonic");

        let top = sk_fdot6_round(y0);
        let bot = sk_fdot6_round(y2);

        // are we a zero-height quad (line)?
        if top == bot {
            return false;
        }

        let delta_x = (2 * x1 - x0 - x2) >> 2;
        let delta_y = (2 * y1 - y0 - y2) >> 2;
        let mut shift = diff_to_steps(delta_x, delta_y, 0);
        debug_assert!(shift >= 0);

        // We need at least 2 line segments to save the derivatives as half their values.
        if shift == 0 {
            shift = 1;
        } else if shift > MAX_COEFF_SHIFT {
            shift = MAX_COEFF_SHIFT;
        }

        self.base.f_winding = winding;
        self.base.f_edge_type = SkEdgeType::Quad;
        self.base.f_segment_count = (1u32 << shift) as u8;
        self.base.f_curve_shift = (shift - 1) as u8;

        // A_half and B_half carry an extra 1/2 factor to avoid overflow.
        let mut a_half = sk_fdot6_to_fixed_div2(x0 - x1 - x1 + x2);
        let mut b_half = sk_fdot6_to_fixed(x1 - x0);

        self.f_qdxdt = b_half.wrapping_add(a_half >> shift);
        self.f_qd2xdt2 = a_half >> (shift - 1);

        a_half = sk_fdot6_to_fixed_div2(y0 - y1 - y1 + y2);
        b_half = sk_fdot6_to_fixed(y1 - y0);

        self.f_qdydt = b_half.wrapping_add(a_half >> shift);
        self.f_qd2ydt2 = a_half >> (shift - 1);

        self.f_qx = sk_fdot6_to_fixed(x0);
        self.f_qy = sk_fdot6_to_fixed(y0);
        self.f_qlastx = sk_fdot6_to_fixed(x2);
        self.f_qlasty = sk_fdot6_to_fixed(y2);

        self.next_segment()
    }

    /// `SkQuadraticEdge::nextSegment`.
    pub fn next_segment(&mut self) -> bool {
        let mut count = self.base.f_segment_count as i32;
        let mut oldx = self.f_qx;
        let mut oldy = self.f_qy;
        let mut dx = self.f_qdxdt;
        let mut dy = self.f_qdydt;
        let shift = self.base.f_curve_shift as i32;
        let mut success;

        debug_assert!(count > 0);

        loop {
            count -= 1;
            if count > 0 {
                let nx = oldx.wrapping_add(dx >> shift);
                dx = dx.wrapping_add(self.f_qd2xdt2);
                let ny = oldy.wrapping_add(dy >> shift);
                dy = dy.wrapping_add(self.f_qd2ydt2);
                success = self.base.update_line(oldx, oldy, nx, ny);
                oldx = nx;
                oldy = ny;
            } else {
                let nx = self.f_qlastx;
                let ny = self.f_qlasty;
                success = self.base.update_line(oldx, oldy, nx, ny);
                oldx = nx;
                oldy = ny;
            }
            if count <= 0 || success {
                break;
            }
        }

        self.f_qx = oldx;
        self.f_qy = oldy;
        self.f_qdxdt = dx;
        self.f_qdydt = dy;
        self.base.f_segment_count = count as u8;
        success
    }
}

fn sk_fdot6_up_shift(x: SkFDot6, up_shift: i32) -> SkFDot6 {
    debug_assert!(((x as u32) << up_shift) as i32 >> up_shift == x);
    ((x as u32) << up_shift) as i32
}

fn cubic_delta_from_line(a: SkFDot6, b: SkFDot6, c: SkFDot6, d: SkFDot6) -> SkFDot6 {
    let one_third = (a.wrapping_mul(8) - b.wrapping_mul(15) + 6 * c + d).wrapping_mul(19) >> 9;
    let two_third = (a + 6 * b - c.wrapping_mul(15) + d.wrapping_mul(8)).wrapping_mul(19) >> 9;
    sk_abs32(one_third).max(sk_abs32(two_third))
}

/// `SkCubicEdge`.
#[derive(Clone, Debug)]
pub struct SkCubicEdge {
    pub base: SkEdgeBase,
    f_cx: SkFixed,
    f_cy: SkFixed,
    f_cdxdt: SkFixed,
    f_cdydt: SkFixed,
    f_cd2xdt2: SkFixed,
    f_cd2ydt2: SkFixed,
    f_cd3xdt3: SkFixed,
    f_cd3ydt3: SkFixed,
    f_clastx: SkFixed,
    f_clasty: SkFixed,
    f_to_fixed_shift: u8,
}

impl SkCubicEdge {
    fn new() -> SkCubicEdge {
        SkCubicEdge {
            base: SkEdgeBase::new(),
            f_cx: 0,
            f_cy: 0,
            f_cdxdt: 0,
            f_cdydt: 0,
            f_cd2xdt2: 0,
            f_cd2ydt2: 0,
            f_cd3xdt3: 0,
            f_cd3ydt3: 0,
            f_clastx: 0,
            f_clasty: 0,
            f_to_fixed_shift: 0,
        }
    }

    /// `SkCubicEdge::setCubic`.
    pub fn set_cubic(&mut self, pts: &[Point]) -> bool {
        let mut x0 = sk_float_to_fdot6(pts[0].x);
        let mut y0 = sk_float_to_fdot6(pts[0].y);
        let mut x1 = sk_float_to_fdot6(pts[1].x);
        let mut y1 = sk_float_to_fdot6(pts[1].y);
        let mut x2 = sk_float_to_fdot6(pts[2].x);
        let mut y2 = sk_float_to_fdot6(pts[2].y);
        let mut x3 = sk_float_to_fdot6(pts[3].x);
        let mut y3 = sk_float_to_fdot6(pts[3].y);

        let mut winding = SkEdgeWinding::Cw;
        if y0 > y3 {
            core::mem::swap(&mut x0, &mut x3);
            core::mem::swap(&mut x1, &mut x2);
            core::mem::swap(&mut y0, &mut y3);
            core::mem::swap(&mut y1, &mut y2);
            winding = SkEdgeWinding::Ccw;
        }

        let top = sk_fdot6_round(y0);
        let bot = sk_fdot6_round(y3);

        // are we a zero-height cubic (line)?
        if top == bot {
            return false;
        }

        // compute number of steps needed (1 << stepExponent)
        let dx = cubic_delta_from_line(x0, x1, x2, x3);
        let dy = cubic_delta_from_line(y0, y1, y2, y3);
        let mut step_exponent = diff_to_steps(dx, dy, 2) + 1;
        debug_assert!(step_exponent > 0);
        if step_exponent > MAX_COEFF_SHIFT {
            step_exponent = MAX_COEFF_SHIFT;
        }

        let mut precision_up_scale = 6; // largest safe value
        let mut to_fixed_shift = step_exponent + precision_up_scale - 10;
        if to_fixed_shift < 0 {
            to_fixed_shift = 0;
            precision_up_scale = 10 - step_exponent;
        }

        self.base.f_winding = winding;
        self.base.f_edge_type = SkEdgeType::Cubic;
        self.base.f_segment_count = ((1u32) << step_exponent) as u8;
        self.base.f_curve_shift = step_exponent as u8;
        self.f_to_fixed_shift = to_fixed_shift as u8;

        let mut a_scaled = sk_fdot6_up_shift(x3 + 3 * (x1 - x2) - x0, precision_up_scale);
        let mut b_scaled = sk_fdot6_up_shift(3 * (x0 - 2 * x1 + x2), precision_up_scale);
        let mut c_scaled = sk_fdot6_up_shift(3 * (x1 - x0), precision_up_scale);

        self.f_cdxdt = (a_scaled >> (2 * step_exponent))
            .wrapping_add(b_scaled >> step_exponent)
            .wrapping_add(c_scaled);
        self.f_cd2xdt2 = ((3 * a_scaled) >> (step_exponent - 1)).wrapping_add(2 * b_scaled);
        self.f_cd3xdt3 = (3 * a_scaled) >> (step_exponent - 1);

        a_scaled = sk_fdot6_up_shift(y3 + 3 * (y1 - y2) - y0, precision_up_scale);
        b_scaled = sk_fdot6_up_shift(3 * (y0 - 2 * y1 + y2), precision_up_scale);
        c_scaled = sk_fdot6_up_shift(3 * (y1 - y0), precision_up_scale);

        self.f_cdydt = (a_scaled >> (2 * step_exponent))
            .wrapping_add(b_scaled >> step_exponent)
            .wrapping_add(c_scaled);
        self.f_cd2ydt2 = ((3 * a_scaled) >> (step_exponent - 1)).wrapping_add(2 * b_scaled);
        self.f_cd3ydt3 = (3 * a_scaled) >> (step_exponent - 1);

        self.f_cx = sk_fdot6_to_fixed(x0);
        self.f_cy = sk_fdot6_to_fixed(y0);
        self.f_clastx = sk_fdot6_to_fixed(x3);
        self.f_clasty = sk_fdot6_to_fixed(y3);

        self.next_segment()
    }

    /// `SkCubicEdge::nextSegment`.
    pub fn next_segment(&mut self) -> bool {
        let mut count = self.base.f_segment_count as i32;
        let mut oldx = self.f_cx;
        let mut oldy = self.f_cy;
        let step_exponent = self.base.f_curve_shift as i32;
        let to_fixed_shift = self.f_to_fixed_shift as i32;
        let mut success;

        debug_assert!(count > 0);

        loop {
            count -= 1;
            let newx;
            let mut newy;
            if count > 0 {
                newx = oldx.wrapping_add(self.f_cdxdt >> to_fixed_shift);
                self.f_cdxdt = self.f_cdxdt.wrapping_add(self.f_cd2xdt2 >> step_exponent);
                self.f_cd2xdt2 = self.f_cd2xdt2.wrapping_add(self.f_cd3xdt3);

                newy = oldy.wrapping_add(self.f_cdydt >> to_fixed_shift);
                self.f_cdydt = self.f_cdydt.wrapping_add(self.f_cd2ydt2 >> step_exponent);
                self.f_cd2ydt2 = self.f_cd2ydt2.wrapping_add(self.f_cd3ydt3);
            } else {
                newx = self.f_clastx;
                newy = self.f_clasty;
            }

            // Our finite fixedpoint doesn't always achieve oldy <= newy, so pin it here.
            if newy < oldy {
                newy = oldy;
            }

            success = self.base.update_line(oldx, oldy, newx, newy);
            oldx = newx;
            oldy = newy;

            if count <= 0 || success {
                break;
            }
        }

        self.f_cx = oldx;
        self.f_cy = oldy;
        self.base.f_segment_count = count as u8;
        success
    }
}

/// `SkEdge`, as one of the three concrete edge types.
#[derive(Clone, Debug)]
pub enum SkEdge {
    Line(SkEdgeBase),
    Quad(SkQuadraticEdge),
    Cubic(SkCubicEdge),
}

impl SkEdge {
    /// `fAlloc.make<SkEdge>()` followed by `setLine`.
    pub fn new_line(p0: Point, p1: Point, clip: Option<&IRect>) -> Option<SkEdge> {
        let mut base = SkEdgeBase::new();
        if base.set_line(p0, p1, clip) {
            Some(SkEdge::Line(base))
        } else {
            None
        }
    }

    /// `fAlloc.make<SkQuadraticEdge>()` followed by `setQuadratic`.
    pub fn new_quad(pts: &[Point]) -> Option<SkEdge> {
        let mut edge = SkQuadraticEdge::new();
        if edge.set_quadratic(pts) {
            Some(SkEdge::Quad(edge))
        } else {
            None
        }
    }

    /// `fAlloc.make<SkCubicEdge>()` followed by `setCubic`.
    pub fn new_cubic(pts: &[Point]) -> Option<SkEdge> {
        let mut edge = SkCubicEdge::new();
        if edge.set_cubic(pts) {
            Some(SkEdge::Cubic(edge))
        } else {
            None
        }
    }

    #[inline]
    pub fn base(&self) -> &SkEdgeBase {
        match self {
            SkEdge::Line(base) => base,
            SkEdge::Quad(edge) => &edge.base,
            SkEdge::Cubic(edge) => &edge.base,
        }
    }

    #[inline]
    pub fn base_mut(&mut self) -> &mut SkEdgeBase {
        match self {
            SkEdge::Line(base) => base,
            SkEdge::Quad(edge) => &mut edge.base,
            SkEdge::Cubic(edge) => &mut edge.base,
        }
    }

    #[inline]
    pub fn f_x(&self) -> SkFixed {
        self.base().f_x
    }

    #[inline]
    pub fn f_dxdy(&self) -> SkFixed {
        self.base().f_dxdy
    }

    #[inline]
    pub fn f_first_y(&self) -> i32 {
        self.base().f_first_y
    }

    #[inline]
    pub fn f_last_y(&self) -> i32 {
        self.base().f_last_y
    }

    #[inline]
    pub fn f_winding(&self) -> SkEdgeWinding {
        self.base().f_winding
    }

    #[inline]
    pub fn has_next_segment(&self) -> bool {
        self.base().has_next_segment()
    }

    /// `SkEdge::nextSegment`.
    pub fn next_segment(&mut self) -> bool {
        match self {
            SkEdge::Line(_) => {
                debug_assert!(
                    false,
                    "Shouldn't be asking a linear edge to go to the next curve."
                );
                false
            }
            SkEdge::Quad(edge) => edge.next_segment(),
            SkEdge::Cubic(edge) => edge.next_segment(),
        }
    }
}
