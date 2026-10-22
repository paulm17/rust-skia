use core::ops::Mul;

use crate::float_bits::scalar_as_2s_compliment;
use crate::floating_point::{ieee_double_divide, ieee_float_divide, is_finite, is_finite2};
use crate::m44::M44;
use crate::point::Point;
use crate::point3::Point3;
use crate::rect::Rect;
use crate::scalar::{
    SCALAR_MAX, SCALAR_NEARLY_ZERO, Scalar, degrees_to_radians, scalar_abs,
    scalar_cos_snap_to_zero, scalar_nearly_equal_within, scalar_nearly_zero,
    scalar_nearly_zero_within, scalar_sin_snap_to_zero,
};
use crate::size::Size;

pub const K_M_SCALE_X: usize = 0;
pub const K_M_SKEW_X: usize = 1;
pub const K_M_TRANS_X: usize = 2;
pub const K_M_SKEW_Y: usize = 3;
pub const K_M_SCALE_Y: usize = 4;
pub const K_M_TRANS_Y: usize = 5;
pub const K_M_PERSP_0: usize = 6;
pub const K_M_PERSP_1: usize = 7;
pub const K_M_PERSP_2: usize = 8;

pub const K_A_SCALE_X: usize = 0;
pub const K_A_SKEW_Y: usize = 1;
pub const K_A_SKEW_X: usize = 2;
pub const K_A_SCALE_Y: usize = 3;
pub const K_A_TRANS_X: usize = 4;
pub const K_A_TRANS_Y: usize = 5;

