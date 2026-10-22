//! Rust adaptation of `src/core/SkBitmaskEnum.h`.
//!
//! C++ allows arithmetic on any opted-in fieldless enum. Rust enums cannot
//! soundly hold arbitrary bit combinations, so this port stores the underlying
//! representation in [`SkBitmask`] and converts back to an enum only when the
//! caller asks for it. The operator surface (`|`, `&`, `^`, `!`) is preserved
//! on the wrapper, while [`sk_bitmask_any`] mirrors `sknonstd::Any`.

use std::marker::PhantomData;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

pub trait SkBitmaskEnum: Copy + 'static {
    type Repr: Copy
        + Default
        + PartialEq
        + Eq
        + BitAnd<Output = Self::Repr>
        + BitOr<Output = Self::Repr>
        + BitXor<Output = Self::Repr>
        + Not<Output = Self::Repr>
        + std::fmt::Debug
        + 'static;

    fn to_repr(self) -> Self::Repr;
}

/// Holds arbitrary bit patterns from a [`SkBitmaskEnum`] without requiring the
/// combination to name a declared enum variant.
pub struct SkBitmask<E: SkBitmaskEnum> {
    bits: E::Repr,
    marker: PhantomData<E>,
}

impl<E: SkBitmaskEnum> Clone for SkBitmask<E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<E: SkBitmaskEnum> Copy for SkBitmask<E> {}
impl<E: SkBitmaskEnum> Default for SkBitmask<E> {
    fn default() -> Self {
        Self {
            bits: E::Repr::default(),
            marker: PhantomData,
        }
    }
}
impl<E: SkBitmaskEnum> std::fmt::Debug for SkBitmask<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkBitmask")
            .field("bits", &self.bits)
            .finish()
    }
}
impl<E: SkBitmaskEnum> PartialEq for SkBitmask<E> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}
impl<E: SkBitmaskEnum> Eq for SkBitmask<E> {}

impl<E: SkBitmaskEnum> SkBitmask<E> {
    #[inline]
    pub fn new(value: E) -> Self {
        Self {
            bits: value.to_repr(),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn from_repr(bits: E::Repr) -> Self {
        Self {
            bits,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn repr(self) -> E::Repr {
        self.bits
    }

    #[inline]
    pub fn any(self) -> bool {
        self.bits != E::Repr::default()
    }

    #[inline]
    pub fn contains(self, value: E) -> bool {
        self.bits & value.to_repr() == value.to_repr()
    }
}

impl<E: SkBitmaskEnum> From<E> for SkBitmask<E> {
    fn from(value: E) -> Self {
        Self::new(value)
    }
}

impl<E: SkBitmaskEnum> BitOr for SkBitmask<E> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self::from_repr(self.bits | rhs.bits)
    }
}
impl<E: SkBitmaskEnum> BitAnd for SkBitmask<E> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self::from_repr(self.bits & rhs.bits)
    }
}
impl<E: SkBitmaskEnum> BitXor for SkBitmask<E> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self::from_repr(self.bits ^ rhs.bits)
    }
}
impl<E: SkBitmaskEnum> Not for SkBitmask<E> {
    type Output = Self;
    fn not(self) -> Self {
        Self::from_repr(!self.bits)
    }
}
impl<E: SkBitmaskEnum> BitOrAssign for SkBitmask<E> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits = self.bits | rhs.bits;
    }
}
impl<E: SkBitmaskEnum> BitAndAssign for SkBitmask<E> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits = self.bits & rhs.bits;
    }
}
impl<E: SkBitmaskEnum> BitXorAssign for SkBitmask<E> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits = self.bits ^ rhs.bits;
    }
}

/// Port of `sknonstd::Any`.
pub trait SkBitmaskAny {
    fn bitmask_any(self) -> bool;
}
impl<E: SkBitmaskEnum> SkBitmaskAny for E {
    fn bitmask_any(self) -> bool {
        self.to_repr() != E::Repr::default()
    }
}
impl<E: SkBitmaskEnum> SkBitmaskAny for SkBitmask<E> {
    fn bitmask_any(self) -> bool {
        self.any()
    }
}

pub fn sk_bitmask_any<E: SkBitmaskAny>(value: E) -> bool {
    value.bitmask_any()
}

impl<E: SkBitmaskEnum> SkBitmask<E> {
    pub fn is_any(self) -> bool {
        self.any()
    }
}

/// Implements the opt-in trait for a fieldless enum.
#[macro_export]
macro_rules! impl_sk_bitmask_enum {
    ($ty:ty, $repr:ty) => {
        impl $crate::SkBitmaskEnum for $ty {
            type Repr = $repr;

            #[inline]
            fn to_repr(self) -> $repr {
                self as $repr
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(u8)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Flags {
        A = 1,
        B = 2,
    }
    impl_sk_bitmask_enum!(Flags, u8);

    #[test]
    fn wrapper_preserves_arbitrary_bits() {
        let ab = SkBitmask::new(Flags::A) | SkBitmask::new(Flags::B);
        assert_eq!(ab.repr(), 3);
        assert!(ab.any());
        assert!(ab.contains(Flags::A));
        assert_eq!((!SkBitmask::new(Flags::A)).repr(), !1u8);
        assert!(sk_bitmask_any(Flags::B));
    }
}
