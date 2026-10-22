//! Port of `include/core/SkColor.h` and `src/core/SkColor.cpp`.

use core::ops::{Index, IndexMut, Mul};

use crate::floating_point;
use crate::scalar::{Scalar, scalar_nearly_zero, scalar_pin};
use crate::sk_alpha_type::SkAlphaType;

/// 8-bit type for an alpha value.
pub type SkAlpha = u8;

/// 32-bit ARGB color value, unpremultiplied (`0xAARRGGBB`).
pub type SkColor = u32;

/// 32-bit ARGB color value, premultiplied. The byte order is the native `kN32` order
/// (`SK_R32_SHIFT == 0` on macOS, i.e. RGBA in memory, `0xAABBGGRR` numerically).
pub type SkPMColor = u32;

#[inline]
pub const fn sk_color_set_argb(a: u8, r: u8, g: u8, b: u8) -> SkColor {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
pub const fn sk_color_set_rgb(r: u8, g: u8, b: u8) -> SkColor {
    sk_color_set_argb(0xFF, r, g, b)
}

#[inline]
pub const fn sk_color_get_a(color: SkColor) -> u8 {
    ((color >> 24) & 0xFF) as u8
}

#[inline]
pub const fn sk_color_get_r(color: SkColor) -> u8 {
    ((color >> 16) & 0xFF) as u8
}

#[inline]
pub const fn sk_color_get_g(color: SkColor) -> u8 {
    ((color >> 8) & 0xFF) as u8
}

#[inline]
pub const fn sk_color_get_b(color: SkColor) -> u8 {
    (color & 0xFF) as u8
}

#[inline]
pub const fn sk_color_set_a(color: SkColor, a: u8) -> SkColor {
    (color & 0x00FF_FFFF) | ((a as u32) << 24)
}

pub const SK_ALPHA_TRANSPARENT: SkAlpha = 0x00;
pub const SK_ALPHA_OPAQUE: SkAlpha = 0xFF;

pub const SK_COLOR_TRANSPARENT: SkColor = 0x0000_0000;
pub const SK_COLOR_BLACK: SkColor = 0xFF00_0000;
pub const SK_COLOR_DKGRAY: SkColor = 0xFF44_4444;
pub const SK_COLOR_GRAY: SkColor = 0xFF88_8888;
pub const SK_COLOR_LTGRAY: SkColor = 0xFFCC_CCCC;
pub const SK_COLOR_WHITE: SkColor = 0xFFFF_FFFF;
pub const SK_COLOR_RED: SkColor = 0xFFFF_0000;
pub const SK_COLOR_GREEN: SkColor = 0xFF00_FF00;
pub const SK_COLOR_BLUE: SkColor = 0xFF00_00FF;
pub const SK_COLOR_YELLOW: SkColor = 0xFFFF_FF00;
pub const SK_COLOR_CYAN: SkColor = 0xFF00_FFFF;
pub const SK_COLOR_MAGENTA: SkColor = 0xFFFF_00FF;

/// `SkPreMultiplyARGB` (`src/core/SkColor.cpp`).
pub fn sk_pre_multiply_argb(a: u8, r: u8, g: u8, b: u8) -> SkPMColor {
    crate::sk_color_priv::sk_premultiply_argb_inline(a, r, g, b)
}

/// `SkPreMultiplyColor` (`src/core/SkColor.cpp`).
pub fn sk_pre_multiply_color(c: SkColor) -> SkPMColor {
    crate::sk_color_priv::sk_premultiply_argb_inline(
        sk_color_get_a(c),
        sk_color_get_r(c),
        sk_color_get_g(c),
        sk_color_get_b(c),
    )
}

/// `SkPMColorSetARGB` (`include/private/chromium/SkPMColor.h`).
pub fn sk_pm_color_set_argb(a: SkAlpha, r: u8, g: u8, b: u8) -> SkPMColor {
    crate::sk_color_priv::sk_pack_argb32(a, r, g, b)
}

pub fn sk_pm_color_get_a(c: SkPMColor) -> SkAlpha {
    crate::sk_color_priv::sk_get_packed_a32(c)
}

pub fn sk_pm_color_get_r(c: SkPMColor) -> u8 {
    crate::sk_color_priv::sk_get_packed_r32(c)
}

pub fn sk_pm_color_get_g(c: SkPMColor) -> u8 {
    crate::sk_color_priv::sk_get_packed_g32(c)
}

pub fn sk_pm_color_get_b(c: SkPMColor) -> u8 {
    crate::sk_color_priv::sk_get_packed_b32(c)
}

#[inline]
fn byte_to_scalar(x: u8) -> Scalar {
    (x as i32 as Scalar) / 255.0
}

#[inline]
fn byte_div_to_scalar(numer: i32, denom: u8) -> Scalar {
    // Cast to keep the answer signed.
    numer as Scalar / (denom as i32 as Scalar)
}

/// `SkRGBToHSV`.
pub fn sk_rgb_to_hsv(r: u8, g: u8, b: u8) -> [Scalar; 3] {
    let min = r.min(g).min(b);
    let max = r.max(g).max(b);
    let delta = (max - min) as i32;

    let v = byte_to_scalar(max);

    if delta == 0 {
        return [0.0, 0.0, v];
    }

    let s = byte_div_to_scalar(delta, max);

    let mut h = if r == max {
        byte_div_to_scalar(g as i32 - b as i32, delta as u8)
    } else if g == max {
        2.0 + byte_div_to_scalar(b as i32 - r as i32, delta as u8)
    } else {
        4.0 + byte_div_to_scalar(r as i32 - g as i32, delta as u8)
    };

    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    }

    [h, s, v]
}

