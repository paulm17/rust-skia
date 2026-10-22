//! Port of `src/core/SkClipStackDevice.h` and `src/core/SkClipStackDevice.cpp`.
//!
//! The reference derives from `SkDevice`, whose port is owned by a later milestone. The port keeps
//! the same clip-stack behavior and method set, and carries the four `SkDevice` facts that every
//! method here reads: the image bounds (`width`/`height`), `localToDevice`, `globalToDevice` and
//! the device origin. When `SkDevice` lands, these become accessors on the base type without
//! changing this module's logic.

use crate::ipoint::IPoint;
use crate::matrix::Matrix;
use crate::rect::IRect;
use crate::rrect::RRect;
use crate::sk_clip_op::SkClipOp;
use crate::sk_clip_stack::{SkClipStack, SkShaderHandle};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_region::BoundsType;
use crate::sk_region::SkRegion;

/// Port of `class SkClipStackDevice`.
#[derive(Clone, Debug)]
pub struct SkClipStackDevice {
    width: i32,
    height: i32,
    local_to_device: Matrix,
    global_to_device: Matrix,
    origin: IPoint,
    clip_stack: SkClipStack,
}

impl Default for SkClipStackDevice {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl SkClipStackDevice {
    /// The `SkDevice(info, props)` constructor for an image of `width`×`height`, with the default
    /// identity local/global transforms and a zero origin.
    pub fn new(width: i32, height: i32) -> SkClipStackDevice {
        SkClipStackDevice {
            width,
            height,
            local_to_device: Matrix::default(),
            global_to_device: Matrix::default(),
            origin: IPoint::new(0, 0),
            clip_stack: SkClipStack::new(),
        }
    }

    /// Constructor with the transforms/origin the reference reads from its `SkDevice` base.
    pub fn with_device_state(
        width: i32,
        height: i32,
        local_to_device: Matrix,
        global_to_device: Matrix,
        origin: IPoint,
    ) -> SkClipStackDevice {
        SkClipStackDevice {
            width,
            height,
            local_to_device,
            global_to_device,
            origin,
            clip_stack: SkClipStack::new(),
        }
    }

    /// `SkClipStackDevice::cs()`.
    pub fn cs(&self) -> &SkClipStack {
        &self.clip_stack
    }

    /// `SkClipStackDevice::cs()` (mutable).
    pub fn cs_mut(&mut self) -> &mut SkClipStack {
        &mut self.clip_stack
    }

    /// `SkDevice::width()`.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// `SkDevice::height()`.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// `SkDevice::localToDevice()`.
    pub fn local_to_device(&self) -> &Matrix {
        &self.local_to_device
    }

    /// `SkDevice::globalToDevice()`.
    pub fn global_to_device(&self) -> &Matrix {
        &self.global_to_device
    }

    /// `SkDevice::getOrigin()`.
    pub fn get_origin(&self) -> IPoint {
        self.origin
    }

    /// `SkDevice::imageInfo().bounds()`.
    pub fn image_bounds(&self) -> IRect {
        IRect::new(0, 0, self.width, self.height)
    }

    /// `SkClipStackDevice::devClipBounds`.
    pub fn dev_clip_bounds(&self) -> IRect {
        let r = self.clip_stack.bounds(&self.image_bounds()).round_out();
        if !r.is_empty() {
            debug_assert!(self.image_bounds().contains_irect(&r));
        }
        r
    }

    /// `SkClipStackDevice::pushClipStack`.
    pub fn push_clip_stack(&mut self) {
        self.clip_stack.save();
    }

    /// `SkClipStackDevice::popClipStack`.
    pub fn pop_clip_stack(&mut self) {
        self.clip_stack.restore();
    }

    /// `SkClipStackDevice::clipRect`.
    pub fn clip_rect(&mut self, rect: &crate::rect::Rect, op: SkClipOp, aa: bool) {
        self.clip_stack
            .clip_rect(rect, &self.local_to_device, op, aa);
    }

    /// `SkClipStackDevice::clipRRect`.
    pub fn clip_rrect(&mut self, rrect: &RRect, op: SkClipOp, aa: bool) {
        self.clip_stack
            .clip_rrect(rrect, &self.local_to_device, op, aa);
    }

