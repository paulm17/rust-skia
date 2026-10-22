//! Port of `include/effects/SkCornerPathEffect.h` and `src/effects/SkCornerPathEffect.cpp`.

use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::rect::Rect;
use crate::scalar::{SCALAR_HALF, Scalar};
use crate::sk_path::{Iter, SkPath};
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::{SkPathEffect, SkPathEffectBase};
use crate::sk_path_types::SkPathVerb;
use crate::sk_stroke_rec::SkStrokeRec;

/// `ComputeStep`.
fn compute_step(a: &Point, b: &Point, radius: Scalar, step: &mut Vector) -> bool {
    let dist = Point::distance(a, b);

    *step = *b - *a;
    if dist <= radius * 2.0 {
        *step *= SCALAR_HALF;
        false
    } else {
        *step *= radius / dist;
        true
    }
}

/// `SkCornerPathEffectImpl`.
struct SkCornerPathEffectImpl {
    f_radius: Scalar,
}

impl SkPathEffectBase for SkCornerPathEffectImpl {
    fn on_filter_path(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        _rec: &mut SkStrokeRec,
        _cull_rect: Option<&Rect>,
        _ctm: &Matrix,
    ) -> bool {
        if self.f_radius <= 0.0 {
            return false;
        }

        // just need a value that won't match when we compare it initially
        let mut prev_verb: Option<SkPathVerb> = None;

        let mut closed;
        let mut move_to = Point::new(0.0, 0.0);
        let mut last_corner = Point::new(0.0, 0.0);
        let mut first_step = Vector::new(0.0, 0.0);
        let mut step = Vector::new(0.0, 0.0);
        let mut prev_is_valid = true;

        let mut iter = Iter::from_path(src, false);
        while let Some(rec) = iter.next_rec() {
            let mut pts = [Point::new(0.0, 0.0); 4];
            let n = rec.f_points.len();
            pts[..n].copy_from_slice(rec.f_points);
            let verb = rec.f_verb;
            let conic_weight = rec.f_conic_weight;
            match verb {
                SkPathVerb::Move => {
                    // close out the previous (open) contour
                    if Some(SkPathVerb::Line) == prev_verb {
                        dst.line_to(last_corner);
                    }
                    closed = iter.is_closed_contour();
                    if closed {
                        move_to = pts[0];
                        prev_is_valid = false;
                    } else {
                        dst.move_to(pts[0]);
                        prev_is_valid = true;
                    }
                }
                SkPathVerb::Line => {
                    let draw_segment = compute_step(&pts[0], &pts[1], self.f_radius, &mut step);
                    // prev corner
                    if !prev_is_valid {
                        dst.move_to(move_to + step);
                    } else {
                        dst.quad_to_xy(pts[0].x, pts[0].y, pts[0].x + step.x, pts[0].y + step.y);
                    }
                    if draw_segment {
                        dst.line_to_xy(pts[1].x - step.x, pts[1].y - step.y);
                    }
                    last_corner = pts[1];
                    prev_is_valid = true;
                }
                SkPathVerb::Quad => {
                    // TBD - just replicate the curve for now
                    if !prev_is_valid {
                        dst.move_to(pts[0]);
                        prev_is_valid = true;
                    }
                    dst.quad_to(pts[1], pts[2]);
                    last_corner = pts[2];
                    first_step.set(0.0, 0.0);
                }
                SkPathVerb::Conic => {
                    // TBD - just replicate the curve for now
                    if !prev_is_valid {
                        dst.move_to(pts[0]);
                        prev_is_valid = true;
                    }
                    dst.conic_to(pts[1], pts[2], conic_weight);
                    last_corner = pts[2];
                    first_step.set(0.0, 0.0);
                }
                SkPathVerb::Cubic => {
                    if !prev_is_valid {
                        dst.move_to(pts[0]);
                        prev_is_valid = true;
                    }
                    // TBD - just replicate the curve for now
                    dst.cubic_to(pts[1], pts[2], pts[3]);
                    last_corner = pts[3];
                    first_step.set(0.0, 0.0);
                }
                SkPathVerb::Close => {
                    if first_step.x != 0.0 || first_step.y != 0.0 {
                        dst.quad_to_xy(
                            last_corner.x,
                            last_corner.y,
                            last_corner.x + first_step.x,
                            last_corner.y + first_step.y,
                        );
                    }
                    dst.close();
                    prev_is_valid = false;
                }
                SkPathVerb::Done => {}
            }
            if Some(SkPathVerb::Move) == prev_verb {
                first_step = step;
            }
            prev_verb = Some(verb);
        }
        if prev_is_valid {
            dst.line_to(last_corner);
        }
        true
    }

    fn compute_fast_bounds(&self, _bounds: Option<&mut Rect>) -> bool {
        // Rounding sharp corners within a path produces a new path that is still contained within
        // the original's bounds, so leave 'bounds' unmodified.
        true
    }
}

/// `SkCornerPathEffect::Make`.
pub fn make_corner_path_effect(radius: Scalar) -> Option<SkPathEffect> {
    if radius.is_finite() && radius > 0.0 {
        Some(SkPathEffect::from_base(SkCornerPathEffectImpl {
            f_radius: radius,
        }))
    } else {
        None
    }
}

/// `SkCornerPathEffect`.
pub struct SkCornerPathEffect;

impl SkCornerPathEffect {
    /// `SkCornerPathEffect::Make`. `radius` must be > 0 to have an effect; it specifies the
    /// distance from each corner that should be "rounded".
    pub fn make(radius: Scalar) -> Option<SkPathEffect> {
        make_corner_path_effect(radius)
    }
}
