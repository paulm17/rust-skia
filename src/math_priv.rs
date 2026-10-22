//! Required integer helpers from `src/core/SkMathPriv.h` and `SkMath.h`.

pub const fn sk_clz(x: u32) -> i32 {
    if x == 0 { 32 } else { x.leading_zeros() as i32 }
}

pub const fn sk_ctz(x: u32) -> i32 {
    if x == 0 {
        32
    } else {
        x.trailing_zeros() as i32
    }
}

pub const fn sk_pop_count(x: u32) -> i32 {
    x.count_ones() as i32
}

pub const fn sk_next_log2(value: u32) -> i32 {
    debug_assert!(value > 0);
    if value <= 1 {
        0
    } else {
        32 - sk_clz(value - 1)
    }
}

pub const fn sk_ceil_log2(value: u32) -> i32 {
    debug_assert!(value > 0);
    if value <= 1 {
        0
    } else {
        32 - sk_clz(value - 1)
    }
}

pub const fn sk_next_pow2(value: i32) -> i32 {
    debug_assert!(value >= 0);
    if value <= 0 {
        1
    } else {
        1 << sk_next_log2(value as u32)
    }
}

pub trait SkIsPow2 {
    fn is_pow2(self) -> bool;
}

macro_rules! impl_sk_is_pow2 {
    ($($t:ty),* $(,)?) => {
        $(
            impl SkIsPow2 for $t {
                #[inline]
                fn is_pow2(self) -> bool {
                    (self & self.wrapping_sub(1)) == 0
                }
            }
        )*
    };
}

impl_sk_is_pow2!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

pub fn sk_is_pow2<T: SkIsPow2>(value: T) -> bool {
    value.is_pow2()
}

pub const fn sk_clamp_pos(value: i32) -> i32 {
    if value < 0 { 0 } else { value }
}

pub fn sk_sqrt_bits(x: i32, count: i32) -> i32 {
    debug_assert!(x >= 0 && count > 0 && count <= 30);
    let mut root: u32 = 0;
    let mut rem_hi: u32 = 0;
    let mut rem_lo = x as u32;
    let mut count = count;
    loop {
        root <<= 1;
        rem_hi = (rem_hi << 2) | (rem_lo >> 30);
        rem_lo <<= 2;
        let test_div = (root << 1) + 1;
        if rem_hi >= test_div {
            rem_hi -= test_div;
            root += 1;
        }
        count -= 1;
        if count < 0 {
            break;
        }
    }
    root as i32
}

pub fn sk_sqrt32(n: i32) -> i32 {
    sk_sqrt_bits(n, 15)
}

pub fn sk_copy_sign32(x: i32, y: i32) -> i32 {
    let sign = (x ^ y) >> 31;
    (x ^ sign) - sign
}

pub const fn sk_prev_log2(value: u32) -> i32 {
    debug_assert!(value != 0);
    32 - sk_clz(value >> 1)
}

pub const fn sk_prev_pow2(value: i32) -> i32 {
    debug_assert!(value > 0);
    1 << sk_prev_log2(value as u32)
}

pub fn sk_next_size_pow2(n: usize) -> usize {
    let bits = usize::BITS as usize;
    let high_bit = 1usize << (bits - 1);
    if n == 0 {
        1
    } else if n >= high_bit {
        n
    } else {
        let mut n = n - 1;
        let mut shift = 1usize;
        while shift < bits {
            n |= n >> shift;
            shift <<= 1;
        }
        n + 1
    }
}

pub const fn sk_tdivmod_i32(numer: i32, denom: i32) -> (i32, i32) {
    (numer / denom, numer % denom)
}

pub fn sk_tdivmod_i64(numer: i64, denom: i64) -> (i64, i64) {
    (numer / denom, numer % denom)
}

macro_rules! tdivmod {
    ($($name:ident, $t:ty;)*) => {$(
        pub const fn $name(numer: $t, denom: $t) -> ($t, $t) {
            (numer / denom, numer % denom)
        }
    )*};
}
tdivmod!(
    sk_tdivmod_u8, u8; sk_tdivmod_u16, u16; sk_tdivmod_u32, u32; sk_tdivmod_u64, u64;
    sk_tdivmod_i8, i8; sk_tdivmod_i16, i16;
);

pub const fn sk_extract_sign(n: i32) -> i32 {
    n >> 31
}

pub const fn sk_apply_sign(n: i32, sign: i32) -> i32 {
    (n ^ sign) - sign
}

pub const fn sk_clamp_u_max(value: u32, max: u32) -> u32 {
    if value > max { max } else { value }
}

pub const fn sk_negate_to_size_t(value: i32) -> usize {
    (-(value as isize)) as usize
}

pub const fn sk_mul_div255_trunc(a: u8, b: u8) -> u32 {
    let prod = a as u32 * b as u32 + 1;
    (prod + (prod >> 8)) >> 8
}

pub const fn sk_mul_div255_ceiling(a: u8, b: u8) -> u32 {
    let prod = a as u32 * b as u32 + 255;
    (prod + (prod >> 8)) >> 8
}

pub const fn sk_div255_round(prod: u32) -> u32 {
    let prod = prod + 128;
    (prod + (prod >> 8)) >> 8
}

pub const fn sk_bswap32(x: u32) -> u32 {
    x.swap_bytes()
}

pub fn sk_fits_in_fixed<T: Into<f64>>(x: T) -> bool {
    x.into().abs() <= 32767.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_math_priv_logs_and_power_of_two() {
        assert_eq!(sk_next_log2(1), 0);
        assert_eq!(sk_next_log2(2), 1);
        assert_eq!(sk_next_log2(3), 2);
        assert_eq!(sk_next_log2(5), 3);
        assert_eq!(sk_next_pow2(5), 8);
        assert!(sk_is_pow2(64u32));
        assert!(sk_is_pow2(0u32));
        assert_eq!(sk_clz(1), 31);
        assert_eq!(sk_ctz(8), 3);
    }
}
