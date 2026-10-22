//! Deterministic pseudo-random generator from `src/core/SkRandom.h`.

use crate::fixed::sk_fixed_to_float;

#[derive(Clone, Debug)]
pub struct SkRandom {
    f_k: u32,
    f_j: u32,
}

impl Default for SkRandom {
    fn default() -> Self {
        Self::new()
    }
}

impl SkRandom {
    pub const fn new() -> Self {
        Self::from_seed(0)
    }

    pub const fn from_seed(seed: u32) -> Self {
        let mut f_k = Self::next_lcg(seed);
        if f_k == 0 {
            f_k = Self::next_lcg(f_k);
        }
        let mut f_j = Self::next_lcg(f_k);
        if f_j == 0 {
            f_j = Self::next_lcg(f_j);
        }
        Self { f_k, f_j }
    }

    pub fn set_seed(&mut self, seed: u32) {
        *self = Self::from_seed(seed);
    }

    pub const fn next_lcg(seed: u32) -> u32 {
        1_664_525u32.wrapping_mul(seed).wrapping_add(1_013_904_223)
    }

    pub fn next_u(&mut self) -> u32 {
        self.f_k = 30345u32
            .wrapping_mul(self.f_k & 0xffff)
            .wrapping_add(self.f_k >> 16);
        self.f_j = 18000u32
            .wrapping_mul(self.f_j & 0xffff)
            .wrapping_add(self.f_j >> 16);
        self.f_k.rotate_right(16).wrapping_add(self.f_j)
    }

    pub fn next_s(&mut self) -> i32 {
        self.next_u() as i32
    }

    pub fn next_f(&mut self) -> f32 {
        let floatint = 0x3f80_0000u32 | (self.next_u() >> 9);
        f32::from_bits(floatint) - 1.0
    }

    pub fn next_range_f(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f() * (max - min)
    }

    pub fn next_bits(&mut self, bit_count: u32) -> u32 {
        debug_assert!(bit_count > 0 && bit_count <= 32);
        self.next_u() >> (32 - bit_count)
    }

    pub fn next_range_u(&mut self, min: u32, max: u32) -> u32 {
        debug_assert!(min <= max);
        let range = max.wrapping_sub(min).wrapping_add(1);
        if range == 0 {
            self.next_u()
        } else {
            min.wrapping_add(self.next_u() % range)
        }
    }

    pub fn next_u_less_than(&mut self, count: u32) -> u32 {
        debug_assert!(count > 0);
        self.next_range_u(0, count - 1)
    }

    pub fn next_u_fixed1(&mut self) -> i32 {
        (self.next_u() >> 16) as i32
    }
    pub fn next_s_fixed1(&mut self) -> i32 {
        self.next_s() >> 15
    }

    pub fn next_u_scalar1(&mut self) -> f32 {
        sk_fixed_to_float(self.next_u_fixed1())
    }
    pub fn next_s_scalar1(&mut self) -> f32 {
        sk_fixed_to_float(self.next_s_fixed1())
    }

    pub fn next_range_scalar(&mut self, min: f32, max: f32) -> f32 {
        self.next_u_scalar1() * (max - min) + min
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_u() >= 0x8000_0000
    }
    pub fn next_biased_bool(&mut self, fraction_true: f32) -> bool {
        debug_assert!((0.0..=1.0).contains(&fraction_true));
        self.next_u_scalar1() <= fraction_true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_is_deterministic() {
        let mut a = SkRandom::new();
        let mut b = SkRandom::new();
        for _ in 0..100 {
            assert_eq!(a.next_u(), b.next_u());
        }
        let mut c = SkRandom::from_seed(42);
        assert_eq!(c.next_u(), SkRandom::from_seed(42).next_u());
    }
}
