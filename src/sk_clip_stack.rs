//! Port of `src/core/SkClipStack.h` and `src/core/SkClipStack.cpp`.
//!
//! The reference keeps its elements in an `SkDeque` of placement-new'd `Element` objects. The port
//! stores them bottom-to-top in a `Vec<Element>`, which preserves the reference's ordering and all
//! iterator semantics (`next` from the bottom, `prev` from the top).
//!
//! `sk_sp<SkShader>` does not exist yet (`include/core/SkShader.h` is owned by M13), so a clip
//! shader is held as an opaque [`SkShaderHandle`] and the `SkShaders::Blend(kSrcIn, ...)`
//! composition is recorded structurally in [`ClipShader`] for M13 to materialize.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::matrix::Matrix;
use crate::rect::{IRect, Rect};
use crate::rect_priv::RectPriv;
use crate::rrect::{RRect, Type as RRectType};
use crate::sk_clip_op::SkClipOp;
use crate::sk_geometry::SkPathDirection;
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_region::BoundsType;

/// Opaque `sk_sp<SkShader>` placeholder until `include/core/SkShader.h` (M13) exists.
#[derive(Clone)]
pub struct SkShaderHandle(Arc<dyn Any + Send + Sync>);

impl SkShaderHandle {
    pub fn new<T: Any + Send + Sync>(shader: T) -> SkShaderHandle {
        SkShaderHandle(Arc::new(shader))
    }

    /// Pointer identity, matching the reference's `getShader() == other.getShader()`.
    pub fn ptr_eq(&self, other: &SkShaderHandle) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn as_any(&self) -> &(dyn Any + Send + Sync) {
        &*self.0
    }
}

impl core::fmt::Debug for SkShaderHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SkShaderHandle(..)")
    }
}

/// `SkShaders::Blend(SkBlendMode::kSrcIn, src, dst)` as used by `SkClipStack`/`SkRasterClip`.
///
/// The shader subsystem that evaluates this tree is M13; until then the composition is stored
/// exactly as the reference builds it so no information is lost.
#[derive(Clone, Debug)]
pub enum ClipShader {
    Shader(SkShaderHandle),
    SrcIn {
        src: Box<ClipShader>,
        dst: Box<ClipShader>,
    },
}

impl ClipShader {
    fn handle(&self) -> Option<&SkShaderHandle> {
        match self {
            ClipShader::Shader(h) => Some(h),
            ClipShader::SrcIn { .. } => None,
        }
    }

    fn same(&self, other: &ClipShader) -> bool {
        match (self, other) {
            (ClipShader::Shader(a), ClipShader::Shader(b)) => a.ptr_eq(b),
            (
                ClipShader::SrcIn {
                    src: asrc,
                    dst: adst,
                },
                ClipShader::SrcIn {
                    src: bsrc,
                    dst: bdst,
                },
            ) => asrc.same(bsrc) && adst.same(bdst),
            _ => false,
        }
    }
}

/// `SkClipStack::Element::DeviceSpaceType`. The discriminants match the reference enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum DeviceSpaceType {
    Empty = 0,
    Rect = 1,
    RRect = 2,
    Path = 3,
    Shader = 4,
}

impl DeviceSpaceType {
    pub const K_LAST_TYPE: DeviceSpaceType = DeviceSpaceType::Shader;
    pub const K_TYPE_CNT: usize = 5;

    pub const fn to_i32(self) -> i32 {
        self as i32
    }
}

/// `SkClipStack::Element::FillCombo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillCombo {
    PrevCur = 0,
    PrevInvCur = 1,
    InvPrevCur = 2,
    InvPrevInvCur = 3,
}

impl FillCombo {
    const fn from_bits(bits: u8) -> FillCombo {
        match bits {
            0 => FillCombo::PrevCur,
            1 => FillCombo::PrevInvCur,
            2 => FillCombo::InvPrevCur,
            _ => FillCombo::InvPrevInvCur,
        }
    }
}

/// `SkClipStack::Element`.
#[derive(Clone)]
pub struct Element {
    device_space_path: Option<SkPath>,
    device_space_rrect: RRect,
    shader: Option<ClipShader>,
    save_count: i32,
    op: SkClipOp,
    device_space_type: DeviceSpaceType,
    do_aa: bool,
    is_replace: bool,
    finite_bound_type: BoundsType,
    finite_bound: Rect,
    is_intersection_of_rects: bool,
    gen_id: u32,
}

impl Default for Element {
    fn default() -> Self {
        Self::new_empty(0)
    }
}

impl Element {
    /// `Element()`.
    pub fn new() -> Element {
        let mut e = Element {
            device_space_path: None,
            device_space_rrect: RRect::make_empty(),
            shader: None,
            save_count: 0,
            op: SkClipOp::Intersect,
            device_space_type: DeviceSpaceType::Empty,
            do_aa: false,
            is_replace: false,
            finite_bound_type: BoundsType::InsideOut,
            finite_bound: Rect::make_empty(),
            is_intersection_of_rects: false,
            gen_id: K_INVALID_GEN_ID,
        };
        e.init_common(0, SkClipOp::Intersect, false);
        e.set_empty();
        e
    }

    fn new_empty(save_count: i32) -> Element {
        let mut e = Element {
            device_space_path: None,
            device_space_rrect: RRect::make_empty(),
            shader: None,
            save_count,
            op: SkClipOp::Intersect,
            device_space_type: DeviceSpaceType::Empty,
            do_aa: false,
            is_replace: false,
            finite_bound_type: BoundsType::InsideOut,
            finite_bound: Rect::make_empty(),
            is_intersection_of_rects: false,
            gen_id: K_INVALID_GEN_ID,
        };
        e.init_common(save_count, SkClipOp::Intersect, false);
        e.set_empty();
        e
    }

