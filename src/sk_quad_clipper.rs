//! Port of `src/core/SkQuadClipper.h` and `src/core/SkQuadClipper.cpp`.
//!
//! `SkQuadClipper2` is declared in the reference header but has no definition in any translation
//! unit, so it has no observable behavior to port. See `plan/handoffs/M06.md`.

use crate::point::Point;
use crate::rect::{IRect, Rect};
use crate::scalar::Scalar;
use crate::sk_geometry::{sk_chop_quad_at, sk_find_unit_quad_roots};

/// This class is initialized with a clip rectangle, and then can be fed quads,
/// which must already be monotonic in Y.
#[derive(Clone, Copy, Debug, Default)]
pub struct SkQuadClipper {
    clip: Rect,
}

impl SkQuadClipper {
    pub fn new() -> Self {
        let mut clipper = Self::default();
        clipper.clip.set_empty();
        clipper
    }

    pub fn set_clip(&mut self, clip: &IRect) {
        // convert to scalars, since that's where we'll see the points
        self.clip.set_irect(*clip);
    }

    pub fn clip(&self) -> &Rect {
        &self.clip
    }

    pub fn clip_quad(&self, src_pts: &[Point; 3], dst: &mut [Point; 3]) -> bool {
        let reverse;

        // we need the data to be monotonically increasing in Y
        if src_pts[0].y > src_pts[2].y {
            dst[0] = src_pts[2];
            dst[1] = src_pts[1];
            dst[2] = src_pts[0];
            reverse = true;
        } else {
            *dst = *src_pts;
            reverse = false;
        }

        // are we completely above or below
        let ctop = self.clip.top;
        let cbot = self.clip.bottom;
        if dst[2].y <= ctop || dst[0].y >= cbot {
            return false;
        }

        let mut tmp = [Point::default(); 5];

        // are we partially above
        if dst[0].y < ctop {
            if let Some(t) = chop_mono_quad_at_y(dst, ctop) {
                // take the 2nd chopped quad
                let src = *dst;
                sk_chop_quad_at(&src, &mut tmp, t);
                dst[0] = tmp[2];
                dst[1] = tmp[3];
            } else {
                // if chopMonoQuadAtY failed, then we may have hit inexact numerics
                // so we just clamp against the top
                for point in dst.iter_mut() {
                    if point.y < ctop {
                        point.y = ctop;
                    }
                }
            }
        }

        // are we partially below
        if dst[2].y > cbot {
            if let Some(t) = chop_mono_quad_at_y(dst, cbot) {
                let src = *dst;
                sk_chop_quad_at(&src, &mut tmp, t);
                dst[1] = tmp[1];
                dst[2] = tmp[2];
            } else {
                // if chopMonoQuadAtY failed, then we may have hit inexact numerics
                // so we just clamp against the bottom
                for point in dst.iter_mut() {
                    if point.y > cbot {
                        point.y = cbot;
                    }
                }
            }
        }

        if reverse {
            dst.swap(0, 2);
        }
        true
    }
}

fn chop_mono_quad_at(c0: Scalar, c1: Scalar, c2: Scalar, target: Scalar, t: &mut Scalar) -> bool {
    // Solve F(t) = y where F(t) := [0](1-t)^2 + 2[1]t(1-t) + [2]t^2
    // We solve for t, using quadratic equation, hence we have to rearrange
    // our coefficients to look like At^2 + Bt + C.
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
