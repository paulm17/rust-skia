// Point3Test.cpp:126 and its full helper closure. BSD license: tests/LICENSE.skia.
use super::*;
use rust_skia::scalar::{SCALAR_ROOT2_OVER2, scalar_nearly_equal as near};
fn test_eq_ops(r: &mut Reporter) {
    let p0 = Point3::make(0.0, 0.0, 0.0);
    let p1 = Point3::make(1.0, 1.0, 1.0);
    let p2 = Point3::make(1.0, 1.0, 1.0);
    check!(r, p0 != p1);
    check!(r, p1 == p2);
}
fn test_ops(r: &mut Reporter) {
    let mut v = Point3::make(1.0, 1.0, 1.0);
    v.normalize();
    check!(r, near(v.length(), 1.0));
    let mut p = v.make_scale(3.0);
    check!(r, near(p.length(), 3.0));
    p.scale(1.0 / 3.0);
    check!(r, near(p.length(), 1.0));
    let p1 = Point3::make(20.0, 2.0, 10.0);
    let p2 = -p1;
    p = p1 - p1;
    check!(r, near(p.x(), 0.0));
    check!(r, near(p.y(), 0.0));
    check!(r, near(p.z(), 0.0));
    p = p1 + p2;
    check!(r, near(p.x(), 0.0));
    check!(r, near(p.y(), 0.0));
    check!(r, near(p.z(), 0.0));
}
fn test_dot(r: &mut Reporter) {
    let x_axis = Point3::make(1.0, 0.0, 0.0);
    let y_axis = Point3::make(0.0, 1.0, 0.0);
    let z_axis = Point3::make(0.0, 0.0, 1.0);
    let mut dot = x_axis.dot(&y_axis);
    check!(r, near(dot, 0.0));
    dot = y_axis.dot(&z_axis);
    check!(r, near(dot, 0.0));
    dot = z_axis.dot(&x_axis);
    check!(r, near(dot, 0.0));
    let mut v = Point3::make(13.0, 2.0, 7.0);
    v.normalize();
    dot = v.dot(&v);
    check!(r, near(dot, 1.0));
    v = Point3::make(SCALAR_ROOT2_OVER2, SCALAR_ROOT2_OVER2, 0.0);
    dot = x_axis.dot(&v);
    check!(r, near(dot, SCALAR_ROOT2_OVER2));
    dot = y_axis.dot(&v);
    check!(r, near(dot, SCALAR_ROOT2_OVER2));
}
fn test_length(r: &mut Reporter, x: f32, y: f32, z: f32, expected_len: f32) {
    let point = Point3::make(x, y, z);
    let s1 = point.length();
    let s2 = Point3::length_xyz(x, y, z);
    check!(r, near(s1, s2));
    check!(r, near(s1, expected_len));
}
fn test_normalize(r: &mut Reporter, x: f32, y: f32, z: f32, expected_len: f32) {
    let mut point = Point3::make(x, y, z);
    let result = point.normalize();
    let new_length = point.length();
    if expected_len == 0.0 {
        let empty = Point3::make(0.0, 0.0, 0.0);
        check!(r, near(new_length, 0.0));
        check!(r, !result);
        check!(r, point == empty);
    } else {
        check!(r, near(new_length, 1.0));
        check!(r, result);
    }
    let mut random = SkRandom::new();
    random.set_seed(1234);
    let mut pt3 = Point3::default();
    let test_count = 100000;
    for _index in 0..test_count {
        let mut test_val;
        loop {
            test_val = random.next_range_f(0.0, 2.0);
            if test_val != 0.0 {
                break;
            }
        }
        pt3.set(test_val, 0.0, 0.0);
        check!(r, !pt3.normalize() || 1.0 == pt3.x());
    }
}
#[test]
fn Point3() {
    let mut r = Reporter::default();
    test_eq_ops(&mut r);
    test_ops(&mut r);
    test_dot(&mut r);
    let g_rec = [
        (0.0, 0.0, 0.0, 0.0),
        (0.3, 0.4, 0.5, SCALAR_ROOT2_OVER2),
        (1.0e-37, 1.0e-37, 1.0e-37, 0.0),
        (3.4e38, 0.0, 0.0, 3.4e38),
    ];
    for (x, y, z, length) in g_rec {
        test_length(&mut r, x, y, z, length);
        test_normalize(&mut r, x, y, z, length);
    }
    r.finish();
}