    /// `SkClipStackDevice::clipPath`.
    pub fn clip_path(&mut self, path: &SkPath, op: SkClipOp, aa: bool) {
        self.clip_stack
            .clip_path(path, &self.local_to_device, op, aa);
    }

    /// `SkClipStackDevice::onClipShader`.
    pub fn on_clip_shader(&mut self, shader: SkShaderHandle) {
        self.clip_stack.clip_shader(shader);
    }

    /// `SkClipStackDevice::clipRegion`.
    pub fn clip_region(&mut self, rgn: &SkRegion, op: SkClipOp) {
        let origin = self.get_origin();
        let mut builder = SkPathBuilder::new();
        rgn.add_boundary_path(&mut builder);
        builder.transform(&Matrix::translate(-origin.x as f32, -origin.y as f32));
        self.clip_stack
            .clip_path(&builder.detach(None), &Matrix::default(), op, false);
    }

    /// `SkClipStackDevice::replaceClip`.
    pub fn replace_clip(&mut self, rect: &IRect) {
        let device_rect = self
            .global_to_device
            .map_rect(&crate::rect::Rect::from(*rect));
        self.clip_stack.replace_clip(&device_rect, false);
    }

    /// `SkClipStackDevice::isClipAntiAliased`.
    pub fn is_clip_anti_aliased(&self) -> bool {
        let mut iter = crate::sk_clip_stack::B2TIter::new(&self.clip_stack);
        while let Some(element) = iter.next() {
            if element.is_aa() {
                return true;
            }
        }
        false
    }

    /// `SkClipStackDevice::isClipWideOpen`.
    pub fn is_clip_wide_open(&self) -> bool {
        self.clip_stack
            .quick_contains(&crate::rect::Rect::make_iwh(self.width(), self.height()))
    }

    /// `SkClipStackDevice::isClipEmpty`.
    pub fn is_clip_empty(&self) -> bool {
        self.clip_stack
            .is_empty(&IRect::new(0, 0, self.width(), self.height()))
    }

    /// `SkClipStackDevice::isClipRect`.
    pub fn is_clip_rect(&self) -> bool {
        if self.is_clip_wide_open() {
            return true;
        } else if self.is_clip_empty() {
            return false;
        }

        let mut bounds = crate::rect::Rect::make_empty();
        let mut bound_type = BoundsType::Normal;
        let mut is_intersection_of_rects = false;
        self.clip_stack.get_bounds_with_intersection(
            &mut bounds,
            &mut bound_type,
            &mut is_intersection_of_rects,
        );
        is_intersection_of_rects && bound_type == BoundsType::Normal
    }

    /// `SkClipStackDevice::android_utils_clipAsRgn`.
    pub fn android_utils_clip_as_rgn(&self, rgn: &mut SkRegion) {
        let mut bounds = crate::rect::Rect::make_empty();
        let mut bound_type = BoundsType::Normal;
        let mut is_intersection_of_rects = false;
        self.clip_stack.get_bounds_with_intersection(
            &mut bounds,
            &mut bound_type,
            &mut is_intersection_of_rects,
        );
        if is_intersection_of_rects && BoundsType::Normal == bound_type {
            rgn.set_rect(&bounds.round());
        } else {
            let bounds_rgn = SkRegion::from_irect(IRect::new(0, 0, self.width(), self.height()));

            *rgn = bounds_rgn.clone();
            let mut iter = crate::sk_clip_stack::B2TIter::new(&self.clip_stack);
            while let Some(elem) = iter.next() {
                let tmp_path = elem.as_device_space_path();
                let mut tmp_rgn = SkRegion::new();
                tmp_rgn.set_path(&tmp_path, &bounds_rgn);
                if elem.is_replace_op() {
                    // All replace elements are rectangles
                    rgn.set_rect(&elem.get_device_space_rect().round());
                } else {
                    rgn.op_region(&tmp_rgn, crate::sk_region::SkRegionOp::from(elem.get_op()));
                }
            }
        }
    }
}
