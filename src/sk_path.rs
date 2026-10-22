//! Port of `include/core/SkPath.h` and `src/core/SkPath.cpp` (plus serialization, interpolation
//! and dump). `SkPathData` is reference counted via `std::sync::Arc` (see `sk_path_data.rs`).

#![allow(
    clippy::too_many_arguments,
    non_camel_case_types,
    clippy::manual_range_contains,
    clippy::needless_return,
    clippy::let_and_return,
    clippy::single_match,
    clippy::redundant_clone,
    clippy::unnecessary_cast,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::excessive_precision
)]

use std::sync::Arc;

use crate::data::SkData;
use crate::floating_point::is_nan;
use crate::matrix::Matrix;
use crate::point::Point;
use crate::point_priv::PointPriv;
use crate::rect::Rect;
use crate::rrect::RRect;
use crate::scalar::Scalar;
use crate::sk_geometry::SkConic;
use crate::sk_path_data::{SkPathData, SkPathOvalInfo, SkPathRRectInfo};
use crate::sk_path_iter::{SkPathIter, SkPathIterRec};
use crate::sk_path_priv as priv_;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_types::{
    K_CONIC_SK_PATH_SEGMENT_MASK, K_CUBIC_SK_PATH_SEGMENT_MASK, K_LINE_SK_PATH_SEGMENT_MASK,
    K_QUAD_SK_PATH_SEGMENT_MASK, SkPathConvexity, SkPathDirection, SkPathFillType, SkPathVerb,
    SkResolveConvexity, sk_path_convexity_is_convex,
};
use crate::stream::SkWStream;
use crate::string::{SkScalarAsStringType, SkString, sk_append_scalar};

/// `SkPath::Verb`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Verb {
    kMove_Verb = 0,
    kLine_Verb = 1,
    kQuad_Verb = 2,
    kConic_Verb = 3,
    kCubic_Verb = 4,
    kClose_Verb = 5,
    kDone_Verb = 6,
}

impl Verb {
    fn from_verb(v: SkPathVerb) -> Verb {
        match v {
            SkPathVerb::Move => Verb::kMove_Verb,
            SkPathVerb::Line => Verb::kLine_Verb,
            SkPathVerb::Quad => Verb::kQuad_Verb,
            SkPathVerb::Conic => Verb::kConic_Verb,
            SkPathVerb::Cubic => Verb::kCubic_Verb,
            SkPathVerb::Close => Verb::kClose_Verb,
            SkPathVerb::Done => Verb::kDone_Verb,
        }
    }
}

/// `SkPath::AddPathMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AddPathMode {
    #[default]
    Append,
    Extend,
}

/// `SkPath::SegmentMask`.
pub struct SegmentMask;

impl SegmentMask {
    pub const LINE: u32 = K_LINE_SK_PATH_SEGMENT_MASK as u32;
    pub const QUAD: u32 = K_QUAD_SK_PATH_SEGMENT_MASK as u32;
    pub const CONIC: u32 = K_CONIC_SK_PATH_SEGMENT_MASK as u32;
    pub const CUBIC: u32 = K_CUBIC_SK_PATH_SEGMENT_MASK as u32;
}

/// Port of `class SkPath`.
#[derive(Clone)]
pub struct SkPath {
    f_path_data: Arc<SkPathData>,
    f_fill_type: SkPathFillType,
    f_is_volatile: bool,
}

impl PartialEq for SkPath {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
            || (self.f_fill_type == other.f_fill_type && *self.f_path_data == *other.f_path_data)
    }
}
impl Eq for SkPath {}

const G_PTS_PER_VERB: [usize; 7] = [1, 1, 2, 2, 3, 0, 0];

impl SkPath {
    // ---- constructors / factories ----

    fn from_parts(pd: Arc<SkPathData>, ft: SkPathFillType, is_volatile: bool) -> SkPath {
        SkPath {
            f_path_data: pd,
            f_fill_type: ft,
            f_is_volatile: is_volatile,
        }
    }

    pub fn new() -> SkPath {
        SkPath::new_with_fill_type(SkPathFillType::Winding)
    }

    pub fn new_with_fill_type(ft: SkPathFillType) -> SkPath {
        SkPath {
            f_path_data: SkPathData::empty(),
            f_fill_type: ft,
            f_is_volatile: false,
        }
    }

