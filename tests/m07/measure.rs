// Copyright 2011–2025 Google Inc. / Google LLC. BSD: tests/LICENSE.skia.
// PathMeasureTest.cpp, complete helper closure.
use super::*;
fn test_small_segment3(r: &mut Reporter) {
    let pts = [
        p(0.0, 0.0),
        p(100000000000.0, 100000000000.0),
        p(0.0, 0.0),
        p(10.0, 10.0),
        p(10.0, 10.0),
        p(0.0, 0.0),
        p(10.0, 10.0),
    ];
    let mut b = SkPathBuilder::new();
    b.move_to(pts[0]);
    for i in (1..pts.len()).step_by(3) {
        b.cubic_to(pts[i], pts[i + 1], pts[i + 2]);
    }
    let path = b.detach(None);
    let meas = SkPathMeasure::from_path(&path, false, 1.0);
    let _ = meas.get_length();
    let n = SkPathMeasurePriv::count_segments(&meas);
    check!(r, n < 300);
    let n = SkPathMeasurePriv::count_segments(&SkPathMeasure::from_path(&path, false, 1000.0));
    check!(r, n < 300);
}
fn test_small_segment2() {
    let pts = [
        p(0.0, 0.0),
        p(100000000000.0, 100000000000.0),
        p(0.0, 0.0),
        p(10.0, 10.0),
        p(0.0, 0.0),
    ];
    let mut b = SkPathBuilder::new();
    b.move_to(pts[0]);
    for i in (1..pts.len()).step_by(2) {
        b.quad_to(pts[i], pts[i + 1]);
    }
    let meas = SkPathMeasure::from_path(&b.detach(None), false, 1.0);
    let _ = meas.get_length();
}
fn test_small_segment() {
    let pts = [p(100000.0, 100000.0), p(1.0005, 0.9999), p(1.0, 1.0)];
    let meas = SkPathMeasure::from_path(&SkPath::polygon(&pts, false, Winding, false), false, 1.0);
    let _ = meas.get_length();
}
#[test]
fn PathMeasure() {
    let mut r = Reporter::default();
    let mut path = SkPathBuilder::new()
        .move_to_xy(0.0, 0.0)
        .line_to_xy(1.0, 0.0)
        .line_to_xy(1.0, 1.0)
        .line_to_xy(0.0, 1.0)
        .detach(None);
    let mut meas = SkPathMeasure::from_path(&path, true, 1.0);
    let mut length = meas.get_length();
    debug_assert!(length == 4.0);
    path = SkPath::line(p(0.0, 0.0), p(3.0, 4.0));
    meas.set_path(Some(&path), false);
    length = meas.get_length();
    check!(r, length == 5.0);
    path = SkPath::circle(0.0, 0.0, 1.0, Cw);
    meas.set_path(Some(&path), true);
    length = meas.get_length();
    path = SkPathBuilder::new()
        .line_to_xy(1.0, 0.0)
        .line_to_xy(1.0, 1.0)
        .line_to_xy(0.0, 1.0)
        .close()
        .line_to_xy(-1.0, 0.0)
        .detach(None);
    meas.set_path(Some(&path), false);
    length = meas.get_length();
    check!(r, length == 1.0 * 4.0);
    meas.next_contour();
    length = meas.get_length();
    check!(r, length == 1.0);
    let mut pos = Point::default();
    let mut tan = Point::default();
    check!(r, meas.get_pos_tan(0.5, Some(&mut pos), Some(&mut tan)));
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.x, -0.5, 0.0001)
    );
    check!(r, pos.y == 0.0);
    check!(r, tan.x == -1.0);
    check!(r, tan.y == 0.0);
    path = SkPathBuilder::new()
        .move_to_xy(0.0, 0.0)
        .line_to_xy(0.0, 0.0)
        .line_to_xy(1.0, 0.0)
        .quad_to_xy(1.0, 0.0, 1.0, 0.0)
        .quad_to_xy(1.0, 1.0, 1.0, 2.0)
        .cubic_to_xy(1.0, 2.0, 1.0, 2.0, 1.0, 2.0)
        .cubic_to_xy(2.0, 2.0, 3.0, 2.0, 4.0, 2.0)
        .detach(None);
    meas.set_path(Some(&path), false);
    length = meas.get_length();
    check!(r, length == 1.0 * 6.0);
    check!(r, meas.get_pos_tan(0.5, Some(&mut pos), Some(&mut tan)));
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.x, 0.5, 0.0001)
    );
    check!(r, pos.y == 0.0);
    check!(r, tan.x == 1.0);
    check!(r, tan.y == 0.0);
    check!(r, meas.get_pos_tan(2.5, Some(&mut pos), Some(&mut tan)));
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.x, 1.0, 0.0001)
    );
    check!(r, near(pos.y, 1.5));
    check!(r, tan.x == 0.0);
    check!(r, tan.y == 1.0);
    check!(r, meas.get_pos_tan(4.5, Some(&mut pos), Some(&mut tan)));
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.x, 2.5, 0.0001)
    );
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.y, 2.0, 0.0001)
    );
    check!(r, tan.x == 1.0);
    check!(r, tan.y == 0.0);
    path = SkPathBuilder::new()
        .move_to_xy(0.0, 0.0)
        .line_to_xy(1.0, 0.0)
        .move_to_xy(1.0, 1.0)
        .move_to_xy(2.0, 2.0)
        .line_to_xy(1.0, 2.0)
        .detach(None);
    meas.set_path(Some(&path), false);
    length = meas.get_length();
    check!(r, length == 1.0);
    check!(r, meas.get_pos_tan(0.5, Some(&mut pos), Some(&mut tan)));
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.x, 0.5, 0.0001)
    );
    check!(r, pos.y == 0.0);
    check!(r, tan.x == 1.0);
    check!(r, tan.y == 0.0);
    meas.next_contour();
    length = meas.get_length();
    check!(r, length == 1.0);
    check!(r, meas.get_pos_tan(0.5, Some(&mut pos), Some(&mut tan)));
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.x, 1.5, 0.0001)
    );
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(pos.y, 2.0, 0.0001)
    );
    check!(r, tan.x == -1.0);
    check!(r, tan.y == 0.0);
    test_small_segment();
    test_small_segment2();
    test_small_segment3(&mut r);
    let meas2 = meas;
    meas = meas2;
    let _ = meas;
    r.finish();
}
#[test]
fn PathMeasureConic() {
    let mut r = Reporter::default();
    let mut stdp = Point::default();
    let mut hip = Point::default();
    let pts = [p(0.0, 0.0), p(100.0, 0.0), p(100.0, 0.0)];
    let mut path = SkPathBuilder::new()
        .move_to_xy(0.0, 0.0)
        .conic_to(pts[1], pts[2], 1.0)
        .detach(None);
    let mut meas = SkPathMeasure::from_path(&path, false, 1.0);
    check!(r, meas.get_pos_tan(20.0, Some(&mut stdp), None));
    path = SkPathBuilder::new()
        .move_to_xy(0.0, 0.0)
        .conic_to(pts[1], pts[2], 10.0)
        .detach(None);
    meas.set_path(Some(&path), false);
    check!(r, meas.get_pos_tan(20.0, Some(&mut hip), None));
    check!(r, 19.5 < stdp.x && stdp.x < 20.5);
    check!(r, 19.5 < hip.x && hip.x < 20.5);
    r.finish();
}
#[test]
fn PathMeasure_nextctr() {
    let mut r = Reporter::default();
    let path = SkPath::line(p(0.0, 0.0), p(100.0, 0.0));
    let mut meas = SkPathMeasure::from_path(&path, false, 1.0);
    check!(r, !meas.next_contour());
    r.finish();
}
fn test_90_degrees(cm: &SkContourMeasure, radius: f32, r: &mut Reporter) {
    let mut pos = Point::default();
    let mut tan = Point::default();
    let distance = cm.length() / 4.0;
    let success = cm.get_pos_tan(distance, Some(&mut pos), Some(&mut tan));
    check!(r, success);
    check!(r, near(pos.x, 0.0));
    check!(r, near(pos.y, radius));
    check!(r, near(tan.x, -1.0));
    check!(r, near(tan.y, 0.0));
}
fn test_empty_contours(r: &mut Reporter) {
    let path = SkPathBuilder::new()
        .move_to_xy(0.0, 0.0)
        .line_to_xy(100.0, 100.0)
        .line_to_xy(200.0, 100.0)
        .move_to_xy(2.0, 2.0)
        .move_to_xy(3.0, 3.0)
        .move_to_xy(4.0, 4.0)
        .close()
        .close()
        .close()
        .move_to_xy(5.0, 5.0)
        .line_to_xy(5.0, 5.0)
        .move_to_xy(5.0, 5.0)
        .line_to_xy(5.0, 5.0)
        .close()
        .move_to_xy(5.0, 5.0)
        .line_to_xy(5.0, 5.0)
        .close()
        .close()
        .move_to_xy(6.0, 6.0)
        .line_to_xy(7.0, 7.0)
        .move_to_xy(10.0, 10.0)
        .detach(None);
    let mut fact = SkContourMeasureIter::from_path(&path, false, 1.0);
    check!(r, fact.next().is_some());
    check!(r, fact.next().is_some());
    check!(r, fact.next().is_none());
}
fn test_MLM_contours(r: &mut Reporter) {
    let path = SkPathBuilder::new()
        .move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 20.0)
        .move_to_xy(30.0, 30.0)
        .detach(None);
    for closed in [false, true] {
        let mut fact = SkContourMeasureIter::from_path(&path, closed, 1.0);
        check!(r, fact.next().is_some());
        check!(r, fact.next().is_none());
    }
}
#[test]
fn contour_measure() {
    let mut r = Reporter::default();
    let path = SkPathBuilder::new()
        .add_circle(p(0.0, 0.0), 100.0, Cw)
        .add_circle(p(0.0, 0.0), 10.0, Cw)
        .detach(None);
    let mut fact = SkContourMeasureIter::from_path(&path, false, 1.0);
    let cm0 = fact.next().unwrap();
    let cm1 = fact.next().unwrap();
    check!(r, cm0.is_closed());
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(
            cm0.length(),
            200.0 * rust_skia::scalar::SCALAR_PI,
            1.5
        )
    );
    test_90_degrees(&cm0, 100.0, &mut r);
    check!(r, cm1.is_closed());
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(
            cm1.length(),
            20.0 * rust_skia::scalar::SCALAR_PI,
            0.5
        )
    );
    test_90_degrees(&cm1, 10.0, &mut r);
    let cm2 = fact.next();
    check!(r, cm2.is_none());
    test_empty_contours(&mut r);
    test_MLM_contours(&mut r);
    r.finish();
}
#[test]
fn contour_measure_verbs() {
    let mut r = Reporter::default();
    let path = SkPathBuilder::new()
        .move_to_xy(10.0, 10.0)
        .line_to_xy(10.0, 30.0)
        .line_to_xy(30.0, 30.0)
        .quad_to(p(40.0, 30.0), p(40.0, 40.0))
        .cubic_to(p(50.0, 40.0), p(50.0, 50.0), p(40.0, 50.0))
        .conic_to(p(50.0, 50.0), p(50.0, 60.0), 1.2)
        .detach(None);
    let mut measure = SkContourMeasureIter::from_path(&path, false, 1.0);
    let cm = measure.next();
    check!(r, cm.is_some());
    let cm = cm.unwrap();
    let mut it = cm.iter();
    // Each row is one original iterator block, including its point count and every point.
    let expected = [
        (L, 20.0, vec![p(10.0, 10.0), p(10.0, 30.0)]),
        (L, 40.0, vec![p(10.0, 30.0), p(30.0, 30.0)]),
        (
            Q,
            56.127525,
            vec![p(30.0, 30.0), p(40.0, 30.0), p(40.0, 40.0)],
        ),
        (
            C,
            76.004692,
            vec![p(40.0, 40.0), p(50.0, 40.0), p(50.0, 50.0), p(40.0, 50.0)],
        ),
        (
            K,
            92.428185,
            vec![p(40.0, 50.0), p(1.2, 0.0), p(50.0, 50.0), p(50.0, 60.0)],
        ),
    ];
    for (i, (verb, distance, pts)) in expected.iter().enumerate() {
        let vm = it.next();
        check!(r, vm.is_some());
        let vm = vm.unwrap();
        check!(r, vm.f_verb == *verb);
        check!(r, near(vm.f_distance, *distance));
        check!(r, vm.f_pts.len() == pts.len());
        for j in 0..pts.len() {
            check!(r, vm.f_pts[j] == pts[j]);
        }
        if i == 4 {
            check!(r, near(vm.f_distance, cm.length()));
        }
    }
    check!(r, it.next().is_none());
    let mut current = 0.0;
    let mut count = 0;
    for vm in cm.iter() {
        check!(r, vm.f_distance > current);
        current = vm.f_distance;
        count += 1;
    }
    check!(r, count == 5);
    r.finish();
}
