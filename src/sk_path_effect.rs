//! Port of `include/core/SkPathEffect.h`, `src/core/SkPathEffectBase.h` and
//! `src/core/SkPathEffect.cpp`.
//!
//! The reference exposes effects through `sk_sp<SkPathEffect>` over a virtual `SkPathEffectBase`
//! interface. The Rust port uses `Arc<dyn SkPathEffectBase>` inside [`SkPathEffect`], which
//! reproduces the shared-ownership and virtual-dispatch behavior without exposing the trait object
//! directly. `SkPairPathEffect` is a protected C++ helper with no observable API of its own, so the
//! compose/sum effects implement the pair management inline.
//!
//! Deferred to M22 (`SkFlattenable`/`SkReadBuffer`/`SkWriteBuffer`): `flatten`, `CreateProc`,
//! `getFactory`, `getTypeName`, `RegisterFlattenables`, `getFlattenableType` and
//! `SkPathEffect::Deserialize`. See `plan/handoffs/M08.md`.

use std::sync::Arc;

use crate::matrix::Matrix;
use crate::point::{Point, Vector};
use crate::rect::Rect;
use crate::scalar::Scalar;
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_stroke_rec::SkStrokeRec;

/// `SkPathEffectBase::PointData::PointFlag`.
pub const K_CIRCLES_POINT_FLAG: u32 = 0x01; // draw points as circles (instead of rects)
pub const K_USE_PATH_POINT_FLAG: u32 = 0x02; // draw points as stamps of the returned path
pub const K_USE_CLIP_POINT_FLAG: u32 = 0x04; // apply 'fClipRect' before drawing the points

/// `SkPathEffectBase::PointData`.
///
/// The reference owns `SkPoint* fPoints` allocated with `new[]` and tracks `fNumPoints`; the Rust
/// port owns a `Vec<Point>` whose length is the point count (`f_num_points` is kept in sync for
/// callers that mirror the reference's field). `fSize`, `fClipRect`, `fPath`, `fFirst` and `fLast`
/// keep their reference meaning.
#[derive(Clone, Default)]
pub struct PointData {
    /// flags that impact the drawing of the points
    pub f_flags: u32,
    /// the center point of each generated point
    pub f_points: Vec<Point>,
    /// number of points in fPoints
    pub f_num_points: i32,
    /// the size to draw the points
    pub f_size: Vector,
    /// clip required to draw the points (if `K_USE_CLIP_POINT_FLAG` is set)
    pub f_clip_rect: Rect,
    /// 'stamp' to be used at each point (if `K_USE_PATH_POINT_FLAG` is set)
    pub f_path: SkPath,
    /// If not empty, contains geometry for first point
    pub f_first: SkPath,
    /// If not empty, contains geometry for last point
    pub f_last: SkPath,
}

impl PointData {
    pub fn new() -> Self {
        PointData {
            f_flags: 0,
            f_points: Vec::new(),
            f_num_points: 0,
            f_size: Vector::new(crate::scalar::SCALAR_1, crate::scalar::SCALAR_1),
            f_clip_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            f_path: SkPath::new(),
            f_first: SkPath::new(),
            f_last: SkPath::new(),
        }
    }

    /// `SkPathEffectBase::PointData::points()`.
    pub fn points(&self) -> &[Point] {
        &self.f_points
    }
}

/// `SkPathEffectBase::DashInfo`.
#[derive(Clone, Debug, PartialEq)]
pub struct DashInfo {
    /// The reference holds an `SkSpan<const SkScalar>` into the effect's own interval storage; the
    /// Rust port returns an owned copy of the same values.
    pub f_intervals: Vec<Scalar>,
    pub f_phase: Scalar,
}

/// `SkPathEffectBase` — the virtual interface implemented by every concrete path effect.
pub trait SkPathEffectBase {
    /// `onFilterPath`. Returns true when the effect produced a usable result in `dst`; on false the
    /// caller ignores `dst` and `rec`.
    fn on_filter_path(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cull_rect: Option<&Rect>,
        ctm: &Matrix,
    ) -> bool;

    /// Path effects *requiring* a valid CTM should override to return true.
    fn on_needs_ctm(&self) -> bool {
        false
    }

    /// `onAsPoints`.
    fn on_as_points(
        &self,
        _results: &mut PointData,
        _src: &SkPath,
        _rec: &SkStrokeRec,
        _matrix: &Matrix,
        _cull_rect: Option<&Rect>,
    ) -> bool {
        false
    }

    /// `asADash`.
    fn as_a_dash(&self) -> Option<DashInfo> {
        None
    }

    /// Compute a conservative bounds for this effect given the path bounds. `bounds` is both input
    /// and output; `None` performs the reference's dry run (`computeFastBounds(nullptr)`).
    fn compute_fast_bounds(&self, bounds: Option<&mut Rect>) -> bool;
}

/// `SkPathEffect`.
#[derive(Clone)]
pub struct SkPathEffect {
    base: Arc<dyn SkPathEffectBase>,
}