/// `SkColorToHSV`: alpha is ignored.
pub fn sk_color_to_hsv(color: SkColor) -> [Scalar; 3] {
    sk_rgb_to_hsv(
        sk_color_get_r(color),
        sk_color_get_g(color),
        sk_color_get_b(color),
    )
}

/// `SkHSVToColor`: out-of-range hsv values are pinned, alpha passes through.
pub fn sk_hsv_to_color(a: u8, hsv: &[Scalar; 3]) -> SkColor {
    let s = scalar_pin(hsv[1], 0.0, 1.0);
    let v = scalar_pin(hsv[2], 0.0, 1.0);

    let v_byte = floating_point::round_to_int(v * 255.0) as u8;

    if scalar_nearly_zero(s) {
        return sk_color_set_argb(a, v_byte, v_byte, v_byte);
    }
    let hx = if hsv[0] < 0.0 || hsv[0] >= 360.0 {
        0.0
    } else {
        hsv[0] / 60.0
    };
    let w = hx.floor();
    let f = hx - w;

    let p = floating_point::round_to_int((1.0 - s) * v * 255.0) as u8;
    let q = floating_point::round_to_int((1.0 - (s * f)) * v * 255.0) as u8;
    let t = floating_point::round_to_int((1.0 - (s * (1.0 - f))) * v * 255.0) as u8;

    let (r, g, b) = match w as u32 {
        0 => (v_byte, t, p),
        1 => (q, v_byte, p),
        2 => (p, v_byte, t),
        3 => (p, q, v_byte),
        4 => (t, p, v_byte),
        _ => (v_byte, p, q),
    };
    sk_color_set_argb(a, r, g, b)
}

/// `SkHSVToColor` with an opaque alpha.
pub fn sk_hsv_to_color_opaque(hsv: &[Scalar; 3]) -> SkColor {
    sk_hsv_to_color(0xFF, hsv)
}

/// `SkColorChannel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SkColorChannel {
    R = 0,
    G = 1,
    B = 2,
    A = 3,
}

impl SkColorChannel {
    pub const K_LAST_ENUM: SkColorChannel = SkColorChannel::A;

    pub const fn to_u32(self) -> u32 {
        self as u32
    }
}