pub const K_IDENTITY_MASK: u32 = 0x00;
pub const K_TRANSLATE_MASK: u32 = 0x01;
pub const K_SCALE_MASK: u32 = 0x02;
pub const K_AFFINE_MASK: u32 = 0x04;
pub const K_PERSPECTIVE_MASK: u32 = 0x08;
pub const K_RECT_STAYS_RECT_MASK: u32 = 0x10;
pub const K_ONLY_PERSPECTIVE_VALID_MASK: u32 = 0x40;
pub const K_UNKNOWN_MASK: u32 = 0x80;
pub const K_ALL_MASKS: u32 = 0x0F;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleToFit {
    Fill,
    Start,
    Center,
    End,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Matrix {
    f_mat: [Scalar; 9],
    f_type_mask: u32,
}

pub type SkMatrix = Matrix;

const _: () = {
    assert!(core::mem::size_of::<Matrix>() == 9 * core::mem::size_of::<Scalar>() + 4);
};

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Matrix {
    pub const IDENTITY_MASK: u32 = K_IDENTITY_MASK;
    pub const TRANSLATE_MASK: u32 = K_TRANSLATE_MASK;
    pub const SCALE_MASK: u32 = K_SCALE_MASK;
    pub const AFFINE_MASK: u32 = K_AFFINE_MASK;
    pub const PERSPECTIVE_MASK: u32 = K_PERSPECTIVE_MASK;
    pub const RECT_STAYS_RECT_MASK: u32 = K_RECT_STAYS_RECT_MASK;
    pub const ONLY_PERSPECTIVE_VALID_MASK: u32 = K_ONLY_PERSPECTIVE_VALID_MASK;
    pub const UNKNOWN_MASK: u32 = K_UNKNOWN_MASK;
    pub const ALL_MASKS: u32 = K_ALL_MASKS;

    pub const M_SCALE_X: usize = K_M_SCALE_X;
    pub const M_SKEW_X: usize = K_M_SKEW_X;
    pub const M_TRANS_X: usize = K_M_TRANS_X;
    pub const M_SKEW_Y: usize = K_M_SKEW_Y;
    pub const M_SCALE_Y: usize = K_M_SCALE_Y;
    pub const M_TRANS_Y: usize = K_M_TRANS_Y;
    pub const M_PERSP_0: usize = K_M_PERSP_0;
    pub const M_PERSP_1: usize = K_M_PERSP_1;
    pub const M_PERSP_2: usize = K_M_PERSP_2;

    pub const A_SCALE_X: usize = K_A_SCALE_X;
    pub const A_SKEW_Y: usize = K_A_SKEW_Y;
    pub const A_SKEW_X: usize = K_A_SKEW_X;
    pub const A_SCALE_Y: usize = K_A_SCALE_Y;
    pub const A_TRANS_X: usize = K_A_TRANS_X;
    pub const A_TRANS_Y: usize = K_A_TRANS_Y;

    pub const fn new() -> Self {
        Self {
            f_mat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            f_type_mask: K_RECT_STAYS_RECT_MASK,
        }
    }

    pub const fn identity() -> Self {
        Self::new()
    }

    #[allow(non_snake_case)]
    pub const fn I() -> Self {
        Self::new()
    }

    pub const fn invalid_matrix() -> Self {
        Self {
            f_mat: [SCALAR_MAX; 9],
            f_type_mask: K_TRANSLATE_MASK | K_SCALE_MASK | K_AFFINE_MASK | K_PERSPECTIVE_MASK,
        }
    }

    #[allow(non_snake_case)]
    pub const fn InvalidMatrix() -> Self {
        Self::invalid_matrix()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn make_all(
        scale_x: Scalar,
        skew_x: Scalar,
        trans_x: Scalar,
        skew_y: Scalar,
        scale_y: Scalar,
        trans_y: Scalar,
        persp_0: Scalar,
        persp_1: Scalar,
        persp_2: Scalar,
    ) -> Self {
        Self {
            f_mat: [
                scale_x, skew_x, trans_x, skew_y, scale_y, trans_y, persp_0, persp_1, persp_2,
            ],
            f_type_mask: K_UNKNOWN_MASK,
        }
    }

    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub fn MakeAll(
        scale_x: Scalar,
        skew_x: Scalar,
        trans_x: Scalar,
        skew_y: Scalar,
        scale_y: Scalar,
        trans_y: Scalar,
        persp_0: Scalar,
        persp_1: Scalar,
        persp_2: Scalar,
    ) -> Self {
        Self::make_all(
            scale_x, skew_x, trans_x, skew_y, scale_y, trans_y, persp_0, persp_1, persp_2,
        )
    }

    #[allow(non_snake_case)]
    pub fn Scale(sx: Scalar, sy: Scalar) -> Self {
        Self::scale(sx, sy)
    }

    pub fn scale(sx: Scalar, sy: Scalar) -> Self {
        if sx == 1.0 && sy == 1.0 {
            return Self::new();
        }
        Self::scale_translate(sx, sy, 0.0, 0.0)
    }

    #[allow(non_snake_case)]
    pub fn Translate(dx: Scalar, dy: Scalar) -> Self {
        Self::translate(dx, dy)
    }

    pub fn translate(dx: Scalar, dy: Scalar) -> Self {
        let mut m = Self::new();
        m.set_translate(dx, dy);
        m
    }

    #[allow(non_snake_case)]
    pub fn TranslateVector(v: Point) -> Self {
        Self::translate(v.x, v.y)
    }

    #[allow(non_snake_case)]
    pub fn ScaleTranslate(sx: Scalar, sy: Scalar, tx: Scalar, ty: Scalar) -> Self {
        Self::scale_translate(sx, sy, tx, ty)
    }

    pub fn scale_translate(sx: Scalar, sy: Scalar, tx: Scalar, ty: Scalar) -> Self {
        let mut mask = 0;
        if sx != 1.0 || sy != 1.0 {
            mask |= K_SCALE_MASK;
        }
        if tx != 0.0 || ty != 0.0 {
            mask |= K_TRANSLATE_MASK;
        }
        if sx != 0.0 && sy != 0.0 {
            mask |= K_RECT_STAYS_RECT_MASK;
        }
        Self {
            f_mat: [sx, 0.0, tx, 0.0, sy, ty, 0.0, 0.0, 1.0],
            f_type_mask: mask,
        }
    }

    #[allow(non_snake_case)]
    pub fn RotateDeg(degrees: Scalar) -> Self {
        Self::rotate_deg(degrees)
    }

    pub fn rotate_deg(degrees: Scalar) -> Self {
        let mut m = Self::new();
        m.set_rotate(degrees);
        m
    }

    pub fn rotate_deg_pivot(degrees: Scalar, px: Scalar, py: Scalar) -> Self {
        let mut m = Self::new();
        m.set_rotate_pivot(degrees, px, py);
        m
    }

    pub fn rotate_rad(radians: Scalar) -> Self {
        let mut m = Self::new();
        m.set_sin_cos(radians.sin(), radians.cos());
        m
    }

    #[allow(non_snake_case)]
    pub fn Skew(sx: Scalar, sy: Scalar) -> Self {
        let mut m = Self::new();
        m.set_skew(sx, sy);
        m
    }

    pub fn skew(sx: Scalar, sy: Scalar) -> Self {
        Self::Skew(sx, sy)
    }

    pub fn get_type(&self) -> u32 {
        if self.f_type_mask & K_UNKNOWN_MASK != 0 {
            let mask = compute_type_mask(self);
            mask & 0x0F
        } else {
            self.f_type_mask & 0x0F
        }
    }

    pub fn get_full_type(&self) -> u32 {
        if self.f_type_mask & K_UNKNOWN_MASK != 0 {
            compute_type_mask(self)
        } else {
            self.f_type_mask
        }
    }

    pub fn set_type_mask(&mut self, mask: u32) {
        self.f_type_mask = mask;
    }

    pub fn clear_type_mask(&mut self, mask: u32) {
        self.f_type_mask &= !mask;
    }

    pub fn is_identity(&self) -> bool {
        self.get_type() == K_IDENTITY_MASK
    }

    #[allow(non_snake_case)]
    pub fn isIdentity(&self) -> bool {
        self.is_identity()
    }

    pub fn is_scale_translate(&self) -> bool {
        self.get_type() & !(K_SCALE_MASK | K_TRANSLATE_MASK) == 0
    }

    #[allow(non_snake_case)]
    pub fn isScaleTranslate(&self) -> bool {
        self.is_scale_translate()
    }

    pub fn is_translate(&self) -> bool {
        self.get_type() & !K_TRANSLATE_MASK == 0
    }

    #[allow(non_snake_case)]
    pub fn isTranslate(&self) -> bool {
        self.is_translate()
    }

    pub fn rect_stays_rect(&self) -> bool {
        self.get_full_type() & K_RECT_STAYS_RECT_MASK != 0
    }

    #[allow(non_snake_case)]
    pub fn rectStaysRect(&self) -> bool {
        self.rect_stays_rect()
    }

    pub fn preserves_axis_alignment(&self) -> bool {
        self.get_type() & (K_AFFINE_MASK | K_PERSPECTIVE_MASK) == 0
    }

    #[allow(non_snake_case)]
    pub fn preservesAxisAlignment(&self) -> bool {
        self.preserves_axis_alignment()
    }

    pub fn has_perspective(&self) -> bool {
        self.get_type() & K_PERSPECTIVE_MASK != 0
    }

    #[allow(non_snake_case)]
    pub fn hasPerspective(&self) -> bool {
        self.has_perspective()
    }

    pub fn get(&self, index: usize) -> Scalar {
        self.f_mat[index]
    }

    pub fn rc(&self, row: usize, col: usize) -> Scalar {
        self.f_mat[row * 3 + col]
    }

    pub fn get9_array(&self) -> [Scalar; 9] {
        self.f_mat
    }

    pub fn get9(&self, result: &mut [Scalar; 9]) {
        *result = self.f_mat;
    }

    pub fn scale_x(&self) -> Scalar {
        self.f_mat[K_M_SCALE_X]
    }

    pub fn skew_x(&self) -> Scalar {
        self.f_mat[K_M_SKEW_X]
    }

    pub fn trans_x(&self) -> Scalar {
        self.f_mat[K_M_TRANS_X]
    }

    pub fn skew_y(&self) -> Scalar {
        self.f_mat[K_M_SKEW_Y]
    }

    pub fn scale_y(&self) -> Scalar {
        self.f_mat[K_M_SCALE_Y]
    }

    pub fn trans_y(&self) -> Scalar {
        self.f_mat[K_M_TRANS_Y]
    }

    pub fn persp_x(&self) -> Scalar {
        self.f_mat[K_M_PERSP_0]
    }

    pub fn persp_y(&self) -> Scalar {
        self.f_mat[K_M_PERSP_1]
    }

    pub fn persp_z(&self) -> Scalar {
        self.f_mat[K_M_PERSP_2]
    }

    #[allow(non_snake_case)]
    pub fn getScaleX(&self) -> Scalar {
        self.scale_x()
    }
    #[allow(non_snake_case)]
    pub fn getSkewX(&self) -> Scalar {
        self.skew_x()
    }
    #[allow(non_snake_case)]
    pub fn getTranslateX(&self) -> Scalar {
        self.trans_x()
    }
    #[allow(non_snake_case)]
    pub fn getTranslateY(&self) -> Scalar {
        self.trans_y()
    }
    #[allow(non_snake_case)]
    pub fn getSkewY(&self) -> Scalar {
        self.skew_y()
    }
    #[allow(non_snake_case)]
    pub fn getScaleY(&self) -> Scalar {
        self.scale_y()
    }
    #[allow(non_snake_case)]
    pub fn getPerspX(&self) -> Scalar {
        self.persp_x()
    }
    #[allow(non_snake_case)]
    pub fn getPerspY(&self) -> Scalar {
        self.persp_y()
    }

    pub fn set(&mut self, index: usize, value: Scalar) -> &mut Self {
        self.f_mat[index] = value;
        self.f_type_mask = K_UNKNOWN_MASK;
        self
    }

    pub fn set_scale_x(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_SCALE_X, value)
    }
    pub fn set_skew_x(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_SKEW_X, value)
    }
    pub fn set_trans_x(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_TRANS_X, value)
    }
    pub fn set_skew_y(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_SKEW_Y, value)
    }
    pub fn set_scale_y(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_SCALE_Y, value)
    }
    pub fn set_trans_y(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_TRANS_Y, value)
    }
    pub fn set_persp_x(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_PERSP_0, value)
    }
    pub fn set_persp_y(&mut self, value: Scalar) -> &mut Self {
        self.set(K_M_PERSP_1, value)
    }

    #[allow(non_snake_case)]
    pub fn setScaleX(&mut self, value: Scalar) -> &mut Self {
        self.set_scale_x(value)
    }
    #[allow(non_snake_case)]
    pub fn setSkewX(&mut self, value: Scalar) -> &mut Self {
        self.set_skew_x(value)
    }
    #[allow(non_snake_case)]
    pub fn setTranslateX(&mut self, value: Scalar) -> &mut Self {
        self.set_trans_x(value)
    }
    #[allow(non_snake_case)]
    pub fn setSkewY(&mut self, value: Scalar) -> &mut Self {
        self.set_skew_y(value)
    }
    #[allow(non_snake_case)]
    pub fn setScaleY(&mut self, value: Scalar) -> &mut Self {
        self.set_scale_y(value)
    }
    #[allow(non_snake_case)]
    pub fn setTranslateY(&mut self, value: Scalar) -> &mut Self {
        self.set_trans_y(value)
    }
    #[allow(non_snake_case)]
    pub fn setPerspX(&mut self, value: Scalar) -> &mut Self {
        self.set_persp_x(value)
    }
    #[allow(non_snake_case)]
    pub fn setPerspY(&mut self, value: Scalar) -> &mut Self {
        self.set_persp_y(value)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_all(
        &mut self,
        scale_x: Scalar,
        skew_x: Scalar,
        trans_x: Scalar,
        skew_y: Scalar,
        scale_y: Scalar,
        trans_y: Scalar,
        persp_0: Scalar,
        persp_1: Scalar,
        persp_2: Scalar,
    ) -> &mut Self {
        *self = Self::make_all(
            scale_x, skew_x, trans_x, skew_y, scale_y, trans_y, persp_0, persp_1, persp_2,
        );
        self
    }

    pub fn set9(&mut self, values: &[Scalar; 9]) -> &mut Self {
        self.f_mat = *values;
        self.f_type_mask = K_UNKNOWN_MASK;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new();
        self
    }

    pub fn set_identity(&mut self) -> &mut Self {
        self.reset()
    }

    pub fn set_translate(&mut self, dx: Scalar, dy: Scalar) -> &mut Self {
        self.f_mat = [1.0, 0.0, dx, 0.0, 1.0, dy, 0.0, 0.0, 1.0];
        self.f_type_mask = if dx != 0.0 || dy != 0.0 {
            K_TRANSLATE_MASK | K_RECT_STAYS_RECT_MASK
        } else {
            K_IDENTITY_MASK | K_RECT_STAYS_RECT_MASK
        };
        self
    }

    pub fn pre_translate(&mut self, dx: Scalar, dy: Scalar) -> &mut Self {
        if dx == 0.0 && dy == 0.0 {
            return self;
        }
        let mask = self.get_type();
        if mask == K_IDENTITY_MASK {
            self.set_translate(dx, dy);
        } else if mask == K_TRANSLATE_MASK {
            self.f_mat[K_M_TRANS_X] += dx;
            self.f_mat[K_M_TRANS_Y] += dy;
        } else {
            self.f_mat[K_M_TRANS_X] += dx * self.f_mat[K_M_SCALE_X] + dy * self.f_mat[K_M_SKEW_X];
            self.f_mat[K_M_TRANS_Y] += dx * self.f_mat[K_M_SKEW_Y] + dy * self.f_mat[K_M_SCALE_Y];
            self.f_type_mask = K_UNKNOWN_MASK;
        }
        self
    }

    pub fn post_translate(&mut self, dx: Scalar, dy: Scalar) -> &mut Self {
        if dx == 0.0 && dy == 0.0 {
            return self;
        }
        if self.has_perspective() {
            let mut m = Self::new();
            m.set_translate(dx, dy);
            self.post_concat(&m);
        } else {
            self.f_mat[K_M_TRANS_X] += dx;
            self.f_mat[K_M_TRANS_Y] += dy;
            self.update_translate_mask();
        }
        self
    }

    fn update_translate_mask(&mut self) {
        if self.f_mat[K_M_TRANS_X] != 0.0 || self.f_mat[K_M_TRANS_Y] != 0.0 {
            self.f_type_mask |= K_TRANSLATE_MASK;
        } else {
            self.f_type_mask &= !K_TRANSLATE_MASK;
        }
    }

    pub fn set_scale(&mut self, sx: Scalar, sy: Scalar) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            self.reset();
        } else {
            self.set_scale_translate(sx, sy, 0.0, 0.0);
        }
        self
    }

    pub fn set_scale_pivot(&mut self, sx: Scalar, sy: Scalar, px: Scalar, py: Scalar) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            self.reset();
        } else {
            self.set_scale_translate(sx, sy, px - sx * px, py - sy * py);
        }
        self
    }

    pub fn set_scale_translate(
        &mut self,
        sx: Scalar,
        sy: Scalar,
        tx: Scalar,
        ty: Scalar,
    ) -> &mut Self {
        *self = Self::scale_translate(sx, sy, tx, ty);
        self
    }

    pub fn pre_scale(&mut self, sx: Scalar, sy: Scalar) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            return self;
        }
        self.f_mat[K_M_SCALE_X] *= sx;
        self.f_mat[K_M_SKEW_Y] *= sx;
        self.f_mat[K_M_PERSP_0] *= sx;
        self.f_mat[K_M_SKEW_X] *= sy;
        self.f_mat[K_M_SCALE_Y] *= sy;
        self.f_mat[K_M_PERSP_1] *= sy;
        if sx == 1.0 && sy == 1.0 {
            self.f_type_mask &= !K_SCALE_MASK;
        }
        if sx == 0.0 || sy == 0.0 {
            self.f_type_mask &= !K_RECT_STAYS_RECT_MASK;
        }
        self.f_type_mask = K_UNKNOWN_MASK;
        self
    }

    pub fn post_scale(&mut self, sx: Scalar, sy: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_scale(sx, sy);
        self.post_concat(&m)
    }

    pub fn set_sin_cos(&mut self, sin_v: Scalar, cos_v: Scalar) -> &mut Self {
        self.f_mat = [cos_v, -sin_v, 0.0, sin_v, cos_v, 0.0, 0.0, 0.0, 1.0];
        self.f_type_mask = K_UNKNOWN_MASK | K_ONLY_PERSPECTIVE_VALID_MASK;
        self
    }

    pub fn set_sin_cos_pivot(
        &mut self,
        sin_v: Scalar,
        cos_v: Scalar,
        px: Scalar,
        py: Scalar,
    ) -> &mut Self {
        let one_minus_cos = 1.0 - cos_v;
        self.f_mat = [
            cos_v,
            -sin_v,
            sin_v * py + one_minus_cos * px,
            sin_v,
            cos_v,
            -sin_v * px + one_minus_cos * py,
            0.0,
            0.0,
            1.0,
        ];
        self.f_type_mask = K_UNKNOWN_MASK | K_ONLY_PERSPECTIVE_VALID_MASK;
        self
    }

    pub fn set_rotate(&mut self, degrees: Scalar) -> &mut Self {
        let radians = degrees_to_radians(degrees);
        self.set_sin_cos(
            scalar_sin_snap_to_zero(radians),
            scalar_cos_snap_to_zero(radians),
        )
    }

    pub fn set_rotate_pivot(&mut self, degrees: Scalar, px: Scalar, py: Scalar) -> &mut Self {
        let radians = degrees_to_radians(degrees);
        self.set_sin_cos_pivot(
            scalar_sin_snap_to_zero(radians),
            scalar_cos_snap_to_zero(radians),
            px,
            py,
        )
    }

    pub fn pre_rotate(&mut self, degrees: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_rotate(degrees);
        self.pre_concat(&m)
    }

    pub fn pre_rotate_pivot(&mut self, degrees: Scalar, px: Scalar, py: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_rotate_pivot(degrees, px, py);
        self.pre_concat(&m)
    }

    pub fn post_rotate(&mut self, degrees: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_rotate(degrees);
        self.post_concat(&m)
    }

    pub fn post_rotate_pivot(&mut self, degrees: Scalar, px: Scalar, py: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_rotate_pivot(degrees, px, py);
        self.post_concat(&m)
    }

    pub fn set_skew(&mut self, sx: Scalar, sy: Scalar) -> &mut Self {
        self.f_mat = [1.0, sx, 0.0, sy, 1.0, 0.0, 0.0, 0.0, 1.0];
        self.f_type_mask = K_UNKNOWN_MASK | K_ONLY_PERSPECTIVE_VALID_MASK;
        self
    }

    pub fn set_skew_pivot(&mut self, sx: Scalar, sy: Scalar, px: Scalar, py: Scalar) -> &mut Self {
        self.f_mat = [1.0, sx, -sx * py, sy, 1.0, -sy * px, 0.0, 0.0, 1.0];
        self.f_type_mask = K_UNKNOWN_MASK | K_ONLY_PERSPECTIVE_VALID_MASK;
        self
    }

    pub fn pre_skew(&mut self, sx: Scalar, sy: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_skew(sx, sy);
        self.pre_concat(&m)
    }

    pub fn post_skew(&mut self, sx: Scalar, sy: Scalar) -> &mut Self {
        let mut m = Self::new();
        m.set_skew(sx, sy);
        self.post_concat(&m)
    }

    pub fn set_rs_xform(&mut self, xform: &crate::rsxform::RSXform) -> &mut Self {
        self.f_mat = [
            xform.scos(),
            -xform.ssin(),
            xform.tx(),
            xform.ssin(),
            xform.scos(),
            xform.ty(),
            0.0,
            0.0,
            1.0,
        ];
        self.f_type_mask = K_UNKNOWN_MASK | K_ONLY_PERSPECTIVE_VALID_MASK;
        self
    }

    pub fn set_concat(&mut self, a: &Matrix, b: &Matrix) -> &mut Self {
        if a.is_identity() {
            *self = *b;
        } else if b.is_identity() {
            *self = *a;
        } else {
            let a_type = a.get_type();
            let b_type = b.get_type();
            if (a_type | b_type) & (K_AFFINE_MASK | K_PERSPECTIVE_MASK) == 0 {
                *self = Self::scale_translate(
                    a.f_mat[K_M_SCALE_X] * b.f_mat[K_M_SCALE_X],
                    a.f_mat[K_M_SCALE_Y] * b.f_mat[K_M_SCALE_Y],
                    a.f_mat[K_M_SCALE_X] * b.f_mat[K_M_TRANS_X] + a.f_mat[K_M_TRANS_X],
                    a.f_mat[K_M_SCALE_Y] * b.f_mat[K_M_TRANS_Y] + a.f_mat[K_M_TRANS_Y],
                );
            } else if (a_type | b_type) & K_PERSPECTIVE_MASK != 0 {
                let mut tmp = [0.0; 9];
                for row in 0..3 {
                    for col in 0..3 {
                        tmp[row * 3 + col] = rowcol3(&a.f_mat[row * 3..row * 3 + 3], b, col);
                    }
                }
                self.f_mat = tmp;
                self.f_type_mask = K_UNKNOWN_MASK;
            } else {
                let mut tmp = Self::new();
                tmp.f_mat[K_M_SCALE_X] = muladdmul(
                    a.f_mat[K_M_SCALE_X],
                    b.f_mat[K_M_SCALE_X],
                    a.f_mat[K_M_SKEW_X],
                    b.f_mat[K_M_SKEW_Y],
                );
                tmp.f_mat[K_M_SKEW_X] = muladdmul(
                    a.f_mat[K_M_SCALE_X],
                    b.f_mat[K_M_SKEW_X],
                    a.f_mat[K_M_SKEW_X],
                    b.f_mat[K_M_SCALE_Y],
                );
                tmp.f_mat[K_M_TRANS_X] = muladdmul(
                    a.f_mat[K_M_SCALE_X],
                    b.f_mat[K_M_TRANS_X],
                    a.f_mat[K_M_SKEW_X],
                    b.f_mat[K_M_TRANS_Y],
                ) + a.f_mat[K_M_TRANS_X];
                tmp.f_mat[K_M_SKEW_Y] = muladdmul(
                    a.f_mat[K_M_SKEW_Y],
                    b.f_mat[K_M_SCALE_X],
                    a.f_mat[K_M_SCALE_Y],
                    b.f_mat[K_M_SKEW_Y],
                );
                tmp.f_mat[K_M_SCALE_Y] = muladdmul(
                    a.f_mat[K_M_SKEW_Y],
                    b.f_mat[K_M_SKEW_X],
                    a.f_mat[K_M_SCALE_Y],
                    b.f_mat[K_M_SCALE_Y],
                );
                tmp.f_mat[K_M_TRANS_Y] = muladdmul(
                    a.f_mat[K_M_SKEW_Y],
                    b.f_mat[K_M_TRANS_X],
                    a.f_mat[K_M_SCALE_Y],
                    b.f_mat[K_M_TRANS_Y],
                ) + a.f_mat[K_M_TRANS_Y];
                tmp.f_mat[K_M_PERSP_0] = 0.0;
                tmp.f_mat[K_M_PERSP_1] = 0.0;
                tmp.f_mat[K_M_PERSP_2] = 1.0;
                tmp.f_type_mask = K_UNKNOWN_MASK | K_ONLY_PERSPECTIVE_VALID_MASK;
                *self = tmp;
            }
        }
        self
    }

    pub fn pre_concat(&mut self, m: &Matrix) -> &mut Self {
        if !m.is_identity() {
            let copy = *self;
            self.set_concat(&copy, m);
        }
        self
    }

    pub fn post_concat(&mut self, m: &Matrix) -> &mut Self {
        if !m.is_identity() {
            let copy = *self;
            self.set_concat(m, &copy);
        }
        self
    }

    #[allow(non_snake_case)]
    pub fn Concat(a: &Matrix, b: &Matrix) -> Self {
        let mut result = Self::new();
        result.set_concat(a, b);
        result
    }

    pub fn as_affine(&self, affine: &mut [Scalar; 6]) -> bool {
        if self.has_perspective() {
            return false;
        }
        affine[K_A_SCALE_X] = self.f_mat[K_M_SCALE_X];
        affine[K_A_SKEW_Y] = self.f_mat[K_M_SKEW_Y];
        affine[K_A_SKEW_X] = self.f_mat[K_M_SKEW_X];
        affine[K_A_SCALE_Y] = self.f_mat[K_M_SCALE_Y];
        affine[K_A_TRANS_X] = self.f_mat[K_M_TRANS_X];
        affine[K_A_TRANS_Y] = self.f_mat[K_M_TRANS_Y];
        true
    }

    #[allow(non_snake_case)]
    pub fn asAffine(&self, affine: &mut [Scalar; 6]) -> bool {
        self.as_affine(affine)
    }

    pub fn set_affine(&mut self, affine: &[Scalar; 6]) -> &mut Self {
        self.f_mat = [
            affine[K_A_SCALE_X],
            affine[K_A_SKEW_X],
            affine[K_A_TRANS_X],
            affine[K_A_SKEW_Y],
            affine[K_A_SCALE_Y],
            affine[K_A_TRANS_Y],
            0.0,
            0.0,
            1.0,
        ];
        self.f_type_mask = K_UNKNOWN_MASK;
        self
    }

    pub fn set_affine_identity(affine: &mut [Scalar; 6]) {
        affine[K_A_SCALE_X] = 1.0;
        affine[K_A_SKEW_Y] = 0.0;
        affine[K_A_SKEW_X] = 0.0;
        affine[K_A_SCALE_Y] = 1.0;
        affine[K_A_TRANS_X] = 0.0;
        affine[K_A_TRANS_Y] = 0.0;
    }

    #[allow(non_snake_case)]
    pub fn SetAffineIdentity(affine: &mut [Scalar; 6]) {
        Self::set_affine_identity(affine);
    }

    pub fn normalize_perspective(&mut self) -> &mut Self {
        if self.f_mat[K_M_PERSP_0] == 0.0
            && self.f_mat[K_M_PERSP_1] == 0.0
            && self.f_mat[K_M_PERSP_2] != 0.0
            && self.f_mat[K_M_PERSP_2] != 1.0
        {
            let inv = 1.0 / self.f_mat[K_M_PERSP_2] as f64;
            for value in self.f_mat[..6].iter_mut() {
                *value = (*value as f64 * inv) as Scalar;
            }
            self.f_mat[K_M_PERSP_2] = 1.0;
            self.f_type_mask = K_UNKNOWN_MASK;
        }
        self
    }

    #[allow(non_snake_case)]
    pub fn normalizePerspective(&mut self) -> &mut Self {
        self.normalize_perspective()
    }

    pub fn rect_2_rect(src: &Rect, dst: &Rect) -> Option<Self> {
        Self::rect_2_rect_with(src, dst, ScaleToFit::Fill)
    }

    pub fn rect_2_rect_with(src: &Rect, dst: &Rect, stf: ScaleToFit) -> Option<Self> {
        if src.is_empty() {
            return None;
        }
        let mut sx = ieee_float_divide(dst.width(), src.width());
        let mut sy = ieee_float_divide(dst.height(), src.height());
        let mut x_larger = false;
        if stf != ScaleToFit::Fill {
            if sx > sy {
                x_larger = true;
                sx = sy;
            } else {
                sy = sx;
            }
        }
        let mut tx = dst.left() - src.left() * sx;
        let mut ty = dst.top() - src.top() * sy;
        if stf == ScaleToFit::Center || stf == ScaleToFit::End {
            let mut diff = if x_larger {
                dst.width() - src.width() * sy
            } else {
                dst.height() - src.height() * sy
            };
            if stf == ScaleToFit::Center {
                diff /= 2.0;
            }
            if x_larger {
                tx += diff;
            } else {
                ty += diff;
            }
        }
        Some(Self::scale_translate(sx, sy, tx, ty))
    }

    #[allow(non_snake_case)]
    pub fn Rect2Rect(src: &Rect, dst: &Rect) -> Option<Self> {
        Self::rect_2_rect(src, dst)
    }

    #[allow(non_snake_case)]
    pub fn Rect2RectWith(src: &Rect, dst: &Rect, stf: ScaleToFit) -> Option<Self> {
        Self::rect_2_rect_with(src, dst, stf)
    }

    pub fn rect_to_rect_or_identity(src: &Rect, dst: &Rect) -> Self {
        Self::rect_2_rect(src, dst).unwrap_or_default()
    }

    #[allow(non_snake_case)]
    pub fn RectToRectOrIdentity(src: &Rect, dst: &Rect) -> Self {
        Self::rect_to_rect_or_identity(src, dst)
    }

    #[allow(non_snake_case)]
    pub fn MakeRectToRect(src: &Rect, dst: &Rect) -> Self {
        Self::rect_to_rect_or_identity(src, dst)
    }

    pub fn set_rect_to_rect(&mut self, src: &Rect, dst: &Rect) -> bool {
        match Self::rect_2_rect(src, dst) {
            Some(matrix) => {
                *self = matrix;
                true
            }
            None => {
                self.reset();
                false
            }
        }
    }

    pub fn invert(&self, inverse: Option<&mut Matrix>) -> bool {
        let mask = self.get_type();
        let result = if mask == K_IDENTITY_MASK {
            Some(*self)
        } else if mask & !(K_SCALE_MASK | K_TRANSLATE_MASK) == 0 {
            if mask & K_SCALE_MASK != 0 {
                let inv_sx = ieee_float_divide(1.0, self.f_mat[K_M_SCALE_X]);
                let inv_sy = ieee_float_divide(1.0, self.f_mat[K_M_SCALE_Y]);
                if !is_finite2(inv_sx, inv_sy) {
                    None
                } else {
                    let inv_tx = -self.f_mat[K_M_TRANS_X] * inv_sx;
                    let inv_ty = -self.f_mat[K_M_TRANS_Y] * inv_sy;
                    if !is_finite2(inv_tx, inv_ty) {
                        None
                    } else {
                        let mut inv = Self::new();
                        inv.f_mat = [inv_sx, 0.0, inv_tx, 0.0, inv_sy, inv_ty, 0.0, 0.0, 1.0];
                        inv.f_type_mask = mask | K_RECT_STAYS_RECT_MASK;
                        Some(inv)
                    }
                }
            } else if !is_finite2(self.f_mat[K_M_TRANS_X], self.f_mat[K_M_TRANS_Y]) {
                None
            } else {
                Some(Self::translate(
                    -self.f_mat[K_M_TRANS_X],
                    -self.f_mat[K_M_TRANS_Y],
                ))
            }
        } else {
            let is_persp = mask & K_PERSPECTIVE_MASK != 0;
            let inv_det = sk_inv_determinant(&self.f_mat, is_persp);
            if inv_det == 0.0 {
                None
            } else {
                let mut inv = Self::new();
                compute_inv(&mut inv.f_mat, &self.f_mat, inv_det, is_persp);
                if !inv.is_finite() {
                    None
                } else {
                    inv.f_type_mask = self.f_type_mask;
                    Some(inv)
                }
            }
        };
        if let Some(inverse) = inverse {
            if let Some(result) = result {
                *inverse = result;
                return true;
            }
            return false;
        }
        result.is_some()
    }

    pub fn inverted(&self) -> Option<Self> {
        let mut result = Self::new();
        self.invert(Some(&mut result)).then_some(result)
    }

    pub fn map_points(&self, dst: &mut [Point], src: &[Point]) {
        let count = dst.len().min(src.len());
        if count == 0 {
            return;
        }
        let mask = self.get_type();
        if mask == K_IDENTITY_MASK {
            dst[..count].copy_from_slice(&src[..count]);
        } else if mask <= K_TRANSLATE_MASK {
            let tx = self.f_mat[K_M_TRANS_X];
            let ty = self.f_mat[K_M_TRANS_Y];
            for i in 0..count {
                dst[i] = Point::new(src[i].x + tx, src[i].y + ty);
            }
        } else if mask & K_PERSPECTIVE_MASK != 0 {
            for i in 0..count {
                dst[i] = self.map_point_perspective(src[i]);
            }
        } else if mask <= (K_SCALE_MASK | K_TRANSLATE_MASK) {
            let tx = self.f_mat[K_M_TRANS_X];
            let ty = self.f_mat[K_M_TRANS_Y];
            let sx = self.f_mat[K_M_SCALE_X];
            let sy = self.f_mat[K_M_SCALE_Y];
            for i in 0..count {
                dst[i] = Point::new(src[i].x * sx + tx, src[i].y * sy + ty);
            }
        } else {
            let tx = self.f_mat[K_M_TRANS_X];
            let ty = self.f_mat[K_M_TRANS_Y];
            let sx = self.f_mat[K_M_SCALE_X];
            let sy = self.f_mat[K_M_SCALE_Y];
            let kx = self.f_mat[K_M_SKEW_X];
            let ky = self.f_mat[K_M_SKEW_Y];
            for i in 0..count {
                let x = src[i].x;
                let y = src[i].y;
                dst[i] = Point::new((x * sx + y * kx) + tx, (x * ky + y * sy) + ty);
            }
        }
    }

    pub fn map_points_in_place(&self, points: &mut [Point]) {
        let mask = self.get_type();
        if mask == K_IDENTITY_MASK {
            return;
        }
        for point in points.iter_mut() {
            *point = self.map_point(*point);
        }
    }

    pub fn map_homogeneous_points(&self, dst: &mut [Point3], src: &[Point3]) {
        let count = dst.len().min(src.len());
        if count == 0 {
            return;
        }
        if self.is_identity() {
            dst[..count].copy_from_slice(&src[..count]);
            return;
        }
        for i in 0..count {
            let s = src[i];
            dst[i] = Point3::new(
                s.x * self.f_mat[K_M_SCALE_X]
                    + s.y * self.f_mat[K_M_SKEW_X]
                    + s.z * self.f_mat[K_M_TRANS_X],
                s.x * self.f_mat[K_M_SKEW_Y]
                    + s.y * self.f_mat[K_M_SCALE_Y]
                    + s.z * self.f_mat[K_M_TRANS_Y],
                s.x * self.f_mat[K_M_PERSP_0]
                    + s.y * self.f_mat[K_M_PERSP_1]
                    + s.z * self.f_mat[K_M_PERSP_2],
            );
        }
    }

    pub fn map_homogeneous_point(&self, src: Point3) -> Point3 {
        let mut dst = [Point3::default()];
        self.map_homogeneous_points(&mut dst, &[src]);
        dst[0]
    }

    pub fn map_points_to_homogeneous(&self, dst: &mut [Point3], src: &[Point]) {
        let count = dst.len().min(src.len());
        if self.is_identity() {
            for i in 0..count {
                dst[i] = Point3::new(src[i].x, src[i].y, 1.0);
            }
        } else if self.has_perspective() {
            for i in 0..count {
                dst[i] = Point3::new(
                    self.f_mat[0] * src[i].x + self.f_mat[1] * src[i].y + self.f_mat[2],
                    self.f_mat[3] * src[i].x + self.f_mat[4] * src[i].y + self.f_mat[5],
                    self.f_mat[6] * src[i].x + self.f_mat[7] * src[i].y + self.f_mat[8],
                );
            }
        } else {
            for i in 0..count {
                dst[i] = Point3::new(
                    self.f_mat[0] * src[i].x + self.f_mat[1] * src[i].y + self.f_mat[2],
                    self.f_mat[3] * src[i].x + self.f_mat[4] * src[i].y + self.f_mat[5],
                    1.0,
                );
            }
        }
    }

    pub fn map_point(&self, point: Point) -> Point {
        let mut dst = [Point::default()];
        self.map_points(&mut dst, &[point]);
        dst[0]
    }

    pub fn map_point_affine(&self, point: Point) -> Point {
        Point::new(
            point.x * self.f_mat[K_M_SCALE_X]
                + point.y * self.f_mat[K_M_SKEW_X]
                + self.f_mat[K_M_TRANS_X],
            point.x * self.f_mat[K_M_SKEW_Y]
                + point.y * self.f_mat[K_M_SCALE_Y]
                + self.f_mat[K_M_TRANS_Y],
        )
    }

    pub fn map_origin(&self) -> Point {
        Point::new(self.f_mat[K_M_TRANS_X], self.f_mat[K_M_TRANS_Y])
    }

    pub fn map_vectors(&self, dst: &mut [Point], src: &[Point]) {
        if self.has_perspective() {
            let origin = self.map_point_perspective(Point::new(0.0, 0.0));
            let count = dst.len().min(src.len());
            for i in (0..count).rev() {
                let mapped = self.map_point_perspective(src[i]);
                dst[i] = Point::new(mapped.x - origin.x, mapped.y - origin.y);
            }
        } else {
            let mut tmp = *self;
            tmp.f_mat[K_M_TRANS_X] = 0.0;
            tmp.f_mat[K_M_TRANS_Y] = 0.0;
            tmp.f_type_mask &= !K_TRANSLATE_MASK;
            tmp.map_points(dst, src);
        }
    }

    pub fn map_vector(&self, vector: Point) -> Point {
        let mut dst = [Point::default()];
        self.map_vectors(&mut dst, &[vector]);
        dst[0]
    }

    pub fn map_rect_scale_translate(&self, src: &Rect) -> Rect {
        let mut points = [
            Point::new(src.left(), src.top()),
            Point::new(src.right(), src.top()),
            Point::new(src.left(), src.bottom()),
            Point::new(src.right(), src.bottom()),
        ];
        self.map_points_in_place(&mut points);
        sort_as_rect(points)
    }

    pub fn map_rect(&self, src: &Rect) -> Rect {
        let mut dst = Rect::default();
        self.map_rect_into(&mut dst, src);
        dst
    }

    pub fn map_rect_into(&self, dst: &mut Rect, src: &Rect) -> bool {
        if self.get_type() <= K_TRANSLATE_MASK {
            let tx = self.f_mat[K_M_TRANS_X];
            let ty = self.f_mat[K_M_TRANS_Y];
            *dst = sort_as_rect([
                Point::new(src.left() + tx, src.top() + ty),
                Point::new(src.right() + tx, src.top() + ty),
                Point::new(src.left() + tx, src.bottom() + ty),
                Point::new(src.right() + tx, src.bottom() + ty),
            ]);
            return true;
        }
        if self.is_scale_translate() {
            *dst = self.map_rect_scale_translate(src);
            return true;
        }
        if self.has_perspective() {
            *dst = M44::from(self).map_rect(src);
            return false;
        }
        let mut quad = src.to_quad(true);
        self.map_points_in_place(&mut quad);
        dst.set_bounds_no_check(&quad);
        self.rect_stays_rect()
    }

    pub fn map_rect_to_quad(&self, src: &Rect, dst: &mut [Point; 4]) {
        let mut quad = src.to_quad(true);
        self.map_points_in_place(&mut quad);
        *dst = quad;
    }

    pub fn map_radius(&self, radius: Scalar) -> Scalar {
        let mut vectors = [Point::new(radius, 0.0), Point::new(0.0, radius)];
        self.map_vectors_in_place(&mut vectors);
        let d0 = vectors[0].length();
        let d1 = vectors[1].length();
        (d0 * d1).sqrt()
    }

    fn map_vectors_in_place(&self, vectors: &mut [Point]) {
        let mut tmp = [Point::default(); 8];
        let count = vectors.len().min(tmp.len());
        tmp[..count].copy_from_slice(&vectors[..count]);
        self.map_vectors(&mut vectors[..count], &tmp[..count]);
    }

    pub fn get_min_scale(&self) -> Scalar {
        let mut result = [0.0];
        if get_scale_factor(self.get_type(), &self.f_mat, MinMax::Min, &mut result) {
            result[0]
        } else {
            -1.0
        }
    }

    pub fn get_max_scale(&self) -> Scalar {
        let mut result = [0.0];
        if get_scale_factor(self.get_type(), &self.f_mat, MinMax::Max, &mut result) {
            result[0]
        } else {
            -1.0
        }
    }

    pub fn get_min_max_scales(&self, scales: &mut [Scalar; 2]) -> bool {
        let mut result = [0.0; 2];
        if !get_scale_factor(self.get_type(), &self.f_mat, MinMax::Both, &mut result) {
            return false;
        }
        *scales = result;
        true
    }

    pub fn decompose_scale(&self, scale: &mut Size, remaining: &mut Matrix) -> bool {
        if self.has_perspective() {
            return false;
        }
        let sx = Point::length_xy(self.f_mat[K_M_SCALE_X], self.f_mat[K_M_SKEW_Y]);
        let sy = Point::length_xy(self.f_mat[K_M_SKEW_X], self.f_mat[K_M_SCALE_Y]);
        if !is_finite2(sx, sy) || scalar_nearly_zero(sx) || scalar_nearly_zero(sy) {
            return false;
        }
        *scale = Size::new(sx, sy);
        *remaining = *self;
        remaining.pre_scale(1.0 / sx, 1.0 / sy);
        true
    }

    pub fn is_finite(&self) -> bool {
        self.f_mat.iter().all(|value| is_finite(*value))
    }

    #[allow(non_snake_case)]
    pub fn isFinite(&self) -> bool {
        self.is_finite()
    }

    pub fn is_similarity(&self) -> bool {
        self.is_similarity_within(SCALAR_NEARLY_ZERO)
    }

    #[allow(non_snake_case)]
    pub fn isSimilarity(&self) -> bool {
        self.is_similarity()
    }

    pub fn is_similarity_within(&self, tol: Scalar) -> bool {
        let mask = self.get_type();
        if mask <= K_TRANSLATE_MASK {
            return true;
        }
        if mask & K_PERSPECTIVE_MASK != 0 {
            return false;
        }
        let mx = self.f_mat[K_M_SCALE_X];
        let my = self.f_mat[K_M_SCALE_Y];
        if mask & K_AFFINE_MASK == 0 {
            return !scalar_nearly_zero(mx)
                && scalar_nearly_equal_within(scalar_abs(mx), scalar_abs(my), SCALAR_NEARLY_ZERO);
        }
        let sx = self.f_mat[K_M_SKEW_X];
        let sy = self.f_mat[K_M_SKEW_Y];
        if is_degenerate_2x2(mx, sx, sy, my) {
            return false;
        }
        (scalar_nearly_equal_within(mx, my, tol) && scalar_nearly_equal_within(sx, -sy, tol))
            || (scalar_nearly_equal_within(mx, -my, tol) && scalar_nearly_equal_within(sx, sy, tol))
    }

    pub fn preserves_right_angles(&self) -> bool {
        self.preserves_right_angles_within(SCALAR_NEARLY_ZERO)
    }

    #[allow(non_snake_case)]
    pub fn preservesRightAngles(&self) -> bool {
        self.preserves_right_angles()
    }

    pub fn preserves_right_angles_within(&self, tol: Scalar) -> bool {
        let mask = self.get_type();
        if mask <= K_TRANSLATE_MASK {
            return true;
        }
        if mask & K_PERSPECTIVE_MASK != 0 {
            return false;
        }
        let mx = self.f_mat[K_M_SCALE_X];
        let my = self.f_mat[K_M_SCALE_Y];
        let sx = self.f_mat[K_M_SKEW_X];
        let sy = self.f_mat[K_M_SKEW_Y];
        if is_degenerate_2x2(mx, sx, sy, my) {
            return false;
        }
        let dot = mx * sx + sy * my;
        scalar_nearly_zero_within(dot, tol * tol)
    }

    pub fn poly_to_poly(src: &[Point], dst: &[Point]) -> Option<Self> {
        if src.len() != dst.len() || src.len() > 4 {
            return None;
        }
        match src.len() {
            0 => Some(Self::new()),
            1 => Some(Self::translate(dst[0].x - src[0].x, dst[0].y - src[0].y)),
            2..=4 => {
                let mut temp = Self::new();
                if !poly_proc(src, &mut temp) {
                    return None;
                }
                let inverse = temp.inverted()?;
                if !poly_proc(dst, &mut temp) {
                    return None;
                }
                Some(temp * inverse)
            }
            _ => None,
        }
    }

    #[allow(non_snake_case)]
    pub fn PolyToPoly(src: &[Point], dst: &[Point]) -> Option<Self> {
        Self::poly_to_poly(src, dst)
    }

    pub fn set_poly_to_poly(&mut self, src: &[Point], dst: &[Point]) -> bool {
        match Self::poly_to_poly(src, dst) {
            Some(matrix) => {
                *self = matrix;
                true
            }
            None => false,
        }
    }

    pub fn post_idiv(&mut self, div_x: i32, div_y: i32) -> bool {
        if div_x == 0 || div_y == 0 {
            return false;
        }
        let inv_x = 1.0 / div_x as Scalar;
        let inv_y = 1.0 / div_y as Scalar;
        self.f_mat[K_M_SCALE_X] *= inv_x;
        self.f_mat[K_M_SKEW_X] *= inv_x;
        self.f_mat[K_M_TRANS_X] *= inv_x;
        self.f_mat[K_M_SCALE_Y] *= inv_y;
        self.f_mat[K_M_SKEW_Y] *= inv_y;
        self.f_mat[K_M_TRANS_Y] *= inv_y;
        self.f_type_mask = K_UNKNOWN_MASK;
        true
    }

    #[allow(non_snake_case)]
    pub fn postIDiv(&mut self, div_x: i32, div_y: i32) -> bool {
        self.post_idiv(div_x, div_y)
    }

    pub fn write_to_memory(&self, buffer: &mut [u8]) -> usize {
        if buffer.len() < 36 {
            return 0;
        }
        for (index, value) in self.f_mat.iter().enumerate() {
            buffer[index * 4..index * 4 + 4].copy_from_slice(&value.to_ne_bytes());
        }
        36
    }

    pub fn read_from_memory(&mut self, buffer: &[u8]) -> usize {
        Matrix::read_from_memory_into(buffer, self)
    }

    pub fn read_from_memory_into(buffer: &[u8], matrix: &mut Matrix) -> usize {
        if buffer.len() < 36 {
            return 0;
        }
        for index in 0..9 {
            matrix.f_mat[index] =
                Scalar::from_ne_bytes(buffer[index * 4..index * 4 + 4].try_into().unwrap());
        }
        matrix.f_type_mask = K_UNKNOWN_MASK;
        36
    }

    pub fn dirty_matrix_type_cache(&mut self) {
        self.f_type_mask = K_UNKNOWN_MASK;
    }

    pub fn dump(&self) {
        for row in 0..3 {
            eprintln!(
                "[{} {} {}]",
                self.f_mat[row * 3],
                self.f_mat[row * 3 + 1],
                self.f_mat[row * 3 + 2]
            );
        }
    }

    fn map_point_perspective(&self, point: Point) -> Point {
        let x = point.x * self.f_mat[K_M_SCALE_X]
            + point.y * self.f_mat[K_M_SKEW_X]
            + self.f_mat[K_M_TRANS_X];
        let y = point.x * self.f_mat[K_M_SKEW_Y]
            + point.y * self.f_mat[K_M_SCALE_Y]
            + self.f_mat[K_M_TRANS_Y];
        let mut z = point.x * self.f_mat[K_M_PERSP_0]
            + point.y * self.f_mat[K_M_PERSP_1]
            + self.f_mat[K_M_PERSP_2];
        if z != 0.0 {
            z = 1.0 / z;
        }
        Point::new(x * z, y * z)
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        self.f_mat == other.f_mat
    }
}

