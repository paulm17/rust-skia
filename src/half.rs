//! IEEE half floats (`SkHalf`) from `src/core/SkHalf.h` / `SkHalf.cpp`.

pub type SkHalf = u16;

#[allow(non_upper_case_globals)]
pub const SK_HALF_NaN: SkHalf = 0x7c01;
pub const SK_HALF_INFINITY: SkHalf = 0x7c00;
pub const SK_HALF_MIN: SkHalf = 0x0400;
pub const SK_HALF_MAX: SkHalf = 0x7bff;
pub const SK_HALF_EPSILON: SkHalf = 0x1400;
pub const SK_HALF_ONE: SkHalf = 0x3c00;

pub fn sk_float_to_half(value: f32) -> SkHalf {
    if value.is_nan() {
        return SK_HALF_NaN;
    }
    sk_float_to_half_bits(value)
}

pub fn sk_float_to_half_bits(x: f32) -> SkHalf {
    debug_assert!(!x.is_nan());
    let sem = x.to_bits() as i32;
    let s = sem & 0x8000_0000u32 as i32;
    let em = (sem ^ s).min(0x4780_0000);
    let magic = ((f32::from_bits(em as u32) * 8192.0).max(0.5).to_bits() as i32) & (255 << 23);
    let rounded = (f32::from_bits(em as u32) + f32::from_bits(magic as u32)).to_bits() as i32;
    let exp = (magic >> 13).wrapping_sub((127 - 15 + 13 + 1) << 10);
    let f16 = rounded.wrapping_add(exp);
    (((s as u32) >> 16) | (f16 as u32)) as SkHalf
}

pub fn sk_half_to_float(value: SkHalf) -> f32 {
    let wide = value as i32;
    let s = wide & 0x8000;
    let em = wide ^ s;
    let inf_or_nan = if em >= (31 << 10) { 255 << 23 } else { 0 };
    let is_norm = em > 0x3ff;
    let sub = ((em as f32) * (1.0 / (1 << 24) as f32)).to_bits() as i32;
    let norm = (em << 13) + ((127 - 15) << 23);
    let finite = if is_norm { norm } else { sub };
    f32::from_bits(((s as u32) << 16) | finite as u32 | inf_or_nan as u32)
}

pub fn sk_float_to_half_check(value: f32) -> SkHalf {
    let h = sk_float_to_half(value);
    debug_assert!((sk_half_to_float(h) - value).abs() <= value.abs() * 0.001 + f32::MIN_POSITIVE);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_skia() {
        assert_eq!(sk_float_to_half(0.0), 0);
        assert_eq!(sk_float_to_half(-0.0), 0x8000);
        assert_eq!(sk_float_to_half(1.0), SK_HALF_ONE);
        assert_eq!(sk_float_to_half(-1.0), 0xbc00);
        assert_eq!(sk_float_to_half(65504.0), SK_HALF_MAX);
        assert_eq!(sk_float_to_half(1.0 / 16384.0), SK_HALF_MIN);
        assert!(sk_half_to_float(sk_float_to_half(f32::INFINITY)).is_infinite());
        assert!(sk_half_to_float(sk_float_to_half(f32::NAN)).is_nan());
    }
}
