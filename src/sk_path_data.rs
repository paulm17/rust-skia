//! Port of `src/core/SkPathData.h` and `src/core/SkPathData.cpp`.
//!
//! Adaptation: C++ reference-counts this immutable container via `SkNVRefCnt` and hands out
//! `sk_sp<SkPathData>`. The Rust port uses `std::sync::Arc<SkPathData>`; the observable
//! copy-on-write / shared-identity / generation-id behavior is preserved (copying an `SkPath`
//! shares the same `Arc`, and `getGenerationID` reports the shared `unique_id`). Convexity is
//! stored in an atomic and gen-id listeners behind a mutex so the container may be shared.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::id_listener::{SkIDChangeListener, SkIDChangeListenerList};
use crate::matrix::Matrix;
use crate::point::Point;
use crate::rect::Rect;
use crate::rrect::RRect;
use crate::sk_path_priv;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_raw_shapes::SkPathRawShapes;
use crate::sk_path_types::{
    SkPathConvexity, SkPathDirection, SkPathFillType, SkPathVerb, SkResolveConvexity,
};

/// `SkPathIsAType` from `include/private/SkPathRef.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SkPathIsAType {
    #[default]
    General = 0,
    Oval = 1,
    RRect = 2,
}

/// `SkPathIsAData` from `include/private/SkPathRef.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkPathIsAData {
    pub f_start_index: u8,
    pub f_direction: SkPathDirection,
}

impl Default for SkPathIsAData {
    fn default() -> Self {
        SkPathIsAData {
            f_start_index: 0,
            f_direction: SkPathDirection::Cw,
        }
    }
}

/// `SkPathRectInfo` from `include/private/SkPathRef.h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkPathRectInfo {
    pub f_rect: Rect,
    pub f_direction: SkPathDirection,
    pub f_start_index: u8,
}

/// `SkPathOvalInfo` from `include/private/SkPathRef.h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkPathOvalInfo {
    pub f_bounds: Rect,
    pub f_direction: SkPathDirection,
    pub f_start_index: u8,
}

/// `SkPathRRectInfo` from `include/private/SkPathRef.h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkPathRRectInfo {
    pub f_rrect: RRect,
    pub f_direction: SkPathDirection,
    pub f_start_index: u8,
}

/// `gPtsPerVerb` from `SkPathData.cpp` (indexed by `SkPathVerb`).
const G_PTS_PER_VERB: [usize; 6] = [1, 1, 2, 2, 3, 0];

static G_NEXT_UNIQUE_ID: AtomicU64 = AtomicU64::new(1);

fn next_path_data_unique_id() -> u64 {
    const K_MAX_ID: u64 = u64::MAX >> 2;
    let id = G_NEXT_UNIQUE_ID.fetch_add(1, Ordering::Relaxed);
    if id > K_MAX_ID {
        G_NEXT_UNIQUE_ID.store(2, Ordering::Relaxed);
        1
    } else {
        id
    }
}

fn valid_conic_weight(w: f32) -> bool {
    w >= 0.0 && w.is_finite()
}

/// `valid_path_data`.
fn valid_path_data(pts: &[Point], verbs: &[SkPathVerb], conics: &[f32]) -> bool {
    if verbs.is_empty() {
        return pts.is_empty() && conics.is_empty();
    }
    if verbs[0] != SkPathVerb::Move {
        return false;
    }

    let mut point_count = 1usize;
    let mut conic_count = 0usize;
    for i in 1..verbs.len() {
        let curr = verbs[i];
        let prev = verbs[i - 1];
        if curr as u8 > SkPathVerb::Close as u8 {
            return false;
        }
        if prev == SkPathVerb::Move && curr == SkPathVerb::Move {
            return false;
        }
        if prev == SkPathVerb::Close && curr != SkPathVerb::Move {
            return false;
        }
        point_count += G_PTS_PER_VERB[curr as usize];
        if curr == SkPathVerb::Conic {
            conic_count += 1;
        }
    }

    if pts.len() != point_count || conics.len() != conic_count {
        return false;
    }
    conics.iter().all(|&w| valid_conic_weight(w))
}

