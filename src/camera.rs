use crate::m44::{M44, V3};
use crate::matrix::Matrix;
use crate::scalar::{SCALAR_PI, Scalar};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Patch3D {
    pub f_u: V3,
    pub f_v: V3,
    pub f_origin: V3,
}

pub type SkPatch3D = Patch3D;

impl Default for Patch3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Patch3D {
    pub fn new() -> Self {
        Self {
            f_u: V3::new(1.0, 0.0, 0.0),
            f_v: V3::new(0.0, -1.0, 0.0),
            f_origin: V3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn transform(&mut self, m: &M44) {
        *self = self.transformed(m);
    }

    pub fn transformed(&self, m: &M44) -> Self {
        let u = *m * self.f_u;
        let v = *m * self.f_v;
        let origin = m.map(self.f_origin.x, self.f_origin.y, self.f_origin.z, 1.0);
        Self {
            f_u: V3::new(u.x, u.y, u.z),
            f_v: V3::new(v.x, v.y, v.z),
            f_origin: V3::new(origin.x, origin.y, origin.z),
        }
    }

    pub fn dot_with(&self, dx: Scalar, dy: Scalar, dz: Scalar) -> Scalar {
        let cx = self.f_u.y * self.f_v.z - self.f_u.z * self.f_v.y;
        let cy = self.f_u.z * self.f_v.x - self.f_u.x * self.f_v.z;
        let cz = self.f_u.x * self.f_v.y - self.f_u.y * self.f_v.x;
        cx * dx + cy * dy + cz * dz
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Camera3D {
    pub f_location: V3,
    pub f_axis: V3,
    pub f_zenith: V3,
    pub f_observer: V3,
    pub f_orientation: Matrix,
    pub f_need_to_update: bool,
}

pub type SkCamera3D = Camera3D;

impl Default for Camera3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera3D {
    pub fn new() -> Self {
        let mut camera = Self {
            f_location: V3::new(0.0, 0.0, -576.0),
            f_axis: V3::new(0.0, 0.0, 1.0),
            f_zenith: V3::new(0.0, -1.0, 0.0),
            f_observer: V3::new(0.0, 0.0, -576.0),
            f_orientation: Matrix::new(),
            f_need_to_update: true,
        };
        camera.reset();
        camera
    }

    pub fn reset(&mut self) {
        self.f_location = V3::new(0.0, 0.0, -576.0);
        self.f_axis = V3::new(0.0, 0.0, 1.0);
        self.f_zenith = V3::new(0.0, -1.0, 0.0);
        self.f_observer = V3::new(0.0, 0.0, self.f_location.z);
        self.f_need_to_update = true;
    }

    pub fn update(&mut self) {
        self.f_need_to_update = true;
    }

    fn do_update(&mut self) {
        let axis = self.f_axis.normalize();
        let zenith = (self.f_zenith - axis * (axis.dot(&self.f_zenith))).normalize();
        let cross = axis.cross(&zenith);
        let x = self.f_observer.x;
        let y = self.f_observer.y;
        let z = self.f_observer.z;
        self.f_orientation = Matrix::make_all(
            x * axis.x - z * cross.x,
            x * axis.y - z * cross.y,
            x * axis.z - z * cross.z,
            y * axis.x - z * zenith.x,
            y * axis.y - z * zenith.y,
            y * axis.z - z * zenith.z,
            axis.x,
            axis.y,
            axis.z,
        );
    }

    pub fn patch_to_matrix(&mut self, quilt: &Patch3D, matrix: &mut Matrix) {
        if self.f_need_to_update {
            self.do_update();
            self.f_need_to_update = false;
        }
        let map = self.f_orientation.get9_array();
        let diff = quilt.f_origin - self.f_location;
        let dot = diff.x * map[6] + diff.y * map[7] + diff.z * map[8];

        let dot_div = |a: [Scalar; 3], b: [Scalar; 3]| -> Scalar {
            (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / dot
        };

        let u = [quilt.f_u.x, quilt.f_u.y, quilt.f_u.z];
        let v = [quilt.f_v.x, quilt.f_v.y, quilt.f_v.z];
        let d = [diff.x, diff.y, diff.z];
        let row0 = [map[0], map[1], map[2]];
        let row1 = [map[3], map[4], map[5]];
        let row2 = [map[6], map[7], map[8]];

        matrix.set_scale_x(dot_div(u, row0));
        matrix.set_skew_y(dot_div(u, row1));
        matrix.set_persp_x(dot_div(u, row2));

        matrix.set_skew_x(dot_div(v, row0));
        matrix.set_scale_y(dot_div(v, row1));
        matrix.set_persp_y(dot_div(v, row2));

        matrix.set_trans_x(dot_div(d, row0));
        matrix.set_trans_y(dot_div(d, row1));
        matrix.set(Matrix::M_PERSP_2, 1.0);
    }

    #[allow(non_snake_case)]
    pub fn patchToMatrix(&mut self, quilt: &Patch3D, matrix: &mut Matrix) {
        self.patch_to_matrix(quilt, matrix);
    }
}

#[derive(Clone, Copy, Debug)]
struct Rec {
    next: Option<usize>,
    matrix: M44,
}

#[derive(Clone, Debug)]
pub struct View3D {
    records: Vec<Rec>,
    current: usize,
    camera: Camera3D,
}

pub type Sk3DView = View3D;

impl Default for View3D {
    fn default() -> Self {
        Self::new()
    }
}

impl View3D {
    pub fn new() -> Self {
        Self {
            records: vec![Rec {
                next: None,
                matrix: M44::new(),
            }],
            current: 0,
            camera: Camera3D::new(),
        }
    }

    pub fn save(&mut self) {
        let record = Rec {
            next: Some(self.current),
            matrix: self.records[self.current].matrix,
        };
        self.records.push(record);
        self.current = self.records.len() - 1;
    }

    pub fn restore(&mut self) {
        assert!(self.current != 0);
        self.current = self.records[self.current]
            .next
            .expect("view stack underflow");
    }

    pub fn translate(&mut self, x: Scalar, y: Scalar, z: Scalar) {
        self.records[self.current].matrix.pre_translate(x, y, z);
    }

    pub fn rotate_x(&mut self, degrees: Scalar) {
        let radians = degrees * SCALAR_PI / 180.0;
        self.records[self.current]
            .matrix
            .pre_concat(&M44::Rotate(V3::new(1.0, 0.0, 0.0), radians));
    }

    pub fn rotate_y(&mut self, degrees: Scalar) {
        let radians = degrees * SCALAR_PI / 180.0;
        self.records[self.current]
            .matrix
            .pre_concat(&M44::Rotate(V3::new(0.0, -1.0, 0.0), radians));
    }

    pub fn rotate_z(&mut self, degrees: Scalar) {
        let radians = degrees * SCALAR_PI / 180.0;
        self.records[self.current]
            .matrix
            .pre_concat(&M44::Rotate(V3::new(0.0, 0.0, 1.0), radians));
    }

    pub fn dot_with_normal(&self, x: Scalar, y: Scalar, z: Scalar) -> Scalar {
        let patch = Patch3D::new().transformed(&self.records[self.current].matrix);
        patch.dot_with(x, y, z)
    }

    pub fn get_matrix(&mut self, matrix: &mut Matrix) {
        let patch = Patch3D::new().transformed(&self.records[self.current].matrix);
        self.camera.patch_to_matrix(&patch, matrix);
    }

    pub fn camera(&self) -> &Camera3D {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera3D {
        &mut self.camera
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_camera_defaults_and_view_stack() {
        let mut view = View3D::new();
        view.translate(1.0, 2.0, 3.0);
        view.save();
        view.rotate_x(90.0);
        let mut matrix = Matrix::new();
        view.get_matrix(&mut matrix);
        assert!(matrix.is_finite());
        view.restore();
        view.get_matrix(&mut matrix);
        assert!(matrix.is_finite());

        let patch = Patch3D::new();
        assert_eq!(patch.dot_with(0.0, 0.0, 1.0), -1.0);
    }
}
