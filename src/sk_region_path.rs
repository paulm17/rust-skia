//! Port of `src/core/SkRegion_path.cpp` (`SkRegion::setPath`, `SkRegion::addBoundaryPath`,
//! `SkRegion::getBoundaryPath`) plus the non-antialiased path fill it reaches.
//!
//! `SkRegion_path.cpp` is listed under M07 in the source inventory but was explicitly deferred to
//! M09 by the M07 handoff. Its `setPath` calls `SkScan::FillPath` (M14). To keep the accepted M09
//! feature set real rather than stubbed, the `SkEdge`/`SkBasicEdgeBuilder` units and the
//! `sk_fill_path` walker were ported here as an early ownership transfer; see
//! `plan/handoffs/M09.md`.

use crate::fixed::sk_fixed_round_to_int;
use crate::floating_point::double_saturate_to_int;
use crate::rect::{IRect, Rect};
use crate::rect_priv::RectPriv;
use crate::safe_math::SkSafeMath;
use crate::scalar::{Scalar, scalar_round_to_int};
use crate::sk_edge::{SkEdge, SkEdgeWinding};
use crate::sk_edge_builder::SkBasicEdgeBuilder;
use crate::sk_geometry::SkPathDirection;
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_iter::SkPathIter;
use crate::sk_path_types::{SkPathFillType, SkPathVerb, SkResolveConvexity};
use crate::sk_region::{Iterator as RegionIterator, SkRegion, SkRegionOp};

/// A sink for horizontal spans, the subset of `SkBlitter` that region scan conversion needs.
pub trait SkScanSpanSink {
    /// `SkBlitter::blitH`.
    fn blit_h(&mut self, x: i32, y: i32, width: i32);

    /// `SkBlitter::blitRect`.
    fn blit_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let mut y = y;
        let mut height = height;
        while height > 0 {
            self.blit_h(x, y, width);
            y += 1;
            height -= 1;
        }
    }

    /// `SkBlitter::blitRectRegion`.
    fn blit_rect_region(&mut self, rect: &IRect, clip: &SkRegion) {
        if clip.is_rect() {
            let mut cr = IRect::make_empty();
            if cr.intersect_irects(*rect, *clip.get_bounds()) {
                self.blit_rect(cr.left, cr.top, cr.width(), cr.height());
            }
            return;
        }
        let mut clipper = crate::sk_region::Cliperator::new(clip, rect);
        while !clipper.done() {
            let cr = *clipper.rect();
            self.blit_rect(cr.left, cr.top, cr.width(), cr.height());
            clipper.next();
        }
    }

    /// `SkBlitter::blitRegion`.
    fn blit_region(&mut self, clip: &SkRegion) {
        // VisitSpans borrows `clip` and the visitor mutably borrows `self`; collect first to keep
        // the borrow checker happy. Span order (Y -> X ascending) is preserved.
        let mut rects: Vec<IRect> = Vec::new();
        crate::sk_region_priv::SkRegionPriv::visit_spans(clip, |r| rects.push(*r));
        for r in rects {
            self.blit_rect(r.left, r.top, r.width(), r.height());
        }
    }

    /// The pre/post hook used by `InverseBlitter`.
    fn prepost(&mut self, _y: i32, _is_start: bool) {}
}

/// `SkRgnBuilder`: builds a region from `blitH` spans.
#[derive(Debug)]
pub struct SkRgnBuilder {
    rows: Vec<BuilderRow>,
    current: Option<BuilderRow>,
    prev: Option<usize>,
    top: i32,
}

#[derive(Clone, Debug)]
struct BuilderRow {
    last_y: i32,
    xs: Vec<i32>,
}

impl SkRgnBuilder {
    pub fn new() -> SkRgnBuilder {
        SkRgnBuilder {
            rows: Vec::new(),
            current: None,
            prev: None,
            top: 0,
        }
    }

