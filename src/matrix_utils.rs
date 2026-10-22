use crate::matrix::Matrix;
use crate::point::Point;
use crate::scalar::{
    SCALAR_NEARLY_ZERO, Scalar, scalar_abs, scalar_nearly_equal, scalar_nearly_equal_within,
    scalar_nearly_zero, scalar_nearly_zero_within, scalar_sqrt,
};

pub fn is_degenerate_2x2(scale_x: Scalar, skew_x: Scalar, skew_y: Scalar, scale_y: Scalar) -> bool {
    let perp_dot = scale_x * scale_y - skew_x * skew_y;
    scalar_nearly_zero_within(perp_dot, SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO)
}

pub fn decompose_upper2x2(
    matrix: &Matrix,
    rotation1: Option<&mut Point>,
    scale: Option<&mut Point>,
    rotation2: Option<&mut Point>,
) -> bool {
    let a = matrix.get(Matrix::M_SCALE_X);
    let b = matrix.get(Matrix::M_SKEW_X);
    let c = matrix.get(Matrix::M_SKEW_Y);
    let d = matrix.get(Matrix::M_SCALE_Y);

    if is_degenerate_2x2(a, b, c, d) {
        return false;
    }

    let (cos_q, sin_q, sa, sb, sd) = if scalar_nearly_equal(b, c) {
        (1.0, 0.0, a as f64, b as f64, d as f64)
    } else {
        let mut cos_q = a + d;
        let mut sin_q = c - b;
        let recip_len = 1.0 / scalar_sqrt(cos_q * cos_q + sin_q * sin_q);
        cos_q *= recip_len;
        sin_q *= recip_len;
        let sa = (a * cos_q + c * sin_q) as f64;
        let sb = (b * cos_q + d * sin_q) as f64;
        let sd = (-b * sin_q + d * cos_q) as f64;
        (cos_q, sin_q, sa, sb, sd)
    };

    let (cos1, sin1, w1, w2, cos2, sin2) = if scalar_nearly_zero(sb as Scalar) {
        (1.0, 0.0, sa, sd, cos_q, sin_q)
    } else {
        let diff = sa - sd;
        let discriminant = (diff * diff + 4.0f64 * sb * sb).sqrt();
        let trace = sa + sd;
        let (w1, w2) = if diff > 0.0 {
            (0.5 * (trace + discriminant), 0.5 * (trace - discriminant))
        } else {
            (0.5 * (trace - discriminant), 0.5 * (trace + discriminant))
        };
        let mut cos1 = sb as Scalar;
        let mut sin1 = (w1 - sa) as Scalar;
        let recip_len = 1.0 / scalar_sqrt(cos1 * cos1 + sin1 * sin1);
        cos1 *= recip_len;
        sin1 *= recip_len;
        let cos2 = cos1 * cos_q - sin1 * sin_q;
        let sin2 = sin1 * cos_q + cos1 * sin_q;
        (cos1, -sin1, w1, w2, cos2, sin2)
    };

    if let Some(scale) = scale {
        *scale = Point::new(w1 as Scalar, w2 as Scalar);
    }
    if let Some(rotation1) = rotation1 {
        *rotation1 = Point::new(cos1, sin1);
    }
    if let Some(rotation2) = rotation2 {
        *rotation2 = Point::new(cos2, sin2);
    }
    true
}

#[allow(non_snake_case)]
pub fn SkDecomposeUpper2x2(
    matrix: &Matrix,
    rotation1: Option<&mut Point>,
    scale: Option<&mut Point>,
    rotation2: Option<&mut Point>,
) -> bool {
    decompose_upper2x2(matrix, rotation1, scale, rotation2)
}

pub fn differential_area_scale(m: &Matrix, p: Point) -> Scalar {
    let m00 = m.get(Matrix::M_SCALE_X);
    let m01 = m.get(Matrix::M_SKEW_X);
    let m02 = m.get(Matrix::M_TRANS_X);
    let m10 = m.get(Matrix::M_SKEW_Y);
    let m11 = m.get(Matrix::M_SCALE_Y);
    let m12 = m.get(Matrix::M_TRANS_Y);
    let m20 = m.get(Matrix::M_PERSP_0);
    let m21 = m.get(Matrix::M_PERSP_1);

    let x = m00 * p.x + m01 * p.y + m02;
    let y = m10 * p.x + m11 * p.y + m12;
    let w = m20 * p.x + m21 * p.y + m.get(Matrix::M_PERSP_2);

    if w < SCALAR_NEARLY_ZERO {
        return Scalar::INFINITY;
    }

    let det =
        x * (m10 * m21 - m11 * m20) - y * (m00 * m21 - m01 * m20) + w * (m00 * m11 - m01 * m10);
    let denom = 1.0f64 / w as f64;
    let denom = denom * denom * denom;
    scalar_abs((det as f64 * denom) as Scalar)
}

pub fn nearly_affine(m: &Matrix, bounds: &crate::rect::Rect, tolerance: Scalar) -> bool {
    if !m.has_perspective() {
        return true;
    }

    let quad = bounds.to_quad(true);
    let mut xyw = [crate::point3::Point3::default(); 4];
    m.map_points_to_homogeneous(&mut xyw, &quad);

    let v1 = crate::point3::Point3::new(
        m.get(Matrix::M_SCALE_X),
        m.get(Matrix::M_SKEW_Y),
        m.get(Matrix::M_PERSP_0),
    );
    let v2 = crate::point3::Point3::new(
        m.get(Matrix::M_SKEW_X),
        m.get(Matrix::M_SCALE_Y),
        m.get(Matrix::M_PERSP_1),
    );
    let cross = crate::point3::Point3::new(
        v1.y * v2.z - v1.z * v2.y,
        v1.z * v2.x - v1.x * v2.z,
        v1.x * v2.y - v1.y * v2.x,
    );

    let area = |p: crate::point3::Point3| -> Option<Scalar> {
        if p.z < SCALAR_NEARLY_ZERO {
            return None;
        }
        let det = cross.x * p.x + cross.y * p.y + cross.z * p.z;
        let denom = 1.0f64 / p.z as f64;
        Some(scalar_abs((det as f64 * denom * denom * denom) as Scalar))
    };

    let Some(a0) = area(xyw[0]) else {
        return false;
    };
    let tolerance = tolerance * tolerance;
    for p in xyw.iter().skip(1) {
        let Some(a) = area(*p) else {
            return false;
        };
        if !scalar_nearly_equal_within(a0, a, tolerance) {
            return false;
        }
    }
    true
}

pub fn compute_res_scale_for_stroking(matrix: &Matrix) -> Scalar {
    let sx = crate::point::Point::new(matrix.get(Matrix::M_SCALE_X), matrix.get(Matrix::M_SKEW_Y))
        .length();
    let sy = crate::point::Point::new(matrix.get(Matrix::M_SKEW_X), matrix.get(Matrix::M_SCALE_Y))
        .length();
    if sx.is_finite() && sy.is_finite() {
        let scale = sx.max(sy);
        if scale > 0.0 {
            return scale;
        }
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_decompose_identity() {
        let mut rotation1 = Point::default();
        let mut scale = Point::default();
        let mut rotation2 = Point::default();
        assert!(decompose_upper2x2(
            &Matrix::new(),
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        ));
        assert_eq!(scale, Point::new(1.0, 1.0));
    }

    #[test]
    fn dev_degeneracy() {
        assert!(is_degenerate_2x2(0.0, 0.0, 0.0, 0.0));
        assert!(!is_degenerate_2x2(1.0, 0.0, 0.0, 1.0));
    }
}
