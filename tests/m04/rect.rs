// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// RectTest.cpp:119,133,163,182,255,398. Other registrations remain M05/M07/M14.
#[test]
fn Rect_largest() {
    let mut r = Reporter::default();
    check!(r, !RectPriv::make_i_large().is_empty());
    check!(r, RectPriv::make_i_largest_inverted().is_empty());
    check!(r, !RectPriv::make_largest().is_empty());
    check!(r, !RectPriv::make_large_s32().is_empty());
    check!(r, RectPriv::make_largest_inverted().is_empty());
    r.finish();
}
#[test]
fn Rect_setbounds() {
    let p0 = [
        Point::new(f32::INFINITY, 0.0),
        Point::new(1.0, 1.0),
        Point::new(2.0, 2.0),
        Point::new(3.0, 3.0),
    ];
    let p1 = [
        Point::new(0.0, f32::INFINITY),
        Point::new(1.0, 1.0),
        Point::new(2.0, 2.0),
        Point::new(3.0, 3.0),
    ];
    let p2 = [
        Point::new(f32::NAN, 0.0),
        Point::new(1.0, 1.0),
        Point::new(2.0, 2.0),
        Point::new(3.0, 3.0),
    ];
    let p3 = [
        Point::new(0.0, f32::NAN),
        Point::new(1.0, 1.0),
        Point::new(2.0, 2.0),
        Point::new(3.0, 3.0),
    ];
    let mut reporter = Reporter::default();
    let mut r = Rect::default();
    let zeror = Rect::new(0.0, 0.0, 0.0, 0.0);
    for pts in [p0, p1, p2, p3] {
        for n in 1..=4 {
            let isfinite = r.set_bounds_check(&pts[..n]);
            check!(reporter, !isfinite);
            check!(reporter, r == zeror);
            r.set_bounds_no_check(&pts[..n]);
            if r.is_finite() {
                r.set_bounds_no_check(&pts[..n]);
            }
            check!(reporter, !r.is_finite());
        }
    }
    reporter.finish();
}
fn make_big_value(reporter: Option<&Reporter>) -> f32 {
    assert!(reporter.is_some());
    if reporter.is_some() {
        f32::MAX * 0.75
    } else {
        0.0
    }
}
#[test]
fn Rect_whOverflow() {
    let mut reporter = Reporter::default();
    let big = make_big_value(Some(&reporter));
    let r = Rect::new(-big, -big, big, big);
    check!(reporter, r.is_finite());
    check!(reporter, !r.width().is_finite());
    check!(reporter, !r.height().is_finite());
    check!(reporter, r.center_x().is_finite());
    check!(reporter, r.center_y().is_finite());
    check!(reporter, RectPriv::half_width(&r).is_finite());
    check!(reporter, RectPriv::half_height(&r).is_finite());
    reporter.finish();
}
#[test]
fn Rect_subtract() {
    let mut reporter = Reporter::default();
    let a = IRect::make_ltrb(2, 3, 12, 15);
    let tests = [
        (a, a.make_outset(2, 2), IRect::make_empty(), true),
        (
            a,
            IRect::new(6, 6, 10, 10),
            IRect::new(2, 10, 12, 15),
            false,
        ),
        (IRect::make_empty(), a, IRect::make_empty(), true),
        (a, IRect::make_empty(), a, true),
        (
            IRect::make_empty(),
            IRect::make_empty(),
            IRect::make_empty(),
            true,
        ),
        (a, IRect::new(15, 17, 20, 40), a, true),
        (a, IRect::new(0, 0, 6, 20), IRect::new(6, 3, 12, 15), true),
        (a, IRect::new(6, 0, 20, 20), IRect::new(2, 3, 6, 15), true),
        (a, IRect::new(0, 0, 20, 6), IRect::new(2, 6, 12, 15), true),
        (a, IRect::new(0, 10, 20, 20), IRect::new(2, 3, 12, 10), true),
        (
            a,
            IRect::new(0, 6, 20, 10),
            IRect::new(2, 10, 12, 15),
            false,
        ),
        (a, IRect::new(6, 0, 10, 20), IRect::new(2, 3, 6, 15), false),
        (a, IRect::new(0, 0, 4, 4), IRect::new(2, 4, 12, 15), false),
        (a, IRect::new(10, 0, 14, 7), IRect::new(2, 3, 10, 15), false),
        (a, IRect::new(0, 12, 5, 20), IRect::new(2, 3, 12, 12), false),
        (
            a,
            IRect::new(10, 12, 20, 20),
            IRect::new(2, 3, 10, 15),
            false,
        ),
        (a, IRect::new(0, 6, 8, 10), IRect::new(2, 10, 12, 15), false),
        (a, IRect::new(8, 6, 20, 10), IRect::new(2, 3, 8, 15), false),
        (a, IRect::new(6, 0, 10, 7), IRect::new(2, 7, 12, 15), false),
        (a, IRect::new(4, 6, 8, 20), IRect::new(8, 3, 12, 15), false),
    ];
    for (a, b, expected, want_exact) in tests {
        let mut difference = IRect::default();
        let mut exact = RectPriv::subtract_irect(&a, &b, &mut difference);
        check!(reporter, exact == want_exact);
        check!(reporter, difference == expected);
        let mut af = Rect::from(a);
        let mut bf = Rect::from(b);
        let mut ef = Rect::from(expected);
        af.offset(0.5, 0.5);
        bf.offset(0.5, 0.5);
        ef.offset(0.5, 0.5);
        let mut df = Rect::default();
        exact = RectPriv::subtract(&af, &bf, &mut df);
        check!(reporter, exact == want_exact);
        check!(reporter, (df.is_empty() && ef.is_empty()) || df == ef);
    }
    reporter.finish();
}
#[test]
fn Rect_subtract_overflow() {
    let mut reporter = Reporter::default();
    let really_big = IRect::make_ltrb(-i32::MAX + 1000, 0, i32::MAX - 1000, 100);
    let reasonable = IRect::make_ltrb(-50, -5, 50, 125);
    let mut difference = IRect::default();
    let mut exact = RectPriv::subtract_irect(&really_big, &reasonable, &mut difference);
    check!(reporter, exact);
    check!(reporter, difference == really_big);
    exact = RectPriv::subtract_irect(&reasonable, &really_big, &mut difference);
    check!(reporter, exact);
    check!(reporter, difference == reasonable);
    reporter.finish();
}
#[test]
fn Rect_ClosestDisjointEdge() {
    let mut reporter = Reporter::default();
    let src = IRect::new(0, 0, 10, 10);
    let tests = [
        (
            "src left edge",
            IRect::new(-15, -5, -2, 15),
            IRect::new(0, 0, 1, 10),
        ),
        (
            "src left edge clipped to dst",
            IRect::new(-15, 2, -2, 8),
            IRect::new(0, 2, 1, 8),
        ),
        (
            "src top-left corner",
            IRect::new(-15, -15, -2, -2),
            IRect::new(0, 0, 1, 1),
        ),
        (
            "src top edge",
            IRect::new(-5, -10, 15, -2),
            IRect::new(0, 0, 10, 1),
        ),
        (
            "src top edge clipped to dst",
            IRect::new(2, -10, 8, -2),
            IRect::new(2, 0, 8, 1),
        ),
        (
            "src top-right corner",
            IRect::new(15, -15, 20, -2),
            IRect::new(9, 0, 10, 1),
        ),
        (
            "src right edge",
            IRect::new(15, -5, 20, 15),
            IRect::new(9, 0, 10, 10),
        ),
        (
            "src right edge clipped to dst",
            IRect::new(15, 2, 20, 8),
            IRect::new(9, 2, 10, 8),
        ),
        (
            "src bottom-right corner",
            IRect::new(15, 15, 20, 20),
            IRect::new(9, 9, 10, 10),
        ),
        (
            "src bottom edge",
            IRect::new(-5, 15, 15, 20),
            IRect::new(0, 9, 10, 10),
        ),
        (
            "src bottom edge clipped to dst",
            IRect::new(2, 15, 8, 20),
            IRect::new(2, 9, 8, 10),
        ),
        (
            "src bottom-left corner",
            IRect::new(-15, 15, -2, 20),
            IRect::new(0, 9, 1, 10),
        ),
        (
            "src intersects dst high",
            IRect::new(2, 2, 15, 15),
            IRect::new(2, 2, 10, 10),
        ),
        (
            "src intersects dst low",
            IRect::new(-5, -5, 8, 8),
            IRect::new(0, 0, 8, 8),
        ),
        (
            "src contains dst",
            IRect::new(2, 2, 8, 8),
            IRect::new(2, 2, 8, 8),
        ),
        (
            "src contained in dst",
            IRect::new(-5, -5, 15, 15),
            IRect::new(0, 0, 10, 10),
        ),
    ];
    for (label, dst, expect) in tests {
        let actual = RectPriv::closest_disjoint_edge(&src, &dst);
        reporter.record(actual == expect, label, file!(), line!());
    }
    check!(
        reporter,
        RectPriv::closest_disjoint_edge(&IRect::make_empty(), &IRect::new(0, 0, 8, 8)).is_empty()
    );
    check!(
        reporter,
        RectPriv::closest_disjoint_edge(&IRect::new(0, 0, 8, 8), &IRect::make_empty()).is_empty()
    );
    check!(
        reporter,
        RectPriv::closest_disjoint_edge(&IRect::new(10, 10, -1, 2), &IRect::new(15, 8, -2, 20))
            .is_empty()
    );
    reporter.finish();
}
