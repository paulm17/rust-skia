//! Port of `include/core/SkPathTypes.h`, `src/core/SkPathEnums.h` and `include/core/SkArc.h`.
//!
//! `SkPathDirection` and `SkPathVerb` are defined alongside the code that first needed them
//! (M06: `sk_geometry.rs` and `sk_edge_clipper.rs`) and are re-exported here so the path
//! subsystem has a single import site.

use crate::point::Point;
use crate::rect::Rect;
use crate::scalar::Scalar;

pub use crate::sk_edge_clipper::SkPathVerb;
pub use crate::sk_geometry::SkPathDirection;

/// `SkPathFillType` from `include/core/SkPathTypes.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SkPathFillType {
    #[default]
    Winding = 0,
    EvenOdd = 1,
    InverseWinding = 2,
    InverseEvenOdd = 3,
}

impl SkPathFillType {
    pub const DEFAULT: SkPathFillType = SkPathFillType::Winding;

    #[inline]
    pub fn is_even_odd(self) -> bool {
        (self as i32 & 1) != 0
    }

    #[inline]
    pub fn is_inverse(self) -> bool {
        (self as i32 & 2) != 0
    }

    #[inline]
    pub fn toggle_inverse(self) -> SkPathFillType {
        match (self as i32) ^ 2 {
            0 => SkPathFillType::Winding,
            1 => SkPathFillType::EvenOdd,
            2 => SkPathFillType::InverseWinding,
            _ => SkPathFillType::InverseEvenOdd,
        }
    }

    #[inline]
    pub fn convert_to_non_inverse(self) -> SkPathFillType {
        if self.is_even_odd() {
            SkPathFillType::EvenOdd
        } else {
            SkPathFillType::Winding
        }
    }
}

/// `SkPathFillType_IsEvenOdd`.
#[inline]
pub fn sk_path_fill_type_is_even_odd(ft: SkPathFillType) -> bool {
    ft.is_even_odd()
}

/// `SkPathFillType_IsInverse`.
#[inline]
pub fn sk_path_fill_type_is_inverse(ft: SkPathFillType) -> bool {
    ft.is_inverse()
}

/// `SkPathFillType_ToggleInverse`.
#[inline]
pub fn sk_path_fill_type_toggle_inverse(ft: SkPathFillType) -> SkPathFillType {
    ft.toggle_inverse()
}

/// `SkPathFillType_ConvertToNonInverse`.
#[inline]
pub fn sk_path_fill_type_convert_to_non_inverse(ft: SkPathFillType) -> SkPathFillType {
    ft.convert_to_non_inverse()
}

/// `kLine_SkPathSegmentMask` etc. from `SkPathTypes.h`.
pub const K_LINE_SK_PATH_SEGMENT_MASK: u8 = 1 << 0;
pub const K_QUAD_SK_PATH_SEGMENT_MASK: u8 = 1 << 1;
pub const K_CONIC_SK_PATH_SEGMENT_MASK: u8 = 1 << 2;
pub const K_CUBIC_SK_PATH_SEGMENT_MASK: u8 = 1 << 3;

/// `SkPathConvexity` from `src/core/SkPathEnums.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SkPathConvexity {
    ConvexCw,
    ConvexCcw,
    ConvexDegenerate,
    Concave,
    #[default]
    Unknown,
}

impl SkPathConvexity {
    #[inline]
    pub fn is_convex(self) -> bool {
        matches!(
            self,
            SkPathConvexity::ConvexCw
                | SkPathConvexity::ConvexCcw
                | SkPathConvexity::ConvexDegenerate
        )
    }

    #[inline]
    pub fn opposite_convex_direction(self) -> SkPathConvexity {
        debug_assert!(self.is_convex());
        match self {
            SkPathConvexity::ConvexCw => SkPathConvexity::ConvexCcw,
            SkPathConvexity::ConvexCcw => SkPathConvexity::ConvexCw,
            other => other,
        }
    }

    #[inline]
    pub fn to_direction(self) -> Option<SkPathDirection> {
        match self {
            SkPathConvexity::ConvexCw => Some(SkPathDirection::Cw),
            SkPathConvexity::ConvexCcw => Some(SkPathDirection::Ccw),
            _ => None,
        }
    }

    #[inline]
    pub fn to_first_direction(self) -> SkPathFirstDirection {
        match self {
            SkPathConvexity::ConvexCw => SkPathFirstDirection::Cw,
            SkPathConvexity::ConvexCcw => SkPathFirstDirection::Ccw,
            _ => SkPathFirstDirection::Unknown,
        }
    }
}