    /// `Element(const SkRect&, const SkMatrix&, SkClipOp, bool)`.
    pub fn from_rect(rect: &Rect, m: &Matrix, op: SkClipOp, do_aa: bool) -> Element {
        let mut e = Element::new_raw(0);
        e.init_rect(0, rect, m, op, do_aa);
        e
    }

    /// `Element(const SkRRect&, const SkMatrix&, SkClipOp, bool)`.
    pub fn from_rrect(rrect: &RRect, m: &Matrix, op: SkClipOp, do_aa: bool) -> Element {
        let mut e = Element::new_raw(0);
        e.init_rrect(0, rrect, m, op, do_aa);
        e
    }

    /// `Element(const SkPath&, const SkMatrix&, SkClipOp, bool)`.
    pub fn from_path(path: &SkPath, m: &Matrix, op: SkClipOp, do_aa: bool) -> Element {
        let mut e = Element::new_raw(0);
        e.init_path(0, path, m, op, do_aa);
        e
    }

    /// `Element(sk_sp<SkShader>)`.
    pub fn from_shader(shader: SkShaderHandle) -> Element {
        let mut e = Element::new_raw(0);
        e.init_shader(0, ClipShader::Shader(shader));
        e
    }

    /// `Element(const SkRect&, bool)` (the replace-rect constructor).
    pub fn from_replace_rect(rect: &Rect, do_aa: bool) -> Element {
        let mut e = Element::new_raw(0);
        e.init_replace_rect(0, rect, do_aa);
        e
    }

    fn new_raw(save_count: i32) -> Element {
        Element {
            device_space_path: None,
            device_space_rrect: RRect::make_empty(),
            shader: None,
            save_count,
            op: SkClipOp::Intersect,
            device_space_type: DeviceSpaceType::Empty,
            do_aa: false,
            is_replace: false,
            finite_bound_type: BoundsType::InsideOut,
            finite_bound: Rect::make_empty(),
            is_intersection_of_rects: false,
            gen_id: K_INVALID_GEN_ID,
        }
    }

    /// `Element::getDeviceSpaceType`.
    pub fn get_device_space_type(&self) -> DeviceSpaceType {
        self.device_space_type
    }

    /// `Element::getSaveCount`.
    pub fn get_save_count(&self) -> i32 {
        self.save_count
    }

    /// `Element::getDeviceSpacePath`.
    pub fn get_device_space_path(&self) -> &SkPath {
        debug_assert_eq!(DeviceSpaceType::Path, self.device_space_type);
        self.device_space_path.as_ref().unwrap()
    }

    /// `Element::getDeviceSpaceRRect`.
    pub fn get_device_space_rrect(&self) -> &RRect {
        debug_assert_eq!(DeviceSpaceType::RRect, self.device_space_type);
        &self.device_space_rrect
    }

    /// `Element::getDeviceSpaceRect`.
    pub fn get_device_space_rect(&self) -> &Rect {
        debug_assert_eq!(DeviceSpaceType::Rect, self.device_space_type);
        debug_assert!(
            self.device_space_rrect.get_type() == RRectType::Rect
                || self.device_space_rrect.get_type() == RRectType::Empty
        );
        self.device_space_rrect.get_bounds()
    }

    /// `Element::refShader`. Returns the whole recorded composition for M13.
    pub fn ref_shader(&self) -> Option<&ClipShader> {
        self.shader.as_ref()
    }

    /// `Element::getShader`. Only a plain shader handle is observable without the M13 shader API.
    pub fn get_shader(&self) -> Option<&SkShaderHandle> {
        self.shader.as_ref().and_then(|s| s.handle())
    }

    /// `Element::getOp`.
    pub fn get_op(&self) -> SkClipOp {
        self.op
    }

    /// `Element::isReplaceOp`.
    pub fn is_replace_op(&self) -> bool {
        self.is_replace
    }

    /// `Element::asDeviceSpacePath`.
    pub fn as_device_space_path(&self) -> SkPath {
        let mut builder = SkPathBuilder::new();
        match self.device_space_type {
            DeviceSpaceType::Empty => {}
            DeviceSpaceType::Rect => {
                builder.add_rect(self.get_device_space_rect(), SkPathDirection::Cw, 0);
            }
            DeviceSpaceType::RRect => {
                builder.add_rrect(&self.device_space_rrect, SkPathDirection::Cw, 0);
            }
            DeviceSpaceType::Path => return self.device_space_path.as_ref().unwrap().clone(),
            DeviceSpaceType::Shader => {
                builder.add_rect(&RectPriv::make_large_s32(), SkPathDirection::Cw, 0);
            }
        }
        builder.detach(None)
    }

    /// `Element::asDeviceSpaceRRect`.
    pub fn as_device_space_rrect(&self) -> &RRect {
        debug_assert_ne!(DeviceSpaceType::Path, self.device_space_type);
        &self.device_space_rrect
    }

    /// `Element::isAA`.
    pub fn is_aa(&self) -> bool {
        self.do_aa
    }

    /// `Element::getGenID`.
    pub fn get_gen_id(&self) -> u32 {
        debug_assert_ne!(K_INVALID_GEN_ID, self.gen_id);
        self.gen_id
    }

    /// `Element::getBounds`.
    pub fn get_bounds(&self) -> Rect {
        match self.device_space_type {
            DeviceSpaceType::Rect | DeviceSpaceType::RRect => *self.device_space_rrect.get_bounds(),
            DeviceSpaceType::Path => *self.device_space_path.as_ref().unwrap().get_bounds(),
            // Shaders have infinite bounds since any pixel could have clipped or full coverage
            // (which is different from wide-open, where every pixel has 1.0 coverage, or empty
            // where every pixel has 0.0 coverage).
            DeviceSpaceType::Shader => RectPriv::make_large_s32(),
            DeviceSpaceType::Empty => Rect::make_empty(),
        }
    }

