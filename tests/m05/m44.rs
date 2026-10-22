// M44Test.cpp:33,79,103,144,198,374. BSD: tests/LICENSE.skia.
// M44_mapRect (:269) remains at M07, including its path-builder comparison.
use super::*;
fn eq(a: M44, b: M44, tol: f32) -> bool {
    let fa = a.get_col_major();
    let fb = b.get_col_major();
    (0..16).all(|i| (fa[i] - fb[i]).abs() <= tol)
}
fn rows(a: [f32; 16]) -> M44 {
    M44::RowMajor(&a)
}
#[test]
fn M44() {
    let mut r = Reporter::default();
    let mut m = M44::I();
    let mut im = M44::I();
    check!(
        r,
        rows([
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.
        ]) == m
    );
    check!(r, M44::I() == m);
    check!(r, m.invert(Some(&mut im)));
    check!(r, M44::I() == im);
    m.set_translate(3., 4., 2.);
    check!(
        r,
        rows([
            1., 0., 0., 3., 0., 1., 0., 4., 0., 0., 1., 2., 0., 0., 0., 1.
        ]) == m
    );
    let f = [
        1., 0., 0., 2., 3., 1., 2., 5., 0., 5., 3., 0., 0., 1., 0., 2.,
    ];
    m = M44::ColMajor(&f);
    check!(
        r,
        rows([
            f[0], f[4], f[8], f[12], f[1], f[5], f[9], f[13], f[2], f[6], f[10], f[14], f[3], f[7],
            f[11], f[15]
        ]) == m
    );
    {
        let t = m.transpose();
        check!(r, t != m);
        check!(r, t.rc(1, 0) == m.rc(0, 1));
        let tt = t.transpose();
        check!(r, tt == m);
    }
    m = M44::RowMajor(&f);
    // Preserve upstream's f[14] at row 2 column 3 (both f[11] and f[14] are zero).
    check!(
        r,
        rows([
            f[0], f[1], f[2], f[3], f[4], f[5], f[6], f[7], f[8], f[9], f[10], f[14], f[12], f[13],
            f[14], f[15]
        ]) == m
    );
    check!(r, m.invert(Some(&mut im)));
    m = m * im;
    check!(r, eq(M44::I(), m, 0.0000005));
    check!(r, M44::I() != m);
    r.finish();
}
#[test]
fn M44_v3() {
    let mut r = Reporter::default();
    let a = V3::new(1., 2., 3.);
    let b = V3::new(1., 2., 2.);
    check!(r, a.length_squared() == 1. + 4. + 9.);
    check!(r, b.length() == 3.);
    check!(r, a.dot(&b) == 1. + 4. + 6.);
    check!(r, b.dot(&a) == 1. + 4. + 6.);
    check!(r, a.cross(&b) == V3::new(-2., 1., 0.));
    check!(r, b.cross(&a) == V3::new(2., -1., 0.));
    let m = rows([
        2., 0., 0., 3., 0., 1., 0., 5., 0., 0., 3., 1., 0., 0., 0., 1.,
    ]);
    let c = m * a;
    check!(r, c == V3::new(2., 2., 9.));
    let d = m.map(4., 3., 2., 1.);
    check!(r, d == V4::new(11., 8., 7., 1.));
    r.finish();
}
#[test]
fn M44_v4() {
    let mut r = Reporter::default();
    let m = rows([
        1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15., 16.,
    ]);
    let (r0, r1, r2, r3) = (m.row(0), m.row(1), m.row(2), m.row(3));
    check!(r, r0 == V4::new(1., 2., 3., 4.));
    check!(r, r1 == V4::new(5., 6., 7., 8.));
    check!(r, r2 == V4::new(9., 10., 11., 12.));
    check!(r, r3 == V4::new(13., 14., 15., 16.));
    check!(r, M44::Rows(r0, r1, r2, r3) == m);
    let (c0, c1, c2, c3) = (m.col(0), m.col(1), m.col(2), m.col(3));
    check!(r, c0 == V4::new(1., 5., 9., 13.));
    check!(r, c1 == V4::new(2., 6., 10., 14.));
    check!(r, c2 == V4::new(3., 7., 11., 15.));
    check!(r, c3 == V4::new(4., 8., 12., 16.));
    check!(r, M44::Cols(c0, c1, c2, c3) == m);
    let v = V4::new(1., 2., 3., 4.);
    let v1 = m * v;
    let v2 = c0 * v.x + c1 * v.y + c2 * v.z + c3 * v.w;
    check!(r, v1 == v2);
    check!(
        r,
        c0 + r0 == V4::new(c0.x + r0.x, c0.y + r0.y, c0.z + r0.z, c0.w + r0.w)
    );
    check!(
        r,
        c0 - r0 == V4::new(c0.x - r0.x, c0.y - r0.y, c0.z - r0.z, c0.w - r0.w)
    );
    check!(
        r,
        c0 * r0 == V4::new(c0.x * r0.x, c0.y * r0.y, c0.z * r0.z, c0.w * r0.w)
    );
    r.finish();
}
#[test]
fn M44_rotate() {
    let mut r = Reporter::default();
    let x = V3::new(1., 0., 0.);
    let y = V3::new(0., 1., 0.);
    let z = V3::new(0., 0., 1.);
    let recs = [
        (0., 1., x, x, y, z),
        (0., 1., y, x, y, z),
        (0., 1., z, x, y, z),
        (0., -1., x, x, -y, -z),
        (0., -1., y, -x, y, -z),
        (0., -1., z, -x, -y, z),
        (1., 0., x, x, z, -y),
        (1., 0., y, -z, y, x),
        (1., 0., z, y, -x, z),
        (-1., 0., x, x, -z, y),
        (-1., 0., y, z, y, -x),
        (-1., 0., z, -y, x, z),
    ];
    for (sin, cos, axis, ex, ey, ez) in recs {
        let mut m = M44::nan();
        m.set_rotate_unit_sin_cos(axis, sin, cos);
        let mut mx = m * x;
        let mut my = m * y;
        let mut mz = m * z;
        check!(r, mx == ex);
        check!(r, my == ey);
        check!(r, mz == ez);
        mx = m * -x;
        my = m * -y;
        mz = m * -z;
        check!(r, mx == -ex);
        check!(r, my == -ey);
        check!(r, mz == -ez);
    }
    r.finish();
}
fn map2d(r: &mut Reporter, m: &M44, x: f32, y: f32) -> V2 {
    let p = m.map(x, y, 0., 1.);
    check!(r, p.z == 0.);
    check!(r, p.w == 1.);
    V2::new(p.x, p.y)
}
fn edges(r: &mut Reporter, lo: f32, hi: f32, elo: f32, ehi: f32) {
    debug_assert!(elo < ehi);
    check!(r, lo < hi);
    check!(r, near(lo, elo));
    check!(r, near(hi, ehi));
}
#[test]
fn M44_rectToRect() {
    let mut r = Reporter::default();
    let mut rand = SkRandom::new();
    for (sx, sy) in [
        (1., 1.),
        (0.25, 0.5),
        (0.5, 0.25),
        (0.5, 0.5),
        (2., 3.),
        (3., 2.),
        (2., 2.),
        (0.5, 2.),
        (2., 0.5),
    ] {
        let src = Rect::make_xywh(
            rand.next_range_f(-10., 10.),
            rand.next_range_f(-10., 10.),
            rand.next_range_f(1., 10.),
            rand.next_range_f(1., 10.),
        );
        let dst = Rect::make_xywh(
            rand.next_range_f(-10., 10.),
            rand.next_range_f(-10., 10.),
            sx * src.width(),
            sy * src.height(),
        );
        let m = M44::RectToRect(&src, &dst);
        let c = map2d(&mut r, &m, src.center_x(), src.center_y());
        check!(r, near(c.x, dst.center_x()));
        check!(r, near(c.y, dst.center_y()));
        let tl = map2d(&mut r, &m, src.left, src.top);
        let tr = map2d(&mut r, &m, src.right, src.top);
        let br = map2d(&mut r, &m, src.right, src.bottom);
        let bl = map2d(&mut r, &m, src.left, src.bottom);
        edges(&mut r, tl.x, tr.x, dst.left, dst.right);
        edges(&mut r, bl.x, br.x, dst.left, dst.right);
        edges(&mut r, tl.y, bl.y, dst.top, dst.bottom);
        edges(&mut r, tr.y, br.y, dst.top, dst.bottom);
    }
    r.finish();
}
#[test]
fn M44_mapRect_skbug12335() {
    let mut r = Reporter::default();
    let rect = Rect::new(0., 0., 319., 620.);
    let m = M44::from(Matrix::MakeAll(
        0.000152695269,
        0.,
        -6.53848401e-05,
        -1.75697533e-05,
        0.000157153074,
        -1.10847975e-06,
        -6.00415362e-08,
        0.,
        0.000169880834,
    ));
    let out = matrix_priv::map_rect(&m, &rect);
    check!(r, !out.is_empty());
    r.finish();
}
