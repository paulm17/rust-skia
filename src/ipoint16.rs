use crate::sk_to::sk_to_s16;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IPoint16 {
    pub x: i16,
    pub y: i16,
}

const _: () = {
    assert!(core::mem::size_of::<IPoint16>() == 4);
    assert!(core::mem::offset_of!(IPoint16, x) == 0);
    assert!(core::mem::offset_of!(IPoint16, y) == 2);
};

impl IPoint16 {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn make(x: i32, y: i32) -> Self {
        Self::new(sk_to_s16(x), sk_to_s16(y))
    }

    pub const fn x(&self) -> i16 {
        self.x
    }

    pub const fn y(&self) -> i16 {
        self.y
    }

    pub fn set(&mut self, x: i32, y: i32) {
        self.x = sk_to_s16(x);
        self.y = sk_to_s16(y);
    }
}
