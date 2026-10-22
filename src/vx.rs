//! Scalar reference implementation of Skia's `skvx` vector abstraction.
//!
//! The C++ header selects NEON or SSE/AVX intrinsics for several operations.  This
//! module deliberately implements the portable, lane-at-a-time reference semantics
//! first.  Every public operation keeps the C++ lane layout, wrapping/rounding
//! behavior, and mask representation (all bits set for true).

use crate::half::{sk_float_to_half_bits, sk_half_to_float};
use std::fmt::Debug;
use std::iter::FromIterator;
use std::ops::{
    Add, BitAnd, BitOr, BitXor, Div, Index, IndexMut, Mul, Neg, Not, Rem, Shl, Shr, Sub,
};

/// Element types that can live in a `Vx`.
///
/// Floating point vectors use signed integer masks, matching NEON/C++:
/// `f32` uses `i32`, `f64` uses `i64`; all other element types use themselves.
pub trait VxElem: Copy + Default + PartialEq + PartialOrd + Debug + 'static {
    type Mask: VxElem;

    fn mask_from_bool(value: bool) -> Self::Mask;
    fn mask_is_true(value: Self::Mask) -> bool;
    fn is_true(value: Self) -> bool;
}

macro_rules! vx_elem_integer {
    ($($t:ty),* $(,)?) => {$(
        impl VxElem for $t {
            type Mask = $t;
            #[inline] fn mask_from_bool(value: bool) -> Self { if value { !0 } else { 0 } }
            #[inline] fn mask_is_true(value: Self) -> bool { value != 0 }
            #[inline] fn is_true(value: Self) -> bool { value != 0 }
        }
    )*};
}

vx_elem_integer!(
    i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize
);

impl VxElem for f32 {
    type Mask = i32;
    #[inline]
    fn mask_from_bool(value: bool) -> i32 {
        if value { -1 } else { 0 }
    }
    #[inline]
    fn mask_is_true(value: i32) -> bool {
        value != 0
    }
    #[inline]
    fn is_true(value: f32) -> bool {
        value != 0.0
    }
}

impl VxElem for f64 {
    type Mask = i64;
    #[inline]
    fn mask_from_bool(value: bool) -> i64 {
        if value { -1 } else { 0 }
    }
    #[inline]
    fn mask_is_true(value: i64) -> bool {
        value != 0
    }
    #[inline]
    fn is_true(value: f64) -> bool {
        value != 0.0
    }
}

/// Fixed-width vector.  `lanes` is public to mirror C++'s transparent layout.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vx<T: VxElem, const N: usize> {
    pub lanes: [T; N],
}

impl<T: VxElem, const N: usize> Default for Vx<T, N> {
    fn default() -> Self {
        Self {
            lanes: std::array::from_fn(|_| T::default()),
        }
    }
}

impl<T: VxElem, const N: usize> Vx<T, N> {
    #[inline]
    pub const fn new(lanes: [T; N]) -> Self {
        Self { lanes }
    }

    #[inline]
    pub fn splat(value: T) -> Self {
        Self { lanes: [value; N] }
    }

    #[inline]
    pub fn zero() -> Self {
        Self::default()
    }

    #[inline]
    pub fn load(slice: &[T]) -> Self {
        assert!(slice.len() >= N);
        Self {
            lanes: std::array::from_fn(|i| slice[i]),
        }
    }

    #[inline]
    pub fn store(&self, slice: &mut [T]) {
        assert!(slice.len() >= N);
        slice[..N].copy_from_slice(&self.lanes);
    }

    #[inline]
    pub fn map<D: VxElem>(&self, mut f: impl FnMut(T) -> D) -> Vx<D, N> {
        Vx {
            lanes: std::array::from_fn(|i| f(self.lanes[i])),
        }
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.lanes.iter().any(|&v| T::is_true(v))
    }

    #[inline]
    pub fn all(&self) -> bool {
        self.lanes.iter().all(|&v| T::is_true(v))
    }

    #[inline]
    pub fn eq_mask(&self, rhs: &Self) -> Vx<T::Mask, N> {
        Vx {
            lanes: std::array::from_fn(|i| T::mask_from_bool(self.lanes[i] == rhs.lanes[i])),
        }
    }

    #[inline]
    pub fn ne_mask(&self, rhs: &Self) -> Vx<T::Mask, N> {
        Vx {
            lanes: std::array::from_fn(|i| T::mask_from_bool(self.lanes[i] != rhs.lanes[i])),
        }
    }

