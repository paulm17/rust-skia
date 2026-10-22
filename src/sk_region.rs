//! Port of `include/core/SkRegion.h` and `src/core/SkRegion.cpp`.
//!
//! The reference stores the run-length encoded region in a ref-counted `RunHead` with the runs
//! allocated inline. The port keeps the identical run encoding, copy-on-write sharing and
//! three-state model (empty sentinel head, rect head and complex head), but the runs live in a
//! `Vec<i32>` behind an `Arc` so that they can be addressed without pointer arithmetic.
//!
//! See `plan/handoffs/M09.md` for the source-to-Rust mapping and the adaptation notes.

use std::borrow::Cow;
use std::sync::Arc;

use crate::rect::IRect;
use crate::safe_math::SkSafeMath;
use crate::sk_clip_op::SkClipOp;

/// `SkRegion_kRunTypeSentinel`.
pub const SK_REGION_K_RUN_TYPE_SENTINEL: i32 = 0x7FFF_FFFF;

/// `SkRegion::kRectRegionRuns`.
pub const K_RECT_REGION_RUNS: usize = 7;

#[inline]
pub fn sk_region_value_is_sentinel(value: i32) -> bool {
    value == SK_REGION_K_RUN_TYPE_SENTINEL
}

/// `SkRegion::RunType`.
pub type RunType = i32;

/// The inline run storage of `SkRegion::RunHead` (`fRunCount`, `fYSpanCount`, `fIntervalCount` and
/// the `fRunCount` run values).
#[derive(Clone, Debug)]
pub(crate) struct RunData {
    pub(crate) runs: Vec<RunType>,
    pub(crate) y_span_count: i32,
    pub(crate) interval_count: i32,
}

/// `SkRegion::RunHead*`. The reference uses two distinguished pointer values for the empty and
/// rect cases and a ref-counted allocation for the complex case.
#[derive(Clone, Debug, Default)]
pub(crate) enum RunHead {
    #[default]
    Empty,
    Rect,
    Complex(Arc<RunData>),
}

impl RunHead {
    fn data(&self) -> Option<&RunData> {
        match self {
            RunHead::Complex(d) => Some(d),
            _ => None,
        }
    }
}

/// `SkRegion::Op`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkRegionOp {
    Difference = 0,
    Intersect = 1,
    Union = 2,
    Xor = 3,
    ReverseDifference = 4,
    Replace = 5,
}

impl SkRegionOp {
    pub const K_LAST_OP: SkRegionOp = SkRegionOp::Replace;
    pub const K_OP_CNT: usize = 6;

    #[inline]
    pub const fn from_i32(value: i32) -> SkRegionOp {
        match value {
            0 => SkRegionOp::Difference,
            1 => SkRegionOp::Intersect,
            2 => SkRegionOp::Union,
            3 => SkRegionOp::Xor,
            4 => SkRegionOp::ReverseDifference,
            _ => SkRegionOp::Replace,
        }
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        self as i32
    }
}

impl From<SkClipOp> for SkRegionOp {
    fn from(op: SkClipOp) -> Self {
        match op {
            SkClipOp::Difference => SkRegionOp::Difference,
            SkClipOp::Intersect => SkRegionOp::Intersect,
        }
    }
}

/// `SkRegion::BoundsType` (shared with `SkClipStack`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundsType {
    Normal,
    InsideOut,
}

/// Port of `class SkRegion`.
#[derive(Clone, Debug)]
pub struct SkRegion {
    pub(crate) bounds: IRect,
    pub(crate) run_head: RunHead,
}

impl Default for SkRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for SkRegion {
    fn eq(&self, b: &SkRegion) -> bool {
        if self.bounds != b.bounds {
            return false;
        }
        match (&self.run_head, &b.run_head) {
            (RunHead::Empty, RunHead::Empty) | (RunHead::Rect, RunHead::Rect) => true,
            (RunHead::Complex(ah), RunHead::Complex(bh)) => {
                Arc::ptr_eq(ah, bh) || ah.runs == bh.runs
            }
            _ => false,
        }
    }
}

impl Eq for SkRegion {}

impl SkRegion {
    /// `SkRegion()`.
    pub fn new() -> SkRegion {
        SkRegion {
            bounds: IRect::make_empty(),
            run_head: RunHead::Empty,
        }
    }

    /// `explicit SkRegion(const SkIRect&)`.
    pub fn from_irect(rect: IRect) -> SkRegion {
        let mut rgn = SkRegion::new();
        rgn.set_rect(&rect);
        rgn
    }

