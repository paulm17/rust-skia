use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use crate::safe32::{sat_add, sat_sub};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IPoint {
    pub x: i32,
    pub y: i32,
}

pub type IVector = IPoint;

const _: () = {
    assert!(core::mem::size_of::<IPoint>() == 2 * core::mem::size_of::<i32>());
    assert!(core::mem::offset_of!(IPoint, x) == 0);
    assert!(core::mem::offset_of!(IPoint, y) == core::mem::size_of::<i32>());
};

impl IPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn make(x: i32, y: i32) -> Self {
        Self::new(x, y)
    }

    pub const fn x(&self) -> i32 {
        self.x
    }

    pub const fn y(&self) -> i32 {
        self.y
    }

    pub const fn is_zero(&self) -> bool {
        (self.x | self.y) == 0
    }

    pub fn set(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub const fn equals(&self, x: i32, y: i32) -> bool {
        self.x == x && self.y == y
    }
}

impl Neg for IPoint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.x.wrapping_neg(), self.y.wrapping_neg())
    }
}

impl AddAssign for IPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.x = sat_add(self.x, rhs.x);
        self.y = sat_add(self.y, rhs.y);
    }
}

impl SubAssign for IPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.x = sat_sub(self.x, rhs.x);
        self.y = sat_sub(self.y, rhs.y);
    }
}

impl Add for IPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(sat_add(self.x, rhs.x), sat_add(self.y, rhs.y))
    }
}

impl Sub for IPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(sat_sub(self.x, rhs.x), sat_sub(self.y, rhs.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{SK_MAX_S32, SK_MIN_S32};

    #[test]
    fn dev_ipoint_layout_and_accessors() {
        assert_eq!(core::mem::size_of::<IPoint>(), 8);
        let point = IPoint::make(3, -4);
        assert_eq!(point.x(), 3);
        assert_eq!(point.y(), -4);
        assert!(IPoint::default().is_zero());
        assert!(point.equals(3, -4));
    }

    #[test]
    fn dev_ipoint_arithmetic_is_saturating() {
        assert_eq!(IPoint::new(2, 3) + IPoint::new(4, -1), IPoint::new(6, 2));
        assert_eq!(IPoint::new(2, 3) - IPoint::new(4, -1), IPoint::new(-2, 4));
        assert_eq!(
            IPoint::new(SK_MAX_S32, SK_MIN_S32) + IPoint::new(1, -1),
            IPoint::new(SK_MAX_S32, SK_MIN_S32)
        );
        assert_eq!(-IPoint::new(2, -3), IPoint::new(-2, 3));
    }
}