/// `SkPathConvexity_IsConvex`.
#[inline]
pub fn sk_path_convexity_is_convex(cv: SkPathConvexity) -> bool {
    cv.is_convex()
}

/// `SkPathConvexity_OppositeConvexDirection`.
#[inline]
pub fn sk_path_convexity_opposite_convex_direction(cv: SkPathConvexity) -> SkPathConvexity {
    cv.opposite_convex_direction()
}

/// `SkPathConvexity_ToDirection`.
#[inline]
pub fn sk_path_convexity_to_direction(cv: SkPathConvexity) -> Option<SkPathDirection> {
    cv.to_direction()
}

/// `SkPathConvexity_ToFirstDirection`.
#[inline]
pub fn sk_path_convexity_to_first_direction(cv: SkPathConvexity) -> SkPathFirstDirection {
    cv.to_first_direction()
}

/// `SkResolveConvexity` from `src/core/SkPathEnums.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SkResolveConvexity {
    #[default]
    No = 0,
    Yes = 1,
}

/// `SkPathFirstDirection` from `src/core/SkPathEnums.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SkPathFirstDirection {
    Cw,
    Ccw,
    #[default]
    Unknown,
}

/// `SkPathDirection_ToConvexity`.
#[inline]
pub fn sk_path_direction_to_convexity(dir: SkPathDirection) -> SkPathConvexity {
    match dir {
        SkPathDirection::Cw => SkPathConvexity::ConvexCw,
        SkPathDirection::Ccw => SkPathConvexity::ConvexCcw,
    }
}

/// `SkPathFirstDirection_ToConvexity`.
#[inline]
pub fn sk_path_first_direction_to_convexity(dir: SkPathFirstDirection) -> SkPathConvexity {
    match dir {
        SkPathFirstDirection::Cw => SkPathConvexity::ConvexCw,
        SkPathFirstDirection::Ccw => SkPathConvexity::ConvexCcw,
        SkPathFirstDirection::Unknown => SkPathConvexity::ConvexDegenerate,
    }
}

/// `SkPathDirectionToFirst`.
#[inline]
pub fn sk_path_direction_to_first(dir: SkPathDirection) -> SkPathFirstDirection {
    match dir {
        SkPathDirection::Cw => SkPathFirstDirection::Cw,
        SkPathDirection::Ccw => SkPathFirstDirection::Ccw,
    }
}

/// `SkArc` from `include/core/SkArc.h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkArc {
    pub f_oval: Rect,
    pub f_start_angle: Scalar,
    pub f_sweep_angle: Scalar,
    pub f_type: SkArcType,
}

/// `SkArc::Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkArcType {
    Arc = 0,
    Wedge = 1,
}

impl Default for SkArc {
    fn default() -> Self {
        SkArc {
            f_oval: Rect::make_empty(),
            f_start_angle: 0.0,
            f_sweep_angle: 0.0,
            f_type: SkArcType::Arc,
        }
    }
}

impl SkArc {
    #[inline]
    pub fn oval(&self) -> &Rect {
        &self.f_oval
    }

    #[inline]
    pub fn start_angle(&self) -> Scalar {
        self.f_start_angle
    }

    #[inline]
    pub fn sweep_angle(&self) -> Scalar {
        self.f_sweep_angle
    }

    #[inline]
    pub fn is_wedge(&self) -> bool {
        self.f_type == SkArcType::Wedge
    }

    pub fn make(
        oval: &Rect,
        start_angle_degrees: Scalar,
        sweep_angle_degrees: Scalar,
        arc_type: SkArcType,
    ) -> SkArc {
        SkArc {
            f_oval: *oval,
            f_start_angle: start_angle_degrees,
            f_sweep_angle: sweep_angle_degrees,
            f_type: arc_type,
        }
    }

    pub fn make_use_center(
        oval: &Rect,
        start_angle_degrees: Scalar,
        sweep_angle_degrees: Scalar,
        use_center: bool,
    ) -> SkArc {
        Self::make(
            oval,
            start_angle_degrees,
            sweep_angle_degrees,
            if use_center {
                SkArcType::Wedge
            } else {
                SkArcType::Arc
            },
        )
    }
}

/// Convenience constructor mirroring `SkPoint::Make`.
#[inline]
pub fn sk_point_make(x: Scalar, y: Scalar) -> Point {
    Point::new(x, y)
}