    pub fn make_null_check(
        pd: Option<Arc<SkPathData>>,
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> SkPath {
        let pd = pd.unwrap_or_else(SkPathData::peek_error_singleton);
        SkPath::from_parts(pd, ft, is_volatile)
    }

    pub fn from_data(pd: Arc<SkPathData>, ft: SkPathFillType, is_volatile: bool) -> SkPath {
        SkPath::from_parts(pd, ft, is_volatile)
    }

    #[allow(non_snake_case)]
    pub fn Raw(
        pts: &[Point],
        vbs: &[SkPathVerb],
        ws: &[f32],
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> SkPath {
        SkPath::make_null_check(SkPathData::make_checked(pts, vbs, ws), ft, is_volatile)
    }

    pub fn make(
        pts: &[Point],
        vbs: &[u8],
        ws: &[f32],
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> SkPath {
        let verbs: Vec<SkPathVerb> = vbs.iter().map(|&b| u8_to_verb(b)).collect();
        SkPath::Raw(pts, &verbs, ws, ft, is_volatile)
    }

    pub fn rect(r: &Rect, ft: SkPathFillType, dir: SkPathDirection, start_index: u32) -> SkPath {
        let start_index = start_index & 3;
        SkPath::make_null_check(SkPathData::rect(r, dir, start_index), ft, false)
    }

    pub fn rect_dir(r: &Rect, dir: SkPathDirection, start_index: u32) -> SkPath {
        SkPath::rect(r, SkPathFillType::Winding, dir, start_index)
    }

    pub fn rect_default(r: &Rect) -> SkPath {
        SkPath::rect(r, SkPathFillType::Winding, SkPathDirection::Cw, 0)
    }

    pub fn oval(r: &Rect, dir: SkPathDirection, start_index: u32) -> SkPath {
        let start_index = start_index & 3;
        SkPath::make_null_check(
            SkPathData::oval(r, dir, start_index),
            SkPathFillType::Winding,
            false,
        )
    }

    pub fn oval_dir(r: &Rect, dir: SkPathDirection) -> SkPath {
        SkPath::oval(r, dir, 1)
    }

    pub fn oval_default(r: &Rect) -> SkPath {
        SkPath::oval(r, SkPathDirection::Cw, 1)
    }

    pub fn circle(cx: Scalar, cy: Scalar, radius: Scalar, dir: SkPathDirection) -> SkPath {
        if radius >= 0.0 {
            SkPath::oval_dir(
                &Rect::new(cx - radius, cy - radius, cx + radius, cy + radius),
                dir,
            )
        } else {
            SkPath::new()
        }
    }

    pub fn rrect(rr: &RRect, dir: SkPathDirection, start_index: u32) -> SkPath {
        let start_index = start_index & 7;
        let bounds = *rr.get_bounds();
        match priv_::simplify_rrect(rr, start_index).0 {
            priv_::RRectAsEnum::Rect => SkPath::rect(
                &bounds,
                SkPathFillType::Winding,
                dir,
                priv_::simplify_rrect(rr, start_index).1,
            ),
            priv_::RRectAsEnum::Oval => {
                SkPath::oval(&bounds, dir, priv_::simplify_rrect(rr, start_index).1)
            }
            priv_::RRectAsEnum::RRect => SkPath::make_null_check(
                SkPathData::rrect(rr, dir, priv_::simplify_rrect(rr, start_index).1),
                SkPathFillType::Winding,
                false,
            ),
        }
    }

    pub fn rrect_dirl(rr: &RRect, dir: SkPathDirection) -> SkPath {
        let start = if dir == SkPathDirection::Cw { 6 } else { 7 };
        SkPath::rrect(rr, dir, start)
    }

    pub fn rrect_default(rr: &RRect) -> SkPath {
        SkPath::rrect(rr, SkPathDirection::Cw, 6)
    }

    pub fn rrect_xy(bounds: &Rect, rx: Scalar, ry: Scalar, dir: SkPathDirection) -> SkPath {
        SkPath::rrect(
            &RRect::make_rect_xy(bounds, rx, ry),
            dir,
            if dir == SkPathDirection::Cw { 6 } else { 7 },
        )
    }

    pub fn polygon(
        pts: &[Point],
        is_closed: bool,
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> SkPath {
        SkPath::make_null_check(Some(SkPathData::polygon(pts, is_closed)), ft, is_volatile)
    }

    pub fn line(a: Point, b: Point) -> SkPath {
        SkPath::polygon(&[a, b], false, SkPathFillType::Winding, false)
    }

    // ---- basic state ----

    pub fn reset(&mut self) -> &mut SkPath {
        *self = SkPath::new();
        self
    }

    pub fn swap(&mut self, other: &mut SkPath) {
        if !std::ptr::eq(self, other) {
            std::mem::swap(&mut self.f_path_data, &mut other.f_path_data);
            std::mem::swap(&mut self.f_fill_type, &mut other.f_fill_type);
            std::mem::swap(&mut self.f_is_volatile, &mut other.f_is_volatile);
        }
    }

    pub fn snapshot(&self) -> SkPath {
        self.clone()
    }

    pub fn set_fill_type(&mut self, ft: SkPathFillType) {
        self.f_fill_type = ft;
    }

    pub fn get_fill_type(&self) -> SkPathFillType {
        self.f_fill_type
    }

    pub fn is_inverse_fill_type(&self) -> bool {
        self.f_fill_type.is_inverse()
    }

    pub fn toggle_inverse_fill_type(&mut self) {
        self.f_fill_type = self.f_fill_type.toggle_inverse();
    }

    pub fn make_fill_type(&self, ft: SkPathFillType) -> SkPath {
        let mut copy = self.clone();
        copy.set_fill_type(ft);
        copy
    }

    pub fn make_toggle_inverse_fill_type(&self) -> SkPath {
        self.make_fill_type(self.f_fill_type.toggle_inverse())
    }

    pub fn make_is_volatile(&self, v: bool) -> SkPath {
        let mut copy = self.clone();
        copy.f_is_volatile = v;
        copy
    }

    pub fn is_volatile(&self) -> bool {
        self.f_is_volatile
    }

    pub fn set_is_volatile(&mut self, v: bool) -> &mut SkPath {
        self.f_is_volatile = v;
        self
    }

    // ---- accessors ----

    pub fn points(&self) -> &[Point] {
        self.f_path_data.points()
    }

    pub fn verbs(&self) -> &[SkPathVerb] {
        self.f_path_data.verbs()
    }

    pub fn conic_weights(&self) -> &[f32] {
        self.f_path_data.conics()
    }

    pub fn count_points(&self) -> i32 {
        self.points().len() as i32
    }

    pub fn count_verbs(&self) -> i32 {
        self.verbs().len() as i32
    }

    pub fn get_last_pt(&self) -> Option<Point> {
        self.points().last().copied()
    }

    pub fn get_point(&self, index: i32) -> Point {
        if index >= 0 && (index as usize) < self.points().len() {
            self.points()[index as usize]
        } else {
            Point::new(0.0, 0.0)
        }
    }

    /// `SkPath::getPoints(SkPoint pts[], int max)` adaptation: returns the number of points written.
    pub fn get_points(&self, dst: &mut [Point]) -> usize {
        let src = self.points();
        let n = dst.len().min(src.len());
        dst[..n].copy_from_slice(&src[..n]);
        n
    }

    pub fn get_verbs(&self, dst: &mut [u8]) -> usize {
        let src = self.verbs();
        let n = dst.len().min(src.len());
        for i in 0..n {
            dst[i] = src[i] as u8;
        }
        src.len()
    }

    pub fn approximate_bytes_used(&self) -> usize {
        std::mem::size_of::<SkPath>()
            + std::mem::size_of_val(self.points())
            + self.verbs().len()
            + std::mem::size_of_val(self.conic_weights())
    }

    pub fn get_bounds(&self) -> &Rect {
        self.f_path_data.bounds()
    }

    pub fn compute_tight_bounds(&self) -> Rect {
        if self.get_segment_masks() == SegmentMask::LINE {
            return *self.get_bounds();
        }
        priv_::compute_tight_bounds(self.points(), self.verbs(), self.conic_weights())
    }

    pub fn get_segment_masks(&self) -> u32 {
        self.f_path_data.segment_mask() as u32
    }

    pub fn get_generation_id(&self) -> u64 {
        self.f_path_data.unique_id()
    }

    pub fn is_valid(&self) -> bool {
        self.is_finite()
    }

    pub fn is_empty(&self) -> bool {
        self.verbs().is_empty()
    }

    pub fn is_last_contour_closed(&self) -> bool {
        let verbs = self.verbs();
        !verbs.is_empty() && verbs[verbs.len() - 1] == SkPathVerb::Close
    }

    pub fn is_finite(&self) -> bool {
        !Arc::ptr_eq(&self.f_path_data, &SkPathData::peek_error_singleton())
    }

    // ---- geometry queries ----

    pub fn is_convex(&self) -> bool {
        sk_path_convexity_is_convex(self.get_convexity())
    }

    pub fn is_line(&self, line: Option<&mut [Point; 2]>) -> bool {
        let verbs = self.verbs();
        if verbs.len() == 2 && verbs[1] == SkPathVerb::Line {
            let pts = self.points();
            if let Some(line) = line {
                line[0] = pts[0];
                line[1] = pts[1];
            }
            return true;
        }
        false
    }

    pub fn is_rect(
        &self,
        rect: Option<&mut Rect>,
        is_closed: Option<&mut bool>,
        direction: Option<&mut SkPathDirection>,
    ) -> bool {
        if let Some(rc) =
            priv_::is_rect_contour(self.points(), self.verbs(), self.get_segment_masks(), false)
        {
            if let Some(rect) = rect {
                *rect = rc.f_rect;
            }
            if let Some(is_closed) = is_closed {
                *is_closed = rc.f_is_closed;
            }
            if let Some(direction) = direction {
                *direction = rc.f_direction;
            }
            return true;
        }
        false
    }

    pub fn is_oval(&self, bounds: Option<&mut Rect>) -> bool {
        if let Some(info) = self.f_path_data.as_oval() {
            if let Some(bounds) = bounds {
                *bounds = info.f_bounds;
            }
            return true;
        }
        false
    }

    pub fn is_rrect(&self, rrect: Option<&mut RRect>) -> bool {
        if let Some(info) = self.f_path_data.as_rrect() {
            if let Some(rrect) = rrect {
                *rrect = info.f_rrect;
            }
            return true;
        }
        false
    }

    pub fn oval_info(&self) -> Option<SkPathOvalInfo> {
        self.f_path_data.as_oval()
    }

    pub fn rrect_info(&self) -> Option<SkPathRRectInfo> {
        self.f_path_data.as_rrect()
    }

    pub fn contains(&self, point: Point) -> bool {
        match self.raw(SkResolveConvexity::No) {
            Some(raw) => priv_::contains(&raw, point),
            None => false,
        }
    }

    pub fn contains_xy(&self, x: Scalar, y: Scalar) -> bool {
        self.contains(Point::new(x, y))
    }

    pub fn conservatively_contains_rect(&self, rect: &Rect) -> bool {
        let convexity = self.get_convexity();
        if !sk_path_convexity_is_convex(convexity) {
            return false;
        }
        let direction = match convexity.to_direction() {
            Some(d) => d,
            None => return false,
        };

        let mut first_pt = Point::new(0.0, 0.0);
        let mut prev_pt = Point::new(0.0, 0.0);
        let mut segment_count = 0i32;

        let mut iter = SkPathIter::new(self.points(), self.verbs(), self.conic_weights());
        while let Some(rec) = iter.next() {
            let verb = rec.f_verb;
            let pts = rec.f_points;
            if verb == SkPathVerb::Close || (segment_count > 0 && verb == SkPathVerb::Move) {
                segment_count += 1;
                break;
            } else if verb == SkPathVerb::Move {
                first_pt = pts[0];
                prev_pt = pts[0];
            } else {
                let point_count = priv_::pts_in_verb(verb) as usize;
                if point_count == 0 {
                    continue;
                }
                let count = (point_count + 1).min(pts.len());
                if !all_points_eq(&pts[..count]) {
                    let next_pt = point_count;
                    if next_pt >= pts.len() {
                        continue;
                    }
                    segment_count += 1;
                    if prev_pt == pts[next_pt] {
                        return false;
                    } else if verb == SkPathVerb::Conic {
                        let mut orig = SkConic::default();
                        orig.set(pts, rec.f_conic_weight);
                        let mut quad_pts = [Point::new(0.0, 0.0); 5];
                        let quad_count = orig.chop_into_quads_pow2(&mut quad_pts, 1);
                        if quad_count != 2 {
                            return false;
                        }
                        if !check_edge_against_rect(quad_pts[0], quad_pts[2], rect, direction) {
                            return false;
                        }
                        if !check_edge_against_rect(quad_pts[2], quad_pts[4], rect, direction) {
                            return false;
                        }
                    } else if !check_edge_against_rect(prev_pt, pts[next_pt], rect, direction) {
                        return false;
                    }
                    prev_pt = pts[next_pt];
                }
            }
        }

        if segment_count != 0 {
            return check_edge_against_rect(prev_pt, first_pt, rect, direction);
        }
        false
    }

    fn compute_convexity(&self) -> SkPathConvexity {
        let c = self.get_convexity_or_unknown();
        if c != SkPathConvexity::Unknown {
            return c;
        }
        let mut convexity = SkPathConvexity::Concave;
        if self.is_finite() {
            convexity = priv_::compute_convexity(self.points(), self.verbs(), self.conic_weights());
        }
        self.set_convexity(convexity);
        convexity
    }

    pub fn get_convexity(&self) -> SkPathConvexity {
        let mut convexity = self.get_convexity_or_unknown();
        if convexity == SkPathConvexity::Unknown {
            convexity = self.compute_convexity();
        }
        convexity
    }

    pub fn get_convexity_or_unknown(&self) -> SkPathConvexity {
        self.f_path_data.get_convexity_or_unknown()
    }

    pub fn set_convexity(&self, c: SkPathConvexity) {
        self.f_path_data.set_convexity(c);
    }

    // ---- degeneracy helpers ----

    pub fn is_line_degenerate(p1: &Point, p2: &Point, exact: bool) -> bool {
        if exact {
            p1 == p2
        } else {
            PointPriv::equals_within_tolerance(p1, p2)
        }
    }

    pub fn is_quad_degenerate(p1: &Point, p2: &Point, p3: &Point, exact: bool) -> bool {
        if exact {
            p1 == p2 && p2 == p3
        } else {
            PointPriv::equals_within_tolerance(p1, p2) && PointPriv::equals_within_tolerance(p2, p3)
        }
    }

    pub fn is_cubic_degenerate(
        p1: &Point,
        p2: &Point,
        p3: &Point,
        p4: &Point,
        exact: bool,
    ) -> bool {
        if exact {
            p1 == p2 && p2 == p3 && p3 == p4
        } else {
            PointPriv::equals_within_tolerance(p1, p2)
                && PointPriv::equals_within_tolerance(p2, p3)
                && PointPriv::equals_within_tolerance(p3, p4)
        }
    }

    pub fn convert_conic_to_quads(
        p0: &Point,
        p1: &Point,
        p2: &Point,
        w: Scalar,
        pts: &mut [Point],
        pow2: i32,
    ) -> i32 {
        let conic = SkConic::new(*p0, *p1, *p2, w);
        conic.chop_into_quads_pow2(pts, pow2)
    }

    // ---- transforms ----

    pub fn try_make_transform(&self, matrix: &Matrix) -> Option<SkPath> {
        self.f_path_data
            .make_transform_from(matrix)
            .map(|pdata| SkPath::from_parts(pdata, self.f_fill_type, self.f_is_volatile))
    }

    pub fn try_make_offset(&self, dx: Scalar, dy: Scalar) -> Option<SkPath> {
        self.try_make_transform(&Matrix::translate(dx, dy))
    }

    pub fn try_make_scale(&self, sx: Scalar, sy: Scalar) -> Option<SkPath> {
        self.try_make_transform(&Matrix::scale(sx, sy))
    }

    pub fn make_transform(&self, matrix: &Matrix) -> SkPath {
        if !self.is_finite() {
            return self.clone();
        }
        match self.try_make_transform(matrix) {
            Some(newpath) => newpath,
            None => SkPath::from_parts(SkPathData::peek_error_singleton(), self.f_fill_type, false),
        }
    }

    pub fn make_offset(&self, dx: Scalar, dy: Scalar) -> SkPath {
        self.make_transform(&Matrix::translate(dx, dy))
    }

    pub fn make_scale(&self, sx: Scalar, sy: Scalar) -> SkPath {
        self.make_transform(&Matrix::scale(sx, sy))
    }

    // ---- interpolation ----

    pub fn is_interpolatable(&self, compare: &SkPath) -> bool {
        can_interpolate(self, compare)
    }

    pub fn make_interpolate(&self, ending: &SkPath, weight: Scalar) -> SkPath {
        interpolate(self, ending, 1.0 - weight).unwrap_or_default()
    }

    pub fn interpolate(&self, ending: &SkPath, weight: Scalar, out: &mut SkPath) -> bool {
        match interpolate(self, ending, 1.0 - weight) {
            Some(result) => {
                *out = result;
                true
            }
            None => false,
        }
    }

    // ---- raw ----

    pub fn raw(&self, rc: SkResolveConvexity) -> Option<SkPathRaw> {
        Some(self.f_path_data.raw(self.f_fill_type, rc))
    }

    pub fn iter(&self) -> SkPathIter<'_> {
        SkPathIter::new(self.points(), self.verbs(), self.conic_weights())
    }

    // ---- serialization ----

    fn write_to_memory_impl(&self) -> Vec<u8> {
        if let Some(bytes) = self.write_to_memory_as_rrect() {
            return bytes;
        }
        let pts = self.points();
        let conics = self.conic_weights();
        let verbs = self.verbs();

        let packed: u32 = ((self.f_fill_type as u32) << 8) | (0u32 << 28) | 5u32;
        let raw_size = 16 + pts.len() * 8 + conics.len() * 4 + verbs.len();
        let size = (raw_size + 3) & !3;

        let mut out: Vec<u8> = Vec::with_capacity(size);
        push_u32(&mut out, packed);
        push_u32(&mut out, pts.len() as u32);
        push_u32(&mut out, conics.len() as u32);
        push_u32(&mut out, verbs.len() as u32);
        for p in pts {
            push_f32(&mut out, p.x);
            push_f32(&mut out, p.y);
        }
        for c in conics {
            push_f32(&mut out, *c);
        }
        for v in verbs {
            out.push(*v as u8);
        }
        while out.len() % 4 != 0 {
            out.push(0);
        }
        out
    }

    fn write_to_memory_as_rrect(&self) -> Option<Vec<u8>> {
        let (rrect, first_dir, start) = if let Some(oinfo) = self.f_path_data.as_oval() {
            let mut rr = RRect::new();
            rr.set_oval(&oinfo.f_bounds);
            (rr, oinfo.f_direction, oinfo.f_start_index as u32 * 2)
        } else if let Some(rinfo) = self.f_path_data.as_rrect() {
            (rinfo.f_rrect, rinfo.f_direction, rinfo.f_start_index as u32)
        } else {
            return None;
        };

        let packed: u32 =
            ((self.f_fill_type as u32) << 8) | ((first_dir as u32) << 26) | (1u32 << 28) | 5u32;

        let mut out: Vec<u8> = Vec::new();
        push_u32(&mut out, packed);
        let mut storage = [0u8; RRect::K_SIZE_IN_MEMORY];
        rrect.write_to_memory(&mut storage);
        out.extend_from_slice(&storage);
        push_u32(&mut out, start as u32);
        while out.len() % 4 != 0 {
            out.push(0);
        }
        Some(out)
    }

    pub fn write_to_memory(&self, storage: Option<&mut [u8]>) -> usize {
        let bytes = self.write_to_memory_impl();
        let n = bytes.len();
        if let Some(dst) = storage {
            let m = n.min(dst.len());
            dst[..m].copy_from_slice(&bytes[..m]);
        }
        n
    }

    pub fn serialize(&self) -> SkData {
        SkData::make_with_copy(&self.write_to_memory_impl())
    }

    pub fn read_from_memory(buffer: &[u8], bytes_read: Option<&mut usize>) -> Option<SkPath> {
        let mut read = 0usize;
        let result = Self::read_from_memory_impl(buffer, &mut read);
        if let Some(bytes_read) = bytes_read {
            *bytes_read = read;
        }
        result
    }

    fn read_from_memory_impl(buffer: &[u8], bytes_read: &mut usize) -> Option<SkPath> {
        if buffer.len() < 4 {
            *bytes_read = 0;
            return None;
        }
        let packed = u32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let version = packed & 0xFF;
        let verbs_are_forward = version == 5;
        if !verbs_are_forward && version != 4 {
            *bytes_read = 0;
            return None;
        }
        let s_type = (packed >> 28) & 0xF;
        match s_type {
            1 => return Self::read_rrect_path(buffer, bytes_read),
            0 => {}
            _ => {
                *bytes_read = 0;
                return None;
            }
        }

        if buffer.len() < 16 {
            *bytes_read = 0;
            return None;
        }
        let pts = u32::from_ne_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;
        let cnx = u32::from_ne_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]) as usize;
        let vbs = u32::from_ne_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]) as usize;

