// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// Upstream guard: !SK_ENABLE_OPTIMIZE_SIZE; normal CPU build includes these APIs.
#[test]
fn InsetConvexPoly() {
    let mut reporter = Reporter::default();
    let mut rrectPoly: Vec<Point> = Vec::new();

    // round rect
    rrectPoly.push(Point::new(-100.0, 55.0));
    rrectPoly.push(Point::new(100.0, 55.0));
    rrectPoly.push(Point::new(100.0 + 2.5, 50.0 + 4.330127));
    rrectPoly.push(Point::new(100.0 + 3.535534, 50.0 + 3.535534));
    rrectPoly.push(Point::new(100.0 + 4.330127, 50.0 + 2.5));
    rrectPoly.push(Point::new(105.0, 50.0));
    rrectPoly.push(Point::new(105.0, -50.0));
    rrectPoly.push(Point::new(100.0 + 4.330127, -50.0 - 2.5));
    rrectPoly.push(Point::new(100.0 + 3.535534, -50.0 - 3.535534));
    rrectPoly.push(Point::new(100.0 + 2.5, -50.0 - 4.330127));
    rrectPoly.push(Point::new(100.0, -55.0));
    rrectPoly.push(Point::new(-100.0, -55.0));
    rrectPoly.push(Point::new(-100.0 - 2.5, -50.0 - 4.330127));
    rrectPoly.push(Point::new(-100.0 - 3.535534, -50.0 - 3.535534));
    rrectPoly.push(Point::new(-100.0 - 4.330127, -50.0 - 2.5));
    rrectPoly.push(Point::new(-105.0, -50.0));
    rrectPoly.push(Point::new(-105.0, 50.0));
    rrectPoly.push(Point::new(-100.0 - 4.330127, 50.0 + 2.5));
    rrectPoly.push(Point::new(-100.0 - 3.535534, 50.0 + 3.535534));
    rrectPoly.push(Point::new(-100.0 - 2.5, 50.0 + 4.330127));
    check!(reporter, sk_is_convex_polygon(&rrectPoly));

    // inset a little
    let mut insetPoly: Vec<Point> = Vec::new();
    let mut result = sk_inset_convex_polygon(&rrectPoly, 3.0, &mut insetPoly);
    check!(reporter, result);
    check!(reporter, sk_is_convex_polygon(&insetPoly));

    // inset to rect
    result = sk_inset_convex_polygon(&rrectPoly, 10.0, &mut insetPoly);
    check!(reporter, result);
    check!(reporter, sk_is_convex_polygon(&insetPoly));
    check!(reporter, insetPoly.len() == 4);
    if (insetPoly.len() == 4) {
        check!(reporter, insetPoly[0] == Point::new(-95.0, 45.0));
        check!(reporter, insetPoly[1] == Point::new(95.0, 45.0));
        check!(reporter, insetPoly[2] == Point::new(95.0, -45.0));
        check!(reporter, insetPoly[3] == Point::new(-95.0, -45.0));
    }

    // just to full inset
    // fails, but outputs a line segment
    result = sk_inset_convex_polygon(&rrectPoly, 55.0, &mut insetPoly);
    check!(reporter, !result);
    check!(reporter, !sk_is_convex_polygon(&insetPoly));
    check!(reporter, insetPoly.len() == 2);
    if (insetPoly.len() == 2) {
        check!(reporter, insetPoly[0] == Point::new(-50.0, 0.0));
        check!(reporter, insetPoly[1] == Point::new(50.0, 0.0));
    }

    // past full inset
    result = sk_inset_convex_polygon(&rrectPoly, 75.0, &mut insetPoly);
    check!(reporter, !result);
    check!(reporter, insetPoly.len() == 1);

    // troublesome case
    let mut clippedRRectPoly: Vec<Point> = Vec::new();
    clippedRRectPoly.push(Point::new(335.928101, 428.219055));
    clippedRRectPoly.push(Point::new(330.414459, 423.034912));
    clippedRRectPoly.push(Point::new(325.749084, 417.395508));
    clippedRRectPoly.push(Point::new(321.931946, 411.300842));
    clippedRRectPoly.push(Point::new(318.963074, 404.750977));
    clippedRRectPoly.push(Point::new(316.842468, 397.745850));
    clippedRRectPoly.push(Point::new(315.570068, 390.285522));
    clippedRRectPoly.push(Point::new(315.145966, 382.369965));
    clippedRRectPoly.push(Point::new(315.570068, 374.454346));
    clippedRRectPoly.push(Point::new(316.842468, 366.994019));
    clippedRRectPoly.push(Point::new(318.963074, 359.988892));
    clippedRRectPoly.push(Point::new(321.931946, 353.439056));
    clippedRRectPoly.push(Point::new(325.749084, 347.344421));
    clippedRRectPoly.push(Point::new(330.414459, 341.705017));
    clippedRRectPoly.push(Point::new(335.928101, 336.520813));
    clippedRRectPoly.push(Point::new(342.289948, 331.791901));
    clippedRRectPoly.push(Point::new(377.312134, 331.791901));
    clippedRRectPoly.push(Point::new(381.195313, 332.532593));
    clippedRRectPoly.push(Point::new(384.464935, 334.754700));
    clippedRRectPoly.push(Point::new(386.687042, 338.024292));
    clippedRRectPoly.push(Point::new(387.427765, 341.907532));
    clippedRRectPoly.push(Point::new(387.427765, 422.832367));
    clippedRRectPoly.push(Point::new(386.687042, 426.715576));
    clippedRRectPoly.push(Point::new(384.464935, 429.985168));
    clippedRRectPoly.push(Point::new(381.195313, 432.207275));
    clippedRRectPoly.push(Point::new(377.312134, 432.947998));
    clippedRRectPoly.push(Point::new(342.289948, 432.947998));
    check!(reporter, sk_is_convex_polygon(&clippedRRectPoly));

    result = sk_inset_convex_polygon(&clippedRRectPoly, 32.3699417, &mut insetPoly);
    check!(reporter, result);
    check!(reporter, sk_is_convex_polygon(&insetPoly));
    reporter.finish();
}
