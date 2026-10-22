//! Port of `src/core/SkRasterClipStack.h`.
//!
//! The reference stores `Rec`s in an `SkTBlockList<Rec, 16>`; the port uses a `Vec<Rec>`. The
//! deferred-copy scheme (a save only marks the record, and the first clip mutation copies it) is
//! preserved exactly, as is the `fDisableAA` decision from `SkScan::PathRequiresTiling`.

use crate::matrix::Matrix;
use crate::rect::{IRect, Rect};
use crate::rrect::RRect;
use crate::sk_clip_op::SkClipOp;
use crate::sk_clip_stack::SkShaderHandle;
use crate::sk_path::SkPath;
use crate::sk_raster_clip::SkRasterClip;
use crate::sk_region::SkRegion;
use crate::sk_region_path::sk_scan_path_requires_tiling;

struct Rec {
    rc: SkRasterClip,
    /// 0 for a "normal" entry
    deferred_count: i32,
}

/// Port of `class SkRasterClipStack`.
pub struct SkRasterClipStack {
    stack: Vec<Rec>,
    root_bounds: IRect,
    disable_aa: bool,
    #[cfg(debug_assertions)]
    counter: i32,
}

impl SkRasterClipStack {
    /// `SkRasterClipStack(int width, int height)`.
    pub fn new(width: i32, height: i32) -> SkRasterClipStack {
        let root_bounds = IRect::new(0, 0, width, height);
        let disable_aa = sk_scan_path_requires_tiling(&root_bounds);
        SkRasterClipStack {
            stack: vec![Rec {
                rc: SkRasterClip::from_irect(&root_bounds),
                deferred_count: 0,
            }],
            root_bounds,
            disable_aa,
            #[cfg(debug_assertions)]
            counter: 0,
        }
    }

    /// `SkRasterClipStack::setNewSize`.
    pub fn set_new_size(&mut self, w: i32, h: i32) {
        self.root_bounds.set_xywh(0, 0, w, h);

        debug_assert_eq!(self.stack.len(), 1);
        let rec = self.stack.last_mut().unwrap();
        debug_assert_eq!(rec.deferred_count, 0);
        rec.rc.set_rect(&self.root_bounds);
    }

    /// `SkRasterClipStack::rc`.
    pub fn rc(&self) -> &SkRasterClip {
        &self.stack.last().unwrap().rc
    }

    /// `SkRasterClipStack::save`.
    pub fn save(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.counter += 1;
        }
        debug_assert!(self.stack.last().unwrap().deferred_count >= 0);
        self.stack.last_mut().unwrap().deferred_count += 1;
    }

    /// `SkRasterClipStack::restore`.
    pub fn restore(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.counter -= 1;
            debug_assert!(self.counter >= 0);
        }

        let back = self.stack.last_mut().unwrap();
        back.deferred_count -= 1;
        if back.deferred_count < 0 {
            debug_assert_eq!(back.deferred_count, -1);
            debug_assert!(self.stack.len() > 1);
            self.stack.pop();
        }
    }

    /// `SkRasterClipStack::clipRect`.
    pub fn clip_rect(&mut self, ctm: &Matrix, rect: &Rect, op: SkClipOp, aa: bool) {
        let aa = self.final_aa(aa);
        self.writable_rc().op_rect(rect, ctm, op, aa);
        self.validate();
    }

    /// `SkRasterClipStack::clipRRect`.
    pub fn clip_rrect(&mut self, ctm: &Matrix, rrect: &RRect, op: SkClipOp, aa: bool) {
        let aa = self.final_aa(aa);
        self.writable_rc().op_rrect(rrect, ctm, op, aa);
        self.validate();
    }

    /// `SkRasterClipStack::clipPath`.
    pub fn clip_path(&mut self, ctm: &Matrix, path: &SkPath, op: SkClipOp, aa: bool) {
        let aa = self.final_aa(aa);
        self.writable_rc().op_path(path, ctm, op, aa);
        self.validate();
    }

    /// `SkRasterClipStack::clipShader`.
    pub fn clip_shader(&mut self, shader: SkShaderHandle) {
        self.writable_rc().op_shader(shader);
        self.validate();
    }

    /// `SkRasterClipStack::clipRegion`.
    pub fn clip_region(&mut self, rgn: &SkRegion, op: SkClipOp) {
        self.writable_rc().op_region(rgn, op);
        self.validate();
    }

    /// `SkRasterClipStack::replaceClip`.
    pub fn replace_clip(&mut self, rect: &IRect) {
        let mut dev_rect = *rect;
        if !dev_rect.intersect(&self.root_bounds) {
            self.writable_rc().set_empty();
        } else {
            self.writable_rc().set_rect(&dev_rect);
        }
    }

    /// `SkRasterClipStack::validate`.
    pub fn validate(&self) {
        #[cfg(debug_assertions)]
        {
            let clip = self.rc();
            if self.root_bounds.is_empty() {
                debug_assert!(clip.is_empty());
            } else if !clip.is_empty() {
                debug_assert!(self.root_bounds.contains_irect(clip.get_bounds()));
            }
        }
    }

    fn writable_rc(&mut self) -> &mut SkRasterClip {
        debug_assert!(self.stack.last().unwrap().deferred_count >= 0);
        if self.stack.last().unwrap().deferred_count > 0 {
            self.stack.last_mut().unwrap().deferred_count -= 1;
            let copy = self.stack.last().unwrap().rc.clone();
            self.stack.push(Rec {
                rc: copy,
                deferred_count: 0,
            });
        }
        &mut self.stack.last_mut().unwrap().rc
    }

    fn final_aa(&self, aa: bool) -> bool {
        aa && !self.disable_aa
    }
}