    #[inline]
    pub fn lt(&self, rhs: &Self) -> Vx<T::Mask, N> {
        Vx {
            lanes: std::array::from_fn(|i| T::mask_from_bool(self.lanes[i] < rhs.lanes[i])),
        }
    }

    #[inline]
    pub fn le(&self, rhs: &Self) -> Vx<T::Mask, N> {
        Vx {
            lanes: std::array::from_fn(|i| T::mask_from_bool(self.lanes[i] <= rhs.lanes[i])),
        }
    }

    #[inline]
    pub fn gt(&self, rhs: &Self) -> Vx<T::Mask, N> {
        Vx {
            lanes: std::array::from_fn(|i| T::mask_from_bool(self.lanes[i] > rhs.lanes[i])),
        }
    }

    #[inline]
    pub fn ge(&self, rhs: &Self) -> Vx<T::Mask, N> {
        Vx {
            lanes: std::array::from_fn(|i| T::mask_from_bool(self.lanes[i] >= rhs.lanes[i])),
        }
    }

    #[inline]
    pub fn eq_scalar(&self, rhs: T) -> Vx<T::Mask, N> {
        self.eq_mask(&Self::splat(rhs))
    }

    #[inline]
    pub fn lt_scalar(&self, rhs: T) -> Vx<T::Mask, N> {
        self.lt(&Self::splat(rhs))
    }

    #[inline]
    pub fn le_scalar(&self, rhs: T) -> Vx<T::Mask, N> {
        self.le(&Self::splat(rhs))
    }

    #[inline]
    pub fn gt_scalar(&self, rhs: T) -> Vx<T::Mask, N> {
        self.gt(&Self::splat(rhs))
    }

    #[inline]
    pub fn ge_scalar(&self, rhs: T) -> Vx<T::Mask, N> {
        self.ge(&Self::splat(rhs))
    }

    #[inline]
    pub fn min_lane(&self) -> T {
        let mut v = self.lanes[0];
        for &x in &self.lanes[1..] {
            if x < v {
                v = x;
            }
        }
        v
    }

    #[inline]
    pub fn max_lane(&self) -> T {
        let mut v = self.lanes[0];
        for &x in &self.lanes[1..] {
            if v < x {
                v = x;
            }
        }
        v
    }

    #[inline]
    pub fn shuffle<const M: usize>(&self, indices: [usize; M]) -> Vx<T, M> {
        Vx {
            lanes: std::array::from_fn(|i| self.lanes[indices[i]]),
        }
    }

    #[inline]
    pub fn low<const M: usize>(&self) -> Vx<T, M> {
        assert!(M <= N);
        Vx {
            lanes: std::array::from_fn(|i| self.lanes[i]),
        }
    }

    #[inline]
    pub fn high<const M: usize>(&self) -> Vx<T, M> {
        assert!(M <= N);
        Vx {
            lanes: std::array::from_fn(|i| self.lanes[N - M + i]),
        }
    }
}

impl<T: VxElem, const N: usize> From<[T; N]> for Vx<T, N> {
    #[inline]
    fn from(lanes: [T; N]) -> Self {
        Self { lanes }
    }
}

impl<T: VxElem, const N: usize> From<T> for Vx<T, N> {
    #[inline]
    fn from(value: T) -> Self {
        Self::splat(value)
    }
}

impl<T: VxElem, const N: usize> FromIterator<T> for Vx<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let v: std::vec::Vec<T> = iter.into_iter().take(N).collect();
        assert_eq!(v.len(), N, "not enough lanes for Vx");
        Self {
            lanes: v.try_into().expect("lane count"),
        }
    }
}

impl<T: VxElem, const N: usize> Index<usize> for Vx<T, N> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.lanes[index]
    }
}

impl<T: VxElem, const N: usize> IndexMut<usize> for Vx<T, N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.lanes[index]
    }
}

macro_rules! impl_vx_binary {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: VxElem + $trait<Output = T>, const N: usize> $trait for Vx<T, N> {
            type Output = Self;
            #[inline]
            fn $method(self, rhs: Self) -> Self {
                Self { lanes: std::array::from_fn(|i| self.lanes[i] $op rhs.lanes[i]) }
            }
        }

        impl<T: VxElem + $trait<Output = T>, const N: usize> $trait<T> for Vx<T, N> {
            type Output = Self;
            #[inline]
            fn $method(self, rhs: T) -> Self {
                Self { lanes: std::array::from_fn(|i| self.lanes[i] $op rhs) }
            }
        }
    };
}