    // ---------------------------------------------------------------- state queries

    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self.run_head, RunHead::Empty)
    }

    #[inline]
    pub fn is_rect(&self) -> bool {
        matches!(self.run_head, RunHead::Rect)
    }

    #[inline]
    pub fn is_complex(&self) -> bool {
        !self.is_empty() && !self.is_rect()
    }

    #[inline]
    pub fn get_bounds(&self) -> &IRect {
        &self.bounds
    }

    /// `computeRegionComplexity`.
    pub fn compute_region_complexity(&self) -> i32 {
        match &self.run_head {
            RunHead::Empty => 0,
            RunHead::Rect => 1,
            RunHead::Complex(d) => d.interval_count,
        }
    }

    fn data(&self) -> Option<&RunData> {
        self.run_head.data()
    }

    /// The runs of a complex region (`RunHead::readonly_runs`).
    #[inline]
    pub(crate) fn complex_runs(&self) -> Option<&[RunType]> {
        self.data().map(|data| data.runs.as_slice())
    }

    /// `set`.
    pub fn set(&mut self, src: &SkRegion) -> bool {
        self.set_region(src);
        !self.is_empty()
    }

    /// `swap`.
    pub fn swap(&mut self, other: &mut SkRegion) {
        std::mem::swap(&mut self.bounds, &mut other.bounds);
        std::mem::swap(&mut self.run_head, &mut other.run_head);
    }

    /// `setEmpty`.
    pub fn set_empty(&mut self) -> bool {
        self.bounds = IRect::make_empty();
        self.run_head = RunHead::Empty;
        false
    }

    /// `setRect`.
    pub fn set_rect(&mut self, r: &IRect) -> bool {
        if r.is_empty()
            || r.right == SK_REGION_K_RUN_TYPE_SENTINEL
            || r.bottom == SK_REGION_K_RUN_TYPE_SENTINEL
        {
            return self.set_empty();
        }
        self.bounds = *r;
        self.run_head = RunHead::Rect;
        true
    }

    /// `setRegion`.
    pub fn set_region(&mut self, src: &SkRegion) -> bool {
        if !std::ptr::eq(self, src) {
            self.bounds = src.bounds;
            self.run_head = src.run_head.clone();
        }
        !self.is_empty()
    }

    /// `setRects`.
    pub fn set_rects(&mut self, rects: &[IRect]) -> bool {
        if rects.is_empty() {
            return self.set_empty();
        }
        if rects.len() == 1 {
            return self.set_rect(&rects[0]);
        }
        let mid = rects.len() / 2;
        let mut left = SkRegion::new();
        let mut right = SkRegion::new();
        left.set_rects(&rects[..mid]);
        right.set_rects(&rects[mid..]);
        self.op(&left, &right, SkRegionOp::Union)
    }

    // ---------------------------------------------------------------- run helpers

    fn build_rect_runs(bounds: &IRect, runs: &mut [RunType; K_RECT_REGION_RUNS]) {
        runs[0] = bounds.top;
        runs[1] = bounds.bottom;
        runs[2] = 1;
        runs[3] = bounds.left;
        runs[4] = bounds.right;
        runs[5] = SK_REGION_K_RUN_TYPE_SENTINEL;
        runs[6] = SK_REGION_K_RUN_TYPE_SENTINEL;
    }

    /// `RunsAreARect`. Returns `None` when the runs do not describe exactly one rectangle.
    pub fn runs_are_a_rect(runs: &[RunType], count: usize) -> Option<IRect> {
        debug_assert!(count >= K_RECT_REGION_RUNS);
        if count == K_RECT_REGION_RUNS {
            debug_assert_eq!(runs[2], 1);
            debug_assert!(sk_region_value_is_sentinel(runs[5]));
            debug_assert!(sk_region_value_is_sentinel(runs[6]));
            if runs[0] < runs[1] && runs[3] < runs[4] {
                return Some(IRect::new(runs[3], runs[0], runs[4], runs[1]));
            }
        }
        None
    }

    /// `getRuns`: the runs for this region, using synthesized runs for the empty and rect cases.
    fn get_runs(&self) -> Cow<'_, [RunType]> {
        match &self.run_head {
            RunHead::Empty => Cow::Owned(vec![SK_REGION_K_RUN_TYPE_SENTINEL]),
            RunHead::Rect => {
                let mut runs = [0; K_RECT_REGION_RUNS];
                Self::build_rect_runs(&self.bounds, &mut runs);
                Cow::Owned(runs.to_vec())
            }
            RunHead::Complex(d) => Cow::Borrowed(&d.runs),
        }
    }

    /// `count_runtype_values`.
    pub(crate) fn count_runtype_values(&self, itop: &mut i32, ibot: &mut i32) -> i32 {
        let max_t = if self.is_rect() {
            2
        } else {
            debug_assert!(self.is_complex());
            self.data().unwrap().interval_count * 2
        };
        *itop = self.bounds.top;
        *ibot = self.bounds.bottom;
        max_t
    }

    fn skip_entire_scanline(runs: &[RunType], i: usize) -> usize {
        debug_assert!(runs[i] < SK_REGION_K_RUN_TYPE_SENTINEL);
        let intervals = runs[i + 1] as usize;
        debug_assert_eq!(runs[i + 2 + intervals * 2], SK_REGION_K_RUN_TYPE_SENTINEL);
        i + 1 + 1 + intervals * 2 + 1
    }

    /// `RunHead::findScanline`, returning the index of the scanline's Bottom value.
    fn find_scanline(&self, y: i32) -> usize {
        let data = self.data().unwrap();
        let runs = &data.runs;
        debug_assert!(y >= runs[0]);
        let mut i = 1;
        loop {
            let bottom = runs[i];
            debug_assert!(bottom < SK_REGION_K_RUN_TYPE_SENTINEL);
            if y < bottom {
                break;
            }
            i = Self::skip_entire_scanline(runs, i);
        }
        i
    }

    fn scanline_next(runs: &[RunType], i: usize) -> usize {
        i + 2 + runs[i + 1] as usize * 2 + 1
    }

    /// `computeRunBounds`: returns `(ySpanCount, intervalCount, bounds)`.
    fn compute_run_bounds(runs: &[RunType]) -> (i32, i32, IRect) {
        let mut i = 0;
        let top = runs[i];
        i += 1;

        let mut bot;
        let mut y_span_count = 0;
        let mut interval_count = 0;
        let mut left = i32::MAX;
        let mut rite = i32::MIN;

        loop {
            bot = runs[i];
            debug_assert!(bot < SK_REGION_K_RUN_TYPE_SENTINEL);
            y_span_count += 1;
            i += 1;

            let intervals = runs[i];
            debug_assert!(intervals >= 0);
            i += 1;

            if intervals > 0 {
                let l = runs[i];
                debug_assert!(l < SK_REGION_K_RUN_TYPE_SENTINEL);
                if left > l {
                    left = l;
                }
                i += intervals as usize * 2;
                let r = runs[i - 1];
                debug_assert!(r < SK_REGION_K_RUN_TYPE_SENTINEL);
                if rite < r {
                    rite = r;
                }
                interval_count += intervals;
            }
            debug_assert_eq!(runs[i], SK_REGION_K_RUN_TYPE_SENTINEL);
            i += 1;

            if runs[i] == SK_REGION_K_RUN_TYPE_SENTINEL {
                break;
            }
        }

        (
            y_span_count,
            interval_count,
            IRect::new(left, top, rite, bot),
        )
    }

    fn set_runs(&mut self, mut runs: Vec<RunType>) -> bool {
        let mut count = runs.len();
        debug_assert!(count > 0);

        if is_run_count_empty(count) {
            debug_assert!(sk_region_value_is_sentinel(runs[count - 1]));
            return self.set_empty();
        }

        let mut start = 0usize;
        let mut stop = count;
        if count > K_RECT_REGION_RUNS {
            // trim off any empty spans from the top and bottom
            if runs[start + 3] == SK_REGION_K_RUN_TYPE_SENTINEL {
                start += 3;
                runs[start] = runs[start - 2];
            }
            if runs[stop - 5] == SK_REGION_K_RUN_TYPE_SENTINEL {
                runs[stop - 4] = SK_REGION_K_RUN_TYPE_SENTINEL;
                stop -= 3;
            }
            count = stop - start;
        }

        debug_assert!(count >= K_RECT_REGION_RUNS);

        if let Some(bounds) = Self::runs_are_a_rect(&runs[start..stop], count) {
            return self.set_rect(&bounds);
        }

        self.set_runs_from_builder(runs[start..stop].to_vec())
    }

    /// The complex branch of `setRuns` / `SkRegion_path`'s builder handoff: store `runs` and compute
    /// the bounds and counters from them without trimming.
    pub(crate) fn set_runs_from_builder(&mut self, runs: Vec<RunType>) -> bool {
        let (y_span_count, interval_count, bounds) = Self::compute_run_bounds(&runs);
        self.run_head = RunHead::Complex(Arc::new(RunData {
            runs,
            y_span_count,
            interval_count,
        }));
        self.bounds = bounds;
        if self.bounds.is_empty() {
            return self.set_empty();
        }
        true
    }

    // ---------------------------------------------------------------- queries

    /// `contains(int32_t x, int32_t y)`.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        if !self.bounds.contains(x, y) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        let data = self.data().unwrap();
        let runs = &data.runs;
        // Skip the Bottom and IntervalCount.
        let mut i = self.find_scanline(y) + 2;
        loop {
            if x < runs[i] {
                break;
            }
            if x < runs[i + 1] {
                return true;
            }
            i += 2;
        }
        false
    }

    /// `contains(const SkIRect&)`.
    pub fn contains_irect(&self, r: &IRect) -> bool {
        if !self.bounds.contains_irect(r) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        let data = self.data().unwrap();
        let runs = &data.runs;
        let mut i = self.find_scanline(r.top);
        loop {
            if !scanline_contains(runs, i, r.left, r.right) {
                return false;
            }
            if r.bottom <= runs[i] {
                break;
            }
            i = Self::scanline_next(runs, i);
        }
        true
    }

    /// `contains(const SkRegion&)`.
    pub fn contains_region(&self, rgn: &SkRegion) -> bool {
        if self.is_empty() || rgn.is_empty() || !self.bounds.contains_irect(&rgn.bounds) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        if rgn.is_rect() {
            return self.contains_irect(rgn.get_bounds());
        }

        // A contains B is equivalent to B - A == 0.
        !SkRegion::oper(rgn, self, SkRegionOp::Difference, None)
    }

    /// `intersects(const SkIRect&)`.
    pub fn intersects(&self, r: &IRect) -> bool {
        if self.is_empty() || r.is_empty() {
            return false;
        }

        let mut sect = IRect::make_empty();
        if !sect.intersect_irects(self.bounds, *r) {
            return false;
        }
        if self.is_rect() {
            return true;
        }

        let data = self.data().unwrap();
        let runs = &data.runs;
        let mut i = self.find_scanline(sect.top);
        loop {
            if scanline_intersects(runs, i, sect.left, sect.right) {
                return true;
            }
            if sect.bottom <= runs[i] {
                break;
            }
            i = Self::scanline_next(runs, i);
        }
        false
    }

    /// `intersects(const SkRegion&)`.
    pub fn intersects_region(&self, rgn: &SkRegion) -> bool {
        if self.is_empty() || rgn.is_empty() {
            return false;
        }
        if !IRect::intersects_irects(&self.bounds, &rgn.bounds) {
            return false;
        }

        let we_are_rect = self.is_rect();
        let they_are_rect = rgn.is_rect();

        if we_are_rect && they_are_rect {
            return true;
        }
        if we_are_rect {
            return rgn.intersects(self.get_bounds());
        }
        if they_are_rect {
            return self.intersects(rgn.get_bounds());
        }

        SkRegion::oper(self, rgn, SkRegionOp::Intersect, None)
    }

    /// `quickContains`.
    pub fn quick_contains(&self, r: &IRect) -> bool {
        r.left < r.right
            && r.top < r.bottom
            && self.is_rect()
            && self.bounds.left <= r.left
            && self.bounds.top <= r.top
            && self.bounds.right >= r.right
            && self.bounds.bottom >= r.bottom
    }

    /// `quickReject(const SkIRect&)`.
    pub fn quick_reject(&self, rect: &IRect) -> bool {
        self.is_empty() || rect.is_empty() || !IRect::intersects_irects(&self.bounds, rect)
    }

    /// `quickReject(const SkRegion&)`.
    pub fn quick_reject_region(&self, rgn: &SkRegion) -> bool {
        self.is_empty() || rgn.is_empty() || !IRect::intersects_irects(&self.bounds, &rgn.bounds)
    }

    // ---------------------------------------------------------------- translation

    /// `translate(int dx, int dy)`.
    pub fn translate(&mut self, dx: i32, dy: i32) {
        if self.is_empty() {
            self.set_empty();
            return;
        }
        let dx = pin_offset_s32(self.bounds.left, self.bounds.right, dx);
        let dy = pin_offset_s32(self.bounds.top, self.bounds.bottom, dy);

        if self.is_rect() {
            let offset = self.bounds.make_offset(dx, dy);
            self.set_rect(&offset);
            return;
        }

        let (mut runs, y_span_count, interval_count) = {
            let data = self.data().unwrap();
            (data.runs.clone(), data.y_span_count, data.interval_count)
        };

        runs[0] = runs[0].wrapping_add(dy); // top
        let mut i = 1usize;
        loop {
            let bottom = runs[i];
            if bottom == SK_REGION_K_RUN_TYPE_SENTINEL {
                break;
            }
            runs[i] = bottom.wrapping_add(dy);
            i += 1;
            i += 1; // copy intervalCount
            loop {
                let x = runs[i];
                if x == SK_REGION_K_RUN_TYPE_SENTINEL {
                    break;
                }
                runs[i] = x.wrapping_add(dx);
                runs[i + 1] = runs[i + 1].wrapping_add(dx);
                i += 2;
            }
            runs[i] = SK_REGION_K_RUN_TYPE_SENTINEL;
            i += 1;
        }
        runs[i] = SK_REGION_K_RUN_TYPE_SENTINEL;

        self.bounds.offset(dx, dy);
        self.run_head = RunHead::Complex(Arc::new(RunData {
            runs,
            y_span_count,
            interval_count,
        }));
    }

    /// `translate(int dx, int dy, SkRegion* dst)`.
    pub fn translate_to(&self, dx: i32, dy: i32, dst: &mut SkRegion) {
        if self.is_empty() {
            dst.set_empty();
            return;
        }
        if std::ptr::eq(self, &*dst) {
            dst.translate(dx, dy);
            return;
        }
        dst.set_region(self);
        dst.translate(dx, dy);
    }

    // ---------------------------------------------------------------- operations

    /// `op(const SkIRect& rect, Op op)`.
    pub fn op_irect(&mut self, rect: &IRect, op: SkRegionOp) -> bool {
        if self.is_rect() && op == SkRegionOp::Intersect {
            if !self.bounds.intersect(rect) {
                return self.set_empty();
            }
            return true;
        }
        let a = self.clone();
        let b = SkRegion::from_irect(*rect);
        self.op(&a, &b, op)
    }

    /// `op(const SkRegion& rgn, Op op)`.
    pub fn op_region(&mut self, rgn: &SkRegion, op: SkRegionOp) -> bool {
        let a = self.clone();
        self.op(&a, rgn, op)
    }

    /// `op(const SkIRect& rect, const SkRegion& rgn, Op op)`.
    pub fn op_irect_region(&mut self, rect: &IRect, rgn: &SkRegion, op: SkRegionOp) -> bool {
        let a = SkRegion::from_irect(*rect);
        self.op(&a, rgn, op)
    }

    /// `op(const SkRegion& rgn, const SkIRect& rect, Op op)`.
    pub fn op_region_irect(&mut self, rgn: &SkRegion, rect: &IRect, op: SkRegionOp) -> bool {
        let b = SkRegion::from_irect(*rect);
        self.op(rgn, &b, op)
    }

    /// `op(const SkRegion& rgna, const SkRegion& rgnb, Op op)`.
    pub fn op(&mut self, rgna: &SkRegion, rgnb: &SkRegion, op: SkRegionOp) -> bool {
        let mut result = SkRegion::new();
        let non_empty = SkRegion::oper(rgna, rgnb, op, Some(&mut result));
        *self = result;
        non_empty
    }

    /// `SkRegion::Oper`.
    fn oper(
        rgna_orig: &SkRegion,
        rgnb_orig: &SkRegion,
        mut op: SkRegionOp,
        mut result: Option<&mut SkRegion>,
    ) -> bool {
        debug_assert!((op.to_i32() as usize) < SkRegionOp::K_OP_CNT);

        if op == SkRegionOp::Replace {
            return set_region_check(result, rgnb_orig);
        }

        let mut rgna = rgna_orig;
        let mut rgnb = rgnb_orig;

        // collapse difference and reverse-difference into just difference
        if op == SkRegionOp::ReverseDifference {
            std::mem::swap(&mut rgna, &mut rgnb);
            op = SkRegionOp::Difference;
        }

        let a_empty = rgna.is_empty();
        let b_empty = rgnb.is_empty();
        let a_rect = rgna.is_rect();
        let b_rect = rgnb.is_rect();

        match op {
            SkRegionOp::Difference => {
                if a_empty {
                    return set_empty_check(result.as_deref_mut());
                }
                if b_empty || !IRect::intersects_irects(&rgna.bounds, &rgnb.bounds) {
                    return set_region_check(result.as_deref_mut(), rgna);
                }
                if b_rect && rgnb.bounds.contains_no_empty_check(&rgna.bounds) {
                    return set_empty_check(result.as_deref_mut());
                }
            }
            SkRegionOp::Intersect => {
                let mut bounds = IRect::make_empty();
                if (a_empty | b_empty) || !bounds.intersect_irects(rgna.bounds, rgnb.bounds) {
                    return set_empty_check(result.as_deref_mut());
                }
                if a_rect & b_rect {
                    return set_rect_check(result.as_deref_mut(), bounds);
                }
                if a_rect && rgna.bounds.contains_irect(&rgnb.bounds) {
                    return set_region_check(result.as_deref_mut(), rgnb);
                }
                if b_rect && rgnb.bounds.contains_irect(&rgna.bounds) {
                    return set_region_check(result.as_deref_mut(), rgna);
                }
            }
            SkRegionOp::Union => {
                if a_empty {
                    return set_region_check(result.as_deref_mut(), rgnb);
                }
                if b_empty {
                    return set_region_check(result.as_deref_mut(), rgna);
                }
                if a_rect && rgna.bounds.contains_irect(&rgnb.bounds) {
                    return set_region_check(result.as_deref_mut(), rgna);
                }
                if b_rect && rgnb.bounds.contains_irect(&rgna.bounds) {
                    return set_region_check(result.as_deref_mut(), rgnb);
                }
            }
            SkRegionOp::Xor => {
                if a_empty {
                    return set_region_check(result.as_deref_mut(), rgnb);
                }
                if b_empty {
                    return set_region_check(result.as_deref_mut(), rgna);
                }
            }
            _ => unreachable!("unknown region op"),
        }

        let a_runs = rgna.get_runs();
        let b_runs = rgnb.get_runs();

        let (array, count) = operate(&a_runs, &b_runs, op, result.is_none());

        match result {
            Some(result) => {
                let count = match count {
                    OperateResult::Count(count) => count,
                    OperateResult::QuickExitTrue => unreachable!("quick exit with a result"),
                };
                debug_assert!(count <= array.data.len());
                result.set_runs(array.data[..count].to_vec())
            }
            None => match count {
                OperateResult::QuickExitTrue => true,
                OperateResult::Count(count) => !is_run_count_empty(count),
            },
        }
    }

    // ---------------------------------------------------------------- serialization

    /// `writeToMemory`.
    pub fn write_to_memory(&self, storage: Option<&mut [u8]>) -> usize {
        if storage.is_none() {
            let mut size = std::mem::size_of::<i32>();
            if !self.is_empty() {
                size += std::mem::size_of::<IRect>();
                if self.is_complex() {
                    size += 2 * std::mem::size_of::<i32>();
                    size += self.data().unwrap().runs.len() * std::mem::size_of::<RunType>();
                }
            }
            return size;
        }

        let buffer = storage.unwrap();
        let mut pos = 0usize;
        let mut put_i32 = |value: i32, pos: &mut usize| {
            buffer[*pos..*pos + 4].copy_from_slice(&value.to_le_bytes());
            *pos += 4;
        };

        if self.is_empty() {
            put_i32(-1, &mut pos);
        } else {
            let is_rect = self.is_rect();
            let run_count = if is_rect {
                0
            } else {
                self.data().unwrap().runs.len() as i32
            };
            put_i32(run_count, &mut pos);

            for value in [
                self.bounds.left,
                self.bounds.top,
                self.bounds.right,
                self.bounds.bottom,
            ] {
                put_i32(value, &mut pos);
            }

            if !is_rect {
                let data = self.data().unwrap();
                put_i32(data.y_span_count, &mut pos);
                put_i32(data.interval_count, &mut pos);
                for &run in &data.runs {
                    put_i32(run, &mut pos);
                }
            }
        }
        pos
    }

    /// `readFromMemory`. Returns the number of bytes read, or 0 on failure.
    pub fn read_from_memory(&mut self, buffer: &[u8]) -> usize {
        let mut tmp = SkRegion::new();
        let mut pos = 0usize;

        let read_i32 = |buffer: &[u8], pos: &mut usize| -> Option<i32> {
            if buffer.len() < *pos + 4 {
                return None;
            }
            let value = i32::from_le_bytes(buffer[*pos..*pos + 4].try_into().unwrap());
            *pos += 4;
            Some(value)
        };

        let count = match read_i32(buffer, &mut pos) {
            Some(count) => count,
            None => return 0,
        };
        if count < -1 {
            return 0;
        }
        if count >= 0 {
            if buffer.len() < pos + 16 {
                return 0;
            }
            tmp.bounds = IRect::new(
                i32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap()),
                i32::from_le_bytes(buffer[pos + 4..pos + 8].try_into().unwrap()),
                i32::from_le_bytes(buffer[pos + 8..pos + 12].try_into().unwrap()),
                i32::from_le_bytes(buffer[pos + 12..pos + 16].try_into().unwrap()),
            );
            pos += 16;
            if tmp.bounds.is_empty() {
                return 0;
            }
            if count == 0 {
                tmp.run_head = RunHead::Rect;
            } else {
                let y_span_count = match read_i32(buffer, &mut pos) {
                    Some(value) => value,
                    None => return 0,
                };
                let interval_count = match read_i32(buffer, &mut pos) {
                    Some(value) => value,
                    None => return 0,
                };
                let run_count = count as usize;
                if buffer.len() < pos + run_count * 4 {
                    return 0;
                }
                let mut runs = vec![0i32; run_count];
                for (i, run) in runs.iter_mut().enumerate() {
                    *run = i32::from_le_bytes(
                        buffer[pos + i * 4..pos + i * 4 + 4].try_into().unwrap(),
                    );
                }
                if !validate_run(&runs, run_count, &tmp.bounds, y_span_count, interval_count) {
                    return 0;
                }
                tmp.run_head = RunHead::Complex(Arc::new(RunData {
                    runs,
                    y_span_count,
                    interval_count,
                }));
                pos += run_count * 4;
            }
        }
        self.swap(&mut tmp);
        pos
    }

    /// `isValid`.
    pub fn is_valid(&self) -> bool {
        if self.is_empty() {
            return self.bounds == IRect::new(0, 0, 0, 0);
        }
        if self.bounds.is_empty() {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        match self.data() {
            Some(data) => validate_run(
                &data.runs,
                data.runs.len(),
                &self.bounds,
                data.y_span_count,
                data.interval_count,
            ),
            None => false,
        }
    }

    /// `dump` (development helper).
    pub fn dump(&self) -> String {
        if self.is_empty() {
            return "  rgn: empty\n".to_string();
        }
        let mut out = format!(
            "  rgn: [{} {} {} {}]",
            self.bounds.left, self.bounds.top, self.bounds.right, self.bounds.bottom
        );
        if let Some(data) = self.data() {
            for run in &data.runs {
                out.push_str(&format!(" {run}"));
            }
        }
        out.push('\n');
        out
    }
}