/// Immutable path payload. See module docs for the `Arc` adaptation.
pub struct SkPathData {
    f_points: Vec<Point>,
    f_conics: Vec<f32>,
    f_verbs: Vec<SkPathVerb>,
    f_bounds: Rect,
    f_unique_id: u64,
    f_convexity: AtomicU8,
    f_segment_mask: u8,
    f_type: SkPathIsAType,
    f_is_a: SkPathIsAData,
    f_gen_id_change_listeners: Mutex<SkIDChangeListenerList>,
}

impl std::fmt::Debug for SkPathData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkPathData")
            .field("points", &self.f_points)
            .field("verbs", &self.f_verbs)
            .field("conics", &self.f_conics)
            .field("bounds", &self.f_bounds)
            .field("unique_id", &self.f_unique_id)
            .finish()
    }
}

impl PartialEq for SkPathData {
    fn eq(&self, other: &Self) -> bool {
        self.f_points == other.f_points
            && self.f_conics == other.f_conics
            && self.f_verbs == other.f_verbs
    }
}
impl Eq for SkPathData {}

impl SkPathData {
    fn empty_instance() -> Arc<SkPathData> {
        static EMPTY: OnceLock<Arc<SkPathData>> = OnceLock::new();
        EMPTY
            .get_or_init(|| {
                Arc::new(SkPathData {
                    f_points: Vec::new(),
                    f_conics: Vec::new(),
                    f_verbs: Vec::new(),
                    f_bounds: Rect::make_empty(),
                    f_unique_id: next_path_data_unique_id(),
                    f_convexity: AtomicU8::new(SkPathConvexity::ConvexDegenerate as u8),
                    f_segment_mask: 0,
                    f_type: SkPathIsAType::General,
                    f_is_a: SkPathIsAData::default(),
                    f_gen_id_change_listeners: Mutex::new(SkIDChangeListenerList::new()),
                })
            })
            .clone()
    }

    /// `SkPathData::Empty()`.
    pub fn empty() -> Arc<SkPathData> {
        Self::empty_instance()
    }

    /// Singleton used by `SkPath` to mark invalid / non-finite payloads.
    pub fn peek_error_singleton() -> Arc<SkPathData> {
        static ERROR: OnceLock<Arc<SkPathData>> = OnceLock::new();
        ERROR
            .get_or_init(|| {
                Arc::new(SkPathData {
                    f_points: Vec::new(),
                    f_conics: Vec::new(),
                    f_verbs: Vec::new(),
                    f_bounds: Rect::make_empty(),
                    f_unique_id: next_path_data_unique_id(),
                    f_convexity: AtomicU8::new(SkPathConvexity::Concave as u8),
                    f_segment_mask: 0,
                    f_type: SkPathIsAType::General,
                    f_is_a: SkPathIsAData::default(),
                    f_gen_id_change_listeners: Mutex::new(SkIDChangeListenerList::new()),
                })
            })
            .clone()
    }

    fn finish_init(
        points: &[Point],
        cons: &[f32],
        verbs: &[SkPathVerb],
        bounds: Option<Rect>,
        segment_mask: Option<u8>,
    ) -> Option<(Rect, u8)> {
        if points.is_empty() {
            return Some((Rect::make_empty(), 0));
        }
        let mask = match segment_mask {
            Some(m) => m,
            None => sk_path_priv::compute_segment_mask(verbs),
        };
        let bounds = match bounds {
            Some(b) => b.make_sorted(),
            None => sk_path_priv::trimmed_bounds(points, verbs)?,
        };
        let _ = cons;
        Some((bounds, mask))
    }

    /// `SkPathData::Make` - validates and constructs, returning `None` on invalid input.
    pub fn make(pts: &[Point], verbs: &[SkPathVerb], conics: &[f32]) -> Option<Arc<SkPathData>> {
        if !valid_path_data(pts, verbs, conics) {
            return None;
        }
        // MakeNoCheck *does* compute/check bounds if we don't pass them in
        Self::make_no_check(pts, verbs, conics, None, None)
    }

