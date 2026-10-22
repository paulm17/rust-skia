//! Port of `src/core/SkRasterClip.h` and `src/core/SkRasterClip.cpp`.
//!
//! `SkAAClipBlitterWrapper` wraps an `SkBlitter`, which belongs to the raster pipeline (M14), so
//! that helper is deferred to **M14**. Everything else is implemented here.
//!
//! `sk_sp<SkShader>` is an opaque [`SkShaderHandle`] until M13, exactly as in `SkClipStack`.

use crate::matrix::Matrix;
use crate::rect::{IRect, Rect};
use crate::rrect::RRect;
use crate::scalar::{Scalar, scalar_floor_to_scalar};
use crate::sk_aa_clip::SkAAClip;
use crate::sk_clip_op::SkClipOp;
use crate::sk_clip_stack::ClipShader;
use crate::sk_clip_stack::SkShaderHandle;
use crate::sk_path::SkPath;
use crate::sk_region::{SkRegion, SkRegionOp};

/// Wraps an `SkRegion` and `SkAAClip`, so we have a single object that can represent either our BW
/// or antialiased clips.
#[derive(Clone, Debug, Default)]
pub struct SkRasterClip {
    f_bw: SkRegion,
    f_aa: SkAAClip,
    f_is_bw: bool,
    // these 2 are caches based on querying the right obj based on fIsBW
    f_is_empty: bool,
    f_is_rect: bool,
    // if present, this augments the clip, not replaces it
    f_shader: Option<ClipShader>,
}

impl SkRasterClip {
    /// `SkRasterClip()`.
    pub fn new() -> SkRasterClip {
        SkRasterClip {
            f_bw: SkRegion::new(),
            f_aa: SkAAClip::new(),
            f_is_bw: true,
            f_is_empty: true,
            f_is_rect: false,
            f_shader: None,
        }
    }

    /// `SkRasterClip(const SkIRect&)`.
    pub fn from_irect(bounds: &IRect) -> SkRasterClip {
        let mut rc = SkRasterClip {
            f_bw: SkRegion::from_irect(*bounds),
            f_aa: SkAAClip::new(),
            f_is_bw: true,
            f_is_empty: false,
            f_is_rect: false,
            f_shader: None,
        };
        rc.f_is_empty = rc.compute_is_empty(); // bounds might be empty, so compute
        rc.f_is_rect = !rc.f_is_empty;
        rc
    }

    /// `SkRasterClip(const SkRegion&)`.
    pub fn from_region(rgn: &SkRegion) -> SkRasterClip {
        let mut rc = SkRasterClip {
            f_bw: rgn.clone(),
            f_aa: SkAAClip::new(),
            f_is_bw: true,
            f_is_empty: false,
            f_is_rect: false,
            f_shader: None,
        };
        rc.f_is_empty = rc.compute_is_empty(); // bounds might be empty, so compute
        rc.f_is_rect = !rc.f_is_empty;
        rc
    }

    /// `SkRasterClip(const SkPath&, const SkIRect&, bool)`.
    pub fn from_path(path: &SkPath, bounds: &IRect, do_aa: bool) -> SkRasterClip {
        let mut rc = SkRasterClip::new();
        if do_aa {
            rc.f_is_bw = false;
            rc.f_aa.set_path(path, bounds, true);
        } else {
            rc.f_is_bw = true;
            rc.f_bw.set_path(path, &SkRegion::from_irect(*bounds));
        }
        rc.f_is_empty = rc.compute_is_empty(); // bounds might be empty, so compute
        rc.f_is_rect = rc.compute_is_rect();
        rc
    }

    /// `SkRasterClip::isBW`.
    pub fn is_bw(&self) -> bool {
        self.f_is_bw
    }

    /// `SkRasterClip::isAA`.
    pub fn is_aa(&self) -> bool {
        !self.f_is_bw
    }

    /// `SkRasterClip::bwRgn`.
    pub fn bw_rgn(&self) -> &SkRegion {
        debug_assert!(self.f_is_bw);
        &self.f_bw
    }

    /// `SkRasterClip::aaRgn`.
    pub fn aa_rgn(&self) -> &SkAAClip {
        debug_assert!(!self.f_is_bw);
        &self.f_aa
    }