    /// `SkRgnBuilder::init`. The reference preallocates a fixed working buffer; the port grows a
    /// `Vec` instead. The same overflow checks decide the return value.
    pub fn init(&mut self, max_height: i32, max_transitions: i32, path_is_inverse: bool) -> bool {
        if (max_height | max_transitions) < 0 {
            return false;
        }
        let mut safe = SkSafeMath::new();
        let mut max_transitions = max_transitions;
        if path_is_inverse {
            max_transitions = safe.add_int(max_transitions, 2);
        }
        let height_term = safe.add_int(max_height, 1) as usize;
        let transition_term = safe.add_int(3, max_transitions) as usize;
        let mut count = safe.mul(height_term, transition_term);
        if path_is_inverse {
            count = safe.add(count, 10);
            let _ = count;
        }
        self.rows.clear();
        self.current = None;
        self.prev = None;
        safe.ok()
    }

    pub fn done(&mut self) {
        if self.current.is_some() {
            self.finish_current();
        }
    }

    /// `SkRgnBuilder::computeRunCount`.
    pub fn compute_run_count(&self) -> usize {
        if self.current.is_none() && self.rows.is_empty() {
            return 0;
        }
        let mut count = 2;
        for row in &self.rows {
            count += 2 + row.xs.len() + 1;
        }
        count
    }

    /// `SkRgnBuilder::copyToRect`.
    pub fn copy_to_rect(&self, r: &mut IRect) {
        debug_assert_eq!(self.rows.len(), 1);
        let row = &self.rows[0];
        debug_assert_eq!(row.xs.len(), 2);
        r.set_ltrb(row.xs[0], self.top, row.xs[1], row.last_y + 1);
    }

    /// `SkRgnBuilder::copyToRgn`.
    pub fn copy_to_rgn(&self) -> Vec<i32> {
        let mut runs = Vec::with_capacity(self.compute_run_count());
        runs.push(self.top);
        for row in &self.rows {
            runs.push(row.last_y + 1);
            runs.push((row.xs.len() / 2) as i32);
            runs.extend_from_slice(&row.xs);
            runs.push(crate::sk_region::SK_REGION_K_RUN_TYPE_SENTINEL);
        }
        runs.push(crate::sk_region::SK_REGION_K_RUN_TYPE_SENTINEL);
        runs
    }

    /// Finalize the current row, merging it into the previous row when the reference's
    /// `collapsWithPrev()` would.
    fn finish_current(&mut self) {
        let curr = self.current.take().unwrap();
        if let Some(p) = self.prev {
            if self.rows[p].last_y + 1 == curr.last_y && self.rows[p].xs == curr.xs {
                self.rows[p].last_y = curr.last_y;
                return;
            }
        }
        self.rows.push(curr);
        self.prev = Some(self.rows.len() - 1);
    }

    fn add_span(&mut self, x: i32, y: i32, width: i32) {
        if self.current.is_none() {
            self.top = y;
            self.current = Some(BuilderRow {
                last_y: y,
                xs: Vec::new(),
            });
        } else if y > self.current.as_ref().unwrap().last_y {
            let prev_last_y = self.current.as_ref().unwrap().last_y;
            self.finish_current();
            if y - 1 > prev_last_y {
                // insert an empty run
                self.rows.push(BuilderRow {
                    last_y: y - 1,
                    xs: Vec::new(),
                });
                self.prev = Some(self.rows.len() - 1);
            }
            self.current = Some(BuilderRow {
                last_y: y,
                xs: Vec::new(),
            });
        }

        let row = self.current.as_mut().unwrap();
        if !row.xs.is_empty() && *row.xs.last().unwrap() == x {
            let last = row.xs.last_mut().unwrap();
            *last = x + width;
        } else {
            row.xs.push(x);
            row.xs.push(x + width);
        }
    }
}

impl Default for SkRgnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SkScanSpanSink for SkRgnBuilder {
    fn blit_h(&mut self, x: i32, y: i32, width: i32) {
        self.add_span(x, y, width);
    }
}