#[inline]
fn is_run_count_empty(count: usize) -> bool {
    count <= 2
}

fn scanline_contains(runs: &[RunType], i: usize, l: RunType, r: RunType) -> bool {
    let mut i = i + 2; // skip Bottom and IntervalCount
    loop {
        if l < runs[i] {
            break;
        }
        if r <= runs[i + 1] {
            return true;
        }
        i += 2;
    }
    false
}

fn scanline_intersects(runs: &[RunType], i: usize, l: RunType, r: RunType) -> bool {
    let mut i = i + 2; // skip Bottom and IntervalCount
    loop {
        if r <= runs[i] {
            break;
        }
        if l < runs[i + 1] {
            return true;
        }
        i += 2;
    }
    false
}

/// `pin_offset_s32`: returns an offset such that applying it to `min` and `max` cannot overflow.
fn pin_offset_s32(min: i32, max: i32, mut offset: i32) -> i32 {
    debug_assert!(min <= max);
    let lo = -i32::MAX - 1;
    let hi = i32::MAX;
    if (min as i64) + (offset as i64) < (lo as i64) {
        offset = (lo as i64 - min as i64) as i32;
    }
    if (max as i64) + (offset as i64) > (hi as i64) {
        offset = (hi as i64 - max as i64) as i32;
    }
    offset
}

