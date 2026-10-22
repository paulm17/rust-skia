//! Port of `src/core/SkColorData.h`.
#![allow(clippy::identity_op)]

use crate::sk_alpha_type::SkAlphaType;
use crate::sk_color::{SkColor, SkPMColor, SkRGBA4f, sk_color_set_rgb};
use crate::sk_color_priv::{
    SK_A32_SHIFT, SK_B32_SHIFT, SK_G32_SHIFT, SK_R32_SHIFT, sk_alpha_mul, sk_alpha255_to256,
    sk_get_packed_a32, sk_get_packed_b32, sk_get_packed_g32, sk_get_packed_r32, sk_pack_argb32,
};

////////////////////////////////////////////////////////////////////////////////////////////
// Convert a 16bit pixel to a 32bit pixel

pub const SK_R32_BITS: u32 = 8;
pub const SK_G32_BITS: u32 = 8;
pub const SK_B32_BITS: u32 = 8;

pub const SK_R16_BITS: u32 = 5;
pub const SK_G16_BITS: u32 = 6;
pub const SK_B16_BITS: u32 = 5;

pub const SK_R16_SHIFT: u32 = SK_B16_BITS + SK_G16_BITS;
pub const SK_G16_SHIFT: u32 = SK_B16_BITS;
pub const SK_B16_SHIFT: u32 = 0;

pub const SK_R16_MASK: u32 = (1 << SK_R16_BITS) - 1;
pub const SK_G16_MASK: u32 = (1 << SK_G16_BITS) - 1;
pub const SK_B16_MASK: u32 = (1 << SK_B16_BITS) - 1;

#[inline]
pub const fn sk_get_packed_r16(color: u32) -> u32 {
    (color >> SK_R16_SHIFT) & SK_R16_MASK
}

#[inline]
pub const fn sk_get_packed_g16(color: u32) -> u32 {
    (color >> SK_G16_SHIFT) & SK_G16_MASK
}

#[inline]
pub const fn sk_get_packed_b16(color: u32) -> u32 {
    (color >> SK_B16_SHIFT) & SK_B16_MASK
}

#[inline]
pub const fn sk_r16_to_r32(r: u32) -> u32 {
    (r << (8 - SK_R16_BITS)) | (r >> (2 * SK_R16_BITS - 8))
}

#[inline]
pub const fn sk_g16_to_g32(g: u32) -> u32 {
    (g << (8 - SK_G16_BITS)) | (g >> (2 * SK_G16_BITS - 8))
}

#[inline]
pub const fn sk_b16_to_b32(b: u32) -> u32 {
    (b << (8 - SK_B16_BITS)) | (b >> (2 * SK_B16_BITS - 8))
}

#[inline]
pub const fn sk_packed16_to_r32(c: u32) -> u32 {
    sk_r16_to_r32(sk_get_packed_r16(c))
}

#[inline]
pub const fn sk_packed16_to_g32(c: u32) -> u32 {
    sk_g16_to_g32(sk_get_packed_g16(c))
}

#[inline]
pub const fn sk_packed16_to_b32(c: u32) -> u32 {
    sk_b16_to_b32(sk_get_packed_b16(c))
}

//////////////////////////////////////////////////////////////////////////////

/// `SkSwizzle_RB`: reverse the bytes corresponding to red and blue in a packed pixel.
#[inline]
pub fn sk_swizzle_rb(c: u32) -> u32 {
    let k_rb_mask = (0xFF << SK_R32_SHIFT) | (0xFF << SK_B32_SHIFT);

    let c0 = (c >> SK_R32_SHIFT) & 0xFF;
    let c1 = (c >> SK_B32_SHIFT) & 0xFF;
    (c & !k_rb_mask) | (c0 << SK_B32_SHIFT) | (c1 << SK_R32_SHIFT)
}

#[inline]
pub fn sk_pack_argb_as_rgba(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 0) | ((g as u32) << 8) | ((b as u32) << 16)
}

#[inline]
pub fn sk_pack_argb_as_bgra(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0)
}

#[inline]
pub fn sk_swizzle_rgba_to_pmcolor(c: u32) -> SkPMColor {
    // SK_PMCOLOR_IS_RGBA on macOS.
    c
}

#[inline]
pub fn sk_swizzle_bgra_to_pmcolor(c: u32) -> SkPMColor {
    // SK_PMCOLOR_IS_BGRA is false on macOS.
    sk_swizzle_rb(c)
}

//////////////////////////////////////////////////////////////////////////////

