//! Port of `src/core/SkEdgeBuilder.h` and the `SkBasicEdgeBuilder` portion of
//! `src/core/SkEdgeBuilder.cpp` (the analytic builder used by the M14 antialiasing pipeline is
//! intentionally not part of this milestone).

use crate::floating_point::is_finite;
use crate::point::Point;
use crate::rect::{IRect, Rect};
use crate::safe_math::SkSafeMath;
use crate::scalar::Scalar;
use crate::sk_edge::SkEdge;
use crate::sk_edge_clipper::SkEdgeClipper;
use crate::sk_geometry::{
    SkAutoConicToQuads, sk_chop_cubic_at_y_extrema, sk_chop_quad_at_y_extrema,
};
use crate::sk_line_clipper::{K_MAX_POINTS, clip_line};
use crate::sk_path::SkPath;
use crate::sk_path_edge_iter::{SkPathEdge, SkPathEdgeIter};
use crate::sk_path_priv::pts_in_iter;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_types::{K_LINE_SK_PATH_SEGMENT_MASK, SkPathVerb};

/// `SkEdgeBuilder::Combine`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combine {
    NoCombine,
    PartialCombine,
    TotalCombine,
}

/// `SkBasicEdgeBuilder`.
#[derive(Clone, Debug, Default)]
pub struct SkBasicEdgeBuilder {
    f_edges: Vec<SkEdge>,
}

fn is_vertical(edge: &SkEdge) -> bool {
    // We only consider edges that were originally lines to be vertical to avoid numerical issues.
    edge.f_dxdy() == 0 && edge.base().f_edge_type == crate::sk_edge::SkEdgeType::Line
}

impl SkBasicEdgeBuilder {
    pub fn new() -> SkBasicEdgeBuilder {
        SkBasicEdgeBuilder::default()
    }

    /// `SkBasicEdgeBuilder::edgeList`.
    #[inline]
    pub fn edge_list(&self) -> &[SkEdge] {
        &self.f_edges
    }

    #[inline]
    pub fn edge_list_mut(&mut self) -> &mut [SkEdge] {
        &mut self.f_edges
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.f_edges.len()
    }

    /// Consume the built edges (the reference's `edgeList()` ownership transfer).
    #[inline]
    pub fn take_edges(&mut self) -> Vec<SkEdge> {
        std::mem::take(&mut self.f_edges)
    }

    fn combine_vertical(edge: &SkEdge, last: &mut SkEdge) -> Combine {
        // We only consider edges that were originally lines to be vertical to avoid numerical
        // issues (crbug.com/1154864).
        if last.base().f_edge_type != crate::sk_edge::SkEdgeType::Line
            || last.f_dxdy() != 0
            || edge.f_x() != last.f_x()
        {
            return Combine::NoCombine;
        }
        let edge_base = edge.base();
        let last_base = last.base();
        if edge_base.f_winding == last_base.f_winding {
            if edge_base.f_last_y + 1 == last_base.f_first_y {
                last.base_mut().f_first_y = edge_base.f_first_y;
                return Combine::PartialCombine;
            }
            if edge_base.f_first_y == last_base.f_last_y + 1 {
                last.base_mut().f_last_y = edge_base.f_last_y;
                return Combine::PartialCombine;
            }
            return Combine::NoCombine;
        }
        if edge_base.f_first_y == last_base.f_first_y {
            if edge_base.f_last_y == last_base.f_last_y {
                return Combine::TotalCombine;
            }
            if edge_base.f_last_y < last_base.f_last_y {
                last.base_mut().f_first_y = edge_base.f_last_y + 1;
                return Combine::PartialCombine;
            }
            let last_last_y = last_base.f_last_y;
            let base = last.base_mut();
            base.f_first_y = last_last_y + 1;
            base.f_last_y = edge_base.f_last_y;
            base.f_winding = edge_base.f_winding;
            return Combine::PartialCombine;
        }
        if edge_base.f_last_y == last_base.f_last_y {
            if edge_base.f_first_y > last_base.f_first_y {
                last.base_mut().f_last_y = edge_base.f_first_y - 1;
                return Combine::PartialCombine;
            }
            let last_first_y = last_base.f_first_y;
            let base = last.base_mut();
            base.f_last_y = last_first_y - 1;
            base.f_first_y = edge_base.f_first_y;
            base.f_winding = edge_base.f_winding;
            return Combine::PartialCombine;
        }
        Combine::NoCombine
    }

