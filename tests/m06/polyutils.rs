// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// Upstream guard: !SK_ENABLE_OPTIMIZE_SIZE; normal CPU build includes these APIs.
#[test]
fn PolyUtils() {
    let mut reporter = Reporter::default();

    let mut poly: Vec<Point> = Vec::new();
    // init simple index map
    let mut indexMap = [0u16; 1024];
    for i in 0..1024 {
        indexMap[i] = i as u16;
    }
    let mut triangleIndices: Vec<u16> = Vec::new();

    // skinny triangle
    poly.push(Point::new(-100.0, 55.0));
    poly.push(Point::new(100.0, 55.0));
    poly.push(Point::new(102.5, 54.330127));
    check!(reporter, sk_get_polygon_winding(&poly) < 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // switch winding
    poly[2].set(102.5, 55.330127);
    check!(reporter, sk_get_polygon_winding(&poly) > 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // make degenerate
    poly[2].set(100.0 + 2.5, 55.0);
    check!(reporter, sk_get_polygon_winding(&poly) == 0);
    // TODO: should these fail?
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        !sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // giant triangle
    poly[0].set(-1.0e+37, 1.0e+37);
    poly[1].set(1.0e+37, 1.0e+37);
    poly[2].set(-1.0e+37, -1.0e+37);
    check!(reporter, sk_get_polygon_winding(&poly) < 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // teeny triangle
    poly[0].set(-1.0e-38, 1.0e-38);
    poly[1].set(-1.0e-38, -1.0e-38);
    poly[2].set(1.0e-38, 1.0e-38);
    check!(reporter, sk_get_polygon_winding(&poly) == 0);
    // TODO: should these fail?
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        !sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // triangle way off in space (relative to size so we don't completely obliterate values)
    poly[0].set(-100.0 + 1.0e+9, 55.0 - 1.0e+9);
    poly[1].set(100.0 + 1.0e+9, 55.0 - 1.0e+9);
    poly[2].set(150.0 + 1.0e+9, 100.0 - 1.0e+9);
    check!(reporter, sk_get_polygon_winding(&poly) > 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    ///////////////////////////////////////////////////////////////////////
    // round rect
    poly.clear();
    poly.push(Point::new(-100.0, 55.0));
    poly.push(Point::new(100.0, 55.0));
    poly.push(Point::new(100.0 + 2.5, 50.0 + 4.330127));
    poly.push(Point::new(100.0 + 3.535534, 50.0 + 3.535534));
    poly.push(Point::new(100.0 + 4.330127, 50.0 + 2.5));
    poly.push(Point::new(105.0, 50.0));
    poly.push(Point::new(105.0, -50.0));
    poly.push(Point::new(100.0 + 4.330127, -50.0 - 2.5));
    poly.push(Point::new(100.0 + 3.535534, -50.0 - 3.535534));
    poly.push(Point::new(100.0 + 2.5, -50.0 - 4.330127));
    poly.push(Point::new(100.0, -55.0));
    poly.push(Point::new(-100.0, -55.0));
    poly.push(Point::new(-100.0 - 2.5, -50.0 - 4.330127));
    poly.push(Point::new(-100.0 - 3.535534, -50.0 - 3.535534));
    poly.push(Point::new(-100.0 - 4.330127, -50.0 - 2.5));
    poly.push(Point::new(-105.0, -50.0));
    poly.push(Point::new(-105.0, 50.0));
    poly.push(Point::new(-100.0 - 4.330127, 50.0 + 2.5));
    poly.push(Point::new(-100.0 - 3.535534, 50.0 + 3.535534));
    poly.push(Point::new(-100.0 - 2.5, 50.0 + 4.330127));
    check!(reporter, sk_get_polygon_winding(&poly) < 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // translate far enough to obliterate some low bits
    for i in 0..poly.len() {
        poly[i].offset(1.0e+7, 1.0e+7);
    }
    check!(reporter, sk_get_polygon_winding(&poly) < 0);
    // Due to floating point error it's no longer convex
    check!(reporter, !sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // translate a little farther to create some coincident vertices
    for i in 0..poly.len() {
        poly[i].offset(4.0e+7, 4.0e+7);
    }
    check!(reporter, sk_get_polygon_winding(&poly) < 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    // This can't handle coincident vertices
    triangleIndices.clear();
    check!(
        reporter,
        !sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // troublesome case -- clipped roundrect
    poly.clear();
    poly.push(Point::new(335.928101, 428.219055));
    poly.push(Point::new(330.414459, 423.034912));
    poly.push(Point::new(325.749084, 417.395508));
    poly.push(Point::new(321.931946, 411.300842));
    poly.push(Point::new(318.963074, 404.750977));
    poly.push(Point::new(316.842468, 397.745850));
    poly.push(Point::new(315.570068, 390.285522));
    poly.push(Point::new(315.145966, 382.369965));
    poly.push(Point::new(315.570068, 374.454346));
    poly.push(Point::new(316.842468, 366.994019));
    poly.push(Point::new(318.963074, 359.988892));
    poly.push(Point::new(321.931946, 353.439056));
    poly.push(Point::new(325.749084, 347.344421));
    poly.push(Point::new(330.414459, 341.705017));
    poly.push(Point::new(335.928101, 336.520813));
    poly.push(Point::new(342.289948, 331.791901));
    poly.push(Point::new(377.312134, 331.791901));
    poly.push(Point::new(381.195313, 332.532593));
    poly.push(Point::new(384.464935, 334.754700));
    poly.push(Point::new(386.687042, 338.024292));
    poly.push(Point::new(387.427765, 341.907532));
    poly.push(Point::new(387.427765, 422.832367));
    poly.push(Point::new(386.687042, 426.715576));
    poly.push(Point::new(384.464935, 429.985168));
    poly.push(Point::new(381.195313, 432.207275));
    poly.push(Point::new(377.312134, 432.947998));
    poly.push(Point::new(342.289948, 432.947998));
    check!(reporter, sk_get_polygon_winding(&poly) > 0);
    check!(reporter, sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // a star is born
    poly.clear();
    poly.push(Point::new(0.0, -50.0));
    poly.push(Point::new(14.43, -25.0));
    poly.push(Point::new(43.30, -25.0));
    poly.push(Point::new(28.86, 0.0));
    poly.push(Point::new(43.30, 25.0));
    poly.push(Point::new(14.43, 25.0));
    poly.push(Point::new(0.0, 50.0));
    poly.push(Point::new(-14.43, 25.0));
    poly.push(Point::new(-43.30, 25.0));
    poly.push(Point::new(-28.86, 0.0));
    poly.push(Point::new(-43.30, -25.0));
    poly.push(Point::new(-14.43, -25.0));
    check!(reporter, sk_get_polygon_winding(&poly) > 0);
    check!(reporter, !sk_is_convex_polygon(&poly));
    check!(reporter, sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // many spiked star
    {
        let c = 45.0f32;
        let r1 = 20.0f32;
        let r2 = 3.0f32;
        let n = 500;
        poly.clear();
        let mut rad = 0.0f32;
        let drad = std::f32::consts::PI / n as f32;
        for _i in 0..n {
            poly.push(Point::new(c + rad.cos() * r1, c + rad.sin() * r1));
            rad += drad;
            poly.push(Point::new(c + rad.cos() * r2, c + rad.sin() * r2));
            rad += drad;
        }
        check!(reporter, sk_get_polygon_winding(&poly) > 0);
        check!(reporter, !sk_is_convex_polygon(&poly));
        check!(reporter, sk_is_simple_polygon(&poly));
        triangleIndices.clear();
        check!(
            reporter,
            sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
        );
    }

    // self-intersecting polygon
    poly.clear();
    poly.push(Point::new(0.0, -50.0));
    poly.push(Point::new(14.43, -25.0));
    poly.push(Point::new(43.30, -25.0));
    poly.push(Point::new(-28.86, 0.0));
    poly.push(Point::new(43.30, 25.0));
    poly.push(Point::new(14.43, 25.0));
    poly.push(Point::new(0.0, 50.0));
    poly.push(Point::new(-14.43, 25.0));
    poly.push(Point::new(-43.30, 25.0));
    poly.push(Point::new(28.86, 0.0));
    poly.push(Point::new(-43.30, -25.0));
    poly.push(Point::new(-14.43, -25.0));
    check!(reporter, sk_get_polygon_winding(&poly) > 0);
    check!(reporter, !sk_is_convex_polygon(&poly));
    check!(reporter, !sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    // running this just to make sure it doesn't crash
    // the fact that it succeeds doesn't mean anything since the input is not simple
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // self-intersecting polygon with coincident point
    poly.clear();
    poly.push(Point::new(0.0, 0.0));
    poly.push(Point::new(-50.0, -50.0));
    poly.push(Point::new(50.0, -50.0));
    poly.push(Point::new(0.00000001, -0.00000001));
    poly.push(Point::new(-50.0, 50.0));
    poly.push(Point::new(50.0, 50.0));
    check!(reporter, sk_get_polygon_winding(&poly) == 0);
    check!(reporter, !sk_is_convex_polygon(&poly));
    check!(reporter, !sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    // running this just to make sure it doesn't crash
    check!(
        reporter,
        !sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // self-intersecting polygon with two equal edges
    poly.clear();
    poly.push(Point::new(0.0, 0.0));
    poly.push(Point::new(10.0, 0.0));
    poly.push(Point::new(0.0, 10.0));
    poly.push(Point::new(10.0, 10.0));
    poly.push(Point::new(10.0, 0.0));
    poly.push(Point::new(0.0, 10.0));
    check!(reporter, sk_get_polygon_winding(&poly) == 0);
    check!(reporter, !sk_is_convex_polygon(&poly));
    check!(reporter, !sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    // running this just to make sure it doesn't crash
    check!(
        reporter,
        !sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );

    // absurd self-intersecting polygon
    poly.clear();
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(
        3284.8125,
        -10411310938997512334153865557442560.0000,
    ));
    poly.push(Point::new(-32768.7500, 0.0000));
    poly.push(Point::new(138.0000, 3172.8125));
    poly.push(Point::new(0.0000, -2147485952.0000));
    poly.push(Point::new(0.0000, 170.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(137.0000, 4105.6875));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3283.0000, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 821.1250));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -30897.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.5625, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3526523879424.0000, 0.0000));
    poly.push(Point::new(-32768.9375, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(129.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 859832320.0000));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 129.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-33554468.0000, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 219.0000));
    poly.push(Point::new(3220.8125, 0.0000));
    poly.push(Point::new(-35840.0625, 0.0000));
    poly.push(Point::new(0.0000, 3284.8125));
    poly.push(Point::new(
        0.0000,
        -41625560509365411790244566154608640.0000,
    ));
    poly.push(Point::new(0.0000, 215282736.0000));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(
        -41625560509365411790244566154608640.0000,
        0.0000,
    ));
    poly.push(Point::new(215282736.0000, 0.0000));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3156.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 129.0000));
    poly.push(Point::new(7433.6875, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(59324728941049917997056.0000, 0.0000));
    poly.push(Point::new(-35840.0625, 0.0000));
    poly.push(Point::new(0.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(137.0000, 4105.6875));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3283.0000, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(138.0000, 3284.5625));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(
        -44882437151680690189392273689542656.0000,
        134217728.0000,
    ));
    poly.push(Point::new(0.0000, 0.0000));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 217055232.0000));
    poly.push(Point::new(138.3125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.5625));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(2152988672.0000, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    poly.push(Point::new(138.0000, 3284.8125));
    poly.push(Point::new(0.0000, -32768.0625));
    poly.push(Point::new(0.0000, 138.0000));
    poly.push(Point::new(3284.8125, 0.0000));
    poly.push(Point::new(-32768.0625, 0.0000));
    check!(reporter, sk_get_polygon_winding(&poly) < 0);
    check!(reporter, !sk_is_convex_polygon(&poly));
    check!(reporter, !sk_is_simple_polygon(&poly));
    triangleIndices.clear();
    // running this just to make sure it doesn't crash
    check!(
        reporter,
        sk_triangulate_simple_polygon(&poly, &indexMap, &mut triangleIndices)
    );
    reporter.finish();
}