    /// `SkPathData::MakeNoCheck` (bounds/segment mask optional).
    pub fn make_no_check(
        pts: &[Point],
        verbs: &[SkPathVerb],
        conics: &[f32],
        bounds: Option<Rect>,
        segment_mask: Option<u8>,
    ) -> Option<Arc<SkPathData>> {
        let (bounds, mask) = Self::finish_init(pts, conics, verbs, bounds, segment_mask)?;
        Some(Arc::new(SkPathData {
            f_points: pts.to_vec(),
            f_conics: conics.to_vec(),
            f_verbs: verbs.to_vec(),
            f_bounds: bounds,
            f_unique_id: next_path_data_unique_id(),
            f_convexity: AtomicU8::new(SkPathConvexity::Unknown as u8),
            f_segment_mask: mask,
            f_type: SkPathIsAType::General,
            f_is_a: SkPathIsAData::default(),
            f_gen_id_change_listeners: Mutex::new(SkIDChangeListenerList::new()),
        }))
    }

    /// `SkPathData::Make` with validation (public path factory `SkPath::Raw`).
    pub fn make_checked(
        pts: &[Point],
        verbs: &[SkPathVerb],
        conics: &[f32],
    ) -> Option<Arc<SkPathData>> {
        Self::make(pts, verbs, conics)
    }

    /// `SkPathData::Rect`.
    pub fn rect(r: &Rect, dir: SkPathDirection, start_index: u32) -> Option<Arc<SkPathData>> {
        if !r.is_finite() {
            return None;
        }
        let raw = SkPathRawShapes::rect(r, dir, start_index as usize);
        Self::make_no_check(&raw.f_points, &raw.f_verbs, &raw.f_conics, None, None)
    }

    /// `SkPathData::Oval`.
    pub fn oval(r: &Rect, dir: SkPathDirection, start_index: u32) -> Option<Arc<SkPathData>> {
        if !r.is_finite() {
            return None;
        }
        let raw = SkPathRawShapes::oval(r, dir, start_index as usize);
        let data = Self::make_no_check(&raw.f_points, &raw.f_verbs, &raw.f_conics, None, None)?;
        data.setup_is_a(SkPathIsAType::Oval, dir, start_index as u8);
        Some(data)
    }

    /// `SkPathData::RRect`.
    pub fn rrect(rr: &RRect, dir: SkPathDirection, start_index: u32) -> Option<Arc<SkPathData>> {
        if !rr.get_bounds().is_finite() {
            return None;
        }
        let raw = SkPathRawShapes::rrect(rr, dir, start_index as usize);
        let data = Self::make_checked(&raw.f_points, &raw.f_verbs, &raw.f_conics)?;
        data.setup_is_a(SkPathIsAType::RRect, dir, start_index as u8);
        Some(data)
    }

    /// `SkPathData::Polygon`.
    pub fn polygon(pts: &[Point], is_closed: bool) -> Arc<SkPathData> {
        if pts.is_empty() || (pts.len() == 1 && !is_closed) {
            return Self::empty();
        }
        let nverbs = pts.len() + if is_closed { 1 } else { 0 };
        let mut verbs = Vec::with_capacity(nverbs);
        verbs.push(SkPathVerb::Move);
        for _ in 1..pts.len() {
            verbs.push(SkPathVerb::Line);
        }
        if is_closed {
            verbs.push(SkPathVerb::Close);
        }
        let mask = crate::sk_path_types::K_LINE_SK_PATH_SEGMENT_MASK;
        Self::make_unchecked_ok(pts.to_vec(), verbs, Vec::new(), Some(mask))
            .unwrap_or_else(Self::peek_error_singleton)
    }

    /// `SkPathData::Line`.
    pub fn line(a: Point, b: Point) -> Arc<SkPathData> {
        Self::polygon(&[a, b], false)
    }

    fn make_unchecked_ok(
        points: Vec<Point>,
        verbs: Vec<SkPathVerb>,
        conics: Vec<f32>,
        segment_mask: Option<u8>,
    ) -> Option<Arc<SkPathData>> {
        let (bounds, mask) = Self::finish_init(&points, &conics, &verbs, None, segment_mask)?;
        Some(Arc::new(SkPathData {
            f_points: points,
            f_conics: conics,
            f_verbs: verbs,
            f_bounds: bounds,
            f_unique_id: next_path_data_unique_id(),
            f_convexity: AtomicU8::new(SkPathConvexity::Unknown as u8),
            f_segment_mask: mask,
            f_type: SkPathIsAType::General,
            f_is_a: SkPathIsAData::default(),
            f_gen_id_change_listeners: Mutex::new(SkIDChangeListenerList::new()),
        }))
    }

