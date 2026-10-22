//! Port of `src/core/SkPathRawShapes.h` and `src/core/SkPathRawShapes.cpp`.
//!
//! The C++ types derive from `SkPathRaw` and hold their own stack storage. This port constructs
//! an owned `SkPathRaw` directly (see `sk_path_raw.rs` for the ownership adaptation).

use crate::point::Point;
use crate::rect::Rect;
use crate::rrect::RRect;
use crate::scalar::SCALAR_ROOT2_OVER2;
use crate::sk_path_makers::{OvalPointIterator, RRectPointIterator, RectPointIterator};
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_types::{
    K_CONIC_SK_PATH_SEGMENT_MASK, K_LINE_SK_PATH_SEGMENT_MASK, SkPathConvexity, SkPathDirection,
    SkPathFillType, SkPathVerb, sk_path_direction_to_convexity,
};

const K_DEF_FILL_TYPE: SkPathFillType = SkPathFillType::Winding;

const G_RECT_VERBS: [SkPathVerb; 5] = [
    SkPathVerb::Move,
    SkPathVerb::Line,
    SkPathVerb::Line,
    SkPathVerb::Line,
    SkPathVerb::Close,
];

const G_RECT_SEG_MASK: u8 = K_LINE_SK_PATH_SEGMENT_MASK;

const G_OVAL_VERBS: [SkPathVerb; 6] = [
    SkPathVerb::Move,
    SkPathVerb::Conic,
    SkPathVerb::Conic,
    SkPathVerb::Conic,
    SkPathVerb::Conic,
    SkPathVerb::Close,
];

const G_OVAL_SEG_MASK: u8 = K_CONIC_SK_PATH_SEGMENT_MASK;

const G_FOUR_QUARTER_CIRCLE_CONICS: [f32; 4] = [
    SCALAR_ROOT2_OVER2,
    SCALAR_ROOT2_OVER2,
    SCALAR_ROOT2_OVER2,
    SCALAR_ROOT2_OVER2,
];

const G_RRECT_VERBS_LINE_START: [SkPathVerb; 10] = [
    SkPathVerb::Move,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Close,
];

const G_RRECT_VERBS_CONIC_START: [SkPathVerb; 9] = [
    SkPathVerb::Move,
    SkPathVerb::Conic,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Line,
    SkPathVerb::Conic,
    SkPathVerb::Close,
];

const G_RRECT_SEG_MASK: u8 = K_LINE_SK_PATH_SEGMENT_MASK | K_CONIC_SK_PATH_SEGMENT_MASK;

const G_TRIANGLE_VERBS: [SkPathVerb; 4] = [
    SkPathVerb::Move,
    SkPathVerb::Line,
    SkPathVerb::Line,
    SkPathVerb::Close,
];

fn set_as_rect(r: &Rect, dir: SkPathDirection, index: usize) -> SkPathRaw {
    let mut iter = RectPointIterator::new(r, dir, index);
    let points = vec![iter.current(), iter.next(), iter.next(), iter.next()];
    SkPathRaw::new(
        points,
        G_RECT_VERBS.to_vec(),
        Vec::new(),
        *r,
        K_DEF_FILL_TYPE,
        sk_path_direction_to_convexity(dir),
        G_RECT_SEG_MASK,
    )
}

fn set_as_oval(r: &Rect, dir: SkPathDirection, index: usize) -> SkPathRaw {
    let mut oval_iter = OvalPointIterator::new(r, dir, index);
    let rect_index = index + if dir == SkPathDirection::Cw { 0 } else { 1 };
    let mut rect_iter = RectPointIterator::new(r, dir, rect_index);

    let mut storage = [Point::new(0.0, 0.0); 9];
    storage[0] = oval_iter.current();
    for i in 0..4 {
        storage[i * 2 + 1] = rect_iter.next();
        storage[i * 2 + 2] = oval_iter.next();
    }

    SkPathRaw::new(
        storage.to_vec(),
        G_OVAL_VERBS.to_vec(),
        G_FOUR_QUARTER_CIRCLE_CONICS.to_vec(),
        *r,
        K_DEF_FILL_TYPE,
        sk_path_direction_to_convexity(dir),
        G_OVAL_SEG_MASK,
    )
}