/// `InverseBlitter`: rewrites the spans it receives into their complement within `clip`.
struct InverseBlitter<'a> {
    sink: &'a mut dyn SkScanSpanSink,
    first_x: i32,
    last_x: i32,
    prev_x: i32,
}

impl<'a> InverseBlitter<'a> {
    fn new(sink: &'a mut dyn SkScanSpanSink, clip: &IRect) -> InverseBlitter<'a> {
        InverseBlitter {
            sink,
            first_x: clip.left,
            last_x: clip.right,
            prev_x: clip.left,
        }
    }
}

impl SkScanSpanSink for InverseBlitter<'_> {
    fn blit_h(&mut self, x: i32, y: i32, width: i32) {
        let inv_width = x - self.prev_x;
        if inv_width > 0 {
            self.sink.blit_h(self.prev_x, y, inv_width);
        }
        self.prev_x = x + width;
    }

    fn prepost(&mut self, y: i32, is_start: bool) {
        if is_start {
            self.prev_x = self.first_x;
        } else {
            let inv_width = self.last_x - self.prev_x;
            if inv_width > 0 {
                self.sink.blit_h(self.prev_x, y, inv_width);
            }
        }
    }
}

/// `SkRectClipBlitter`: clips spans horizontally to a rect.
struct RectClipSpanSink<'a> {
    sink: &'a mut dyn SkScanSpanSink,
    clip: IRect,
}

impl SkScanSpanSink for RectClipSpanSink<'_> {
    fn blit_h(&mut self, left: i32, y: i32, width: i32) {
        if y < self.clip.top || y >= self.clip.bottom {
            return;
        }
        let mut left = left;
        let mut right = left + width;
        if left < self.clip.left {
            left = self.clip.left;
        }
        if right > self.clip.right {
            right = self.clip.right;
        }
        let width = right - left;
        if width > 0 {
            self.sink.blit_h(left, y, width);
        }
    }

    fn prepost(&mut self, y: i32, is_start: bool) {
        self.sink.prepost(y, is_start);
    }
}

/// `SkScan::PathRequiresTiling`.
pub fn sk_scan_path_requires_tiling(bounds: &IRect) -> bool {
    let mut out = SkRegion::new();
    clip_to_limit(&SkRegion::from_irect(*bounds), &mut out)
}

fn clip_to_limit(orig: &SkRegion, reduced: &mut SkRegion) -> bool {
    // Need to limit coordinates such that the width/height of our rect can be represented in
    // SkFixed (16.16). See skbug.com/40039252.
    let limit = 32767 >> 1;
    let limit_r = IRect::new(-limit, -limit, limit, limit);
    if limit_r.contains_irect(orig.get_bounds()) {
        return false;
    }
    reduced.op_region_irect(orig, &limit_r, SkRegionOp::Intersect);
    true
}

// Bias used for conservative rounding of float rects to int rects.
const K_CONSERVATIVE_ROUND_BIAS: f64 = 0.5 + 1.5 / 64.0;

fn round_down_to_int(x: Scalar) -> i32 {
    let xx = x as f64 - K_CONSERVATIVE_ROUND_BIAS;
    double_saturate_to_int(xx.ceil())
}

fn round_up_to_int(x: Scalar) -> i32 {
    let xx = x as f64 + K_CONSERVATIVE_ROUND_BIAS;
    double_saturate_to_int(xx.floor())
}

fn conservative_round_to_int(src: &Rect) -> IRect {
    IRect::new(
        round_down_to_int(src.left),
        round_down_to_int(src.top),
        round_up_to_int(src.right),
        round_up_to_int(src.bottom),
    )
}

