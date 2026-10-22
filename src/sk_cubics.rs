use crate::floating_point::{
    SK_DOUBLE_PI, double_nearly_zero, doubles_nearly_equal_ulps, ieee_double_divide,
};
use crate::sk_quads::SkQuads;

pub struct SkCubics;

fn nearly_equal(x: f64, y: f64) -> bool {
    if double_nearly_zero(x) {
        return double_nearly_zero(y);
    }
    doubles_nearly_equal_ulps(x, y)
}

fn close_to_a_quadratic(a: f64, b: f64) -> bool {
    if double_nearly_zero(b) {
        return double_nearly_zero(a);
    }
    (a / b).abs() < 1.0e-7
}

impl SkCubics {
    pub fn roots_real(a: f64, b: f64, c: f64, d: f64, solution: &mut [f64; 3]) -> usize {
        if close_to_a_quadratic(a, b) {
            let mut quad = [0.0; 2];
            let count = SkQuads::roots_real(b, c, d, &mut quad);
            solution[..count].copy_from_slice(&quad[..count]);
            return count;
        }
        if double_nearly_zero(d) {
            let mut quad = [0.0; 2];
            let mut num = SkQuads::roots_real(a, b, c, &mut quad);
            solution[..num].copy_from_slice(&quad[..num]);
            for value in solution.iter().take(num) {
                if double_nearly_zero(*value) {
                    return num;
                }
            }
            solution[num] = 0.0;
            num += 1;
            return num;
        }
        if double_nearly_zero(a + b + c + d) {
            let mut quad = [0.0; 2];
            let mut num = SkQuads::roots_real(a, a + b, -d, &mut quad);
            solution[..num].copy_from_slice(&quad[..num]);
            for value in solution.iter().take(num) {
                if doubles_nearly_equal_ulps(*value, 1.0) {
                    return num;
                }
            }
            solution[num] = 1.0;
            num += 1;
            return num;
        }

        let inv_a = ieee_double_divide(1.0, a);
        let aa = b * inv_a;
        let bb = c * inv_a;
        let cc = d * inv_a;
        let a2 = aa * aa;
        let q = (a2 - bb * 3.0) / 9.0;
        let r = (2.0 * a2 * aa - 9.0 * aa * bb + 27.0 * cc) / 54.0;
        let r2 = r * r;
        let q3 = q * q * q;
        let r2_minus_q3 = r2 - q3;
        if !r2_minus_q3.is_finite() {
            return 0;
        }
        let a_div3 = aa / 3.0;
        let mut roots = 1usize;
        if r2_minus_q3 < 0.0 {
            let theta = pin_double(r / q3.sqrt(), -1.0, 1.0).acos();
            let neg2_root_q = -2.0 * q.sqrt();
            let root = neg2_root_q * (theta / 3.0).cos() - a_div3;
            solution[0] = root;
            let root = neg2_root_q * ((theta + 2.0 * SK_DOUBLE_PI) / 3.0).cos() - a_div3;
            if !nearly_equal(solution[0], root) {
                solution[roots] = root;
                roots += 1;
            }
            let root = neg2_root_q * ((theta - 2.0 * SK_DOUBLE_PI) / 3.0).cos() - a_div3;
            if !nearly_equal(solution[0], root) && (roots == 1 || !nearly_equal(solution[1], root))
            {
                solution[roots] = root;
                roots += 1;
            }
        } else {
            let sqrt_r2_minus_q3 = r2_minus_q3.sqrt();
            let mut a = r.abs() + sqrt_r2_minus_q3;
            a = a.cbrt();
            if r > 0.0 {
                a = -a;
            }
            if !double_nearly_zero(a) {
                a += q / a;
            }
            let root = a - a_div3;
            solution[0] = root;
            if !double_nearly_zero(r2) && doubles_nearly_equal_ulps(r2, q3) {
                let root = -a / 2.0 - a_div3;
                if !nearly_equal(solution[0], root) {
                    solution[roots] = root;
                    roots += 1;
                }
            }
        }
        roots
    }