pub const K_RED_SK_COLOR_CHANNEL_FLAG: u32 = 1 << 0;
pub const K_GREEN_SK_COLOR_CHANNEL_FLAG: u32 = 1 << 1;
pub const K_BLUE_SK_COLOR_CHANNEL_FLAG: u32 = 1 << 2;
pub const K_ALPHA_SK_COLOR_CHANNEL_FLAG: u32 = 1 << 3;
pub const K_GRAY_SK_COLOR_CHANNEL_FLAG: u32 = 0x10;
pub const K_GRAY_ALPHA_SK_COLOR_CHANNEL_FLAGS: u32 =
    K_GRAY_SK_COLOR_CHANNEL_FLAG | K_ALPHA_SK_COLOR_CHANNEL_FLAG;
pub const K_RG_SK_COLOR_CHANNEL_FLAGS: u32 =
    K_RED_SK_COLOR_CHANNEL_FLAG | K_GREEN_SK_COLOR_CHANNEL_FLAG;
pub const K_RGB_SK_COLOR_CHANNEL_FLAGS: u32 =
    K_RG_SK_COLOR_CHANNEL_FLAGS | K_BLUE_SK_COLOR_CHANNEL_FLAG;
pub const K_RGBA_SK_COLOR_CHANNEL_FLAGS: u32 =
    K_RGB_SK_COLOR_CHANNEL_FLAGS | K_ALPHA_SK_COLOR_CHANNEL_FLAG;

const _: () = assert!(
    K_GRAY_SK_COLOR_CHANNEL_FLAG & K_RGBA_SK_COLOR_CHANNEL_FLAGS == 0,
    "bitfield conflict"
);

/// `SkRGBA4f<kAT>`: RGBA color value with four floating point components.
///
/// `AT` is the numeric value of `SkAlphaType`: `1` (`Opaque`) or `2` (`Premul`) select
/// premultiplied semantics; `3` (`Unpremul`) selects unpremultiplied semantics.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkRGBA4f<const AT: i32> {
    pub f_r: Scalar,
    pub f_g: Scalar,
    pub f_b: Scalar,
    pub f_a: Scalar,
}

/// `SkColor4f` (`SkRGBA4f<kUnpremul_SkAlphaType>`).
pub type SkColor4f = SkRGBA4f<{ SkAlphaType::Unpremul as i32 }>;
/// `SkPMColor4f` (`SkRGBA4f<kPremul_SkAlphaType>`).
pub type SkPMColor4f = SkRGBA4f<{ SkAlphaType::Premul as i32 }>;

impl<const AT: i32> SkRGBA4f<AT> {
    pub const fn new(r: Scalar, g: Scalar, b: Scalar, a: Scalar) -> Self {
        Self {
            f_r: r,
            f_g: g,
            f_b: b,
            f_a: a,
        }
    }

    /// `operator* (float scale)`.
    #[inline]
    pub fn mul_float(self, scale: Scalar) -> Self {
        Self::new(
            self.f_r * scale,
            self.f_g * scale,
            self.f_b * scale,
            self.f_a * scale,
        )
    }

    /// `operator* (const SkRGBA4f& scale)`.
    #[inline]
    pub fn mul_component_wise(self, scale: Self) -> Self {
        Self::new(
            self.f_r * scale.f_r,
            self.f_g * scale.f_g,
            self.f_b * scale.f_b,
            self.f_a * scale.f_a,
        )
    }

    /// `vec() const`.
    pub fn vec(&self) -> [Scalar; 4] {
        [self.f_r, self.f_g, self.f_b, self.f_a]
    }

    /// `vec()`.
    pub fn vec_mut(&mut self) -> &mut [Scalar; 4] {
        // The fields are declared contiguously, matching the reference's `&fR`.
        unsafe { &mut *(self as *mut Self as *mut [Scalar; 4]) }
    }

    /// `array()`.
    pub fn array(&self) -> [Scalar; 4] {
        self.vec()
    }

    /// `isOpaque()`. Asserts (in debug) that `fA` is in range.
    pub fn is_opaque(&self) -> bool {
        debug_assert!(self.f_a <= 1.0 && self.f_a >= 0.0);
        self.f_a == 1.0
    }