    /// `SkBasicEdgeBuilder::addLine`.
    fn add_line(&mut self, pts: &[Point]) {
        if let Some(edge) = SkEdge::new_line(pts[0], pts[1], None) {
            let combine = if is_vertical(&edge) && !self.f_edges.is_empty() {
                let last = self.f_edges.last_mut().unwrap();
                Self::combine_vertical(&edge, last)
            } else {
                Combine::NoCombine
            };

            match combine {
                Combine::TotalCombine => {
                    self.f_edges.pop();
                }
                Combine::PartialCombine => {}
                Combine::NoCombine => self.f_edges.push(edge),
            }
        }
    }

    /// `SkBasicEdgeBuilder::addQuad`.
    fn add_quad(&mut self, pts: &[Point]) {
        if let Some(edge) = SkEdge::new_quad(pts) {
            self.f_edges.push(edge);
        }
    }

    /// `SkBasicEdgeBuilder::addCubic`.
    fn add_cubic(&mut self, pts: &[Point]) {
        if let Some(edge) = SkEdge::new_cubic(pts) {
            self.f_edges.push(edge);
        }
    }

    /// `SkBasicEdgeBuilder::recoverClip`.
    fn recover_clip(src: &IRect) -> Rect {
        Rect::from(*src)
    }

    /// `SkEdgeBuilder::buildPoly`.
    fn build_poly(
        &mut self,
        raw: &SkPathRaw,
        iclip: Option<&IRect>,
        can_cull_to_the_right: bool,
    ) -> i32 {
        let max_edge_count = raw.points().len();
        if let Some(_iclip) = iclip {
            // clipping can turn 1 line into (up to) kMaxClippedLineSegments
            let mut safe = SkSafeMath::new();
            let _ = safe.mul(
                max_edge_count,
                crate::sk_line_clipper::K_MAX_CLIPPED_LINE_SEGMENTS,
            );
            if !safe.ok() {
                return 0;
            }
        }

        let mut iter = SkPathEdgeIter::from_raw(raw);
        if let Some(iclip) = iclip {
            let clip = Self::recover_clip(iclip);
            loop {
                let e = iter.next();
                if e.is_done() {
                    break;
                }
                match e.f_edge {
                    SkPathEdge::Line => {
                        let mut lines = [Point::default(); K_MAX_POINTS];
                        let line_count = clip_line(
                            &[e.f_pts[0], e.f_pts[1]],
                            &clip,
                            &mut lines,
                            can_cull_to_the_right,
                        );
                        debug_assert!(
                            line_count as usize
                                <= crate::sk_line_clipper::K_MAX_CLIPPED_LINE_SEGMENTS
                        );
                        for i in 0..line_count as usize {
                            self.add_line(&lines[i..i + 2]);
                        }
                    }
                    _ => {
                        debug_assert!(false, "unexpected verb");
                    }
                }
            }
        } else {
            loop {
                let e = iter.next();
                if e.is_done() {
                    break;
                }
                match e.f_edge {
                    SkPathEdge::Line => self.add_line(&e.f_pts[..2]),
                    _ => {
                        debug_assert!(false, "unexpected verb");
                    }
                }
            }
        }
        self.f_edges.len() as i32
    }