        let mut offset = 16usize;
        let points_bytes = pts.checked_mul(8)?;
        let conics_bytes = cnx.checked_mul(4)?;
        let needed = offset
            .checked_add(points_bytes)?
            .checked_add(conics_bytes)?
            .checked_add(vbs)?;
        if buffer.len() < needed {
            *bytes_read = 0;
            return None;
        }

        let mut points = Vec::with_capacity(pts);
        for _ in 0..pts {
            let x = f32::from_ne_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            let y = f32::from_ne_bytes([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);
            points.push(Point::new(x, y));
            offset += 8;
        }
        let mut conics = Vec::with_capacity(cnx);
        for _ in 0..cnx {
            let w = f32::from_ne_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            conics.push(w);
            offset += 4;
        }
        let mut verbs: Vec<SkPathVerb> = Vec::with_capacity(vbs);
        for i in 0..vbs {
            verbs.push(u8_to_verb(buffer[offset + i]));
        }
        offset += vbs;
        offset = (offset + 3) & !3;
        *bytes_read = offset;

        let fill = fill_type_from_u32(packed);

        if vbs == 0 {
            if pts == 0 && cnx == 0 {
                return Some(SkPath::new_with_fill_type(fill));
            }
            *bytes_read = 0;
            return None;
        }
        if !verbs_are_forward {
            verbs.reverse();
        }
        let path = SkPath::Raw(&points, &verbs, &conics, fill, false);
        if !path.is_finite() {
            // `SkPath::Raw` returned null in the reference: invalid verbs/points/weights.
            *bytes_read = 0;
            return None;
        }
        Some(path)
    }