/// ITU-R Recommendation BT.709 luminance coefficients.
pub const SK_ITU_BT709_LUM_COEFF_R: f32 = 0.2126;
pub const SK_ITU_BT709_LUM_COEFF_G: f32 = 0.7152;
pub const SK_ITU_BT709_LUM_COEFF_B: f32 = 0.0722;

pub const SK_LUM_COEFF_R: f32 = SK_ITU_BT709_LUM_COEFF_R;
pub const SK_LUM_COEFF_G: f32 = SK_ITU_BT709_LUM_COEFF_G;
pub const SK_LUM_COEFF_B: f32 = SK_ITU_BT709_LUM_COEFF_B;

/// `SkComputeLuminance`: BT.709 coefficients in 1.8 fixed point (summing to 256).
#[inline]
pub fn sk_compute_luminance(r: u8, g: u8, b: u8) -> u8 {
    ((r as u32 * 54 + g as u32 * 183 + b as u32 * 19) >> 8) as u8
}

/// `SkAlphaMulInv256`: `256 - (value * alpha256) / 255` in [0, 256].
#[inline]
pub fn sk_alpha_mul_inv256(value: u16, alpha256: u16) -> u16 {
    let prod = 0xFFFF - value as u32 * alpha256 as u32;
    ((prod + (prod >> 8)) >> 8) as u16
}

/// `SkAlphaBlend`.
#[inline]
pub fn sk_alpha_blend(src: i32, dst: i32, scale256: i32) -> i32 {
    debug_assert!((0..=256).contains(&scale256));
    dst + sk_alpha_mul(src - dst, scale256)
}

#[inline]
pub fn sk_pack_rgb16(r: u32, g: u32, b: u32) -> u16 {
    debug_assert!(r <= SK_R16_MASK && g <= SK_G16_MASK && b <= SK_B16_MASK);
    ((r << SK_R16_SHIFT) | (g << SK_G16_SHIFT) | (b << SK_B16_SHIFT)) as u16
}

pub const SK_R16_MASK_IN_PLACE: u32 = SK_R16_MASK << SK_R16_SHIFT;
pub const SK_G16_MASK_IN_PLACE: u32 = SK_G16_MASK << SK_G16_SHIFT;
pub const SK_B16_MASK_IN_PLACE: u32 = SK_B16_MASK << SK_B16_SHIFT;

///////////////////////////////////////////////////////////////////////////////

/// `SkFourByteInterp256`: scale is [0..256].
pub fn sk_four_byte_interp256(src: SkPMColor, dst: SkPMColor, scale: i32) -> SkPMColor {
    let a = sk_alpha_blend(
        sk_get_packed_a32(src) as i32,
        sk_get_packed_a32(dst) as i32,
        scale,
    ) as u8;
    let r = sk_alpha_blend(
        sk_get_packed_r32(src) as i32,
        sk_get_packed_r32(dst) as i32,
        scale,
    ) as u8;
    let g = sk_alpha_blend(
        sk_get_packed_g32(src) as i32,
        sk_get_packed_g32(dst) as i32,
        scale,
    ) as u8;
    let b = sk_alpha_blend(
        sk_get_packed_b32(src) as i32,
        sk_get_packed_b32(dst) as i32,
        scale,
    ) as u8;

    sk_pack_argb32(a, r, g, b)
}

/// `SkFourByteInterp`.
pub fn sk_four_byte_interp(src: SkPMColor, dst: SkPMColor, src_weight: u8) -> SkPMColor {
    let scale = sk_alpha255_to256(src_weight) as i32;
    sk_four_byte_interp256(src, dst, scale)
}

/// `SkSplay(uint32_t, uint32_t*, uint32_t*)`: `0xAARRGGBB -> 0x00AA00GG, 0x00RR00BB`.
#[inline]
pub fn sk_splay(color: u32) -> (u32, u32) {
    const K_MASK: u32 = 0x00FF_00FF;
    let ag = (color >> 8) & K_MASK;
    let rb = color & K_MASK;
    (ag, rb)
}

/// `SkSplay(uint32_t) -> uint64_t`: `0xAARRGGBB -> 0x00AA00GG00RR00BB`.
#[inline]
pub fn sk_splay_64(color: u32) -> u64 {
    const K_MASK: u32 = 0x00FF_00FF;
    let mut agrb = ((color >> 8) & K_MASK) as u64;
    agrb <<= 32;
    agrb |= (color & K_MASK) as u64;
    agrb
}

