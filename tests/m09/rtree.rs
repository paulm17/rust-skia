// Copyright 2012 Google Inc. BSD: tests/LICENSE.skia.
// Faithful translation: tests/RTreeTest.cpp:23–103; registration RTree:65.
use crate::support::{Reporter, reporter_assert as check};
use rust_skia::{random::SkRandom, rect::Rect, sk_bbh_factory::SkBBoxHierarchy, sk_rtree::SkRTree};
const NUM_RECTS: usize = 200;
const NUM_ITERATIONS: usize = 100;
const NUM_QUERIES: usize = 50;
fn random_rect(rand: &mut SkRandom) -> Rect {
    let mut rect = Rect::default();
    while rect.is_empty() {
        // Preserve the original left, right, top, bottom RNG consumption order.
        let left = rand.next_range_f(0.0, 1000.0);
        let right = rand.next_range_f(0.0, 1000.0);
        let top = rand.next_range_f(0.0, 1000.0);
        let bottom = rand.next_range_f(0.0, 1000.0);
        rect = Rect::new(left, top, right, bottom);
        rect.sort();
    }
    rect
}
fn verify_query(query: Rect, rects: &[Rect], found: &[i32]) -> bool {
    let mut expected = Vec::new();
    for (i, rect) in rects.iter().enumerate().take(NUM_RECTS) {
        if Rect::intersects_rects(&query, rect) {
            expected.push(i as i32);
        }
    }
    if expected.len() != found.len() {
        return false;
    }
    if expected.is_empty() {
        return true;
    }
    found == expected
}
fn run_queries(reporter: &mut Reporter, rand: &mut SkRandom, rects: &[Rect], tree: &SkRTree) {
    for _ in 0..NUM_QUERIES {
        let mut hits = Vec::new();
        let query = random_rect(rand);
        tree.search(&query, &mut hits);
        check!(reporter, verify_query(query, rects, &hits));
    }
}
#[test]
fn RTree() {
    let mut reporter = Reporter::default();
    let mut expectedDepthMin = -1;
    let mut tmp = NUM_RECTS as i32;
    while tmp > 0 {
        tmp -=
            (rust_skia::sk_rtree::K_MAX_CHILDREN as f64).powf((expectedDepthMin + 1) as f64) as i32;
        expectedDepthMin += 1;
    }
    let mut expectedDepthMax = -1;
    tmp = NUM_RECTS as i32;
    while tmp > 0 {
        tmp -=
            (rust_skia::sk_rtree::K_MIN_CHILDREN as f64).powf((expectedDepthMax + 1) as f64) as i32;
        expectedDepthMax += 1;
    }
    let mut rand = SkRandom::new();
    let mut rects = [Rect::default(); NUM_RECTS];
    for _ in 0..NUM_ITERATIONS {
        let mut rtree = SkRTree::new();
        check!(reporter, 0 == rtree.get_count());
        for rect in &mut rects {
            *rect = random_rect(&mut rand);
        }
        rtree.insert_rects(&rects);
        run_queries(&mut reporter, &mut rand, &rects, &rtree);
        check!(reporter, NUM_RECTS as i32 == rtree.get_count());
        check!(
            reporter,
            expectedDepthMin <= rtree.get_depth() && expectedDepthMax >= rtree.get_depth()
        );
    }
    reporter.finish();
}