    /// `fitsInBytes()`.
    pub fn fits_in_bytes(&self) -> bool {
        debug_assert!(self.f_a >= 0.0 && self.f_a <= 1.0);
        self.f_r >= 0.0
            && self.f_r <= 1.0
            && self.f_g >= 0.0
            && self.f_g <= 1.0
            && self.f_b >= 0.0
            && self.f_b <= 1.0
    }

    /// `toBytes_RGBA()`: bytes in RGBA order (e.g. `GrColor`).
    pub fn to_bytes_rgba(&self) -> u32 {
        sk4f_to_l32(self.vec())
    }

    /// `FromBytes_RGBA`.
    pub fn from_bytes_rgba(color: u32) -> Self {
        let v = sk4f_from_l32(color);
        Self::new(v[0], v[1], v[2], v[3])
    }

    /// `makeOpaque()`.
    pub fn make_opaque(&self) -> Self {
        Self::new(self.f_r, self.f_g, self.f_b, 1.0)
    }

    /// `pinAlpha()`.
    pub fn pin_alpha(&self) -> Self {
        Self::new(self.f_r, self.f_g, self.f_b, scalar_pin(self.f_a, 0.0, 1.0))
    }

    /// `withAlpha(float)`.
    pub fn with_alpha(&self, a: Scalar) -> Self {
        Self::new(self.f_r, self.f_g, self.f_b, a)
    }

    /// `withAlphaByte(uint8_t)`.
    pub fn with_alpha_byte(&self, a: u8) -> Self {
        Self::new(self.f_r, self.f_g, self.f_b, a as Scalar / 255.0)
    }
}

impl<const AT: i32> Index<usize> for SkRGBA4f<AT> {
    type Output = Scalar;

    fn index(&self, index: usize) -> &Scalar {
        match index {
            0 => &self.f_r,
            1 => &self.f_g,
            2 => &self.f_b,
            3 => &self.f_a,
            _ => panic!("SkRGBA4f index out of range: {index}"),
        }
    }
}

impl<const AT: i32> IndexMut<usize> for SkRGBA4f<AT> {
    fn index_mut(&mut self, index: usize) -> &mut Scalar {
        debug_assert!(index < 4);
        &mut self.vec_mut()[index]
    }
}

impl<const AT: i32> Mul<Scalar> for SkRGBA4f<AT> {
    type Output = Self;

    fn mul(self, scale: Scalar) -> Self {
        self.mul_float(scale)
    }
}

impl<const AT: i32> Mul<SkRGBA4f<AT>> for SkRGBA4f<AT> {
    type Output = Self;

    fn mul(self, scale: SkRGBA4f<AT>) -> Self {
        self.mul_component_wise(scale)
    }
}

impl SkRGBA4f<{ SkAlphaType::Unpremul as i32 }> {
    /// `SkRGBA4f<kUnpremul>::FromColor`.
    pub fn from_color(color: SkColor) -> Self {
        let v = sk4f_from_l32(color);
        // `swizzle_rb`: the little-endian load is BGRA; FromColor is RGBA.
        Self::new(v[2], v[1], v[0], v[3])
    }

    /// `SkRGBA4f<kUnpremul>::toSkColor()`.
    pub fn to_sk_color(&self) -> SkColor {
        let v = [self.f_b, self.f_g, self.f_r, self.f_a];
        sk4f_to_l32(v)
    }

    /// `premul()`.
    pub fn premul(&self) -> SkRGBA4f<{ SkAlphaType::Premul as i32 }> {
        SkRGBA4f::new(
            self.f_r * self.f_a,
            self.f_g * self.f_a,
            self.f_b * self.f_a,
            self.f_a,
        )
    }
}

impl SkRGBA4f<{ SkAlphaType::Premul as i32 }> {
    /// `SkRGBA4f<kPremul>::unpremul()`.
    pub fn unpremul(&self) -> SkRGBA4f<{ SkAlphaType::Unpremul as i32 }> {
        if self.f_a == 0.0 {
            SkRGBA4f::new(0.0, 0.0, 0.0, 0.0)
        } else {
            let inv_alpha = 1.0 / self.f_a;
            SkRGBA4f::new(
                self.f_r * inv_alpha,
                self.f_g * inv_alpha,
                self.f_b * inv_alpha,
                self.f_a,
            )
        }
    }