impl_vx_binary!(Add, add, +);
impl_vx_binary!(Sub, sub, -);
impl_vx_binary!(Mul, mul, *);
impl_vx_binary!(Div, div, /);
impl_vx_binary!(Rem, rem, %);
impl_vx_binary!(BitAnd, bitand, &);
impl_vx_binary!(BitOr, bitor, |);
impl_vx_binary!(BitXor, bitxor, ^);

impl<T: VxElem + Not<Output = T>, const N: usize> Not for Vx<T, N> {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Self {
            lanes: std::array::from_fn(|i| !self.lanes[i]),
        }
    }
}

impl<T: VxElem + Neg<Output = T>, const N: usize> Neg for Vx<T, N> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            lanes: std::array::from_fn(|i| -self.lanes[i]),
        }
    }
}

impl<T: VxElem + Shl<u32, Output = T>, const N: usize> Shl<u32> for Vx<T, N> {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u32) -> Self {
        Self {
            lanes: std::array::from_fn(|i| self.lanes[i] << rhs),
        }
    }
}

impl<T: VxElem + Shr<u32, Output = T>, const N: usize> Shr<u32> for Vx<T, N> {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u32) -> Self {
        Self {
            lanes: std::array::from_fn(|i| self.lanes[i] >> rhs),
        }
    }
}

impl<T: VxElem, const N: usize> Vx<T, N> {
    #[inline]
    pub fn wrapping_add(self, rhs: Self) -> Self
    where
        T: std::ops::Add<Output = T>,
    {
        self + rhs
    }
}

/// Join two 2-lane vectors into a 4-lane vector.
#[inline]
pub fn join<T: VxElem>(lo: Vx<T, 2>, hi: Vx<T, 2>) -> Vx<T, 4> {
    Vx::new([lo[0], lo[1], hi[0], hi[1]])
}

/// Join two 4-lane vectors into an 8-lane vector.
#[inline]
pub fn join4<T: VxElem>(lo: Vx<T, 4>, hi: Vx<T, 4>) -> Vx<T, 8> {
    Vx::new([lo[0], lo[1], lo[2], lo[3], hi[0], hi[1], hi[2], hi[3]])
}

/// Free-function form of `Vx::shuffle`.
#[inline]
pub fn shuffle<T: VxElem, const N: usize, const M: usize>(
    value: Vx<T, N>,
    indices: [usize; M],
) -> Vx<T, M> {
    value.shuffle(indices)
}

#[inline]
pub fn any<T: VxElem, const N: usize>(value: Vx<T, N>) -> bool {
    value.any()
}

#[inline]
pub fn all<T: VxElem, const N: usize>(value: Vx<T, N>) -> bool {
    value.all()
}

/// Lane-wise `y < x ? y : x`, matching `skvx::min`.
#[inline]
pub fn vx_min<T: VxElem, const N: usize>(x: Vx<T, N>, y: Vx<T, N>) -> Vx<T, N> {
    Vx {
        lanes: std::array::from_fn(|i| if y[i] < x[i] { y[i] } else { x[i] }),
    }
}

/// Lane-wise `y < x ? x : y`, matching `skvx::max`.
#[inline]
pub fn vx_max<T: VxElem, const N: usize>(x: Vx<T, N>, y: Vx<T, N>) -> Vx<T, N> {
    Vx {
        lanes: std::array::from_fn(|i| if x[i] < y[i] { y[i] } else { x[i] }),
    }
}

#[inline]
pub fn vx_min_scalar<T: VxElem, const N: usize>(x: Vx<T, N>, y: T) -> Vx<T, N> {
    vx_min(x, Vx::splat(y))
}

#[inline]
pub fn vx_max_scalar<T: VxElem, const N: usize>(x: Vx<T, N>, y: T) -> Vx<T, N> {
    vx_max(x, Vx::splat(y))
}

#[inline]
pub fn pin<T: VxElem, const N: usize>(
    x: Vx<T, N>,
    lo: impl Into<Vx<T, N>>,
    hi: impl Into<Vx<T, N>>,
) -> Vx<T, N> {
    vx_max(vx_min(x, hi.into()), lo.into())
}

#[inline]
pub fn pin_vectors<T: VxElem, const N: usize>(x: Vx<T, N>, lo: Vx<T, N>, hi: Vx<T, N>) -> Vx<T, N> {
    vx_max(vx_min(x, hi), lo)
}

#[inline]
pub fn naive_if_then_else<T: VxElem, const N: usize>(
    cond: Vx<T::Mask, N>,
    then_value: Vx<T, N>,
    else_value: Vx<T, N>,
) -> Vx<T, N> {
    Vx {
        lanes: std::array::from_fn(|i| {
            if T::mask_is_true(cond[i]) {
                then_value[i]
            } else {
                else_value[i]
            }
        }),
    }
}

