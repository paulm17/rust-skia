#![allow(clippy::eq_op)]

use core::mem::size_of;

use crate::floating_point;
use crate::ipoint::IPoint;
use crate::point::Point;
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_nearly_zero_within, scalar_sqrt};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Left = -1,
    On = 0,
    Right = 1,
}

impl Side {
    pub const fn from_sign(sign: i32) -> Self {
        if sign < 0 {
            Self::Left
        } else if sign > 0 {
            Self::Right
        } else {
            Self::On
        }
    }
}

pub struct PointPriv;

impl PointPriv {
    pub fn are_finite(points: &[Point]) -> bool {
        let Some(first) = points.first() else {
            return true;
        };
        let mut prod = first.x - first.x;
        prod *= first.y;
        for point in &points[1..] {
            prod *= point.x;
            prod *= point.y;
        }
        prod == prod
    }

    pub fn as_scalars(point: &Point) -> *const Scalar {
        &point.x as *const Scalar
    }

    pub fn can_normalize(dx: Scalar, dy: Scalar) -> bool {
        floating_point::is_finite2(dx, dy) && (dx != 0.0 || dy != 0.0)
    }

    pub fn distance_to_line_between_sqd(
        pt: &Point,
        a: &Point,
        b: &Point,
        side: Option<&mut Side>,
    ) -> Scalar {
        let u = *b - *a;
        let v = *pt - *a;

        let u_length_sqd = Self::length_sqd(&u);
        let det = u.cross(&v);
        if let Some(side) = side {
            *side = Side::from_sign(floating_point::float_sgn(det));
        }
        let mut temp = floating_point::ieee_float_divide(det, u_length_sqd);
        temp *= det;
        if !floating_point::is_finite(temp) {
            return Self::length_sqd(&v);
        }
        temp
    }

    pub fn distance_to_line_between(
        pt: &Point,
        a: &Point,
        b: &Point,
        side: Option<&mut Side>,
    ) -> Scalar {
        scalar_sqrt(Self::distance_to_line_between_sqd(pt, a, b, side))
    }

    pub fn distance_to_line_segment_between_sqd(pt: &Point, a: &Point, b: &Point) -> Scalar {
        let u = *b - *a;
        let v = *pt - *a;

        let u_length_sqd = Self::length_sqd(&u);
        let u_dot_v = Point::dot_product(&u, &v);

        if u_dot_v <= 0.0 {
            Self::length_sqd(&v)
        } else if u_dot_v > u_length_sqd {
            Self::distance_to_sqd(b, pt)
        } else {
            let det = u.cross(&v);
            let mut temp = floating_point::ieee_float_divide(det, u_length_sqd);
            temp *= det;
            if !floating_point::is_finite(temp) {
                Self::length_sqd(&v)
            } else {
                temp
            }
        }
    }

    pub fn distance_to_line_segment_between(pt: &Point, a: &Point, b: &Point) -> Scalar {
        scalar_sqrt(Self::distance_to_line_segment_between_sqd(pt, a, b))
    }

    pub fn distance_to_sqd(pt: &Point, a: &Point) -> Scalar {
        let dx = pt.x - a.x;
        let dy = pt.y - a.y;
        dx * dx + dy * dy
    }

    pub fn equals_within_tolerance(p1: &Point, p2: &Point) -> bool {
        !Self::can_normalize(p1.x - p2.x, p1.y - p2.y)
    }

    pub fn equals_within_tolerance_within(pt: &Point, p: &Point, tol: Scalar) -> bool {
        scalar_nearly_zero_within(pt.x - p.x, tol) && scalar_nearly_zero_within(pt.y - p.y, tol)
    }

    pub fn length_sqd(pt: &Point) -> Scalar {
        Point::dot_product(pt, pt)
    }

    pub fn negate(pt: &mut IPoint) {
        pt.x = pt.x.wrapping_neg();
        pt.y = pt.y.wrapping_neg();
    }

    pub fn rotate_ccw(src: &Point, dst: &mut Point) {
        let tmp = src.x;
        dst.x = src.y;
        dst.y = -tmp;
    }

    pub fn rotate_ccw_in_place(pt: &mut Point) {
        let tmp = pt.x;
        pt.x = pt.y;
        pt.y = -tmp;
    }

    pub fn rotate_cw(src: &Point, dst: &mut Point) {
        let tmp = src.x;
        dst.x = -src.y;
        dst.y = tmp;
    }

    pub fn rotate_cw_in_place(pt: &mut Point) {
        let tmp = pt.x;
        pt.x = -pt.y;
        pt.y = tmp;
    }

    pub fn set_length_fast(point: &mut Point, length: Scalar) -> bool {
        point.set_length(length)
    }

    pub fn make_orthog(vec: &Point, side: Side) -> Point {
        debug_assert!(side == Side::Left || side == Side::Right);
        if side == Side::Right {
            Point::new(-vec.y, vec.x)
        } else {
            Point::new(vec.y, -vec.x)
        }
    }

