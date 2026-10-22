// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/SkVxTest.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
// Rust comparisons use mask methods; initializer lists zero-fill, parentheses splat.
use super::*;
use rust_skia::vx::{abs, round as vx_round};
macro_rules! f2 {
    ($a:expr,$b:expr) => {
        Float2::new([$a as f32, $b as f32])
    };
}
macro_rules! f4 {
    ($a:expr,$b:expr,$c:expr,$d:expr) => {
        Float4::new([$a as f32, $b as f32, $c as f32, $d as f32])
    };
}
macro_rules! i2 {
    ($a:expr,$b:expr) => {
        Int2::new([$a, $b])
    };
}
macro_rules! i4 {
    ($a:expr,$b:expr,$c:expr,$d:expr) => {
        Int4::new([$a, $b, $c, $d])
    };
}
macro_rules! eq {
    ($r:expr,$a:expr,$b:expr) => {
        check!($r, all(($a).eq_mask(&($b))))
    };
}
// tests/SkVxTest.cpp:18
#[test]
fn SkVx() {
    const _: () = {
        assert!(std::mem::size_of::<Float2>() == 8);
        assert!(std::mem::size_of::<Float4>() == 16);
        assert!(std::mem::size_of::<Float8>() == 32);
        assert!(std::mem::size_of::<Byte2>() == 2);
        assert!(std::mem::size_of::<Byte4>() == 4);
        assert!(std::mem::size_of::<Byte8>() == 8);
    };
    let mut r = Reporter::default();
    {
        let mask = f4!(1, 2, 3, 4).lt(&f4!(1, 2, 4, 8));
        check!(r, mask[0] == 0);
        check!(r, mask[1] == 0);
        check!(r, mask[2] == -1);
        check!(r, mask[3] == -1);
        check!(r, any(mask));
        check!(r, !all(mask));
    }
    {
        let mask = Double4::new([1., 2., 3., 4.]).lt(&Double4::new([1., 2., 4., 8.]));
        check!(r, mask[0] == 0);
        check!(r, mask[1] == 0);
        check!(r, mask[2] == -1);
        check!(r, mask[3] == -1);
        check!(r, any(mask));
        check!(r, !all(mask));
    }
    {
        check!(r, all(i4!(1, 2, 3, 4)));
        check!(r, !all(i4!(1, 2, 3, 0)));
        check!(r, any(i4!(1, 2, 0, 0)));
        check!(r, !any(Int4::default()));
    }
    check!(r, f4!(1, 2, 3, 4).min_lane() == 1.);
    check!(r, f4!(1, 2, 3, 4).max_lane() == 4.);
    eq!(r, i4!(1, 2, 3, 4), i4!(1, 2, 3, 4));
    eq!(r, Int4::from([1, 2, 3, 0]), i4!(1, 2, 3, 0));
    eq!(r, Int4::from([1, 2, 0, 0]), i4!(1, 2, 0, 0));
    eq!(r, Int4::from([1, 0, 0, 0]), i4!(1, 0, 0, 0));
    eq!(r, Int4::splat(1), i4!(1, 1, 1, 1));
    eq!(r, Int4::default(), i4!(0, 0, 0, 0));
    eq!(r, Int4::zero(), i4!(0, 0, 0, 0));
    eq!(r, i4!(1, 2, 2, 1), vx_min(i4!(1, 2, 3, 4), i4!(4, 3, 2, 1)));
    eq!(r, i4!(4, 3, 3, 4), vx_max(i4!(1, 2, 3, 4), i4!(4, 3, 2, 1)));
    eq!(
        r,
        if_then_else(
            f4!(1, 2, 3, 2).le(&f4!(2, 2, 2, 2)),
            Float4::splat(42.),
            Float4::splat(47.)
        ),
        f4!(42, 42, 47, 42)
    );
    eq!(r, floor(f4!(-1.5, 1.5, 1., -1.)), f4!(-2., 1., 1., -1.));
    eq!(r, ceil(f4!(-1.5, 1.5, 1., -1.)), f4!(-1., 2., 1., -1.));
    eq!(r, trunc(f4!(-1.5, 1.5, 1., -1.)), f4!(-1., 1., 1., -1.));
    eq!(r, vx_round(f4!(-1.5, 1.5, 1., -1.)), f4!(-2., 2., 1., -1.));
    eq!(r, abs(f4!(-2, -1, 0, 1)), f4!(2, 1, 0, 1));
    check!(r, all(sqrt(f4!(2, 3, 4, 5)).lt(&f4!(2, 2, 3, 3))));
    check!(r, all(sqrt(f2!(2, 3)).lt(&f2!(2, 2))));
    eq!(r, f4!(-1.5, 0.5, 1., 1.5).cast::<i32>(), i4!(-1, 0, 1, 1));
    let mut buf = [1f32, 2., 3., 4., 5., 6.];
    eq!(r, Float4::load(&buf), f4!(1, 2, 3, 4));
    f4!(2, 3, 4, 5).store(&mut buf);
    check!(
        r,
        buf[0] == 2.
            && buf[1] == 3.
            && buf[2] == 4.
            && buf[3] == 5.
            && buf[4] == 5.
            && buf[5] == 6.
    );
    eq!(r, Float4::load(&buf[0..]), f4!(2, 3, 4, 5));
    eq!(r, Float4::load(&buf[2..]), f4!(4, 5, 5, 6));
    eq!(r, shuffle(f4!(1, 2, 3, 4), [2, 1, 0, 3]), f4!(3, 2, 1, 4));
    eq!(r, shuffle(f4!(1, 2, 3, 4), [2, 1]), f2!(3, 2));
    eq!(r, shuffle(f4!(1, 2, 3, 4), [3, 3, 3, 3]), f4!(4, 4, 4, 4));
    eq!(
        r,
        shuffle(f4!(1, 2, 3, 4), [2, 1, 2, 1, 2, 1, 2, 1]),
        Float8::new([3., 2., 3., 2., 3., 2., 3., 2.])
    );
    check!(r, all(f4!(1, 2, 3, 4).lt_scalar(5.)));
    check!(r, all(Byte4::new([1, 2, 3, 4]).lt_scalar(5)));
    check!(r, all(i4!(1, 2, 3, 4).lt_scalar(5.0f32 as i32)));
    let five = Float4::splat(5.);
    check!(r, all(five.eq_scalar(5.0)));
    check!(r, all(five.eq_scalar(5i32 as f32)));
    eq!(
        r,
        vx_max_scalar(vx_min_scalar(f4!(1, 2, 3, 4), 3.), 2.),
        f4!(2, 2, 3, 3)
    );
    for x in 0..256 {
        for y in 0..256 {
            let want = (255. * (x as f64 / 255. * (y as f64 / 255.)) + 0.5) as u8;
            {
                let got = div255(Ushort8::splat(x as u16) * Ushort8::splat(y as u16))[0];
                check!(r, got == want);
            }
            {
                let got = approx_scale(Byte8::splat(x as u8), Byte8::splat(y as u8))[0];
                check!(
                    r,
                    got as i32 == want as i32 - 1 || got == want || got as i32 == want as i32 + 1
                );
                if x == 0 || y == 0 || x == 255 || y == 255 {
                    check!(r, got == want);
                }
            }
        }
    }
    for x in 0..256 {
        for y in 0..256 {
            let xy = (x * y) as u16;
            check!(
                r,
                all(mull(Byte2::splat(x as u8), Byte2::splat(y as u8)).eq_scalar(xy))
            );
            check!(
                r,
                all(mull(Byte4::splat(x as u8), Byte4::splat(y as u8)).eq_scalar(xy))
            );
            check!(
                r,
                all(mull(Byte8::splat(x as u8), Byte8::splat(y as u8)).eq_scalar(xy))
            );
            check!(
                r,
                all(mull(Byte16::splat(x as u8), Byte16::splat(y as u8)).eq_scalar(xy))
            );
        }
    }
    {
        let fs = Float8::new([0., 0.5, 1., 2., -4., -0.5, -1., -2.]);
        let hs = Half8::new([
            0x0000, 0x3800, 0x3c00, 0x4000, 0xc400, 0xb800, 0xbc00, 0xc000,
        ]);
        eq!(r, to_half(fs), hs);
        eq!(r, from_half::<f32, 8>(hs), fs);
    }
    r.finish();
}
// tests/SkVxTest.cpp:159; getters plus setters replace C++ reference-valued swizzles.
#[test]
fn SkVx_xy() {
    let mut r = Reporter::default();
    let mut f = f2!(1, 2);
    eq!(r, f, f2!(1, 2));
    check!(r, f.x() == 1.);
    check!(r, f.y() == 2.);
    f.set_y(9.);
    eq!(r, f, f2!(1, 9));
    f.set_x(0.);
    eq!(r, f, f2!(0, 9));
    f[0] = 8.;
    check!(r, f.x() == 8.);
    f[1] = 6.;
    check!(r, f.y() == 6.);
    eq!(r, f, f2!(8, 6));
    f = f.yx();
    eq!(r, f, f2!(6, 8));
    check!(r, sk_bit_cast::<Point, _>(&f) == Point::make(6., 8.));
    let mut p = Point::default();
    // Preserve store into Point storage, rather than replacing the layout observation with construction.
    unsafe {
        f.store(std::slice::from_raw_parts_mut(
            (&mut p as *mut Point).cast::<f32>(),
            2,
        ));
    }
    check!(r, p == Point::make(6., 8.));
    unsafe {
        f.yx().store(std::slice::from_raw_parts_mut(
            (&mut p as *mut Point).cast::<f32>(),
            2,
        ));
    }
    check!(r, p == Point::make(8., 6.));
    eq!(r, f.xyxy(), f4!(6, 8, 6, 8));
    eq!(r, f.xyxy(), join(f, f));
    eq!(r, join(f, f), f.xyxy());
    eq!(r, join(f.yx(), f), f4!(f.y(), f.x(), f.x(), f.y()));
    eq!(r, join(f.yx(), f), f4!(f.yx()[0], f.yx()[1], f.x(), f.y()));
    eq!(r, join(f, f.yx()), f4!(f.x(), f.y(), f.yx()[0], f.yx()[1]));
    eq!(r, join(f.yx(), f.yx()), join(f.yx(), f.yx()));
    r.finish();
}
// tests/SkVxTest.cpp:190
#[test]
fn SkVx_xyzw() {
    let mut r = Reporter::default();
    let mut f = f4!(1, 2, 3, 4);
    eq!(r, f, f4!(1, 2, 3, 4));
    eq!(r, f, join(f2!(1, 2), f2!(3, 4)));
    eq!(r, f, join(f2!(1, 2), f2!(3, 4)));
    eq!(r, f, join(f2!(1, 2), f2!(3, 4)));
    f.set_xy(f2!(9, 8));
    eq!(r, f, f4!(9, 8, 3, 4));
    let mut zw = f.zw();
    zw.set_x(7.);
    f.set_zw(zw);
    let mut zw = f.zw();
    zw.set_y(6.);
    f.set_zw(zw);
    eq!(r, f, f4!(9, 8, 7, 6));
    f.set_x(5.);
    f.set_y(4.);
    f.set_z(3.);
    f.set_w(2.);
    eq!(r, f, f4!(5, 4, 3, 2));
    f[0] = 0.;
    check!(r, f.x() == 0.);
    f[1] = 1.;
    check!(r, f.y() == 1.);
    f[2] = 2.;
    check!(r, f.z() == 2.);
    f[3] = 3.;
    check!(r, f.w() == 3.);
    eq!(r, f.xy(), f2!(0, 1));
    eq!(r, f.zw(), f2!(2, 3));
    eq!(r, f, f4!(0, 1, 2, 3));
    eq!(r, f.yxwz().lo(), shuffle(f, [1, 0]));
    eq!(r, f.yxwz().hi(), shuffle(f, [3, 2]));
    check!(r, all(f.zwxy().lo().lo().eq_scalar(f.z())));
    check!(r, all(f.zwxy().lo().hi().eq_scalar(f.w())));
    check!(r, all(f.zwxy().hi().lo().eq_scalar(f.x())));
    check!(r, all(f.zwxy().hi().hi().eq_scalar(f.y())));
    check!(r, f.yxwz().lo().lo().val() == f.y());
    check!(r, f.yxwz().lo().hi().val() == f.x());
    check!(r, f.yxwz().hi().lo().val() == f.w());
    check!(r, f.yxwz().hi().hi().val() == f.z());
    eq!(
        r,
        naive_if_then_else(
            i2!(0, !0),
            shuffle(f4!(0, 1, 2, 3), [3, 2]),
            f4!(4, 5, 6, 7).xy()
        ),
        f2!(4, 2)
    );
    eq!(
        r,
        if_then_else(
            i2!(0, !0),
            shuffle(f4!(0, 1, 2, 3), [3, 2]),
            f4!(4, 5, 6, 7).xy()
        ),
        f2!(4, 2)
    );
    eq!(
        r,
        naive_if_then_else(i2!(0, !0).xyxy(), f4!(0, 1, 2, 3).zwxy(), f4!(4, 5, 6, 7)),
        f4!(4, 3, 6, 1)
    );
    eq!(
        r,
        if_then_else(i2!(0, !0).xyxy(), f4!(0, 1, 2, 3).zwxy(), f4!(4, 5, 6, 7)),
        f4!(4, 3, 6, 1)
    );
    eq!(
        r,
        pin_vectors(
            f4!(0, 1, 2, 3).yxwz(),
            Float2::splat(1.).xyxy(),
            Float2::splat(2.).xyxy()
        ),
        f4!(1, 1, 2, 2)
    );
    r.finish();
}
// tests/SkVxTest.cpp:246
#[test]
fn SkVx_cross_dot() {
    let mut r = Reporter::default();
    check!(r, cross(i2!(0, 1), i2!(0, 1)) == 0);
    check!(r, cross(i2!(1, 0), i2!(1, 0)) == 0);
    check!(r, cross(i2!(1, 1), i2!(1, 1)) == 0);
    check!(r, cross(i2!(1, 1), i2!(1, -1)) == -2);
    check!(r, cross(i2!(1, 1), i2!(-1, 1)) == 2);
    check!(r, dot(i2!(0, 1), i2!(1, 0)) == 0);
    check!(r, dot(i2!(1, 0), i2!(0, 1)) == 0);
    check!(r, dot(i2!(1, 1), i2!(1, -1)) == 0);
    check!(r, dot(i2!(1, 1), i2!(1, 1)) == 2);
    check!(r, dot(i2!(1, 1), i2!(-1, -1)) == -2);
    let mut rand = SkRandom::new();
    for _ in 0..100 {
        let a = rand.next_range_f(-1., 1.);
        let b = rand.next_range_f(-1., 1.);
        let c = rand.next_range_f(-1., 1.);
        let d = rand.next_range_f(-1., 1.);
        let tolerance = 1f32 / (1 << 20) as f32;
        check!(
            r,
            scalar_nearly_equal_within(
                cross(f2!(a, b), f2!(c, d)),
                Point::cross_product(&Point::new(a, b), &Point::new(c, d)),
                tolerance
            )
        );
        check!(
            r,
            scalar_nearly_equal_within(
                dot(f2!(a, b), f2!(c, d)),
                Point::dot_product(&Point::new(a, b), &Point::new(c, d)),
                tolerance
            )
        );
    }
    // Upstream SkScalarNearlyEqual takes floats even for these double results.
    let mut assert_doubles_equal = |left: f64, right: f64| {
        check!(r, scalar_nearly_equal(left as f32, right as f32));
    };
    assert_doubles_equal(
        cross(Double2::new([1.2, 3.4]), Double2::new([3.4, -1.2])),
        -13.000000,
    );
    assert_doubles_equal(
        cross(Double2::new([12.34, 5.6]), Double2::new([7.8, -9.0])),
        -154.740000,
    );
    assert_doubles_equal(
        cross(Double2::new([12.34, 5.6]), Double2::new([7.8, 9.012345678])),
        67.532346,
    );
    r.finish();
}
fn check_strided_loads<T: VxElem, const N: usize>(r: &mut Reporter, value: impl Fn(usize) -> T) {
    let values: Vec<T> = (0..N * 4).map(value).collect();
    let (a, b) = strided_load2::<T, N>(&values);
    for i in 0..N {
        check!(r, a[i] == values[i * 2]);
        check!(r, b[i] == values[i * 2 + 1]);
    }
    let (a, b, c, d) = strided_load4::<T, N>(&values);
    for i in 0..N {
        check!(r, a[i] == values[i * 4]);
        check!(r, b[i] == values[i * 4 + 1]);
        check!(r, c[i] == values[i * 4 + 2]);
        check!(r, d[i] == values[i * 4 + 3]);
    }
}
// tests/SkVxTest.cpp:306
#[test]
fn SkVx_strided_loads() {
    let mut r = Reporter::default();
    macro_rules! widths {
        ($t:ty) => {
            check_strided_loads::<$t, 1>(&mut r, |i| i as $t);
            check_strided_loads::<$t, 2>(&mut r, |i| i as $t);
            check_strided_loads::<$t, 4>(&mut r, |i| i as $t);
            check_strided_loads::<$t, 8>(&mut r, |i| i as $t);
            check_strided_loads::<$t, 16>(&mut r, |i| i as $t);
            check_strided_loads::<$t, 32>(&mut r, |i| i as $t);
        };
    }
    widths!(u32);
    widths!(u16);
    widths!(u8);
    widths!(i32);
    widths!(i16);
    widths!(i8);
    widths!(f32);
    r.finish();
}
// tests/SkVxTest.cpp:316; preserve diagnostic-only errorBounds in the inner loop.
#[test]
fn SkVx_ScaledDividerU32() {
    let mut r = Reporter::default();
    let max = u32::MAX;
    let error_bounds = |actual: u32, expected: u32| {
        let lower = if expected == 0 { 0 } else { expected - 1 };
        let upper = if expected == max { max } else { expected + 1 };
        lower <= actual && actual <= upper
    };
    let mut test = |denom: u32| {
        let d = ScaledDividerU32::new(denom);
        let max_check = (((max - d.half()) as f64 / denom as f64) + 0.5).floor() as u32;
        check!(r, error_bounds(d.divide(max)[0], max_check));
        let mut i = 0u32;
        while i < max - d.half() {
            let expected = (i as f64 / denom as f64 + 0.5).floor() as u32;
            let actual = d.divide(i + d.half());
            if !error_bounds(actual[0], expected) {
                eprintln!("i: {i} expected: {expected} actual: {}", actual[0]);
            }
            for e in 1..4 {
                debug_assert!(actual[0] == actual[e]);
            }
            i = i.wrapping_add(65535);
        }
    };
    test(2);
    test(3);
    test(5);
    test(7);
    test(27);
    test(65535);
    test(15485863);
    test(512927377);
    r.finish();
}
// tests/SkVxTest.cpp:354
#[test]
fn SkVx_saturated_add() {
    let mut r = Reporter::default();
    for a in 0..1 << 8 {
        for b in 0..1 << 8 {
            let mut exact = a + b;
            if exact > 255 {
                exact = 255;
            }
            if exact < 0 {
                exact = 0;
            }
            check!(
                r,
                saturated_add(Byte16::splat(a as u8), Byte16::splat(b as u8))[0] as i32 == exact
            );
        }
    }
    r.finish();
}
// tests/SkVxTest.cpp:366
#[test]
fn SkVx_length() {
    let mut r = Reporter::default();
    {
        let mut equal = |l: f32, v: f32| {
            check!(r, scalar_nearly_equal(l, v));
        };
        equal(length(f2!(0, 1)), 1.000000);
        equal(length(f2!(2, 0)), 2.000000);
        equal(length(f2!(3, 4)), 5.000000);
        equal(length(f2!(1, 1)), 1.414214);
        equal(length(f2!(2.5, 2.5)), 3.535534);
        equal(length(f4!(1, 2, 3, 4)), 5.477226);
    }
    {
        let mut equal = |l: f64, v: f64| {
            check!(r, scalar_nearly_equal(l as f32, v as f32));
        };
        equal(length(Double2::new([2.5, 2.5])), 3.535534);
        equal(length(Double4::new([1.5, 2.5, 3.5, 4.5])), 6.403124);
    }
    r.finish();
}
// tests/SkVxTest.cpp:385
#[test]
fn SkVx_normalize() {
    let mut r = Reporter::default();
    {
        let mut equal = |l: f32, v: f32| {
            check!(r, scalar_nearly_equal(l, v));
        };
        let v = normalize(f2!(1.2, 3.4));
        equal(v[0], 0.332820);
        equal(v[1], 0.942990);
    }
    {
        let mut equal = |l: f64, v: f64| {
            check!(r, scalar_nearly_equal(l as f32, v as f32));
        };
        let v = normalize(Double2::new([2.3, -4.5]));
        equal(v[0], 0.455111);
        equal(v[1], -0.890435);
        let v = normalize(Double4::new([1.2, 3.4, 5.6, 7.8]));
        equal(v[0], 0.116997);
        equal(v[1], 0.331490);
        equal(v[2], 0.545984);
        equal(v[3], 0.760478);
    }
    r.finish();
}
// tests/SkVxTest.cpp:408
#[test]
fn SkVx_normalize_infinity_and_nan() {
    let mut r = Reporter::default();
    let zero = normalize(f2!(0, 0));
    check!(r, zero[0].is_nan());
    check!(r, zero[1].is_nan());
    check!(r, !isfinite(zero));
    let big = normalize(f2!(f32::MAX, f32::MAX));
    check!(r, big[0] == 0.);
    check!(r, big[1] == 0.);
    let big = normalize(Double2::new([f64::MAX, f64::MAX]));
    check!(r, big[0] == 0.);
    check!(r, big[1] == 0.);
    r.finish();
}
// tests/SkVxTest.cpp:425
#[test]
fn SkVx_isfinite() {
    let mut r = Reporter::default();
    check!(r, isfinite(f2!(0, 0)));
    check!(r, isfinite(Double4::new([1.2, 3.4, 5.6, 7.8])));
    check!(r, isfinite(Float8::new([8., 7., 6., 5., 4., 3., 2., 1.])));
    check!(r, !isfinite(f2!(0, f32::NAN)));
    check!(r, !isfinite(f2!(f32::INFINITY, 10)));
    check!(r, !isfinite(f2!(f32::NAN, f32::INFINITY)));
    for i in 0..4 {
        let mut v = Double4::new([4., 3., 2., 1.]);
        v[i] = f64::INFINITY;
        check!(r, !isfinite(v));
        v[i] = f64::NAN;
        check!(r, !isfinite(v));
    }
    for i in 0..8 {
        let mut v = Float8::new([8., 7., 6., 5., 4., 3., 2., 1.]);
        v[i] = f32::INFINITY;
        check!(r, !isfinite(v));
        v[i] = f32::NAN;
        check!(r, !isfinite(v));
    }
    r.finish();
}