#[inline]
pub fn if_then_else<T: VxElem, const N: usize>(
    cond: Vx<T::Mask, N>,
    then_value: Vx<T, N>,
    else_value: Vx<T, N>,
) -> Vx<T, N> {
    naive_if_then_else(cond, then_value, else_value)
}

pub trait VxCast<D> {
    fn cast_lane(self) -> D;
}

macro_rules! vx_cast_pairs {
    ($($src:ty => $($dst:ty),+;)*) => {$(
        $(impl VxCast<$dst> for $src {
            #[inline] fn cast_lane(self) -> $dst { self as $dst }
        })+
    )*};
}

vx_cast_pairs! {
    i8 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    u8 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    i16 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    u16 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    i32 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    u32 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    i64 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    u64 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    f32 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
    f64 => i8,u8,i16,u16,i32,u32,i64,u64,f32,f64;
}

impl<T: VxElem, const N: usize> Vx<T, N> {
    #[inline]
    pub fn cast<D>(&self) -> Vx<D, N>
    where
        D: VxElem,
        T: VxCast<D>,
    {
        Vx {
            lanes: std::array::from_fn(|i| self.lanes[i].cast_lane()),
        }
    }
}

pub trait VxFloat:
    VxElem
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + PartialOrd
{
    fn vx_floor(self) -> Self;
    fn vx_ceil(self) -> Self;
    fn vx_trunc(self) -> Self;
    fn vx_round(self) -> Self;
    fn vx_sqrt(self) -> Self;
    fn vx_abs(self) -> Self;
    fn vx_fma(self, b: Self, c: Self) -> Self;
    fn vx_is_nan(self) -> bool;
    fn vx_is_finite(self) -> bool;
    fn vx_to_f64(self) -> f64;
    fn vx_from_f64(value: f64) -> Self;
}

impl VxFloat for f32 {
    #[inline]
    fn vx_floor(self) -> Self {
        self.floor()
    }
    #[inline]
    fn vx_ceil(self) -> Self {
        self.ceil()
    }
    #[inline]
    fn vx_trunc(self) -> Self {
        self.trunc()
    }
    #[inline]
    fn vx_round(self) -> Self {
        self.round()
    }
    #[inline]
    fn vx_sqrt(self) -> Self {
        self.sqrt()
    }
    #[inline]
    fn vx_abs(self) -> Self {
        self.abs()
    }
    #[inline]
    fn vx_fma(self, b: Self, c: Self) -> Self {
        self.mul_add(b, c)
    }
    #[inline]
    fn vx_is_nan(self) -> bool {
        self.is_nan()
    }
    #[inline]
    fn vx_is_finite(self) -> bool {
        self.is_finite()
    }
    #[inline]
    fn vx_to_f64(self) -> f64 {
        self as f64
    }
    #[inline]
    fn vx_from_f64(value: f64) -> Self {
        value as f32
    }
}

impl VxFloat for f64 {
    #[inline]
    fn vx_floor(self) -> Self {
        self.floor()
    }
    #[inline]
    fn vx_ceil(self) -> Self {
        self.ceil()
    }
    #[inline]
    fn vx_trunc(self) -> Self {
        self.trunc()
    }
    #[inline]
    fn vx_round(self) -> Self {
        self.round()
    }
    #[inline]
    fn vx_sqrt(self) -> Self {
        self.sqrt()
    }
    #[inline]
    fn vx_abs(self) -> Self {
        self.abs()
    }
    #[inline]
    fn vx_fma(self, b: Self, c: Self) -> Self {
        self.mul_add(b, c)
    }
    #[inline]
    fn vx_is_nan(self) -> bool {
        self.is_nan()
    }
    #[inline]
    fn vx_is_finite(self) -> bool {
        self.is_finite()
    }
    #[inline]
    fn vx_to_f64(self) -> f64 {
        self
    }
    #[inline]
    fn vx_from_f64(value: f64) -> Self {
        value
    }
}

macro_rules! vx_float_unary {
    ($name:ident, $method:ident) => {
        #[inline]
        pub fn $name<T: VxFloat, const N: usize>(x: Vx<T, N>) -> Vx<T, N> {
            x.map(|v| v.$method())
        }
    };
}

vx_float_unary!(floor, vx_floor);
vx_float_unary!(ceil, vx_ceil);
vx_float_unary!(trunc, vx_trunc);
vx_float_unary!(round, vx_round);
vx_float_unary!(sqrt, vx_sqrt);
vx_float_unary!(abs, vx_abs);

