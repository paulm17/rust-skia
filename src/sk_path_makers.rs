//! Port of `src/core/SkPathMakers.h`. The templated `SkPath_PointIterator<N>` is expressed as
//! three concrete iterators (`RectPointIterator`, `OvalPointIterator`, `RRectPointIterator`).

use crate::point::Point;
use crate::rect::Rect;
use crate::rrect::{Corner, RRect};
use crate::sk_path_types::SkPathDirection;

pub struct RectPointIterator {
    f_pts: [Point; 4],
    f_current: usize,
    f_advance: usize,
}

impl RectPointIterator {
    pub fn new(rect: &Rect, dir: SkPathDirection, start_index: usize) -> Self {
        RectPointIterator {
            f_pts: [
                Point::new(rect.left, rect.top),
                Point::new(rect.right, rect.top),
                Point::new(rect.right, rect.bottom),
                Point::new(rect.left, rect.bottom),
            ],
            f_current: start_index % 4,
            f_advance: if dir == SkPathDirection::Cw { 1 } else { 3 },
        }
    }

    #[inline]
    pub fn current(&self) -> Point {
        debug_assert!(self.f_current < 4);
        self.f_pts[self.f_current]
    }

    #[inline]
    pub fn next(&mut self) -> Point {
        self.f_current = (self.f_current + self.f_advance) % 4;
        self.current()
    }
}

pub struct OvalPointIterator {
    f_pts: [Point; 4],
    f_current: usize,
    f_advance: usize,
}

impl OvalPointIterator {
    pub fn new(oval: &Rect, dir: SkPathDirection, start_index: usize) -> Self {
        let cx = oval.center_x();
        let cy = oval.center_y();
        OvalPointIterator {
            f_pts: [
                Point::new(cx, oval.top),
                Point::new(oval.right, cy),
                Point::new(cx, oval.bottom),
                Point::new(oval.left, cy),
            ],
            f_current: start_index % 4,
            f_advance: if dir == SkPathDirection::Cw { 1 } else { 3 },
        }
    }

    #[inline]
    pub fn current(&self) -> Point {
        debug_assert!(self.f_current < 4);
        self.f_pts[self.f_current]
    }

    #[inline]
    pub fn next(&mut self) -> Point {
        self.f_current = (self.f_current + self.f_advance) % 4;
        self.current()
    }
}

pub struct RRectPointIterator {
    f_pts: [Point; 8],
    f_current: usize,
    f_advance: usize,
}

impl RRectPointIterator {
    pub fn new(rrect: &RRect, dir: SkPathDirection, start_index: usize) -> Self {
        let bounds = rrect.get_bounds();
        let l = bounds.left;
        let t = bounds.top;
        let r = bounds.right;
        let b = bounds.bottom;

        let ul = rrect.radii_at(Corner::UpperLeft);
        let ur = rrect.radii_at(Corner::UpperRight);
        let lr = rrect.radii_at(Corner::LowerRight);
        let ll = rrect.radii_at(Corner::LowerLeft);

        RRectPointIterator {
            f_pts: [
                Point::new(l + ul.x, t),
                Point::new(r - ur.x, t),
                Point::new(r, t + ur.y),
                Point::new(r, b - lr.y),
                Point::new(r - lr.x, b),
                Point::new(l + ll.x, b),
                Point::new(l, b - ll.y),
                Point::new(l, t + ul.y),
            ],
            f_current: start_index % 8,
            f_advance: if dir == SkPathDirection::Cw { 1 } else { 7 },
        }
    }

    #[inline]
    pub fn current(&self) -> Point {
        debug_assert!(self.f_current < 8);
        self.f_pts[self.f_current]
    }

    #[inline]
    pub fn next(&mut self) -> Point {
        self.f_current = (self.f_current + self.f_advance) % 8;
        self.current()
    }
}