    /// # Safety
    ///
    /// `v` must be non-null and valid for writing four `Point` values at byte
    /// offsets `0`, `stride`, `2 * stride`, and `3 * stride`. The allocation
    /// must be appropriately aligned for `Point` at each offset.
    pub unsafe fn set_rect_fan(
        v: *mut Point,
        l: Scalar,
        t: Scalar,
        r: Scalar,
        b: Scalar,
        stride: usize,
    ) {
        debug_assert!(stride >= size_of::<Point>());
        unsafe {
            set_point_at(v, 0, stride, l, t);
            set_point_at(v, 1, stride, l, b);
            set_point_at(v, 2, stride, r, b);
            set_point_at(v, 3, stride, r, t);
        }
    }

    /// # Safety
    ///
    /// `v` must be non-null and valid for writing four `Point` values at byte
    /// offsets `0`, `stride`, `2 * stride`, and `3 * stride`. The allocation
    /// must be appropriately aligned for `Point` at each offset.
    pub unsafe fn set_rect_tri_strip(
        v: *mut Point,
        l: Scalar,
        t: Scalar,
        r: Scalar,
        b: Scalar,
        stride: usize,
    ) {
        debug_assert!(stride >= size_of::<Point>());
        unsafe {
            set_point_at(v, 0, stride, l, t);
            set_point_at(v, 1, stride, l, b);
            set_point_at(v, 2, stride, r, t);
            set_point_at(v, 3, stride, r, b);
        }
    }

    /// # Safety
    ///
    /// `v` must be non-null and valid for writing four `Point` values at byte
    /// offsets `0`, `stride`, `2 * stride`, and `3 * stride`. The allocation
    /// must be appropriately aligned for `Point` at each offset.
    pub unsafe fn set_rect_tri_strip_rect(v: *mut Point, rect: &Rect, stride: usize) {
        unsafe {
            Self::set_rect_tri_strip(v, rect.left, rect.top, rect.right, rect.bottom, stride);
        }
    }
}

unsafe fn set_point_at(v: *mut Point, index: usize, stride: usize, x: Scalar, y: Scalar) {
    unsafe {
        let point = (v as *mut u8).add(index * stride).cast::<Point>();
        (*point).set(x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_point_priv_finite_and_distances() {
        assert!(PointPriv::are_finite(&[]));
        assert!(PointPriv::are_finite(&[Point::new(1.0, 2.0)]));
        assert!(!PointPriv::are_finite(&[
            Point::new(1.0, 2.0),
            Point::new(f32::NAN, 0.0)
        ]));
        assert!(!PointPriv::are_finite(&[Point::new(f32::INFINITY, 0.0)]));

        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);
        let pt = Point::new(5.0, 3.0);
        let mut side = Side::On;
        assert_eq!(
            PointPriv::distance_to_line_between_sqd(&pt, &a, &b, Some(&mut side)),
            9.0
        );
        assert_eq!(side, Side::Right);

        assert_eq!(
            PointPriv::distance_to_line_segment_between_sqd(&pt, &a, &b),
            9.0
        );
        assert_eq!(
            PointPriv::distance_to_line_segment_between_sqd(&Point::new(-1.0, 0.0), &a, &b),
            1.0
        );
        assert_eq!(
            PointPriv::distance_to_line_segment_between_sqd(&Point::new(11.0, 0.0), &a, &b),
            1.0
        );
        assert_eq!(PointPriv::distance_to_sqd(&pt, &a), 34.0);
        assert_eq!(PointPriv::length_sqd(&Point::new(3.0, 4.0)), 25.0);
    }

    #[test]
    fn dev_point_priv_rotation_and_rect_helpers() {
        let mut dst = Point::default();
        PointPriv::rotate_ccw(&Point::new(1.0, 0.0), &mut dst);
        assert_eq!(dst, Point::new(0.0, -1.0));
        PointPriv::rotate_cw(&Point::new(1.0, 0.0), &mut dst);
        assert_eq!(dst, Point::new(0.0, 1.0));

        assert_eq!(
            PointPriv::make_orthog(&Point::new(1.0, 0.0), Side::Left),
            Point::new(0.0, -1.0)
        );
        assert_eq!(
            PointPriv::make_orthog(&Point::new(1.0, 0.0), Side::Right),
            Point::new(0.0, 1.0)
        );

        let mut fan = [Point::default(); 4];
        unsafe {
            PointPriv::set_rect_fan(fan.as_mut_ptr(), 1.0, 2.0, 3.0, 4.0, size_of::<Point>());
        }
        assert_eq!(
            fan,
            [
                Point::new(1.0, 2.0),
                Point::new(1.0, 4.0),
                Point::new(3.0, 4.0),
                Point::new(3.0, 2.0),
            ]
        );

        let mut strip = [Point::default(); 4];
        unsafe {
            PointPriv::set_rect_tri_strip_rect(
                strip.as_mut_ptr(),
                &Rect::new(1.0, 2.0, 3.0, 4.0),
                size_of::<Point>(),
            );
        }
        assert_eq!(
            strip,
            [
                Point::new(1.0, 2.0),
                Point::new(1.0, 4.0),
                Point::new(3.0, 2.0),
                Point::new(3.0, 4.0),
            ]
        );
    }
}
