//! Port of `include/effects/SkTrimPathEffect.h`, `src/effects/SkTrimPE.h` and
//! `src/effects/SkTrimPathEffect.cpp`.

use crate::matrix::Matrix;
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_pin};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::{SkPathEffect, SkPathEffectBase};
use crate::sk_path_measure::SkPathMeasure;
use crate::sk_stroke_rec::SkStrokeRec;

/// `SkTrimPathEffect::Mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkTrimPathEffectMode {
    /// return the subset path [start,stop]
    Normal,
    /// return the complement/subset paths [0,start] + [stop,1]
    Inverted,
}

/// Returns the number of contours iterated to satisfy the request.
fn add_segments(
    src: &SkPath,
    start: Scalar,
    stop: Scalar,
    dst: &mut SkPathBuilder,
    requires_moveto: bool,
) -> usize {
    debug_assert!(start < stop);

    let mut measure = SkPathMeasure::from_path(src, false, 1.0);

    let mut current_segment_offset = 0.0;
    let mut contour_count = 1usize;

    loop {
        let next_offset = current_segment_offset + measure.get_length();

        if start < next_offset {
            measure.get_segment(
                start - current_segment_offset,
                stop - current_segment_offset,
                dst,
                requires_moveto,
            );

            if stop <= next_offset {
                break;
            }
        }

        contour_count += 1;
        current_segment_offset = next_offset;

        if !measure.next_contour() {
            break;
        }
    }

    contour_count
}

/// `SkTrimPE`.
pub struct SkTrimPE {
    f_start_t: Scalar,
    f_stop_t: Scalar,
    f_mode: SkTrimPathEffectMode,
}

impl SkTrimPE {
    pub fn new(start_t: Scalar, stop_t: Scalar, mode: SkTrimPathEffectMode) -> Self {
        SkTrimPE {
            f_start_t: start_t,
            f_stop_t: stop_t,
            f_mode: mode,
        }
    }
}

impl SkPathEffectBase for SkTrimPE {
    fn on_filter_path(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        _rec: &mut SkStrokeRec,
        _cull_rect: Option<&Rect>,
        _ctm: &Matrix,
    ) -> bool {
        if self.f_start_t >= self.f_stop_t {
            debug_assert!(self.f_mode == SkTrimPathEffectMode::Normal);
            return true;
        }

        // First pass: compute the total len.
        let mut len = 0.0;
        let mut meas = SkPathMeasure::from_path(src, false, 1.0);
        loop {
            len += meas.get_length();
            if !meas.next_contour() {
                break;
            }
        }

        let arc_start = len * self.f_start_t;
        let arc_stop = len * self.f_stop_t;

        // Second pass: actually add segments.
        if self.f_mode == SkTrimPathEffectMode::Normal {
            // Normal mode -> one span.
            if arc_start < arc_stop {
                add_segments(src, arc_start, arc_stop, dst, true);
            }
        } else {
            // Inverted mode -> one logical span which wraps around at the end -> two actual spans.
            // In order to preserve closed path continuity:
            //
            //   1) add the second/tail span first
            //
            //   2) skip the head span move-to for single-closed-contour paths

            let mut requires_moveto = true;
            if arc_stop < len {
                // since we're adding the "tail" first, this is the total number of contours
                let contour_count = add_segments(src, arc_stop, len, dst, true);

                // if the path consists of a single closed contour, we don't want to disconnect
                // the two parts with a moveto.
                if contour_count == 1 && src.is_last_contour_closed() {
                    requires_moveto = false;
                }
            }
            if 0.0 < arc_start {
                add_segments(src, 0.0, arc_start, dst, requires_moveto);
            }
        }

        true
    }

    fn compute_fast_bounds(&self, _bounds: Option<&mut Rect>) -> bool {
        // Trimming a path returns a subset of the input path so just return true and leave bounds
        // unmodified
        true
    }
}

/// `SkTrimPathEffect`.
pub struct SkTrimPathEffect;

impl SkTrimPathEffect {
    /// `SkTrimPathEffect::Make`.
    ///
    /// Take start and stop "t" values (values between 0...1) and return a path that is that subset
    /// of the original path. `start_t`/`stop_t` are pinned to 0..1; a NaN returns `None`.
    pub fn make(
        start_t: Scalar,
        stop_t: Scalar,
        mode: SkTrimPathEffectMode,
    ) -> Option<SkPathEffect> {
        if !crate::floating_point::is_finite2(start_t, stop_t) {
            return None;
        }

        if start_t <= 0.0 && stop_t >= 1.0 && mode == SkTrimPathEffectMode::Normal {
            return None;
        }

        let start_t = scalar_pin(start_t, 0.0, 1.0);
        let stop_t = scalar_pin(stop_t, 0.0, 1.0);

        if start_t >= stop_t && mode == SkTrimPathEffectMode::Inverted {
            return None;
        }

        Some(SkPathEffect::from_base(SkTrimPE::new(
            start_t, stop_t, mode,
        )))
    }
}
