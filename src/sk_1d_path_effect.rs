//! Port of `include/effects/Sk1DPathEffect.h` and `src/effects/Sk1DPathEffect.cpp`.
//!
//! The reference declares an abstract `Sk1DPathEffect` whose `onFilterPath` walks the source
//! contours with a governor and delegates to the virtual `begin`/`next`. Its only concrete
//! subclass in this translation unit is `SkPath1DPathEffectImpl`; the port keeps the shared walk
//! as a private method on that struct and preserves the same `begin` (`fInitialOffset`) and
//! `next` behavior.

use crate::floating_point::{float_midpoint, is_finite2};
use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_mod};
use crate::sk_contour_measure::MatrixFlags;
use crate::sk_path::{AddPathMode, Iter, SkPath};
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::{SkPathEffect, SkPathEffectBase};
use crate::sk_path_measure::SkPathMeasure;
use crate::sk_path_types::SkPathVerb;
use crate::sk_stroke_rec::SkStrokeRec;

/// Since we are stepping by a float, the do/while loop might go on forever (or nearly so).
/// Put in a governor to limit crash values from looping too long (and allocating too much ram).
pub const MAX_REASONABLE_ITERATIONS: i32 = 100000;

/// `SkPath1DPathEffect::Style`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPath1DPathEffectStyle {
    /// translate the shape to each position
    Translate,
    /// rotate the shape about its center
    Rotate,
    /// transform each point, and turn lines into curves
    Morph,
}

/// `SkPath1DPathEffectImpl`.
struct SkPath1DPathEffectImpl {
    /// copied from constructor
    f_path: SkPath,
    /// copied from constructor
    f_advance: Scalar,
    /// computed from phase
    f_initial_offset: Scalar,
    /// copied from constructor
    f_style: SkPath1DPathEffectStyle,
}

impl SkPath1DPathEffectImpl {
    fn new(
        path: &SkPath,
        advance: Scalar,
        mut phase: Scalar,
        style: SkPath1DPathEffectStyle,
    ) -> Self {
        debug_assert!(advance > 0.0 && !path.is_empty());

        // cleanup their phase parameter, inverting it so that it becomes an
        // offset along the path (to match the interpretation in PostScript)
        if phase < 0.0 {
            phase = -phase;
            if phase > advance {
                phase = scalar_mod(phase, advance);
            }
        } else {
            if phase > advance {
                phase = scalar_mod(phase, advance);
            }
            phase = advance - phase;
        }
        // now catch the edge case where phase == advance (within epsilon)
        if phase >= advance {
            phase = 0.0;
        }
        debug_assert!(phase >= 0.0);

        SkPath1DPathEffectImpl {
            f_path: path.clone(),
            f_advance: advance,
            f_initial_offset: phase,
            f_style: style,
        }
    }

    /// `Sk1DPathEffect::onFilterPath`.
    fn filter_path_impl(&self, builder: &mut SkPathBuilder, src: &SkPath) -> bool {
        let mut meas = SkPathMeasure::from_path(src, false, 1.0);
        loop {
            let mut governor = MAX_REASONABLE_ITERATIONS;
            let length = meas.get_length();
            let mut distance = self.f_initial_offset;
            while distance < length {
                governor -= 1;
                if governor < 0 {
                    break;
                }
                let delta = self.next(builder, distance, &mut meas);
                if delta <= 0.0 {
                    break;
                }
                distance += delta;
            }
            if governor < 0 {
                return false;
            }
            if !meas.next_contour() {
                break;
            }
        }
        true
    }

    /// Called with the current distance along the path, with the current matrix for the
    /// point/tangent at the specified distance. Returns the distance to travel for the next call.
    fn next(
        &self,
        builder: &mut SkPathBuilder,
        distance: Scalar,
        meas: &mut SkPathMeasure,
    ) -> Scalar {
        match self.f_style {
            SkPath1DPathEffectStyle::Translate => {
                let mut pos = Point::new(0.0, 0.0);
                if meas.get_pos_tan(distance, Some(&mut pos), None) {
                    builder.add_path_offset(&self.f_path, pos.x, pos.y, AddPathMode::Append);
                }
            }
            SkPath1DPathEffectStyle::Rotate => {
                let mut matrix = Matrix::new();
                if meas.get_matrix(distance, Some(&mut matrix), MatrixFlags::GET_POS_AND_TAN) {
                    builder.add_path_matrix(&self.f_path, &matrix, AddPathMode::Append);
                }
            }
            SkPath1DPathEffectStyle::Morph => {
                morphpath(builder, &self.f_path, meas, distance);
            }
        }
        self.f_advance
    }
}

