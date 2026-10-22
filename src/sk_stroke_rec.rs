//! Port of `include/core/SkStrokeRec.h` and `src/core/SkStrokeRec.cpp`.
//!
//! The reference packs `fCap : 16`, `fJoin : 15` and `fStrokeAndFill : 1` into a single `uint32_t`
//! so that `SK_BEGIN_REQUIRE_DENSE` layouts compare byte-for-byte. The Rust port keeps the same
//! packed `u32` and the same field order, so the type is 16 bytes with no padding.
//!
//! `SkPaint` is owned by M13. The constructor that reads a paint is therefore expressed as
//! [`SkStrokeRec::from_paint_params`] (the only paint inputs are width, miter, cap, join and the
//! style). `applyToPaint` and `GetInflationRadius(const SkPaint&, Style)` stay pending on M13.

use crate::scalar::{SCALAR_1, SCALAR_SQRT2, Scalar};
use crate::sk_paint_types::{
    SK_PAINT_DEFAULT_CAP, SK_PAINT_DEFAULT_JOIN, SK_PAINT_DEFAULTS_MITER_LIMIT, SkPaintCap,
    SkPaintJoin, SkPaintStyle,
};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_stroke::SkStroke;

/// `kStrokeRec_FillStyleWidth`: must be `< 0`, since `0` means hairline and `> 0` means normal
/// stroke.
pub const K_STROKE_REC_FILL_STYLE_WIDTH: Scalar = -1.0;

const CAP_MASK: u32 = 0xFFFF;
const JOIN_SHIFT: u32 = 16;
const JOIN_MASK: u32 = 0x7FFF;
const STROKE_AND_FILL_SHIFT: u32 = 31;

/// `SkStrokeRec::InitStyle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkStrokeRecInitStyle {
    Hairline,
    Fill,
}

/// `SkStrokeRec::Style`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkStrokeRecStyle {
    Hairline,
    Fill,
    Stroke,
    StrokeAndFill,
}

/// `SkStrokeRec::kStyleCount` (`kStrokeAndFill_Style + 1`).
pub const SK_STROKE_REC_STYLE_COUNT: usize = 4;

/// `SkStrokeRec`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkStrokeRec {
    f_res_scale: Scalar,
    f_width: Scalar,
    f_miter_limit: Scalar,
    /// `fCap : 16`, `fJoin : 15`, `fStrokeAndFill : 1`, packed exactly like the reference.
    f_flags: u32,
}

#[inline]
fn pack_flags(cap: SkPaintCap, join: SkPaintJoin, stroke_and_fill: bool) -> u32 {
    ((cap as u32) & CAP_MASK)
        | (((join as u32) & JOIN_MASK) << JOIN_SHIFT)
        | ((stroke_and_fill as u32) << STROKE_AND_FILL_SHIFT)
}

impl SkStrokeRec {
    pub fn new(init_style: SkStrokeRecInitStyle) -> Self {
        SkStrokeRec {
            f_res_scale: 1.0,
            f_width: if init_style == SkStrokeRecInitStyle::Fill {
                K_STROKE_REC_FILL_STYLE_WIDTH
            } else {
                0.0
            },
            f_miter_limit: SK_PAINT_DEFAULTS_MITER_LIMIT,
            f_flags: pack_flags(SK_PAINT_DEFAULT_CAP, SK_PAINT_DEFAULT_JOIN, false),
        }
    }

    /// `SkStrokeRec(const SkPaint&, SkPaint::Style, SkScalar resScale)` without `SkPaint`.
    ///
    /// The reference copies `getStrokeWidth()`, `getStrokeMiter()`, `getStrokeCap()`,
    /// `getStrokeJoin()` and uses the caller-supplied style; those are the only paint inputs.
    pub fn from_paint_params(
        style: SkPaintStyle,
        stroke_width: Scalar,
        stroke_miter: Scalar,
        stroke_cap: SkPaintCap,
        stroke_join: SkPaintJoin,
        res_scale: Scalar,
    ) -> Self {
        let (width, stroke_and_fill) = match style {
            SkPaintStyle::Fill => (K_STROKE_REC_FILL_STYLE_WIDTH, false),
            SkPaintStyle::Stroke => (stroke_width, false),
            SkPaintStyle::StrokeAndFill => {
                if 0.0 == stroke_width {
                    // hairline+fill == fill
                    (K_STROKE_REC_FILL_STYLE_WIDTH, false)
                } else {
                    (stroke_width, true)
                }
            }
        };

        SkStrokeRec {
            f_res_scale: res_scale,
            f_width: width,
            f_miter_limit: stroke_miter,
            f_flags: pack_flags(stroke_cap, stroke_join, stroke_and_fill),
        }
    }