    /// `SkRGBA4f<kPremul>::FromPMColor`.
    pub fn from_pm_color(color: SkPMColor) -> Self {
        let v = sk4f_from_l32(color);
        Self::new(v[0], v[1], v[2], v[3])
    }
}

/// `Sk4f_fromL32`: little-endian byte load, scaled to [0, 1].
fn sk4f_from_l32(px: u32) -> [Scalar; 4] {
    [
        (px & 0xFF) as Scalar * (1.0 / 255.0),
        ((px >> 8) & 0xFF) as Scalar * (1.0 / 255.0),
        ((px >> 16) & 0xFF) as Scalar * (1.0 / 255.0),
        ((px >> 24) & 0xFF) as Scalar * (1.0 / 255.0),
    ]
}

/// `Sk4f_toL32`: rounds, pins to [0, 255], and packs little-endian bytes.
fn sk4f_to_l32(px: [Scalar; 4]) -> u32 {
    let mut bytes = [0u8; 4];
    for i in 0..4 {
        let v = (px[i] * 255.0 + 0.5).clamp(0.0, 255.0);
        bytes[i] = v as u8;
    }
    u32::from_le_bytes(bytes)
}

impl From<SkColor> for SkColor4f {
    fn from(color: SkColor) -> SkColor4f {
        SkColor4f::from_color(color)
    }
}

pub mod sk_colors {
    use super::{SkColor4f, SkRGBA4f};
    use crate::sk_alpha_type::SkAlphaType;

    type Unpremul = SkRGBA4f<{ SkAlphaType::Unpremul as i32 }>;

    pub const TRANSPARENT: SkColor4f = Unpremul::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: SkColor4f = Unpremul::new(0.0, 0.0, 0.0, 1.0);
    pub const DK_GRAY: SkColor4f = Unpremul::new(0.25, 0.25, 0.25, 1.0);
    pub const GRAY: SkColor4f = Unpremul::new(0.50, 0.50, 0.50, 1.0);
    pub const LT_GRAY: SkColor4f = Unpremul::new(0.75, 0.75, 0.75, 1.0);
    pub const WHITE: SkColor4f = Unpremul::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: SkColor4f = Unpremul::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: SkColor4f = Unpremul::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: SkColor4f = Unpremul::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: SkColor4f = Unpremul::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: SkColor4f = Unpremul::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: SkColor4f = Unpremul::new(1.0, 0.0, 1.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sk_color_pack_unpack() {
        assert_eq!(sk_color_set_argb(0x12, 0x34, 0x56, 0x78), 0x1234_5678);
        assert_eq!(sk_color_get_a(0x1234_5678), 0x12);
        assert_eq!(sk_color_get_r(0x1234_5678), 0x34);
        assert_eq!(sk_color_get_g(0x1234_5678), 0x56);
        assert_eq!(sk_color_get_b(0x1234_5678), 0x78);
        assert_eq!(sk_color_set_a(0x1234_5678, 0xAB), 0xAB34_5678);
        assert_eq!(SK_COLOR_RED, 0xFFFF_0000);
    }

    #[test]
    fn dev_sk_color4f_round_trip() {
        let c4 = SkColor4f::from_color(SK_COLOR_WHITE);
        assert_eq!(c4, SkColor4f::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(c4.to_sk_color(), SK_COLOR_WHITE);
        assert_eq!(
            SkColor4f::from_bytes_rgba(0x1122_3344).to_bytes_rgba(),
            0x1122_3344
        );
    }

    #[test]
    fn dev_sk_hsv_gray() {
        assert_eq!(sk_rgb_to_hsv(10, 10, 10), [0.0, 0.0, 10.0 / 255.0]);
        assert_eq!(
            sk_hsv_to_color(0xFF, &[0.0, 0.0, 10.0 / 255.0]),
            sk_color_set_rgb(10, 10, 10)
        );
    }
}