    pub fn roots_valid_t(a: f64, b: f64, c: f64, d: f64, solution: &mut [f64; 3]) -> usize {
        let mut all_roots = [0.0; 3];
        let real_roots = Self::roots_real(a, b, c, d, &mut all_roots);
        let mut found_roots = 0usize;
        for index in 0..real_roots {
            let t_value = all_roots[index];
            if t_value <= 1.00005 && (t_value >= 1.0 || doubles_nearly_equal_ulps(t_value, 1.0)) {
                if (found_roots < 1 || !doubles_nearly_equal_ulps(solution[0], 1.0))
                    && (found_roots < 2 || !doubles_nearly_equal_ulps(solution[1], 1.0))
                {
                    solution[found_roots] = 1.0;
                    found_roots += 1;
                }
            } else if t_value >= -0.00005 && (t_value <= 0.0 || double_nearly_zero(t_value)) {
                if (found_roots < 1 || !double_nearly_zero(solution[0]))
                    && (found_roots < 2 || !double_nearly_zero(solution[1]))
                {
                    solution[found_roots] = 0.0;
                    found_roots += 1;
                }
            } else if t_value > 0.0 && t_value < 1.0 {
                solution[found_roots] = t_value;
                found_roots += 1;
            }
        }
        found_roots
    }

    pub fn binary_search_roots_valid_t(
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        solution: &mut [f64; 3],
    ) -> usize {
        if !(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()) {
            return 0;
        }
        let mut regions = [0.0, 0.0, 0.0, 1.0];
        let mut min_max = [0.0; 2];
        let extrema_count = find_extrema_valid_t(a, b, c, &mut min_max);
        let mut start_index = 2 - extrema_count;
        if extrema_count == 1 {
            regions[start_index + 1] = min_max[0];
        }
        if extrema_count == 2 {
            regions[start_index + 1] = min_max[0].min(min_max[1]);
            regions[start_index + 2] = min_max[0].max(min_max[1]);
        }
        let mut found_roots = 0usize;
        while start_index < 3 {
            let root = binary_search(a, b, c, d, regions[start_index], regions[start_index + 1]);
            if root >= 0.0
                && (found_roots < 1 || !approximately_zero(solution[0] - root))
                && (found_roots < 2 || !approximately_zero(solution[1] - root))
            {
                solution[found_roots] = root;
                found_roots += 1;
            }
            start_index += 1;
        }
        found_roots
    }

    pub fn eval_at(a: f64, b: f64, c: f64, d: f64, t: f64) -> f64 {
        t.mul_add(t.mul_add(t.mul_add(a, b), c), d)
    }

    pub fn eval_at_coefficients(coefficients: &[f64; 4], t: f64) -> f64 {
        Self::eval_at(
            coefficients[0],
            coefficients[1],
            coefficients[2],
            coefficients[3],
            t,
        )
    }
}

fn pin_double(x: f64, lo: f64, hi: f64) -> f64 {
    let min = if hi < x { hi } else { x };
    if lo < min { min } else { lo }
}

fn approximately_zero(x: f64) -> bool {
    x.abs() < 0.00000001
}

fn find_extrema_valid_t(a: f64, b: f64, c: f64, t: &mut [f64; 2]) -> usize {
    let mut roots = [0.0; 2];
    let num_roots = SkQuads::roots_real(3.0 * a, 2.0 * b, c, &mut roots);
    let mut valid_roots = 0;
    for root in roots.iter().take(num_roots) {
        if *root >= 0.0 && *root <= 1.0 {
            t[valid_roots] = *root;
            valid_roots += 1;
        }
    }
    valid_roots
}

fn binary_search(a: f64, b: f64, c: f64, d: f64, start: f64, stop: f64) -> f64 {
    let mut start = start;
    let mut stop = stop;
    let left = SkCubics::eval_at(a, b, c, d, start);
    if approximately_zero(left) {
        return start;
    }
    let right = SkCubics::eval_at(a, b, c, d, stop);
    if !(left.is_finite() && right.is_finite()) {
        return -1.0;
    }
    if (left > 0.0 && right > 0.0) || (left < 0.0 && right < 0.0) {
        return -1.0;
    }
    for _ in 0..1000 {
        let step = (start + stop) / 2.0;
        let curr = SkCubics::eval_at(a, b, c, d, step);
        if approximately_zero(curr) {
            return step;
        }
        if (curr < 0.0 && left < 0.0) || (curr > 0.0 && left > 0.0) {
            start = step;
        } else {
            stop = step;
        }
    }
    -1.0
}