    /// `SkStrokeRec(const SkPaint&, SkScalar resScale)` — style is `paint.getStyle()`.
    pub fn from_paint_params_with_paint_style(
        paint_style: SkPaintStyle,
        stroke_width: Scalar,
        stroke_miter: Scalar,
        stroke_cap: SkPaintCap,
        stroke_join: SkPaintJoin,
        res_scale: Scalar,
    ) -> Self {
        Self::from_paint_params(
            paint_style,
            stroke_width,
            stroke_miter,
            stroke_cap,
            stroke_join,
            res_scale,
        )
    }

    #[inline]
    fn cap_bits(&self) -> SkPaintCap {
        SkPaintCap::from_u32(self.f_flags & CAP_MASK)
    }

    #[inline]
    fn join_bits(&self) -> SkPaintJoin {
        SkPaintJoin::from_u32((self.f_flags >> JOIN_SHIFT) & JOIN_MASK)
    }

    #[inline]
    fn stroke_and_fill_bit(&self) -> bool {
        (self.f_flags >> STROKE_AND_FILL_SHIFT) & 1 != 0
    }

    pub fn get_style(&self) -> SkStrokeRecStyle {
        if self.f_width < 0.0 {
            SkStrokeRecStyle::Fill
        } else if 0.0 == self.f_width {
            SkStrokeRecStyle::Hairline
        } else if self.stroke_and_fill_bit() {
            SkStrokeRecStyle::StrokeAndFill
        } else {
            SkStrokeRecStyle::Stroke
        }
    }

    #[inline]
    pub fn get_width(&self) -> Scalar {
        self.f_width
    }

    #[inline]
    pub fn get_miter(&self) -> Scalar {
        self.f_miter_limit
    }

    #[inline]
    pub fn get_cap(&self) -> SkPaintCap {
        self.cap_bits()
    }

    #[inline]
    pub fn get_join(&self) -> SkPaintJoin {
        self.join_bits()
    }

    #[inline]
    pub fn is_hairline_style(&self) -> bool {
        SkStrokeRecStyle::Hairline == self.get_style()
    }

    #[inline]
    pub fn is_fill_style(&self) -> bool {
        SkStrokeRecStyle::Fill == self.get_style()
    }

    pub fn set_fill_style(&mut self) {
        self.f_width = K_STROKE_REC_FILL_STYLE_WIDTH;
        self.set_stroke_and_fill(false);
    }

    pub fn set_hairline_style(&mut self) {
        self.f_width = 0.0;
        self.set_stroke_and_fill(false);
    }

    /// Specify the stroke width, and optionally stroke + fill.
    ///
    /// If `width == 0`, `stroke_and_fill == true` means the new style is Fill, and
    /// `stroke_and_fill == false` means Hairline.
    pub fn set_stroke_style(&mut self, width: Scalar, stroke_and_fill: bool) {
        if stroke_and_fill && (0.0 == width) {
            // hairline+fill == fill
            self.set_fill_style();
        } else {
            self.f_width = width;
            self.set_stroke_and_fill(stroke_and_fill);
        }
    }

    pub fn set_stroke_params(&mut self, cap: SkPaintCap, join: SkPaintJoin, miter_limit: Scalar) {
        self.set_cap(cap);
        self.set_join(join);
        self.f_miter_limit = miter_limit;
    }

    #[inline]
    pub fn get_res_scale(&self) -> Scalar {
        self.f_res_scale
    }

    pub fn set_res_scale(&mut self, rs: Scalar) {
        debug_assert!(rs > 0.0 && rs.is_finite());
        self.f_res_scale = rs;
    }

    fn set_cap(&mut self, cap: SkPaintCap) {
        self.f_flags = (self.f_flags & !CAP_MASK) | ((cap as u32) & CAP_MASK);
    }

    fn set_join(&mut self, join: SkPaintJoin) {
        self.f_flags = (self.f_flags & !(JOIN_MASK << JOIN_SHIFT))
            | (((join as u32) & JOIN_MASK) << JOIN_SHIFT);
    }

