//! Fixed-point coordinate tiling helpers from
//! `SkBitmapProcState_matrixProcs.cpp`.
//!
//! The full `MatrixProc` selection path also requires the bitmap shader state
//! (`SkBitmapProcState`, `SkPixmap`, and tile-mode enums), which is introduced
//! with the bitmap shading milestone. The source-faithful tiling and packing
//! primitives are implemented here without depending on that state.

use crate::fixed::{
    SK_FIXED_1, SkFixed, SkFixed3232, sk_fixed_floor_to_int, sk_fixed3232_to_fixed,
    sk_fixed3232_to_int,
};

pub const DECAL_DX_MIN: SkFixed = SK_FIXED_1 / 256;

pub fn clamp(fx: SkFixed, max: i32) -> u32 {
    (fx >> 16).clamp(0, max) as u32
}

pub fn repeat(fx: SkFixed, max: i32) -> u32 {
    debug_assert!(max < 65535);
    ((fx as u32 & 0xffff).wrapping_mul((max + 1) as u32)) >> 16
}

pub fn mirror(fx: SkFixed, max: i32) -> u32 {
    debug_assert!(max < 65535);
    let s = (fx << 15) >> 31;
    ((((fx ^ s) as u32) & 0xffff).wrapping_mul((max + 1) as u32)) >> 16
}

pub fn int_clamp(x: i32, n: i32) -> u16 {
    if x < 0 {
        0
    } else if x >= n {
        (n - 1) as u16
    } else {
        x as u16
    }
}

pub fn sk_int_mod(x: i32, n: i32) -> i32 {
    debug_assert!(n > 0);
    if (x as u32) >= n as u32 {
        if x < 0 { n + !(!x % n) } else { x % n }
    } else {
        x
    }
}

pub fn int_repeat(x: i32, n: i32) -> u16 {
    sk_int_mod(x, n) as u16
}

pub fn int_mirror(x: i32, n: i32) -> u16 {
    let x = sk_int_mod(x, 2 * n);
    if x >= n {
        (n + !(x - n)) as u16
    } else {
        x as u16
    }
}

pub fn can_truncate_to_fixed_for_decal(fx: SkFixed, dx: SkFixed, count: usize, max: u32) -> bool {
    debug_assert!(count > 0);
    if dx <= DECAL_DX_MIN {
        return false;
    }
    if sk_fixed_floor_to_int(fx) as u32 >= max {
        return false;
    }
    let last_fx = fx as i64 + dx as i64 * (count as i64 - 1);
    last_fx >= i32::MIN as i64
        && last_fx <= i32::MAX as i64
        && (((last_fx as i32) >> 16) as u32) < max
}

pub fn decal_nofilter_scale(dst: &mut [u16], mut fx: SkFixed, dx: SkFixed, mut count: usize) {
    let mut index = 0;
    while count > 2 {
        if index + 1 >= dst.len() {
            return;
        }
        dst[index] = ((fx.wrapping_add(0)) >> 16) as u16;
        dst[index + 1] = ((fx.wrapping_add(dx)) >> 16) as u16;
        index += 2;
        fx = fx.wrapping_add(dx.wrapping_add(dx));
        count -= 2;
    }
    if count == 2 {
        if index >= dst.len() {
            return;
        }
        dst[index] = (fx >> 16) as u16;
        if index + 1 < dst.len() {
            dst[index + 1] = (fx.wrapping_add(dx) >> 16) as u16;
        }
    } else if count == 1 && index < dst.len() {
        dst[index] = (fx >> 16) as u16;
    }
}

pub fn extract_low_bits_clamp_clamp(fx: SkFixed, _max: i32) -> u32 {
    ((fx >> 12) & 0xf) as u32
}

pub fn extract_low_bits_general(fx: SkFixed, max: i32) -> u32 {
    extract_low_bits_clamp_clamp((fx as u32 & 0xffff) as i32 * (max + 1), max)
}

pub fn pack(
    f: SkFixed,
    max: i32,
    one: SkFixed,
    tile: fn(SkFixed, i32) -> u32,
    low_bits: fn(SkFixed, i32) -> u32,
) -> u32 {
    let mut packed = tile(f, max);
    packed = (packed << 4) | low_bits(f, max);
    (packed << 14) | tile(f.wrapping_add(one), max)
}

pub fn pack_clamp(f: SkFixed, max: u32) -> u32 {
    pack(
        f,
        max as i32,
        SK_FIXED_1,
        clamp,
        extract_low_bits_clamp_clamp,
    )
}

pub fn pack_repeat(f: SkFixed, max: u32, width: usize) -> u32 {
    pack(
        f,
        max as i32,
        (SK_FIXED_1 as i64 / width as i64) as SkFixed,
        repeat,
        extract_low_bits_general,
    )
}

pub fn pack_mirror(f: SkFixed, max: u32, width: usize) -> u32 {
    pack(
        f,
        max as i32,
        (SK_FIXED_1 as i64 / width as i64) as SkFixed,
        mirror,
        extract_low_bits_general,
    )
}

pub fn sk_fixed3232_saturate2fixed(mut x: SkFixed3232) -> SkFixed {
    const MAX_HI: i64 = i16::MAX as i64;
    const MIN_HI: i64 = i16::MIN as i64;
    const MAX_FX: SkFixed3232 = (i16::MAX as i64) << 32;
    const MIN_FX: SkFixed3232 = (0xffff_8000u64 as i64) << 32;

    if (x >> 32) >= MAX_HI {
        x = MAX_FX;
    }
    if (x >> 32) <= MIN_HI {
        x = MIN_FX;
    }
    sk_fixed3232_to_fixed(x)
}

pub fn fixed3232_floor_to_int(x: SkFixed3232) -> i32 {
    sk_fixed3232_to_int(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_tile_helpers() {
        assert_eq!(clamp(-SK_FIXED_1, 10), 0);
        assert_eq!(clamp(3 * SK_FIXED_1, 10), 3);
        assert_eq!(repeat(SK_FIXED_1, 9), 0);
        assert_eq!(int_clamp(-1, 5), 0);
        assert_eq!(int_repeat(-1, 5), 4);
        assert_eq!(int_mirror(-1, 5), 0);
    }

    #[test]
    fn dev_pack_and_decal() {
        let mut dst = [0u16; 4];
        decal_nofilter_scale(&mut dst, SK_FIXED_1, SK_FIXED_1 / 2, 4);
        assert_eq!(dst, [1, 1, 2, 2]);
        assert!(can_truncate_to_fixed_for_decal(
            SK_FIXED_1 / 2,
            SK_FIXED_1,
            4,
            10
        ));
    }
}
