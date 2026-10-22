use crate::point::Point;
use crate::scalar::{Scalar, scalar_cos, scalar_sin};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RSXform {
    pub f_scos: Scalar,
    pub f_ssin: Scalar,
    pub f_tx: Scalar,
    pub f_ty: Scalar,
}

pub type SkRSXform = RSXform;

const _: () = {
    assert!(core::mem::size_of::<RSXform>() == 4 * core::mem::size_of::<Scalar>());
};

impl RSXform {
    pub const fn make(scos: Scalar, ssin: Scalar, tx: Scalar, ty: Scalar) -> Self {
        Self {
            f_scos: scos,
            f_ssin: ssin,
            f_tx: tx,
            f_ty: ty,
        }
    }

    #[allow(non_snake_case)]
    pub const fn Make(scos: Scalar, ssin: Scalar, tx: Scalar, ty: Scalar) -> Self {
        Self::make(scos, ssin, tx, ty)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn make_from_radians(
        scale: Scalar,
        radians: Scalar,
        tx: Scalar,
        ty: Scalar,
        ax: Scalar,
        ay: Scalar,
    ) -> Self {
        let s = scalar_sin(radians) * scale;
        let c = scalar_cos(radians) * scale;
        Self::make(c, s, tx + -c * ax + s * ay, ty + -s * ax - c * ay)
    }

    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub fn MakeFromRadians(
        scale: Scalar,
        radians: Scalar,
        tx: Scalar,
        ty: Scalar,
        ax: Scalar,
        ay: Scalar,
    ) -> Self {
        Self::make_from_radians(scale, radians, tx, ty, ax, ay)
    }

    pub const fn set_identity(&mut self) -> &mut Self {
        self.f_scos = 1.0;
        self.f_ssin = 0.0;
        self.f_tx = 0.0;
        self.f_ty = 0.0;
        self
    }

    pub fn set(&mut self, scos: Scalar, ssin: Scalar, tx: Scalar, ty: Scalar) -> &mut Self {
        *self = Self::make(scos, ssin, tx, ty);
        self
    }

    pub fn scos(&self) -> Scalar {
        self.f_scos
    }

    pub fn ssin(&self) -> Scalar {
        self.f_ssin
    }

    pub fn tx(&self) -> Scalar {
        self.f_tx
    }

    pub fn ty(&self) -> Scalar {
        self.f_ty
    }

    pub fn rect_stays_rect(&self) -> bool {
        self.f_scos == 0.0 || self.f_ssin == 0.0
    }

    #[allow(non_snake_case)]
    pub fn rectStaysRect(&self) -> bool {
        self.rect_stays_rect()
    }

    pub fn to_quad(&self, width: Scalar, height: Scalar) -> [Point; 4] {
        let mut quad = [Point::default(); 4];
        self.to_quad_into(width, height, &mut quad);
        quad
    }

    pub fn to_quad_into(&self, width: Scalar, height: Scalar, quad: &mut [Point; 4]) {
        let m00 = self.f_scos;
        let m01 = -self.f_ssin;
        let m02 = self.f_tx;
        let m10 = -m01;
        let m11 = m00;
        let m12 = self.f_ty;

        quad[0] = Point::new(m02, m12);
        quad[1] = Point::new(m00 * width + m02, m10 * width + m12);
        quad[2] = Point::new(
            m00 * width + m01 * height + m02,
            m10 * width + m11 * height + m12,
        );
        quad[3] = Point::new(m01 * height + m02, m11 * height + m12);
    }

    #[allow(non_snake_case)]
    pub fn toQuad(&self, width: Scalar, height: Scalar, quad: &mut [Point; 4]) {
        self.to_quad_into(width, height, quad);
    }

    pub fn to_tri_strip(&self, width: Scalar, height: Scalar, strip: &mut [Point; 4]) {
        let mut quad = [Point::default(); 4];
        self.to_quad_into(width, height, &mut quad);
        strip[0] = quad[0];
        strip[1] = quad[3];
        strip[2] = quad[1];
        strip[3] = quad[2];
    }

    #[allow(non_snake_case)]
    pub fn toTriStrip(&self, width: Scalar, height: Scalar, strip: &mut [Point; 4]) {
        self.to_tri_strip(width, height, strip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_rsxform_quad() {
        let xform = RSXform::make(1.0, 0.0, 5.0, 6.0);
        let quad = xform.to_quad(10.0, 20.0);
        assert_eq!(quad[0], Point::new(5.0, 6.0));
        assert_eq!(quad[1], Point::new(15.0, 6.0));
        assert_eq!(quad[2], Point::new(15.0, 26.0));
        assert_eq!(quad[3], Point::new(5.0, 26.0));
    }
}
