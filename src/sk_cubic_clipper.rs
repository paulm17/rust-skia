//! Port of `src/core/SkCubicClipper.h` and `src/core/SkCubicClipper.cpp`.
//!
//! The reference selects the bisection fallback because `NEWTON_RAPHSON` is not defined; the
//! Newton-Raphson branch is dead code upstream and is not ported.

use crate::point::Point;
use crate::rect::{IRect, Rect};
use crate::scalar::Scalar;
use crate::sk_geometry::sk_chop_cubic_at;

/// This class is initialized with a clip rectangle, and then can be fed cubics,
/// which must already be monotonic in Y.
#[derive(Clone, Copy, Debug, Default)]
pub struct SkCubicClipper {
    clip: Rect,
}

impl SkCubicClipper {
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

    /// Returns the parameter of the monotonic cubic's Y intersection, if one exists in [0, 1].
    pub fn chop_mono_at_y(pts: &[Point; 4], y: Scalar) -> Option<Scalar> {
        let ycrv = [pts[0].y - y, pts[1].y - y, pts[2].y - y, pts[3].y - y];

        // Check that the endpoints straddle zero.
        let (mut t_neg, mut t_pos); // Negative and positive function parameters.
        if ycrv[0] < 0.0 {
            if ycrv[3] < 0.0 {
                return None;
            }
            t_neg = 0.0;
            t_pos = 1.0;
        } else if ycrv[0] > 0.0 {
            if ycrv[3] > 0.0 {
                return None;
            }
            t_neg = 1.0;
            t_pos = 0.0;
        } else {
            return Some(0.0);
        }

        let tol = 1.0 / 65536.0; // 1 for fixed, 1e-5 for float.
        loop {
            let t_mid = (t_pos + t_neg) / 2.0;
            let y01 = ycrv[0] + (ycrv[1] - ycrv[0]) * t_mid;
            let y12 = ycrv[1] + (ycrv[2] - ycrv[1]) * t_mid;
            let y23 = ycrv[2] + (ycrv[3] - ycrv[2]) * t_mid;
            let y012 = y01 + (y12 - y01) * t_mid;
            let y123 = y12 + (y23 - y12) * t_mid;
            let y0123 = y012 + (y123 - y012) * t_mid;
            if y0123 == 0.0 {
                return Some(t_mid);
            }
            if y0123 < 0.0 {
                t_neg = t_mid;
            } else {
                t_pos = t_mid;
            }
            if (t_pos - t_neg).abs() <= tol {
                break;
            }
        }

        Some((t_neg + t_pos) / 2.0)
    }

    pub fn clip_cubic(&self, src_pts: &[Point; 4], dst: &mut [Point; 4]) -> bool {
        let reverse;

        // we need the data to be monotonically descending in Y
        if src_pts[0].y > src_pts[3].y {
            dst[0] = src_pts[3];
            dst[1] = src_pts[2];
            dst[2] = src_pts[1];
            dst[3] = src_pts[0];
            reverse = true;
        } else {
            *dst = *src_pts;
            reverse = false;
        }

        // are we completely above or below
        let ctop = self.clip.top;
        let cbot = self.clip.bottom;
        if dst[3].y <= ctop || dst[0].y >= cbot {
            return false;
        }

        let mut tmp = [Point::default(); 7]; // for SkChopCubicAt

        // are we partially above
        if dst[0].y < ctop {
            if let Some(t) = Self::chop_mono_at_y(dst, ctop) {
                let src = *dst;
                sk_chop_cubic_at(&src, &mut tmp, t);
                dst[0] = tmp[3];
                dst[1] = tmp[4];
                dst[2] = tmp[5];
            }
        }

        // are we partially below
        if dst[3].y > cbot {
            if let Some(t) = Self::chop_mono_at_y(dst, cbot) {
                let src = *dst;
                sk_chop_cubic_at(&src, &mut tmp, t);
                dst[1] = tmp[1];
                dst[2] = tmp[2];
                dst[3] = tmp[3];
            }
        }

        if reverse {
            dst.swap(0, 3);
            dst.swap(1, 2);
        }
        true
    }
}