/// `sk_fill_path`: convert `raw` into spans inside `clip_rect` and send them to `sink`.
fn sk_fill_path(
    raw: &crate::sk_path_raw::SkPathRaw,
    clip_rect: &IRect,
    sink: &mut dyn SkScanSpanSink,
    mut start_y: i32,
    mut stop_y: i32,
    path_contained_in_clip: bool,
) {
    let mut builder = SkBasicEdgeBuilder::new();
    let count = builder.build_edges(
        raw,
        if path_contained_in_clip {
            None
        } else {
            Some(clip_rect)
        },
    );
    let mut edges = builder.take_edges();

    if count == 0 {
        if raw.is_inverse_fill_type() {
            // Since we are in inverse-fill, our caller has already drawn above our top (start_y)
            // and will draw below our bottom (stop_y).
            let mut rect = *clip_rect;
            if rect.top < start_y {
                rect.top = start_y;
            }
            if rect.bottom > stop_y {
                rect.bottom = stop_y;
            }
            if !rect.is_empty() {
                sink.blit_rect(rect.left, rect.top, rect.width(), rect.height());
            }
        }
        return;
    }

    if !path_contained_in_clip {
        start_y = start_y.max(clip_rect.top);
        stop_y = stop_y.min(clip_rect.bottom);
    }

    let inverse = raw.is_inverse_fill_type();
    let fill_type = raw.fill_type();
    if inverse {
        let mut inverse_blitter = InverseBlitter::new(sink, clip_rect);
        walk_edges(
            &mut edges,
            fill_type,
            &mut inverse_blitter,
            start_y,
            stop_y,
            clip_rect.right,
        );
    } else {
        walk_edges(
            &mut edges,
            fill_type,
            sink,
            start_y,
            stop_y,
            clip_rect.right,
        );
    }
}

/// `walk_edges`. The reference's linked-list maintenance is replaced by keeping the live edges in
/// a `Vec` that is x-sorted before each scanline; the per-scanline span sequence is unchanged.
fn walk_edges(
    edges: &mut [SkEdge],
    fill_type: SkPathFillType,
    sink: &mut dyn SkScanSpanSink,
    start_y: i32,
    stop_y: i32,
    right_clip: i32,
) {
    if edges.is_empty() {
        return;
    }
    // compare_edges
    edges.sort_by_key(|a| (a.f_first_y(), a.f_x()));

    // Indices into `edges`, x-sorted. Edges are never removed from `edges`, which keeps the
    // indices stable; finished edges are dropped from `active` instead.
    let mut active: Vec<usize> = Vec::new();
    let mut next_unstarted = 0usize;
    let mut curr_y = start_y;
    let winding_mask = if fill_type.is_even_odd() { 1 } else { -1 };

    while curr_y < stop_y {
        // insert_new_edges: every edge whose first scanline is at or above `curr_y`.
        while next_unstarted < edges.len() && edges[next_unstarted].f_first_y() <= curr_y {
            if edges[next_unstarted].f_last_y() >= curr_y {
                active.push(next_unstarted);
            }
            next_unstarted += 1;
        }
        active.sort_by_key(|&i| edges[i].f_x());

        let mut w = 0i32;
        let mut left = 0i32;
        sink.prepost(curr_y, true);

        let mut keep = vec![true; active.len()];
        for (pos, &i) in active.iter().enumerate() {
            let edge = &mut edges[i];
            let x = sk_fixed_round_to_int(edge.f_x());

            if (w & winding_mask) == 0 {
                left = x;
            }
            w += match edge.base().f_winding {
                SkEdgeWinding::Cw => 1,
                SkEdgeWinding::Ccw => -1,
            };
            if (w & winding_mask) == 0 {
                let width = x - left;
                debug_assert!(width >= 0);
                if width > 0 {
                    sink.blit_h(left, curr_y, width);
                }
            }

            if edge.f_last_y() == curr_y {
                if edge.has_next_segment() && edge.next_segment() {
                    debug_assert_eq!(edge.f_first_y(), curr_y + 1);
                } else {
                    keep[pos] = false;
                }
            } else {
                let new_x = edge.f_x().wrapping_add(edge.f_dxdy());
                edge.base_mut().f_x = new_x;
            }
        }

        if (w & winding_mask) != 0 {
            // was our right-edge culled away?
            let width = right_clip - left;
            if width > 0 {
                sink.blit_h(left, curr_y, width);
            }
        }

        sink.prepost(curr_y, false);

        let mut pos = 0usize;
        active.retain(|_| {
            let keep_this = keep[pos];
            pos += 1;
            keep_this
        });

        curr_y += 1;
    }
}

