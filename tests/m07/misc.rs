// Copyright 2011–2025 Google Inc. / Google LLC. BSD: tests/LICENSE.skia.
use super::*;
#[test]
fn IsClosedSingleContourTest() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    check!(
        r,
        !SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.close();
    check!(
        r,
        !SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.move_to_xy(10.0, 10.0).close();
    check!(
        r,
        SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.move_to_xy(10.0, 10.0).line_to_xy(20.0, 20.0).close();
    check!(
        r,
        SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 20.0)
        .quad_to_xy(30.0, 30.0, 40.0, 40.0)
        .cubic_to_xy(50.0, 50.0, 60.0, 60.0, 70.0, 70.0)
        .conic_to_xy(30.0, 30.0, 40.0, 40.0, 0.5)
        .close();
    check!(
        r,
        SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 20.0)
        .line_to_xy(20.0, 30.0);
    check!(
        r,
        !SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 20.0)
        .move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 30.0)
        .close();
    check!(
        r,
        !SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    b.move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 20.0)
        .close()
        .line_to_xy(20.0, 30.0)
        .close();
    check!(
        r,
        !SkPathPriv::is_closed_single_contour_verbs(b.detach(None).verbs())
    );
    r.finish();
}
#[test]
fn Rect_path_nan() {
    let mut r = Reporter::default();
    let bounds = rect(0.0, 0.0, f32::NAN, 100.0);
    check!(r, !SkPath::rect_default(&bounds).is_finite());
    r.finish();
}
fn next_point(rand: &mut SkRandom) -> Point {
    p(rand.next_f(), rand.next_f())
}
#[test]
fn SkPath_RangeIter() {
    let mut r = Reporter::default();
    let verbs = [
        None,
        Some(L),
        Some(K),
        Some(X),
        None,
        Some(C),
        Some(M),
        Some(K),
        Some(L),
        Some(X),
        Some(M),
    ];
    let mut rng = SkRandom::new();
    let mut test = SkRandom::new();
    for _ in 0..10 {
        if next_point(&mut rng) != next_point(&mut test) || rng.next_f() != test.next_f() {
            check!(r, false);
            r.finish();
            return;
        }
    }
    let mut b = SkPathBuilder::new();
    for v in verbs {
        match v {
            None => {}
            Some(M) => {
                b.move_to(next_point(&mut rng));
            }
            Some(L) => {
                b.line_to(next_point(&mut rng));
            }
            Some(Q) => {
                let a = next_point(&mut rng);
                let bpt = next_point(&mut rng);
                b.quad_to(a, bpt);
            }
            Some(C) => {
                let a = next_point(&mut rng);
                let bpt = next_point(&mut rng);
                let c = next_point(&mut rng);
                b.cubic_to(a, bpt, c);
            }
            Some(K) => {
                let a = next_point(&mut rng);
                let bpt = next_point(&mut rng);
                b.conic_to(a, bpt, rng.next_f());
            }
            Some(X) => {
                b.close();
            }
            _ => unreachable!(),
        }
    }
    let path = b.detach(None);
    let mut it = range(&path);
    let mut start = p(0.0, 0.0);
    let mut last = p(0.0, 0.0);
    for v in verbs {
        let (_pv, pts, w) = it.next().unwrap();
        match v {
            None => {
                check!(r, pts[0] == start);
                last = pts[0];
            }
            Some(M) => {
                check!(r, pts[0] == next_point(&mut test));
                start = pts[0];
                last = pts[0];
            }
            Some(L) => {
                check!(r, pts[0] == last);
                check!(r, pts[1] == next_point(&mut test));
                last = pts[1];
            }
            Some(Q) => {
                check!(r, pts[0] == last);
                check!(r, pts[1] == next_point(&mut test));
                check!(r, pts[2] == next_point(&mut test));
                last = pts[2];
            }
            Some(C) => {
                check!(r, pts[0] == last);
                check!(r, pts[1] == next_point(&mut test));
                check!(r, pts[2] == next_point(&mut test));
                check!(r, pts[3] == next_point(&mut test));
                last = pts[3];
            }
            Some(K) => {
                check!(r, pts[0] == last);
                check!(r, pts[1] == next_point(&mut test));
                check!(r, pts[2] == next_point(&mut test));
                check!(r, w == test.next_f());
                last = pts[2];
            }
            Some(X) => {
                check!(r, pts[0] == last);
            }
            _ => unreachable!(),
        }
    }
    check!(r, it.at_end());
    r.finish();
}
fn cheap_distance(dx: f32, dy: f32) -> i32 {
    let mut idx = rust_skia::scalar::scalar_round_to_int(dx).abs();
    let idy = rust_skia::scalar::scalar_round_to_int(dy).abs();
    if idx > idy {
        idx += idy >> 1;
    } else {
        idx = idy + (idx >> 1);
    }
    idx
}
fn estimate_distance(points: &[Point; 3]) -> i32 {
    cheap_distance(
        points[1].x * 2.0 - points[2].x - points[0].x,
        points[1].y * 2.0 - points[2].y - points[0].y,
    )
}
fn compute_distance(points: &[Point; 3]) -> f32 {
    rust_skia::point_priv::PointPriv::distance_to_line_segment_between(
        &points[1], &points[0], &points[2],
    )
}
fn estimate_pointCount(distance: i32) -> u32 {
    let mut shift = 30 - distance.leading_zeros() as i32;
    shift &= !(shift >> 31);
    if shift > 6 {
        shift = 6;
    }
    1 << shift
}
fn compute_pointCount(d: f32, tol: f32) -> u32 {
    if d < tol {
        1
    } else {
        let temp = rust_skia::scalar::scalar_ceil_to_int(rust_skia::scalar::scalar_sqrt(d / tol));
        (rust_skia::math_priv::sk_next_pow2(temp) as u32).min(1 << 6)
    }
}
fn quadraticPointCount_EE(points: &[Point; 3]) -> u32 {
    estimate_pointCount(estimate_distance(points))
}
fn quadraticPointCount_EC(points: &[Point; 3], tol: f32) -> u32 {
    compute_pointCount(estimate_distance(points) as f32, tol)
}
fn quadraticPointCount_CE(points: &[Point; 3]) -> u32 {
    estimate_pointCount(rust_skia::scalar::scalar_round_to_int(compute_distance(
        points,
    )))
}
fn quadraticPointCount_CC(points: &[Point; 3], tol: f32) -> u32 {
    compute_pointCount(compute_distance(points), tol)
}
fn one_d_pe(array: &[i32], r: &mut Reporter) -> bool {
    let mut path = [Point::default(); 3];
    path[1] = p(array[0] as f32, array[1] as f32);
    path[2] = p(array[2] as f32, array[3] as f32);
    let mut errors = 0;
    for i in (4..array.len()).step_by(2) {
        path[0] = path[1];
        path[1] = path[2];
        path[2] = p(array[i] as f32, array[i + 1] as f32);
        let mut computed = quadraticPointCount_CC(&path, 1.0);
        let mut estimated = quadraticPointCount_EE(&path);
        if false {
            computed = quadraticPointCount_EC(&path, 1.0);
            estimated = quadraticPointCount_CE(&path);
        }
        let accurate = estimated >= computed && estimated <= 2 * computed;
        if !accurate {
            eprintln!(
                "Curve from {:?} through {:?} to {:?} computes {computed}, estimates {estimated}",
                path[0], path[1], path[2]
            );
            check!(r, false);
            errors += 1;
        }
    }
    errors == 0
}
fn TestQuadPointCount(r: &mut Reporter) {
    one_d_pe(&[4, 0, 0, -4, 8, -4, 12, 0, 8, 4, 0, 4], r);
    one_d_pe(
        &[
            0, 0, 10, 10, 20, 20, 30, 10, 40, 0, 50, -10, 60, -20, 70, -10, 80, 0,
        ],
        r,
    );
    one_d_pe(&[0, 0, 5, 15, 20, 20, 35, 15, 40, 0], r);
    one_d_pe(&[0, 0, 1, 10, 2, 0, 3, -10, 4, 0], r);
    one_d_pe(&[-4, 0, 4, 20, 0, 25, -4, 20, 4, 0], r);
}
#[test]
fn PathCoverage() {
    let mut r = Reporter::default();
    TestQuadPointCount(&mut r);
    r.finish();
}
fn assertRectsNearlyEqual(r: &mut Reporter, actual: Rect, expected: Rect, e: Rect) {
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(actual.left, expected.left, e.left)
    );
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(actual.top, expected.top, e.top)
    );
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(actual.right, expected.right, e.right)
    );
    check!(
        r,
        rust_skia::scalar::scalar_nearly_equal_within(actual.bottom, expected.bottom, e.bottom)
    );
}
fn assertMapRect(r: &mut Reporter, m: &rust_skia::m44::M44, src: Rect, expected: Option<Rect>) {
    let mut epsilon = rect(1e-5, 1e-5, 1e-5, 1e-5);
    let actual = rust_skia::matrix_priv::map_rect(m, &src);
    check!(r, !actual.is_empty());
    if let Some(expected) = expected {
        assertRectsNearlyEqual(r, actual, expected, epsilon);
    }
    let corners = [
        (src.left, src.top),
        (src.right, src.top),
        (src.right, src.bottom),
        (src.left, src.bottom),
    ];
    let (mut lf, mut tf, mut rf, mut bf, mut clipped) = (false, false, false, false, false);
    for (cx, cy) in corners {
        let mapped = m.map(cx, cy, 0.0, 1.0);
        if mapped.w > 0.0 {
            let x = mapped.x / mapped.w;
            let y = mapped.y / mapped.w;
            check!(r, actual.left <= x && x <= actual.right);
            check!(r, actual.top <= y && y <= actual.bottom);
            lf |= near(x, actual.left);
            tf |= near(y, actual.top);
            rf |= near(x, actual.right);
            bf |= near(y, actual.bottom);
        } else {
            clipped = true;
        }
    }
    if clipped {
        check!(r, lf || tf || rf || bf);
        if !lf {
            epsilon.left = 0.01 * actual.left;
        }
        if !tf {
            epsilon.top = 0.01 * actual.top;
        }
        if !rf {
            epsilon.right = 0.01 * actual.right;
        }
        if !bf {
            epsilon.bottom = 0.01 * actual.bottom;
        }
    } else {
        check!(r, lf && tf && rf && bf);
    }
    let path = SkPathBuilder::new()
        .add_rect(&src, Cw, 0)
        .transform(&m.as_m33())
        .detach(None);
    assertRectsNearlyEqual(r, actual, *path.get_bounds(), epsilon);
}
#[test]
fn M44_mapRect() {
    use rust_skia::m44::{M44, V3, V4};
    use rust_skia::scalar::SCALAR_PI;
    let mut r = Reporter::default();
    let src = rect(4.83, -0.48, 5.53, 30.68);
    assertMapRect(&mut r, &M44::new(), src, Some(src));
    let st = rect(
        10.0 + 2.0 * src.left,
        8.0 + 4.0 * src.top,
        10.0 + 2.0 * src.right,
        8.0 + 4.0 * src.bottom,
    );
    let mut m = M44::scale_2d(2.0, 4.0);
    m.post_translate_2d(10.0, 8.0);
    assertMapRect(&mut r, &m, src, Some(st));
    let mut m = M44::Rotate(V3::new(0.0, 0.0, 1.0), SCALAR_PI / 4.0);
    m.pre_translate_2d(-src.center_x(), -src.center_y())
        .post_translate_2d(src.center_x(), src.center_y());
    assertMapRect(&mut r, &m, src, None);
    let mut m = M44::perspective(0.01, 10.0, SCALAR_PI / 3.0);
    m.pre_translate(0.0, 5.0, -0.1);
    m.pre_concat(&M44::Rotate(V3::new(0.0, 1.0, 0.0), 0.008));
    assertMapRect(&mut r, &m, src, None);
    m.set_identity();
    m.set_row(3, V4::new(-0.2, -0.6, 0.0, 8.0));
    assertMapRect(&mut r, &m, src, None);
    r.finish();
}