    /// `Element::contains(const SkRect&)`.
    pub fn contains_rect(&self, rect: &Rect) -> bool {
        match self.device_space_type {
            DeviceSpaceType::Rect => self.get_device_space_rect().contains_rect(rect),
            DeviceSpaceType::RRect => self.device_space_rrect.contains_rect(rect),
            DeviceSpaceType::Path => self
                .device_space_path
                .as_ref()
                .unwrap()
                .conservatively_contains_rect(rect),
            DeviceSpaceType::Empty | DeviceSpaceType::Shader => false,
        }
    }

    /// `Element::contains(const SkRRect&)`.
    pub fn contains_rrect(&self, rrect: &RRect) -> bool {
        match self.device_space_type {
            DeviceSpaceType::Rect => self
                .get_device_space_rect()
                .contains_rect(rrect.get_bounds()),
            DeviceSpaceType::RRect => {
                // We don't currently have a generalized rrect-rrect containment.
                self.device_space_rrect.contains_rect(rrect.get_bounds())
                    || rrect == &self.device_space_rrect
            }
            DeviceSpaceType::Path => self
                .device_space_path
                .as_ref()
                .unwrap()
                .conservatively_contains_rect(rrect.get_bounds()),
            DeviceSpaceType::Empty | DeviceSpaceType::Shader => false,
        }
    }

    /// `Element::isInverseFilled`.
    pub fn is_inverse_filled(&self) -> bool {
        DeviceSpaceType::Path == self.device_space_type
            && self
                .device_space_path
                .as_ref()
                .unwrap()
                .is_inverse_fill_type()
    }

    fn init_common(&mut self, save_count: i32, op: SkClipOp, do_aa: bool) {
        self.save_count = save_count;
        self.op = op;
        self.do_aa = do_aa;
        self.is_replace = false;
        // A default of inside-out and empty bounds means the bounds are effectively void as it
        // indicates that nothing is known to be outside the clip.
        self.finite_bound_type = BoundsType::InsideOut;
        self.finite_bound.set_empty();
        self.is_intersection_of_rects = false;
        self.gen_id = K_INVALID_GEN_ID;
    }

    fn init_rect(&mut self, save_count: i32, rect: &Rect, m: &Matrix, op: SkClipOp, do_aa: bool) {
        if m.rect_stays_rect() {
            let dev_rect = m.map_rect(rect);
            self.device_space_rrect.set_rect(&dev_rect);
            self.device_space_type = DeviceSpaceType::Rect;
            self.init_common(save_count, op, do_aa);
            return;
        }
        self.init_as_path(save_count, &SkPath::rect_default(rect), m, op, do_aa);
    }

    fn init_rrect(
        &mut self,
        save_count: i32,
        rrect: &RRect,
        m: &Matrix,
        op: SkClipOp,
        do_aa: bool,
    ) {
        if let Some(rr) = rrect.transform(m) {
            self.device_space_rrect = rr;
            let kind = self.device_space_rrect.get_type();
            if RRectType::Rect == kind || RRectType::Empty == kind {
                self.device_space_type = DeviceSpaceType::Rect;
            } else {
                self.device_space_type = DeviceSpaceType::RRect;
            }
            self.init_common(save_count, op, do_aa);
            return;
        }
        self.init_as_path(save_count, &SkPath::rrect_default(rrect), m, op, do_aa);
    }

    fn init_path(&mut self, save_count: i32, path: &SkPath, m: &Matrix, op: SkClipOp, do_aa: bool) {
        if !path.is_inverse_fill_type() {
            let mut r = Rect::make_empty();
            if path.is_rect(Some(&mut r), None, None) {
                self.init_rect(save_count, &r, m, op, do_aa);
                return;
            }
            let mut oval_rect = Rect::make_empty();
            if path.is_oval(Some(&mut oval_rect)) {
                let rrect = RRect::make_oval(&oval_rect);
                self.init_rrect(save_count, &rrect, m, op, do_aa);
                return;
            }
        }
        self.init_as_path(save_count, path, m, op, do_aa);
    }

    fn init_as_path(
        &mut self,
        save_count: i32,
        path: &SkPath,
        m: &Matrix,
        op: SkClipOp,
        do_aa: bool,
    ) {
        let mut builder = SkPathBuilder::from_path(path);
        builder.transform(m);
        builder.set_is_volatile(true);
        self.device_space_path = Some(builder.detach(None));

        self.device_space_type = DeviceSpaceType::Path;
        self.init_common(save_count, op, do_aa);
    }

    fn init_shader(&mut self, save_count: i32, shader: ClipShader) {
        self.device_space_type = DeviceSpaceType::Shader;
        self.shader = Some(shader);
        self.init_common(save_count, SkClipOp::Intersect, false);
    }

    fn init_replace_rect(&mut self, save_count: i32, rect: &Rect, do_aa: bool) {
        self.device_space_rrect.set_rect(rect);
        self.device_space_type = DeviceSpaceType::Rect;
        self.init_common(save_count, SkClipOp::Intersect, do_aa);
        self.is_replace = true;
    }

    fn set_empty(&mut self) {
        self.device_space_type = DeviceSpaceType::Empty;
        self.finite_bound.set_empty();
        self.finite_bound_type = BoundsType::Normal;
        self.is_intersection_of_rects = false;
        self.device_space_rrect.set_empty();
        self.device_space_path = None;
        self.shader = None;
        self.gen_id = K_EMPTY_GEN_ID;
        self.check_empty();
    }

