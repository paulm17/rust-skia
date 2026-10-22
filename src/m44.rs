use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::floating_point::{ieee_float_divide, is_finite};
use crate::matrix::Matrix;
use crate::matrix_invert::invert_4x4_matrix;
use crate::point::Point;
use crate::rect::Rect;
use crate::scalar::{
    SCALAR_1, SCALAR_HALF, Scalar, scalar_cos, scalar_nearly_zero, scalar_sin, scalar_tan,
};

pub const KW0_PLANE_DISTANCE: Scalar = 1.0 / (1 << 14) as Scalar;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: Scalar,
    pub y: Scalar,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V4 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
    pub w: Scalar,
}

pub type SkV2 = V2;
pub type SkV3 = V3;
pub type SkV4 = V4;

impl V2 {
    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    pub const fn make(x: Scalar, y: Scalar) -> Self {
        Self::new(x, y)
    }

    pub fn length_squared(&self) -> Scalar {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, v: &V2) -> Scalar {
        self.x * v.x + self.y * v.y
    }

    pub fn cross(&self, v: &V2) -> Scalar {
        self.x * v.y - self.y * v.x
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if scalar_nearly_zero(len) {
            *self
        } else {
            *self * (SCALAR_1 / len)
        }
    }

    pub fn ptr(&self) -> *const Scalar {
        &self.x
    }

    pub fn ptr_mut(&mut self) -> *mut Scalar {
        &mut self.x
    }
}

