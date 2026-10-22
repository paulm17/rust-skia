//! Port of `include/core/SkPathIter.h` and `src/core/SkPathIter.cpp`.

use crate::point::Point;
use crate::sk_path_types::SkPathVerb;

/// `SkPathIter::Rec`.
pub struct SkPathIterRec<'a> {
    pub f_points: &'a [Point],
    pub f_conic_weight: f32,
    pub f_verb: SkPathVerb,
}

impl SkPathIterRec<'_> {
    #[inline]
    pub fn conic_weight(&self) -> f32 {
        debug_assert!(self.f_verb == SkPathVerb::Conic);
        self.f_conic_weight
    }
}

/// `SkPathIter` from `include/core/SkPathIter.h`.
pub struct SkPathIter<'p> {
    p_index: usize,
    v_index: usize,
    c_index: usize,
    f_points: &'p [Point],
    f_verbs: &'p [SkPathVerb],
    f_conics: &'p [f32],
    f_close_point_storage: [Point; 2],
}

impl<'p> SkPathIter<'p> {
    pub fn new(pts: &'p [Point], vbs: &'p [SkPathVerb], cns: &'p [f32]) -> Self {
        // For compat older iterators, we trim off a trailing Move.
        let verbs = if let Some((&SkPathVerb::Move, rest)) = vbs.split_last() {
            rest
        } else {
            vbs
        };
        SkPathIter {
            p_index: 0,
            v_index: 0,
            c_index: 0,
            f_points: pts,
            f_verbs: verbs,
            f_conics: cns,
            f_close_point_storage: [Point::new(0.0, 0.0); 2],
        }
    }

    pub fn next(&mut self) -> Option<SkPathIterRec<'_>> {
        if self.v_index >= self.f_verbs.len() {
            return None;
        }

        let n: usize;

        let mut w = -1.0f32;
        let v = self.f_verbs[self.v_index];
        self.v_index += 1;
        match v {
            SkPathVerb::Move => {
                self.f_close_point_storage[1] = self.f_points[self.p_index];
                self.p_index += 1;
                return Some(SkPathIterRec {
                    f_points: &self.f_close_point_storage[1..2],
                    f_conic_weight: w,
                    f_verb: v,
                });
            }
            SkPathVerb::Line => n = 1,
            SkPathVerb::Quad => n = 2,
            SkPathVerb::Conic => {
                n = 2;
                w = self.f_conics[self.c_index];
                self.c_index += 1;
            }
            SkPathVerb::Cubic => n = 3,
            SkPathVerb::Close => {
                debug_assert!(self.p_index > 0);
                self.f_close_point_storage[0] = self.f_points[self.p_index - 1];
                return Some(SkPathIterRec {
                    f_points: &self.f_close_point_storage[..2],
                    f_conic_weight: w,
                    f_verb: v,
                });
            }
            SkPathVerb::Done => {
                // SkPathIter only sees storage verbs; treat Done as terminal.
                return None;
            }
        }

        debug_assert!(self.p_index > 0);
        let start = self.p_index - 1;
        debug_assert!((1..=3).contains(&n));
        self.p_index += n;
        Some(SkPathIterRec {
            f_points: &self.f_points[start..start + n + 1],
            f_conic_weight: w,
            f_verb: v,
        })
    }

    pub fn peek_next_verb(&self) -> Option<SkPathVerb> {
        if self.v_index < self.f_verbs.len() {
            Some(self.f_verbs[self.v_index])
        } else {
            None
        }
    }
}

/// `SkPathContourIter::Rec`.
pub struct SkPathContourIterRec<'a> {
    pub f_points: &'a [Point],
    pub f_verbs: &'a [SkPathVerb],
    pub f_conics: &'a [f32],
}

/// `SkPathContourIter` from `include/core/SkPathIter.h`.
pub struct SkPathContourIter<'p> {
    f_points: &'p [Point],
    f_verbs: &'p [SkPathVerb],
    f_conics: &'p [f32],
}

impl<'p> SkPathContourIter<'p> {
    pub fn new(pts: &'p [Point], vbs: &'p [SkPathVerb], cns: &'p [f32]) -> Self {
        SkPathContourIter {
            f_points: pts,
            f_verbs: vbs,
            f_conics: cns,
        }
    }

    pub fn next(&mut self) -> Option<SkPathContourIterRec<'_>> {
        if self.f_verbs.is_empty() {
            return None;
        }

        debug_assert!(self.f_verbs[0] == SkPathVerb::Move);
        let mut npts = 1usize;
        let mut nvbs = 1usize;
        let mut nws = 0usize;

        'outer: {
            for i in 1..self.f_verbs.len() {
                match self.f_verbs[i] {
                    SkPathVerb::Move => break 'outer,
                    SkPathVerb::Line => npts += 1,
                    SkPathVerb::Quad => npts += 2,
                    SkPathVerb::Conic => {
                        npts += 2;
                        nws += 1;
                    }
                    SkPathVerb::Cubic => npts += 3,
                    SkPathVerb::Close => {
                        nvbs += 1;
                        break 'outer;
                    }
                    SkPathVerb::Done => break 'outer,
                }
                nvbs += 1;
            }
        }

        let rec = SkPathContourIterRec {
            f_points: &self.f_points[..npts],
            f_verbs: &self.f_verbs[..nvbs],
            f_conics: &self.f_conics[..nws],
        };
        self.f_points = &self.f_points[npts..];
        self.f_verbs = &self.f_verbs[nvbs..];
        self.f_conics = &self.f_conics[nws..];
        Some(rec)
    }
}