    fn read_rrect_path(buffer: &[u8], bytes_read: &mut usize) -> Option<SkPath> {
        if buffer.len() < 4 {
            *bytes_read = 0;
            return None;
        }
        let packed = u32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let dir = (packed >> 26) & 0x3;
        let rrect_dir = match dir {
            0 => SkPathDirection::Cw,
            1 => SkPathDirection::Ccw,
            _ => {
                *bytes_read = 0;
                return None;
            }
        };
        let fill = fill_type_from_u32(packed);
        if buffer.len() < 4 + RRect::K_SIZE_IN_MEMORY + 4 {
            *bytes_read = 0;
            return None;
        }
        let start_offset = 4 + RRect::K_SIZE_IN_MEMORY;
        let mut rrect = RRect::new();
        if rrect.read_from_memory(&buffer[4..start_offset]) != RRect::K_SIZE_IN_MEMORY {
            *bytes_read = 0;
            return None;
        }
        let start = i32::from_ne_bytes([
            buffer[start_offset],
            buffer[start_offset + 1],
            buffer[start_offset + 2],
            buffer[start_offset + 3],
        ]);
        if start != start.clamp(0, 7) {
            *bytes_read = 0;
            return None;
        }
        let mut path = SkPath::rrect(&rrect, rrect_dir, start as u32);
        path.set_fill_type(fill);
        let mut end = start_offset + 4;
        end = (end + 3) & !3;
        *bytes_read = end;
        Some(path)
    }

