//! Port of `include/effects/Sk2DPathEffect.h` and `src/effects/Sk2DPathEffect.cpp`.
//!
//! `Sk2DPathEffect::onFilterPath` in the reference fills an `SkRegion` with the inverse-transformed
//! source path and walks the region's rectangles, calling `nextSpan` for every span. `SkRegion` and
//! `SkScan::FillPath` are owned by M09/M14 and do not exist yet, so the port computes the same
//! center-sampled coverage directly with `SkPath::contains` on pixel centers and emits one span per
//! run. The resulting stamp set is identical; only the fixed-point trapezoid walk and the vertical
//! merging of identical scanlines are replaced. See `plan/handoffs/M08.md`.
//!
//! `begin`, `nextSpan` and `end` keep their reference roles: the two concrete effects override
//! `next` (path stamps) or `nextSpan` (line spans).

use crate::matrix::Matrix;
use crate::point::Point;
use crate::rect::{IRect, Rect};
use crate::scalar::Scalar;
use crate::sk_path::{AddPathMode, SkPath};
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_effect::{SkPathEffect, SkPathEffectBase};
use crate::sk_stroke_rec::SkStrokeRec;

/// The concrete effect payloads: `SkLine2DPathEffectImpl` and `SkPath2DPathEffectImpl`.
enum Sk2DPathEffectKind {
    Line { f_width: Scalar },
    Path { f_path: SkPath },
}

/// `Sk2DPathEffect`.
struct Sk2DPathEffectImpl {
    f_matrix: Matrix,
    f_inverse: Matrix,
    f_matrix_is_invertible: bool,
    kind: Sk2DPathEffectKind,
}

impl Sk2DPathEffectImpl {
    fn new(matrix: &Matrix, kind: Sk2DPathEffectKind) -> Self {
        let f_matrix = *matrix;
        let mut f_inverse = Matrix::new();
        // Calling invert will set the type mask on both matrices, making them thread safe.
        let f_matrix_is_invertible = f_matrix.invert(Some(&mut f_inverse));
        Sk2DPathEffectImpl {
            f_matrix,
            f_inverse,
            f_matrix_is_invertible,
            kind,
        }
    }

    fn get_matrix(&self) -> &Matrix {
        &self.f_matrix
    }

    /// New virtual, to be overridden by subclasses.
    ///
    /// This is called once from `filterPath`, and provides the `uv` parameter bounds for the path.
    /// Subsequent calls to `next_span` receive `u` and `v` values within these bounds, and then a
    /// call to `end` signals the end of processing.
    fn begin(&self, _uv_bounds: &IRect, _dst: &mut SkPathBuilder) {}

    /// Low-level virtual called per span of locations in the u-direction.
    ///
    /// The default implementation calls `next` repeatedly with each location.
    fn next_span(&self, x: i32, y: i32, ucount: i32, builder: &mut SkPathBuilder) {
        if !self.f_matrix_is_invertible {
            return;
        }

        let mat = self.get_matrix();
        let mut src = Point::new(
            x as Scalar + crate::scalar::SCALAR_HALF,
            y as Scalar + crate::scalar::SCALAR_HALF,
        );
        let mut x = x;
        let mut ucount = ucount;
        loop {
            let dst = mat.map_point(src);
            self.next(dst, x, y, builder);
            x += 1;
            src.x += crate::scalar::SCALAR_1;
            ucount -= 1;
            if ucount <= 0 {
                break;
            }
        }
    }

    /// Called once per location when the effect does not override `next_span`.
    fn next(&self, loc: Point, _u: i32, _v: i32, dst: &mut SkPathBuilder) {
        match &self.kind {
            Sk2DPathEffectKind::Path { f_path } => {
                dst.add_path_offset(f_path, loc.x, loc.y, AddPathMode::Append);
            }
            Sk2DPathEffectKind::Line { .. } => {
                // SkLine2DPathEffectImpl overrides nextSpan and never reaches the base next().
                unreachable!("SkLine2DPathEffectImpl must override nextSpan")
            }
        }
    }

    fn end(&self, _dst: &mut SkPathBuilder) {}

