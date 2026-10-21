use crate::floating_point;
use crate::point::Point;
use crate::scalar::Scalar;

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

    pub const fn make_empty() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn is_empty(&self) -> bool {
        !(self.left < self.right && self.top < self.bottom)
    }

    pub fn is_finite(&self) -> bool {
        floating_point::is_finite4(self.left, self.top, self.right, self.bottom)
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

    pub fn as_scalars(&self) -> *const Scalar {
        &self.left as *const Scalar
    }
}
