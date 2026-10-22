use crate::m44::M44;
use crate::matrix::Matrix;
use crate::matrix_utils;
use crate::point::Point;
use crate::point3::Point3;
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_nearly_zero};

pub const MAX_FLATTEN_SIZE: usize =
    9 * core::mem::size_of::<Scalar>() + core::mem::size_of::<u32>();

pub fn write_to_memory(matrix: &Matrix, buffer: &mut [u8]) -> usize {
    matrix.write_to_memory(buffer)
}

pub fn read_from_memory(matrix: &mut Matrix, buffer: &[u8]) -> usize {
    matrix.read_from_memory(buffer)
}

pub fn inverse_map_rect(mx: &Matrix, dst: &mut Rect, src: &Rect) -> bool {
    if mx.is_scale_translate() {
        if mx.get(Matrix::M_SCALE_X) == 0.0 || mx.get(Matrix::M_SCALE_Y) == 0.0 {
            return false;
        }
        let tx = mx.get(Matrix::M_TRANS_X);
        let ty = mx.get(Matrix::M_TRANS_Y);
        let sx = if mx.get_type() > Matrix::TRANSLATE_MASK {
            1.0 / mx.get(Matrix::M_SCALE_X)
        } else {
            1.0
        };
        let sy = if mx.get_type() > Matrix::TRANSLATE_MASK {
            1.0 / mx.get(Matrix::M_SCALE_Y)
        } else {
            1.0
        };
        let mut left = (src.left - tx) * sx;
        let mut right = (src.right - tx) * sx;
        let mut top = (src.top - ty) * sy;
        let mut bottom = (src.bottom - ty) * sy;
        if sx < 0.0 {
            core::mem::swap(&mut left, &mut right);
        }
        if sy < 0.0 {
            core::mem::swap(&mut top, &mut bottom);
        }
        *dst = Rect::new(left, top, right, bottom);
        return true;
    }

    let mut inverse = Matrix::new();
    if mx.invert(Some(&mut inverse)) {
        *dst = inverse.map_rect(src);
        true
    } else {
        false
    }
}

pub fn map_points_in_place_with_stride(
    mx: &Matrix,
    pts: &mut [Point],
    stride: usize,
    count: usize,
) {
    if stride == 0 {
        return;
    }
    for i in 0..count {
        let index = i * stride;
        if index >= pts.len() {
            break;
        }
        pts[index] = mx.map_point(pts[index]);
    }
}

pub fn map_points_with_stride(
    mx: &Matrix,
    dst: &mut [Point],
    dst_stride: usize,
    src: &[Point],
    src_stride: usize,
    count: usize,
) {
    if dst_stride == 0 || src_stride == 0 {
        return;
    }
    for i in 0..count {
        let di = i * dst_stride;
        let si = i * src_stride;
        if di >= dst.len() || si >= src.len() {
            break;
        }
        dst[di] = mx.map_point(src[si]);
    }
}

pub fn map_homogeneous_points_with_stride(
    mx: &Matrix,
    dst: &mut [Point3],
    dst_stride: usize,
    src: &[Point3],
    src_stride: usize,
    count: usize,
) {
    if dst_stride == 0 || src_stride == 0 {
        return;
    }
    for i in 0..count {
        let di = i * dst_stride;
        let si = i * src_stride;
        if di >= dst.len() || si >= src.len() {
            break;
        }
        dst[di] = mx.map_homogeneous_point(src[si]);
    }
}

pub fn post_idiv(matrix: &mut Matrix, div_x: i32, div_y: i32) -> bool {
    matrix.post_idiv(div_x, div_y)
}

pub fn cheap_equal(a: &Matrix, b: &Matrix) -> bool {
    core::ptr::eq(a, b) || a.get9_array() == b.get9_array()
}

pub fn m44_col_major(m: &M44) -> &[Scalar; 16] {
    &m.f_mat
}

pub fn is_scale_translate_as_m33(m: &M44) -> bool {
    m.rc(1, 0) == 0.0
        && m.rc(3, 0) == 0.0
        && m.rc(0, 1) == 0.0
        && m.rc(3, 1) == 0.0
        && m.rc(3, 3) == 1.0
}

pub fn map_rect(m: &M44, r: &Rect) -> Rect {
    m.map_rect(r)
}

pub fn differential_area_scale(m: &Matrix, p: Point) -> Scalar {
    matrix_utils::differential_area_scale(m, p)
}

pub fn nearly_affine(m: &Matrix, bounds: &Rect) -> bool {
    matrix_utils::nearly_affine(m, bounds, crate::scalar::SCALAR_NEARLY_ZERO)
}

pub fn compute_res_scale_for_stroking(matrix: &Matrix) -> Scalar {
    matrix_utils::compute_res_scale_for_stroking(matrix)
}

pub fn is_degenerate_2x2(scale_x: Scalar, skew_x: Scalar, skew_y: Scalar, scale_y: Scalar) -> bool {
    matrix_utils::is_degenerate_2x2(scale_x, skew_x, skew_y, scale_y)
}

pub fn nearly_zero(value: Scalar) -> bool {
    scalar_nearly_zero(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_inverse_map_rect_scale_translate() {
        let mut m = Matrix::new();
        m.set_scale_translate(2.0, 4.0, 10.0, 20.0);
        let mut dst = Rect::new(0.0, 0.0, 0.0, 0.0);
        assert!(inverse_map_rect(
            &m,
            &mut dst,
            &Rect::new(10.0, 20.0, 30.0, 60.0)
        ));
        assert_eq!(dst, Rect::new(0.0, 0.0, 10.0, 10.0));
    }
}