impl Mul for Matrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Concat(&self, &rhs)
    }
}

impl core::ops::Index<usize> for Matrix {
    type Output = Scalar;
    fn index(&self, index: usize) -> &Self::Output {
        &self.f_mat[index]
    }
}

impl core::ops::IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Scalar {
        self.f_type_mask = K_UNKNOWN_MASK;
        &mut self.f_mat[index]
    }
}

fn compute_type_mask(matrix: &Matrix) -> u32 {
    let m = &matrix.f_mat;
    if m[K_M_PERSP_0] != 0.0 || m[K_M_PERSP_1] != 0.0 || m[K_M_PERSP_2] != 1.0 {
        return 0x0F;
    }
    let mut mask = 0;
    if m[K_M_TRANS_X] != 0.0 || m[K_M_TRANS_Y] != 0.0 {
        mask |= K_TRANSLATE_MASK;
    }
    let mut m00 = scalar_as_2s_compliment(m[K_M_SCALE_X]);
    let mut m01 = scalar_as_2s_compliment(m[K_M_SKEW_X]);
    let mut m10 = scalar_as_2s_compliment(m[K_M_SKEW_Y]);
    let mut m11 = scalar_as_2s_compliment(m[K_M_SCALE_Y]);
    if m01 | m10 != 0 {
        mask |= K_AFFINE_MASK | K_SCALE_MASK;
        m01 = (m01 != 0) as i32;
        m10 = (m10 != 0) as i32;
        let dp0 = (m00 | m11) == 0;
        let ds1 = m01 & m10;
        if dp0 && ds1 != 0 {
            mask |= K_RECT_STAYS_RECT_MASK;
        }
    } else {
        if (m00 ^ 0x3f80_0000) | (m11 ^ 0x3f80_0000) != 0 {
            mask |= K_SCALE_MASK;
        }
        m00 = (m00 != 0) as i32;
        m11 = (m11 != 0) as i32;
        if m00 & m11 != 0 {
            mask |= K_RECT_STAYS_RECT_MASK;
        }
    }
    mask
}

