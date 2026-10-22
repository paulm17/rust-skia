//! Port of `SkPathEdgeIter` from `src/core/SkPathPriv.h`.
//!
//! Lightweight variant of `SkPath::Iter` that only returns segments (lines/quads/
//! conics/cubics). It does not return `kMove`/`kClose`, and always auto-closes each
//! contour. It is roughly `SkPath::Iter(path, true)` without the moves/closes.
//!
//! ADAPTATION: the C++ iterator exposes raw pointers into the path storage (and into a
//! small scratch buffer for auto-close lines). The Rust port instead yields an owned
//! fixed-size array plus its valid length, which preserves the observable sequence.

use crate::point::Point;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_types::SkPathVerb;

/// Represents the segment kinds that can be produced. Values match the low verb values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathEdge {
    Line,
    Quad,
    Conic,
    Cubic,
}

impl SkPathEdge {
    pub fn to_verb(self) -> SkPathVerb {
        match self {
            SkPathEdge::Line => SkPathVerb::Line,
            SkPathEdge::Quad => SkPathVerb::Quad,
            SkPathEdge::Conic => SkPathVerb::Conic,
            SkPathEdge::Cubic => SkPathVerb::Cubic,
        }
    }
}

/// Result of [`SkPathEdgeIter::next`].
#[derive(Clone, Copy, Debug)]
pub struct SkPathEdgeIterResult {
    pub f_pts: [Point; 4],
    pub f_pts_len: usize,
    pub f_edge: SkPathEdge,
    pub f_is_new_contour: bool,
}

impl SkPathEdgeIterResult {
    #[inline]
    pub fn is_done(&self) -> bool {
        self.f_pts_len == 0
    }

    #[inline]
    pub fn pts(&self) -> &[Point] {
        &self.f_pts[..self.f_pts_len]
    }
}

/// Edge iterator over a raw path.
pub struct SkPathEdgeIter<'a> {
    f_verbs: &'a [SkPathVerb],
    f_verb_index: usize,
    f_pts: &'a [Point],
    f_pts_index: usize,
    f_move_to: Point,
    f_conic_weights: &'a [f32],
    f_conic_index: usize,
    f_needs_close_line: bool,
    f_next_is_new_contour: bool,
}

impl<'a> SkPathEdgeIter<'a> {
    pub fn from_raw(raw: &'a SkPathRaw) -> Self {
        SkPathEdgeIter {
            f_verbs: &raw.f_verbs,
            f_verb_index: 0,
            f_pts: &raw.f_points,
            f_pts_index: 0,
            f_move_to: Point::new(0.0, 0.0),
            f_conic_weights: &raw.f_conics,
            f_conic_index: 0,
            f_needs_close_line: false,
            f_next_is_new_contour: false,
        }
    }

    #[inline]
    pub fn conic_weight(&self) -> f32 {
        self.f_conic_weights[self.f_conic_index - 1]
    }

    fn close_line(&mut self) -> SkPathEdgeIterResult {
        let mut pts = [Point::new(0.0, 0.0); 4];
        pts[0] = self.f_pts[self.f_pts_index - 1];
        pts[1] = self.f_move_to;
        self.f_needs_close_line = false;
        self.f_next_is_new_contour = true;
        SkPathEdgeIterResult {
            f_pts: pts,
            f_pts_len: 2,
            f_edge: SkPathEdge::Line,
            f_is_new_contour: false,
        }
    }

    pub fn next(&mut self) -> SkPathEdgeIterResult {
        loop {
            if self.f_verb_index >= self.f_verbs.len() {
                return if self.f_needs_close_line {
                    self.close_line()
                } else {
                    SkPathEdgeIterResult {
                        f_pts: [Point::new(0.0, 0.0); 4],
                        f_pts_len: 0,
                        f_edge: SkPathEdge::Line,
                        f_is_new_contour: false,
                    }
                };
            }

            let verb = self.f_verbs[self.f_verb_index];
            self.f_verb_index += 1;
            match verb {
                SkPathVerb::Move => {
                    if self.f_needs_close_line {
                        let res = self.close_line();
                        self.f_move_to = self.f_pts[self.f_pts_index];
                        self.f_pts_index += 1;
                        return res;
                    }
                    self.f_move_to = self.f_pts[self.f_pts_index];
                    self.f_pts_index += 1;
                    self.f_next_is_new_contour = true;
                }
                SkPathVerb::Close => {
                    if self.f_needs_close_line {
                        return self.close_line();
                    }
                }
                _ => {
                    let v = verb as u32;
                    let pts_count = ((v + 2) / 2) as usize;
                    let cws_count = ((v & (v - 1)) / 2) as usize;

                    self.f_needs_close_line = true;
                    self.f_pts_index += pts_count;
                    self.f_conic_index += cws_count;

                    let is_new_contour = self.f_next_is_new_contour;
                    self.f_next_is_new_contour = false;

                    let start = self.f_pts_index - (pts_count + 1);
                    let mut pts = [Point::new(0.0, 0.0); 4];
                    let n = pts_count + 1;
                    pts[..n].copy_from_slice(&self.f_pts[start..start + n]);
                    let edge = match verb {
                        SkPathVerb::Line => SkPathEdge::Line,
                        SkPathVerb::Quad => SkPathEdge::Quad,
                        SkPathVerb::Conic => SkPathEdge::Conic,
                        SkPathVerb::Cubic => SkPathEdge::Cubic,
                        _ => unreachable!(),
                    };
                    return SkPathEdgeIterResult {
                        f_pts: pts,
                        f_pts_len: n,
                        f_edge: edge,
                        f_is_new_contour: is_new_contour,
                    };
                }
            }
        }
    }
}