/// `SkUnsplay(uint32_t ag, uint32_t rb)`.
#[inline]
pub fn sk_unsplay(ag: u32, rb: u32) -> u32 {
    const K_MASK: u32 = 0xFF00_FF00;
    (ag & K_MASK) | ((rb & K_MASK) >> 8)
}

/// `SkUnsplay(uint64_t agrb)`.
#[inline]
pub fn sk_unsplay_64(agrb: u64) -> u32 {
    const K_MASK: u64 = 0xFF00_FF00;
    (((agrb & K_MASK) >> 8) | ((agrb >> 32) & K_MASK)) as u32
}

/// `SkFastFourByteInterp256_32`.
pub fn sk_fast_four_byte_interp256_32(src: SkPMColor, dst: SkPMColor, scale: u32) -> SkPMColor {
    debug_assert!(scale <= 256);

    let (src_ag, src_rb) = sk_splay(src);
    let (dst_ag, dst_rb) = sk_splay(dst);

    let ret_ag = src_ag * scale + (256 - scale) * dst_ag;
    let ret_rb = src_rb * scale + (256 - scale) * dst_rb;

    sk_unsplay(ret_ag, ret_rb)
}

/// `SkFastFourByteInterp256_64`.
pub fn sk_fast_four_byte_interp256_64(src: SkPMColor, dst: SkPMColor, scale: u32) -> SkPMColor {
    debug_assert!(scale <= 256);
    sk_unsplay_64(sk_splay_64(src) * scale as u64 + (256 - scale) as u64 * sk_splay_64(dst))
}

/// `SkFastFourByteInterp256`. The reference selects the 32-bit variant on 32-bit targets and
/// the 64-bit variant on 64-bit targets; Apple Silicon takes the 64-bit path.
#[inline]
pub fn sk_fast_four_byte_interp256(src: SkPMColor, dst: SkPMColor, scale: u32) -> SkPMColor {
    sk_fast_four_byte_interp256_64(src, dst, scale)
}

/// `SkFastFourByteInterp`.
pub fn sk_fast_four_byte_interp(src: SkPMColor, dst: SkPMColor, src_weight: u8) -> SkPMColor {
    let src_weight = src_weight as u32;
    sk_fast_four_byte_interp256(src, dst, src_weight + (src_weight >> 7))
}

/// `SkPMLerp`.
#[inline]
pub fn sk_pm_lerp(src: SkPMColor, dst: SkPMColor, scale: u32) -> SkPMColor {
    sk_fast_four_byte_interp256(src, dst, scale)
}

/// `SkBlendARGB32`.
pub fn sk_blend_argb32(src: SkPMColor, dst: SkPMColor, aa: u8) -> SkPMColor {
    let src_scale = sk_alpha255_to256(aa);
    let dst_scale = sk_alpha_mul_inv256(sk_get_packed_a32(src) as u16, src_scale as u16) as u32;

    const K_MASK: u32 = 0x00FF_00FF;

    let src_rb = (src & K_MASK) * src_scale;
    let src_ag = ((src >> 8) & K_MASK) * src_scale;

    let dst_rb = (dst & K_MASK) * dst_scale;
    let dst_ag = ((dst >> 8) & K_MASK) * dst_scale;

    (((src_rb + dst_rb) >> 8) & K_MASK) | ((src_ag + dst_ag) & !K_MASK)
}

////////////////////////////////////////////////////////////////////////////////////////////
// Convert a 32bit pixel to a 16bit pixel (no dither)

#[inline]
pub const fn sk_r32_to_r16(r: u32) -> u32 {
    r >> (SK_R32_BITS - SK_R16_BITS)
}

#[inline]
pub const fn sk_g32_to_g16(g: u32) -> u32 {
    g >> (SK_G32_BITS - SK_G16_BITS)
}

#[inline]
pub const fn sk_b32_to_b16(b: u32) -> u32 {
    b >> (SK_B32_BITS - SK_B16_BITS)
}

#[inline]
pub fn sk_pixel32_to_pixel16(c: SkPMColor) -> u16 {
    let r = ((c >> (SK_R32_SHIFT + (8 - SK_R16_BITS))) & SK_R16_MASK) << SK_R16_SHIFT;
    let g = ((c >> (SK_G32_SHIFT + (8 - SK_G16_BITS))) & SK_G16_MASK) << SK_G16_SHIFT;
    let b = ((c >> (SK_B32_SHIFT + (8 - SK_B16_BITS))) & SK_B16_MASK) << SK_B16_SHIFT;
    (r | g | b) as u16
}

