//! Port of `include/core/SkPathMeasure.h` and `src/core/SkPathMeasure.cpp`.

use std::sync::Arc;

use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::scalar::Scalar;
use crate::sk_contour_measure::{
    MatrixFlags, SkContourMeasure, SkContourMeasureIter, SkPathMeasurePriv,
};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;

/// `SkPathMeasure`.
pub struct SkPathMeasure {
    f_iter: SkContourMeasureIter,
    f_contour: Option<Arc<SkContourMeasure>>,
}

impl Default for SkPathMeasure {
    fn default() -> Self {
        Self::new()
    }
}

impl SkPathMeasure {
    pub fn new() -> Self {
        SkPathMeasure {
            f_iter: SkContourMeasureIter::new(),
            f_contour: None,
        }
    }

    pub fn from_path(path: &SkPath, force_closed: bool, res_scale: Scalar) -> Self {
        let mut iter = SkContourMeasureIter::from_path(path, force_closed, res_scale);
        let contour = iter.next();
        SkPathMeasure {
            f_iter: iter,
            f_contour: contour,
        }
    }

    /// Reset the pathmeasure with the specified path (`None` clears it).
    pub fn set_path(&mut self, path: Option<&SkPath>, force_closed: bool) {
        match path {
            Some(path) => self.f_iter.reset(path, force_closed, 1.0),
            None => self.f_iter.reset(&SkPath::new(), force_closed, 1.0),
        }
        self.f_contour = self.f_iter.next();
    }

    pub fn get_length(&self) -> Scalar {
        match &self.f_contour {
            Some(c) => c.length(),
            None => 0.0,
        }
    }

    pub fn get_pos_tan(
        &self,
        distance: Scalar,
        position: Option<&mut Point>,
        tangent: Option<&mut Vector>,
    ) -> bool {
        self.f_contour
            .as_ref()
            .map(|c| c.get_pos_tan(distance, position, tangent))
            .unwrap_or(false)
    }

    pub fn get_matrix(
        &self,
        distance: Scalar,
        matrix: Option<&mut Matrix>,
        flags: MatrixFlags,
    ) -> bool {
        self.f_contour
            .as_ref()
            .map(|c| c.get_matrix(distance, matrix, flags))
            .unwrap_or(false)
    }

    pub fn get_segment(
        &self,
        start_d: Scalar,
        stop_d: Scalar,
        dst: &mut SkPathBuilder,
        start_with_move_to: bool,
    ) -> bool {
        self.f_contour
            .as_ref()
            .map(|c| c.get_segment(start_d, stop_d, dst, start_with_move_to))
            .unwrap_or(false)
    }

    pub fn is_closed(&self) -> bool {
        self.f_contour
            .as_ref()
            .map(|c| c.is_closed())
            .unwrap_or(false)
    }

    pub fn next_contour(&mut self) -> bool {
        self.f_contour = self.f_iter.next();
        self.f_contour.is_some()
    }

    pub fn current_measure(&self) -> Option<&SkContourMeasure> {
        self.f_contour.as_deref()
    }

    /// `SkPathMeasurePriv::CountSegments`.
    pub fn count_segments(&self) -> usize {
        SkPathMeasurePriv::count_segments(self)
    }
}