impl SkPathEffect {
    pub fn from_base<T: SkPathEffectBase + 'static>(effect: T) -> Self {
        SkPathEffect {
            base: Arc::new(effect),
        }
    }

    /// Borrow the underlying effect interface (the reference's `as_PEB(effect)`).
    pub fn base(&self) -> &dyn SkPathEffectBase {
        self.base.as_ref()
    }

    /// `SkPathEffect::filterPath(dst, src, rec)` — identity CTM and no cull rect.
    pub fn filter_path(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
    ) -> bool {
        self.filter_path_with(dst, src, rec, None, &Matrix::new())
    }

    /// `SkPathEffect::filterPath(dst, src, rec, bounds, ctm)`.
    pub fn filter_path_with(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cull_rect: Option<&Rect>,
        ctm: &Matrix,
    ) -> bool {
        self.base.on_filter_path(dst, src, rec, cull_rect, ctm)
    }

    /// `SkPathEffectBase::asPoints`.
    pub fn as_points(
        &self,
        results: &mut PointData,
        src: &SkPath,
        rec: &SkStrokeRec,
        matrix: &Matrix,
        cull_rect: Option<&Rect>,
    ) -> bool {
        self.base.on_as_points(results, src, rec, matrix, cull_rect)
    }

    /// `SkPathEffect::needsCTM`.
    pub fn needs_ctm(&self) -> bool {
        self.base.on_needs_ctm()
    }

    /// `SkPathEffectBase::asADash`.
    pub fn as_a_dash(&self) -> Option<DashInfo> {
        self.base.as_a_dash()
    }

    /// `SkPathEffectBase::computeFastBounds`.
    pub fn compute_fast_bounds(&self, bounds: Option<&mut Rect>) -> bool {
        self.base.compute_fast_bounds(bounds)
    }

    /// `SkPathEffect::MakeSum`.
    ///
    /// Returns a path effect that applies each effect (first and second) to the original path and
    /// returns a path with the sum of these: `result = first(path) + second(path)`.
    pub fn make_sum(
        first: Option<SkPathEffect>,
        second: Option<SkPathEffect>,
    ) -> Option<SkPathEffect> {
        match (first, second) {
            (None, second) => second,
            (first, None) => first,
            (Some(first), Some(second)) => Some(SkPathEffect::from_base(SkSumPathEffect {
                f_pe0: first,
                f_pe1: second,
            })),
        }
    }

    /// `SkPathEffect::MakeCompose`.
    ///
    /// Returns a path effect that applies the inner effect to the path, and then applies the outer
    /// effect to the result of the inner's: `result = outer(inner(path))`.
    pub fn make_compose(
        outer: Option<SkPathEffect>,
        inner: Option<SkPathEffect>,
    ) -> Option<SkPathEffect> {
        match (outer, inner) {
            (None, inner) => inner,
            (outer, None) => outer,
            (Some(outer), Some(inner)) => Some(SkPathEffect::from_base(SkComposePathEffect {
                f_pe0: outer,
                f_pe1: inner,
            })),
        }
    }
}

/// `SkComposePathEffect`.
struct SkComposePathEffect {
    // fPE0 is the outer effect; fPE1 is the inner effect.
    f_pe0: SkPathEffect,
    f_pe1: SkPathEffect,
}

impl SkPathEffectBase for SkComposePathEffect {
    fn on_filter_path(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cull_rect: Option<&Rect>,
        ctm: &Matrix,
    ) -> bool {
        let tmp: SkPath;
        let mut ptr = src;

        if self
            .f_pe1
            .filter_path_with(builder, src, rec, cull_rect, ctm)
        {
            tmp = builder.detach(None);
            ptr = &tmp;
        }
        self.f_pe0
            .filter_path_with(builder, ptr, rec, cull_rect, ctm)
    }

    fn compute_fast_bounds(&self, bounds: Option<&mut Rect>) -> bool {
        // inner (fPE1) is computed first, automatically updating bounds before computing outer.
        match bounds {
            None => self.f_pe1.compute_fast_bounds(None) && self.f_pe0.compute_fast_bounds(None),
            Some(bounds) => {
                let mut value = *bounds;
                let inner = self.f_pe1.compute_fast_bounds(Some(&mut value));
                if !inner {
                    return false;
                }
                let outer = self.f_pe0.compute_fast_bounds(Some(&mut value));
                *bounds = value;
                outer
            }
        }
    }
}

/// `SkSumPathEffect`.
struct SkSumPathEffect {
    // fPE0 is the first effect; fPE1 is the second effect.
    f_pe0: SkPathEffect,
    f_pe1: SkPathEffect,
}

impl SkPathEffectBase for SkSumPathEffect {
    fn on_filter_path(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cull_rect: Option<&Rect>,
        ctm: &Matrix,
    ) -> bool {
        // always call both, even if the first one succeeds
        let filtered_first = self
            .f_pe0
            .filter_path_with(builder, src, rec, cull_rect, ctm);
        let filtered_second = self
            .f_pe1
            .filter_path_with(builder, src, rec, cull_rect, ctm);
        filtered_first || filtered_second
    }

    fn compute_fast_bounds(&self, bounds: Option<&mut Rect>) -> bool {
        // Unlike Compose(), PE0 modifies the path first for Sum
        match bounds {
            None => self.f_pe0.compute_fast_bounds(None) && self.f_pe1.compute_fast_bounds(None),
            Some(bounds) => {
                let mut value = *bounds;
                let first = self.f_pe0.compute_fast_bounds(Some(&mut value));
                if !first {
                    return false;
                }
                let second = self.f_pe1.compute_fast_bounds(Some(&mut value));
                *bounds = value;
                second
            }
        }
    }
}
