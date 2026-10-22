use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::floating_point;
use crate::scalar::{SCALAR_NEARLY_ZERO, Scalar};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

pub type Vector3 = Point3;
pub type Color3f = Point3;

const _: () = {
    assert!(core::mem::size_of::<Point3>() == 3 * core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Point3, x) == 0);
    assert!(core::mem::offset_of!(Point3, y) == core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Point3, z) == 2 * core::mem::size_of::<Scalar>());
};

fn get_length_squared(x: Scalar, y: Scalar, z: Scalar) -> Scalar {
    x * x + y * y + z * z
}

fn is_length_nearly_zero(x: Scalar, y: Scalar, z: Scalar, length_squared: &mut Scalar) -> bool {
    *length_squared = get_length_squared(x, y, z);
    *length_squared <= SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO
}

impl Point3 {
    pub const fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    pub const fn make(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::new(x, y, z)
    }

    pub const fn x(&self) -> Scalar {
        self.x
    }

    pub const fn y(&self) -> Scalar {
        self.y
    }

    pub const fn z(&self) -> Scalar {
        self.z
    }

    pub fn set(&mut self, x: Scalar, y: Scalar, z: Scalar) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn length_xyz(x: Scalar, y: Scalar, z: Scalar) -> Scalar {
        let mag_sq = get_length_squared(x, y, z);
        if floating_point::is_finite(mag_sq) {
            mag_sq.sqrt()
        } else {
            let xx = x as f64;
            let yy = y as f64;
            let zz = z as f64;
            floating_point::double_to_float((xx * xx + yy * yy + zz * zz).sqrt())
        }
    }

    #[allow(non_snake_case)]
    pub fn Length(x: Scalar, y: Scalar, z: Scalar) -> Scalar {
        Self::length_xyz(x, y, z)
    }

    pub fn length(&self) -> Scalar {
        Self::length_xyz(self.x, self.y, self.z)
    }

    pub fn normalize(&mut self) -> bool {
        let mut mag_sq = 0.0;
        if is_length_nearly_zero(self.x, self.y, self.z, &mut mag_sq) {
            self.set(0.0, 0.0, 0.0);
            return false;
        }

        let inv_scale = if floating_point::is_finite(mag_sq) {
            mag_sq as f64
        } else {
            let xx = self.x as f64;
            let yy = self.y as f64;
            let zz = self.z as f64;
            xx * xx + yy * yy + zz * zz
        };

        let scale = 1.0 / inv_scale.sqrt();
        self.x = (self.x as f64 * scale) as Scalar;
        self.y = (self.y as f64 * scale) as Scalar;
        self.z = (self.z as f64 * scale) as Scalar;

        if !self.is_finite() {
            self.set(0.0, 0.0, 0.0);
            return false;
        }
        true
    }

    pub fn make_scale(&self, scale: Scalar) -> Self {
        Self::new(scale * self.x, scale * self.y, scale * self.z)
    }

    pub fn scale(&mut self, value: Scalar) {
        self.x *= value;
        self.y *= value;
        self.z *= value;
    }

    pub fn is_finite(&self) -> bool {
        floating_point::is_finite3(self.x, self.y, self.z)
    }

    pub fn dot_product(a: &Point3, b: &Point3) -> Scalar {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    pub fn dot(&self, vec: &Point3) -> Scalar {
        Self::dot_product(self, vec)
    }

    pub fn cross_product(a: &Point3, b: &Point3) -> Point3 {
        Point3::new(
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        )
    }

    pub fn cross(&self, vec: &Point3) -> Point3 {
        Self::cross_product(self, vec)
    }
}

impl Neg for Point3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Point3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Point3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl AddAssign for Point3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Point3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<Scalar> for Point3 {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Point3> for f32 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        Point3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_point3_length_normalize_and_cross() {
        assert_eq!(Point3::length_xyz(0.0, 0.0, 0.0), 0.0);
        assert!((Point3::length_xyz(0.3, 0.4, 0.5) - 0.70710678).abs() < 1.0e-6);
        assert_eq!(Point3::length_xyz(1.0e-37, 1.0e-37, 1.0e-37), 0.0);
        assert_eq!(Point3::length_xyz(3.4e38, 0.0, 0.0), 3.4e38);

        let mut point = Point3::new(1.0, 1.0, 1.0);
        assert!(point.normalize());
        assert!((point.length() - 1.0).abs() < 1.0e-6);
        assert_eq!(
            Point3::cross_product(&Point3::new(1.0, 0.0, 0.0), &Point3::new(0.0, 1.0, 0.0)),
            Point3::new(0.0, 0.0, 1.0)
        );
    }
}