// ---------------------------------------------------------------------------------------------
// Span operation machinery (`RunArray`, `spanRec`, `RgnOper`, `operate`).

/// `RunArray`: a growable, never-shrinking run buffer.
struct RunArray {
    data: Vec<RunType>,
}

impl RunArray {
    fn new() -> RunArray {
        RunArray { data: Vec::new() }
    }

    fn resize_to_at_least(&mut self, count: usize) {
        if count > self.data.len() {
            self.data.resize(count, 0);
        }
    }
}

struct SpanRec<'a> {
    a_runs: &'a [RunType],
    b_runs: &'a [RunType],
    a_pos: usize,
    b_pos: usize,
    a_left: i32,
    a_rite: i32,
    b_left: i32,
    b_rite: i32,
    left: i32,
    rite: i32,
    inside: i32,
}

impl<'a> SpanRec<'a> {
    fn new(a_runs: &'a [RunType], a_pos: usize, b_runs: &'a [RunType], b_pos: usize) -> Self {
        let mut rec = SpanRec {
            a_runs,
            b_runs,
            a_pos,
            b_pos,
            a_left: 0,
            a_rite: 0,
            b_left: 0,
            b_rite: 0,
            left: 0,
            rite: 0,
            inside: 0,
        };
        rec.a_left = rec.a_runs[rec.a_pos];
        rec.a_rite = rec.a_runs[rec.a_pos + 1];
        rec.b_left = rec.b_runs[rec.b_pos];
        rec.b_rite = rec.b_runs[rec.b_pos + 1];
        rec.a_pos += 2;
        rec.b_pos += 2;
        rec
    }

