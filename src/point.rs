use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::floating_point::{self, is_finite2};
use crate::ipoint::IPoint;
use crate::scalar::{SCALAR_1, Scalar};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: Scalar,
    pub y: Scalar,
}

pub type Vector = Point;

const _: () = {
    assert!(core::mem::size_of::<Point>() == 2 * core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Point, x) == 0);
    assert!(core::mem::offset_of!(Point, y) == core::mem::size_of::<Scalar>());
};

impl Point {
    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    pub const fn make(x: Scalar, y: Scalar) -> Self {
        Self::new(x, y)
    }

    pub const fn x(&self) -> Scalar {
        self.x
    }

    pub const fn y(&self) -> Scalar {
        self.y
    }

    pub const fn is_zero(&self) -> bool {
        (self.x == 0.0) & (self.y == 0.0)
    }

    pub fn set(&mut self, x: Scalar, y: Scalar) {
        self.x = x;
        self.y = y;
    }

    pub fn iset(&mut self, x: i32, y: i32) {
        self.x = x as Scalar;
        self.y = y as Scalar;
    }

    pub fn iset_ipoint(&mut self, point: &IPoint) {
        self.iset(point.x, point.y);
    }

    pub fn set_abs(&mut self, point: &Point) {
        self.x = point.x.abs();
        self.y = point.y.abs();
    }

    pub fn offset_all(points: &mut [Point], offset: &Vector) {
        Self::offset_all_by(points, offset.x, offset.y);
    }

    pub fn offset_all_by(points: &mut [Point], dx: Scalar, dy: Scalar) {
        for point in points {
            point.offset(dx, dy);
        }
    }

    pub fn offset(&mut self, dx: Scalar, dy: Scalar) {
        self.x += dx;
        self.y += dy;
    }

    pub fn length(&self) -> Scalar {
        Self::length_xy(self.x, self.y)
    }

    pub fn distance_to_origin(&self) -> Scalar {
        self.length()
    }

    pub fn normalize(&mut self) -> bool {
        self.set_length(SCALAR_1)
    }

    pub fn set_normalize(&mut self, x: Scalar, y: Scalar) -> bool {
        self.set_length_xy(x, y, SCALAR_1)
    }

    pub fn normalize_returning_length(&mut self) -> Scalar {
        let (x, y) = (self.x, self.y);
        set_point_length(self, x, y, SCALAR_1).unwrap_or(0.0)
    }

    pub fn set_length(&mut self, length: Scalar) -> bool {
        let (x, y) = (self.x, self.y);
        self.set_length_xy(x, y, length)
    }

    pub fn set_length_xy(&mut self, x: Scalar, y: Scalar, length: Scalar) -> bool {
        set_point_length(self, x, y, length).is_some()
    }

    pub fn scale(&self, scale: Scalar, dst: &mut Point) {
        dst.set(self.x * scale, self.y * scale);
    }

    pub fn scale_in_place(&mut self, scale: Scalar) {
        self.x *= scale;
        self.y *= scale;
    }

    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
    }

    pub fn is_finite(&self) -> bool {
        is_finite2(self.x, self.y)
    }

    pub const fn equals(&self, x: Scalar, y: Scalar) -> bool {
        self.x == x && self.y == y
    }

    pub fn length_xy(dx: Scalar, dy: Scalar) -> Scalar {
        let mag2 = dx * dx + dy * dy;
        if floating_point::is_finite(mag2) {
            mag2.sqrt()
        } else {
            let xx = dx as f64;
            let yy = dy as f64;
            floating_point::double_to_float((xx * xx + yy * yy).sqrt())
        }
    }

    pub fn distance(a: &Point, b: &Point) -> Scalar {
        Self::length_xy(a.x - b.x, a.y - b.y)
    }

    pub fn dot_product(a: &Point, b: &Point) -> Scalar {
        a.x * b.x + a.y * b.y
    }

    pub fn cross_product(a: &Point, b: &Point) -> Scalar {
        a.x * b.y - a.y * b.x
    }

    pub fn cross(&self, vec: &Point) -> Scalar {
        Self::cross_product(self, vec)
    }

    pub fn dot(&self, vec: &Point) -> Scalar {
        Self::dot_product(self, vec)
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Scalar> for Point {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<Scalar> for Point {
    fn mul_assign(&mut self, rhs: Scalar) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

fn set_point_length(point: &mut Point, x: Scalar, y: Scalar, length: Scalar) -> Option<Scalar> {
    let xx = x as f64;
    let yy = y as f64;
    let dmag = (xx * xx + yy * yy).sqrt();
    let dscale = floating_point::ieee_double_divide(length as f64, dmag);
    let nx = (xx * dscale) as Scalar;
    let ny = (yy * dscale) as Scalar;

    if !is_finite2(nx, ny) || (nx == 0.0 && ny == 0.0) {
        point.set(0.0, 0.0);
        return None;
    }

    point.set(nx, ny);
    Some(floating_point::double_to_float(dmag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_point_accessors_and_conversions() {
        let mut point = Point::make(3.0, -4.0);
        assert_eq!(point.x(), 3.0);
        assert_eq!(point.y(), -4.0);
        assert!(!point.is_zero());
        point.iset(5, -6);
        assert_eq!(point, Point::new(5.0, -6.0));
        point.iset_ipoint(&IPoint::new(-7, 8));
        assert_eq!(point, Point::new(-7.0, 8.0));
        point.set_abs(&Point::new(-2.5, 3.5));
        assert_eq!(point, Point::new(2.5, 3.5));
    }

    #[test]
    fn dev_point_arithmetic_and_products() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(3.0, 4.0);
        assert_eq!(a + b, Point::new(4.0, 6.0));
        assert_eq!(b - a, Point::new(2.0, 2.0));
        assert_eq!(-a, Point::new(-1.0, -2.0));
        assert_eq!(a * 2.0, Point::new(2.0, 4.0));
        assert_eq!(Point::dot_product(&a, &b), 11.0);
        assert_eq!(Point::cross_product(&a, &b), -2.0);
        assert_eq!(Point::distance(&a, &b), 8.0f32.sqrt());
    }

    #[test]
    fn dev_point_scale_offset_and_normalize() {
        let mut point = Point::new(2.0, 3.0);
        let mut scaled = Point::default();
        point.scale(2.0, &mut scaled);
        assert_eq!(scaled, Point::new(4.0, 6.0));
        point.scale_in_place(0.5);
        assert_eq!(point, Point::new(1.0, 1.5));

        let mut points = [Point::new(0.0, 0.0), Point::new(1.0, 2.0)];
        Point::offset_all_by(&mut points, 1.0, -1.0);
        assert_eq!(points, [Point::new(1.0, -1.0), Point::new(2.0, 1.0)]);
        Point::offset_all(&mut points, &Point::new(-1.0, 1.0));
        assert_eq!(points, [Point::new(0.0, 0.0), Point::new(1.0, 2.0)]);

        let mut normalized = Point::new(3.0, 4.0);
        assert_eq!(normalized.normalize_returning_length(), 5.0);
        assert!(normalized.is_finite());
        assert!((normalized.length() - 1.0).abs() < 1.0e-6);
    }
}