    // ---- dump ----

    pub fn dump(&self, stream: Option<&mut dyn SkWStream>, dump_as_hex: bool) {
        let as_type = if dump_as_hex {
            SkScalarAsStringType::Hex
        } else {
            SkScalarAsStringType::Dec
        };

        let mut builder = SkString::new();
        let ft = g_fill_type_strs(self.get_fill_type() as usize);
        builder.append_str(&format!("path.setFillType(SkPathFillType::k{ft});\n"));

        self.dump_iter(&mut builder, "path", as_type, true);

        if let Some(stream) = stream {
            stream.write(builder.as_bytes());
        } else {
            print!("{}", builder.as_str());
        }
    }

    pub fn dump_default(&self) {
        self.dump(None, false);
    }

    pub fn dump_hex(&self) {
        self.dump(None, true);
    }

    fn dump_iter(
        &self,
        builder: &mut SkString,
        cmd_prefix: &str,
        as_type: SkScalarAsStringType,
        use_semicolon: bool,
    ) {
        let mut iter = SkPathIter::new(self.points(), self.verbs(), self.conic_weights());
        while let Some(rec) = iter.next() {
            let mut cmd = cmd_prefix.to_string();
            let mut pts: &[Point] = &[];
            let mut cw = K_SENTINEL_CONIC_WEIGHT;
            match rec.f_verb {
                SkPathVerb::Move => {
                    cmd.push_str(".moveTo");
                    pts = &rec.f_points[0..1];
                }
                SkPathVerb::Line => {
                    cmd.push_str(".lineTo");
                    pts = &rec.f_points[1..2];
                }
                SkPathVerb::Quad => {
                    cmd.push_str(".quadTo");
                    pts = &rec.f_points[1..3];
                }
                SkPathVerb::Conic => {
                    cmd.push_str(".conicTo");
                    pts = &rec.f_points[1..3];
                    cw = rec.conic_weight();
                }
                SkPathVerb::Cubic => {
                    cmd.push_str(".cubicTo");
                    pts = &rec.f_points[1..4];
                }
                SkPathVerb::Close => {
                    cmd.push_str(".close()");
                    if use_semicolon {
                        cmd.push(';');
                    }
                    cmd.push('\n');
                    builder.append_str(&cmd);
                }
                SkPathVerb::Done => {}
            }
            if !pts.is_empty() {
                append_params(builder, &cmd, pts, as_type, use_semicolon, cw);
            }
        }
    }
}