/// `SkScan::FillPath(raw, const SkRegion& clip, SkBlitter*)` for a rect clip.
///
/// `SkRegion::setPath` (and `SkAAClip::setPath` for the non-AA case) only pass rectangular clips
/// here: complex clips are handled by an intersect afterwards. The port keeps that precondition
/// explicit instead of porting `SkRgnClipBlitter`, which belongs to the M14 raster pipeline.
pub fn sk_scan_fill_path(
    raw: &crate::sk_path_raw::SkPathRaw,
    clip: &SkRegion,
    sink: &mut dyn SkScanSpanSink,
) {
    if clip.is_empty() {
        return;
    }
    debug_assert!(clip.is_rect());

    let mut finite_clip_storage = SkRegion::new();
    let clip_ptr: &SkRegion = if clip_to_limit(clip, &mut finite_clip_storage) {
        if finite_clip_storage.is_empty() {
            return;
        }
        &finite_clip_storage
    } else {
        clip
    };

    let large = RectPriv::make_large_s32();
    let mut bounds = raw.bounds();
    let mut ir_pre_clipped = false;
    if !large.contains_rect(&bounds) {
        if !bounds.intersect(&large) {
            bounds.set_empty();
        }
        ir_pre_clipped = true;
    }

    let ir = conservative_round_to_int(&bounds);
    if ir.is_empty() {
        if raw.is_inverse_fill_type() {
            sink.blit_region(clip_ptr);
        }
        return;
    }

    let clip_bounds = *clip_ptr.get_bounds();
    let use_direct = !ir_pre_clipped && clip_bounds.contains_irect(&ir);
    let above = IRect::new(clip_bounds.left, clip_bounds.top, clip_bounds.right, ir.top);
    let below = IRect::new(
        clip_bounds.left,
        ir.bottom,
        clip_bounds.right,
        clip_bounds.bottom,
    );

    if use_direct {
        if raw.is_inverse_fill_type() && !above.is_empty() {
            sink.blit_rect_region(&above, clip_ptr);
        }
        sk_fill_path(raw, &clip_bounds, sink, ir.top, ir.bottom, true);
        if raw.is_inverse_fill_type() && !below.is_empty() {
            sink.blit_rect_region(&below, clip_ptr);
        }
    } else {
        let mut clipped = RectClipSpanSink {
            sink,
            clip: clip_bounds,
        };
        if raw.is_inverse_fill_type() && !above.is_empty() {
            clipped.blit_rect_region(&above, clip_ptr);
        }
        sk_fill_path(raw, &clip_bounds, &mut clipped, ir.top, ir.bottom, false);
        if raw.is_inverse_fill_type() && !below.is_empty() {
            clipped.blit_rect_region(&below, clip_ptr);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// `SkRegion` path methods.

fn verb_to_initial_last_index(verb: SkPathVerb) -> usize {
    match verb {
        SkPathVerb::Move => 0,
        SkPathVerb::Line => 1,
        SkPathVerb::Quad | SkPathVerb::Conic => 2,
        SkPathVerb::Cubic => 3,
        SkPathVerb::Close | SkPathVerb::Done => 0,
    }
}

fn verb_to_max_edges(verb: SkPathVerb) -> i32 {
    match verb {
        SkPathVerb::Move => 0,
        SkPathVerb::Line => 1,
        SkPathVerb::Quad | SkPathVerb::Conic => 2,
        SkPathVerb::Cubic => 3,
        SkPathVerb::Close | SkPathVerb::Done => 0,
    }
}

fn count_path_runtype_values(path: &SkPath, itop: &mut i32, ibot: &mut i32) -> i32 {
    let mut max_edges = 0i32;
    // SkIntToScalar(SK_MaxS16) / SkIntToScalar(SK_MinS16).
    let mut top = 32767.0 as Scalar;
    let mut bot = -32767.0 as Scalar;

    // `SkPath::Iter iter(path, true)`: the raw iterator is used here. A force-injected Close
    // contributes neither edges nor points, so the counts are identical.
    let mut iter = SkPathIter::new(path.points(), path.verbs(), path.conic_weights());
    while let Some(rec) = iter.next() {
        let verb = rec.f_verb;
        let pts = rec.f_points;
        max_edges += verb_to_max_edges(verb);

        let last_index = verb_to_initial_last_index(verb);
        if last_index > 0 {
            for i in 1..=last_index {
                if top > pts[i].y {
                    top = pts[i].y;
                } else if bot < pts[i].y {
                    bot = pts[i].y;
                }
            }
        } else if verb == SkPathVerb::Move {
            if top > pts[0].y {
                top = pts[0].y;
            } else if bot < pts[0].y {
                bot = pts[0].y;
            }
        }
    }
    if max_edges == 0 {
        return 0; // we have only moves+closes
    }
    debug_assert!(top <= bot);
    *itop = scalar_round_to_int(top);
    *ibot = scalar_round_to_int(bot);
    max_edges
}

fn check_inverse_on_empty_return(dst: &mut SkRegion, path: &SkPath, clip: &SkRegion) -> bool {
    if path.is_inverse_fill_type() {
        dst.set(clip)
    } else {
        dst.set_empty()
    }
}

impl SkRegion {
    /// `SkRegion::setPath`.
    pub fn set_path(&mut self, path: &SkPath, clip: &SkRegion) -> bool {
        let raw = path.raw(SkResolveConvexity::Yes);

        if clip.is_empty() || raw.is_none() || path.is_empty() {
            // This treats non-finite paths (no raw) as empty as well, so this returns empty or
            // 'clip' if it's inverse-filled.
            return check_inverse_on_empty_return(self, path, clip);
        }
        let raw = raw.unwrap();

        // Our builder is very fragile, and can't be called with spans/rects out of Y->X order.
        // To ensure this, we only "fill" clipped to a rect (the clip's bounds), and if the clip is
        // more complex than that, we just post-intersect the result with the clip.
        let clip_bounds = *clip.get_bounds();
        if clip.is_complex() {
            let rect_clip = SkRegion::from_irect(clip_bounds);
            if !self.set_path(path, &rect_clip) {
                return false;
            }
            return self.op_region(clip, SkRegionOp::Intersect);
        }
        debug_assert!(clip.is_rect());

        // SkScan::FillPath has limits on the coordinate range of the clipping SkRegion. If it's
        // too big, tile the clip bounds and union the pieces back together.
        if sk_scan_path_requires_tiling(&clip_bounds) {
            const K_TILE_SIZE: i32 = 32767 >> 1;
            const K_TILE_LIMIT: i32 = 1000;
            let path_bounds = path.get_bounds().round_out();

            self.set_empty();

            let mut intersection = IRect::make_empty();
            if !intersection.intersect_irects(path_bounds, clip_bounds) {
                return false;
            }

            let mut safe = SkSafeMath::new();
            let width = safe.sub_int(intersection.right, intersection.left);
            let height = safe.sub_int(intersection.bottom, intersection.top);

            let width_term = safe.add_int(width, K_TILE_SIZE - 1);
            let height_term = safe.add_int(height, K_TILE_SIZE - 1);
            let tiles_x = safe.div_int(width_term, K_TILE_SIZE);
            let tiles_y = safe.div_int(height_term, K_TILE_SIZE);

            let total_tiles = safe.mul_int(tiles_x, tiles_y);
            if !safe.ok() || total_tiles > K_TILE_LIMIT {
                return false;
            }

            let mut top = intersection.top as i64;
            while top < intersection.bottom as i64 {
                let bot = (top + K_TILE_SIZE as i64).min(intersection.bottom as i64);
                let mut left = intersection.left as i64;
                while left < intersection.right as i64 {
                    let right = (left + K_TILE_SIZE as i64).min(intersection.right as i64);

                    let mut tile_clip_bounds =
                        IRect::new(left as i32, top as i32, right as i32, bot as i32);
                    debug_assert!(IRect::intersects_irects(&path_bounds, &tile_clip_bounds));

                    // Shift coordinates so the top left is (0,0) during scan conversion and then
                    // translate the SkRegion afterwards.
                    tile_clip_bounds.offset(-(left as i32), -(top as i32));
                    debug_assert!(!sk_scan_path_requires_tiling(&tile_clip_bounds));
                    let mut tile = SkRegion::new();
                    match path.try_make_offset(-(left as Scalar), -(top as Scalar)) {
                        Some(newpath) => {
                            tile.set_path(&newpath, &SkRegion::from_irect(tile_clip_bounds));
                            tile.translate(left as i32, top as i32);
                            self.op_region(&tile, SkRegionOp::Union);
                        }
                        None => return false,
                    }
                    left += K_TILE_SIZE as i64;
                }
                top += K_TILE_SIZE as i64;
            }
            // During tiling we only applied the bounds of the tile, now that we have a full
            // SkRegion, apply the original clip.
            return self.op_region(clip, SkRegionOp::Intersect);
        }

        // compute worst-case rgn-size for the path
        let mut path_top = 0;
        let mut path_bot = 0;
        let path_transitions = count_path_runtype_values(path, &mut path_top, &mut path_bot);
        if path_transitions == 0 {
            return check_inverse_on_empty_return(self, path, clip);
        }

        let mut clip_top = 0;
        let mut clip_bot = 0;
        let clip_transitions = clip.count_runtype_values(&mut clip_top, &mut clip_bot);

        let top = path_top.max(clip_top);
        let bot = path_bot.min(clip_bot);
        if top >= bot {
            return check_inverse_on_empty_return(self, path, clip);
        }

        let mut builder = SkRgnBuilder::new();
        if !builder.init(
            bot - top,
            path_transitions.max(clip_transitions),
            path.is_inverse_fill_type(),
        ) {
            // can't allocate working space, so return false
            return self.set_empty();
        }

        sk_scan_fill_path(&raw, clip, &mut builder);
        builder.done();

        let count = builder.compute_run_count();
        if count == 0 {
            self.set_empty()
        } else if count == crate::sk_region::K_RECT_REGION_RUNS {
            let mut bounds = IRect::make_empty();
            builder.copy_to_rect(&mut bounds);
            self.set_rect(&bounds)
        } else {
            self.set_runs_from_builder(builder.copy_to_rgn())
        }
    }

    /// `SkRegion::addBoundaryPath`.
    pub fn add_boundary_path(&self, builder: &mut SkPathBuilder) -> bool {
        if self.is_empty() {
            return false;
        }

        let bounds = *self.get_bounds();

        if self.is_rect() {
            builder.add_rect(&Rect::from(bounds), SkPathDirection::Cw, 0);
            return true;
        }

        #[derive(Clone, Debug)]
        struct Edge {
            x: i32,
            y0: i32,
            y1: i32,
            flags: u8,
            next: Option<usize>,
        }

        const K_Y0_LINK: u8 = 0x01;
        const K_Y1_LINK: u8 = 0x02;
        const K_COMPLETE_LINK: u8 = K_Y0_LINK | K_Y1_LINK;

        impl Edge {
            fn set(x: i32, y0: i32, y1: i32) -> Edge {
                debug_assert!(y0 != y1);
                Edge {
                    x,
                    y0,
                    y1,
                    flags: 0,
                    next: None,
                }
            }

            fn top(&self) -> i32 {
                self.y0.min(self.y1)
            }
        }

        let mut iter = RegionIterator::from_region(self);
        let mut edges: Vec<Edge> = Vec::new();
        while !iter.done() {
            let r = *iter.rect();
            edges.push(Edge::set(r.left, r.bottom, r.top));
            edges.push(Edge::set(r.right, r.top, r.bottom));
            iter.next();
        }

        let mut indices: Vec<usize> = (0..edges.len()).collect();
        indices.sort_by(|&a, &b| {
            if edges[a].x == edges[b].x {
                edges[a].top().cmp(&edges[b].top())
            } else {
                edges[a].x.cmp(&edges[b].x)
            }
        });
        let mut edges: Vec<Edge> = indices.into_iter().map(|i| edges[i].clone()).collect();
        let count = edges.len();
        let stop = count;

        fn find_link(edges: &mut [Edge], base: usize, stop: usize) {
            debug_assert!(base < stop);

            if edges[base].flags == K_COMPLETE_LINK {
                debug_assert!(edges[base].next.is_some());
                return;
            }

            debug_assert!(base + 1 < stop);

            let y0 = edges[base].y0;
            let y1 = edges[base].y1;

            if (edges[base].flags & K_Y0_LINK) == 0 {
                let mut e = base;
                loop {
                    e += 1;
                    if (edges[e].flags & K_Y1_LINK) == 0 && y0 == edges[e].y1 {
                        debug_assert!(edges[e].next.is_none());
                        edges[e].next = Some(base);
                        edges[e].flags |= K_Y1_LINK;
                        break;
                    }
                }
            }

            if (edges[base].flags & K_Y1_LINK) == 0 {
                let mut e = base;
                loop {
                    e += 1;
                    if (edges[e].flags & K_Y0_LINK) == 0 && y1 == edges[e].y0 {
                        debug_assert!(edges[base].next.is_none());
                        edges[base].next = Some(e);
                        edges[e].flags |= K_Y0_LINK;
                        break;
                    }
                }
            }

            edges[base].flags = K_COMPLETE_LINK;
        }

        fn extract_path(
            edges: &mut [Edge],
            mut edge: usize,
            stop: usize,
            builder: &mut SkPathBuilder,
        ) -> usize {
            while edges[edge].flags == 0 {
                edge += 1; // skip over "used" edges
                debug_assert!(edge <= stop);
            }
            debug_assert!(edge < stop);

            let base = edge;
            let mut prev = edge;
            edge = edges[edge].next.unwrap();
            debug_assert!(edge != base);

            let mut count = 1;
            builder.move_to_xy(edges[prev].x as Scalar, edges[prev].y0 as Scalar);
            edges[prev].flags = 0;
            loop {
                if edges[prev].x != edges[edge].x || edges[prev].y1 != edges[edge].y0 {
                    // skip collinear
                    builder.line_to_xy(edges[prev].x as Scalar, edges[prev].y1 as Scalar); // V
                    builder.line_to_xy(edges[edge].x as Scalar, edges[edge].y0 as Scalar); // H
                }
                prev = edge;
                edge = edges[edge].next.unwrap();
                count += 1;
                edges[prev].flags = 0;
                if edge == base {
                    break;
                }
            }
            builder.line_to_xy(edges[prev].x as Scalar, edges[prev].y1 as Scalar); // V
            builder.close();
            count
        }

        for e in 0..stop {
            find_link(&mut edges, e, stop);
        }

        builder.inc_reserve(count << 1, 0, 0);
        let mut remaining = count;
        loop {
            debug_assert!(remaining > 1);
            remaining -= extract_path(&mut edges, 0, stop, builder);
            if remaining == 0 {
                break;
            }
        }

        true
    }

    /// `SkRegion::getBoundaryPath`.
    pub fn get_boundary_path(&self) -> SkPath {
        let mut builder = SkPathBuilder::new();
        let _ = self.add_boundary_path(&mut builder);
        builder.detach(None)
    }
}
