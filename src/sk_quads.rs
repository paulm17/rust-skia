use crate::floating_point::{double_nearly_zero, doubles_nearly_equal_ulps};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RootResult {
    pub discriminant: f64,
    pub root0: f64,
    pub root1: f64,
}

pub struct SkQuads;

impl SkQuads {
    pub fn discriminant(a: f64, b: f64, c: f64) -> f64 {
        let b2 = b * b;
        let ac = a * c;
        let rough_discriminant = b2 - ac;
        if 3.0 * rough_discriminant.abs() >= b2 + ac {
            return rough_discriminant;
        }
        let b2_rounding_error = b.mul_add(b, -b2);
        let ac_rounding_error = a.mul_add(c, -ac);
        (b2 - ac) + (b2_rounding_error - ac_rounding_error)
    }

    pub fn roots(a: f64, b: f64, c: f64) -> RootResult {
        let discriminant = Self::discriminant(a, b, c);
        if a == 0.0 {
            let root = if b == 0.0 {
                if c == 0.0 { f64::INFINITY } else { f64::NAN }
            } else {
                c / (2.0 * b)
            };
            return RootResult {
                discriminant,
                root0: root,
                root1: root,
            };
        }
        if discriminant == 0.0 {
            return RootResult {
                discriminant,
                root0: b / a,
                root1: b / a,
            };
        }
        if discriminant > 0.0 {
            let d = discriminant.sqrt();
            let r = if b > 0.0 { b + d } else { b - d };
            return RootResult {
                discriminant,
                root0: r / a,
                root1: c / r,
            };
        }
        RootResult {
            discriminant,
            root0: f64::NAN,
            root1: f64::NAN,
        }
    }

    pub fn roots_real(a: f64, b: f64, c: f64, solution: &mut [f64; 2]) -> usize {
        if close_to_linear(a, b) {
            return solve_linear(b, c, solution);
        }
        let root_result = Self::roots(a, -0.5 * b, c);
        if !root_result.discriminant.is_finite() || root_result.discriminant < 0.0 {
            return 0;
        }
        let mut roots = 0;
        let r0 = zero_if_tiny(root_result.root0);
        if r0.is_finite() {
            solution[roots] = r0;
            roots += 1;
        }
        let r1 = zero_if_tiny(root_result.root1);
        if r1.is_finite() {
            solution[roots] = r1;
            roots += 1;
        }
        if roots == 2 && doubles_nearly_equal_ulps(solution[0], solution[1]) {
            roots = 1;
        }
        roots
    }

    pub fn eval_at(a: f64, b: f64, c: f64, t: f64) -> f64 {
        a.mul_add(t, b).mul_add(t, c)
    }
}

fn solve_linear(m: f64, b: f64, solution: &mut [f64; 2]) -> usize {
    if double_nearly_zero(m) {
        solution[0] = 0.0;
        return usize::from(double_nearly_zero(b));
    }
    solution[0] = -b / m;
    usize::from(solution[0].is_finite())
}

fn close_to_linear(a: f64, b: f64) -> bool {
    if a != 0.0 {
        return (b / a).abs() >= 1.0e+16;
    }
    true
}

fn zero_if_tiny(x: f64) -> f64 {
    if double_nearly_zero(x) { 0.0 } else { x }
}