fn set_as_rrect(rrect: &RRect, dir: SkPathDirection, index: usize) -> SkPathRaw {
    // we start with a conic on odd indices when moving CW vs. even indices when moving CCW
    let starts_with_conic = (index & 1) == (if dir == SkPathDirection::Cw { 1 } else { 0 });
    // if we start with a conic, we end with a line, which we can skip (relying on close())
    let npoints = if starts_with_conic { 12 } else { 13 };
    let bounds = *rrect.get_bounds();

    let mut rrect_iter = RRectPointIterator::new(rrect, dir, index);
    // Corner iterator indices follow the collapsed radii model, adjusted such that the start pt
    // is "behind" the radii start pt.
    let rect_start_index = index / 2 + if dir == SkPathDirection::Cw { 0 } else { 1 };
    let mut rect_iter = RectPointIterator::new(&bounds, dir, rect_start_index);

    let mut storage = [Point::new(0.0, 0.0); 13];
    storage[0] = rrect_iter.current();
    if starts_with_conic {
        for i in 0..3 {
            // conic points
            storage[i * 3 + 1] = rect_iter.next();
            storage[i * 3 + 2] = rrect_iter.next();
            // line point
            storage[i * 3 + 3] = rrect_iter.next();
        }
        // last conic points
        storage[10] = rect_iter.next();
        storage[11] = rrect_iter.next();
        // the final line is accomplished by close()
    } else {
        for i in 0..4 {
            // line point
            storage[i * 3 + 1] = rrect_iter.next();
            // conic points
            storage[i * 3 + 2] = rect_iter.next();
            storage[i * 3 + 3] = rrect_iter.next();
        }
    }

    let points = storage[..npoints].to_vec();
    let verbs = if starts_with_conic {
        G_RRECT_VERBS_CONIC_START.to_vec()
    } else {
        G_RRECT_VERBS_LINE_START.to_vec()
    };
    SkPathRaw::new(
        points,
        verbs,
        G_FOUR_QUARTER_CIRCLE_CONICS.to_vec(),
        bounds,
        K_DEF_FILL_TYPE,
        sk_path_direction_to_convexity(dir),
        G_RRECT_SEG_MASK,
    )
}

fn tri_to_convexity(pts: &[Point]) -> SkPathConvexity {
    let u = pts[1] - pts[0];
    let v = pts[2] - pts[1];
    let cross = u.x * v.y - u.y * v.x;
    if cross > 0.0 {
        SkPathConvexity::ConvexCw
    } else if cross < 0.0 {
        SkPathConvexity::ConvexCcw
    } else {
        SkPathConvexity::ConvexDegenerate
    }
}

/// Namespace mirror of `SkPathRawShapes`. C++ construction (`SkPathRawShapes::Rect shape(r);`)
/// maps to `SkPathRawShapes::rect(&r, dir, index)`.
pub struct SkPathRawShapes;

impl SkPathRawShapes {
    pub fn rect(r: &Rect, dir: SkPathDirection, index: usize) -> SkPathRaw {
        set_as_rect(r, dir, index)
    }

    pub fn rect_default(r: &Rect) -> SkPathRaw {
        set_as_rect(r, SkPathDirection::Cw, 0)
    }

    pub fn oval(r: &Rect, dir: SkPathDirection, index: usize) -> SkPathRaw {
        set_as_oval(r, dir, index)
    }

    pub fn oval_default(r: &Rect) -> SkPathRaw {
        set_as_oval(r, SkPathDirection::Cw, 1)
    }

    pub fn rrect(rrect: &RRect, dir: SkPathDirection, index: usize) -> SkPathRaw {
        let bounds = *rrect.get_bounds();
        if rrect.is_rect() || rrect.is_empty() {
            // degenerate(rect) => radii points are collapsing
            set_as_rect(&bounds, dir, (index + 1) / 2)
        } else if rrect.is_oval() {
            // degenerate(oval) => line points are collapsing
            set_as_oval(&bounds, dir, index / 2)
        } else {
            set_as_rrect(rrect, dir, index)
        }
    }

    pub fn rrect_dirl(rrect: &RRect, dir: SkPathDirection) -> SkPathRaw {
        let index = if dir == SkPathDirection::Cw { 6 } else { 7 };
        Self::rrect(rrect, dir, index)
    }

    pub fn rrect_default(rrect: &RRect) -> SkPathRaw {
        Self::rrect(rrect, SkPathDirection::Cw, 6)
    }

    pub fn triangle(three_points: &[Point], bounds: Rect) -> SkPathRaw {
        debug_assert!(three_points.len() == 3);
        SkPathRaw::new(
            three_points.to_vec(),
            G_TRIANGLE_VERBS.to_vec(),
            Vec::new(),
            bounds,
            SkPathFillType::Winding,
            tri_to_convexity(three_points),
            K_LINE_SK_PATH_SEGMENT_MASK,
        )
    }
}