fn is_degenerate_2x2(scale_x: Scalar, skew_x: Scalar, skew_y: Scalar, scale_y: Scalar) -> bool {
    let perp_dot = scale_x * scale_y - skew_x * skew_y;
    scalar_nearly_zero_within(perp_dot, SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO)
}

fn muladdmul(a: Scalar, b: Scalar, c: Scalar, d: Scalar) -> Scalar {
    (a as f64 * b as f64 + c as f64 * d as f64) as Scalar
}

fn rowcol3(row: &[Scalar], b: &Matrix, col: usize) -> Scalar {
    row[0] * b.f_mat[col] + row[1] * b.f_mat[3 + col] + row[2] * b.f_mat[6 + col]
}

fn scross(a: Scalar, b: Scalar, c: Scalar, d: Scalar) -> Scalar {
    a * b - c * d
}

fn scross_dscale(a: Scalar, b: Scalar, c: Scalar, d: Scalar, scale: f64) -> Scalar {
    (scross(a, b, c, d) as f64 * scale) as Scalar
}

fn dcross(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * b - c * d
}

fn dcross_dscale(a: f64, b: f64, c: f64, d: f64, scale: f64) -> Scalar {
    (dcross(a, b, c, d) * scale) as Scalar
}

fn compute_inv(dst: &mut [Scalar; 9], src: &[Scalar; 9], inv_det: f64, is_persp: bool) {
    if is_persp {
        dst[K_M_SCALE_X] = scross_dscale(
            src[K_M_SCALE_Y],
            src[K_M_PERSP_2],
            src[K_M_TRANS_Y],
            src[K_M_PERSP_1],
            inv_det,
        );
        dst[K_M_SKEW_X] = scross_dscale(
            src[K_M_TRANS_X],
            src[K_M_PERSP_1],
            src[K_M_SKEW_X],
            src[K_M_PERSP_2],
            inv_det,
        );
        dst[K_M_TRANS_X] = scross_dscale(
            src[K_M_SKEW_X],
            src[K_M_TRANS_Y],
            src[K_M_TRANS_X],
            src[K_M_SCALE_Y],
            inv_det,
        );
        dst[K_M_SKEW_Y] = scross_dscale(
            src[K_M_TRANS_Y],
            src[K_M_PERSP_0],
            src[K_M_SKEW_Y],
            src[K_M_PERSP_2],
            inv_det,
        );
        dst[K_M_SCALE_Y] = scross_dscale(
            src[K_M_SCALE_X],
            src[K_M_PERSP_2],
            src[K_M_TRANS_X],
            src[K_M_PERSP_0],
            inv_det,
        );
        dst[K_M_TRANS_Y] = scross_dscale(
            src[K_M_TRANS_X],
            src[K_M_SKEW_Y],
            src[K_M_SCALE_X],
            src[K_M_TRANS_Y],
            inv_det,
        );
        dst[K_M_PERSP_0] = scross_dscale(
            src[K_M_SKEW_Y],
            src[K_M_PERSP_1],
            src[K_M_SCALE_Y],
            src[K_M_PERSP_0],
            inv_det,
        );
        dst[K_M_PERSP_1] = scross_dscale(
            src[K_M_SKEW_X],
            src[K_M_PERSP_0],
            src[K_M_SCALE_X],
            src[K_M_PERSP_1],
            inv_det,
        );
        dst[K_M_PERSP_2] = scross_dscale(
            src[K_M_SCALE_X],
            src[K_M_SCALE_Y],
            src[K_M_SKEW_X],
            src[K_M_SKEW_Y],
            inv_det,
        );
    } else {
        dst[K_M_SCALE_X] = (src[K_M_SCALE_Y] as f64 * inv_det) as Scalar;
        dst[K_M_SKEW_X] = (-src[K_M_SKEW_X] as f64 * inv_det) as Scalar;
        dst[K_M_TRANS_X] = dcross_dscale(
            src[K_M_SKEW_X] as f64,
            src[K_M_TRANS_Y] as f64,
            src[K_M_SCALE_Y] as f64,
            src[K_M_TRANS_X] as f64,
            inv_det,
        );
        dst[K_M_SKEW_Y] = (-src[K_M_SKEW_Y] as f64 * inv_det) as Scalar;
        dst[K_M_SCALE_Y] = (src[K_M_SCALE_X] as f64 * inv_det) as Scalar;
        dst[K_M_TRANS_Y] = dcross_dscale(
            src[K_M_SKEW_Y] as f64,
            src[K_M_TRANS_X] as f64,
            src[K_M_SCALE_X] as f64,
            src[K_M_TRANS_Y] as f64,
            inv_det,
        );
        dst[K_M_PERSP_0] = 0.0;
        dst[K_M_PERSP_1] = 0.0;
        dst[K_M_PERSP_2] = 1.0;
    }
}

