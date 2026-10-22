//! Port of `src/core/SkColorPriv.h`.

use crate::math::sk_mul_div255_round;
use crate::scalar::{Scalar, scalar_pin};
use crate::sk_color::{
    SkColor, SkColor4f, SkPMColor, sk_color_get_a, sk_color_get_b, sk_color_get_g, sk_color_get_r,
};

// macOS uses the default 32-bit packing (`SkTypes.h`): SK_R32_SHIFT == 0.
pub const SK_A32_SHIFT: u32 = 24;
pub const SK_R32_SHIFT: u32 = 0;
pub const SK_G32_SHIFT: u32 = 8;
pub const SK_B32_SHIFT: u32 = 16;

pub const SK_A32_BITS: u32 = 8;
pub const SK_R32_BITS: u32 = 8;
pub const SK_G32_BITS: u32 = 8;
pub const SK_B32_BITS: u32 = 8;

pub const SK_A32_MASK: u32 = (1 << SK_A32_BITS) - 1;
pub const SK_R32_MASK: u32 = (1 << SK_R32_BITS) - 1;
pub const SK_G32_MASK: u32 = (1 << SK_G32_BITS) - 1;
pub const SK_B32_MASK: u32 = (1 << SK_B32_BITS) - 1;

/// `SkAlpha255To256`.
#[inline]
pub fn sk_alpha255_to256(alpha: u8) -> u32 {
    alpha as u32 + 1
}

/// `SkAlphaMul(value, alpha256)`.
#[inline]
pub fn sk_alpha_mul(value: i32, alpha256: i32) -> i32 {
    (value * alpha256) >> 8
}

/// `SkUnitScalarClampToByte`.
#[inline]
pub fn sk_unit_scalar_clamp_to_byte(x: Scalar) -> u8 {
    (scalar_pin(x, 0.0, 1.0) * 255.0 + 0.5) as u8
}

#[inline]
pub const fn sk_get_packed_a32(packed: SkPMColor) -> u8 {
    (((packed << (24 - SK_A32_SHIFT)) >> 24) & 0xFF) as u8
}

#[inline]
pub const fn sk_get_packed_r32(packed: SkPMColor) -> u8 {
    (((packed << (24 - SK_R32_SHIFT)) >> 24) & 0xFF) as u8
}

#[inline]
pub const fn sk_get_packed_g32(packed: SkPMColor) -> u8 {
    (((packed << (24 - SK_G32_SHIFT)) >> 24) & 0xFF) as u8
}

#[inline]
pub const fn sk_get_packed_b32(packed: SkPMColor) -> u8 {
    (((packed << (24 - SK_B32_SHIFT)) >> 24) & 0xFF) as u8
}

/// `SkPackARGB32`.
#[inline]
pub fn sk_pack_argb32(a: u8, r: u8, g: u8, b: u8) -> SkPMColor {
    ((a as u32) << SK_A32_SHIFT)
        | ((r as u32) << SK_R32_SHIFT)
        | ((g as u32) << SK_G32_SHIFT)
        | ((b as u32) << SK_B32_SHIFT)
}

/// `SkPremultiplyARGBInline`.
#[inline]
pub fn sk_premultiply_argb_inline(a: u8, r: u8, g: u8, b: u8) -> SkPMColor {
    let (r, g, b) = if a != 255 {
        (
            sk_mul_div255_round(r as u16, a as u16) as u8,
            sk_mul_div255_round(g as u16, a as u16) as u8,
            sk_mul_div255_round(b as u16, a as u16) as u8,
        )
    } else {
        (r, g, b)
    };
    sk_pack_argb32(a, r, g, b)
}

/// `SkAlphaMulQ`.
#[inline]
pub fn sk_alpha_mul_q(c: u32, scale: u32) -> u32 {
    const K_MASK: u32 = 0x00FF_00FF;
    let rb = ((c & K_MASK) * scale) >> 8;
    let ag = ((c >> 8) & K_MASK) * scale;
    (rb & K_MASK) | (ag & !K_MASK)
}

/// `SkPMSrcOver`.
#[inline]
pub fn sk_pm_src_over(src: SkPMColor, dst: SkPMColor) -> SkPMColor {
    let scale = sk_alpha255_to256(255 - sk_get_packed_a32(src));

    const K_MASK: u32 = 0x00FF_00FF;
    let mut rb = (((dst & K_MASK) * scale) >> 8) & K_MASK;
    let mut ag = (((dst >> 8) & K_MASK) * scale) & !K_MASK;

    rb += src & K_MASK;
    ag += src & !K_MASK;

    // Color channels (but not alpha) can overflow, so we have to saturate to 0xFF in each lane.
    (rb & 0x0000_01FF).min(0x0000_00FF)
        | (ag & 0x0001_FF00).min(0x0000_FF00)
        | (rb & 0x01FF_0000).min(0x00FF_0000)
        | (ag & 0xFF00_0000)
}

/// `SkColorConverter`: converts an array of `SkColor` to `SkColor4f` once.
pub struct SkColorConverter {
    f_colors4f: Vec<SkColor4f>,
}

impl SkColorConverter {
    pub fn new(src: &[SkColor]) -> Self {
        Self {
            f_colors4f: src.iter().map(|&c| SkColor4f::from_color(c)).collect(),
        }
    }

    pub fn colors4f(&self) -> &[SkColor4f] {
        &self.f_colors4f
    }

    pub fn colors4f_mut(&mut self) -> &mut [SkColor4f] {
        &mut self.f_colors4f
    }
}

/// Re-interpret a color's channel access through `SkColorGet*`, mirroring the macros.
#[inline]
pub const fn sk_color_components(color: SkColor) -> (u8, u8, u8, u8) {
    (
        sk_color_get_a(color),
        sk_color_get_r(color),
        sk_color_get_g(color),
        sk_color_get_b(color),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sk_color::sk_color_set_argb;

    #[test]
    fn dev_packed_layout_is_rgba_on_macos() {
        assert_eq!(sk_pack_argb32(0xFE, 0xDC, 0xBA, 0x98), 0xFE98_BADC);
        assert_eq!(sk_get_packed_a32(0xFE98_BADC), 0xFE);
        assert_eq!(sk_get_packed_r32(0xFE98_BADC), 0xDC);
        assert_eq!(sk_get_packed_g32(0xFE98_BADC), 0xBA);
        assert_eq!(sk_get_packed_b32(0xFE98_BADC), 0x98);
    }

    #[test]
    fn dev_premultiply_rounds() {
        assert_eq!(
            sk_premultiply_argb_inline(255, 1, 2, 3),
            sk_pack_argb32(255, 1, 2, 3)
        );
        let pm = sk_premultiply_argb_inline(128, 200, 100, 50);
        assert_eq!(sk_get_packed_a32(pm), 128);
        assert_eq!(sk_get_packed_r32(pm), 100);
        assert_eq!(sk_get_packed_g32(pm), 50);
        assert_eq!(sk_get_packed_b32(pm), 25);
    }

    #[test]
    fn dev_color_converter() {
        let conv = SkColorConverter::new(&[sk_color_set_argb(255, 0, 0, 0), 0xFFFF_FFFF]);
        assert_eq!(conv.colors4f()[0], SkColor4f::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(conv.colors4f()[1], SkColor4f::new(1.0, 1.0, 1.0, 1.0));
    }
}