    fn check_empty(&self) {
        debug_assert!(self.finite_bound.is_empty());
        debug_assert!(BoundsType::Normal == self.finite_bound_type);
        debug_assert!(!self.is_intersection_of_rects);
        debug_assert_eq!(K_EMPTY_GEN_ID, self.gen_id);
        debug_assert!(self.device_space_rrect.get_type() == RRectType::Empty);
        debug_assert!(self.device_space_path.is_none());
        debug_assert!(self.shader.is_none());
    }

    fn can_be_intersected_in_place(&self, save_count: i32, op: SkClipOp) -> bool {
        if DeviceSpaceType::Empty == self.device_space_type
            && (SkClipOp::Difference == op || SkClipOp::Intersect == op)
        {
            return true;
        }
        // Only clips within the same save/restore frame (as captured by
        // the save count) can be merged
        self.save_count == save_count
            && SkClipOp::Intersect == op
            && (SkClipOp::Intersect == self.op || self.is_replace_op())
    }

    fn rect_rect_intersect_allowed(&self, new_r: &Rect, new_aa: bool) -> bool {
        debug_assert_eq!(DeviceSpaceType::Rect, self.device_space_type);

        if self.do_aa == new_aa {
            // if the AA setting is the same there is no issue
            return true;
        }

        if !Rect::intersects_rects(self.get_device_space_rect(), new_r) {
            // The calling code will correctly set the result to the empty clip
            return true;
        }

        if self.get_device_space_rect().contains_rect(new_r) {
            // if the new rect carves out a portion of the old one there is no issue
            return true;
        }

        // So either the two overlap in some complex manner or newR contains oldR.
        // In the first, case the edges will require different AA. In the second,
        // the AA setting that would be carried forward is incorrect (e.g., oldR
        // is AA while newR is BW but since newR contains oldR, oldR will be
        // drawn BW) since the new AA setting will predominate.
        false
    }

    // a mirror of combineBoundsRevDiff
    fn combine_bounds_diff(&mut self, combination: FillCombo, prev_finite: &Rect) {
        match combination {
            FillCombo::InvPrevInvCur => {
                // In this case the only pixels that can remain set
                // are inside the current clip rect since the extensions
                // to infinity of both clips cancel out and whatever
                // is outside of the current clip is removed
                self.finite_bound_type = BoundsType::Normal;
            }
            FillCombo::InvPrevCur => {
                // In this case the current op is finite so the only pixels
                // that aren't set are whatever isn't set in the previous
                // clip and whatever this clip carves out
                self.finite_bound.join(prev_finite);
                self.finite_bound_type = BoundsType::InsideOut;
            }
            FillCombo::PrevInvCur => {
                // In this case everything outside of this clip's bound
                // is erased, so the only pixels that can remain set
                // occur w/in the intersection of the two finite bounds
                if !self.finite_bound.intersect(prev_finite) {
                    self.finite_bound.set_empty();
                    self.gen_id = K_EMPTY_GEN_ID;
                }
                self.finite_bound_type = BoundsType::Normal;
            }
            FillCombo::PrevCur => {
                // The most conservative result bound is that of the
                // prior clip. This could be wildly incorrect if the
                // second clip either exactly matches the first clip
                // (which should yield the empty set) or reduces the
                // size of the prior bound (e.g., if the second clip
                // exactly matched the bottom half of the prior clip).
                // We ignore these two possibilities.
                self.finite_bound = *prev_finite;
            }
        }
    }

    // a mirror of combineBoundsUnion
    fn combine_bounds_intersection(&mut self, combination: FillCombo, prev_finite: &Rect) {
        match combination {
            FillCombo::InvPrevInvCur => {
                // The only pixels that aren't writable in this case
                // occur in the union of the two finite bounds
                self.finite_bound.join(prev_finite);
                self.finite_bound_type = BoundsType::InsideOut;
            }
            FillCombo::InvPrevCur => {
                // In this case the only pixels that will remain writeable
                // are within the current clip
            }
            FillCombo::PrevInvCur => {
                // In this case the only pixels that will remain writeable
                // are with the previous clip
                self.finite_bound = *prev_finite;
                self.finite_bound_type = BoundsType::Normal;
            }
            FillCombo::PrevCur => {
                if !self.finite_bound.intersect(prev_finite) {
                    self.set_empty();
                }
            }
        }
    }

    fn update_bound_and_gen_id(&mut self, prior: Option<&Element>) {
        // We set this first here but we may overwrite it later if we determine that the clip is
        // either wide-open or empty.
        self.gen_id = get_next_gen_id();

        // First, optimistically update the current Element's bound information
        // with the current clip's bound
        self.is_intersection_of_rects = false;
        match self.device_space_type {
            DeviceSpaceType::Rect => {
                self.finite_bound = *self.get_device_space_rect();
                self.finite_bound_type = BoundsType::Normal;

                if self.is_replace_op()
                    || (SkClipOp::Intersect == self.op && prior.is_none())
                    || (SkClipOp::Intersect == self.op
                        && prior.unwrap().is_intersection_of_rects
                        && prior
                            .unwrap()
                            .rect_rect_intersect_allowed(self.get_device_space_rect(), self.do_aa))
                {
                    self.is_intersection_of_rects = true;
                }
            }
            DeviceSpaceType::RRect => {
                self.finite_bound = *self.device_space_rrect.get_bounds();
                self.finite_bound_type = BoundsType::Normal;
            }
            DeviceSpaceType::Path => {
                self.finite_bound = *self.device_space_path.as_ref().unwrap().get_bounds();

                if self
                    .device_space_path
                    .as_ref()
                    .unwrap()
                    .is_inverse_fill_type()
                {
                    self.finite_bound_type = BoundsType::InsideOut;
                } else {
                    self.finite_bound_type = BoundsType::Normal;
                }
            }
            DeviceSpaceType::Shader => {
                // A shader is infinite. We don't act as wide-open here (which is an empty bounds
                // with the inside out type). This is because when the bounds is empty and
                // inside-out, we know there's full coverage everywhere. With a shader, there's
                // *unknown* coverage everywhere.
                self.finite_bound = RectPriv::make_large_s32();
                self.finite_bound_type = BoundsType::Normal;
            }
            DeviceSpaceType::Empty => {
                debug_assert!(false, "We shouldn't get here with an empty element.");
            }
        }

        // Now determine the previous Element's bound information taking into
        // account that there may be no previous clip
        let (prev_finite, prev_type) = match prior {
            None => {
                // no prior clip means the entire plane is writable
                (Rect::make_empty(), BoundsType::InsideOut)
            }
            Some(prior) => (prior.finite_bound, prior.finite_bound_type),
        };

        let mut bits = 0u8;
        if BoundsType::InsideOut == self.finite_bound_type {
            bits |= 0x01;
        }
        if BoundsType::InsideOut == prev_type {
            bits |= 0x02;
        }
        let combination = FillCombo::from_bits(bits);

        // Now integrate with clip with the prior clips
        if !self.is_replace_op() {
            match self.op {
                SkClipOp::Difference => self.combine_bounds_diff(combination, &prev_finite),
                SkClipOp::Intersect => self.combine_bounds_intersection(combination, &prev_finite),
            }
        } // else Replace just ignores everything prior and should already have filled in bounds.
    }

