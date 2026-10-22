use crate::point::Point;
use crate::scalar::{Scalar, scalar_nearly_equal, scalar_pin};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum CubicMapType {
    Line,
    CubeRoot,
    #[default]
    Solver,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkCubicMap {
    coeff: [Point; 3],
    map_type: CubicMapType,
}

pub type CubicMap = SkCubicMap;

impl SkCubicMap {
    pub fn new(p1: Point, p2: Point) -> Self {
        let p1 = Point::new(p1.x.clamp(0.0, 1.0), p1.y);
        let p2 = Point::new(p2.x.clamp(0.0, 1.0), p2.y);
        let s1 = p1 * 3.0;
        let s2 = p2 * 3.0;
        let coeff = [Point::new(1.0, 1.0) + s1 - s2, s2 - s1 - s1, s1];
        let mut map_type = CubicMapType::Solver;
        if scalar_nearly_equal(p1.x, p1.y) && scalar_nearly_equal(p2.x, p2.y) {
            map_type = CubicMapType::Line;
        } else if coeff_nearly_zero(coeff[1].x) && coeff_nearly_zero(coeff[2].x) {
            map_type = CubicMapType::CubeRoot;
        }
        Self { coeff, map_type }
    }

    pub fn is_linear(p1: Point, p2: Point) -> bool {
        scalar_nearly_equal(p1.x, p1.y) && scalar_nearly_equal(p2.x, p2.y)
    }

    pub fn compute_y_from_x(&self, x: Scalar) -> Scalar {
        let x = scalar_pin(x, 0.0, 1.0);
        if nearly_zero(x) || nearly_zero(1.0 - x) {
            return x;
        }
        if self.map_type == CubicMapType::Line {
            return x;
        }
        let t = if self.map_type == CubicMapType::CubeRoot {
            (x / self.coeff[0].x).powf(1.0 / 3.0)
        } else {
            cubic_solver(self.coeff[0].x, self.coeff[1].x, self.coeff[2].x, -x)
        };
        let a = self.coeff[0].y;
        let b = self.coeff[1].y;
        let c = self.coeff[2].y;
        ((a * t + b) * t + c) * t
    }

    pub fn compute_from_t(&self, t: Scalar) -> Point {
        let a = self.coeff[0];
        let b = self.coeff[1];
        let c = self.coeff[2];
        ((a * t + b) * t + c) * t
    }
}

fn cubic_solver(a: Scalar, b: Scalar, c: Scalar, d: Scalar) -> Scalar {
    let mut t = -d;
    for _ in 0..8 {
        debug_assert!((0.0..=1.0).contains(&t));
        let f = eval_poly(t, &[a, b, c, d]);
        if f.abs() <= 0.00005 {
            break;
        }
        let fp = eval_poly(t, &[3.0 * a, 2.0 * b, c]);
        let fpp = eval_poly(t, &[3.0 * a + 3.0 * a, 2.0 * b]);
        let numer = 2.0 * fp * f;
        let denom = (2.0 * fp).mul_add(fp, -(f * fpp));
        t -= numer / denom;
    }
    debug_assert!((0.0..=1.0).contains(&t));
    t
}

fn eval_poly(t: Scalar, coefficients: &[Scalar]) -> Scalar {
    let mut value = coefficients[0];
    for coefficient in &coefficients[1..] {
        value = value.mul_add(t, *coefficient);
    }
    value
}

fn nearly_zero(x: Scalar) -> bool {
    debug_assert!(x >= 0.0);
    x <= 0.0000000001
}

fn coeff_nearly_zero(delta: Scalar) -> bool {
    delta.abs() <= 0.0000001
}