    fn done(&self) -> bool {
        self.a_left == SK_REGION_K_RUN_TYPE_SENTINEL && self.b_left == SK_REGION_K_RUN_TYPE_SENTINEL
    }

    fn next(&mut self) {
        let mut a_flush = false;
        let mut b_flush = false;

        let mut a_left = self.a_left;
        let mut a_rite = self.a_rite;
        let mut b_left = self.b_left;
        let mut b_rite = self.b_rite;

        let inside;
        let left;
        let mut rite;

        if a_left < b_left {
            inside = 1;
            left = a_left;
            if a_rite <= b_left {
                rite = a_rite;
                a_flush = true;
            } else {
                rite = b_left;
                a_left = b_left;
            }
        } else if b_left < a_left {
            inside = 2;
            left = b_left;
            if b_rite <= a_left {
                rite = b_rite;
                b_flush = true;
            } else {
                rite = a_left;
                b_left = a_left;
            }
        } else {
            inside = 3;
            left = a_left;
            if a_rite <= b_rite {
                rite = a_rite;
                b_left = a_rite;
                a_flush = true;
            } else {
                rite = b_rite;
            }
            if b_rite <= a_rite {
                rite = b_rite;
                a_left = b_rite;
                b_flush = true;
            }
        }

        if a_flush {
            a_left = self.a_runs[self.a_pos];
            a_rite = self.a_runs[self.a_pos + 1];
            self.a_pos += 2;
        }
        if b_flush {
            b_left = self.b_runs[self.b_pos];
            b_rite = self.b_runs[self.b_pos + 1];
            self.b_pos += 2;
        }

        self.a_left = a_left;
        self.a_rite = a_rite;
        self.b_left = b_left;
        self.b_rite = b_rite;
        self.left = left;
        self.rite = rite;
        self.inside = inside;
    }
}

