use crate::floating_point;
use crate::scalar::{Scalar, int_to_scalar};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ISize {
    pub width: i32,
    pub height: i32,
}

const _: () = {
    assert!(core::mem::size_of::<ISize>() == 2 * core::mem::size_of::<i32>());
    assert!(core::mem::offset_of!(ISize, width) == 0);
    assert!(core::mem::offset_of!(ISize, height) == core::mem::size_of::<i32>());
};

impl ISize {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub const fn make(width: i32, height: i32) -> Self {
        Self::new(width, height)
    }

    pub const fn make_empty() -> Self {
        Self::new(0, 0)
    }

    pub fn set(&mut self, width: i32, height: i32) {
        *self = Self::new(width, height);
    }

    pub const fn is_zero(&self) -> bool {
        0 == self.width && 0 == self.height
    }

    pub const fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn set_empty(&mut self) {
        self.width = 0;
        self.height = 0;
    }

    pub const fn width(&self) -> i32 {
        self.width
    }

    pub const fn height(&self) -> i32 {
        self.height
    }

    pub const fn area(&self) -> i64 {
        (self.width as i64).wrapping_mul(self.height as i64)
    }

    pub const fn equals(&self, w: i32, h: i32) -> bool {
        self.width == w && self.height == h
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: Scalar,
    pub height: Scalar,
}

const _: () = {
    assert!(core::mem::size_of::<Size>() == 2 * core::mem::size_of::<Scalar>());
    assert!(core::mem::offset_of!(Size, width) == 0);
    assert!(core::mem::offset_of!(Size, height) == core::mem::size_of::<Scalar>());
};

impl Size {
    pub const fn new(width: Scalar, height: Scalar) -> Self {
        Self { width, height }
    }

    pub const fn make(width: Scalar, height: Scalar) -> Self {
        Self::new(width, height)
    }

    pub fn make_isize(size: ISize) -> Self {
        Self::new(int_to_scalar(size.width), int_to_scalar(size.height))
    }

    pub const fn make_empty() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn set(&mut self, width: Scalar, height: Scalar) {
        *self = Self::new(width, height);
    }

    pub const fn is_zero(&self) -> bool {
        0.0 == self.width && 0.0 == self.height
    }

    pub const fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn set_empty(&mut self) {
        *self = Self::make_empty();
    }

    pub const fn width(&self) -> Scalar {
        self.width
    }

    pub const fn height(&self) -> Scalar {
        self.height
    }

    pub const fn equals(&self, w: Scalar, h: Scalar) -> bool {
        self.width == w && self.height == h
    }

    pub fn to_round(&self) -> ISize {
        ISize::new(
            floating_point::round_to_int(self.width),
            floating_point::round_to_int(self.height),
        )
    }

    pub fn to_ceil(&self) -> ISize {
        ISize::new(
            floating_point::ceil_to_int(self.width),
            floating_point::ceil_to_int(self.height),
        )
    }

    pub fn to_floor(&self) -> ISize {
        ISize::new(
            floating_point::floor_to_int(self.width),
            floating_point::floor_to_int(self.height),
        )
    }
}

impl From<ISize> for Size {
    fn from(value: ISize) -> Self {
        Self::make_isize(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_isize_and_size_empty_semantics() {
        assert!(ISize::make_empty().is_empty());
        assert!(ISize::new(5, -5).is_empty());
        assert!(ISize::new(5, 0).is_empty());
        assert!(!ISize::new(3, 5).is_empty());
        assert_eq!(ISize::new(5, 3).area(), 15);

        let mut size = Size::make(5.0, 3.0);
        assert!(!size.is_empty());
        size.set(5.0, 0.0);
        assert!(size.is_empty());
        size.set(5.0, 3.0);
        assert_eq!(size.to_round(), ISize::new(5, 3));
    }
}