#[inline]
pub fn fract<T: VxFloat, const N: usize>(x: Vx<T, N>) -> Vx<T, N> {
    x - floor(x)
}

#[inline]
pub fn fma<T: VxFloat, const N: usize>(x: Vx<T, N>, y: Vx<T, N>, z: Vx<T, N>) -> Vx<T, N> {
    Vx {
        lanes: std::array::from_fn(|i| x[i].vx_fma(y[i], z[i])),
    }
}

#[inline]
pub fn lrint<T: VxFloat, const N: usize>(x: Vx<T, N>) -> Vx<i32, N> {
    Vx {
        lanes: std::array::from_fn(|i| x[i].vx_to_f64().round_ties_even() as i32),
    }
}

pub fn to_half<T: VxFloat, const N: usize>(x: Vx<T, N>) -> Vx<u16, N> {
    Vx {
        lanes: std::array::from_fn(|i| {
            let v = x[i].vx_to_f64() as f32;
            debug_assert!(!v.is_nan());
            sk_float_to_half_bits(v)
        }),
    }
}

pub fn from_half<T: VxFloat, const N: usize>(x: Vx<u16, N>) -> Vx<T, N> {
    Vx {
        lanes: std::array::from_fn(|i| T::vx_from_f64(sk_half_to_float(x[i]) as f64)),
    }
}

/// `(x + 127) / 255`, packing a u16 lane down to u8.
#[inline]
pub fn div255<const N: usize>(x: Vx<u16, N>) -> Vx<u8, N> {
    Vx {
        lanes: std::array::from_fn(|i| ((x[i] as u32 + 127) / 255) as u8),
    }
}

/// `(X*Y + X) / 256`, matching Skia's historical approximation.
#[inline]
pub fn approx_scale<const N: usize>(x: Vx<u8, N>, y: Vx<u8, N>) -> Vx<u8, N> {
    Vx {
        lanes: std::array::from_fn(|i| {
            let x = x[i] as u32;
            let y = y[i] as u32;
            ((x * y + x) / 256) as u8
        }),
    }
}

pub trait VxMull {
    type Output;
    fn mull_lanes(self, rhs: Self) -> Self::Output;
}

impl<const N: usize> VxMull for Vx<u8, N> {
    type Output = Vx<u16, N>;
    #[inline]
    fn mull_lanes(self, rhs: Self) -> Self::Output {
        Vx {
            lanes: std::array::from_fn(|i| self[i] as u16 * rhs[i] as u16),
        }
    }
}

impl<const N: usize> VxMull for Vx<u16, N> {
    type Output = Vx<u32, N>;
    #[inline]
    fn mull_lanes(self, rhs: Self) -> Self::Output {
        Vx {
            lanes: std::array::from_fn(|i| self[i] as u32 * rhs[i] as u32),
        }
    }
}

#[inline]
pub fn mull<V: VxMull>(x: V, y: V) -> V::Output {
    x.mull_lanes(y)
}

#[inline]
pub fn mull_u8<const N: usize>(x: Vx<u8, N>, y: Vx<u8, N>) -> Vx<u16, N> {
    mull(x, y)
}

#[inline]
pub fn mull_u16<const N: usize>(x: Vx<u16, N>, y: Vx<u16, N>) -> Vx<u32, N> {
    mull(x, y)
}

#[inline]
pub fn mulhi<const N: usize>(x: Vx<u16, N>, y: Vx<u16, N>) -> Vx<u16, N> {
    Vx {
        lanes: std::array::from_fn(|i| (mull_u16(x, y).lanes[i] >> 16) as u16),
    }
}

pub trait VxSaturatedAdd: Sized {
    fn saturated_add_lanes(self, rhs: Self) -> Self;
}

impl<const N: usize> VxSaturatedAdd for Vx<u8, N> {
    #[inline]
    fn saturated_add_lanes(self, rhs: Self) -> Self {
        Vx {
            lanes: std::array::from_fn(|i| self[i].saturating_add(rhs[i])),
        }
    }
}

impl<const N: usize> VxSaturatedAdd for Vx<u16, N> {
    #[inline]
    fn saturated_add_lanes(self, rhs: Self) -> Self {
        Vx {
            lanes: std::array::from_fn(|i| self[i].saturating_add(rhs[i])),
        }
    }
}

#[inline]
pub fn saturated_add<V: VxSaturatedAdd>(x: V, y: V) -> V {
    x.saturated_add_lanes(y)
}

