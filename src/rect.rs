use crate::floating_point;
use crate::ipoint::IPoint;
use crate::point::Point;
use crate::safe32::{pin_to_s32, sat_add, sat_sub, sk32_can_overflow_sub};
use crate::scalar::{
    SCALAR_NAN, Scalar, scalar_ceil_to_int, scalar_floor_to_int, scalar_round_to_int,
};
use crate::size::{ISize, Size};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

const _: () = {
    assert!(core::mem::size_of::<IRect>() == 4 * core::mem::size_of::<i32>());
    assert!(core::mem::offset_of!(IRect, left) == 0);
    assert!(core::mem::offset_of!(IRect, top) == core::mem::size_of::<i32>());
    assert!(core::mem::offset_of!(IRect, right) == 2 * core::mem::size_of::<i32>());
    assert!(core::mem::offset_of!(IRect, bottom) == 3 * core::mem::size_of::<i32>());
};

impl IRect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn make_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self::new(left, top, right, bottom)
    }

    pub const fn make_empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub const fn make_wh(w: i32, h: i32) -> Self {
        Self::new(0, 0, w, h)
    }

    pub const fn make_size(size: ISize) -> Self {
        Self::new(0, 0, size.width, size.height)
    }

    pub const fn make_pt_size(pt: IPoint, size: ISize) -> Self {
        Self::make_xywh(pt.x, pt.y, size.width, size.height)
    }

    pub const fn make_xywh(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self::new(x, y, sat_add(x, w), sat_add(y, h))
    }

    pub const fn left(&self) -> i32 {
        self.left
    }

    pub const fn top(&self) -> i32 {
        self.top
    }

    pub const fn right(&self) -> i32 {
        self.right
    }

    pub const fn bottom(&self) -> i32 {
        self.bottom
    }

    pub const fn x(&self) -> i32 {
        self.left
    }

    pub const fn y(&self) -> i32 {
        self.top
    }

    pub const fn top_left(&self) -> IPoint {
        IPoint::new(self.left, self.top)
    }

    pub const fn width(&self) -> i32 {
        sk32_can_overflow_sub(self.right, self.left)
    }

    pub const fn height(&self) -> i32 {
        sk32_can_overflow_sub(self.bottom, self.top)
    }

    pub const fn size(&self) -> ISize {
        ISize::new(self.width(), self.height())
    }

    pub const fn width64(&self) -> i64 {
        self.right as i64 - self.left as i64
    }

    pub const fn height64(&self) -> i64 {
        self.bottom as i64 - self.top as i64
    }

    pub const fn is_empty64(&self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }

    pub fn is_empty(&self) -> bool {
        let w = self.width64();
        let h = self.height64();
        if w <= 0 || h <= 0 {
            return true;
        }
        (w | h) > i32::MAX as i64
    }

    pub fn set_empty(&mut self) {
        *self = Self::make_empty();
    }

    pub fn set_ltrb(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        *self = Self::new(left, top, right, bottom);
    }

    pub fn set_xywh(&mut self, x: i32, y: i32, width: i32, height: i32) {
        *self = Self::make_xywh(x, y, width, height);
    }

    pub fn set_wh(&mut self, width: i32, height: i32) {
        *self = Self::make_wh(width, height);
    }

    pub fn set_size(&mut self, size: ISize) {
        *self = Self::make_size(size);
    }

    pub const fn make_offset(&self, dx: i32, dy: i32) -> Self {
        Self::new(
            sat_add(self.left, dx),
            sat_add(self.top, dy),
            sat_add(self.right, dx),
            sat_add(self.bottom, dy),
        )
    }

    pub const fn make_offset_vector(&self, offset: IPoint) -> Self {
        self.make_offset(offset.x, offset.y)
    }

    pub const fn make_inset(&self, dx: i32, dy: i32) -> Self {
        Self::new(
            sat_add(self.left, dx),
            sat_add(self.top, dy),
            sat_sub(self.right, dx),
            sat_sub(self.bottom, dy),
        )
    }

    pub const fn make_outset(&self, dx: i32, dy: i32) -> Self {
        Self::new(
            sat_sub(self.left, dx),
            sat_sub(self.top, dy),
            sat_add(self.right, dx),
            sat_add(self.bottom, dy),
        )
    }

    pub fn offset(&mut self, dx: i32, dy: i32) {
        self.left = sat_add(self.left, dx);
        self.top = sat_add(self.top, dy);
        self.right = sat_add(self.right, dx);
        self.bottom = sat_add(self.bottom, dy);
    }

    pub fn offset_point(&mut self, delta: IPoint) {
        self.offset(delta.x, delta.y);
    }

    pub fn offset_to(&mut self, new_x: i32, new_y: i32) {
        self.right = pin_to_s32(self.right as i64 + new_x as i64 - self.left as i64);
        self.bottom = pin_to_s32(self.bottom as i64 + new_y as i64 - self.top as i64);
        self.left = new_x;
        self.top = new_y;
    }

    pub fn inset(&mut self, dx: i32, dy: i32) {
        self.left = sat_add(self.left, dx);
        self.top = sat_add(self.top, dy);
        self.right = sat_sub(self.right, dx);
        self.bottom = sat_sub(self.bottom, dy);
    }

    pub fn outset(&mut self, dx: i32, dy: i32) {
        self.inset(dx.wrapping_neg(), dy.wrapping_neg());
    }

    pub fn adjust(&mut self, dl: i32, dt: i32, dr: i32, db: i32) {
        self.left = sat_add(self.left, dl);
        self.top = sat_add(self.top, dt);
        self.right = sat_add(self.right, dr);
        self.bottom = sat_add(self.bottom, db);
    }

    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub fn contains_irect(&self, r: &IRect) -> bool {
        !r.is_empty()
            && !self.is_empty()
            && self.left <= r.left
            && self.top <= r.top
            && self.right >= r.right
            && self.bottom >= r.bottom
    }

    pub fn contains_no_empty_check(&self, r: &IRect) -> bool {
        debug_assert!(self.left < self.right && self.top < self.bottom);
        debug_assert!(r.left < r.right && r.top < r.bottom);
        self.left <= r.left && self.top <= r.top && self.right >= r.right && self.bottom >= r.bottom
    }

    pub fn intersect(&mut self, r: &IRect) -> bool {
        self.intersect_irects(*self, *r)
    }

    pub fn intersect_irects(&mut self, a: IRect, b: IRect) -> bool {
        let tmp = IRect::new(
            a.left.max(b.left),
            a.top.max(b.top),
            a.right.min(b.right),
            a.bottom.min(b.bottom),
        );
        if tmp.is_empty() {
            return false;
        }
        *self = tmp;
        true
    }

    pub fn intersects(&self, r: &IRect) -> bool {
        Self::intersects_irects(self, r)
    }

    pub fn intersects_irects(a: &IRect, b: &IRect) -> bool {
        let mut tmp = Self::make_empty();
        tmp.intersect_irects(*a, *b)
    }

    pub fn join(&mut self, r: &IRect) {
        if r.left >= r.right || r.top >= r.bottom {
            return;
        }
        if self.left >= self.right || self.top >= self.bottom {
            *self = *r;
        } else {
            if r.left < self.left {
                self.left = r.left;
            }
            if r.top < self.top {
                self.top = r.top;
            }
            if r.right > self.right {
                self.right = r.right;
            }
            if r.bottom > self.bottom {
                self.bottom = r.bottom;
            }
        }
    }

    pub fn sort(&mut self) {
        if self.left > self.right {
            core::mem::swap(&mut self.left, &mut self.right);
        }
        if self.top > self.bottom {
            core::mem::swap(&mut self.top, &mut self.bottom);
        }
    }

    pub fn make_sorted(&self) -> Self {
        Self::new(
            self.left.min(self.right),
            self.top.min(self.bottom),
            self.left.max(self.right),
            self.top.max(self.bottom),
        )
    }

    pub fn as_int32s(&self) -> *const i32 {
        &self.left as *const i32
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: Scalar,
    pub top: Scalar,
    pub right: Scalar,
    pub bottom: Scalar,
}