fn all_points_eq(pts: &[Point]) -> bool {
    for i in 1..pts.len() {
        if pts[0] != pts[i] {
            return false;
        }
    }
    true
}

fn check_edge_against_rect(p0: Point, p1: Point, rect: &Rect, dir: SkPathDirection) -> bool {
    let (v, edge_begin) = if dir == SkPathDirection::Cw {
        (p1 - p0, p0)
    } else {
        (p0 - p1, p1)
    };
    if v.x != 0.0 || v.y != 0.0 {
        let yl = v.y * (rect.left - edge_begin.x);
        let xt = v.x * (rect.top - edge_begin.y);
        let yr = v.y * (rect.right - edge_begin.x);
        let xb = v.x * (rect.bottom - edge_begin.y);
        if xt < yl || xt < yr || xb < yl || xb < yr {
            return false;
        }
    }
    true
}

fn can_interpolate(from: &SkPath, to: &SkPath) -> bool {
    from.points().len() == to.points().len()
        && from.verbs() == to.verbs()
        && from.conic_weights() == to.conic_weights()
}

fn interpolate(from: &SkPath, to: &SkPath, t: Scalar) -> Option<SkPath> {
    if !can_interpolate(from, to) {
        return None;
    }
    let fill_type = from.get_fill_type();
    let from_pts = from.points();
    let to_pts = to.points();
    let mut dst: Vec<Point> = Vec::with_capacity(from_pts.len());
    for i in 0..from_pts.len() {
        dst.push(lerp(from_pts[i], to_pts[i], t));
    }
    Some(SkPath::Raw(
        dst.as_slice(),
        from.verbs(),
        from.conic_weights(),
        fill_type,
        false,
    ))
}

fn lerp(from: Point, to: Point, t: Scalar) -> Point {
    from + (to - from) * t
}

fn u8_to_verb(b: u8) -> SkPathVerb {
    match b {
        0 => SkPathVerb::Move,
        1 => SkPathVerb::Line,
        2 => SkPathVerb::Quad,
        3 => SkPathVerb::Conic,
        4 => SkPathVerb::Cubic,
        5 => SkPathVerb::Close,
        _ => SkPathVerb::Done,
    }
}

/// `extract_filltype`: the fill-type occupies bits 8..9 of the packed header.
fn fill_type_from_u32(v: u32) -> SkPathFillType {
    match (v >> 8) & 0x3 {
        0 => SkPathFillType::Winding,
        1 => SkPathFillType::EvenOdd,
        2 => SkPathFillType::InverseWinding,
        _ => SkPathFillType::InverseEvenOdd,
    }
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_ne_bytes());
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_ne_bytes());
}

fn g_fill_type_strs(index: usize) -> &'static str {
    const STRS: [&str; 4] = ["Winding", "EvenOdd", "InverseWinding", "InverseEvenOdd"];
    STRS[index & 3]
}

const K_SENTINEL_CONIC_WEIGHT: f32 = -12345.0;

fn append_params(
    str: &mut SkString,
    label: &str,
    pts: &[Point],
    str_type: SkScalarAsStringType,
    use_semicolon: bool,
    conic_weight: f32,
) {
    str.append_str(label);
    str.append_str("(");

    let mut values: Vec<Scalar> = Vec::with_capacity(pts.len() * 2);
    for p in pts {
        values.push(p.x);
        values.push(p.y);
    }
    let count = values.len();

    for (i, value) in values.iter().enumerate() {
        sk_append_scalar(str, *value, str_type);
        if i < count - 1 {
            str.append_str(", ");
        }
    }
    if conic_weight != K_SENTINEL_CONIC_WEIGHT {
        str.append_str(", ");
        sk_append_scalar(str, conic_weight, str_type);
    }
    str.append_str(if use_semicolon { ");" } else { ")" });
    if str_type == SkScalarAsStringType::Hex {
        str.append_str("  // ");
        for (i, value) in values.iter().enumerate() {
            let tmp = format!("{value}");
            str.append_str(&tmp);
            if i < count - 1 {
                str.append_str(", ");
            }
        }
        if conic_weight >= 0.0 {
            str.append_str(", ");
            str.append_str(&format!("{conic_weight}"));
        }
    }
    str.append_str("\n");
}

impl Default for SkPath {
    fn default() -> Self {
        Self::new()
    }
}

/// Port of `SkPath::Iter`.
pub struct Iter<'p> {
    f_pts: &'p [Point],
    f_pt_index: usize,
    f_verbs: &'p [SkPathVerb],
    f_verb_index: usize,
    f_verb_stop: usize,
    f_conic_weights: &'p [f32],
    f_conic_index: i64,
    f_move_to: Point,
    f_last_pt: Point,
    f_storage: [Point; 4],
    f_force_close: bool,
    f_need_close: bool,
    f_close_line: bool,
}

