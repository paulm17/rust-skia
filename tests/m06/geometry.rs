// GeometryTest.cpp:632,978 and all reached helpers. BSD license: tests/LICENSE.skia.
use super::*;
use SkCubicType::*;
include!("geometry_fixtures.rs");
fn check_pairs(
    r: &mut Reporter,
    index: usize,
    t: f32,
    name: &str,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
) {
    let eq = near(x0, x1) && near(y0, y1);
    if !eq {
        eprintln!("{name} [{index} {t}] p0 [{x0:10.8} {y0:10.8}] p1 [{x1:10.8} {y1:10.8}]");
        check!(r, eq);
    }
}
fn testChopCubic(r: &mut Reporter) {
    let src = [
        p(2190., 5130.),
        p(2190., 5070.),
        p(2220., 5010.),
        p(2205., 4980.),
    ];
    let mut dst = [Point::default(); 13];
    let mut ts = [0.; 3];
    let count = sk_chop_cubic_at_max_curvature(&src, Some(&mut dst), Some(&mut ts));
    if false {
        check!(r, count != 0);
    }
    {
        let mut pts = [Point::default(); 7];
        for i in 0..7 {
            pts[i].set(i as f32, i as f32);
        }
        sk_chop_cubic_at_in_place(&mut pts, 0.5);
        for i in 0..7 {
            check!(r, pts[i].x == pts[i].y);
            check!(r, pts[i].x == i as f32 * 0.5);
        }
    }
    let chopTs = [
        0.,
        3. / 83.,
        3. / 79.,
        3. / 73.,
        3. / 71.,
        3. / 67.,
        3. / 61.,
        3. / 59.,
        3. / 53.,
        3. / 47.,
        3. / 43.,
        3. / 41.,
        3. / 37.,
        3. / 31.,
        3. / 29.,
        3. / 23.,
        3. / 19.,
        3. / 17.,
        3. / 13.,
        3. / 11.,
        3. / 7.,
        3. / 5.,
        1.,
    ];
    let ones = [1.; 5];
    const {
        assert!(23 % 2 == 1);
        assert!(5 % 2 == 1);
    }
    let mut rand = SkRandom::default();
    for _ in 0..5 {
        let pts = [
            p(rand.next_f(), rand.next_f()),
            p(rand.next_f(), rand.next_f()),
            p(rand.next_f(), rand.next_f()),
            p(rand.next_f(), rand.next_f()),
        ];
        let mut allChops = [Point::default(); 4 + 23 * 3];
        sk_chop_cubic_at_ts(&pts, &mut allChops, &chopTs);
        let mut i = 3;
        for chopT in chopTs {
            let mut expected = Point::default();
            sk_eval_cubic_at(&pts, chopT, Some(&mut expected), None, None);
            check!(r, near(allChops[i].x, expected.x));
            check!(r, near(allChops[i].y, expected.y));
            if chopT == 0. {
                check!(r, allChops[i] == pts[0]);
            }
            if chopT == 1. {
                check!(r, allChops[i] == pts[3]);
            }
            i += 3;
            let mut local = [Point::default(); 10];
            sk_chop_cubic_at_two(&pts, &mut local, chopT, chopT);
            check!(r, local[3] == local[4]);
            check!(r, local[3] == local[5]);
            check!(r, local[3] == local[6]);
            if chopT == 0. {
                for j in 0..4 {
                    check!(r, local[j] == pts[0]);
                }
            }
            if chopT == 1. {
                for j in 6..10 {
                    check!(r, local[j] == pts[3]);
                }
            }
        }
        let mut oneChops = [Point::default(); 4 + 5 * 3];
        sk_chop_cubic_at_ts(&pts, &mut oneChops, &ones);
        check!(r, oneChops[0] == pts[0]);
        check!(r, oneChops[1] == pts[1]);
        check!(r, oneChops[2] == pts[2]);
        for index in 3..oneChops.len() {
            check!(r, oneChops[index] == pts[3]);
        }
    }
}
fn test_evalquadat(r: &mut Reporter) {
    let mut rand = SkRandom::default();
    for i in 0..1000 {
        let mut pts = [Point::default(); 3];
        for j in 0..3 {
            pts[j].set(rand.next_s_scalar1() * 100., rand.next_s_scalar1() * 100.);
        }
        let dt = 1. / 128.;
        let mut t = dt;
        for _j in 1..128 {
            let mut r0 = Point::default();
            sk_eval_quad_at_with(&pts, t, Some(&mut r0), None);
            let r1 = sk_eval_quad_at(&pts, t);
            check_pairs(r, i, t, "quad-pos", r0.x, r0.y, r1.x, r1.y);
            let mut v0 = Point::default();
            sk_eval_quad_at_with(&pts, t, None, Some(&mut v0));
            let v1 = sk_eval_quad_tangent_at(&pts, t);
            check_pairs(r, i, t, "quad-tan", v0.x, v0.y, v1.x, v1.y);
            t += dt;
        }
    }
}
fn test_conic_eval_pos(r: &mut Reporter, c: &SkConic, t: f32) {
    let mut p0 = Point::default();
    c.eval_at_with(t, Some(&mut p0), None);
    let p1 = c.eval_at(t);
    check_pairs(r, 0, t, "conic-pos", p0.x, p0.y, p1.x, p1.y);
}
fn test_conic_eval_tan(r: &mut Reporter, c: &SkConic, t: f32) {
    let mut v0 = Point::default();
    c.eval_at_with(t, None, Some(&mut v0));
    let v1 = c.eval_tangent_at(t);
    check_pairs(r, 0, t, "conic-tan", v0.x, v0.y, v1.x, v1.y);
}
fn test_conic(r: &mut Reporter) {
    let mut rand = SkRandom::default();
    for _i in 0..1000 {
        let mut pts = [Point::default(); 3];
        for j in 0..3 {
            pts[j].set(rand.next_s_scalar1() * 100., rand.next_s_scalar1() * 100.);
        }
        for _k in 0..10 {
            let w = rand.next_u_scalar1() * 2.;
            let conic = SkConic::from_pts(&pts, w);
            let dt = 1. / 128.;
            let mut t = dt;
            for _j in 1..128 {
                test_conic_eval_pos(r, &conic, t);
                test_conic_eval_tan(r, &conic, t);
                t += dt;
            }
        }
    }
}
fn tangent_checks(r: &mut Reporter, start: Point, mid: Point, end: Point) {
    check!(r, start.x != 0. && start.y != 0.);
    check!(r, mid.x != 0. && mid.y != 0.);
    check!(r, end.x != 0. && end.y != 0.);
    check!(r, near(start.cross(&mid), 0.));
    check!(r, near(mid.cross(&end), 0.));
}
fn test_quad_tangents(r: &mut Reporter) {
    let pts = [
        p(10., 20.),
        p(10., 20.),
        p(20., 30.),
        p(10., 20.),
        p(15., 25.),
        p(20., 30.),
        p(10., 20.),
        p(20., 30.),
        p(20., 30.),
    ];
    for index in 0..pts.len() / 3 {
        let _conic = SkConic::from_pts(&pts[index * 3..], 0.707);
        let start = sk_eval_quad_tangent_at(&pts[index * 3..], 0.);
        let mid = sk_eval_quad_tangent_at(&pts[index * 3..], 0.5);
        let end = sk_eval_quad_tangent_at(&pts[index * 3..], 1.);
        tangent_checks(r, start, mid, end);
    }
}
fn test_conic_tangents(r: &mut Reporter) {
    let pts = [
        p(10., 20.),
        p(10., 20.),
        p(20., 30.),
        p(10., 20.),
        p(15., 25.),
        p(20., 30.),
        p(10., 20.),
        p(20., 30.),
        p(20., 30.),
    ];
    for index in 0..pts.len() / 3 {
        let c = SkConic::from_pts(&pts[index * 3..], 0.707);
        tangent_checks(
            r,
            c.eval_tangent_at(0.),
            c.eval_tangent_at(0.5),
            c.eval_tangent_at(1.),
        );
    }
}
fn test_this_conic_to_quad(r: &mut Reporter, pts: &[Point], w: f32) {
    let mut quadder = SkAutoConicToQuads::new();
    // Copy only to release the storage borrow before querying count_quads; values/order unchanged.
    let qpts = quadder.compute_quads_pts(pts, w, 0.25).to_vec();
    let qcount = quadder.count_quads();
    let pcount = qcount * 2 + 1;
    check!(
        r,
        rust_skia::point_priv::PointPriv::are_finite(&qpts[..pcount as usize])
    );
}
fn test_conic_to_quads(r: &mut Reporter) {
    let triples = [
        p(0., 0.),
        p(1., 0.),
        p(1., 1.),
        p(0., 0.),
        p(3.58732e-43, 2.72084),
        p(3.00392, 3.00392),
        p(0., 0.),
        p(100000., 0.),
        p(100000., 100000.),
        p(0., 0.),
        p(1e30, 0.),
        p(1e30, 1e30),
    ];
    for pts in triples.chunks_exact(3) {
        let mut w = 1e30f32;
        loop {
            w *= 2.;
            test_this_conic_to_quad(r, pts, w);
            if !w.is_finite() {
                break;
            }
        }
        test_this_conic_to_quad(r, pts, f32::NAN);
    }
}
fn test_cubic_tangents(r: &mut Reporter) {
    let pts = [
        p(10., 20.),
        p(10., 20.),
        p(20., 30.),
        p(30., 40.),
        p(10., 20.),
        p(15., 25.),
        p(20., 30.),
        p(30., 40.),
        p(10., 20.),
        p(20., 30.),
        p(30., 40.),
        p(30., 40.),
    ];
    for index in 0..pts.len() / 4 {
        // Preserve the original, otherwise unused conic construction and its index * 3.
        let _conic = SkConic::from_pts(&pts[index * 3..], 0.707);
        let mut start = Point::default();
        let mut mid = Point::default();
        let mut end = Point::default();
        sk_eval_cubic_at(&pts[index * 4..], 0., None, Some(&mut start), None);
        sk_eval_cubic_at(&pts[index * 4..], 0.5, None, Some(&mut mid), None);
        sk_eval_cubic_at(&pts[index * 4..], 1., None, Some(&mut end), None);
        tangent_checks(r, start, mid, end);
    }
}
fn check_cubic_type(r: &mut Reporter, pts: &[Point], expected: SkCubicType, undefined: bool) {
    let actual = classify_cubic(pts);
    if !undefined {
        check!(r, actual == expected);
    }
}
fn check_cubic_around_rect(
    name: &str,
    r: &mut Reporter,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    undefined: bool,
) {
    eprintln!("{name}");
    let expectations = [
        Loop,
        CuspAtInfinity,
        LocalCusp,
        LocalCusp,
        CuspAtInfinity,
        Loop,
        CuspAtInfinity,
        Loop,
        CuspAtInfinity,
        Loop,
        LocalCusp,
        LocalCusp,
        LocalCusp,
        LocalCusp,
        Loop,
        CuspAtInfinity,
        Loop,
        CuspAtInfinity,
        Loop,
        CuspAtInfinity,
        LocalCusp,
        LocalCusp,
        CuspAtInfinity,
        Loop,
    ];
    let points = [p(x1, y1), p(x2, y1), p(x2, y2), p(x1, y2)];
    let mut bezier = [Point::default(); 4];
    for i in 0..4 {
        bezier[0] = points[i];
        for j in 0..3 {
            let jidx = if j < i { j } else { j + 1 };
            bezier[1] = points[jidx];
            let mut kidx = 0;
            for k in 0..2 {
                for _n in 0..2 {
                    kidx = if kidx == i || kidx == jidx {
                        kidx + 1
                    } else {
                        kidx
                    };
                }
                bezier[2] = points[kidx];
                for l in 0..4 {
                    if l != i && l != jidx && l != kidx {
                        bezier[3] = points[l];
                        break;
                    }
                }
                check_cubic_type(r, &bezier, expectations[i * 6 + j * 2 + k], undefined);
                kidx += 1;
            }
        }
    }
    for i in 0..4 {
        bezier[0] = points[i];
        for j in 0..3 {
            let jidx = if j < i { j } else { j + 1 };
            bezier[1] = points[jidx];
            bezier[2] = points[jidx];
            let mut kidx = 0;
            for _k in 0..2 {
                for _n in 0..2 {
                    kidx = if kidx == i || kidx == jidx {
                        kidx + 1
                    } else {
                        kidx
                    };
                }
                bezier[3] = points[kidx];
                check_cubic_type(r, &bezier, Serpentine, undefined);
                kidx += 1;
            }
        }
    }
}
fn test_classify_cubic(r: &mut Reporter) {
    for serp in kSerpentines {
        check_cubic_type(r, serp, Serpentine, false);
    }
    for curve in kLoops {
        check_cubic_type(r, curve, Loop, false);
    }
    for line in kLinearCubics {
        check_cubic_type(r, line, LineOrPoint, false);
    }
    check_cubic_around_rect("small box", r, 0., 0., 1., 1., false);
    check_cubic_around_rect(
        "biggest box",
        r,
        -f32::MAX,
        -f32::MAX,
        f32::MAX,
        f32::MAX,
        false,
    );
    check_cubic_around_rect(
        "large quadrant",
        r,
        1.,
        1.,
        f32::MIN_POSITIVE,
        f32::MAX,
        false,
    );
    check_cubic_around_rect(
        "smallest box",
        r,
        -f32::MIN_POSITIVE,
        -f32::MIN_POSITIVE,
        f32::MIN_POSITIVE,
        f32::MIN_POSITIVE,
        false,
    );
    check_cubic_around_rect(
        "slightly negative box",
        r,
        1.,
        -f32::MIN_POSITIVE,
        -1.,
        -1.,
        false,
    );
    check_cubic_around_rect(
        "infinite box",
        r,
        -f32::INFINITY,
        -f32::INFINITY,
        f32::INFINITY,
        f32::INFINITY,
        true,
    );
    check_cubic_around_rect("one sided infinite box", r, 0., 0., 1., f32::INFINITY, true);
    check_cubic_around_rect("nan box", r, -f32::NAN, -f32::NAN, f32::NAN, f32::NAN, true);
    check_cubic_around_rect("partial nan box", r, 0., 0., 1., f32::NAN, true);
}
fn test_cubic_cusps(r: &mut Reporter) {
    let noCusps = [
        [p(0., 0.), p(1., 1.), p(2., 2.), p(3., 3.)],
        [p(0., 0.), p(1., 0.), p(1., 1.), p(0., 1.)],
        [p(0., 0.), p(1., 0.), p(2., 1.), p(2., 2.)],
        [p(0., 0.), p(1., 0.), p(1., 1.), p(2., 1.)],
    ];
    for curve in noCusps {
        check!(r, sk_find_cubic_cusp(&curve) < 0.);
    }
    for cusp in kCusps {
        check!(r, sk_find_cubic_cusp(cusp) > 0.);
    }
}
fn skew_matrices() -> [Matrix; 3] {
    [
        Matrix::make_all(1., 0., 0., 0., 1., 0., 0., 0., 1.),
        Matrix::make_all(1., -1., 0., 1., 1., 0., 0., 0., 1.),
        Matrix::make_all(0.889, 0.553, 0., -0.443, 0.123, 0., 0., 0., 1.),
    ]
}
fn test_chop_quad_at_midtangent(r: &mut Reporter, pts: &[Point]) {
    let tolerance = 1e-3;
    for m in skew_matrices() {
        let mut mapped = [Point::default(); 3];
        m.map_points(&mut mapped, pts);
        // Upstream intentionally evaluates pts, not mapped, in this helper.
        let full = sk_measure_quad_rotation(pts);
        let mut chopped = [Point::default(); 5];
        sk_chop_quad_at_mid_tangent(pts, &mut chopped);
        let left = sk_measure_quad_rotation(&chopped);
        let right = sk_measure_quad_rotation(&chopped[2..]);
        check!(r, (left - full / 2.).abs() <= tolerance);
        check!(r, (right - full / 2.).abs() <= tolerance);
    }
}
fn test_chop_cubic_at_midtangent(r: &mut Reporter, pts: &[Point], kind: SkCubicType) {
    let tolerance = 1e-3;
    let pi = std::f32::consts::PI;
    let n = if kind == LocalCusp || kind == LineOrPoint {
        2
    } else {
        3
    };
    for m in &skew_matrices()[..n] {
        let mut mapped = [Point::default(); 4];
        m.map_points(&mut mapped, pts);
        let full = sk_measure_non_inflect_cubic_rotation(&mapped);
        let mut chopped = [Point::default(); 7];
        sk_chop_cubic_at_mid_tangent(&mapped, &mut chopped);
        let left = sk_measure_non_inflect_cubic_rotation(&chopped);
        let right = sk_measure_non_inflect_cubic_rotation(&chopped[3..]);
        if kind == LineOrPoint && ((full - 2. * pi).abs() <= tolerance || full.abs() <= tolerance) {
            check!(r, left.abs() - tolerance <= pi);
            check!(r, right.abs() - tolerance <= pi);
            continue;
        }
        let mut expected = full / 2.;
        if kind == LocalCusp || (kind == LineOrPoint && (full - pi).abs() <= tolerance) {
            expected = (full - pi) / 2.;
        }
        check!(r, (left - expected).abs() <= tolerance);
        check!(r, (right - expected).abs() <= tolerance);
    }
}
fn lerp(a: Point, b: Point, t: f32) -> Point {
    a * (1. - t) + b * t
}
fn test_measure_rotation(r: &mut Reporter) {
    let pi = std::f32::consts::PI;
    let flat = [p(0., 0.), p(0., 1.), p(0., 2.), p(0., 3.)];
    check!(r, near(sk_measure_non_inflect_cubic_rotation(&flat), 0.));
    let flat180_1 = [p(0., 0.), p(1., 0.), p(3., 0.), p(2., 0.)];
    check!(
        r,
        near(sk_measure_non_inflect_cubic_rotation(&flat180_1), pi)
    );
    let flat180_2 = [p(0., 1.), p(0., 0.), p(0., 2.), p(0., 3.)];
    check!(
        r,
        near(sk_measure_non_inflect_cubic_rotation(&flat180_2), pi)
    );
    let flat360 = [p(0., 1.), p(0., 0.), p(0., 3.), p(0., 2.)];
    check!(
        r,
        near(sk_measure_non_inflect_cubic_rotation(&flat360), 2. * pi)
    );
    let square180 = [p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.)];
    check!(
        r,
        near(sk_measure_non_inflect_cubic_rotation(&square180), pi)
    );
    let mut checkQuadRotation = |pts: &[Point], expected: f32| {
        let rotation = sk_measure_quad_rotation(pts);
        check!(r, near(rotation, expected));
        let cubic1 = [pts[0], pts[0], pts[1], pts[2]];
        check!(
            r,
            near(sk_measure_non_inflect_cubic_rotation(&cubic1), expected)
        );
        let cubic2 = [pts[0], pts[1], pts[1], pts[2]];
        check!(
            r,
            near(sk_measure_non_inflect_cubic_rotation(&cubic2), expected)
        );
        let cubic3 = [pts[0], pts[1], pts[2], pts[2]];
        check!(
            r,
            near(sk_measure_non_inflect_cubic_rotation(&cubic3), expected)
        );
    };
    checkQuadRotation(&[p(0., 0.), p(0., 1.), p(0., 2.), Point::default()], 0.);
    checkQuadRotation(&[p(1., 0.), p(0., 0.), p(2., 0.), Point::default()], pi);
    checkQuadRotation(&[p(0., 0.), p(0., 2.), p(0., 1.), Point::default()], pi);
    checkQuadRotation(
        &[p(0., 0.), p(0.5, 3.0f32.sqrt() / 2.), p(1., 0.)],
        2. * pi / 3.,
    );
}
fn test_chop_at_midtangent(r: &mut Reporter) {
    let mut chops = [Point::default(); 10];
    for serp in kSerpentines {
        check!(r, classify_cubic(serp) == Serpentine);
        let n = sk_chop_cubic_at_inflections(serp, &mut chops);
        for i in 0..n as usize {
            test_chop_cubic_at_midtangent(r, &chops[i * 3..i * 3 + 4], Serpentine);
        }
    }
    for curve in kLoops {
        check!(r, classify_cubic(curve) == Loop);
        test_chop_cubic_at_midtangent(r, curve, Loop);
    }
    for line in kLinearCubics {
        check!(r, classify_cubic(line) == LineOrPoint);
        test_chop_cubic_at_midtangent(r, line, LineOrPoint);
    }
    for cusp in kCusps {
        check!(r, classify_cubic(cusp) == LocalCusp);
        test_chop_cubic_at_midtangent(r, cusp, LocalCusp);
    }
    for quad in kQuads {
        test_chop_quad_at_midtangent(r, quad);
        let asCubic = [
            quad[0],
            lerp(quad[0], quad[1], 2. / 3.),
            lerp(quad[1], quad[2], 1. / 3.),
            quad[2],
        ];
        test_chop_cubic_at_midtangent(r, &asCubic, Quadratic);
    }
    let exactQuad = [p(0., 0.), p(6., 2.), p(10., 2.), p(12., 0.)];
    check!(r, classify_cubic(&exactQuad) == Quadratic);
    test_chop_cubic_at_midtangent(r, &exactQuad, Quadratic);
    let exactCuspAtInf = [p(0., 0.), p(1., 0.), p(0., 1.), p(1., 1.)];
    check!(r, classify_cubic(&exactCuspAtInf) == CuspAtInfinity);
    let n = sk_chop_cubic_at_inflections(&exactCuspAtInf, &mut chops);
    for i in 0..n as usize {
        test_chop_cubic_at_midtangent(r, &chops[i * 3..i * 3 + 4], CuspAtInfinity);
    }
}
#[test]
fn Geometry() {
    let mut r = Reporter::default();
    let mut pts = [Point::default(); 5];
    pts[0].set(0., 0.);
    pts[1].set(100., 50.);
    pts[2].set(0., 100.);
    let mut count = sk_chop_quad_at_max_curvature_in_place(&mut pts);
    check!(r, count == 1 || count == 2);
    pts[0].set(15.1213, 7.77647);
    pts[1].set(6.2168e19, 1.51338e20);
    pts[2].set(1.4579e19, 1.55558e21);
    count = sk_chop_quad_at_max_curvature_in_place(&mut pts);
    pts[0].set(0., 0.);
    pts[1].set(3., 0.);
    pts[2].set(3., 3.);
    // Rust shared/exclusive borrows cannot alias: snapshot source of this in-place conversion.
    let src = [pts[0], pts[1], pts[2]];
    sk_convert_quad_to_cubic(&src, &mut pts);
    let cubic = [p(0., 0.), p(2., 0.), p(3., 1.), p(3., 3.)];
    for i in 0..4 {
        check!(r, point_near(cubic[i], pts[i]));
    }
    testChopCubic(&mut r);
    test_evalquadat(&mut r);
    test_conic(&mut r);
    test_cubic_tangents(&mut r);
    test_quad_tangents(&mut r);
    test_conic_tangents(&mut r);
    test_conic_to_quads(&mut r);
    test_classify_cubic(&mut r);
    test_cubic_cusps(&mut r);
    test_measure_rotation(&mut r);
    test_chop_at_midtangent(&mut r);
    r.finish();
}
#[test]
fn ConicsWithCrazyW() {
    let mut r = Reporter::default();
    let max = f32::MAX;
    let inf = f32::INFINITY;
    let nanq = f32::NAN;
    let nans = f32::from_bits(0x7f800001);
    let weights = [
        0.,
        1. / 65535.,
        1.,
        65535.,
        max / 4.,
        max / 2.,
        max,
        inf,
        nanq,
        nans,
    ];
    let length = 100.;
    let mut conic = SkConic::new(p(0., length), p(0., 0.), p(length, 0.), 1.);
    let is_in_convex_hull = |p: Point| {
        let sum = p.x + p.y;
        let tinySlop = 1. / 32768.;
        p.x >= 0. && p.y >= 0. && sum <= length + tinySlop
    };
    let tol = 0.;
    for sign in [1., -1.] {
        for w in weights {
            conic.w = w * sign;
            let pow2 = conic.compute_quad_pow2(tol);
            check!(r, pow2 >= 0 && pow2 <= 30);
            let numQuads = 1i32 << pow2;
            check!(r, numQuads > 0);
            let numPoints = numQuads * 2 + 1;
            check!(r, numPoints >= 3);
            let mut pts = vec![Point::default(); numPoints as usize];
            let numQuads2 = conic.chop_into_quads_pow2(&mut pts, pow2);
            check!(r, numQuads2 <= numQuads);
            let numPoints2 = numQuads2 * 2 + 1;
            check!(r, numPoints2 <= numPoints);
            for i in 0..numPoints2 as usize {
                let pt = pts[i];
                check!(r, pt.x.is_finite() && pt.y.is_finite());
                check!(r, is_in_convex_hull(pt));
            }
        }
    }
    r.finish();
}