    fn same(&self, other: &Element) -> bool {
        if self.op != other.op
            || self.device_space_type != other.device_space_type
            || self.do_aa != other.do_aa
            || self.is_replace != other.is_replace
            || self.save_count != other.save_count
        {
            return false;
        }
        match self.device_space_type {
            DeviceSpaceType::Shader => match (&self.shader, &other.shader) {
                (Some(a), Some(b)) => a.same(b),
                (None, None) => true,
                _ => false,
            },
            DeviceSpaceType::Path => {
                self.device_space_path.as_ref() == other.device_space_path.as_ref()
            }
            DeviceSpaceType::RRect => self.device_space_rrect == other.device_space_rrect,
            DeviceSpaceType::Rect => self.get_device_space_rect() == other.get_device_space_rect(),
            DeviceSpaceType::Empty => true,
        }
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self.same(other)
    }
}

impl core::fmt::Debug for Element {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Element")
            .field("device_space_type", &self.device_space_type)
            .field("save_count", &self.save_count)
            .field("op", &self.op)
            .field("do_aa", &self.do_aa)
            .field("is_replace", &self.is_replace)
            .field("finite_bound_type", &self.finite_bound_type)
            .field("finite_bound", &self.finite_bound)
            .field("is_intersection_of_rects", &self.is_intersection_of_rects)
            .field("gen_id", &self.gen_id)
            .finish()
    }
}

impl Eq for Element {}

/// `SkClipStack::kInvalidGenID`.
pub const K_INVALID_GEN_ID: u32 = 0;
/// `SkClipStack::kEmptyGenID`.
pub const K_EMPTY_GEN_ID: u32 = 1;
/// `SkClipStack::kWideOpenGenID`.
pub const K_WIDE_OPEN_GEN_ID: u32 = 2;

const K_FIRST_UNRESERVED_GEN_ID: u32 = 3;

static NEXT_GEN_ID: AtomicU32 = AtomicU32::new(K_FIRST_UNRESERVED_GEN_ID);

/// `SkClipStack::GetNextGenID`.
pub fn get_next_gen_id() -> u32 {
    let mut id;
    loop {
        id = NEXT_GEN_ID.fetch_add(1, Ordering::Relaxed);
        if id >= K_FIRST_UNRESERVED_GEN_ID {
            break;
        }
    }
    id
}

/// `SkClipStack::Iter::IterStart`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterStart {
    Bottom,
    Top,
}

/// Port of `class SkClipStack`.
#[derive(Clone, Debug, Default)]
pub struct SkClipStack {
    /// Elements bottom-to-top, matching `SkDeque` front-to-back.
    f_deque: Vec<Element>,
    f_save_count: i32,
}

impl SkClipStack {
    /// `SkClipStack()`. The reference's `(void* storage, size_t size)` constructor only changes
    /// where the deque allocates; the port's `Vec` makes that parameter unobservable.
    pub fn new() -> SkClipStack {
        SkClipStack {
            f_deque: Vec::new(),
            f_save_count: 0,
        }
    }

    /// `SkClipStack::reset`.
    pub fn reset(&mut self) {
        self.f_deque.clear();
        self.f_save_count = 0;
    }

    /// `SkClipStack::getSaveCount`.
    pub fn get_save_count(&self) -> i32 {
        self.f_save_count
    }

    /// `SkClipStack::save`.
    pub fn save(&mut self) {
        self.f_save_count += 1;
    }

    /// `SkClipStack::restore`.
    pub fn restore(&mut self) {
        self.f_save_count -= 1;
        self.restore_to(self.f_save_count);
    }

    fn restore_to(&mut self, save_count: i32) {
        while let Some(back) = self.f_deque.last() {
            if back.save_count <= save_count {
                break;
            }
            self.f_deque.pop();
        }
    }

    /// `SkClipStack::bounds`.
    pub fn bounds(&self, device_bounds: &IRect) -> Rect {
        // TODO: optimize this.
        let mut r = Rect::make_empty();
        let bounds = self.get_bounds_only(&mut r);
        if bounds == BoundsType::InsideOut {
            return Rect::from(*device_bounds);
        }
        if r.intersect(&Rect::from(*device_bounds)) {
            r
        } else {
            Rect::make_empty()
        }
    }

