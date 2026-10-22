// Supplemental M09 contracts; these do not replace the two upstream registrations.
use rust_skia::{
    matrix::Matrix,
    rect::{IRect, Rect},
    sk_aa_clip::SkAAClip,
    sk_bbh_factory::{Metadata, SkBBoxHierarchy, SkRTreeFactory},
    sk_clip_op::SkClipOp,
    sk_clip_stack::{SkClipStack, SkShaderHandle},
    sk_region::{SkRegion, SkRegionOp},
    sk_rtree::SkRTree,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn encoded(region: &SkRegion) -> Vec<u8> {
    let n = region.write_to_memory(None);
    let mut bytes = vec![0; n];
    assert_eq!(n, region.write_to_memory(Some(&mut bytes)));
    bytes
}
fn words(values: &[i32]) -> Vec<u8> {
    values.iter().flat_map(|x| x.to_le_bytes()).collect()
}
fn two_rects() -> SkRegion {
    let mut region = SkRegion::new();
    assert!(region.set_rects(&[IRect::new(0, 0, 2, 2), IRect::new(4, 0, 6, 2)]));
    region
}
#[test]
fn supplemental_region_byte_order_and_transactional_decode() {
    // Fixed wire fixtures, independent of the encoder under test.
    let empty = vec![255, 255, 255, 255];
    let rect = vec![
        0, 0, 0, 0, 254, 255, 255, 255, 3, 0, 0, 0, 4, 3, 2, 1, 8, 7, 6, 5,
    ];
    let complex = words(&[9, 0, 0, 6, 2, 1, 2, 0, 2, 2, 0, 2, 4, 6, i32::MAX, i32::MAX]);
    let regions = [
        SkRegion::new(),
        SkRegion::from_irect(IRect::new(-2, 3, 0x01020304, 0x05060708)),
        two_rects(),
    ];
    for (region, fixture) in regions.iter().zip([empty, rect, complex]) {
        assert_eq!(encoded(region), fixture);
        for len in 0..fixture.len() {
            let mut dst = two_rects();
            let before = dst.clone();
            assert_eq!(0, dst.read_from_memory(&fixture[..len]), "truncation {len}");
            assert_eq!(dst, before);
        }
        let mut with_suffix = fixture.clone();
        with_suffix.extend_from_slice(&[0x81, 0x23, 0x45]);
        let mut dst = SkRegion::new();
        assert_eq!(fixture.len(), dst.read_from_memory(&with_suffix));
        assert_eq!(&dst, region);
        assert!(dst.is_valid());
    }
    // Invalid count, empty rect bounds, impossible allocation counts, invalid run counters,
    // broken x ordering and missing terminal sentinel must fail without modifying the target.
    let mut invalid = vec![
        words(&[-2]),
        words(&[0, 2, 0, 1, 1]),
        words(&[i32::MAX, 0, 0, 6, 2, 1, 2]),
    ];
    for (index, value) in [(5, 0), (6, -1), (9, 0), (10, 3), (15, 0)] {
        let mut w = [9, 0, 0, 6, 2, 1, 2, 0, 2, 2, 0, 2, 4, 6, i32::MAX, i32::MAX];
        w[index] = value;
        invalid.push(words(&w));
    }
    for fixture in invalid {
        let mut dst = two_rects();
        let before = dst.clone();
        assert_eq!(0, dst.read_from_memory(&fixture));
        assert_eq!(dst, before);
    }
}
#[test]
fn supplemental_region_boundaries_overflow_and_all_ops() {
    let mut region = SkRegion::new();
    for rect in [
        IRect::new(0, 0, 0, 1),
        IRect::new(2, 0, 1, 1),
        IRect::new(0, 0, i32::MAX, 1),
        IRect::new(0, 0, 1, i32::MAX),
    ] {
        assert!(!region.set_rect(&rect));
        assert!(region.is_empty());
        assert!(region.is_valid());
    }
    let rect = IRect::new(-3, -2, 4, 5);
    assert!(region.set_rect(&rect));
    for (x, y, expected) in [
        (-3, -2, true),
        (3, 4, true),
        (4, 0, false),
        (0, 5, false),
        (-4, 0, false),
        (0, -3, false),
    ] {
        assert_eq!(region.contains(x, y), expected);
    }
    assert!(!region.intersects(&IRect::new(4, -2, 5, 5)));
    assert!(!region.contains_irect(&IRect::new(0, 0, 0, 0)));
    // pin_offset_s32 pins against INT32_MIN, then SkIRect's saturating offset
    // clamps left/top to SK_MinS32 (INT32_MIN + 1), reducing the rect's extent.
    // At the positive limit, setRect rejects the reserved right/bottom sentinel.
    let mut lo = SkRegion::from_irect(IRect::new(
        i32::MIN + 2,
        i32::MIN + 3,
        i32::MIN + 12,
        i32::MIN + 13,
    ));
    lo.translate(-100, -100);
    assert_eq!(
        *lo.get_bounds(),
        IRect::new(i32::MIN + 1, i32::MIN + 1, i32::MIN + 10, i32::MIN + 10)
    );
    let mut hi = SkRegion::from_irect(IRect::new(i32::MAX - 12, 0, i32::MAX - 2, 10));
    hi.translate(100, 0);
    assert!(hi.is_empty());
    let large = SkRegion::from_irect(IRect::new(
        -1_000_000_000,
        -1_000_000_000,
        1_000_000_000,
        1_000_000_000,
    ));
    assert!(large.contains(999_999_999, 999_999_999));
    assert!(!large.contains(1_000_000_000, 0));
    let a = two_rects();
    let b = SkRegion::from_irect(IRect::new(1, -1, 5, 1));
    for op in [
        SkRegionOp::Difference,
        SkRegionOp::Intersect,
        SkRegionOp::Union,
        SkRegionOp::Xor,
        SkRegionOp::ReverseDifference,
        SkRegionOp::Replace,
    ] {
        let mut out = SkRegion::new();
        let nonempty = out.op(&a, &b, op);
        let mut any = false;
        for y in -2..4 {
            for x in -2..8 {
                let aa = a.contains(x, y);
                let bb = b.contains(x, y);
                let expected = match op {
                    SkRegionOp::Difference => aa && !bb,
                    SkRegionOp::Intersect => aa && bb,
                    SkRegionOp::Union => aa || bb,
                    SkRegionOp::Xor => aa ^ bb,
                    SkRegionOp::ReverseDifference => bb && !aa,
                    SkRegionOp::Replace => bb,
                };
                assert_eq!(out.contains(x, y), expected, "{op:?} ({x},{y})");
                any |= expected;
            }
        }
        assert_eq!(nonempty, any);
        assert!(out.is_valid());
        let mut inplace = a.clone();
        assert_eq!(inplace.op_region(&b, op), nonempty);
        assert_eq!(inplace, out);
    }
}
#[test]
fn supplemental_region_and_aa_copy_ownership() {
    let mut source = two_rects();
    let mut copy = source.clone();
    let fixture = encoded(&source);
    copy.translate(10, 20);
    assert_eq!(encoded(&source), fixture);
    assert!(copy.contains(10, 20));
    source.set_empty();
    drop(source);
    assert!(copy.contains(14, 20));
    assert!(!copy.contains(12, 20));
    let mut decoded = SkRegion::new();
    {
        let bytes = fixture.clone();
        assert_eq!(bytes.len(), decoded.read_from_memory(&bytes));
    }
    assert_eq!(encoded(&decoded), fixture);
    let mut aa = SkAAClip::new();
    assert!(aa.set_region(&decoded));
    let mut aa_copy = aa.clone();
    aa.set_empty();
    drop(aa);
    drop(decoded);
    // quickContains requires the inclusive lastY of this run (1) >= bottom.
    assert!(!aa_copy.quick_contains(&IRect::new(0, 0, 2, 2)));
    assert!(aa_copy.quick_contains(&IRect::new(0, 0, 2, 1)));
    aa_copy.set_empty();
    assert!(aa_copy.is_empty());
}
#[test]
fn supplemental_rtree_capacity_boundaries_and_input_lifetime() {
    for n in [
        0, 1, 5, 6, 7, 10, 11, 12, 16, 17, 21, 22, 23, 65, 66, 67, 120, 121, 122, 200,
    ] {
        let mut tree = SkRTree::new();
        {
            let mut rects = Vec::new();
            for i in 0..n {
                let x = i as f32 * 2.0;
                rects.push(Rect::new(x, 0.0, x + 1.0, 1.0));
            }
            tree.insert_rects(&rects);
            rects.fill(Rect::default()); // the tree must own bounds independent of caller storage
        }
        assert_eq!(tree.get_count(), n);
        let mut hits = Vec::new();
        tree.search(&Rect::new(-1.0, -1.0, 1000.0, 2.0), &mut hits);
        assert_eq!(hits, (0..n).collect::<Vec<_>>());
        for i in 0..n {
            let x = i as f32 * 2.0;
            let mut hits = Vec::new();
            tree.search(&Rect::new(x, 0.0, x + 1.0, 1.0), &mut hits);
            assert_eq!(hits, vec![i]);
            let mut edge = Vec::new();
            tree.search(&Rect::new(x + 1.0, 0.0, x + 2.0, 1.0), &mut edge);
            assert!(edge.is_empty());
        }
    }
    // Empty entries do not renumber subsequent leaves; metadata default forwarding is equivalent.
    let boxes = [
        Rect::default(),
        Rect::new(-10.0, -10.0, 0.0, 0.0),
        Rect::new(1.0, 1.0, 1.0, 2.0),
        Rect::new(0.0, 0.0, 10.0, 10.0),
    ];
    let mut tree = SkRTree::new();
    tree.insert_with_metadata(&boxes, &[Metadata { is_draw: false }; 4]);
    assert_eq!(tree.get_count(), 2);
    let mut hits = Vec::new();
    tree.search(&Rect::new(-20.0, -20.0, 20.0, 20.0), &mut hits);
    assert_eq!(hits, vec![1, 3]);
    let mut hierarchy = SkRTreeFactory.make_hierarchy().unwrap();
    Arc::get_mut(&mut hierarchy).unwrap().insert(&boxes);
    let survivor = hierarchy.clone();
    drop(hierarchy);
    let mut hits = Vec::new();
    survivor.search(&Rect::new(-20.0, -20.0, 20.0, 20.0), &mut hits);
    assert_eq!(hits, vec![1, 3]);
}
#[test]
fn supplemental_clip_stack_geometry_and_shader_lifetime() {
    let mut stack = SkClipStack::new();
    {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        stack.clip_rect(&rect, &Matrix::default(), SkClipOp::Intersect, false);
    }
    stack.save();
    let snapshot = stack.clone();
    stack.clip_empty();
    assert!(!stack.is_wide_open());
    stack.restore();
    assert!(stack.quick_contains(&Rect::new(1.0, 1.0, 99.0, 99.0)));
    drop(stack);
    assert_eq!(snapshot.get_save_count(), 1);
    assert!(snapshot.quick_contains(&Rect::new(1.0, 1.0, 99.0, 99.0)));
    struct Payload(Arc<AtomicUsize>);
    impl Drop for Payload {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let mut stack = SkClipStack::new();
    let shader = SkShaderHandle::new(Payload(drops.clone()));
    stack.save();
    stack.clip_shader(shader.clone());
    let copy = stack.clone();
    drop(shader);
    stack.restore();
    drop(stack);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(copy);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
