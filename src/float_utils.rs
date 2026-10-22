//! Port of `src/utils/SkFloatUtils.h`.
//!
//! Only raw IEEE bit inspection is required; arithmetic is delegated to Rust's
//! float types.  `SkFloatingPoint<T, ULPS>` preserves Skia's biased
//! sign-and-magnitude ULP distance and its "large values are close to
//! infinity" behavior.

use std::fmt::Debug;
use std::ops::{Add, BitAnd, BitOr, Not, Sub};

pub trait SkFloatBits: Copy + Debug + PartialEq {
    type Bits: Copy
        + Debug
        + Eq
        + Ord
        + BitAnd<Output = Self::Bits>
        + BitOr<Output = Self::Bits>
        + Not<Output = Self::Bits>
        + Sub<Output = Self::Bits>
        + Add<Output = Self::Bits>
        + From<u8>
        + Into<u64>
        + 'static;

    const BIT_COUNT: usize;
    const FRACTION_BIT_COUNT: usize;
    const SIGN_BIT_MASK: Self::Bits;
    const FRACTION_BIT_MASK: Self::Bits;
    const EXPONENT_BIT_MASK: Self::Bits;

    fn to_bits(self) -> Self::Bits;
    fn from_bits(bits: Self::Bits) -> Self;
}

macro_rules! impl_float_bits {
    ($t:ty, $bits:ty, $count:expr, $fraction:expr, $sign:expr, $fraction_mask:expr, $exponent_mask:expr) => {
        impl SkFloatBits for $t {
            type Bits = $bits;
            const BIT_COUNT: usize = $count;
            const FRACTION_BIT_COUNT: usize = $fraction;
            const SIGN_BIT_MASK: $bits = $sign;
            const FRACTION_BIT_MASK: $bits = $fraction_mask;
            const EXPONENT_BIT_MASK: $bits = $exponent_mask;

            #[inline]
            fn to_bits(self) -> $bits {
                self.to_bits()
            }

            #[inline]
            fn from_bits(bits: $bits) -> Self {
                <$t>::from_bits(bits)
            }
        }
    };
}

impl_float_bits!(f32, u32, 32, 23, 1 << 31, 0x007f_ffff, 0x7f80_0000);
impl_float_bits!(
    f64,
    u64,
    64,
    52,
    1 << 63,
    0x000f_ffff_ffff_ffff,
    0x7ff0_0000_0000_0000
);

#[derive(Clone, Copy, Debug)]
pub struct SkFloatingPoint<T: SkFloatBits, const ULPS: usize> {
    value: T,
}

impl<T: SkFloatBits, const ULPS: usize> SkFloatingPoint<T, ULPS> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self { value }
    }

    #[inline]
    pub fn value(self) -> T {
        self.value
    }

    #[inline]
    pub fn exponent_bits(self) -> T::Bits {
        self.value.to_bits() & T::EXPONENT_BIT_MASK
    }

    #[inline]
    pub fn fraction_bits(self) -> T::Bits {
        self.value.to_bits() & T::FRACTION_BIT_MASK
    }

    #[inline]
    pub fn is_nan(self) -> bool {
        self.exponent_bits() == T::EXPONENT_BIT_MASK && self.fraction_bits() != T::Bits::from(0)
    }

    fn sign_and_magnitude_to_biased(bits: T::Bits) -> T::Bits {
        if (T::SIGN_BIT_MASK & bits) != T::Bits::from(0) {
            !bits + T::Bits::from(1)
        } else {
            T::SIGN_BIT_MASK | bits
        }
    }

    fn distance_between(bits1: T::Bits, bits2: T::Bits) -> T::Bits {
        let biased1 = Self::sign_and_magnitude_to_biased(bits1);
        let biased2 = Self::sign_and_magnitude_to_biased(bits2);
        if biased1 >= biased2 {
            biased1 - biased2
        } else {
            biased2 - biased1
        }
    }

    pub fn almost_equals(self, rhs: Self) -> bool {
        if self.is_nan() || rhs.is_nan() {
            return false;
        }
        let distance = Self::distance_between(self.value.to_bits(), rhs.value.to_bits());
        let max = usize::try_from(distance.into()).unwrap_or(usize::MAX);
        max <= ULPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ulp_distance_matches_ieee_edges() {
        type F = SkFloatingPoint<f32, 16>;
        assert!(F::new(1.0).almost_equals(F::new(1.0 + f32::EPSILON * 16.0)));
        assert!(!F::new(1.0).almost_equals(F::new(1.0 + f32::EPSILON * 18.0)));
        assert!(!F::new(f32::NAN).almost_equals(F::new(f32::NAN)));
        assert!(F::new(f32::INFINITY).almost_equals(F::new(f32::INFINITY)));
        assert!(F::new(0.0).almost_equals(F::new(-0.0)));
        assert!(F::new(f32::from_bits(1)).almost_equals(F::new(0.0)));
    }
}