    /// `SkClipStack::isEmpty`.
    pub fn is_empty(&self, r: &IRect) -> bool {
        self.bounds(r).is_empty()
    }

    fn get_bounds_only(&self, canv_finite_bound: &mut Rect) -> BoundsType {
        match self.f_deque.last() {
            None => {
                // the clip is wide open - the infinite plane w/ no pixels un-writeable
                canv_finite_bound.set_empty();
                BoundsType::InsideOut
            }
            Some(element) => {
                *canv_finite_bound = element.finite_bound;
                element.finite_bound_type
            }
        }
    }

    /// `SkClipStack::getBounds`.
    pub fn get_bounds(&self, canv_finite_bound: &mut Rect, bound_type: &mut BoundsType) {
        *bound_type = self.get_bounds_only(canv_finite_bound);
    }

    /// `SkClipStack::getBounds` with the optional `isIntersectionOfRects` out-parameter.
    pub fn get_bounds_with_intersection(
        &self,
        canv_finite_bound: &mut Rect,
        bound_type: &mut BoundsType,
        is_intersection_of_rects: &mut bool,
    ) {
        match self.f_deque.last() {
            None => {
                canv_finite_bound.set_empty();
                *bound_type = BoundsType::InsideOut;
                *is_intersection_of_rects = false;
            }
            Some(element) => {
                *canv_finite_bound = element.finite_bound;
                *bound_type = element.finite_bound_type;
                *is_intersection_of_rects = element.is_intersection_of_rects;
            }
        }
    }

    fn internal_quick_contains_rect(&self, rect: &Rect) -> bool {
        let mut iter = Iter::new(self, IterStart::Top);
        while let Some(element) = iter.prev() {
            // TODO: Once expanding ops are removed, this condition is equiv. to op == kDifference.
            if SkClipOp::Intersect != element.get_op() && !element.is_replace_op() {
                return false;
            }
            if element.is_inverse_filled() {
                // Part of 'rect' could be trimmed off by the inverse-filled clip element
                if Rect::intersects_rects(&element.get_bounds(), rect) {
                    return false;
                }
            } else if !element.contains_rect(rect) {
                return false;
            }
            if element.is_replace_op() {
                break;
            }
        }
        true
    }

    fn internal_quick_contains_rrect(&self, rrect: &RRect) -> bool {
        let mut iter = Iter::new(self, IterStart::Top);
        while let Some(element) = iter.prev() {
            // TODO: Once expanding ops are removed, this condition is equiv. to op == kDifference.
            if SkClipOp::Intersect != element.get_op() && !element.is_replace_op() {
                return false;
            }
            if element.is_inverse_filled() {
                // Part of 'rrect' could be trimmed off by the inverse-filled clip element
                if Rect::intersects_rects(&element.get_bounds(), rrect.get_bounds()) {
                    return false;
                }
            } else if !element.contains_rrect(rrect) {
                return false;
            }
            if element.is_replace_op() {
                break;
            }
        }
        true
    }

    /// `SkClipStack::quickContains(const SkRect&)`.
    pub fn quick_contains(&self, dev_rect: &Rect) -> bool {
        self.is_wide_open() || self.internal_quick_contains_rect(dev_rect)
    }

    /// `SkClipStack::quickContains(const SkRRect&)`.
    pub fn quick_contains_rrect(&self, dev_rrect: &RRect) -> bool {
        self.is_wide_open() || self.internal_quick_contains_rrect(dev_rrect)
    }

    /// `SkClipStack::clipDevRect`.
    pub fn clip_dev_rect(&mut self, ir: &IRect, op: SkClipOp) {
        self.clip_rect(&Rect::from(*ir), &Matrix::default(), op, false);
    }

    /// `SkClipStack::clipRect`.
    pub fn clip_rect(&mut self, rect: &Rect, matrix: &Matrix, op: SkClipOp, do_aa: bool) {
        let mut element = Element::new_raw(self.f_save_count);
        element.init_rect(self.f_save_count, rect, matrix, op, do_aa);
        self.push_element(element);
    }

    /// `SkClipStack::clipRRect`.
    pub fn clip_rrect(&mut self, rrect: &RRect, matrix: &Matrix, op: SkClipOp, do_aa: bool) {
        let mut element = Element::new_raw(self.f_save_count);
        element.init_rrect(self.f_save_count, rrect, matrix, op, do_aa);
        self.push_element(element);
    }

    /// `SkClipStack::clipPath`.
    pub fn clip_path(&mut self, path: &SkPath, matrix: &Matrix, op: SkClipOp, do_aa: bool) {
        let mut element = Element::new_raw(self.f_save_count);
        element.init_path(self.f_save_count, path, matrix, op, do_aa);
        self.push_element(element);
    }

    /// `SkClipStack::clipShader`.
    pub fn clip_shader(&mut self, shader: SkShaderHandle) {
        let mut element = Element::new_raw(self.f_save_count);
        element.init_shader(self.f_save_count, ClipShader::Shader(shader));
        self.push_element(element);
    }

    /// `SkClipStack::replaceClip`.
    pub fn replace_clip(&mut self, rect: &Rect, do_aa: bool) {
        let mut element = Element::new_raw(self.f_save_count);
        element.init_replace_rect(self.f_save_count, rect, do_aa);
        self.push_element(element);
    }

    /// `SkClipStack::clipEmpty`.
    pub fn clip_empty(&mut self) {
        if let Some(back) = self.f_deque.last_mut() {
            if back.can_be_intersected_in_place(self.f_save_count, SkClipOp::Intersect) {
                back.set_empty();
            }
        }
        let mut element = Element::new_empty(self.f_save_count);
        element.gen_id = K_EMPTY_GEN_ID;
        self.f_deque.push(element);
    }