    pub fn setup_is_a(&self, ty: SkPathIsAType, dir: SkPathDirection, start_index: u8) {
        self.set_convexity(crate::sk_path_types::sk_path_direction_to_convexity(dir));
        debug_assert!(
            ty == SkPathIsAType::Oval && start_index < 4
                || ty == SkPathIsAType::RRect && start_index < 8
                || ty == SkPathIsAType::General
        );
        // f_type / f_is_a are logically immutable after construction; the only writers are the
        // factories above, which own the Arc uniquely at that point.
        let ptr = self as *const SkPathData as *mut SkPathData;
        unsafe {
            (*ptr).f_type = ty;
            (*ptr).f_is_a = SkPathIsAData {
                f_start_index: start_index,
                f_direction: dir,
            };
        }
    }

    // ---- accessors ----

    pub fn points(&self) -> &[Point] {
        &self.f_points
    }

    pub fn verbs(&self) -> &[SkPathVerb] {
        &self.f_verbs
    }

    pub fn conics(&self) -> &[f32] {
        &self.f_conics
    }

    pub fn bounds(&self) -> &Rect {
        &self.f_bounds
    }

    pub fn segment_mask(&self) -> u8 {
        self.f_segment_mask
    }

    pub fn unique_id(&self) -> u64 {
        self.f_unique_id
    }

    pub fn is_empty(&self) -> bool {
        self.f_points.is_empty()
    }

    pub fn get_convexity_or_unknown(&self) -> SkPathConvexity {
        u8_to_convexity(self.f_convexity.load(Ordering::Relaxed))
    }

    pub fn get_resolved_convexity(&self) -> SkPathConvexity {
        let c = self.get_convexity_or_unknown();
        if c != SkPathConvexity::Unknown {
            return c;
        }
        let computed =
            sk_path_priv::compute_convexity(&self.f_points, &self.f_verbs, &self.f_conics);
        self.set_convexity(computed);
        computed
    }

    pub fn set_convexity(&self, c: SkPathConvexity) {
        self.f_convexity.store(c as u8, Ordering::Relaxed);
    }

    pub fn is_convex(&self) -> bool {
        self.get_resolved_convexity().is_convex()
    }

    pub fn compute_tight_bounds(&self) -> Rect {
        sk_path_priv::compute_tight_bounds(&self.f_points, &self.f_verbs, &self.f_conics)
    }

    pub fn raw(&self, fill_type: SkPathFillType, resolve: SkResolveConvexity) -> SkPathRaw {
        let convexity = match resolve {
            SkResolveConvexity::Yes => self.get_resolved_convexity(),
            SkResolveConvexity::No => self.get_convexity_or_unknown(),
        };
        SkPathRaw {
            f_points: self.f_points.clone(),
            f_verbs: self.f_verbs.clone(),
            f_conics: self.f_conics.clone(),
            f_bounds: self.f_bounds,
            f_fill_type: fill_type,
            f_convexity: convexity,
            f_segment_mask: self.f_segment_mask,
        }
    }

    pub fn contains(&self, p: Point, fill_type: SkPathFillType) -> bool {
        sk_path_priv::contains(&self.raw(fill_type, SkResolveConvexity::No), p)
    }

    pub fn as_line(&self) -> Option<[Point; 2]> {
        if self.f_points.len() == 2
            && self.f_verbs.len() == 2
            && self.f_conics.is_empty()
            && self.f_verbs[1] == SkPathVerb::Line
        {
            return Some([self.f_points[0], self.f_points[1]]);
        }
        None
    }

    pub fn as_rect(&self) -> Option<SkPathRectInfo> {
        let rc = sk_path_priv::is_rect_contour(
            &self.f_points,
            &self.f_verbs,
            self.f_segment_mask as u32,
            false,
        )?;
        Some(SkPathRectInfo {
            f_rect: rc.f_rect,
            f_direction: rc.f_direction,
            f_start_index: 0,
        })
    }

    pub fn as_oval(&self) -> Option<SkPathOvalInfo> {
        if self.f_type != SkPathIsAType::Oval {
            return None;
        }
        Some(SkPathOvalInfo {
            f_bounds: self.f_bounds,
            f_direction: self.f_is_a.f_direction,
            f_start_index: self.f_is_a.f_start_index,
        })
    }

