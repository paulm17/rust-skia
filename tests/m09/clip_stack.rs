// Copyright 2011 Google Inc. BSD: tests/LICENSE.skia.
// Faithful translation of tests/ClipStackTest.cpp, registration ClipStack:828.
use crate::support::{Reporter, reporter_assert as check};
use SkClipOp::{Difference, Intersect};
use rust_skia::{
    matrix::Matrix,
    rect::{IRect, Rect},
    rrect::RRect,
    sk_clip_op::SkClipOp,
    sk_clip_stack::{
        B2TIter, DeviceSpaceType, Iter, IterStart, K_EMPTY_GEN_ID, K_WIDE_OPEN_GEN_ID, SkClipStack,
    },
    sk_path::SkPath,
    sk_path_builder::SkPathBuilder,
    sk_path_types::{SkPathDirection, SkPathFillType},
    sk_region::{BoundsType, SkRegion, SkRegionOp},
};
fn r(l: f32, t: f32, rr: f32, b: f32) -> Rect {
    Rect::new(l, t, rr, b)
}
fn cr(s: &mut SkClipStack, rect: Rect, op: SkClipOp, aa: bool) {
    s.clip_rect(&rect, &Matrix::default(), op, aa);
}
fn cp(s: &mut SkClipStack, path: &SkPath, op: SkClipOp, aa: bool) {
    s.clip_path(path, &Matrix::default(), op, aa);
}
fn circle(x: f32, y: f32, radius: f32) -> SkPath {
    SkPath::circle(x, y, radius, SkPathDirection::Cw)
}
fn assert_count(reporter: &mut Reporter, stack: &SkClipStack, count: usize) {
    let mut iter = B2TIter::new(stack);
    let mut counter = 0;
    while iter.next().is_some() {
        counter += 1;
    }
    check!(reporter, count == counter);
}
fn count(stack: &SkClipStack) -> usize {
    let mut iter = Iter::new(stack, IterStart::Top);
    let mut count = 0;
    while iter.prev().is_some() {
        count += 1;
    }
    count
}
fn test_assign_and_comparison(reporter: &mut Reporter) {
    let mut s = SkClipStack::new();
    let doAA = false;
    check!(reporter, 0 == s.get_save_count());
    s.save();
    check!(reporter, 1 == s.get_save_count());
    let mut builder = SkPathBuilder::new();
    builder.move_to_xy(5.0, 6.0);
    builder.line_to_xy(7.0, 8.0);
    builder.line_to_xy(5.0, 9.0);
    builder.close();
    cp(&mut s, &builder.snapshot(None), Intersect, doAA);
    s.save();
    check!(reporter, 2 == s.get_save_count());
    let mut rect = r(1.0, 2.0, 103.0, 104.0);
    cr(&mut s, rect, Intersect, doAA);
    rect = r(4.0, 5.0, 56.0, 57.0);
    cr(&mut s, rect, Intersect, doAA);
    s.save();
    check!(reporter, 3 == s.get_save_count());
    rect = r(14.0, 15.0, 16.0, 17.0);
    cr(&mut s, rect, Difference, doAA);
    let mut copy = s.clone();
    check!(reporter, s == copy);
    s.restore();
    check!(reporter, 2 == s.get_save_count());
    check!(reporter, s != copy);
    s.save();
    check!(reporter, 3 == s.get_save_count());
    rect = r(14.0, 15.0, 16.0, 17.0);
    cr(&mut s, rect, Difference, doAA);
    check!(reporter, s == copy);
    s.restore();
    check!(reporter, 2 == s.get_save_count());
    s.save();
    check!(reporter, 3 == s.get_save_count());
    rect = r(14.0, 15.0, 16.0, 17.0);
    cr(&mut s, rect, Intersect, doAA);
    check!(reporter, s != copy);
    s.restore();
    s.save();
    let rp = SkPath::rect_default(&rect);
    cp(&mut s, &rp, Difference, doAA);
    check!(reporter, s == copy);
    s.restore();
    check!(reporter, 2 == s.get_save_count());
    s.save();
    check!(reporter, 3 == s.get_save_count());
    rect = r(24.0, 25.0, 26.0, 27.0);
    cr(&mut s, rect, Difference, doAA);
    check!(reporter, s != copy);
    s.restore();
    check!(reporter, 2 == s.get_save_count());
    copy.restore();
    check!(reporter, 2 == copy.get_save_count());
    check!(reporter, s == copy);
    s.restore();
    check!(reporter, 1 == s.get_save_count());
    copy.restore();
    check!(reporter, 1 == copy.get_save_count());
    check!(reporter, s == copy);
    s.restore();
    check!(reporter, 0 == s.get_save_count());
    s.save();
    check!(reporter, 1 == s.get_save_count());
    builder.add_rect(&rect, SkPathDirection::Cw, 0);
    cp(&mut s, &builder.detach(None), Intersect, doAA);
    check!(reporter, s != copy);
}
fn test_iterators(reporter: &mut Reporter) {
    let mut stack = SkClipStack::new();
    let rects = [
        r(0.0, 0.0, 40.0, 40.0),
        r(60.0, 0.0, 100.0, 40.0),
        r(0.0, 60.0, 40.0, 100.0),
        r(60.0, 60.0, 100.0, 100.0),
    ];
    for rect in rects {
        cr(&mut stack, rect, Difference, false);
    }
    assert_count(reporter, &stack, 4);
    {
        let mut iter = B2TIter::new(&stack);
        let mut i = 0;
        while let Some(element) = iter.next() {
            check!(
                reporter,
                DeviceSpaceType::Rect == element.get_device_space_type()
            );
            check!(reporter, *element.get_device_space_rect() == rects[i]);
            i += 1;
        }
        debug_assert!(i == 4);
    }
    {
        let mut iter = Iter::new(&stack, IterStart::Top);
        let mut i: i32 = 3;
        while let Some(element) = iter.prev() {
            check!(
                reporter,
                DeviceSpaceType::Rect == element.get_device_space_type()
            );
            check!(
                reporter,
                *element.get_device_space_rect() == rects[i as usize]
            );
            i -= 1;
        }
        debug_assert!(i == -1);
    }
    let mut iter = Iter::new(&stack, IterStart::Bottom);
    let element = iter
        .skip_to_topmost(Difference)
        .expect("original dereferences element");
    check!(
        reporter,
        DeviceSpaceType::Rect == element.get_device_space_type()
    );
    check!(reporter, *element.get_device_space_rect() == rects[3]);
}
fn test_bounds(reporter: &mut Reporter, primType: DeviceSpaceType) {
    let answers = [
        r(40.0, 40.0, 50.0, 50.0),
        r(10.0, 10.0, 50.0, 50.0),
        r(40.0, 40.0, 80.0, 80.0),
        r(0.0, 0.0, 100.0, 100.0),
        r(10.0, 10.0, 50.0, 50.0),
        r(40.0, 40.0, 50.0, 50.0),
        r(0.0, 0.0, 100.0, 100.0),
        r(40.0, 40.0, 80.0, 80.0),
    ];
    let ops = [Intersect, Difference];
    let rectA = r(10.0, 10.0, 50.0, 50.0);
    let rectB = r(40.0, 40.0, 80.0, 80.0);
    let rrectA = RRect::make_oval(&rectA);
    let rrectB = RRect::make_rect_xy(&rectB, 1.0, 2.0);
    let mut pathA = SkPath::rrect_xy(&rectA, 5.0, 5.0, SkPathDirection::Cw);
    let mut pathB = SkPath::rrect_xy(&rectB, 5.0, 5.0, SkPathDirection::Cw);
    let mut stack = SkClipStack::new();
    let mut bound = Rect::default();
    let mut intersection = false;
    let mut testCase = 0;
    let numBitTests = if primType == DeviceSpaceType::Path {
        4
    } else {
        1
    };
    for invBits in 0..numBitTests {
        for op in ops {
            stack.save();
            pathA.set_fill_type(if invBits & 1 != 0 {
                SkPathFillType::InverseEvenOdd
            } else {
                SkPathFillType::EvenOdd
            });
            pathB.set_fill_type(if invBits & 2 != 0 {
                SkPathFillType::InverseEvenOdd
            } else {
                SkPathFillType::EvenOdd
            });
            match primType {
                DeviceSpaceType::Rect => {
                    cr(&mut stack, rectA, Intersect, false);
                    cr(&mut stack, rectB, op, false);
                }
                DeviceSpaceType::RRect => {
                    stack.clip_rrect(&rrectA, &Matrix::default(), Intersect, false);
                    stack.clip_rrect(&rrectB, &Matrix::default(), op, false);
                }
                DeviceSpaceType::Path => {
                    cp(&mut stack, &pathA, Intersect, false);
                    cp(&mut stack, &pathB, op, false);
                }
                _ => panic!("Don't call this with Empty or Shader"),
            }
            check!(reporter, !stack.is_wide_open());
            check!(reporter, K_WIDE_OPEN_GEN_ID != stack.get_topmost_gen_id());
            stack.get_conservative_bounds(0, 0, 100, 100, &mut bound, Some(&mut intersection));
            if primType == DeviceSpaceType::Rect {
                check!(reporter, intersection == (op == Intersect));
            } else {
                check!(reporter, !intersection);
            }
            debug_assert!(testCase < 8);
            check!(reporter, bound == answers[testCase]);
            testCase += 1;
            stack.restore();
        }
    }
}
fn test_isWideOpen(reporter: &mut Reporter) {
    {
        let stack = SkClipStack::new();
        check!(reporter, stack.is_wide_open());
        check!(reporter, K_WIDE_OPEN_GEN_ID == stack.get_topmost_gen_id());
    }
    let rectA = r(10.0, 10.0, 40.0, 40.0);
    let _rectB = r(50.0, 50.0, 80.0, 80.0);
    {
        let stack = SkClipStack::new();
        check!(reporter, stack.is_wide_open());
        check!(reporter, K_WIDE_OPEN_GEN_ID == stack.get_topmost_gen_id());
    }
    {
        let mut stack = SkClipStack::new();
        cr(&mut stack, Rect::default(), Difference, false);
        check!(reporter, stack.is_wide_open());
        check!(reporter, K_WIDE_OPEN_GEN_ID == stack.get_topmost_gen_id());
    }
    {
        let mut stack = SkClipStack::new();
        stack.save();
        cr(&mut stack, rectA, Intersect, false);
        check!(reporter, !stack.is_wide_open());
        check!(reporter, K_WIDE_OPEN_GEN_ID != stack.get_topmost_gen_id());
        stack.restore();
        check!(reporter, stack.is_wide_open());
        check!(reporter, K_WIDE_OPEN_GEN_ID == stack.get_topmost_gen_id());
    }
}
fn test_rect_inverse_fill(reporter: &mut Reporter) {
    let rect = r(0.0, 0.0, 10.0, 10.0);
    let mut path = SkPath::rect_default(&rect);
    path.toggle_inverse_fill_type();
    let mut stack = SkClipStack::new();
    cp(&mut stack, &path, Intersect, false);
    let mut bounds = Rect::default();
    let mut kind = BoundsType::Normal;
    stack.get_bounds(&mut bounds, &mut kind);
    check!(reporter, BoundsType::InsideOut == kind);
    check!(reporter, bounds == rect);
}
fn test_rect_replace(reporter: &mut Reporter) {
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let rect2 = r(50.0, 50.0, 150.0, 150.0);
    let mut bound = Rect::default();
    let mut kind = BoundsType::Normal;
    let mut intersection = false;
    for aa in [false, true] {
        let mut stack = SkClipStack::new();
        check!(reporter, 0 == count(&stack));
        stack.replace_clip(&rect, aa);
        check!(reporter, 1 == count(&stack));
        stack.replace_clip(&rect, aa);
        check!(reporter, 1 == count(&stack));
    }
    {
        let mut stack = SkClipStack::new();
        check!(reporter, 0 == count(&stack));
        stack.replace_clip(&rect, false);
        check!(reporter, 1 == count(&stack));
        stack.replace_clip(&rect, true);
        check!(reporter, 1 == count(&stack));
        stack.replace_clip(&rect, false);
        check!(reporter, 1 == count(&stack));
    }
    let mut stack = SkClipStack::new();
    stack.replace_clip(&rect, false);
    cr(&mut stack, rect2, Intersect, false);
    check!(reporter, 1 == count(&stack));
    stack.save();
    stack.replace_clip(&rect, false);
    check!(reporter, 2 == count(&stack));
    stack.get_bounds_with_intersection(&mut bound, &mut kind, &mut intersection);
    check!(reporter, bound == rect);
    stack.restore();
    check!(reporter, 1 == count(&stack));
    stack.save();
    stack.replace_clip(&rect, false);
    stack.replace_clip(&rect, false);
    check!(reporter, 2 == count(&stack));
    stack.restore();
    check!(reporter, 1 == count(&stack));
    stack.save();
    stack.replace_clip(&rect, false);
    cr(&mut stack, rect2, Intersect, false);
    stack.replace_clip(&rect, false);
    check!(reporter, 2 == count(&stack));
    stack.restore();
    check!(reporter, 1 == count(&stack));
}
fn test_path_replace(reporter: &mut Reporter) {
    let replacePath = |stack: &mut SkClipStack, path: &SkPath, aa| {
        stack.replace_clip(&r(-1000.0, -1000.0, 1000.0, 1000.0), false);
        cp(stack, path, Intersect, aa);
    };
    let rect = r(0.0, 0.0, 100.0, 100.0);
    let path = circle(50.0, 50.0, 50.0);
    {
        let mut stack = SkClipStack::new();
        check!(reporter, 0 == count(&stack));
        replacePath(&mut stack, &path, false);
        check!(reporter, 2 == count(&stack));
        replacePath(&mut stack, &path, false);
        check!(reporter, 2 == count(&stack));
    }
    {
        let mut stack = SkClipStack::new();
        stack.replace_clip(&rect, true);
        check!(reporter, 1 == count(&stack));
        replacePath(&mut stack, &path, true);
        check!(reporter, 2 == count(&stack));
    }
}
fn test_rect_merging(reporter: &mut Reporter) {
    let overlapLeft = r(10.0, 10.0, 50.0, 50.0);
    let overlapRight = r(40.0, 40.0, 80.0, 80.0);
    let nestedParent = r(10.0, 10.0, 90.0, 90.0);
    let nestedChild = r(40.0, 40.0, 60.0, 60.0);
    let mut bound = Rect::default();
    let mut kind = BoundsType::Normal;
    let mut intersection = false;
    // Six original independent scopes, in their original order.
    for (a, b, aa, bb, n, expected) in [
        (overlapLeft, overlapRight, false, false, 1, true),
        (overlapLeft, overlapRight, true, true, 1, true),
        (overlapLeft, overlapRight, true, false, 2, false),
        (nestedParent, nestedChild, true, false, 1, true),
        (nestedParent, nestedChild, false, true, 1, true),
        (nestedChild, nestedParent, false, true, 2, false),
    ] {
        let mut stack = SkClipStack::new();
        cr(&mut stack, a, Intersect, aa);
        cr(&mut stack, b, Intersect, bb);
        check!(reporter, n == count(&stack));
        stack.get_bounds_with_intersection(&mut bound, &mut kind, &mut intersection);
        check!(reporter, intersection == expected);
    }
}
fn test_quickContains(reporter: &mut Reporter) {
    let testRect = r(10.0, 10.0, 40.0, 40.0);
    let insideRect = r(20.0, 20.0, 30.0, 30.0);
    let intersectingRect = r(25.0, 25.0, 50.0, 50.0);
    let outsideRect = r(0.0, 0.0, 50.0, 50.0);
    let nonIntersectingRect = r(100.0, 100.0, 110.0, 110.0);
    let insideCircle = circle(25.0, 25.0, 5.0);
    let intersectingCircle = circle(25.0, 40.0, 10.0);
    let outsideCircle = circle(25.0, 25.0, 50.0);
    let nonIntersectingCircle = circle(100.0, 100.0, 5.0);
    {
        let mut stack = SkClipStack::new();
        cr(&mut stack, outsideRect, Difference, false);
        check!(reporter, !stack.quick_contains(&testRect));
    }
    {
        let mut stack = SkClipStack::new();
        stack.replace_clip(&outsideRect, false);
        check!(reporter, stack.quick_contains(&testRect));
    }
    {
        let mut stack = SkClipStack::new();
        cr(&mut stack, insideRect, Intersect, false);
        stack.save();
        stack.replace_clip(&outsideRect, false);
        check!(reporter, stack.quick_contains(&testRect));
        stack.restore();
    }
    {
        let mut stack = SkClipStack::new();
        cr(&mut stack, outsideRect, Intersect, false);
        stack.save();
        stack.replace_clip(&insideRect, false);
        check!(reporter, !stack.quick_contains(&testRect));
        stack.restore();
    }
    {
        let mut stack = SkClipStack::new();
        cr(&mut stack, insideRect, Intersect, false);
        cp(&mut stack, &outsideCircle, Intersect, false);
        check!(reporter, !stack.quick_contains(&testRect));
    }
    for (rect, expected) in [
        (outsideRect, true),
        (insideRect, false),
        (intersectingRect, false),
        (nonIntersectingRect, false),
    ] {
        let mut stack = SkClipStack::new();
        cr(&mut stack, rect, Intersect, false);
        check!(reporter, expected == stack.quick_contains(&testRect));
    }
    for (path, expected) in [
        (&outsideCircle, true),
        (&insideCircle, false),
        (&intersectingCircle, false),
        (&nonIntersectingCircle, false),
    ] {
        let mut stack = SkClipStack::new();
        cp(&mut stack, path, Intersect, false);
        check!(reporter, expected == stack.quick_contains(&testRect));
    }
    for (rect, expected) in [
        (outsideRect, false),
        (insideRect, false),
        (intersectingRect, false),
        (nonIntersectingRect, true),
    ] {
        let mut stack = SkClipStack::new();
        let mut path = SkPath::rect_default(&rect);
        path.toggle_inverse_fill_type();
        cp(&mut stack, &path, Intersect, false);
        check!(reporter, expected == stack.quick_contains(&testRect));
    }
    for (source, expected) in [
        (&outsideCircle, false),
        (&insideCircle, false),
        (&intersectingCircle, false),
        (&nonIntersectingCircle, true),
    ] {
        let mut stack = SkClipStack::new();
        let mut path = source.clone();
        path.toggle_inverse_fill_type();
        cp(&mut stack, &path, Intersect, false);
        check!(reporter, expected == stack.quick_contains(&testRect));
    }
}
fn set_region_to_stack(stack: &SkClipStack, bounds: &IRect, region: &mut SkRegion) {
    region.set_rect(bounds);
    let mut iter = Iter::new(stack, IterStart::Bottom);
    while let Some(element) = iter.next() {
        let mut elemRegion = SkRegion::new();
        let boundsRgn = SkRegion::from_irect(*bounds);
        match element.get_device_space_type() {
            DeviceSpaceType::Empty => {
                elemRegion.set_empty();
            }
            _ => {
                elemRegion.set_path(&element.as_device_space_path(), &boundsRgn);
            }
        }
        region.op_region(
            &elemRegion,
            if element.is_replace_op() {
                SkRegionOp::Replace
            } else {
                element.get_op().into()
            },
        );
    }
}
fn test_invfill_diff_bug(reporter: &mut Reporter) {
    let mut stack = SkClipStack::new();
    cr(&mut stack, r(10.0, 10.0, 20.0, 20.0), Intersect, false);
    let mut path = SkPath::rect_default(&r(30.0, 10.0, 40.0, 20.0));
    path.set_fill_type(SkPathFillType::InverseWinding);
    cp(&mut stack, &path, Difference, false);
    check!(reporter, K_EMPTY_GEN_ID == stack.get_topmost_gen_id());
    let mut bounds = Rect::default();
    let mut kind = BoundsType::Normal;
    stack.get_bounds(&mut bounds, &mut kind);
    check!(reporter, bounds.is_empty());
    check!(reporter, BoundsType::Normal == kind);
    let mut region = SkRegion::new();
    set_region_to_stack(&stack, &IRect::new(0, 0, 50, 30), &mut region);
    check!(reporter, region.is_empty());
}
fn test_is_rrect_deep_rect_stack(reporter: &mut Reporter) {
    let target = r(0.0, 0.0, 1000.0, 500.0);
    for aa in [false, true] {
        let mut stack = SkClipStack::new();
        for i in 0..=100 {
            stack.save();
            cr(
                &mut stack,
                r(i as f32, 0.5, target.width(), target.height()),
                Intersect,
                aa,
            );
        }
        let mut rrect = RRect::default();
        let mut isAA = false;
        let expected = RRect::make_rect(&r(100.0, 0.5, target.width(), target.height()));
        if stack.is_rrect(&target, &mut rrect, &mut isAA) {
            check!(reporter, rrect == expected);
            check!(reporter, aa == isAA);
        } else {
            check!(reporter, false);
        }
    }
    let mut stack = SkClipStack::new();
    for i in 0..=100 {
        let aa = i & 1 != 0;
        let j = 100 - i;
        stack.save();
        cr(
            &mut stack,
            r(i as f32, j as f32 + 0.5, target.width(), target.height()),
            Intersect,
            aa,
        );
    }
    let mut rrect = RRect::default();
    let mut isAA = false;
    check!(reporter, !stack.is_rrect(&target, &mut rrect, &mut isAA));
}
#[test]
fn ClipStack() {
    let mut reporter = Reporter::default();
    let mut stack = SkClipStack::new();
    check!(reporter, 0 == stack.get_save_count());
    assert_count(&mut reporter, &stack, 0);
    let rects = [
        IRect::new(0, 0, 100, 100),
        IRect::new(25, 25, 125, 125),
        IRect::new(0, 0, 1000, 1000),
        IRect::new(0, 0, 75, 75),
    ];
    for rect in rects {
        stack.clip_dev_rect(&rect, Intersect);
    }
    {
        let mut iter = B2TIter::new(&stack);
        let element = iter.next();
        let answer = r(25.0, 25.0, 75.0, 75.0);
        check!(reporter, element.is_some());
        let element = element.expect("original dereferences element");
        check!(
            reporter,
            DeviceSpaceType::Rect == element.get_device_space_type()
        );
        check!(reporter, Intersect == element.get_op());
        check!(reporter, *element.get_device_space_rect() == answer);
        check!(reporter, iter.next().is_none());
    }
    stack.reset();
    check!(reporter, 0 == stack.get_save_count());
    assert_count(&mut reporter, &stack, 0);
    test_assign_and_comparison(&mut reporter);
    test_iterators(&mut reporter);
    test_bounds(&mut reporter, DeviceSpaceType::Rect);
    test_bounds(&mut reporter, DeviceSpaceType::RRect);
    test_bounds(&mut reporter, DeviceSpaceType::Path);
    test_isWideOpen(&mut reporter);
    test_rect_merging(&mut reporter);
    test_rect_replace(&mut reporter);
    test_rect_inverse_fill(&mut reporter);
    test_path_replace(&mut reporter);
    test_quickContains(&mut reporter);
    test_invfill_diff_bug(&mut reporter);
    test_is_rrect_deep_rect_stack(&mut reporter);
    reporter.finish();
}
