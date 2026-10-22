//! Port of `include/core/SkBBHFactory.h` and `src/core/SkBBHFactory.cpp`.
//!
//! `SkBBoxHierarchy` is an `SkRefCnt` in the reference. The port models ownership with `Arc`, so a
//! factory returns `Arc<dyn SkBBoxHierarchy>` in place of `sk_sp<SkBBoxHierarchy>` (including the
//! reference's `nullptr` failure case).

use std::sync::Arc;

use crate::rect::Rect;
use crate::sk_rtree::SkRTree;

/// `SkBBoxHierarchy::Metadata`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    /// The corresponding `SkRect` bounds a draw command, not a pure state change.
    pub is_draw: bool,
}

/// `SkBBoxHierarchy`.
pub trait SkBBoxHierarchy: Send + Sync {
    /// Insert `N` bounding boxes into the hierarchy.
    fn insert(&mut self, rects: &[Rect]);

    /// `SkBBoxHierarchy::insert(const SkRect[], const Metadata[], int N)`. The reference's default
    /// implementation ignores the metadata and forwards to `insert(rects, N)`.
    fn insert_with_metadata(&mut self, rects: &[Rect], _metadata: &[Metadata]) {
        self.insert(rects);
    }

    /// Populate `results` with the indices of bounding boxes intersecting `query`.
    fn search(&self, query: &Rect, results: &mut Vec<i32>);

    /// Return approximate size in memory of `*this`.
    fn bytes_used(&self) -> usize;
}

/// `SkBBHFactory`.
pub trait SkBBHFactory {
    /// Allocate a new `SkBBoxHierarchy`. Returns `None` on failure.
    fn create(&self) -> Option<Arc<dyn SkBBoxHierarchy>>;
}

/// `SkRTreeFactory`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SkRTreeFactory;

impl SkBBHFactory for SkRTreeFactory {
    fn create(&self) -> Option<Arc<dyn SkBBoxHierarchy>> {
        Some(Arc::new(SkRTree::new()))
    }
}

impl SkRTreeFactory {
    /// `SkRTreeFactory::operator()()`.
    pub fn make_hierarchy(&self) -> Option<Arc<dyn SkBBoxHierarchy>> {
        self.create()
    }
}

/// `SkBBoxHierarchy::insert(const SkRect[], const Metadata[], int N)`, usable without a trait
/// object.
pub fn insert_with_metadata<T: SkBBoxHierarchy + ?Sized>(
    hierarchy: &mut T,
    rects: &[Rect],
    metadata: &[Metadata],
) {
    hierarchy.insert_with_metadata(rects, metadata);
}
