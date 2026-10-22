// SizeTest.cpp:12,34. BSD license: tests/LICENSE.skia.
use super::*;
#[test]
fn ISize() {
    let mut r = Reporter::default();
    let mut a = ISize::default();
    let mut b = ISize::default();
    a.set(0, 0);
    check!(r, a.is_empty());
    a.set(5, -5);
    check!(r, a.is_empty());
    a = ISize::new(5, 0);
    check!(r, a.is_empty());
    b.set(5, 0);
    check!(r, a == b);
    a.set(3, 5);
    check!(r, !a.is_empty());
    b = a;
    check!(r, !b.is_empty());
    check!(r, a == b);
    check!(r, !(a != b));
    check!(r, a.width == b.width && a.height == b.height);
    r.finish();
}
#[test]
fn Size() {
    let mut r = Reporter::default();
    let mut a = Size::default();
    let mut b = Size::default();
    let ix = 5;
    let iy = 3;
    let x = rust_skia::scalar::int_to_scalar(ix);
    let y = rust_skia::scalar::int_to_scalar(iy);
    a.set(0.0, 0.0);
    check!(r, a.is_empty());
    a.set(x, -x);
    check!(r, a.is_empty());
    a = Size::new(x, 0.0);
    check!(r, a.is_empty());
    b.set(x, 0.0);
    check!(r, a == b);
    a.set(y, x);
    check!(r, !a.is_empty());
    b = a;
    check!(r, !b.is_empty());
    check!(r, a == b);
    check!(r, !(a != b));
    check!(r, a.width == b.width && a.height == b.height);
    let mut ia = ISize::default();
    ia.set(ix, iy);
    a.set(x, y);
    check!(r, a.to_round() == ia);
    r.finish();
}