    /// `SkEdgeBuilder::build`.
    fn build(
        &mut self,
        raw: &SkPathRaw,
        iclip: Option<&IRect>,
        can_cull_to_the_right: bool,
    ) -> i32 {
        if let Some(iclip) = iclip {
            let clip: Rect = Self::recover_clip(iclip);
            let mut is_finite_result = true;

            SkEdgeClipper::clip_path(raw, &clip, can_cull_to_the_right, |clipper, _| {
                let mut pts = [Point::new(0.0, 0.0); 4];
                while let Some(verb) = clipper.next(&mut pts) {
                    let count = pts_in_iter(verb) as usize;
                    if !pts[..count]
                        .iter()
                        .all(|p| is_finite(p.x) && is_finite(p.y))
                    {
                        is_finite_result = false;
                        return;
                    }
                    match verb {
                        SkPathVerb::Line => self.add_line(&pts[..2]),
                        SkPathVerb::Quad => self.add_quad(&pts[..3]),
                        SkPathVerb::Cubic => self.add_cubic(&pts[..4]),
                        _ => {}
                    }
                }
            });
            return if is_finite_result {
                self.f_edges.len() as i32
            } else {
                0
            };
        }

        let mut iter = SkPathEdgeIter::from_raw(raw);
        let mut quadder = SkAutoConicToQuads::new();
        const K_CONIC_TOL: Scalar = 0.25;
        let mut mono_y = [Point::new(0.0, 0.0); 10];
        let mut mono_x = [Point::new(0.0, 0.0); 5];

        loop {
            let e = iter.next();
            if e.is_done() {
                break;
            }
            match e.f_edge {
                SkPathEdge::Line => self.add_line(&e.f_pts[..2]),
                SkPathEdge::Quad => {
                    let n = sk_chop_quad_at_y_extrema(&e.f_pts[..3], &mut mono_x);
                    for i in 0..=n as usize {
                        self.add_quad(&mono_x[i * 2..i * 2 + 3]);
                    }
                }
                SkPathEdge::Conic => {
                    let quad_pts = quadder
                        .compute_quads_pts(&e.f_pts[..3], iter.conic_weight(), K_CONIC_TOL)
                        .to_vec();
                    let count = quadder.count_quads();
                    for i in 0..count as usize {
                        let base = i * 2;
                        let n = sk_chop_quad_at_y_extrema(&quad_pts[base..base + 3], &mut mono_x);
                        for j in 0..=n as usize {
                            self.add_quad(&mono_x[j * 2..j * 2 + 3]);
                        }
                    }
                }
                SkPathEdge::Cubic => {
                    let n = sk_chop_cubic_at_y_extrema(&e.f_pts[..4], &mut mono_y);
                    for i in 0..=n as usize {
                        self.add_cubic(&mono_y[i * 3..i * 3 + 4]);
                    }
                }
            }
        }
        self.f_edges.len() as i32
    }

    /// `SkEdgeBuilder::buildEdges(const SkPathRaw&, const SkIRect*)`.
    pub fn build_edges(&mut self, raw: &SkPathRaw, shifted_clip: Option<&IRect>) -> i32 {
        // If we're convex, then we need both edges, even if the right edge is past the clip.
        let can_cull_to_the_right = !raw.is_known_to_be_convex();

        // We can use our buildPoly() optimization if all the segments are lines.
        let count = if raw.segment_masks() == K_LINE_SK_PATH_SEGMENT_MASK as u32 {
            self.build_poly(raw, shifted_clip, can_cull_to_the_right)
        } else {
            self.build(raw, shifted_clip, can_cull_to_the_right)
        };

        debug_assert!(can_cull_to_the_right || count != 1);
        count
    }

    /// `SkEdgeBuilder::buildEdges(const SkPath&, const SkIRect*)`.
    pub fn build_edges_path(&mut self, path: &SkPath, shifted_clip: Option<&IRect>) -> i32 {
        match path.raw(crate::sk_path_types::SkResolveConvexity::Yes) {
            Some(raw) => self.build_edges(&raw, shifted_clip),
            None => 0,
        }
    }
}
