// Temporary Deepseek development check; not a Codex acceptance test.
#![allow(clippy::all)]
use rust_skia::*;

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1.0 / 4096.0
}

fn rrect_poly() -> Vec<Point> {
    vec![
        Point::new(-100.0, 55.0),
        Point::new(100.0, 55.0),
        Point::new(100.0 + 2.5, 50.0 + 4.330127),
        Point::new(100.0 + 3.535534, 50.0 + 3.535534),
        Point::new(100.0 + 4.330127, 50.0 + 2.5),
        Point::new(105.0, 50.0),
        Point::new(105.0, -50.0),
        Point::new(100.0 + 4.330127, -50.0 - 2.5),
        Point::new(100.0 + 3.535534, -50.0 - 3.535534),
        Point::new(100.0 + 2.5, -50.0 - 4.330127),
        Point::new(100.0, -55.0),
        Point::new(-100.0, -55.0),
        Point::new(-100.0 - 2.5, -50.0 - 4.330127),
        Point::new(-100.0 - 3.535534, -50.0 - 3.535534),
        Point::new(-100.0 - 4.330127, -50.0 - 2.5),
        Point::new(-105.0, -50.0),
        Point::new(-105.0, 50.0),
        Point::new(-100.0 - 4.330127, 50.0 + 2.5),
        Point::new(-100.0 - 3.535534, 50.0 + 3.535534),
        Point::new(-100.0 - 2.5, 50.0 + 4.330127),
    ]
}

#[test]
fn dev_chop_mono_cubic_at_y() {
    let src = [
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 10.0),
    ];
    let mut out = [Point::default(); 7];
    assert!(sk_chop_mono_cubic_at_y(&src, 2.5, &mut out));
    let expected = [
        (0.0, 0.0),
        (0.0, 0.0),
        (1.065055, 1.065055),
        (2.5, 2.5),
        (5.461981, 5.461981),
        (10.0, 10.0),
        (10.0, 10.0),
    ];
    for (i, (x, y)) in expected.iter().enumerate() {
        assert!(near(out[i].x, *x) && near(out[i].y, *y), "{i}: {:?}", out[i]);
    }

    let src = [
        Point::new(1.0, 1.0),
        Point::new(5.0, 2.0),
        Point::new(7.0, 4.0),
        Point::new(8.0, 7.0),
    ];
    assert!(sk_chop_mono_cubic_at_x(&src, 2.0, &mut out));
    let expected = [
        (1.0, 1.0),
        (1.348275, 1.087069),
        (1.681389, 1.181719),
        (2.0, 1.283949),
        (5.340694, 2.355856),
        (7.087069, 4.261207),
        (8.0, 7.0),
    ];
    for (i, (x, y)) in expected.iter().enumerate() {
        assert!(near(out[i].x, *x) && near(out[i].y, *y), "{i}: {:?}", out[i]);
    }

    let flat = [
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 10.0),
    ];
    assert!(!sk_chop_mono_cubic_at_y(&flat, -10.0, &mut out));
    assert!(!sk_chop_mono_cubic_at_y(&flat, 20.0, &mut out));
}

