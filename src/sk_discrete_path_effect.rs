//! Port of `include/effects/SkDiscretePathEffect.h` and `src/effects/SkDiscretePathEffect.cpp`.

use crate::fixed::sk_fixed_to_scalar;
use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::point_priv::PointPriv;
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_round_to_int};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::{SkPathEffect, SkPathEffectBase};
use crate::sk_path_measure::SkPathMeasure;
use crate::sk_stroke_rec::SkStrokeRec;

/// `LCGRandom` (copied from the original implementation of `SkRandom`; only the methods used by
/// `SkDiscretePathEffect::filterPath` are kept, with methods not called directly removed).
struct LcgRandom {
    f_seed: u32,
}

impl LcgRandom {
    // See "Numerical Recipes in C", 1992 page 284 for these constants
    const K_MUL: u32 = 1664525;
    const K_ADD: u32 = 1013904223;

    fn new(seed: u32) -> Self {
        LcgRandom { f_seed: seed }
    }

    /// Return the next pseudo random number expressed as a `Scalar` in the range
    /// `[-SK_Scalar1..SK_Scalar1)`.
    fn next_sscalar1(&mut self) -> Scalar {
        sk_fixed_to_scalar(self.next_sfixed1())
    }

    /// Return the next pseudo random number as an unsigned 32-bit value.
    fn next_u(&mut self) -> u32 {
        let r = self
            .f_seed
            .wrapping_mul(Self::K_MUL)
            .wrapping_add(Self::K_ADD);
        self.f_seed = r;
        r
    }

    /// Return the next pseudo random number as a signed 32-bit value.
    fn next_s(&mut self) -> i32 {
        self.next_u() as i32
    }

    /// Return the next pseudo random number expressed as a signed `SkFixed` in the range
    /// `[-SK_Fixed1..SK_Fixed1)`.
    fn next_sfixed1(&mut self) -> i32 {
        self.next_s() >> 15
    }
}

/// `Perterb`.
fn perterb(p: &mut Point, tangent: &Vector, scale: Scalar) {
    let mut normal = *tangent;
    PointPriv::rotate_ccw_in_place(&mut normal);
    normal.set_length(scale);
    *p += normal;
}

/// `SkDiscretePathEffectImpl`.
struct SkDiscretePathEffectImpl {
    f_seg_length: Scalar,
    f_perterb: Scalar,
    f_seed_assist: u32,
}

impl SkPathEffectBase for SkDiscretePathEffectImpl {
    fn on_filter_path(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        _cull_rect: Option<&Rect>,
        _ctm: &Matrix,
    ) -> bool {
        let do_fill = rec.is_fill_style();

        let mut meas = SkPathMeasure::from_path(src, do_fill, 1.0);

        /* Caller may supply their own seed assist, which by default is 0 */
        let seed = self.f_seed_assist ^ (scalar_round_to_int(meas.get_length()) as u32);

        // `seed ^ ((seed << 16) | (seed >> 16))` — a 16-bit rotate of the seed.
        let mut rand = LcgRandom::new(seed ^ seed.rotate_right(16));
        let scale = self.f_perterb;
        let mut p = Point::new(0.0, 0.0);
        let mut v = Vector::new(0.0, 0.0);

        loop {
            let length = meas.get_length();

            if self.f_seg_length * (2.0 + if do_fill { 1.0 } else { 0.0 }) > length {
                meas.get_segment(0.0, length, dst, true); // to short for us to mangle
            } else {
                let mut n = scalar_round_to_int(length / self.f_seg_length);
                const K_MAX_REASONABLE_ITERATIONS: i32 = 100000;
                n = if n < K_MAX_REASONABLE_ITERATIONS {
                    n
                } else {
                    K_MAX_REASONABLE_ITERATIONS
                };
                let delta = length / n as Scalar;
                let mut distance = 0.0;

                if meas.is_closed() {
                    n -= 1;
                    distance += delta / 2.0;
                }

                if meas.get_pos_tan(distance, Some(&mut p), Some(&mut v)) {
                    perterb(&mut p, &v, rand.next_sscalar1() * scale);
                    dst.move_to(p);
                }
                loop {
                    n -= 1;
                    if n < 0 {
                        break;
                    }
                    distance += delta;
                    if meas.get_pos_tan(distance, Some(&mut p), Some(&mut v)) {
                        perterb(&mut p, &v, rand.next_sscalar1() * scale);
                        dst.line_to(p);
                    }
                }
                if meas.is_closed() {
                    dst.close();
                }
            }

            if !meas.next_contour() {
                break;
            }
        }
        true
    }

    fn compute_fast_bounds(&self, bounds: Option<&mut Rect>) -> bool {
        if let Some(bounds) = bounds {
            let max_outset = crate::scalar::scalar_abs(self.f_perterb);
            bounds.outset(max_outset, max_outset);
        }
        true
    }
}

/// `SkDiscretePathEffect::Make`.
pub fn make_discrete_path_effect(
    seg_length: Scalar,
    deviation: Scalar,
    seed_assist: u32,
) -> Option<SkPathEffect> {
    if !crate::floating_point::is_finite2(seg_length, deviation) {
        return None;
    }
    if seg_length <= crate::scalar::SCALAR_NEARLY_ZERO {
        return None;
    }
    Some(SkPathEffect::from_base(SkDiscretePathEffectImpl {
        f_seg_length: seg_length,
        f_perterb: deviation,
        f_seed_assist: seed_assist,
    }))
}

/// `SkDiscretePathEffect`.
pub struct SkDiscretePathEffect;

impl SkDiscretePathEffect {
    /// `SkDiscretePathEffect::Make`: break the path into segments of `seg_length` length, and
    /// randomly move the endpoints away from the original path by a maximum of `deviation`.
    ///
    /// `seed_assist` modifies the seed used to randomize the segment endpoints. A default of 0
    /// makes filtering a path multiple times produce the same set of segments.
    pub fn make(seg_length: Scalar, deviation: Scalar, seed_assist: u32) -> Option<SkPathEffect> {
        make_discrete_path_effect(seg_length, deviation, seed_assist)
    }
}