fn distance_to_sentinel(runs: &[RunType], first: usize) -> usize {
    let intervals = runs[first - 1] as usize;
    debug_assert_eq!(runs[first + intervals * 2], SK_REGION_K_RUN_TYPE_SENTINEL);
    intervals * 2
}

// A direct port of the reference helper, which takes the same eight parameters.
#[allow(clippy::too_many_arguments)]
fn operate_on_span(
    a_runs: &[RunType],
    a_first: usize,
    b_runs: &[RunType],
    b_first: usize,
    array: &mut RunArray,
    dst_offset: usize,
    min: u8,
    max: u8,
) -> usize {
    array.resize_to_at_least(
        dst_offset
            + distance_to_sentinel(a_runs, a_first)
            + distance_to_sentinel(b_runs, b_first)
            + 2,
    );
    let dst = &mut array.data;
    let mut d = dst_offset;
    let mut first_interval = true;

    let mut rec = SpanRec::new(a_runs, a_first, b_runs, b_first);
    while !rec.done() {
        rec.next();

        let left = rec.left;
        let rite = rec.rite;

        let inside = rec.inside;
        if ((inside - min as i32) as u32) <= ((max - min) as u32) && left < rite {
            if first_interval || dst[d - 1] < left {
                dst[d] = left;
                dst[d + 1] = rite;
                d += 2;
                first_interval = false;
            } else {
                dst[d - 1] = rite;
            }
        }
    }
    dst[d] = SK_REGION_K_RUN_TYPE_SENTINEL;
    d + 1
}

struct OpMinMax {
    min: u8,
    max: u8,
}

const fn op_min_max(op: SkRegionOp) -> OpMinMax {
    match op {
        SkRegionOp::Difference => OpMinMax { min: 1, max: 1 },
        SkRegionOp::Intersect => OpMinMax { min: 3, max: 3 },
        SkRegionOp::Union => OpMinMax { min: 1, max: 3 },
        _ => OpMinMax { min: 1, max: 2 }, // XOR
    }
}

struct RgnOper {
    min: u8,
    max: u8,
    array: RunArray,
    start_dst: usize,
    prev_dst: usize,
    prev_len: usize,
    top: RunType,
}

impl RgnOper {
    fn new(top: i32, op: SkRegionOp) -> RgnOper {
        let min_max = op_min_max(op);
        RgnOper {
            min: min_max.min,
            max: min_max.max,
            array: RunArray::new(),
            start_dst: 0,
            prev_dst: 1,
            prev_len: 0,
            top,
        }
    }

    fn add_span(
        &mut self,
        bottom: i32,
        a_runs: &[RunType],
        a_pos: usize,
        b_runs: &[RunType],
        b_pos: usize,
    ) {
        // skip X values and slots for the next Y+intervalCount
        let start = self.prev_dst + self.prev_len + 2;
        let stop = operate_on_span(
            a_runs,
            a_pos,
            b_runs,
            b_pos,
            &mut self.array,
            start,
            self.min,
            self.max,
        );
        let len = stop - start;
        debug_assert!(len >= 1 && (len & 1) == 1);
        debug_assert_eq!(self.array.data[stop - 1], SK_REGION_K_RUN_TYPE_SENTINEL);

        let same = self.prev_len == len
            && (len == 1
                || self.array.data[self.prev_dst..self.prev_dst + len - 1]
                    == self.array.data[start..start + len - 1]);

        if same {
            self.array.data[self.prev_dst - 2] = bottom;
        } else if len == 1 && self.prev_len == 0 {
            self.top = bottom;
        } else {
            self.array.data[start - 2] = bottom;
            self.array.data[start - 1] = (len >> 1) as i32;
            self.prev_dst = start;
            self.prev_len = len;
        }
    }

    fn flush(&mut self) -> usize {
        self.array.data[self.start_dst] = self.top;
        self.array.data[self.prev_dst + self.prev_len] = SK_REGION_K_RUN_TYPE_SENTINEL;
        self.prev_dst - self.start_dst + self.prev_len + 1
    }

    fn is_empty(&self) -> bool {
        self.prev_len == 0
    }
}

enum OperateResult {
    Count(usize),
    QuickExitTrue,
}

/// `operate`.
fn operate(
    a_runs: &[RunType],
    b_runs: &[RunType],
    op: SkRegionOp,
    quick_exit: bool,
) -> (RunArray, OperateResult) {
    const G_EMPTY_SCANLINE: [RunType; 4] = [0, 0, SK_REGION_K_RUN_TYPE_SENTINEL, 0];
    let g_sentinel: &[RunType] = &G_EMPTY_SCANLINE;
    let g_sentinel_pos = 2usize;

    let mut a_pos = 0usize;
    let mut b_pos = 0usize;

    let mut a_top = a_runs[a_pos];
    a_pos += 1;
    let mut a_bot = a_runs[a_pos];
    a_pos += 1;
    let mut b_top = b_runs[b_pos];
    b_pos += 1;
    let mut b_bot = b_runs[b_pos];
    b_pos += 1;

    a_pos += 1; // skip the intervalCount
    b_pos += 1;

    let mut oper = RgnOper::new(a_top.min(b_top), op);

    let mut prev_bot = SK_REGION_K_RUN_TYPE_SENTINEL; // so we fail the first test

    while a_bot < SK_REGION_K_RUN_TYPE_SENTINEL || b_bot < SK_REGION_K_RUN_TYPE_SENTINEL {
        let mut run0: (&[RunType], usize) = (g_sentinel, g_sentinel_pos);
        let mut run1: (&[RunType], usize) = (g_sentinel, g_sentinel_pos);
        let mut a_flush = false;
        let mut b_flush = false;
        let top;
        let mut bot = 0;

        if a_top < b_top {
            top = a_top;
            run0 = (a_runs, a_pos);
            if a_bot <= b_top {
                bot = a_bot;
                a_flush = true;
            } else {
                bot = b_top;
                a_top = b_top;
            }
        } else if b_top < a_top {
            top = b_top;
            run1 = (b_runs, b_pos);
            if b_bot <= a_top {
                bot = b_bot;
                b_flush = true;
            } else {
                bot = a_top;
                b_top = a_top;
            }
        } else {
            top = a_top;
            run0 = (a_runs, a_pos);
            run1 = (b_runs, b_pos);
            if a_bot <= b_bot {
                bot = a_bot;
                b_top = a_bot;
                a_flush = true;
            }
            if b_bot <= a_bot {
                bot = b_bot;
                a_top = b_bot;
                b_flush = true;
            }
        }

        if top > prev_bot {
            oper.add_span(top, g_sentinel, g_sentinel_pos, g_sentinel, g_sentinel_pos);
        }
        oper.add_span(bot, run0.0, run0.1, run1.0, run1.1);

        if quick_exit && !oper.is_empty() {
            let array = std::mem::replace(&mut oper.array, RunArray::new());
            return (array, OperateResult::QuickExitTrue);
        }

        if a_flush {
            a_pos = skip_intervals(a_runs, a_pos);
            a_top = a_bot;
            a_bot = a_runs[a_pos];
            a_pos += 1;
            a_pos += 1; // skip uninitialized intervalCount
            if a_bot == SK_REGION_K_RUN_TYPE_SENTINEL {
                a_top = a_bot;
            }
        }
        if b_flush {
            b_pos = skip_intervals(b_runs, b_pos);
            b_top = b_bot;
            b_bot = b_runs[b_pos];
            b_pos += 1;
            b_pos += 1; // skip uninitialized intervalCount
            if b_bot == SK_REGION_K_RUN_TYPE_SENTINEL {
                b_top = b_bot;
            }
        }

        prev_bot = bot;
    }
    let count = oper.flush();
    let array = std::mem::replace(&mut oper.array, RunArray::new());
    (array, OperateResult::Count(count))
}