#[test]
fn dev_classify_and_inflections() {
    let serpentine = [
        Point::new(149.325, 107.705),
        Point::new(149.325, 103.783),
        Point::new(151.638, 100.127),
        Point::new(156.263, 96.736),
    ];
    assert_eq!(classify_cubic(&serpentine), SkCubicType::Serpentine);
    let loop_curve = [
        Point::new(635.625, 614.687),
        Point::new(171.625, 236.188),
        Point::new(1064.62, 135.688),
        Point::new(516.625, 570.187),
    ];
    assert_eq!(classify_cubic(&loop_curve), SkCubicType::Loop);
    let cusp = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
    ];
    assert_eq!(classify_cubic(&cusp), SkCubicType::LocalCusp);
    let cusp_inf = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
    ];
    assert_eq!(classify_cubic(&cusp_inf), SkCubicType::CuspAtInfinity);
    let line = [
        Point::new(0.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(0.0, 2.0),
        Point::new(0.0, 3.0),
    ];
    assert_eq!(classify_cubic(&line), SkCubicType::LineOrPoint);
    let quad = [
        Point::new(0.0, 0.0),
        Point::new(6.0, 2.0),
        Point::new(10.0, 2.0),
        Point::new(12.0, 0.0),
    ];
    assert_eq!(classify_cubic(&quad), SkCubicType::Quadratic);

    let mut chops = [Point::default(); 10];
    // One inflection at t=0.5, so two chopped cubics.
    assert_eq!(sk_chop_cubic_at_inflections(&cusp_inf, &mut chops), 2);
    // Two inflections inside (0, 1) produce three cubics.
    let two_inflections = [
        Point::new(0.0, 0.0),
        Point::new(-4.0, -4.0),
        Point::new(-4.0, -3.0),
        Point::new(-3.0, -4.0),
    ];
    assert_eq!(
        sk_find_cubic_inflections(&two_inflections, &mut [0.0; 2]),
        2
    );
    assert_eq!(sk_chop_cubic_at_inflections(&two_inflections, &mut chops), 3);

    let mut max_curves = [Point::default(); 13];
    let mut ts = [0.0; 3];
    let src = [
        Point::new(2190.0, 5130.0),
        Point::new(2190.0, 5070.0),
        Point::new(2220.0, 5010.0),
        Point::new(2205.0, 4980.0),
    ];
    let count = sk_chop_cubic_at_max_curvature(&src, Some(&mut max_curves), Some(&mut ts));
    assert!(count >= 1);

    // SkFindCubicCusp: no cusps for these, cusps for the kCusps set.
    let no_cusp = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(2.0, 2.0),
        Point::new(3.0, 3.0),
    ];
    assert!(sk_find_cubic_cusp(&no_cusp) < 0.0);
    assert!(sk_find_cubic_cusp(&cusp) > 0.0);
}

#[test]
fn dev_conic_crazy_weights() {
    let max = f32::MAX;
    let inf = f32::INFINITY;
    let nanq = f32::NAN;
    let weights = [0.0, 1.0 / 65535.0, 1.0, 65535.0, max / 4.0, max / 2.0, max, inf, nanq];

    const LENGTH: f32 = 100.0;
    let mut conic = SkConic::new(
        Point::new(0.0, LENGTH),
        Point::new(0.0, 0.0),
        Point::new(LENGTH, 0.0),
        1.0,
    );

    for sign in [1.0f32, -1.0] {
        for w in weights {
            conic.w = w * sign;
            let pow2 = conic.compute_quad_pow2(0.0);
            assert!((0..=30).contains(&pow2));
            let num_quads = 1usize << pow2;
            let mut pts = vec![Point::default(); num_quads * 2 + 1];
            let num_quads2 = conic.chop_into_quads_pow2(&mut pts, pow2);
            assert!(num_quads2 as usize <= num_quads);
            let num_points2 = num_quads2 as usize * 2 + 1;
            for p in pts.iter().take(num_points2) {
                assert!(p.is_finite(), "non-finite point {p:?} for w {w}");
                let sum = p.x + p.y;
                assert!(p.x >= 0.0 && p.y >= 0.0 && sum <= LENGTH + 1.0 / 32768.0);
            }
        }
    }

    // SkAutoConicToQuads with non-positive weight degenerates to a line quad.
    let mut quadder = SkAutoConicToQuads::new();
    let tri = [Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(1.0, 1.0)];
    let quads = quadder.compute_quads_pts(&tri, 0.0, 0.25);
    assert_eq!(((quads.len() - 1) / 2) as i32, 1);
    assert_eq!(quads.len(), 3);
    assert_eq!(quads[1], Point::new(0.5, 0.5));
    assert_eq!(quadder.count_quads(), 1);
}