impl<'p> Iter<'p> {
    pub fn new() -> Iter<'p> {
        Iter {
            f_pts: &[],
            f_pt_index: 0,
            f_verbs: &[],
            f_verb_index: 0,
            f_verb_stop: 0,
            f_conic_weights: &[],
            f_conic_index: -1,
            f_move_to: Point::new(0.0, 0.0),
            f_last_pt: Point::new(0.0, 0.0),
            f_storage: [Point::new(0.0, 0.0); 4],
            f_force_close: false,
            f_need_close: false,
            f_close_line: false,
        }
    }

    pub fn from_path(path: &'p SkPath, force_close: bool) -> Iter<'p> {
        let mut iter = Iter::new();
        iter.set_path(path, force_close);
        iter
    }

    pub fn set_path(&mut self, path: &'p SkPath, force_close: bool) {
        self.f_pts = path.points();
        self.f_pt_index = 0;
        self.f_verbs = path.verbs();
        self.f_verb_index = 0;
        self.f_verb_stop = path.verbs().len();
        self.f_conic_weights = path.conic_weights();
        self.f_conic_index = -1;
        self.f_last_pt = Point::new(0.0, 0.0);
        self.f_move_to = Point::new(0.0, 0.0);
        self.f_force_close = force_close;
        self.f_need_close = false;
        self.f_close_line = false;
    }

    fn auto_close(&mut self, pts: &mut [Point; 4]) -> Verb {
        if self.f_last_pt != self.f_move_to {
            if is_nan(self.f_last_pt.x)
                || is_nan(self.f_last_pt.y)
                || is_nan(self.f_move_to.x)
                || is_nan(self.f_move_to.y)
            {
                return Verb::kClose_Verb;
            }
            pts[0] = self.f_last_pt;
            pts[1] = self.f_move_to;
            self.f_last_pt = self.f_move_to;
            self.f_close_line = true;
            return Verb::kLine_Verb;
        }
        pts[0] = self.f_move_to;
        Verb::kClose_Verb
    }

    pub fn next(&mut self, pts: &mut [Point; 4]) -> Verb {
        if self.f_verb_index == self.f_verb_stop {
            if self.f_need_close {
                if self.auto_close(pts) == Verb::kLine_Verb {
                    return Verb::kLine_Verb;
                }
                self.f_need_close = false;
                return Verb::kClose_Verb;
            }
            return Verb::kDone_Verb;
        }

        let mut verb = self.f_verbs[self.f_verb_index];
        self.f_verb_index += 1;

        match verb {
            SkPathVerb::Move => {
                if self.f_need_close {
                    self.f_verb_index -= 1;
                    verb = match self.auto_close(pts) {
                        Verb::kLine_Verb => SkPathVerb::Line,
                        _ => SkPathVerb::Close,
                    };
                    if verb == SkPathVerb::Close {
                        self.f_need_close = false;
                    }
                    return Verb::from_verb(verb);
                }
                if self.f_verb_index == self.f_verb_stop {
                    return Verb::kDone_Verb;
                }
                self.f_move_to = self.f_pts[self.f_pt_index];
                pts[0] = self.f_pts[self.f_pt_index];
                self.f_pt_index += 1;
                self.f_last_pt = self.f_move_to;
                self.f_need_close = self.f_force_close;
            }
            SkPathVerb::Line => {
                pts[0] = self.f_last_pt;
                pts[1] = self.f_pts[self.f_pt_index];
                self.f_last_pt = self.f_pts[self.f_pt_index];
                self.f_close_line = false;
                self.f_pt_index += 1;
            }
            SkPathVerb::Conic => {
                self.f_conic_index += 1;
                pts[0] = self.f_last_pt;
                pts[1] = self.f_pts[self.f_pt_index];
                pts[2] = self.f_pts[self.f_pt_index + 1];
                self.f_last_pt = self.f_pts[self.f_pt_index + 1];
                self.f_pt_index += 2;
            }
            SkPathVerb::Quad => {
                pts[0] = self.f_last_pt;
                pts[1] = self.f_pts[self.f_pt_index];
                pts[2] = self.f_pts[self.f_pt_index + 1];
                self.f_last_pt = self.f_pts[self.f_pt_index + 1];
                self.f_pt_index += 2;
            }
            SkPathVerb::Cubic => {
                pts[0] = self.f_last_pt;
                pts[1] = self.f_pts[self.f_pt_index];
                pts[2] = self.f_pts[self.f_pt_index + 1];
                pts[3] = self.f_pts[self.f_pt_index + 2];
                self.f_last_pt = self.f_pts[self.f_pt_index + 2];
                self.f_pt_index += 3;
            }
            SkPathVerb::Close => {
                let v = self.auto_close(pts);
                if v == Verb::kLine_Verb {
                    self.f_verb_index -= 1;
                    verb = SkPathVerb::Line;
                } else {
                    self.f_need_close = false;
                    verb = SkPathVerb::Close;
                }
                self.f_last_pt = self.f_move_to;
            }
            SkPathVerb::Done => return Verb::kDone_Verb,
        }
        Verb::from_verb(verb)
    }

    pub fn next_rec(&mut self) -> Option<SkPathIterRec<'_>> {
        let mut pts = [Point::new(0.0, 0.0); 4];
        let v = self.next(&mut pts);
        if v == Verb::kDone_Verb {
            return None;
        }
        let n = match v {
            Verb::kMove_Verb => 1,
            Verb::kLine_Verb => 2,
            Verb::kQuad_Verb => 3,
            Verb::kConic_Verb => 3,
            Verb::kCubic_Verb => 4,
            Verb::kClose_Verb => 2,
            Verb::kDone_Verb => 0,
        };
        self.f_storage = pts;
        let weight = if v == Verb::kConic_Verb {
            self.conic_weight()
        } else {
            1.0
        };
        Some(SkPathIterRec {
            f_points: &self.f_storage[..n],
            f_conic_weight: weight,
            f_verb: verb_from_verb(v),
        })
    }

    pub fn conic_weight(&self) -> f32 {
        if self.f_conic_index >= 0 && (self.f_conic_index as usize) < self.f_conic_weights.len() {
            self.f_conic_weights[self.f_conic_index as usize]
        } else {
            0.0
        }
    }

    pub fn is_close_line(&self) -> bool {
        self.f_close_line
    }

    pub fn is_closed_contour(&self) -> bool {
        if self.f_verbs.is_empty() || self.f_verb_index == self.f_verb_stop {
            return false;
        }
        if self.f_force_close {
            return true;
        }
        let mut i = self.f_verb_index;
        if self.f_verbs[i] == SkPathVerb::Move {
            i += 1;
        }
        while i < self.f_verb_stop {
            let v = self.f_verbs[i];
            i += 1;
            if v == SkPathVerb::Move {
                break;
            }
            if v == SkPathVerb::Close {
                return true;
            }
        }
        false
    }
}

fn verb_from_verb(v: Verb) -> SkPathVerb {
    match v {
        Verb::kMove_Verb => SkPathVerb::Move,
        Verb::kLine_Verb => SkPathVerb::Line,
        Verb::kQuad_Verb => SkPathVerb::Quad,
        Verb::kConic_Verb => SkPathVerb::Conic,
        Verb::kCubic_Verb => SkPathVerb::Cubic,
        Verb::kClose_Verb => SkPathVerb::Close,
        Verb::kDone_Verb => SkPathVerb::Done,
    }
}

