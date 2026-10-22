// BezierCurveTest.cpp:52,107,121,201. BSD license: tests/LICENSE.skia.
use super::*;
use rust_skia::floating_point::{double_nearly_zero, doubles_nearly_equal_ulps_with};
fn nearly_equal(expected: f64, actual: f64) -> bool {
    if double_nearly_zero(expected) {
        double_nearly_zero(actual)
    } else {
        doubles_nearly_equal_ulps_with(expected, actual, 64)
    }
}
fn testCubicEvalAtT(r: &mut Reporter, name: &str, input: &[f64; 8], t: f64, expected: [f64; 2]) {
    eprintln!("{name}");
    check!(r, input.len() / 2 == 4);
    check!(r, t >= 0.0 && t <= 1.0);
    let [x, y] = SkBezierCubic::eval_at(input, t);
    check!(r, nearly_equal(expected[0], x));
    check!(r, nearly_equal(expected[1], y));
}
#[test]
fn BezierCubicEvalAt() {
    let mut r = Reporter::default();
    testCubicEvalAtT(
        &mut r,
        "linear curve @0.1234",
        &[0., 0., 0., 0., 10., 10., 10., 10.],
        0.1234,
        [0.4192451819200000, 0.4192451819200000],
    );
    testCubicEvalAtT(
        &mut r,
        "linear curve @0.2345",
        &[0., 0., 5., 5., 5., 5., 10., 10.],
        0.2345,
        [2.8215983862500000, 2.8215983862500000],
    );
    let input = [-10., -20., -7., 5., 14., -2., 3., 13.];
    testCubicEvalAtT(&mut r, "Arbitrary Cubic, t=0.0", &input, 0., [-10., -20.]);
    testCubicEvalAtT(
        &mut r,
        "Arbitrary Cubic, t=0.3456",
        &input,
        0.3456,
        [-2.503786700800000, -3.31715344793600],
    );
    testCubicEvalAtT(&mut r, "Arbitrary Cubic, t=0.5", &input, 0.5, [1.75, 0.25]);
    testCubicEvalAtT(
        &mut r,
        "Arbitrary Cubic, t=0.7891",
        &input,
        0.7891,
        [6.158763291450000, 5.938550084434000],
    );
    testCubicEvalAtT(&mut r, "Arbitrary Cubic, t=1.0", &input, 1., [3., 13.]);
    r.finish();
}
fn testCubicConvertToPolynomial(
    r: &mut Reporter,
    name: &str,
    input: &[f64; 8],
    y: bool,
    expected: [f64; 4],
) {
    eprintln!("{name}: SkBezierCurve Implementation");
    check!(r, input.len() / 2 == 4);
    let actual = SkBezierCubic::convert_to_polynomial(input, y);
    for i in 0..4 {
        check!(r, nearly_equal(expected[i], actual[i]));
    }
}
#[test]
fn BezierCubicToPolynomials() {
    let mut r = Reporter::default();
    let input = [1., 2., -3., 4., 5., -6., 7., 8.];
    testCubicConvertToPolynomial(
        &mut r,
        "Arbitrary control points X direction",
        &input,
        false,
        [-18., 36., -12., 1.],
    );
    testCubicConvertToPolynomial(
        &mut r,
        "Arbitrary control points Y direction",
        &input,
        true,
        [36., -36., 6., 2.],
    );
    r.finish();
}
#[test]
fn QuadRoots_CheckTRange() {
    let mut r = Reporter::default();
    let roots = [
        -1000., -10., -1., -0.1, -0.0001, 0., 0.0001, 0.1, 0.9, 0.9999, 1., 1.0001, 1.1, 10., 1000.,
    ];
    let scales = [
        -1000., -10., -1., -0.1, -0.0001, 0.0001, 0.1, 1., 10., 1000.,
    ];
    let outside = |x: f64| x < 0. || 1. < x;
    let inside = |x: f64| !outside(x);
    let equal = |a: f64, b: f64| a as f32 == b as f32;
    for r1 in roots {
        for r0 in roots {
            for s in scales {
                let a = s;
                let b = s * 0.5 * (r0 + r1);
                let c = s * r0 * r1;
                let intersections = SkBezierQuad::intersect(0., -0.5, 0., a, b, c, 0.);
                if intersections.is_empty() {
                    check!(
                        r,
                        SkQuads::discriminant(a, b, c) < 0. || (outside(r0) && outside(r1))
                    );
                } else if intersections.len() == 1 {
                    check!(r, inside(r0) || inside(r1));
                    let root = if inside(r0) { r0 } else { r1 };
                    check!(r, equal(root, intersections[0] as f64));
                } else {
                    check!(r, intersections.len() == 2);
                    check!(r, inside(r0) && inside(r1));
                    let smaller = intersections[0].min(intersections[1]);
                    let bigger = intersections[0].max(intersections[1]);
                    check!(r, equal(smaller as f64, r0.min(r1)));
                    check!(r, equal(bigger as f64, r0.max(r1)));
                }
            }
        }
    }
    {
        let a = 0.;
        let b = -2.;
        let c = -1.;
        let intersections = SkBezierQuad::intersect(0., -0.5, 0., a, b, c, 0.);
        check!(r, intersections.len() == 1);
        check!(r, intersections[0] == 0.25);
    }
    r.finish();
}
#[test]
fn SkBezierCubic_CheckTRange() {
    let mut r = Reporter::default();
    let roots = [-10., -5., -2., -1., 0., 0.5, 1., 2., 5., 10.];
    let scales = [
        -1000., -10., -1., -0.1, -0.0001, 0.0001, 0.1, 1., 10., 1000.,
    ];
    let outside = |x: f64| x < 0. || 1. < x;
    let inside = |x: f64| !outside(x);
    let specialEqual = |actual: f64, test: f64| {
        (test - actual).abs() <= f32::EPSILON as f64 * test.abs().max(actual.abs())
    };
    for r2 in roots {
        for r1 in roots {
            for r0 in roots {
                for s in scales {
                    let a = s;
                    let b = -s * (r0 + r1 + r2);
                    let c = s * (r0 * r1 + r1 * r2 + r0 * r2);
                    let d = -s * r0 * r1 * r2;
                    // Sorted unique values preserve std::set<double>'s iteration order.
                    let mut inRangeRoots = Vec::new();
                    for v in [r0, r1, r2] {
                        if inside(v) {
                            inRangeRoots.push(v);
                        }
                    }
                    inRangeRoots.sort_by(f64::total_cmp);
                    inRangeRoots.dedup();
                    let intersections = SkBezierCubic::intersect(0., 0., 1., 0., a, b, c, d, 0.);
                    let mut correct = 0;
                    for candidate in &intersections {
                        for answer in &inRangeRoots {
                            if specialEqual(*candidate as f64, *answer) {
                                correct += 1;
                                break;
                            }
                        }
                    }
                    check!(r, correct == intersections.len());
                }
            }
        }
    }
    r.finish();
}