#[inline]
pub fn sk_pack888_to_rgb16(r: u8, g: u8, b: u8) -> u16 {
    ((sk_r32_to_r16(r as u32) << SK_R16_SHIFT)
        | (sk_g32_to_g16(g as u32) << SK_G16_SHIFT)
        | (sk_b32_to_b16(b as u32) << SK_B16_SHIFT)) as u16
}

/////////////////////////////////////////////////////////////////////////////////////////

#[inline]
pub fn sk_pixel16_to_color(src: u16) -> SkColor {
    let src = src as u32;
    let r = sk_packed16_to_r32(src);
    let g = sk_packed16_to_g32(src);
    let b = sk_packed16_to_b32(src);

    debug_assert!((r >> (8 - SK_R16_BITS)) == sk_get_packed_r16(src));
    debug_assert!((g >> (8 - SK_G16_BITS)) == sk_get_packed_g16(src));
    debug_assert!((b >> (8 - SK_B16_BITS)) == sk_get_packed_b16(src));

    sk_color_set_rgb(r as u8, g as u8, b as u8)
}

///////////////////////////////////////////////////////////////////////////////

pub type SkPMColor16 = u16;

// Put in OpenGL order (r g b a)
pub const SK_A4444_SHIFT: u32 = 0;
pub const SK_R4444_SHIFT: u32 = 12;
pub const SK_G4444_SHIFT: u32 = 8;
pub const SK_B4444_SHIFT: u32 = 4;

#[inline]
pub fn sk_replicate_nibble(nib: u32) -> u8 {
    debug_assert!(nib <= 0xF);
    ((nib << 4) | nib) as u8
}

#[inline]
pub const fn sk_get_packed_a4444(c: u16) -> u32 {
    ((c as u32) >> SK_A4444_SHIFT) & 0xF
}

#[inline]
pub const fn sk_get_packed_r4444(c: u16) -> u32 {
    ((c as u32) >> SK_R4444_SHIFT) & 0xF
}

#[inline]
pub const fn sk_get_packed_g4444(c: u16) -> u32 {
    ((c as u32) >> SK_G4444_SHIFT) & 0xF
}

#[inline]
pub const fn sk_get_packed_b4444(c: u16) -> u32 {
    ((c as u32) >> SK_B4444_SHIFT) & 0xF
}

#[inline]
pub fn sk_packed4444_to_a32(c: u16) -> u8 {
    sk_replicate_nibble(sk_get_packed_a4444(c))
}

#[inline]
pub fn sk_pixel4444_to_pixel32(c: u16) -> SkPMColor {
    let d = (sk_get_packed_a4444(c) << SK_A32_SHIFT)
        | (sk_get_packed_r4444(c) << SK_R32_SHIFT)
        | (sk_get_packed_g4444(c) << SK_G32_SHIFT)
        | (sk_get_packed_b4444(c) << SK_B32_SHIFT);
    d | (d << 4)
}

pub const SK_PMCOLOR4F_TRANSPARENT: SkRGBA4f<{ SkAlphaType::Premul as i32 }> =
    SkRGBA4f::new(0.0, 0.0, 0.0, 0.0);
pub const SK_PMCOLOR4F_BLACK: SkRGBA4f<{ SkAlphaType::Premul as i32 }> =
    SkRGBA4f::new(0.0, 0.0, 0.0, 1.0);
pub const SK_PMCOLOR4F_WHITE: SkRGBA4f<{ SkAlphaType::Premul as i32 }> =
    SkRGBA4f::new(1.0, 1.0, 1.0, 1.0);
pub const SK_PMCOLOR4F_ILLEGAL: SkRGBA4f<{ SkAlphaType::Premul as i32 }> = SkRGBA4f::new(
    f32::NEG_INFINITY,
    f32::NEG_INFINITY,
    f32::NEG_INFINITY,
    f32::NEG_INFINITY,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_splay_and_unsplay() {
        let color: SkPMColor = 0xA1B2_C3D4;
        let (ag, rb) = sk_splay(color);
        assert_eq!(ag, 0x00A1_00C3);
        assert_eq!(rb, 0x00B2_00D4);
        assert_eq!(sk_unsplay(ag << 8, rb << 8), color);

        let agrb = sk_splay_64(color);
        assert_eq!(agrb, 0x00A1_00C3_00B2_00D4u64);
        assert_eq!(sk_unsplay_64(agrb << 8), color);
    }
}