    /// `SkRasterClip::isEmpty`.
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.compute_is_empty(), self.f_is_empty);
        self.f_is_empty
    }

    /// `SkRasterClip::isRect`.
    pub fn is_rect(&self) -> bool {
        debug_assert_eq!(self.compute_is_rect(), self.f_is_rect);
        self.f_is_rect
    }

    /// `SkRasterClip::isComplex`.
    pub fn is_complex(&self) -> bool {
        if self.f_is_bw {
            self.f_bw.is_complex()
        } else {
            !self.f_aa.is_empty()
        }
    }

    /// `SkRasterClip::getBounds`.
    pub fn get_bounds(&self) -> &IRect {
        if self.f_is_bw {
            self.f_bw.get_bounds()
        } else {
            self.f_aa.get_bounds()
        }
    }

    /// `SkRasterClip::setEmpty`.
    pub fn set_empty(&mut self) -> bool {
        self.f_is_bw = true;
        self.f_bw.set_empty();
        self.f_aa.set_empty();
        self.f_is_empty = true;
        self.f_is_rect = false;
        false
    }

    /// `SkRasterClip::setRect`.
    pub fn set_rect(&mut self, rect: &IRect) -> bool {
        self.f_is_bw = true;
        self.f_aa.set_empty();
        self.f_is_rect = self.f_bw.set_rect(rect);
        self.f_is_empty = !self.f_is_rect;
        self.f_is_rect
    }

    fn compute_is_empty(&self) -> bool {
        if self.f_is_bw {
            self.f_bw.is_empty()
        } else {
            self.f_aa.is_empty()
        }
    }

    fn compute_is_rect(&self) -> bool {
        if self.f_is_bw {
            self.f_bw.is_rect()
        } else {
            self.f_aa.is_rect()
        }
    }

    fn update_cache_and_return_non_empty(&mut self, detect_aa_rect: bool) -> bool {
        self.f_is_empty = self.compute_is_empty();

        // detect that our computed AA is really just a (hard-edged) rect
        if detect_aa_rect && !self.f_is_empty && !self.f_is_bw && self.f_aa.is_rect() {
            let bounds = *self.f_aa.get_bounds();
            self.f_bw.set_rect(&bounds);
            self.f_aa.set_empty(); // don't need this anymore
            self.f_is_bw = true;
        }

        self.f_is_rect = self.compute_is_rect();
        !self.f_is_empty
    }

    fn convert_to_aa(&mut self) {
        debug_assert!(self.f_is_bw);
        let bw = self.f_bw.clone();
        self.f_aa.set_region(&bw);
        self.f_is_bw = false;

        // since we are being explicitly asked to convert-to-aa, we pass false so we don't
        // "optimize" ourselves back to BW.
        let _ = self.update_cache_and_return_non_empty(false);
    }

    /// `SkRasterClip::op(const SkIRect&, SkClipOp)`.
    pub fn op_irect(&mut self, rect: &IRect, op: SkClipOp) -> bool {
        if self.f_is_bw {
            self.f_bw.op_irect(rect, SkRegionOp::from(op));
        } else {
            self.f_aa.op_irect(rect, op);
        }
        self.update_cache_and_return_non_empty(true)
    }

    /// `SkRasterClip::op(const SkRegion&, SkClipOp)`.
    pub fn op_region(&mut self, rgn: &SkRegion, op: SkClipOp) -> bool {
        if self.f_is_bw {
            let _ = self.f_bw.op_region(rgn, SkRegionOp::from(op));
        } else {
            let mut tmp = SkAAClip::new();
            tmp.set_region(rgn);
            let _ = self.f_aa.op(&tmp, op);
        }
        self.update_cache_and_return_non_empty(true)
    }

    /// `SkRasterClip::op(const SkRect&, const SkMatrix&, SkClipOp, bool)`.
    pub fn op_rect(
        &mut self,
        local_rect: &Rect,
        matrix: &Matrix,
        op: SkClipOp,
        mut do_aa: bool,
    ) -> bool {
        let is_scale_trans = matrix.is_scale_translate();
        if !is_scale_trans {
            return self.op_path(&SkPath::rect_default(local_rect), matrix, op, do_aa);
        }

        let dev_rect = matrix.map_rect(local_rect);
        if self.f_is_bw && do_aa {
            // check that the rect really needs aa, or is it close enought to
            // integer boundaries that we can just treat it as a BW rect?
            if nearly_integral(dev_rect.left)
                && nearly_integral(dev_rect.top)
                && nearly_integral(dev_rect.right)
                && nearly_integral(dev_rect.bottom)
            {
                do_aa = false;
            }
        }

        if self.f_is_bw && !do_aa {
            let _ = self.f_bw.op_irect(&dev_rect.round(), SkRegionOp::from(op));
        } else {
            if self.f_is_bw {
                self.convert_to_aa();
            }
            let _ = self.f_aa.op_rect(&dev_rect, op, do_aa);
        }
        self.update_cache_and_return_non_empty(true)
    }

    /// `SkRasterClip::op(const SkRRect&, const SkMatrix&, SkClipOp, bool)`.
    pub fn op_rrect(&mut self, rrect: &RRect, matrix: &Matrix, op: SkClipOp, do_aa: bool) -> bool {
        self.op_path(&SkPath::rrect_default(rrect), matrix, op, do_aa)
    }

    /// `SkRasterClip::op(const SkPath&, const SkMatrix&, SkClipOp, bool)`.
    pub fn op_path(&mut self, path: &SkPath, matrix: &Matrix, op: SkClipOp, do_aa: bool) -> bool {
        let dev_path = path.make_transform(matrix);

        // Since op is either intersect or difference, the clip is always shrinking; that means we
        // can always use our current bounds as the limiting factor for region/aaclip operations.
        if self.is_rect() && op == SkClipOp::Intersect {
            // However, in the relatively common case of intersecting a new path with a rectangular
            // clip, it's faster to convert the path into a region/aa-mask in place than evaluate
            // the actual intersection. See skbug.com/40043482
            if do_aa && self.f_is_bw {
                self.convert_to_aa();
            }
            let bounds = *self.get_bounds();
            if self.f_is_bw {
                self.f_bw.set_path(&dev_path, &SkRegion::from_irect(bounds));
            } else {
                self.f_aa.set_path(&dev_path, &bounds, do_aa);
            }
            self.update_cache_and_return_non_empty(true)
        } else {
            let bounds = *self.get_bounds();
            let clip = SkRasterClip::from_path(&dev_path, &bounds, do_aa);
            self.op_raster_clip(&clip, op)
        }
    }

    /// `SkRasterClip::op(sk_sp<SkShader>)`.
    pub fn op_shader(&mut self, shader: SkShaderHandle) -> bool {
        let new_shader = ClipShader::Shader(shader);
        if self.f_shader.is_none() {
            self.f_shader = Some(new_shader);
        } else {
            let prev = self.f_shader.take().unwrap();
            self.f_shader = Some(ClipShader::SrcIn {
                src: Box::new(new_shader),
                dst: Box::new(prev),
            });
        }
        !self.is_empty()
    }

    /// `SkRasterClip::op(const SkRasterClip&, SkClipOp)`.
    pub fn op_raster_clip(&mut self, clip: &SkRasterClip, op: SkClipOp) -> bool {
        if self.is_bw() && clip.is_bw() {
            let _ = self.f_bw.op_region(clip.bw_rgn(), SkRegionOp::from(op));
        } else {
            if self.is_bw() {
                self.convert_to_aa();
            }
            if clip.is_bw() {
                let mut tmp = SkAAClip::new();
                tmp.set_region(clip.bw_rgn());
                let _ = self.f_aa.op(&tmp, op);
            } else {
                let other = clip.aa_rgn().clone();
                let _ = self.f_aa.op(&other, op);
            }
        }
        self.update_cache_and_return_non_empty(true)
    }

    /// `SkRasterClip::translate`.
    pub fn translate(&self, dx: i32, dy: i32, dst: &mut SkRasterClip) {
        if self.is_empty() {
            dst.set_empty();
            return;
        }
        if 0 == (dx | dy) {
            *dst = self.clone();
            return;
        }

        dst.f_is_bw = self.f_is_bw;
        if self.f_is_bw {
            self.f_bw.translate_to(dx, dy, &mut dst.f_bw);
            dst.f_aa.set_empty();
        } else {
            self.f_aa.translate(dx, dy, &mut dst.f_aa);
            dst.f_bw.set_empty();
        }
        dst.update_cache_and_return_non_empty(true);
    }

    /// `SkRasterClip::quickContains`.
    pub fn quick_contains(&self, rect: &IRect) -> bool {
        if self.f_is_bw {
            self.f_bw.quick_contains(rect)
        } else {
            self.f_aa.quick_contains(rect)
        }
    }

    /// `SkRasterClip::quickReject`.
    pub fn quick_reject(&self, rect: &IRect) -> bool {
        !IRect::intersects_irects(self.get_bounds(), rect)
    }

    /// `SkRasterClip::clipShader`.
    pub fn clip_shader(&self) -> Option<&ClipShader> {
        self.f_shader.as_ref()
    }
}

/// Our antialiasing currently has a granularity of 1/4 of a pixel along each axis. Thus we can
/// treat an axis coordinate as an integer if it differs from its nearest int by < half of that
/// value (1/8 in this case).
fn nearly_integral(x: Scalar) -> bool {
    let domain: Scalar = 1.0 / 4.0;
    let half_domain: Scalar = domain / 2.0;

    let x = x + half_domain;
    x - scalar_floor_to_scalar(x) < domain
}