#[test]
fn dev_conic_eval_and_quads() {
    let mut quadder = SkAutoConicToQuads::new();
    let tri = [Point::new(0.0, 0.0), Point::new(100000.0, 0.0), Point::new(100000.0, 100000.0)];
    let mut w = 1e30f32;
    loop {
        w *= 2.0;
        let quads = quadder.compute_quads_pts(&tri, w, 0.25);
        assert!(quads.iter().all(|p| p.is_finite()));
        if !w.is_finite() {
            break;
        }
    }

    let conic = SkConic::new(
        Point::new(10.0, 20.0),
        Point::new(15.0, 25.0),
        Point::new(20.0, 30.0),
        0.707,
    );
    for i in 1..128 {
        let t = i as f32 / 128.0;
        let p = conic.eval_at(t);
        let mut p2 = Point::default();
        conic.eval_at_with(t, Some(&mut p2), None);
        assert_eq!(p, p2);
        assert!(near(conic.find_mid_tangent(), 0.5));
    }
    let bounds = conic.compute_tight_bounds();
    assert!(bounds.left() <= 10.0 && bounds.right() >= 20.0);
}

#[test]
fn dev_chop_quads_and_cubics() {
    let pts = [Point::new(0.0, 0.0), Point::new(100.0, 50.0), Point::new(0.0, 100.0)];
    let mut dst = [Point::default(); 5];
    let count = sk_chop_quad_at_max_curvature(&pts, &mut dst);
    assert!(count == 1 || count == 2);

    // NaN max-curvature t must not assert and must copy the source.
    let pts = [
        Point::new(15.1213, 7.77647),
        Point::new(6.2168e+19, 1.51338e+20),
        Point::new(1.4579e+19, 1.55558e+21),
    ];
    let mut in_place = [pts[0], pts[1], pts[2], Point::default(), Point::default()];
    let _ = sk_chop_quad_at_max_curvature_in_place(&mut in_place);

    let src = [
        Point::new(0.0, 0.0),
        Point::new(3.0, 0.0),
        Point::new(3.0, 3.0),
        Point::default(),
    ];
    let mut cubic = [Point::default(); 4];
    sk_convert_quad_to_cubic(&src[..3], &mut cubic);
    let expected = [
        Point::new(0.0, 0.0),
        Point::new(2.0, 0.0),
        Point::new(3.0, 1.0),
        Point::new(3.0, 3.0),
    ];
    for i in 0..4 {
        assert!(near(cubic[i].x, expected[i].x) && near(cubic[i].y, expected[i].y));
    }

    // Multi-chop: output points lie on the curve, and degenerate middle for equal Ts.
    let src = [
        Point::new(10.0, 20.0),
        Point::new(15.0, 35.0),
        Point::new(30.0, 40.0),
        Point::new(50.0, 10.0),
    ];
    let ts = [0.0, 1.0 / 3.0, 0.5, 0.7, 1.0];
    let mut all = [Point::default(); 4 + 5 * 3];
    sk_chop_cubic_at_ts(&src, &mut all, &ts);
    for (i, t) in ts.iter().enumerate() {
        let mut expected = Point::default();
        sk_eval_cubic_at(&src, *t, Some(&mut expected), None, None);
        let got = all[3 + i * 3];
        assert!(near(got.x, expected.x) && near(got.y, expected.y), "t={t} {got:?} {expected:?}");
    }

    let mut chops = [Point::default(); 10];
    sk_chop_cubic_at_two(&src, &mut chops, 0.25, 0.25);
    assert_eq!(chops[3], chops[4]);
    assert_eq!(chops[3], chops[5]);
    assert_eq!(chops[3], chops[6]);
}