const _: () = {
    assert!(core::mem::size_of::<Rect>() == 4 * core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Rect, left) == 0);
    assert!(core::mem::offset_of!(Rect, top) == core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Rect, right) == 2 * core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Rect, bottom) == 3 * core::mem::size_of::<Scalar>());
};

impl Rect {
    pub const fn new(left: Scalar, top: Scalar, right: Scalar, bottom: Scalar) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn make_ltrb(left: Scalar, top: Scalar, right: Scalar, bottom: Scalar) -> Self {
        Self::new(left, top, right, bottom)
    }

    pub const fn make_empty() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub const fn make_wh(w: Scalar, h: Scalar) -> Self {
        Self::new(0.0, 0.0, w, h)
    }

    pub const fn make_iwh(w: i32, h: i32) -> Self {
        Self::new(0.0, 0.0, w as Scalar, h as Scalar)
    }

    pub const fn make_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }

    pub const fn make_xywh(x: Scalar, y: Scalar, w: Scalar, h: Scalar) -> Self {
        Self::new(x, y, x + w, y + h)
    }

    pub const fn make_isize(size: ISize) -> Self {
        Self::make_iwh(size.width, size.height)
    }

    pub const fn make_irect(irect: IRect) -> Self {
        Self::new(
            irect.left as Scalar,
            irect.top as Scalar,
            irect.right as Scalar,
            irect.bottom as Scalar,
        )
    }

    pub fn is_empty(&self) -> bool {
        !(self.left < self.right && self.top < self.bottom)
    }

    pub fn is_sorted(&self) -> bool {
        self.left <= self.right && self.top <= self.bottom
    }

    pub fn is_finite(&self) -> bool {
        floating_point::is_finite4(self.left, self.top, self.right, self.bottom)
    }

    pub const fn x(&self) -> Scalar {
        self.left
    }

    pub const fn y(&self) -> Scalar {
        self.top
    }

    pub const fn left(&self) -> Scalar {
        self.left
    }

    pub const fn top(&self) -> Scalar {
        self.top
    }

    pub const fn right(&self) -> Scalar {
        self.right
    }

    pub const fn bottom(&self) -> Scalar {
        self.bottom
    }

    pub const fn width(&self) -> Scalar {
        self.right - self.left
    }

    pub const fn height(&self) -> Scalar {
        self.bottom - self.top
    }

    pub fn center_x(&self) -> Scalar {
        floating_point::float_midpoint(self.left, self.right)
    }

    pub fn center_y(&self) -> Scalar {
        floating_point::float_midpoint(self.top, self.bottom)
    }

    pub fn center(&self) -> Point {
        Point::new(self.center_x(), self.center_y())
    }

    pub fn tl(&self) -> Point {
        Point::new(self.left, self.top)
    }

    pub fn tr(&self) -> Point {
        Point::new(self.right, self.top)
    }

    pub fn bl(&self) -> Point {
        Point::new(self.left, self.bottom)
    }

    pub fn br(&self) -> Point {
        Point::new(self.right, self.bottom)
    }

    pub fn to_quad(&self, clockwise: bool) -> [Point; 4] {
        let mut storage = [Point::default(); 4];
        self.copy_to_quad(&mut storage, clockwise);
        storage
    }

    pub fn copy_to_quad(&self, storage: &mut [Point; 4], clockwise: bool) {
        if clockwise {
            *storage = [self.tl(), self.tr(), self.br(), self.bl()];
        } else {
            *storage = [self.tl(), self.bl(), self.br(), self.tr()];
        }
    }

    pub fn set_empty(&mut self) {
        *self = Self::make_empty();
    }

    pub fn set_irect(&mut self, irect: IRect) {
        *self = Self::make_irect(irect);
    }

    pub fn set_ltrb(&mut self, left: Scalar, top: Scalar, right: Scalar, bottom: Scalar) {
        *self = Self::new(left, top, right, bottom);
    }

    pub fn bounds(points: &[Point]) -> Option<Rect> {
        let Some(first) = points.first() else {
            return Some(Self::make_empty());
        };

        let mut left = first.x;
        let mut top = first.y;
        let mut right = first.x;
        let mut bottom = first.y;
        let mut nx = 0.0f32;
        let mut ny = 0.0f32;

        for point in points {
            left = point.x.min(left);
            top = point.y.min(top);
            right = point.x.max(right);
            bottom = point.y.max(bottom);

            nx *= point.x;
            ny *= point.y;
        }

        if nx == 0.0 && ny == 0.0 {
            Some(Self::new(left, top, right, bottom))
        } else {
            None
        }
    }

    pub fn bounds_or_empty(points: &[Point]) -> Rect {
        Self::bounds(points).unwrap_or_else(Self::make_empty)
    }

    pub fn set_bounds(&mut self, points: &[Point]) {
        let _ = self.set_bounds_check(points);
    }

    pub fn set_bounds_check(&mut self, points: &[Point]) -> bool {
        if let Some(bounds) = Self::bounds(points) {
            *self = bounds;
            true
        } else {
            *self = Self::make_empty();
            false
        }
    }

    pub fn set_bounds_no_check(&mut self, points: &[Point]) {
        if let Some(bounds) = Self::bounds(points) {
            *self = bounds;
        } else {
            self.set_ltrb(SCALAR_NAN, SCALAR_NAN, SCALAR_NAN, SCALAR_NAN);
        }
    }

    pub fn set_points(&mut self, p0: Point, p1: Point) {
        self.left = p0.x.min(p1.x);
        self.top = p0.y.min(p1.y);
        self.right = p0.x.max(p1.x);
        self.bottom = p0.y.max(p1.y);
    }

    pub fn set_xywh(&mut self, x: Scalar, y: Scalar, width: Scalar, height: Scalar) {
        *self = Self::make_xywh(x, y, width, height);
    }

    pub fn set_wh(&mut self, width: Scalar, height: Scalar) {
        *self = Self::make_wh(width, height);
    }

    pub fn set_iwh(&mut self, width: i32, height: i32) {
        *self = Self::make_iwh(width, height);
    }

    pub fn make_offset(&self, dx: Scalar, dy: Scalar) -> Self {
        Self::new(
            self.left + dx,
            self.top + dy,
            self.right + dx,
            self.bottom + dy,
        )
    }

    pub fn make_offset_vector(&self, offset: Point) -> Self {
        self.make_offset(offset.x, offset.y)
    }

    pub fn make_inset(&self, dx: Scalar, dy: Scalar) -> Self {
        Self::new(
            self.left + dx,
            self.top + dy,
            self.right - dx,
            self.bottom - dy,
        )
    }

    pub fn make_outset(&self, dx: Scalar, dy: Scalar) -> Self {
        Self::new(
            self.left - dx,
            self.top - dy,
            self.right + dx,
            self.bottom + dy,
        )
    }

    pub fn offset(&mut self, dx: Scalar, dy: Scalar) {
        self.left += dx;
        self.top += dy;
        self.right += dx;
        self.bottom += dy;
    }

    pub fn offset_point(&mut self, delta: Point) {
        self.offset(delta.x, delta.y);
    }

    pub fn offset_to(&mut self, new_x: Scalar, new_y: Scalar) {
        self.right += new_x - self.left;
        self.bottom += new_y - self.top;
        self.left = new_x;
        self.top = new_y;
    }

    pub fn inset(&mut self, dx: Scalar, dy: Scalar) {
        self.left += dx;
        self.top += dy;
        self.right -= dx;
        self.bottom -= dy;
    }

    pub fn outset(&mut self, dx: Scalar, dy: Scalar) {
        self.inset(-dx, -dy);
    }

    pub fn intersect(&mut self, r: &Rect) -> bool {
        self.intersect_rects(*r, *self)
    }

    pub fn intersect_rects(&mut self, a: Rect, b: Rect) -> bool {
        let l = cpp_max(a.left, b.left);
        let r = cpp_min(a.right, b.right);
        let t = cpp_max(a.top, b.top);
        let btm = cpp_min(a.bottom, b.bottom);
        if !(l < r && t < btm) {
            return false;
        }
        self.set_ltrb(l, t, r, btm);
        true
    }

    fn intersects_internal(a: &Rect, b: &Rect) -> bool {
        let l = cpp_max(a.left, b.left);
        let r = cpp_min(a.right, b.right);
        let t = cpp_max(a.top, b.top);
        let btm = cpp_min(a.bottom, b.bottom);
        l < r && t < btm
    }

    pub fn intersects(&self, r: &Rect) -> bool {
        Self::intersects_internal(self, r)
    }

    pub fn intersects_rects(a: &Rect, b: &Rect) -> bool {
        Self::intersects_internal(a, b)
    }

    pub fn join(&mut self, r: &Rect) {
        if r.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = *r;
        } else {
            self.left = self.left.min(r.left);
            self.top = self.top.min(r.top);
            self.right = self.right.max(r.right);
            self.bottom = self.bottom.max(r.bottom);
        }
    }

    pub fn join_non_empty_arg(&mut self, r: &Rect) {
        debug_assert!(!r.is_empty());
        if self.is_empty() {
            *self = *r;
        } else {
            self.join_possibly_empty_rect(r);
        }
    }

    pub fn join_possibly_empty_rect(&mut self, r: &Rect) {
        self.left = self.left.min(r.left);
        self.top = self.top.min(r.top);
        self.right = self.right.max(r.right);
        self.bottom = self.bottom.max(r.bottom);
    }

    pub fn contains(&self, x: Scalar, y: Scalar) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub fn contains_point(&self, point: &Point) -> bool {
        self.contains(point.x, point.y)
    }

    pub fn contains_rect(&self, r: &Rect) -> bool {
        !r.is_empty()
            && !self.is_empty()
            && self.left <= r.left
            && self.top <= r.top
            && self.right >= r.right
            && self.bottom >= r.bottom
    }

    pub fn contains_irect(&self, r: &IRect) -> bool {
        !r.is_empty()
            && !self.is_empty()
            && self.left <= r.left as Scalar
            && self.top <= r.top as Scalar
            && self.right >= r.right as Scalar
            && self.bottom >= r.bottom as Scalar
    }

    pub fn round(&self) -> IRect {
        IRect::new(
            scalar_round_to_int(self.left),
            scalar_round_to_int(self.top),
            scalar_round_to_int(self.right),
            scalar_round_to_int(self.bottom),
        )
    }

    pub fn round_into(&self, dst: &mut IRect) {
        *dst = self.round();
    }

    pub fn round_out(&self) -> IRect {
        IRect::new(
            scalar_floor_to_int(self.left),
            scalar_floor_to_int(self.top),
            scalar_ceil_to_int(self.right),
            scalar_ceil_to_int(self.bottom),
        )
    }

    pub fn round_out_into(&self, dst: &mut IRect) {
        *dst = self.round_out();
    }

    pub fn round_out_rect(&self) -> Rect {
        Self::new(
            self.left.floor(),
            self.top.floor(),
            self.right.ceil(),
            self.bottom.ceil(),
        )
    }

    pub fn round_out_rect_into(&self, dst: &mut Rect) {
        *dst = self.round_out_rect();
    }

    pub fn round_in(&self) -> IRect {
        IRect::new(
            scalar_ceil_to_int(self.left),
            scalar_ceil_to_int(self.top),
            scalar_floor_to_int(self.right),
            scalar_floor_to_int(self.bottom),
        )
    }

    pub fn round_in_into(&self, dst: &mut IRect) {
        *dst = self.round_in();
    }

    pub fn sort(&mut self) {
        if self.left > self.right {
            core::mem::swap(&mut self.left, &mut self.right);
        }
        if self.top > self.bottom {
            core::mem::swap(&mut self.top, &mut self.bottom);
        }
    }

    pub fn make_sorted(&self) -> Self {
        Self::new(
            self.left.min(self.right),
            self.top.min(self.bottom),
            self.left.max(self.right),
            self.top.max(self.bottom),
        )
    }

    pub fn as_scalars(&self) -> *const Scalar {
        &self.left as *const Scalar
    }
}

fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

impl From<IRect> for Rect {
    fn from(value: IRect) -> Self {
        Self::make_irect(value)
    }
}

impl From<Rect> for IRect {
    fn from(value: Rect) -> Self {
        IRect::new(
            value.left as i32,
            value.top as i32,
            value.right as i32,
            value.bottom as i32,
        )
    }
}

impl From<ISize> for Rect {
    fn from(value: ISize) -> Self {
        Self::make_isize(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_irect_empty_and_overflow() {
        assert!(IRect::make_empty().is_empty());
        assert!(!IRect::new(0, 0, 1, 1).is_empty());
        let wide = IRect::new(-i32::MAX + 1000, 0, i32::MAX - 1000, 100);
        assert!(wide.is_empty());
        assert_eq!(wide.width64(), 2 * (i32::MAX as i64 - 1000));
    }

    #[test]
    fn dev_rect_intersect_join_round() {
        let mut r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.intersect(&Rect::new(2.0, 2.0, 8.0, 8.0)));
        assert_eq!(r, Rect::new(2.0, 2.0, 8.0, 8.0));
        let rounded = Rect::new(0.2, 0.6, 3.7, 4.1);
        assert_eq!(rounded.round(), IRect::new(0, 1, 4, 4));
        assert_eq!(rounded.round_out(), IRect::new(0, 0, 4, 5));
        assert_eq!(rounded.round_in(), IRect::new(1, 1, 3, 4));
    }
}