impl Add for V2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for V2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Neg for V2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl Mul for V2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<Scalar> for V2 {
    type Output = Self;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<Scalar> for V2 {
    type Output = Self;
    fn div(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl V3 {
    pub const fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    pub const fn make(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::new(x, y, z)
    }

    pub fn length_squared(&self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, v: &V3) -> Scalar {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn cross(&self, v: &V3) -> Self {
        Self::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }

    pub fn normalize(&self) -> Self {
        normalize_v3(*self)
    }

    pub fn ptr(&self) -> *const Scalar {
        &self.x
    }

    pub fn ptr_mut(&mut self) -> *mut Scalar {
        &mut self.x
    }
}

impl Add for V3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for V3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for V3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul for V3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<Scalar> for V3 {
    type Output = Self;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl V4 {
    pub const fn new(x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> Self {
        Self { x, y, z, w }
    }

    pub const fn make(x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> Self {
        Self::new(x, y, z, w)
    }

    pub fn length_squared(&self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, v: &V4) -> Scalar {
        self.x * v.x + self.y * v.y + self.z * v.z + self.w * v.w
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if scalar_nearly_zero(len) {
            *self
        } else {
            *self * (SCALAR_1 / len)
        }
    }

    pub fn ptr(&self) -> *const Scalar {
        &self.x
    }

    pub fn ptr_mut(&mut self) -> *mut Scalar {
        &mut self.x
    }
}

impl Add for V4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Sub for V4 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl Neg for V4 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl Mul for V4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
            self.w * rhs.w,
        )
    }
}

impl Mul<Scalar> for V4 {
    type Output = Self;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Mul<V4> for Scalar {
    type Output = V4;
    fn mul(self, rhs: V4) -> Self::Output {
        rhs * self
    }
}

impl Div<Scalar> for V4 {
    type Output = Self;
    fn div(self, rhs: Scalar) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl core::ops::Index<usize> for V4 {
    type Output = Scalar;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("V4 index out of range"),
        }
    }
}

impl core::ops::IndexMut<usize> for V4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("V4 index out of range"),
        }
    }
}

fn normalize_v3(v: V3) -> V3 {
    let len = v.length();
    if scalar_nearly_zero(len) {
        v
    } else {
        v * (SCALAR_1 / len)
    }
}

fn v4(v: V3, w: Scalar) -> V4 {
    V4::new(v.x, v.y, v.z, w)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct M44 {
    pub f_mat: [Scalar; 16],
}

pub type SkM44 = M44;

const _: () = {
    assert!(core::mem::size_of::<M44>() == 16 * core::mem::size_of::<Scalar>());
};

impl Default for M44 {
    fn default() -> Self {
        Self::new()
    }
}

impl M44 {
    pub const fn new() -> Self {
        Self {
            f_mat: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub const fn identity() -> Self {
        Self::new()
    }

    #[allow(non_snake_case)]
    pub const fn I() -> Self {
        Self::new()
    }

    pub const fn nan() -> Self {
        Self {
            f_mat: [Scalar::NAN; 16],
        }
    }

    pub const fn uninitialized() -> Self {
        Self { f_mat: [0.0; 16] }
    }

    pub fn concat(a: &M44, b: &M44) -> Self {
        let mut result = Self::uninitialized();
        result.set_concat(a, b);
        result
    }

    /// The arguments are interpreted in row-major order.
    pub fn from_row_major_array(r: &[Scalar; 16]) -> Self {
        let mut f_mat = [0.0; 16];
        for row in 0..4 {
            for col in 0..4 {
                f_mat[col * 4 + row] = r[row * 4 + col];
            }
        }
        Self { f_mat }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_row_major(
        m0: Scalar,
        m4: Scalar,
        m8: Scalar,
        m12: Scalar,
        m1: Scalar,
        m5: Scalar,
        m9: Scalar,
        m13: Scalar,
        m2: Scalar,
        m6: Scalar,
        m10: Scalar,
        m14: Scalar,
        m3: Scalar,
        m7: Scalar,
        m11: Scalar,
        m15: Scalar,
    ) -> Self {
        Self {
            f_mat: [
                m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15,
            ],
        }
    }

    #[allow(non_snake_case)]
    pub fn RowMajor(r: &[Scalar; 16]) -> Self {
        Self::from_row_major_array(r)
    }

    #[allow(non_snake_case)]
    pub fn ColMajor(c: &[Scalar; 16]) -> Self {
        Self { f_mat: *c }
    }

    #[allow(non_snake_case)]
    pub fn Rows(r0: V4, r1: V4, r2: V4, r3: V4) -> Self {
        let mut m = Self::uninitialized();
        m.set_row(0, r0);
        m.set_row(1, r1);
        m.set_row(2, r2);
        m.set_row(3, r3);
        m
    }

    #[allow(non_snake_case)]
    pub fn Cols(c0: V4, c1: V4, c2: V4, c3: V4) -> Self {
        let mut m = Self::uninitialized();
        m.set_col(0, c0);
        m.set_col(1, c1);
        m.set_col(2, c2);
        m.set_col(3, c3);
        m
    }

    #[allow(non_snake_case)]
    pub fn Translate(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::from_row_major(
            1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn translate_3d(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::Translate(x, y, z)
    }

    pub fn translate_2d(x: Scalar, y: Scalar) -> Self {
        Self::Translate(x, y, 0.0)
    }

    #[allow(non_snake_case)]
    pub fn Scale(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::from_row_major(
            x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn scale_3d(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self::Scale(x, y, z)
    }

    pub fn scale_2d(x: Scalar, y: Scalar) -> Self {
        Self::Scale(x, y, 1.0)
    }

    #[allow(non_snake_case)]
    pub fn Rotate(axis: V3, radians: Scalar) -> Self {
        let mut m = Self::new();
        m.set_rotate(axis, radians);
        m
    }

    pub fn rect_to_rect(src: &Rect, dst: &Rect) -> Self {
        if src.is_empty() {
            return Self::new();
        }
        if dst.is_empty() {
            return Self::Scale(0.0, 0.0, 0.0);
        }
        let sx = dst.width() / src.width();
        let sy = dst.height() / src.height();
        let tx = dst.left() - sx * src.left();
        let ty = dst.top() - sy * src.top();
        Self::from_row_major(
            sx, 0.0, 0.0, tx, 0.0, sy, 0.0, ty, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }

    #[allow(non_snake_case)]
    pub fn RectToRect(src: &Rect, dst: &Rect) -> Self {
        Self::rect_to_rect(src, dst)
    }

    pub fn look_at(eye: V3, center: V3, up: V3) -> Self {
        let f = normalize_v3(center - eye);
        let u = normalize_v3(up);
        let s = normalize_v3(f.cross(&u));
        let mut m = Self::Cols(v4(s, 0.0), v4(s.cross(&f), 0.0), v4(-f, 0.0), v4(eye, 1.0));
        if m.invert_in_place() { m } else { Self::new() }
    }

    #[allow(non_snake_case)]
    pub fn LookAt(eye: V3, center: V3, up: V3) -> Self {
        Self::look_at(eye, center, up)
    }

    pub fn perspective(near: Scalar, far: Scalar, angle: Scalar) -> Self {
        debug_assert!(far > near);
        let denom_inv = ieee_float_divide(1.0, far - near);
        let half_angle = angle * SCALAR_HALF;
        let cot = ieee_float_divide(1.0, scalar_tan(half_angle));
        let mut m = Self::new();
        m.set_rc(0, 0, cot);
        m.set_rc(1, 1, cot);
        m.set_rc(2, 2, (far + near) * denom_inv);
        m.set_rc(2, 3, 2.0 * far * near * denom_inv);
        m.set_rc(3, 2, -1.0);
        m
    }

    #[allow(non_snake_case)]
    pub fn Perspective(near: Scalar, far: Scalar, angle: Scalar) -> Self {
        Self::perspective(near, far, angle)
    }

    pub fn get_col_major(&self) -> [Scalar; 16] {
        self.f_mat
    }

    pub fn get_row_major(&self) -> [Scalar; 16] {
        let mut result = [0.0; 16];
        for row in 0..4 {
            for col in 0..4 {
                result[row * 4 + col] = self.f_mat[col * 4 + row];
            }
        }
        result
    }

    pub fn rc(&self, row: usize, col: usize) -> Scalar {
        self.f_mat[col * 4 + row]
    }

    pub fn set_rc(&mut self, row: usize, col: usize, value: Scalar) {
        self.f_mat[col * 4 + row] = value;
    }

    pub fn row(&self, index: usize) -> V4 {
        V4::new(
            self.f_mat[index],
            self.f_mat[index + 4],
            self.f_mat[index + 8],
            self.f_mat[index + 12],
        )
    }

    pub fn col(&self, index: usize) -> V4 {
        V4::new(
            self.f_mat[index * 4],
            self.f_mat[index * 4 + 1],
            self.f_mat[index * 4 + 2],
            self.f_mat[index * 4 + 3],
        )
    }

    pub fn set_row(&mut self, index: usize, value: V4) {
        self.f_mat[index] = value.x;
        self.f_mat[index + 4] = value.y;
        self.f_mat[index + 8] = value.z;
        self.f_mat[index + 12] = value.w;
    }

    pub fn set_col(&mut self, index: usize, value: V4) {
        self.f_mat[index * 4] = value.x;
        self.f_mat[index * 4 + 1] = value.y;
        self.f_mat[index * 4 + 2] = value.z;
        self.f_mat[index * 4 + 3] = value.w;
    }

    pub fn set_identity(&mut self) -> &mut Self {
        *self = Self::new();
        self
    }

    pub fn set_translate(&mut self, x: Scalar, y: Scalar, z: Scalar) -> &mut Self {
        *self = Self::Translate(x, y, z);
        self
    }

    pub fn set_scale(&mut self, x: Scalar, y: Scalar, z: Scalar) -> &mut Self {
        *self = Self::Scale(x, y, z);
        self
    }

    pub fn set_rotate_unit_sin_cos(
        &mut self,
        axis: V3,
        sin_angle: Scalar,
        cos_angle: Scalar,
    ) -> &mut Self {
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;
        let c = cos_angle;
        let s = sin_angle;
        let t = 1.0 - c;
        *self = Self::from_row_major(
            t * x * x + c,
            t * x * y - s * z,
            t * x * z + s * y,
            0.0,
            t * x * y + s * z,
            t * y * y + c,
            t * y * z - s * x,
            0.0,
            t * x * z - s * y,
            t * y * z + s * x,
            t * z * z + c,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self
    }

    pub fn set_rotate_unit(&mut self, axis: V3, radians: Scalar) -> &mut Self {
        self.set_rotate_unit_sin_cos(axis, scalar_sin(radians), scalar_cos(radians))
    }

    pub fn set_rotate(&mut self, axis: V3, radians: Scalar) -> &mut Self {
        let len = axis.length();
        if len > 0.0 && is_finite(len) {
            self.set_rotate_unit(axis * (SCALAR_1 / len), radians)
        } else {
            self.set_identity()
        }
    }

    pub fn set_concat(&mut self, a: &M44, b: &M44) -> &mut Self {
        let a0 = a.col(0);
        let a1 = a.col(1);
        let a2 = a.col(2);
        let a3 = a.col(3);
        let mut result = Self::uninitialized();
        for index in 0..4 {
            let col = b.col(index);
            let value = a0 * col.x + (a1 * col.y + (a2 * col.z + a3 * col.w));
            result.set_col(index, value);
        }
        *self = result;
        self
    }

    pub fn pre_concat(&mut self, m: &M44) -> &mut Self {
        let result = Self::concat(self, m);
        *self = result;
        self
    }

    pub fn post_concat(&mut self, m: &M44) -> &mut Self {
        let result = Self::concat(m, self);
        *self = result;
        self
    }

    pub fn pre_concat_matrix(&mut self, m: &Matrix) -> &mut Self {
        let b = m.get9_array();
        let c0 = self.col(0);
        let c1 = self.col(1);
        let c3 = self.col(3);
        let compute = |r0: Scalar, r1: Scalar, r3: Scalar| c0 * r0 + (c1 * r1 + c3 * r3);
        self.set_col(0, compute(b[0], b[3], b[6]));
        self.set_col(1, compute(b[1], b[4], b[7]));
        self.set_col(3, compute(b[2], b[5], b[8]));
        self
    }

    pub fn pre_translate_2d(&mut self, x: Scalar, y: Scalar) -> &mut Self {
        self.pre_translate(x, y, 0.0)
    }

    pub fn pre_translate(&mut self, x: Scalar, y: Scalar, z: Scalar) -> &mut Self {
        let c0 = self.col(0);
        let c1 = self.col(1);
        let c2 = self.col(2);
        let c3 = self.col(3);
        self.set_col(3, c0 * x + (c1 * y + (c2 * z + c3)));
        self
    }

    pub fn post_translate_2d(&mut self, x: Scalar, y: Scalar) -> &mut Self {
        self.post_translate(x, y, 0.0)
    }

    pub fn post_translate(&mut self, x: Scalar, y: Scalar, z: Scalar) -> &mut Self {
        let t = V4::new(x, y, z, 0.0);
        for index in 0..4 {
            let column = self.col(index);
            let scale = self.f_mat[index * 4 + 3];
            self.set_col(index, t * scale + column);
        }
        self
    }

    pub fn pre_scale(&mut self, x: Scalar, y: Scalar, z: Scalar) -> &mut Self {
        let c0 = self.col(0);
        let c1 = self.col(1);
        let c2 = self.col(2);
        self.set_col(0, c0 * x);
        self.set_col(1, c1 * y);
        self.set_col(2, c2 * z);
        self
    }

    pub fn pre_scale_2d(&mut self, x: Scalar, y: Scalar) -> &mut Self {
        let c0 = self.col(0);
        let c1 = self.col(1);
        self.set_col(0, c0 * x);
        self.set_col(1, c1 * y);
        self
    }

    pub fn normalize_perspective(&mut self) -> &mut Self {
        if self.f_mat[15] != 1.0
            && self.f_mat[15] != 0.0
            && self.f_mat[3] == 0.0
            && self.f_mat[7] == 0.0
            && self.f_mat[11] == 0.0
        {
            let inv = 1.0 / self.f_mat[15] as f64;
            for value in self.f_mat.iter_mut() {
                *value = (*value as f64 * inv) as Scalar;
            }
            self.f_mat[15] = 1.0;
        }
        self
    }

    pub fn is_finite(&self) -> bool {
        self.f_mat.iter().all(|value| is_finite(*value))
    }

    pub fn invert(&self, inverse: Option<&mut M44>) -> bool {
        let mut tmp = [0.0; 16];
        if invert_4x4_matrix(&self.f_mat, &mut tmp) == 0.0 {
            return false;
        }
        if let Some(inverse) = inverse {
            inverse.f_mat = tmp;
        }
        true
    }

    pub fn invert_in_place(&mut self) -> bool {
        let mut tmp = [0.0; 16];
        if invert_4x4_matrix(&self.f_mat, &mut tmp) == 0.0 {
            return false;
        }
        self.f_mat = tmp;
        true
    }

    pub fn inverted(&self) -> Option<Self> {
        let mut result = Self::new();
        self.invert(Some(&mut result)).then_some(result)
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::uninitialized();
        for row in 0..4 {
            for col in 0..4 {
                result.f_mat[col * 4 + row] = self.f_mat[row * 4 + col];
            }
        }
        result
    }

    pub fn map(&self, x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> V4 {
        let c0 = self.col(0);
        let c1 = self.col(1);
        let c2 = self.col(2);
        let c3 = self.col(3);
        c0 * x + (c1 * y + (c2 * z + c3 * w))
    }

    pub fn map2d(&self, x: Scalar, y: Scalar) -> Point {
        let mapped = self.map(x, y, 0.0, 1.0);
        Point::new(mapped.x, mapped.y)
    }

    pub fn as_m33(&self) -> Matrix {
        Matrix::make_all(
            self.f_mat[0],
            self.f_mat[4],
            self.f_mat[12],
            self.f_mat[1],
            self.f_mat[5],
            self.f_mat[13],
            self.f_mat[3],
            self.f_mat[7],
            self.f_mat[15],
        )
    }

    pub fn has_perspective(&self) -> bool {
        self.f_mat[3] != 0.0
            || self.f_mat[7] != 0.0
            || self.f_mat[11] != 0.0
            || self.f_mat[15] != 1.0
    }

    pub fn map_rect(&self, src: &Rect) -> Rect {
        if self.has_perspective() {
            map_rect_perspective(self, src)
        } else {
            map_rect_affine(self, src)
        }
    }

    pub fn dump(&self) {
        for row in 0..4 {
            eprintln!(
                "|{} {} {} {}|",
                self.f_mat[row],
                self.f_mat[row + 4],
                self.f_mat[row + 8],
                self.f_mat[row + 12]
            );
        }
    }
}

impl From<&Matrix> for M44 {
    fn from(src: &Matrix) -> Self {
        let m = src.get9_array();
        Self::from_row_major(
            m[0], m[1], 0.0, m[2], m[3], m[4], 0.0, m[5], 0.0, 0.0, 1.0, 0.0, m[6], m[7], 0.0, m[8],
        )
    }
}

impl From<Matrix> for M44 {
    fn from(src: Matrix) -> Self {
        Self::from(&src)
    }
}

impl Mul<M44> for M44 {
    type Output = M44;
    fn mul(self, rhs: M44) -> Self::Output {
        Self::concat(&self, &rhs)
    }
}

impl Mul<V4> for M44 {
    type Output = V4;
    fn mul(self, rhs: V4) -> Self::Output {
        self.map(rhs.x, rhs.y, rhs.z, rhs.w)
    }
}

impl Mul<V3> for M44 {
    type Output = V3;
    fn mul(self, rhs: V3) -> Self::Output {
        let mapped = self.map(rhs.x, rhs.y, rhs.z, 0.0);
        V3::new(mapped.x, mapped.y, mapped.z)
    }
}

impl core::ops::Index<usize> for M44 {
    type Output = Scalar;
    fn index(&self, index: usize) -> &Self::Output {
        &self.f_mat[index]
    }
}

impl core::ops::IndexMut<usize> for M44 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.f_mat[index]
    }
}

fn map_rect_affine(m: &M44, src: &Rect) -> Rect {
    let corners = [
        m.map2d(src.left(), src.top()),
        m.map2d(src.right(), src.top()),
        m.map2d(src.left(), src.bottom()),
        m.map2d(src.right(), src.bottom()),
    ];
    let mut min_x = corners[0].x;
    let mut max_x = corners[0].x;
    let mut min_y = corners[0].y;
    let mut max_y = corners[0].y;
    for point in corners.iter().skip(1) {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    Rect::new(min_x, min_y, max_x, max_y)
}

fn map_rect_perspective(m: &M44, src: &Rect) -> Rect {
    let c0 = m.col(0);
    let c1 = m.col(1);
    let c3 = m.col(3);
    let corner = |x: Scalar, y: Scalar| c0 * x + c1 * y + c3;

    let tl = corner(src.left(), src.top());
    let tr = corner(src.right(), src.top());
    let bl = corner(src.left(), src.bottom());
    let br = corner(src.right(), src.bottom());

    let flip = V4::new(1.0, 1.0, -1.0, -1.0);
    let project = |p0: V4, p1: V4, p2: V4| -> V4 {
        let w0 = p0.w;
        if w0 >= KW0_PLANE_DISTANCE {
            flip * V4::new(p0.x / w0, p0.y / w0, p0.x / w0, p0.y / w0)
        } else {
            let clip = |p: V4| -> V4 {
                let w = p.w;
                if w >= KW0_PLANE_DISTANCE {
                    let t = (KW0_PLANE_DISTANCE - w0) / (w - w0);
                    let cx = (t * p.x + (1.0 - t) * p0.x) / KW0_PLANE_DISTANCE;
                    let cy = (t * p.y + (1.0 - t) * p0.y) / KW0_PLANE_DISTANCE;
                    flip * V4::new(cx, cy, cx, cy)
                } else {
                    V4::new(
                        Scalar::INFINITY,
                        Scalar::INFINITY,
                        Scalar::INFINITY,
                        Scalar::INFINITY,
                    )
                }
            };
            min_v4(clip(p1), clip(p2))
        }
    };

    let min_max = flip
        * min_v4(
            min_v4(project(tl, tr, bl), project(tr, br, tl)),
            min_v4(project(br, bl, tr), project(bl, tl, br)),
        );

    // The mapping of the min/max components follows the reference's vector layout.
    Rect::new(min_max.x, min_max.y, min_max.z, min_max.w)
}

fn min_v4(a: V4, b: V4) -> V4 {
    V4::new(
        cpp_min(a.x, b.x),
        cpp_min(a.y, b.y),
        cpp_min(a.z, b.z),
        cpp_min(a.w, b.w),
    )
}

fn cpp_min(a: Scalar, b: Scalar) -> Scalar {
    if b < a { b } else { a }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_m44_identity_and_row_major() {
        let m = M44::new();
        let r = m.get_row_major();
        assert_eq!(r, M44::new().get_row_major());
        let mut values = [0.0; 16];
        for (index, value) in values.iter_mut().enumerate() {
            *value = index as Scalar;
        }
        let row_major = M44::from_row_major_array(&values);
        assert_eq!(row_major.rc(0, 0), 0.0);
        assert_eq!(row_major.rc(0, 1), 1.0);
        assert_eq!(row_major.rc(1, 0), 4.0);
    }

    #[test]
    fn dev_m44_rotate_translate_and_map() {
        let r = M44::Rotate(V3::new(0.0, 0.0, 1.0), core::f32::consts::FRAC_PI_2);
        let p = r * V3::new(1.0, 0.0, 0.0);
        assert!((p.x - 0.0).abs() < 1.0e-6);
        assert!((p.y - 1.0).abs() < 1.0e-6);

        let mut m = M44::new();
        m.pre_translate(3.0, 4.0, 2.0);
        assert_eq!(m.map(0.0, 0.0, 0.0, 1.0), V4::new(3.0, 4.0, 2.0, 1.0));
    }
}