#[test]
fn dev_inset_and_offset_polygons() {
    let rrect = rrect_poly();
    assert!(sk_is_convex_polygon(&rrect));
    assert_eq!(sk_get_polygon_winding(&rrect), -1);

    let mut inset = Vec::new();
    assert!(sk_inset_convex_polygon(&rrect, 10.0, &mut inset));
    assert_eq!(inset.len(), 4);
    assert!(inset[0].equals(-95.0, 45.0), "{:?}", inset);
    assert!(inset[1].equals(95.0, 45.0));
    assert!(inset[2].equals(95.0, -45.0));
    assert!(inset[3].equals(-95.0, -45.0));

    assert!(!sk_inset_convex_polygon(&rrect, 55.0, &mut inset));
    assert_eq!(inset.len(), 2);
    assert!(inset[0].equals(-50.0, 0.0), "{:?}", inset);
    assert!(inset[1].equals(50.0, 0.0));

    assert!(!sk_inset_convex_polygon(&rrect, 75.0, &mut inset));
    assert_eq!(inset.len(), 1);

    let mut bounds = Rect::default();
    assert!(bounds.set_bounds_check(&rrect));
    let mut offset = Vec::new();
    let mut indices = Vec::new();
    assert!(sk_offset_simple_polygon(
        &rrect,
        &bounds,
        10.0,
        &mut offset,
        Some(&mut indices)
    ));
    assert_eq!(offset.len(), 4);
    assert!(offset[0].equals(-95.0, 45.0), "{:?}", offset);
    assert!(offset[3].equals(-95.0, -45.0));
    assert_eq!(indices.len(), 4);

    assert!(!sk_offset_simple_polygon(&rrect, &bounds, 55.0, &mut offset, None));
    assert_eq!(offset.len(), 2);

    let mut star = Vec::new();
    for (x, y) in [
        (0.0, -50.0),
        (14.43, -25.0),
        (43.30, -25.0),
        (28.86, 0.0),
        (43.30, 25.0),
        (14.43, 25.0),
        (0.0, 50.0),
    ] {
        star.push(Point::new(x, y));
    }
    assert!(!sk_is_convex_polygon(&star));
    assert!(sk_is_simple_polygon(&star));
}

#[test]
fn dev_triangulate_and_simple() {
    let mut index_map = [0u16; 1024];
    for (i, v) in index_map.iter_mut().enumerate() {
        *v = i as u16;
    }
    let mut poly = vec![
        Point::new(-100.0, 55.0),
        Point::new(100.0, 55.0),
        Point::new(102.5, 54.330127),
    ];
    assert_eq!(sk_get_polygon_winding(&poly), -1);
    assert!(sk_is_convex_polygon(&poly));
    assert!(sk_is_simple_polygon(&poly));
    let mut triangles = Vec::new();
    assert!(sk_triangulate_simple_polygon(&poly, &index_map, &mut triangles));
    assert_eq!(triangles.len(), 3);

    poly[2] = Point::new(102.5, 55.330127);
    assert_eq!(sk_get_polygon_winding(&poly), 1);

    poly[2] = Point::new(100.0 + 2.5, 55.0);
    assert_eq!(sk_get_polygon_winding(&poly), 0);
    triangles.clear();
    assert!(!sk_triangulate_simple_polygon(&poly, &index_map, &mut triangles));

    // Concave round rect: simplicity sweep and triangulation.
    let rrect = rrect_poly();
    assert_eq!(sk_get_polygon_winding(&rrect), -1);
    assert!(sk_is_convex_polygon(&rrect));
    assert!(sk_is_simple_polygon(&rrect));
    triangles.clear();
    assert!(sk_triangulate_simple_polygon(&rrect, &index_map, &mut triangles));
    assert_eq!(triangles.len(), 3 * (rrect.len() - 2));

    // Self-intersecting bowtie must be rejected by the sweep line.
    let bowtie = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 0.0),
        Point::new(0.0, 10.0),
    ];
    assert!(!sk_is_simple_polygon(&bowtie));
}