    fn push_element(&mut self, element: Element) {
        let mut prior_index = self.f_deque.len().checked_sub(1);

        if let Some(prior_idx) = prior_index {
            if element.is_replace_op() {
                self.restore_to(self.f_save_count - 1);
                prior_index = self.f_deque.len().checked_sub(1);
            } else if self.f_deque[prior_idx]
                .can_be_intersected_in_place(self.f_save_count, element.get_op())
            {
                let prior_prior = prior_idx.checked_sub(1);
                match self.f_deque[prior_idx].device_space_type {
                    DeviceSpaceType::Empty => {
                        self.f_deque[prior_idx].check_empty();
                        return;
                    }
                    DeviceSpaceType::Shader => {
                        if DeviceSpaceType::Shader == element.get_device_space_type() {
                            // prior->fShader = SkShaders::Blend(kSrcIn, element.fShader,
                            //                                   prior->fShader)
                            let prev = self.f_deque[prior_idx].shader.take().unwrap();
                            let new = element.shader.unwrap();
                            self.f_deque[prior_idx].shader = Some(ClipShader::SrcIn {
                                src: Box::new(new),
                                dst: Box::new(prev),
                            });
                            let prior_prior_ref = prior_prior.map(|i| self.f_deque[i].clone());
                            self.f_deque[prior_idx]
                                .update_bound_and_gen_id(prior_prior_ref.as_ref());
                            return;
                        }
                    }
                    DeviceSpaceType::Rect => {
                        if DeviceSpaceType::Rect == element.get_device_space_type() {
                            let allowed = self.f_deque[prior_idx].rect_rect_intersect_allowed(
                                element.get_device_space_rect(),
                                element.is_aa(),
                            );
                            if allowed {
                                let mut isect_rect = Rect::make_empty();
                                if !isect_rect.intersect_rects(
                                    *self.f_deque[prior_idx].get_device_space_rect(),
                                    *element.get_device_space_rect(),
                                ) {
                                    self.f_deque[prior_idx].set_empty();
                                    return;
                                }

                                self.f_deque[prior_idx]
                                    .device_space_rrect
                                    .set_rect(&isect_rect);
                                self.f_deque[prior_idx].do_aa = element.is_aa();
                                let prior_prior_ref = prior_prior.map(|i| self.f_deque[i].clone());
                                self.f_deque[prior_idx]
                                    .update_bound_and_gen_id(prior_prior_ref.as_ref());
                                return;
                            }
                        }
                        if !Rect::intersects_rects(
                            &self.f_deque[prior_idx].get_bounds(),
                            &element.get_bounds(),
                        ) {
                            self.f_deque[prior_idx].set_empty();
                            return;
                        }
                    }
                    _ => {
                        if !Rect::intersects_rects(
                            &self.f_deque[prior_idx].get_bounds(),
                            &element.get_bounds(),
                        ) {
                            self.f_deque[prior_idx].set_empty();
                            return;
                        }
                    }
                }
            }
        }
        let prior_ref = prior_index.map(|i| &self.f_deque[i]);
        let mut new_element = element;
        new_element.update_bound_and_gen_id(prior_ref);
        self.f_deque.push(new_element);
    }

    /// `SkClipStack::isWideOpen`.
    pub fn is_wide_open(&self) -> bool {
        self.get_topmost_gen_id() == K_WIDE_OPEN_GEN_ID
    }

    /// `SkClipStack::getTopmostGenID`.
    pub fn get_topmost_gen_id(&self) -> u32 {
        let Some(back) = self.f_deque.last() else {
            return K_WIDE_OPEN_GEN_ID;
        };
        if BoundsType::InsideOut == back.finite_bound_type
            && back.finite_bound.is_empty()
            && DeviceSpaceType::Shader != back.device_space_type
        {
            return K_WIDE_OPEN_GEN_ID;
        }
        back.get_gen_id()
    }

    /// `SkClipStack::isRRect`.
    pub fn is_rrect(&self, bounds: &Rect, rrect: &mut RRect, aa: &mut bool) -> bool {
        let Some(back) = self.f_deque.last() else {
            // TODO: return bounds?
            return false;
        };
        // First check if the entire stack is known to be a rect by the top element.
        if back.is_intersection_of_rects && back.finite_bound_type == BoundsType::Normal {
            rrect.set_rect(&back.finite_bound);
            *aa = back.is_aa();
            return true;
        }

        if back.get_device_space_type() != DeviceSpaceType::Rect
            && back.get_device_space_type() != DeviceSpaceType::RRect
        {
            return false;
        }
        if back.is_replace_op() {
            *rrect = *back.as_device_space_rrect();
            *aa = back.is_aa();
            return true;
        }

        if back.get_op() == SkClipOp::Intersect {
            let mut back_bounds = Rect::make_empty();
            if !back_bounds.intersect_rects(*bounds, *back.as_device_space_rrect().rect()) {
                return false;
            }
            // We limit to 17 elements. This means the back element will be bounds checked at most
            // 16 times if it is an rrect.
            let cnt = self.f_deque.len();
            if cnt > 17 {
                return false;
            }
            if cnt > 1 {
                let mut idx = cnt - 1;
                while let Some(prior) = idx.checked_sub(1).map(|i| &self.f_deque[i]) {
                    // TODO: Once expanding clip ops are removed, this is equiv. to op ==
                    // kDifference
                    if (prior.get_op() != SkClipOp::Intersect && !prior.is_replace_op())
                        || !prior.contains_rect(&back_bounds)
                    {
                        return false;
                    }
                    if prior.is_replace_op() {
                        break;
                    }
                    idx -= 1;
                }
            }
            *rrect = *back.as_device_space_rrect();
            *aa = back.is_aa();
            return true;
        }
        false
    }

