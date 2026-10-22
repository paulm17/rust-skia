use crate::floating_point::{ieee_double_divide, is_finite_slice};
use crate::scalar::Scalar;

pub fn invert_2x2_matrix(input: &[Scalar; 4], output: &mut [Scalar; 4]) -> Scalar {
    invert_2x2_matrix_impl(input, Some(output))
}

pub fn invert_2x2_matrix_impl(input: &[Scalar; 4], output: Option<&mut [Scalar; 4]>) -> Scalar {
    let a00 = input[0] as f64;
    let a01 = input[1] as f64;
    let a10 = input[2] as f64;
    let a11 = input[3] as f64;

    let mut determinant = a00 * a11 - a01 * a10;
    if let Some(output) = output {
        let invdet = ieee_double_divide(1.0, determinant);
        output[0] = (a11 * invdet) as Scalar;
        output[1] = (-a01 * invdet) as Scalar;
        output[2] = (-a10 * invdet) as Scalar;
        output[3] = (a00 * invdet) as Scalar;
        if !is_finite_slice(output) {
            determinant = 0.0;
        }
    }
    determinant as Scalar
}

pub fn invert_3x3_matrix(input: &[Scalar; 9], output: &mut [Scalar; 9]) -> Scalar {
    invert_3x3_matrix_impl(input, Some(output))
}

pub fn invert_3x3_matrix_impl(input: &[Scalar; 9], output: Option<&mut [Scalar; 9]>) -> Scalar {
    let a00 = input[0] as f64;
    let a01 = input[1] as f64;
    let a02 = input[2] as f64;
    let a10 = input[3] as f64;
    let a11 = input[4] as f64;
    let a12 = input[5] as f64;
    let a20 = input[6] as f64;
    let a21 = input[7] as f64;
    let a22 = input[8] as f64;

    let b01 = a22 * a11 - a12 * a21;
    let b11 = -a22 * a10 + a12 * a20;
    let b21 = a21 * a10 - a11 * a20;

    let mut determinant = a00 * b01 + a01 * b11 + a02 * b21;
    if let Some(output) = output {
        let invdet = ieee_double_divide(1.0, determinant);
        output[0] = (b01 * invdet) as Scalar;
        output[1] = ((-a22 * a01 + a02 * a21) * invdet) as Scalar;
        output[2] = ((a12 * a01 - a02 * a11) * invdet) as Scalar;
        output[3] = (b11 * invdet) as Scalar;
        output[4] = ((a22 * a00 - a02 * a20) * invdet) as Scalar;
        output[5] = ((-a12 * a00 + a02 * a10) * invdet) as Scalar;
        output[6] = (b21 * invdet) as Scalar;
        output[7] = ((-a21 * a00 + a01 * a20) * invdet) as Scalar;
        output[8] = ((a11 * a00 - a01 * a10) * invdet) as Scalar;
        if !is_finite_slice(output) {
            determinant = 0.0;
        }
    }
    determinant as Scalar
}

pub fn invert_4x4_matrix(input: &[Scalar; 16], output: &mut [Scalar; 16]) -> Scalar {
    invert_4x4_matrix_impl(input, Some(output))
}

pub fn invert_4x4_matrix_impl(input: &[Scalar; 16], output: Option<&mut [Scalar; 16]>) -> Scalar {
    let a00 = input[0] as f64;
    let a01 = input[1] as f64;
    let a02 = input[2] as f64;
    let a03 = input[3] as f64;
    let a10 = input[4] as f64;
    let a11 = input[5] as f64;
    let a12 = input[6] as f64;
    let a13 = input[7] as f64;
    let a20 = input[8] as f64;
    let a21 = input[9] as f64;
    let a22 = input[10] as f64;
    let a23 = input[11] as f64;
    let a30 = input[12] as f64;
    let a31 = input[13] as f64;
    let a32 = input[14] as f64;
    let a33 = input[15] as f64;

    let mut b00 = a00 * a11 - a01 * a10;
    let mut b01 = a00 * a12 - a02 * a10;
    let mut b02 = a00 * a13 - a03 * a10;
    let mut b03 = a01 * a12 - a02 * a11;
    let mut b04 = a01 * a13 - a03 * a11;
    let mut b05 = a02 * a13 - a03 * a12;
    let mut b06 = a20 * a31 - a21 * a30;
    let mut b07 = a20 * a32 - a22 * a30;
    let mut b08 = a20 * a33 - a23 * a30;
    let mut b09 = a21 * a32 - a22 * a31;
    let mut b10 = a21 * a33 - a23 * a31;
    let mut b11 = a22 * a33 - a23 * a32;

    let mut determinant = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    if let Some(output) = output {
        let invdet = ieee_double_divide(1.0, determinant);
        b00 *= invdet;
        b01 *= invdet;
        b02 *= invdet;
        b03 *= invdet;
        b04 *= invdet;
        b05 *= invdet;
        b06 *= invdet;
        b07 *= invdet;
        b08 *= invdet;
        b09 *= invdet;
        b10 *= invdet;
        b11 *= invdet;

        output[0] = (a11 * b11 - a12 * b10 + a13 * b09) as Scalar;
        output[1] = (a02 * b10 - a01 * b11 - a03 * b09) as Scalar;
        output[2] = (a31 * b05 - a32 * b04 + a33 * b03) as Scalar;
        output[3] = (a22 * b04 - a21 * b05 - a23 * b03) as Scalar;
        output[4] = (a12 * b08 - a10 * b11 - a13 * b07) as Scalar;
        output[5] = (a00 * b11 - a02 * b08 + a03 * b07) as Scalar;
        output[6] = (a32 * b02 - a30 * b05 - a33 * b01) as Scalar;
        output[7] = (a20 * b05 - a22 * b02 + a23 * b01) as Scalar;
        output[8] = (a10 * b10 - a11 * b08 + a13 * b06) as Scalar;
        output[9] = (a01 * b08 - a00 * b10 - a03 * b06) as Scalar;
        output[10] = (a30 * b04 - a31 * b02 + a33 * b00) as Scalar;
        output[11] = (a21 * b02 - a20 * b04 - a23 * b00) as Scalar;
        output[12] = (a11 * b07 - a10 * b09 - a12 * b06) as Scalar;
        output[13] = (a00 * b09 - a01 * b07 + a02 * b06) as Scalar;
        output[14] = (a31 * b01 - a30 * b03 - a32 * b00) as Scalar;
        output[15] = (a20 * b03 - a21 * b01 + a22 * b00) as Scalar;
        if !is_finite_slice(output) {
            determinant = 0.0;
        }
    }
    determinant as Scalar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invert_identity_and_singular() {
        let identity = [1.0, 0.0, 0.0, 1.0];
        let mut out = [0.0; 4];
        assert_eq!(invert_2x2_matrix(&identity, &mut out), 1.0);
        assert_eq!(out, identity);

        let singular = [1.0, 2.0, 2.0, 4.0];
        assert_eq!(invert_2x2_matrix(&singular, &mut out), 0.0);
    }

    #[test]
    fn invert_3x3_and_4x4_identity() {
        let mut out3 = [0.0; 9];
        let identity3 = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(invert_3x3_matrix(&identity3, &mut out3), 1.0);
        assert_eq!(out3, identity3);

        let mut out4 = [0.0; 16];
        let identity4 = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert_eq!(invert_4x4_matrix(&identity4, &mut out4), 1.0);
        assert_eq!(out4, identity4);
    }
}