fn sk_inv_determinant(mat: &[Scalar; 9], is_persp: bool) -> f64 {
    let det = if is_persp {
        mat[K_M_SCALE_X] as f64
            * dcross(
                mat[K_M_SCALE_Y] as f64,
                mat[K_M_PERSP_2] as f64,
                mat[K_M_TRANS_Y] as f64,
                mat[K_M_PERSP_1] as f64,
            )
            + mat[K_M_SKEW_X] as f64
                * dcross(
                    mat[K_M_TRANS_Y] as f64,
                    mat[K_M_PERSP_0] as f64,
                    mat[K_M_SKEW_Y] as f64,
                    mat[K_M_PERSP_2] as f64,
                )
            + mat[K_M_TRANS_X] as f64
                * dcross(
                    mat[K_M_SKEW_Y] as f64,
                    mat[K_M_PERSP_1] as f64,
                    mat[K_M_SCALE_Y] as f64,
                    mat[K_M_PERSP_0] as f64,
                )
    } else {
        dcross(
            mat[K_M_SCALE_X] as f64,
            mat[K_M_SCALE_Y] as f64,
            mat[K_M_SKEW_X] as f64,
            mat[K_M_SKEW_Y] as f64,
        )
    };
    if scalar_nearly_zero_within(
        det as Scalar,
        SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO,
    ) {
        0.0
    } else {
        ieee_double_divide(1.0, det)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MinMax {
    Min,
    Max,
    Both,
}

fn get_scale_factor(type_mask: u32, m: &[Scalar; 9], mode: MinMax, results: &mut [Scalar]) -> bool {
    if type_mask & K_PERSPECTIVE_MASK != 0 {
        return false;
    }
    if type_mask == K_IDENTITY_MASK {
        results[0] = 1.0;
        if mode == MinMax::Both {
            results[1] = 1.0;
        }
        return true;
    }
    if type_mask & K_AFFINE_MASK == 0 {
        if mode == MinMax::Min {
            results[0] = scalar_abs(m[K_M_SCALE_X]).min(scalar_abs(m[K_M_SCALE_Y]));
        } else if mode == MinMax::Max {
            results[0] = scalar_abs(m[K_M_SCALE_X]).max(scalar_abs(m[K_M_SCALE_Y]));
        } else {
            results[0] = scalar_abs(m[K_M_SCALE_X]);
            results[1] = scalar_abs(m[K_M_SCALE_Y]);
            if results[0] > results[1] {
                results.swap(0, 1);
            }
        }
        return true;
    }
    let a = m[K_M_SCALE_X] * m[K_M_SCALE_X] + m[K_M_SKEW_Y] * m[K_M_SKEW_Y];
    let b = m[K_M_SCALE_X] * m[K_M_SKEW_X] + m[K_M_SCALE_Y] * m[K_M_SKEW_Y];
    let c = m[K_M_SKEW_X] * m[K_M_SKEW_X] + m[K_M_SCALE_Y] * m[K_M_SCALE_Y];
    let b_sqd = b * b;
    if b_sqd <= SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO {
        if mode == MinMax::Min {
            results[0] = a.min(c);
        } else if mode == MinMax::Max {
            results[0] = a.max(c);
        } else {
            results[0] = a;
            results[1] = c;
            if results[0] > results[1] {
                results.swap(0, 1);
            }
        }
    } else {
        let a_minus_c = a - c;
        let a_plus_c_div2 = (a + c) / 2.0;
        let x = (a_minus_c * a_minus_c + 4.0 * b_sqd).sqrt() / 2.0;
        if mode == MinMax::Min {
            results[0] = a_plus_c_div2 - x;
        } else if mode == MinMax::Max {
            results[0] = a_plus_c_div2 + x;
        } else {
            results[0] = a_plus_c_div2 - x;
            results[1] = a_plus_c_div2 + x;
        }
    }
    if !is_finite(results[0]) {
        return false;
    }
    if results[0] < 0.0 {
        results[0] = 0.0;
    }
    results[0] = results[0].sqrt();
    if mode == MinMax::Both {
        if !is_finite(results[1]) {
            return false;
        }
        if results[1] < 0.0 {
            results[1] = 0.0;
        }
        results[1] = results[1].sqrt();
    }
    true
}

fn sort_as_rect(points: [Point; 4]) -> Rect {
    let mut min_x = points[0].x;
    let mut max_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_y = points[0].y;
    for point in points.iter().skip(1) {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    Rect::new(min_x, min_y, max_x, max_y)
}

fn poly_proc(src: &[Point], dst: &mut Matrix) -> bool {
    match src.len() {
        2 => {
            dst.f_mat[K_M_SCALE_X] = src[1].y - src[0].y;
            dst.f_mat[K_M_SKEW_Y] = src[0].x - src[1].x;
            dst.f_mat[K_M_PERSP_0] = 0.0;
            dst.f_mat[K_M_SKEW_X] = src[1].x - src[0].x;
            dst.f_mat[K_M_SCALE_Y] = src[1].y - src[0].y;
            dst.f_mat[K_M_PERSP_1] = 0.0;
            dst.f_mat[K_M_TRANS_X] = src[0].x;
            dst.f_mat[K_M_TRANS_Y] = src[0].y;
            dst.f_mat[K_M_PERSP_2] = 1.0;
            dst.f_type_mask = K_UNKNOWN_MASK;
            true
        }
        3 => {
            dst.f_mat[K_M_SCALE_X] = src[2].x - src[0].x;
            dst.f_mat[K_M_SKEW_Y] = src[2].y - src[0].y;
            dst.f_mat[K_M_PERSP_0] = 0.0;
            dst.f_mat[K_M_SKEW_X] = src[1].x - src[0].x;
            dst.f_mat[K_M_SCALE_Y] = src[1].y - src[0].y;
            dst.f_mat[K_M_PERSP_1] = 0.0;
            dst.f_mat[K_M_TRANS_X] = src[0].x;
            dst.f_mat[K_M_TRANS_Y] = src[0].y;
            dst.f_mat[K_M_PERSP_2] = 1.0;
            dst.f_type_mask = K_UNKNOWN_MASK;
            true
        }
        4 => {
            let x0 = src[2].x - src[0].x;
            let y0 = src[2].y - src[0].y;
            let x1 = src[2].x - src[1].x;
            let y1 = src[2].y - src[1].y;
            let x2 = src[2].x - src[3].x;
            let y2 = src[2].y - src[3].y;
            let a1 = if abs_x_gt_y(x2, y2) {
                let denom = ieee_float_divide(x1 * y2, x2) - y1;
                if check_for_zero(denom) {
                    return false;
                }
                (((x0 - x1) * y2 / x2) - y0 + y1) / denom
            } else {
                let denom = x1 - ieee_float_divide(y1 * x2, y2);
                if check_for_zero(denom) {
                    return false;
                }
                (x0 - x1 - ieee_float_divide((y0 - y1) * x2, y2)) / denom
            };
            let a2 = if abs_x_gt_y(x1, y1) {
                let denom = y2 - ieee_float_divide(x2 * y1, x1);
                if check_for_zero(denom) {
                    return false;
                }
                (y0 - y2 - ieee_float_divide((x0 - x2) * y1, x1)) / denom
            } else {
                let denom = ieee_float_divide(y2 * x1, y1) - x2;
                if check_for_zero(denom) {
                    return false;
                }
                (ieee_float_divide((y0 - y2) * x1, y1) - x0 + x2) / denom
            };
            dst.f_mat[K_M_SCALE_X] = a2 * src[3].x + src[3].x - src[0].x;
            dst.f_mat[K_M_SKEW_Y] = a2 * src[3].y + src[3].y - src[0].y;
            dst.f_mat[K_M_PERSP_0] = a2;
            dst.f_mat[K_M_SKEW_X] = a1 * src[1].x + src[1].x - src[0].x;
            dst.f_mat[K_M_SCALE_Y] = a1 * src[1].y + src[1].y - src[0].y;
            dst.f_mat[K_M_PERSP_1] = a1;
            dst.f_mat[K_M_TRANS_X] = src[0].x;
            dst.f_mat[K_M_TRANS_Y] = src[0].y;
            dst.f_mat[K_M_PERSP_2] = 1.0;
            dst.f_type_mask = K_UNKNOWN_MASK;
            true
        }
        _ => false,
    }
}

fn check_for_zero(x: Scalar) -> bool {
    x * x == 0.0
}

fn abs_x_gt_y(x: Scalar, y: Scalar) -> bool {
    if x > 0.0 {
        if y > 0.0 { x > y } else { x > -y }
    } else if y > 0.0 {
        -x > y
    } else {
        x < y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_matrix_identity_and_scale_translate() {
        let m = Matrix::new();
        assert!(m.is_identity());
        assert_eq!(m.get_type(), K_IDENTITY_MASK);
        assert!(m.rect_stays_rect());

        let mut m = Matrix::scale(2.0, 3.0);
        m.post_translate(1.0, 4.0);
        assert!(m.is_scale_translate());
        assert_eq!(m.map_point(Point::new(2.0, 2.0)), Point::new(5.0, 10.0));

        let inverse = m.inverted().unwrap();
        assert!(Matrix::Concat(&m, &inverse).is_identity());
    }

    #[test]
    fn dev_matrix_rect_mapping() {
        let m = Matrix::rotate_deg(90.0);
        assert!(m.rect_stays_rect());
        let r = m.map_rect(&Rect::new(0.0, 0.0, 10.0, 20.0));
        assert_eq!(r, Rect::new(-20.0, 0.0, 0.0, 10.0));
    }
}