#[inline]
pub fn saturated_add_u8<const N: usize>(x: Vx<u8, N>, y: Vx<u8, N>) -> Vx<u8, N> {
    saturated_add(x, y)
}

#[inline]
pub fn saturated_add_u16<const N: usize>(x: Vx<u16, N>, y: Vx<u16, N>) -> Vx<u16, N> {
    saturated_add(x, y)
}

#[inline]
pub fn dot<T: VxElem + Add<Output = T> + Mul<Output = T>, const N: usize>(
    a: Vx<T, N>,
    b: Vx<T, N>,
) -> T {
    let mut sum = a[0] * b[0];
    for i in 1..N {
        sum = sum + a[i] * b[i];
    }
    sum
}

#[inline]
pub fn reduce_add<T: VxElem + Add<Output = T>, const N: usize>(x: Vx<T, N>) -> T {
    let mut sum = x[0];
    for i in 1..N {
        sum = sum + x[i];
    }
    sum
}

#[inline]
pub fn cross<T: VxElem + Sub<Output = T> + Mul<Output = T>>(a: Vx<T, 2>, b: Vx<T, 2>) -> T {
    a[0] * b[1] - a[1] * b[0]
}

#[inline]
pub fn length<T: VxFloat, const N: usize>(v: Vx<T, N>) -> T {
    dot(v, v).vx_sqrt()
}

#[inline]
pub fn normalize<T: VxFloat, const N: usize>(v: Vx<T, N>) -> Vx<T, N> {
    v / length(v)
}

#[inline]
pub fn isfinite<T: VxFloat, const N: usize>(v: Vx<T, N>) -> bool {
    v.lanes.iter().all(|&x| x.vx_is_finite())
}

/// De-interleaving load of two vectors from `v`.
#[inline]
pub fn strided_load2<T: VxElem, const N: usize>(v: &[T]) -> (Vx<T, N>, Vx<T, N>) {
    assert!(v.len() >= N * 2);
    (
        Vx {
            lanes: std::array::from_fn(|i| v[i * 2]),
        },
        Vx {
            lanes: std::array::from_fn(|i| v[i * 2 + 1]),
        },
    )
}

/// De-interleaving load of four vectors from `v`.
#[inline]
pub fn strided_load4<T: VxElem, const N: usize>(
    v: &[T],
) -> (Vx<T, N>, Vx<T, N>, Vx<T, N>, Vx<T, N>) {
    assert!(v.len() >= N * 4);
    (
        Vx {
            lanes: std::array::from_fn(|i| v[i * 4]),
        },
        Vx {
            lanes: std::array::from_fn(|i| v[i * 4 + 1]),
        },
        Vx {
            lanes: std::array::from_fn(|i| v[i * 4 + 2]),
        },
        Vx {
            lanes: std::array::from_fn(|i| v[i * 4 + 3]),
        },
    )
}

#[inline]
pub fn strided_load2_into<T: VxElem, const N: usize>(v: &[T], a: &mut Vx<T, N>, b: &mut Vx<T, N>) {
    (*a, *b) = strided_load2(v);
}

#[inline]
pub fn strided_load4_into<T: VxElem, const N: usize>(
    v: &[T],
    a: &mut Vx<T, N>,
    b: &mut Vx<T, N>,
    c: &mut Vx<T, N>,
    d: &mut Vx<T, N>,
) {
    (*a, *b, *c, *d) = strided_load4(v);
}

/// Scaled integer divider from `skvx::ScaledDividerU32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaledDividerU32 {
    divisor_factor: u32,
    half: u32,
}

impl ScaledDividerU32 {
    pub fn new(divisor: u32) -> Self {
        assert!(divisor > 1);
        let divisor_factor = ((1.0 / divisor as f64) * (1u64 << 32) as f64).round() as u32;
        Self {
            divisor_factor,
            half: (divisor + 1) >> 1,
        }
    }

    #[inline]
    pub fn divide<V: Into<Vx<u32, 4>>>(&self, numerator: V) -> Vx<u32, 4> {
        let numerator = numerator.into();
        Vx {
            lanes: std::array::from_fn(|i| {
                ((numerator[i] as u64 * self.divisor_factor as u64) >> 32) as u32
            }),
        }
    }

    #[inline]
    pub fn half(&self) -> u32 {
        self.half
    }

    #[inline]
    pub fn divisor_factor(&self) -> u32 {
        self.divisor_factor
    }

    /// Convenience scalar form (equivalent to dividing a splatted vector lane 0).
    #[inline]
    pub fn divide_scalar(&self, numerator: u32) -> u32 {
        self.divide(numerator).lanes[0]
    }
}