    fn set_stroke_and_fill(&mut self, stroke_and_fill: bool) {
        if stroke_and_fill {
            self.f_flags |= 1 << STROKE_AND_FILL_SHIFT;
        } else {
            self.f_flags &= !(1 << STROKE_AND_FILL_SHIFT);
        }
    }

    /// Returns true if this specifies any thick stroking, i.e. `apply_to_path` returns true.
    pub fn need_to_apply(&self) -> bool {
        let style = self.get_style();
        SkStrokeRecStyle::Stroke == style || SkStrokeRecStyle::StrokeAndFill == style
    }

    /// Apply these stroke parameters to the source path, returning the result in `dst`.
    ///
    /// If there was no change (style hairline or fill) this returns false and `dst` is unchanged.
    /// `src` and `dst` may be the same path in C++; in Rust they are distinct objects.
    pub fn apply_to_path(&self, dst: &mut SkPathBuilder, src: &SkPath) -> bool {
        if self.f_width <= 0.0 {
            // hairline or fill
            return false;
        }

        let mut stroker = SkStroke::new();
        stroker.set_cap(self.cap_bits());
        stroker.set_join(self.join_bits());
        stroker.set_miter_limit(self.f_miter_limit);
        stroker.set_width(self.f_width);
        stroker.set_do_fill(self.stroke_and_fill_bit());
        stroker.set_res_scale(self.f_res_scale);
        stroker.stroke_path(src, dst);
        true
    }

    /// Conservative outset to apply to a geometry's bounds for this stroke rec.
    pub fn get_inflation_radius(&self) -> Scalar {
        Self::get_inflation_radius_for(
            self.join_bits(),
            self.f_miter_limit,
            self.cap_bits(),
            self.f_width,
        )
    }

    /// `SkStrokeRec::GetInflationRadius(SkPaint::Join, SkScalar, SkPaint::Cap, SkScalar)`.
    pub fn get_inflation_radius_for(
        join: SkPaintJoin,
        miter_limit: Scalar,
        cap: SkPaintCap,
        stroke_width: Scalar,
    ) -> Scalar {
        if stroke_width < 0.0 {
            // fill
            return 0.0;
        } else if 0.0 == stroke_width {
            // FIXME: We need a "matrixScale" parameter here in order to properly handle hairlines.
            // Their width is determined in device space (skbug.com/40039419).
            return SCALAR_1;
        }

        // since we're stroked, outset the rect by the radius (and join type, caps)
        let mut multiplier = SCALAR_1;
        if SkPaintJoin::Miter == join {
            multiplier = if multiplier < miter_limit {
                miter_limit
            } else {
                multiplier
            };
        }
        if SkPaintCap::Square == cap {
            multiplier = if multiplier < SCALAR_SQRT2 {
                SCALAR_SQRT2
            } else {
                multiplier
            };
        }
        stroke_width / 2.0 * multiplier
    }

    /// `SkStrokeRec::GetInflationRadius(const SkPaint&, SkPaint::Style)` without `SkPaint`.
    pub fn get_inflation_radius_for_style(
        style: SkPaintStyle,
        stroke_width: Scalar,
        stroke_join: SkPaintJoin,
        stroke_miter: Scalar,
        stroke_cap: SkPaintCap,
    ) -> Scalar {
        let width = if SkPaintStyle::Fill == style {
            -SCALAR_1
        } else {
            stroke_width
        };
        Self::get_inflation_radius_for(stroke_join, stroke_miter, stroke_cap, width)
    }

    /// Compare if two `SkStrokeRec`s have an equal effect on a path.
    ///
    /// Equal records produce equal paths. `res_scale` is not part of the comparison.
    pub fn has_equal_effect(&self, other: &SkStrokeRec) -> bool {
        if !self.need_to_apply() {
            return self.get_style() == other.get_style();
        }
        self.f_width == other.f_width
            && (self.join_bits() != SkPaintJoin::Miter || self.f_miter_limit == other.f_miter_limit)
            && self.cap_bits() == other.cap_bits()
            && self.join_bits() == other.join_bits()
            && self.stroke_and_fill_bit() == other.stroke_and_fill_bit()
    }
}

impl Default for SkStrokeRec {
    fn default() -> Self {
        Self::new(SkStrokeRecInitStyle::Hairline)
    }
}
