//! Paint enumerations and defaults reached by `SkStroke`, `SkStrokeRec` and the path effects.
//!
//! `include/core/SkPaint.h` and `src/core/SkPaintDefaults.h` are owned by M13. The stroking
//! subsystem only consumes `SkPaint::Cap`, `SkPaint::Join`, `SkPaint::Style` and the default
//! miter limit, so those are declared here with the reference numeric values and default
//! selections. M13's `SkPaint` adopts these types unchanged instead of redeclaring them.

use crate::scalar::Scalar;

/// `SkPaint::Cap` from `include/core/SkPaint.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SkPaintCap {
    #[default]
    Butt = 0,
    Round = 1,
    Square = 2,
}

/// `SkPaint::kCapCount`.
pub const SK_PAINT_CAP_COUNT: usize = 3;

impl SkPaintCap {
    #[inline]
    pub fn from_u32(value: u32) -> SkPaintCap {
        match value {
            1 => SkPaintCap::Round,
            2 => SkPaintCap::Square,
            _ => SkPaintCap::Butt,
        }
    }
}

/// `SkPaint::Join` from `include/core/SkPaint.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SkPaintJoin {
    #[default]
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

/// `SkPaint::kJoinCount`.
pub const SK_PAINT_JOIN_COUNT: usize = 3;

impl SkPaintJoin {
    #[inline]
    pub fn from_u32(value: u32) -> SkPaintJoin {
        match value {
            1 => SkPaintJoin::Round,
            2 => SkPaintJoin::Bevel,
            _ => SkPaintJoin::Miter,
        }
    }
}

/// `SkPaint::Style` from `include/core/SkPaint.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SkPaintStyle {
    #[default]
    Fill = 0,
    Stroke = 1,
    StrokeAndFill = 2,
}

/// `SkPaint::kStyleCount`.
pub const SK_PAINT_STYLE_COUNT: usize = 3;

impl SkPaintStyle {
    #[inline]
    pub fn from_u32(value: u32) -> SkPaintStyle {
        match value {
            1 => SkPaintStyle::Stroke,
            2 => SkPaintStyle::StrokeAndFill,
            _ => SkPaintStyle::Fill,
        }
    }
}

/// `SkPaint::kDefault_Cap`.
pub const SK_PAINT_DEFAULT_CAP: SkPaintCap = SkPaintCap::Butt;
/// `SkPaint::kDefault_Join`.
pub const SK_PAINT_DEFAULT_JOIN: SkPaintJoin = SkPaintJoin::Miter;
/// `SkPaintDefaults_MiterLimit` (`src/core/SkPaintDefaults.h`).
pub const SK_PAINT_DEFAULTS_MITER_LIMIT: Scalar = 4.0;