impl<T: VxElem> Vx<T, 2> {
    #[inline]
    pub fn x(&self) -> T {
        self.lanes[0]
    }

    #[inline]
    pub fn y(&self) -> T {
        self.lanes[1]
    }

    #[inline]
    pub fn set_x(&mut self, value: T) {
        self.lanes[0] = value;
    }

    #[inline]
    pub fn set_y(&mut self, value: T) {
        self.lanes[1] = value;
    }

    #[inline]
    pub fn yx(&self) -> Self {
        Self::new([self.lanes[1], self.lanes[0]])
    }

    #[inline]
    pub fn xyxy(&self) -> Vx<T, 4> {
        Vx::new([self.lanes[0], self.lanes[1], self.lanes[0], self.lanes[1]])
    }

    #[inline]
    pub fn lo(&self) -> Vx<T, 1> {
        Vx::new([self.lanes[0]])
    }

    #[inline]
    pub fn hi(&self) -> Vx<T, 1> {
        Vx::new([self.lanes[1]])
    }
}

impl<T: VxElem> Vx<T, 1> {
    #[inline]
    pub fn val(&self) -> T {
        self.lanes[0]
    }

    #[inline]
    pub fn set_val(&mut self, value: T) {
        self.lanes[0] = value;
    }
}

impl<T: VxElem> Vx<T, 4> {
    #[inline]
    pub fn x(&self) -> T {
        self.lanes[0]
    }

    #[inline]
    pub fn y(&self) -> T {
        self.lanes[1]
    }

    #[inline]
    pub fn z(&self) -> T {
        self.lanes[2]
    }

    #[inline]
    pub fn w(&self) -> T {
        self.lanes[3]
    }

    #[inline]
    pub fn set_x(&mut self, value: T) {
        self.lanes[0] = value;
    }

    #[inline]
    pub fn set_y(&mut self, value: T) {
        self.lanes[1] = value;
    }

    #[inline]
    pub fn set_z(&mut self, value: T) {
        self.lanes[2] = value;
    }

    #[inline]
    pub fn set_w(&mut self, value: T) {
        self.lanes[3] = value;
    }

    #[inline]
    pub fn xy(&self) -> Vx<T, 2> {
        Vx::new([self.lanes[0], self.lanes[1]])
    }

    #[inline]
    pub fn zw(&self) -> Vx<T, 2> {
        Vx::new([self.lanes[2], self.lanes[3]])
    }

    #[inline]
    pub fn lo(&self) -> Vx<T, 2> {
        self.xy()
    }

    #[inline]
    pub fn hi(&self) -> Vx<T, 2> {
        self.zw()
    }

    #[inline]
    pub fn yxwz(&self) -> Self {
        Self::new([self.lanes[1], self.lanes[0], self.lanes[3], self.lanes[2]])
    }

    #[inline]
    pub fn zwxy(&self) -> Self {
        Self::new([self.lanes[2], self.lanes[3], self.lanes[0], self.lanes[1]])
    }

    #[inline]
    pub fn set_xy(&mut self, xy: Vx<T, 2>) {
        self.lanes[0] = xy.lanes[0];
        self.lanes[1] = xy.lanes[1];
    }

    #[inline]
    pub fn set_zw(&mut self, zw: Vx<T, 2>) {
        self.lanes[2] = zw.lanes[0];
        self.lanes[3] = zw.lanes[1];
    }
}

impl<T: VxElem> Vx<T, 8> {
    #[inline]
    pub fn lo(&self) -> Vx<T, 4> {
        Vx::new([self.lanes[0], self.lanes[1], self.lanes[2], self.lanes[3]])
    }

    #[inline]
    pub fn hi(&self) -> Vx<T, 4> {
        Vx::new([self.lanes[4], self.lanes[5], self.lanes[6], self.lanes[7]])
    }
}

pub type Float2 = Vx<f32, 2>;
pub type Float4 = Vx<f32, 4>;
pub type Float8 = Vx<f32, 8>;
pub type Double2 = Vx<f64, 2>;
pub type Double4 = Vx<f64, 4>;
pub type Double8 = Vx<f64, 8>;
pub type Byte2 = Vx<u8, 2>;
pub type Byte4 = Vx<u8, 4>;
pub type Byte8 = Vx<u8, 8>;
pub type Byte16 = Vx<u8, 16>;
pub type Int2 = Vx<i32, 2>;
pub type Int4 = Vx<i32, 4>;
pub type Int8 = Vx<i32, 8>;
pub type Ushort2 = Vx<u16, 2>;
pub type Ushort4 = Vx<u16, 4>;
pub type Ushort8 = Vx<u16, 8>;
pub type Uint2 = Vx<u32, 2>;
pub type Uint4 = Vx<u32, 4>;
pub type Uint8 = Vx<u32, 8>;
pub type Long2 = Vx<i64, 2>;
pub type Long4 = Vx<i64, 4>;
pub type Long8 = Vx<i64, 8>;
pub type Half2 = Vx<u16, 2>;
pub type Half4 = Vx<u16, 4>;
pub type Half8 = Vx<u16, 8>;