#[test]
fn dev_clippers() {
    // Line clipping through a 0..10 square.
    let clip = Rect::new(0.0, 0.0, 10.0, 10.0);
    let pts = [Point::new(-5.0, 5.0), Point::new(15.0, 5.0)];
    let mut out = [Point::default(); 2];
    assert!(intersect_line(&pts, &clip, &mut out));
    assert_eq!(out[0], Point::new(0.0, 5.0));
    assert_eq!(out[1], Point::new(10.0, 5.0));

    // The out-of-clip portions become vertical segments on the clip edges.
    let mut lines = [Point::default(); 4];
    assert_eq!(clip_line(&pts, &clip, &mut lines, false), 3);
    assert_eq!(lines[0], Point::new(0.0, 5.0));
    assert_eq!(lines[1], Point::new(0.0, 5.0));
    assert_eq!(lines[2], Point::new(10.0, 5.0));
    assert_eq!(lines[3], Point::new(10.0, 5.0));

    let above = [Point::new(-5.0, -5.0), Point::new(15.0, -5.0)];
    assert_eq!(clip_line(&above, &clip, &mut lines, false), 0);
    assert!(!intersect_line(&above, &clip, &mut out));

    // Edge clipper: a line through the square produces one Line verb.
    let mut clipper = SkEdgeClipper::new(false);
    assert!(clipper.clip_line(Point::new(-5.0, 5.0), Point::new(15.0, 5.0), &clip));
    let mut verb_pts = [Point::default(); 4];
    // The portions outside the clip become degenerate vertical segments at the clip edges.
    for _ in 0..3 {
        assert_eq!(clipper.next(&mut verb_pts), Some(SkPathVerb::Line));
    }
    assert_eq!(clipper.next(&mut verb_pts), None);

    // Cubic clipper: monotonic Y cubic clipped to 0..10.
    let mut cubic_clipper = SkCubicClipper::new();
    cubic_clipper.set_clip(&IRect::new(0, 0, 10, 10));
    let cubic = [
        Point::new(1.0, 1.0),
        Point::new(5.0, 2.0),
        Point::new(7.0, 4.0),
        Point::new(8.0, 7.0),
    ];
    let mut clipped = [Point::default(); 4];
    assert!(cubic_clipper.clip_cubic(&cubic, &mut clipped));
    assert!(clipped[0].y >= 0.0 && clipped[3].y <= 10.0);

    // Quad clipper: quad spanning the top edge.
    let mut quad_clipper = SkQuadClipper::new();
    quad_clipper.set_clip(&IRect::new(0, 0, 10, 10));
    let quad = [
        Point::new(1.0, -2.0),
        Point::new(5.0, -2.0),
        Point::new(9.0, 8.0),
    ];
    let mut clipped_quad = [Point::default(); 3];
    assert!(quad_clipper.clip_quad(&quad, &mut clipped_quad));
    assert!(clipped_quad[0].y >= 0.0);
}

#[test]
fn dev_cubic_map() {
    let values = [0.0f32, 1.0, 0.5, 0.0000001, 0.999999];
    for x0 in values {
        for y0 in values {
            for x1 in values {
                for y1 in values {
                    let cmap = SkCubicMap::new(Point::new(x0, y0), Point::new(x1, y1));
                    let mut prev_y = 0.0f32;
                    let dx = 1.0 / 512.0;
                    let mut x = dx;
                    while x < 1.0 {
                        let y = cmap.compute_y_from_x(x);
                        assert!(prev_y <= y || (prev_y - y).abs() <= 1.0 / 4096.0);
                        prev_y = y;
                        x += dx;
                    }
                    // compute_from_t must reproduce the bezier evaluation.
                    let t = 0.25;
                    let p = cmap.compute_from_t(t);
                    let p0 = Point::new(0.0, 0.0);
                    let p3 = Point::new(1.0, 1.0);
                    let p1 = Point::new(x0, y0);
                    let p2 = Point::new(x1, y1);
                    let one_t = 1.0 - t;
                    let expected = p0 * (one_t * one_t * one_t)
                        + p1 * (3.0 * t * one_t * one_t)
                        + p2 * (3.0 * t * t * one_t)
                        + p3 * (t * t * t);
                    assert!(near(p.x, expected.x) && near(p.y, expected.y));
                }
            }
        }
    }
}

#[test]
fn cmap_debug() {
    let m = SkCubicMap::new(Point::new(0.0, 0.0), Point::new(1.0, 0.0));
    for t in [0.25f32, 0.5, 0.75] {
        let q = m.compute_from_t(t);
        println!("t={t} -> {} {}", q.x, q.y);
    }
    for x in [0.1f32, 0.5, 0.9] {
        println!("x={x} -> {}", m.compute_y_from_x(x));
    }
}
