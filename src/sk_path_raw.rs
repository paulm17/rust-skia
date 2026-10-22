//! Port of `src/core/SkPathRaw.h` and `src/core/SkPathRaw.cpp`.
//!
//! Adaptation: the C++ `SkPathRaw` is a non-owning view (spans into caller storage). Rust cannot
//! safely express the self-referential stack shapes, so this port owns its data in `Vec`s. The
//! observable API (`points()`, `verbs()`, `conics()`, `bounds()`, ...) is preserved.

use crate::point::Point;
use crate::rect::Rect;
use crate::sk_path_iter::SkPathIter;
use crate::sk_path_types::{
    SkPathConvexity, SkPathDirection, SkPathFillType, SkPathVerb, sk_path_convexity_is_convex,
    sk_path_fill_type_is_inverse,
};

/// `SkPathRaw` from `src/core/SkPathRaw.h`.
#[derive(Clone, Debug)]
pub struct SkPathRaw {
    pub f_points: Vec<Point>,
    pub f_verbs: Vec<SkPathVerb>,
    pub f_conics: Vec<f32>,
    pub f_bounds: Rect,
    pub f_fill_type: SkPathFillType,
    pub f_convexity: SkPathConvexity,
    pub f_segment_mask: u8,
}

impl SkPathRaw {
    pub fn new(
        f_points: Vec<Point>,
        f_verbs: Vec<SkPathVerb>,
        f_conics: Vec<f32>,
        f_bounds: Rect,
        f_fill_type: SkPathFillType,
        f_convexity: SkPathConvexity,
        f_segment_mask: u8,
    ) -> Self {
        SkPathRaw {
            f_points,
            f_verbs,
            f_conics,
            f_bounds,
            f_fill_type,
            f_convexity,
            f_segment_mask,
        }
    }

    #[inline]
    pub fn points(&self) -> &[Point] {
        &self.f_points
    }

    #[inline]
    pub fn verbs(&self) -> &[SkPathVerb] {
        &self.f_verbs
    }

    #[inline]
    pub fn conics(&self) -> &[f32] {
        &self.f_conics
    }

    #[inline]
    pub fn bounds(&self) -> Rect {
        self.f_bounds
    }

    #[inline]
    pub fn fill_type(&self) -> SkPathFillType {
        self.f_fill_type
    }

    #[inline]
    pub fn convexity(&self) -> SkPathConvexity {
        self.f_convexity
    }

    #[inline]
    pub fn segment_masks(&self) -> u32 {
        self.f_segment_mask as u32
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.f_verbs.is_empty()
    }

    #[inline]
    pub fn is_inverse_fill_type(&self) -> bool {
        sk_path_fill_type_is_inverse(self.f_fill_type)
    }

    #[inline]
    pub fn is_known_to_be_convex(&self) -> bool {
        sk_path_convexity_is_convex(self.f_convexity)
    }

    pub fn is_rect(&self) -> Option<Rect> {
        if let Some(rc) = crate::sk_path_priv::is_rect_contour(
            &self.f_points,
            &self.f_verbs,
            self.f_segment_mask as u32,
            false,
        ) {
            Some(rc.f_rect)
        } else {
            None
        }
    }

    pub fn iter(&self) -> SkPathIter<'_> {
        SkPathIter::new(&self.f_points, &self.f_verbs, &self.f_conics)
    }

    pub fn empty_raw(ft: SkPathFillType) -> SkPathRaw {
        SkPathRaw {
            f_points: Vec::new(),
            f_verbs: Vec::new(),
            f_conics: Vec::new(),
            f_bounds: Rect::make_empty(),
            f_fill_type: ft,
            f_convexity: SkPathConvexity::ConvexDegenerate,
            f_segment_mask: 0,
        }
    }

    /// The C++ default argument is `SkPathFillType::kDefault`.
    pub fn empty_default() -> SkPathRaw {
        Self::empty_raw(SkPathFillType::Winding)
    }
}

impl Default for SkPathRaw {
    fn default() -> Self {
        Self::empty_default()
    }
}

/// Convenience alias kept close to the direction helper `SkPathDirection_ToConvexity`.
#[allow(dead_code)]
fn _unused_dir(_d: SkPathDirection) {}