impl SkPathEffectBase for SkPath1DPathEffectImpl {
    fn on_filter_path(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        _cull_rect: Option<&Rect>,
        _ctm: &Matrix,
    ) -> bool {
        rec.set_fill_style();
        self.filter_path_impl(builder, src)
    }

    fn compute_fast_bounds(&self, _bounds: Option<&mut Rect>) -> bool {
        // For simplicity, assume fast bounds cannot be computed
        false
    }
}

/// `morphpoints`.
fn morphpoints(dst: &mut [Point], src: &[Point], meas: &SkPathMeasure, dist: Scalar) -> bool {
    debug_assert!(dst.len() >= src.len());
    for (i, src_pt) in src.iter().enumerate() {
        let mut pos = Point::new(0.0, 0.0);
        let mut tangent = Vector::new(0.0, 0.0);

        let sx = src_pt.x;
        let sy = src_pt.y;

        if !meas.get_pos_tan(dist + sx, Some(&mut pos), Some(&mut tangent)) {
            return false;
        }

        let mut matrix = Matrix::new();
        let pt = Point::new(sx, sy);

        matrix.set_sin_cos_pivot(tangent.y, tangent.x, 0.0, 0.0);
        matrix.pre_translate(-sx, 0.0);
        matrix.post_translate(pos.x, pos.y);
        dst[i] = matrix.map_point(pt);
    }
    true
}

/// `morphpath`.
fn morphpath(dst: &mut SkPathBuilder, src: &SkPath, meas: &mut SkPathMeasure, dist: Scalar) {
    let mut iter = Iter::from_path(src, false);
    let mut dst_p = [Point::new(0.0, 0.0); 3];

    while let Some(rec) = iter.next_rec() {
        let src_p = rec.f_points;
        let verb = rec.f_verb;
        let conic_weight = rec.f_conic_weight;
        match verb {
            SkPathVerb::Move => {
                if morphpoints(&mut dst_p[..1], src_p, meas, dist) {
                    dst.move_to(dst_p[0]);
                }
            }
            SkPathVerb::Line => {
                let scratch = [
                    src_p[0],
                    Point::new(
                        float_midpoint(src_p[0].x, src_p[1].x),
                        float_midpoint(src_p[0].y, src_p[1].y),
                    ),
                    src_p[1],
                ];
                // now we look like a quad
                if morphpoints(&mut dst_p[..2], &scratch[1..], meas, dist) {
                    dst.quad_to(dst_p[0], dst_p[1]);
                }
            }
            SkPathVerb::Quad => {
                if morphpoints(&mut dst_p[..2], &src_p[1..], meas, dist) {
                    dst.quad_to(dst_p[0], dst_p[1]);
                }
            }
            SkPathVerb::Conic => {
                if morphpoints(&mut dst_p[..2], &src_p[1..], meas, dist) {
                    dst.conic_to(dst_p[0], dst_p[1], conic_weight);
                }
            }
            SkPathVerb::Cubic => {
                if morphpoints(&mut dst_p[..3], &src_p[1..], meas, dist) {
                    dst.cubic_to(dst_p[0], dst_p[1], dst_p[2]);
                }
            }
            SkPathVerb::Close => {
                dst.close();
            }
            SkPathVerb::Done => {}
        }
    }
}

/// `SkPath1DPathEffect::Make`.
pub fn make_1d_path_effect(
    path: &SkPath,
    advance: Scalar,
    phase: Scalar,
    style: SkPath1DPathEffectStyle,
) -> Option<SkPathEffect> {
    if advance <= 0.0 || !is_finite2(advance, phase) || path.is_empty() {
        return None;
    }
    Some(SkPathEffect::from_base(SkPath1DPathEffectImpl::new(
        path, advance, phase, style,
    )))
}

/// `SkPath1DPathEffect`.
pub struct SkPath1DPathEffect;

impl SkPath1DPathEffect {
    /// `SkPath1DPathEffect::Make`: dash by replicating the specified path.
    ///
    /// - `path` is the path to replicate (dash)
    /// - `advance` is the space between instances of `path`
    /// - `phase` is the distance (mod advance) along `path` for its initial position
    /// - `style` is how to transform `path` at each point (based on the current position and
    ///   tangent)
    pub fn make(
        path: &SkPath,
        advance: Scalar,
        phase: Scalar,
        style: SkPath1DPathEffectStyle,
    ) -> Option<SkPathEffect> {
        make_1d_path_effect(path, advance, phase, style)
    }
}