fn pts_advance_after_verb(v: SkPathVerb) -> usize {
    match v {
        SkPathVerb::Move => 1,
        SkPathVerb::Line => 1,
        SkPathVerb::Quad => 2,
        SkPathVerb::Conic => 2,
        SkPathVerb::Cubic => 3,
        SkPathVerb::Close => 0,
        SkPathVerb::Done => 0,
    }
}

/// Port of `SkPath::RangeIter`.
pub struct RangeIter<'p> {
    f_verbs: &'p [SkPathVerb],
    f_points: &'p [Point],
    f_weights: &'p [f32],
    f_verb_index: usize,
    f_point_index: usize,
    f_weight_index: usize,
}

impl<'p> RangeIter<'p> {
    pub fn new() -> RangeIter<'p> {
        RangeIter {
            f_verbs: &[],
            f_points: &[],
            f_weights: &[],
            f_verb_index: 0,
            f_point_index: 0,
            f_weight_index: 0,
        }
    }

    pub fn from_slices(
        verbs: &'p [SkPathVerb],
        points: &'p [Point],
        weights: &'p [f32],
    ) -> RangeIter<'p> {
        RangeIter {
            f_verbs: verbs,
            f_points: points,
            f_weights: weights,
            f_verb_index: 0,
            f_point_index: 0,
            f_weight_index: 0,
        }
    }

    pub fn at_end(&self) -> bool {
        self.f_verb_index >= self.f_verbs.len()
    }

    pub fn peek_verb(&self) -> SkPathVerb {
        if self.f_verb_index < self.f_verbs.len() {
            self.f_verbs[self.f_verb_index]
        } else {
            SkPathVerb::Done
        }
    }

    pub fn next(&mut self) -> Option<(SkPathVerb, &'p [Point], f32)> {
        if self.f_verb_index >= self.f_verbs.len() {
            return None;
        }
        let verb = self.f_verbs[self.f_verb_index];
        let backset = if verb == SkPathVerb::Move { 0 } else { 1 };
        let start = self.f_point_index.saturating_sub(backset);
        let want = G_PTS_PER_VERB[verb as usize] + 1;
        let available = self.f_points.len().saturating_sub(start);
        let len = want.min(available);
        let pts = &self.f_points[start..start + len];
        let weight = if verb == SkPathVerb::Conic {
            let w = self
                .f_weights
                .get(self.f_weight_index)
                .copied()
                .unwrap_or(1.0);
            self.f_weight_index += 1;
            w
        } else {
            1.0
        };
        self.f_point_index += pts_advance_after_verb(verb);
        self.f_verb_index += 1;
        Some((verb, pts, weight))
    }
}

/// Port of `SkPath::RawIter`.
pub struct RawIter<'p> {
    f_verbs: &'p [SkPathVerb],
    f_points: &'p [Point],
    f_weights: &'p [f32],
    f_verb_index: usize,
    f_point_index: usize,
    f_weight_index: usize,
    f_conic_weight: f32,
    f_storage: [Point; 4],
}

impl<'p> RawIter<'p> {
    pub fn new() -> RawIter<'p> {
        RawIter {
            f_verbs: &[],
            f_points: &[],
            f_weights: &[],
            f_verb_index: 0,
            f_point_index: 0,
            f_weight_index: 0,
            f_conic_weight: 0.0,
            f_storage: [Point::new(0.0, 0.0); 4],
        }
    }

    pub fn from_path(path: &'p SkPath) -> RawIter<'p> {
        let mut iter = RawIter::new();
        iter.set_path(path);
        iter
    }

    pub fn set_path(&mut self, path: &'p SkPath) {
        self.f_verbs = path.verbs();
        self.f_points = path.points();
        self.f_weights = path.conic_weights();
        self.f_verb_index = 0;
        self.f_point_index = 0;
        self.f_weight_index = 0;
        self.f_conic_weight = 0.0;
    }

    pub fn peek(&self) -> Verb {
        if self.f_verb_index < self.f_verbs.len() {
            Verb::from_verb(self.f_verbs[self.f_verb_index])
        } else {
            Verb::kDone_Verb
        }
    }

    pub fn next(&mut self, pts: &mut [Point; 4]) -> Verb {
        if self.f_verb_index >= self.f_verbs.len() {
            return Verb::kDone_Verb;
        }
        let verb = self.f_verbs[self.f_verb_index];
        let backset = if verb == SkPathVerb::Move { 0 } else { 1 };
        let start = self.f_point_index.saturating_sub(backset);
        let want = G_PTS_PER_VERB[verb as usize] + 1;
        let available = self.f_points.len().saturating_sub(start);
        let len = want.min(available);
        pts[..len].copy_from_slice(&self.f_points[start..start + len]);
        if verb == SkPathVerb::Conic {
            self.f_conic_weight = self
                .f_weights
                .get(self.f_weight_index)
                .copied()
                .unwrap_or(1.0);
            self.f_weight_index += 1;
        }
        self.f_point_index += pts_advance_after_verb(verb);
        self.f_verb_index += 1;
        Verb::from_verb(verb)
    }

    pub fn next_rec(&mut self) -> Option<SkPathIterRec<'_>> {
        let mut pts = [Point::new(0.0, 0.0); 4];
        let v = self.next(&mut pts);
        if v == Verb::kDone_Verb {
            return None;
        }
        let n = match v {
            Verb::kMove_Verb => 1,
            Verb::kLine_Verb => 2,
            Verb::kQuad_Verb => 3,
            Verb::kConic_Verb => 3,
            Verb::kCubic_Verb => 4,
            Verb::kClose_Verb => 0,
            Verb::kDone_Verb => 0,
        };
        let weight = if v == Verb::kConic_Verb {
            self.f_conic_weight
        } else {
            1.0
        };
        self.f_storage = pts;
        Some(SkPathIterRec {
            f_points: &self.f_storage[..n],
            f_conic_weight: weight,
            f_verb: verb_from_verb(v),
        })
    }

    pub fn conic_weight(&self) -> f32 {
        self.f_conic_weight
    }
}

impl Default for Iter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RangeIter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RawIter<'_> {
    fn default() -> Self {
        Self::new()
    }
}