fn skip_intervals(runs: &[RunType], pos: usize) -> usize {
    let intervals = runs[pos - 1];
    debug_assert!(intervals >= 0);
    pos + intervals as usize * 2 + 1
}

// ---------------------------------------------------------------------------------------------
// Result helpers used by `Oper`.

fn set_empty_check(result: Option<&mut SkRegion>) -> bool {
    match result {
        Some(result) => result.set_empty(),
        None => false,
    }
}

fn set_rect_check(result: Option<&mut SkRegion>, rect: IRect) -> bool {
    match result {
        Some(result) => result.set_rect(&rect),
        None => !rect.is_empty(),
    }
}

fn set_region_check(result: Option<&mut SkRegion>, rgn: &SkRegion) -> bool {
    match result {
        Some(result) => result.set_region(rgn),
        None => !rgn.is_empty(),
    }
}

// ---------------------------------------------------------------------------------------------
// Serialized-region validation.

fn validate_run_count(y_span_count: i32, interval_count: i32, run_count: i32) -> bool {
    if y_span_count < 1 || interval_count < 2 {
        return false;
    }
    let mut safe = SkSafeMath::new();
    let mut sum = 2;
    sum = safe.add_int(sum, y_span_count);
    sum = safe.add_int(sum, y_span_count);
    sum = safe.add_int(sum, y_span_count);
    sum = safe.add_int(sum, interval_count);
    sum = safe.add_int(sum, interval_count);
    safe.ok() && sum == run_count
}

/// `validate_run`: never reads beyond `runs[run_count - 1]`.
fn validate_run(
    runs: &[i32],
    run_count: usize,
    given_bounds: &IRect,
    mut y_span_count: i32,
    mut interval_count: i32,
) -> bool {
    if !validate_run_count(y_span_count, interval_count, run_count as i32) {
        return false;
    }
    if run_count < 7 {
        return false;
    }
    if runs[run_count - 1] != SK_REGION_K_RUN_TYPE_SENTINEL
        || runs[run_count - 2] != SK_REGION_K_RUN_TYPE_SENTINEL
    {
        return false;
    }

    let mut bounds = IRect::new(0, 0, 0, 0);
    let mut rect = IRect::new(0, 0, 0, 0);
    let mut prev_was_empty = true;
    let mut i = 0usize;

    rect.top = runs[i];
    i += 1;
    if rect.top == SK_REGION_K_RUN_TYPE_SENTINEL {
        return false;
    }
    if rect.top != given_bounds.top {
        return false;
    }

    loop {
        y_span_count -= 1;
        if y_span_count < 0 {
            return false;
        }
        rect.bottom = runs[i];
        i += 1;
        if rect.bottom == SK_REGION_K_RUN_TYPE_SENTINEL {
            return false;
        }
        if rect.bottom > given_bounds.bottom {
            return false;
        }
        if rect.bottom <= rect.top {
            return false;
        }

        let mut x_intervals = runs[i];
        i += 1;
        if x_intervals < 0
            || x_intervals > interval_count
            || i + 1 + 2 * x_intervals as usize > run_count
        {
            return false;
        }
        if x_intervals == 0 {
            if prev_was_empty {
                return false;
            }
            prev_was_empty = true;
        } else {
            prev_was_empty = false;
        }
        interval_count -= x_intervals;
        let mut first_interval = true;
        let mut last_right = 0i32;
        while x_intervals > 0 {
            x_intervals -= 1;
            rect.left = runs[i];
            rect.right = runs[i + 1];
            i += 2;
            if rect.left == SK_REGION_K_RUN_TYPE_SENTINEL
                || rect.right == SK_REGION_K_RUN_TYPE_SENTINEL
                || rect.left >= rect.right
                || (!first_interval && rect.left <= last_right)
            {
                return false;
            }
            last_right = rect.right;
            first_interval = false;
            bounds.join(&rect);
        }
        if runs[i] != SK_REGION_K_RUN_TYPE_SENTINEL {
            return false;
        }
        i += 1;
        rect.top = rect.bottom;
        if runs[i] == SK_REGION_K_RUN_TYPE_SENTINEL {
            break;
        }
    }
    i += 1;
    if y_span_count != 0 || interval_count != 0 || *given_bounds != bounds {
        return false;
    }
    i == run_count
}

// ---------------------------------------------------------------------------------------------
// Iterators.

/// `SkRegion::Iterator`.
#[derive(Clone, Debug)]
pub struct Iterator {
    f_rgn: Option<SkRegion>,
    // See the RLE encoding at the top of SkRegion.cpp.
    f_runs: Option<usize>,
    f_is_rect: bool,
    f_rect: IRect,
    f_done: bool,
}

impl Iterator {
    /// `Iterator()`.
    pub fn new() -> Iterator {
        Iterator {
            f_rgn: None,
            f_runs: None,
            f_is_rect: false,
            f_rect: IRect::new(0, 0, 0, 0),
            f_done: true,
        }
    }