    /// `SkClipStack::getConservativeBounds`.
    pub fn get_conservative_bounds(
        &self,
        offset_x: i32,
        offset_y: i32,
        max_width: i32,
        max_height: i32,
        dev_bounds: &mut Rect,
        is_intersection_of_rects: Option<&mut bool>,
    ) {
        dev_bounds.set_ltrb(
            0.0,
            0.0,
            max_width as crate::scalar::Scalar,
            max_height as crate::scalar::Scalar,
        );

        let mut temp = Rect::make_empty();
        let mut bound_type = BoundsType::Normal;
        match is_intersection_of_rects {
            Some(i) => self.get_bounds_with_intersection(&mut temp, &mut bound_type, i),
            None => {
                let mut unused = false;
                self.get_bounds_with_intersection(&mut temp, &mut bound_type, &mut unused);
            }
        }
        if BoundsType::InsideOut == bound_type {
            return;
        }

        // but is converted to device space here
        temp.offset(
            offset_x as crate::scalar::Scalar,
            offset_y as crate::scalar::Scalar,
        );

        if !dev_bounds.intersect(&temp) {
            dev_bounds.set_empty();
        }
    }

    /// `SkClipStack::Element` access by index, used by the iterators.
    pub fn element_at(&self, index: usize) -> Option<&Element> {
        self.f_deque.get(index)
    }

    /// Number of elements currently stored (the reference's `fDeque.count()`).
    pub fn count(&self) -> usize {
        self.f_deque.len()
    }

    /// `SkClipStack::operator==`.
    pub fn equals(&self, b: &SkClipStack) -> bool {
        if self.get_topmost_gen_id() == b.get_topmost_gen_id() {
            return true;
        }
        if self.f_save_count != b.f_save_count || self.f_deque.len() != b.f_deque.len() {
            return false;
        }
        self.f_deque == b.f_deque
    }
}

impl PartialEq for SkClipStack {
    fn eq(&self, other: &SkClipStack) -> bool {
        self.equals(other)
    }
}

impl Eq for SkClipStack {}

/// `SkClipStack::Iter`.
pub struct Iter<'a> {
    stack: Option<&'a SkClipStack>,
    idx: usize,
    forward: bool,
    done: bool,
}

impl<'a> Iter<'a> {
    /// `Iter()` — an uninitialized iterator; call `reset` before use.
    pub fn new_uninitialized() -> Iter<'a> {
        Iter {
            stack: None,
            idx: 0,
            forward: true,
            done: true,
        }
    }

    /// `Iter(const SkClipStack&, IterStart)`.
    pub fn new(stack: &'a SkClipStack, start_loc: IterStart) -> Iter<'a> {
        let mut iter = Iter {
            stack: None,
            idx: 0,
            forward: true,
            done: true,
        };
        iter.reset(stack, start_loc);
        iter
    }

    /// `Iter::reset`.
    pub fn reset(&mut self, stack: &'a SkClipStack, start_loc: IterStart) {
        self.stack = Some(stack);
        self.done = stack.f_deque.is_empty();
        match start_loc {
            IterStart::Bottom => {
                self.forward = true;
                self.idx = 0;
            }
            IterStart::Top => {
                self.forward = false;
                self.idx = stack.f_deque.len().saturating_sub(1);
            }
        }
    }

    /// `Iter::next`.
    pub fn next(&mut self) -> Option<&'a Element> {
        if self.done {
            return None;
        }
        let stack = self.stack?;
        let element = stack.f_deque.get(self.idx)?;
        debug_assert!(self.forward);
        if self.idx + 1 < stack.f_deque.len() {
            self.idx += 1;
        } else {
            self.done = true;
        }
        Some(element)
    }

    /// `Iter::prev`.
    pub fn prev(&mut self) -> Option<&'a Element> {
        if self.done {
            return None;
        }
        let stack = self.stack?;
        let element = stack.f_deque.get(self.idx)?;
        if self.idx > 0 {
            self.idx -= 1;
        } else {
            self.done = true;
        }
        Some(element)
    }

    /// `Iter::skipToTopmost`. Moves the iterator to the topmost element with `op` and returns it;
    /// if no such element exists, the first element is returned.
    pub fn skip_to_topmost(&mut self, op: SkClipOp) -> Option<&'a Element> {
        let stack = self.stack?;

        let mut found = None;
        for i in (0..stack.f_deque.len()).rev() {
            if stack.f_deque[i].op == op {
                found = Some(i);
                break;
            }
        }

        match found {
            Some(i) => {
                // The reference's reverse iterator ends one pace before the returned element; it
                // then steps forward so that this iterator's next() yields that element. When the
                // match is the bottom element the reverse iterator has run off the front and is
                // reset to the front, which is the same position.
                self.done = false;
                self.forward = true;
                self.idx = i;
            }
            None => {
                self.done = stack.f_deque.is_empty();
                self.forward = true;
                self.idx = 0;
            }
        }
        self.next()
    }
}

/// `SkClipStack::B2TIter`.
pub struct B2TIter<'a> {
    iter: Iter<'a>,
}

impl<'a> B2TIter<'a> {
    /// `B2TIter()`.
    pub fn new_uninitialized() -> B2TIter<'a> {
        B2TIter {
            iter: Iter::new_uninitialized(),
        }
    }

    /// `B2TIter(const SkClipStack&)`.
    pub fn new(stack: &'a SkClipStack) -> B2TIter<'a> {
        B2TIter {
            iter: Iter::new(stack, IterStart::Bottom),
        }
    }

    /// `B2TIter::next`.
    pub fn next(&mut self) -> Option<&'a Element> {
        self.iter.next()
    }

    /// `B2TIter::reset`.
    pub fn reset(&mut self, stack: &'a SkClipStack) {
        self.iter.reset(stack, IterStart::Bottom);
    }
}