/// Lower-case spellings matching the C++ `skvx` names.
#[allow(non_camel_case_types)]
pub mod skvx {
    pub use super::{
        Byte2, Byte4, Byte8, Byte16, Double2, Double4, Double8, Float2, Float4, Float8, Half2,
        Half4, Half8, Int2, Int4, Int8, Long2, Long4, Long8, ScaledDividerU32, Uint2, Uint4, Uint8,
        Ushort2, Ushort4, Ushort8, Vx, VxElem, VxFloat, any, approx_scale, ceil, cross, div255,
        dot, floor, fma, fract, from_half, if_then_else, isfinite, join, join4, length, lrint,
        mulhi, mull, mull_u8, mull_u16, naive_if_then_else, normalize, pin, pin_vectors,
        reduce_add, saturated_add, saturated_add_u8, saturated_add_u16, shuffle, sqrt,
        strided_load2, strided_load2_into, strided_load4, strided_load4_into, to_half, trunc,
        vx_max as max, vx_min as min,
    };

    pub type float2 = Float2;
    pub type float4 = Float4;
    pub type float8 = Float8;
    pub type double2 = Double2;
    pub type double4 = Double4;
    pub type double8 = Double8;
    pub type byte2 = Byte2;
    pub type byte4 = Byte4;
    pub type byte8 = Byte8;
    pub type byte16 = Byte16;
    pub type int2 = Int2;
    pub type int4 = Int4;
    pub type int8 = Int8;
    pub type ushort2 = Ushort2;
    pub type ushort4 = Ushort4;
    pub type ushort8 = Ushort8;
    pub type uint2 = Uint2;
    pub type uint4 = Uint4;
    pub type uint8 = Uint8;
    pub type long2 = Long2;
    pub type long4 = Long4;
    pub type long8 = Long8;
    pub type half2 = Half2;
    pub type half4 = Half4;
    pub type half8 = Half8;
    pub type Vec<T, const N: usize> = Vx<T, N>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_and_masks() {
        assert_eq!(std::mem::size_of::<Float2>(), 8);
        assert_eq!(std::mem::size_of::<Float4>(), 16);
        assert_eq!(std::mem::size_of::<Float8>(), 32);
        let mask: Int4 = Float4::new([1.0, 2.0, 3.0, 4.0]).lt(&Float4::new([1.0, 2.0, 4.0, 8.0]));
        assert_eq!(mask.lanes, [0, 0, -1, -1]);
        assert!(any(mask));
        assert!(!all(mask));
    }

    #[test]
    fn conversions_and_half() {
        let f = Float8::new([0.0, 0.5, 1.0, 2.0, -4.0, -0.5, -1.0, -2.0]);
        let h = to_half(f);
        assert_eq!(
            h.lanes,
            [
                0x0000, 0x3800, 0x3c00, 0x4000, 0xc400, 0xb800, 0xbc00, 0xc000
            ]
        );
        assert_eq!(from_half::<f32, 8>(h), f);
        assert_eq!(to_half(Float4::splat(f32::INFINITY)).lanes, [0x7c00; 4]);
    }

    #[test]
    fn scaled_divider_matches_rounding() {
        for &denom in &[2u32, 3, 5, 7, 27, 65_535, 15_485_863, 512_927_377] {
            let d = ScaledDividerU32::new(denom);
            for &i in &[0u32, 1, 1000, u32::MAX / 2] {
                let want = ((i as f64 + d.half() as f64) / denom as f64).floor() as u32;
                let got = d.divide(i + d.half()).lanes[0];
                assert!(got.abs_diff(want) <= 1);
            }
        }
    }

    #[test]
    fn dot_cross_length() {
        assert_eq!(dot(Int2::new([1, 1]), Int2::new([1, -1])), 0);
        assert_eq!(cross(Int2::new([1, 1]), Int2::new([1, -1])), -2);
        assert!((length(Float2::new([3.0, 4.0])) - 5.0).abs() < 1e-6);
        let n = normalize(Float2::new([0.0, 0.0]));
        assert!(!isfinite(n));
    }
}