    /// `Iterator(const SkRegion& region)`.
    pub fn from_region(region: &SkRegion) -> Iterator {
        let mut iter = Iterator::new();
        iter.reset(region);
        iter
    }

    /// `rewind`.
    pub fn rewind(&mut self) -> bool {
        if let Some(rgn) = self.f_rgn.clone() {
            self.reset(&rgn);
            return true;
        }
        false
    }

    /// `reset`.
    pub fn reset(&mut self, rgn: &SkRegion) {
        self.f_rgn = Some(rgn.clone());
        self.f_is_rect = false;
        self.f_runs = None;
        if rgn.is_empty() {
            self.f_done = true;
            return;
        }
        self.f_done = false;
        if rgn.is_rect() {
            self.f_rect = rgn.bounds;
            self.f_is_rect = true;
            self.f_runs = None;
        } else {
            let runs = &rgn.data().unwrap().runs;
            self.f_rect = IRect::new(runs[3], runs[0], runs[4], runs[1]);
            self.f_runs = Some(5);
        }
    }

    /// `done`.
    #[inline]
    pub fn done(&self) -> bool {
        self.f_done
    }

    /// `next`.
    pub fn next(&mut self) {
        if self.f_done {
            return;
        }
        if self.f_is_rect {
            self.f_done = true;
            return;
        }

        let runs = &self.f_rgn.as_ref().unwrap().data().unwrap().runs;
        let mut i = self.f_runs.unwrap();

        if runs[i] < SK_REGION_K_RUN_TYPE_SENTINEL {
            // valid X value
            self.f_rect.left = runs[i];
            self.f_rect.right = runs[i + 1];
            i += 2;
        } else {
            // we're at the end of a line
            i += 1;
            if runs[i] < SK_REGION_K_RUN_TYPE_SENTINEL {
                // valid Y value
                let intervals = runs[i + 1];
                if intervals == 0 {
                    // empty line
                    self.f_rect.top = runs[i];
                    i += 3;
                } else {
                    self.f_rect.top = self.f_rect.bottom;
                }
                self.f_rect.bottom = runs[i];
                self.f_rect.left = runs[i + 2];
                self.f_rect.right = runs[i + 3];
                i += 4;
            } else {
                // end of rgn
                self.f_done = true;
            }
        }
        self.f_runs = Some(i);
    }

    /// `rect`.
    #[inline]
    pub fn rect(&self) -> &IRect {
        &self.f_rect
    }

    /// `rgn`.
    #[inline]
    pub fn rgn(&self) -> Option<&SkRegion> {
        self.f_rgn.as_ref()
    }
}

impl Default for Iterator {
    fn default() -> Self {
        Self::new()
    }
}

/// `SkRegion::Cliperator`.
#[derive(Clone, Debug)]
pub struct Cliperator {
    f_iter: Iterator,
    f_clip: IRect,
    f_rect: IRect,
    f_done: bool,
}

impl Cliperator {
    /// `Cliperator(const SkRegion& region, const SkIRect& clip)`.
    pub fn new(region: &SkRegion, clip: &IRect) -> Cliperator {
        let mut f_iter = Iterator::from_region(region);
        let mut f_rect = IRect::new(0, 0, 0, 0);
        let mut f_done = true;

        while !f_iter.done() {
            let r = *f_iter.rect();
            if r.top >= clip.bottom {
                break;
            }
            if f_rect.intersect_irects(*clip, r) {
                f_done = false;
                break;
            }
            f_iter.next();
        }

        Cliperator {
            f_iter,
            f_clip: *clip,
            f_rect,
            f_done,
        }
    }

    /// `done`.
    #[inline]
    pub fn done(&self) -> bool {
        self.f_done
    }

    /// `next`.
    pub fn next(&mut self) {
        if self.f_done {
            return;
        }
        self.f_done = true;
        self.f_iter.next();
        while !self.f_iter.done() {
            let r = *self.f_iter.rect();
            if r.top >= self.f_clip.bottom {
                break;
            }
            if self.f_rect.intersect_irects(self.f_clip, r) {
                self.f_done = false;
                break;
            }
            self.f_iter.next();
        }
    }

    /// `rect`.
    #[inline]
    pub fn rect(&self) -> &IRect {
        &self.f_rect
    }
}

/// `SkRegion::Spanerator`.
#[derive(Clone, Debug)]
pub struct Spanerator {
    f_rgn: Option<SkRegion>,
    f_runs: Option<usize>,
    f_is_rect: bool,
    f_left: i32,
    f_right: i32,
    f_done: bool,
}

impl Spanerator {
    /// `Spanerator(const SkRegion& region, int y, int left, int right)`.
    pub fn new(rgn: &SkRegion, y: i32, left: i32, right: i32) -> Spanerator {
        let r = *rgn.get_bounds();
        let mut span = Spanerator {
            f_rgn: Some(rgn.clone()),
            f_runs: None,
            f_is_rect: false,
            f_left: 0,
            f_right: 0,
            f_done: true,
        };

        if !rgn.is_empty() && y >= r.top && y < r.bottom && right > r.left && left < r.right {
            if rgn.is_rect() {
                span.f_left = left.max(r.left);
                span.f_right = right.min(r.right);
                span.f_is_rect = true;
                span.f_done = false;
            } else {
                let runs = &rgn.data().unwrap().runs;
                let mut i = rgn.find_scanline(y) + 2; // skip Bottom and IntervalCount
                loop {
                    // runs[i..i+1] is to the right of the span, so we're done
                    if runs[i] >= right {
                        break;
                    }
                    // runs[i..i+1] is to the left of the span, so continue
                    if runs[i + 1] <= left {
                        i += 2;
                        continue;
                    }
                    span.f_runs = Some(i);
                    span.f_left = left;
                    span.f_right = right;
                    span.f_done = false;
                    break;
                }
            }
        }
        span
    }

    /// `next`.
    pub fn next(&mut self, left: Option<&mut i32>, right: Option<&mut i32>) -> bool {
        if self.f_done {
            return false;
        }

        if self.f_is_rect {
            self.f_done = true;
            if let Some(left) = left {
                *left = self.f_left;
            }
            if let Some(right) = right {
                *right = self.f_right;
            }
            return true;
        }

        let runs = &self.f_rgn.as_ref().unwrap().data().unwrap().runs;
        let i = self.f_runs.unwrap();

        if runs[i] >= self.f_right {
            self.f_done = true;
            return false;
        }

        debug_assert!(runs[i + 1] > self.f_left);
        if let Some(left) = left {
            *left = self.f_left.max(runs[i]);
        }
        if let Some(right) = right {
            *right = self.f_right.min(runs[i + 1]);
        }
        self.f_runs = Some(i + 2);
        true
    }
}