    /// `SkLine2DPathEffectImpl::nextSpan`.
    fn next_span_line(&self, x: i32, y: i32, ucount: i32, dst: &mut SkPathBuilder) {
        if ucount > 1 {
            let src = [
                Point::new(
                    x as Scalar + crate::scalar::SCALAR_HALF,
                    y as Scalar + crate::scalar::SCALAR_HALF,
                ),
                Point::new(
                    (x + ucount) as Scalar + crate::scalar::SCALAR_HALF,
                    y as Scalar + crate::scalar::SCALAR_HALF,
                ),
            ];
            let mut dst_pts = [Point::new(0.0, 0.0); 2];
            self.get_matrix().map_points(&mut dst_pts, &src);

            dst.move_to(dst_pts[0]);
            dst.line_to(dst_pts[1]);
        }
    }

    fn dispatch_next_span(&self, x: i32, y: i32, ucount: i32, dst: &mut SkPathBuilder) {
        match &self.kind {
            Sk2DPathEffectKind::Line { .. } => self.next_span_line(x, y, ucount, dst),
            Sk2DPathEffectKind::Path { .. } => self.next_span(x, y, ucount, dst),
        }
    }
}

impl SkPathEffectBase for Sk2DPathEffectImpl {
    fn on_filter_path(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        _cull_rect: Option<&Rect>,
        _ctm: &Matrix,
    ) -> bool {
        if !self.f_matrix_is_invertible {
            return false;
        }

        let tmp = src.make_transform(&self.f_inverse);
        let ir = tmp.get_bounds().round();
        if !ir.is_empty() {
            self.begin(&ir, dst);

            // Coverage of `tmp` clipped to `ir`, sampled at pixel centers. One span per run of
            // covered pixels in each row.
            let mut y = ir.top();
            while y < ir.bottom() {
                let mut x = ir.left();
                while x < ir.right() {
                    if !tmp.contains_xy(
                        x as Scalar + crate::scalar::SCALAR_HALF,
                        y as Scalar + crate::scalar::SCALAR_HALF,
                    ) {
                        x += 1;
                        continue;
                    }
                    let start = x;
                    x += 1;
                    while x < ir.right()
                        && tmp.contains_xy(
                            x as Scalar + crate::scalar::SCALAR_HALF,
                            y as Scalar + crate::scalar::SCALAR_HALF,
                        )
                    {
                        x += 1;
                    }
                    self.dispatch_next_span(start, y, x - start, dst);
                }
                y += 1;
            }

            self.end(dst);
        }

        if let Sk2DPathEffectKind::Line { f_width } = &self.kind {
            rec.set_stroke_style(*f_width, false);
        }
        true
    }

    fn compute_fast_bounds(&self, _bounds: Option<&mut Rect>) -> bool {
        // For simplicity, assume fast bounds cannot be computed
        false
    }
}

/// `SkLine2DPathEffect::Make`.
// The reference validates with `!(width >= 0)`; keeping the negated comparison rejects NaN.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub fn make_line_2d_path_effect(width: Scalar, matrix: &Matrix) -> Option<SkPathEffect> {
    if !(width >= 0.0) {
        return None;
    }
    Some(SkPathEffect::from_base(Sk2DPathEffectImpl::new(
        matrix,
        Sk2DPathEffectKind::Line { f_width: width },
    )))
}

/// `SkPath2DPathEffect::Make`.
pub fn make_path_2d_path_effect(matrix: &Matrix, path: &SkPath) -> Option<SkPathEffect> {
    Some(SkPathEffect::from_base(Sk2DPathEffectImpl::new(
        matrix,
        Sk2DPathEffectKind::Path {
            f_path: path.clone(),
        },
    )))
}

/// `SkLine2DPathEffect`.
pub struct SkLine2DPathEffect;

impl SkLine2DPathEffect {
    pub fn make(width: Scalar, matrix: &Matrix) -> Option<SkPathEffect> {
        make_line_2d_path_effect(width, matrix)
    }
}

/// `SkPath2DPathEffect`.
pub struct SkPath2DPathEffect;

impl SkPath2DPathEffect {
    pub fn make(matrix: &Matrix, path: &SkPath) -> Option<SkPathEffect> {
        make_path_2d_path_effect(matrix, path)
    }
}
