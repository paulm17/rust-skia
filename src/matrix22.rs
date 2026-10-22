use crate::matrix::Matrix;
use crate::point::Point;
use crate::scalar::{Scalar, scalar_abs, scalar_copy_sign, scalar_sqrt};

pub fn compute_givens_rotation(h: Point, g: &mut Matrix) {
    let a = h.x;
    let b = h.y;
    let c;
    let s;
    if b == 0.0 {
        c = scalar_copy_sign(1.0, a);
        s = 0.0;
    } else if a == 0.0 {
        c = 0.0;
        s = -scalar_copy_sign(1.0, b);
    } else if scalar_abs(b) > scalar_abs(a) {
        let t = a / b;
        let u = scalar_copy_sign(scalar_sqrt(1.0 + t * t), b);
        s = -1.0 / u;
        c = -s * t;
    } else {
        let t = b / a;
        let u = scalar_copy_sign(scalar_sqrt(1.0 + t * t), a);
        c = 1.0 / u;
        s = -c * t;
    }
    g.set_sin_cos(s, c);
}

#[allow(non_snake_case)]
pub fn SkComputeGivensRotation(h: Point, g: &mut Matrix) {
    compute_givens_rotation(h, g);
}

pub fn givens_rotation(h: Point) -> Matrix {
    let mut result = Matrix::new();
    compute_givens_rotation(h, &mut result);
    result
}

pub fn givens_cos_sin(h: Point) -> (Scalar, Scalar) {
    let m = givens_rotation(h);
    (m.get(0), m.get(3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_givens_rotation_axes() {
        let m = givens_rotation(Point::new(3.0, 4.0));
        assert!((m.get(0) - 0.6).abs() < 1.0e-6);
        assert!((m.get(3) + 0.8).abs() < 1.0e-6);
    }
}