    pub fn as_rrect(&self) -> Option<SkPathRRectInfo> {
        if self.f_type != SkPathIsAType::RRect {
            return None;
        }
        Some(SkPathRRectInfo {
            f_rrect: sk_path_priv::deduce_rrect_from_contour(
                &self.f_bounds,
                &self.f_points,
                &self.f_verbs,
            ),
            f_direction: self.f_is_a.f_direction,
            f_start_index: self.f_is_a.f_start_index,
        })
    }

    pub fn add_gen_id_change_listener(&self, listener: Arc<dyn SkIDChangeListener>) {
        self.f_gen_id_change_listeners
            .lock()
            .unwrap()
            .add(Some(listener));
    }

    pub fn gen_id_change_listener_count(&self) -> usize {
        self.f_gen_id_change_listeners.lock().unwrap().len()
    }

    // ---- transforms ----

    /// `SkPathData::MakeTransform` (perspective branch defers to `SkPathBuilder`).
    pub fn make_transform(raw: &SkPathRaw, mx: &Matrix) -> Option<Arc<SkPathData>> {
        if raw.f_points.is_empty() {
            return Some(Self::empty());
        }
        if mx.has_perspective() {
            return Some(
                crate::sk_path_builder::SkPathBuilder::new_with_fill_type(SkPathFillType::Winding)
                    .add_raw(raw, crate::sk_path_builder::Reserve::Exact)
                    .transform(mx)
                    .detach_data(),
            );
        }

        let mut points = raw.f_points.clone();
        mx.map_points(&mut points, &raw.f_points);
        let conics = raw.f_conics.clone();
        let verbs = raw.f_verbs.clone();

        let transformed_bounds = if mx.rect_stays_rect() {
            let b = mx.map_rect(&raw.f_bounds);
            if !b.is_finite() {
                return None;
            }
            Some(b)
        } else {
            None
        };

        let data = Self::make_no_check(
            &points,
            &verbs,
            &conics,
            transformed_bounds,
            Some(raw.f_segment_mask),
        )?;
        data.set_convexity(sk_path_priv::transform_convexity(
            mx,
            &raw.f_points,
            raw.f_convexity,
        ));
        Some(data)
    }

    /// `SkPathData::makeTransform`.
    pub fn make_transform_from(&self, mx: &Matrix) -> Option<Arc<SkPathData>> {
        if mx.is_identity() {
            return Some(Arc::new(SkPathData {
                f_points: self.f_points.clone(),
                f_conics: self.f_conics.clone(),
                f_verbs: self.f_verbs.clone(),
                f_bounds: self.f_bounds,
                f_unique_id: self.f_unique_id,
                f_convexity: AtomicU8::new(self.f_convexity.load(Ordering::Relaxed)),
                f_segment_mask: self.f_segment_mask,
                f_type: self.f_type,
                f_is_a: self.f_is_a,
                f_gen_id_change_listeners: Mutex::new(SkIDChangeListenerList::new()),
            }));
        }
        let raw = self.raw(SkPathFillType::Winding, SkResolveConvexity::No);
        let result = Self::make_transform(&raw, mx)?;
        if self.f_type == SkPathIsAType::General {
            return Some(result);
        }
        let can_maintain = mx.rect_stays_rect()
            && sk_path_priv::is_axis_aligned(&self.f_points)
            && !result.f_bounds.is_empty();
        if !can_maintain {
            return Some(result);
        }
        let (dir, start) = sk_path_priv::transform_dir_and_start(
            mx,
            self.f_type == SkPathIsAType::RRect,
            self.f_is_a.f_direction,
            self.f_is_a.f_start_index as u32,
        );
        result.setup_is_a(self.f_type, dir, start as u8);
        Some(result)
    }

    /// `SkPathData::makeOffset`.
    pub fn make_offset(&self, v: Point) -> Option<Arc<SkPathData>> {
        self.make_transform_from(&Matrix::translate(v.x, v.y))
    }
}

fn u8_to_convexity(v: u8) -> SkPathConvexity {
    match v {
        0 => SkPathConvexity::ConvexCw,
        1 => SkPathConvexity::ConvexCcw,
        2 => SkPathConvexity::ConvexDegenerate,
        3 => SkPathConvexity::Concave,
        _ => SkPathConvexity::Unknown,
    }
}
