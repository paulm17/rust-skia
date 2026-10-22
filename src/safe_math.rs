//! Overflow-checked arithmetic corresponding to `src/core/SkSafeMath.h`.

#[derive(Clone, Copy, Debug)]
pub struct SkSafeMath {
    ok: bool,
}

impl Default for SkSafeMath {
    fn default() -> Self {
        Self { ok: true }
    }
}

impl SkSafeMath {
    pub fn new() -> Self {
        Self { ok: true }
    }

    pub fn ok(&self) -> bool {
        self.ok
    }

    pub fn set_ok(&mut self, ok: bool) {
        self.ok &= ok;
    }

    pub fn mul(&mut self, x: usize, y: usize) -> usize {
        match x.checked_mul(y) {
            Some(v) => v,
            None => {
                self.ok = false;
                x.wrapping_mul(y)
            }
        }
    }

    pub fn add(&mut self, x: usize, y: usize) -> usize {
        let result = x.wrapping_add(y);
        self.ok &= result >= x;
        result
    }

    pub fn sub(&mut self, x: usize, y: usize) -> usize {
        self.ok &= x >= y;
        x.wrapping_sub(y)
    }

    pub fn add_int(&mut self, a: i32, b: i32) -> i32 {
        let result = a as i64 + b as i64;
        if result < i32::MIN as i64 || result > i32::MAX as i64 {
            self.ok = false;
        }
        result as i32
    }

    pub fn sub_int(&mut self, a: i32, b: i32) -> i32 {
        let result = a as i64 - b as i64;
        if result < i32::MIN as i64 || result > i32::MAX as i64 {
            self.ok = false;
        }
        result as i32
    }

    pub fn mul_int(&mut self, a: i32, b: i32) -> i32 {
        let result = a as i64 * b as i64;
        if result < i32::MIN as i64 || result > i32::MAX as i64 {
            self.ok = false;
        }
        result as i32
    }

    pub fn div_int(&mut self, a: i32, b: i32) -> i32 {
        match a.checked_div(b) {
            Some(v) => v,
            None => {
                self.ok = false;
                a
            }
        }
    }

    pub fn mod_int(&mut self, a: i32, b: i32) -> i32 {
        match a.checked_rem(b) {
            Some(v) => v,
            None => {
                self.ok = false;
                a
            }
        }
    }

    pub fn align_up(&mut self, x: usize, alignment: usize) -> usize {
        debug_assert!(alignment != 0 && alignment.is_power_of_two());
        let v = self.add(x, alignment - 1);
        v & !(alignment - 1)
    }

    pub fn align_up_non_pow2(&mut self, x: usize, alignment: usize) -> usize {
        if alignment == 0 {
            self.ok = false;
            return 0;
        }
        let aligned = self.add(x, alignment - 1);
        (aligned / alignment) * alignment
    }

    pub fn lcm(&mut self, a: usize, b: usize) -> usize {
        if a == 0 || b == 0 {
            self.ok = false;
            return 0;
        }
        let gcd = gcd(a, b);
        self.mul(a / gcd, b)
    }

    pub fn cast_to_i32<T: SafeCastI32>(&mut self, value: T) -> i32 {
        let (fits, cast) = value.cast_i32();
        self.ok &= fits;
        cast
    }

    pub fn cast_to_usize<T: SafeCastUsize>(&mut self, value: T) -> usize {
        let (fits, cast) = value.cast_usize();
        self.ok &= fits;
        cast
    }

    pub fn add_saturating(x: usize, y: usize) -> usize {
        x.saturating_add(y)
    }

    pub fn mul_saturating(x: usize, y: usize) -> usize {
        x.saturating_mul(y)
    }

    pub fn align4(x: usize) -> usize {
        let mut safe = Self::new();
        safe.align_up(x, 4)
    }
}

pub trait SafeCastI32: Copy {
    fn cast_i32(self) -> (bool, i32);
}
pub trait SafeCastUsize: Copy {
    fn cast_usize(self) -> (bool, usize);
}

macro_rules! safe_cast_i32 {
    ($($t:ty),* $(,)?) => {$(
        impl SafeCastI32 for $t {
            fn cast_i32(self) -> (bool, i32) {
                let v = self as i64;
                (v >= i32::MIN as i64 && v <= i32::MAX as i64, self as i32)
            }
        }
        impl SafeCastUsize for $t {
            fn cast_usize(self) -> (bool, usize) {
                let v = self as i128;
                (v >= 0 && v <= usize::MAX as i128, self as usize)
            }
        }
    )*};
}
safe_cast_i32!(
    i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, i128, u128
);

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn sk_safe_math_add(x: usize, y: usize) -> usize {
    SkSafeMath::add_saturating(x, y)
}

pub fn sk_safe_math_mul(x: usize, y: usize) -> usize {
    SkSafeMath::mul_saturating(x, y)
}

pub fn sk_safe_math_align4(x: usize) -> usize {
    SkSafeMath::align4(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_safe_math_detects_overflow_and_aligns() {
        let mut safe = SkSafeMath::new();
        assert_eq!(safe.align_up(9, 4), 12);
        assert!(safe.ok());
        assert_eq!(safe.mul(usize::MAX, 2), usize::MAX - 1);
        assert!(!safe.ok());

        let mut safe = SkSafeMath::new();
        assert_eq!(safe.add_int(i32::MAX, 1), i32::MIN);
        assert!(!safe.ok());
        assert_eq!(SkSafeMath::align4(17), 20);
    }
}
