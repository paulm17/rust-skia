// CubicMapTest.cpp:63 and complete helper closure. BSD license: tests/LICENSE.skia.
use super::*;
fn accurate_t(a: f32, b: f32, c: f32, d: f32) -> f32 {
    let mut roots = [0.; 3];
    let count = SkCubics::roots_valid_t(a as f64, b as f64, c as f64, d as f64, &mut roots);
    debug_assert_eq!(count, 1);
    roots[0] as f32
}
fn accurate_solve(p1: Point, p2: Point, x: f32) -> f32 {
    let array = [p(0., 0.), p1, p2, p(1., 1.)];
    let coeff = SkCubicCoeff::new(&array);
    let t = accurate_t(coeff.a.x, coeff.b.x, coeff.c.x, coeff.d.x - x);
    debug_assert!(t >= 0. && t <= 1.);
    let y = coeff.eval(t).y;
    debug_assert!(y >= 0. && y <= 1.0001);
    y
}
fn nearly_le(a: f32, b: f32) -> bool {
    a <= b || near(a - b, 0.)
}
fn exercise_cubicmap(p1: Point, p2: Point, r: &mut Reporter) {
    let cmap = SkCubicMap::new(p1, p2);
    let mut prev_y = 0.;
    let dx = 1.0f32 / 512.;
    let mut x = dx;
    while x < 1. {
        let y = cmap.compute_y_from_x(x);
        if !nearly_le(prev_y, y) {
            cmap.compute_y_from_x(x);
            check!(r, false);
        }
        prev_y = y;
        let yy = accurate_solve(p1, p2, x);
        let diff = (yy - y).abs();
        check!(r, diff < 0.008);
        x += dx;
    }
}
#[test]
fn CubicMap() {
    let mut r = Reporter::default();
    let values = [0., 1., 0.5, 0.0000001, 0.999999];
    for x0 in values {
        for y0 in values {
            for x1 in values {
                for y1 in values {
                    exercise_cubicmap(p(x0, y0), p(x1, y1), &mut r);
                }
            }
        }
    }
    r.finish();
}
