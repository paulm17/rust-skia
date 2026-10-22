// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/UtilsTest.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
use rust_skia::extras::{SkEnumerate, SkZip};
use std::cell::Cell;
struct RefClass {
    base: RefCntBase,
    n: i32,
}
impl RefClass {
    fn new(n: i32) -> Self {
        Self {
            base: RefCntBase::new(),
            n,
        }
    }
    fn get(&self) -> i32 {
        self.n
    }
}
impl RefCnt for RefClass {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
}
fn test_autounref(r: &mut Reporter) {
    let mut obj = RefClass::new(0);
    check!(r, obj.base.unique());
    let ptr = &mut obj as *mut RefClass;
    let mut tmp = SkSp::adopt(ptr);
    check!(r, std::ptr::eq(ptr, tmp.as_ptr()));
    check!(r, obj.base.unique());
    check!(r, ptr == tmp.take());
    check!(r, obj.base.unique());
    check!(r, tmp.take().is_null());
    check!(r, tmp.as_ptr().is_null());
    obj.base.ref_();
    check!(r, !obj.base.unique());
    {
        let _tmp2 = SkSp::adopt(ptr);
    }
    check!(r, obj.base.unique());
    // Neither temporary owns the final reference to this stack fixture.
}
fn test_autostarray(r: &mut Reporter) {
    let obj0 = RefClass::new(0);
    let obj1 = RefClass::new(1);
    check!(r, obj0.base.unique());
    check!(r, obj1.base.unique());
    {
        let mut tmp = AutoSTArray::<SkSp<RefClass>, 2>::new();
        check!(r, 0 == tmp.count());
        tmp.reset(0);
        tmp.reset(4);
        check!(r, 4 == tmp.count());
        tmp[0].reset(Some(sk_ref_sp(&obj0)));
        tmp[1].reset(Some(sk_ref_sp(&obj1)));
        check!(r, !obj0.base.unique());
        check!(r, !obj1.base.unique());
        tmp.reset(0);
        check!(r, 0 == tmp.count());
        check!(r, obj0.base.unique());
        check!(r, obj1.base.unique());
        tmp.reset(2);
        check!(r, 2 == tmp.count());
        tmp[0].reset(Some(sk_ref_sp(&obj0)));
        tmp[1].reset(Some(sk_ref_sp(&obj1)));
    }
    check!(r, obj0.base.unique());
    check!(r, obj1.base.unique());
    {
        let mut tmp = AutoSTArray::<SkSp<RefClass>, 2>::with_count(4);
        check!(r, 4 == tmp.count());
        tmp[0].reset(Some(sk_ref_sp(&obj0)));
        tmp[1].reset(Some(sk_ref_sp(&obj1)));
        check!(r, !obj0.base.unique());
        check!(r, !obj1.base.unique());
        tmp.reset(0);
        check!(r, obj0.base.unique());
        check!(r, obj1.base.unique());
        tmp.reset(2);
        tmp[0].reset(Some(sk_ref_sp(&obj0)));
        tmp[1].reset(Some(sk_ref_sp(&obj1)));
        check!(r, !obj0.base.unique());
        check!(r, !obj1.base.unique());
        tmp.reset(4);
        check!(r, obj0.base.unique());
        check!(r, obj1.base.unique());
        tmp[0].reset(Some(sk_ref_sp(&obj0)));
        tmp[1].reset(Some(sk_ref_sp(&obj1)));
        check!(r, !obj0.base.unique());
        check!(r, !obj1.base.unique());
    }
    check!(r, obj0.base.unique());
    check!(r, obj1.base.unique());
}
fn test_search(r: &mut Reporter) {
    let mut array = [0; 91];
    let mut rand = SkRandom::new();
    for i in 0..91 {
        array[i] = rand.next_s();
    }
    sk_theapsort(&mut array, |a, b| a < b);
    for i in 1..91 {
        check!(r, array[i - 1] <= array[i]);
    }
    for i in 0..91 {
        let index = sk_tsearch(&array, |v| v.cmp(&array[i]));
        check!(r, index == Ok(i));
    }
    for _ in 0..10000 {
        let value = rand.next_s();
        let index = sk_tsearch(&array, |v| v.cmp(&value));
        match index {
            Ok(index) => {
                check!(r, index < 91 && array[index] == value);
            }
            Err(index) => {
                // Err(index) is Rust's representation of C++'s bitwise-complement insertion index.
                check!(r, index <= 91);
                if index < 91 {
                    check!(r, value < array[index]);
                    if index > 0 {
                        check!(r, value > array[index - 1]);
                    }
                } else {
                    check!(r, value > array[90]);
                }
            }
        }
    }
}
// tests/UtilsTest.cpp:181
#[test]
fn Utils() {
    let mut r = Reporter::default();
    test_search(&mut r);
    test_autounref(&mut r);
    test_autostarray(&mut r);
    r.finish();
}
// tests/UtilsTest.cpp:187. Rich enumeration views must come from production, not a test-side clone.
#[test]
fn SkEnumerate() {
    let mut r = Reporter::default();
    let a = [1, 2, 3, 4];
    let enumeration = SkEnumerate::new(&a);
    let mut count = 0;
    for (i, v) in enumeration.iter() {
        check!(r, i == count);
        check!(r, *v == count as i32 + 1);
        count += 1;
    }
    count = 0;
    for (i, v) in SkEnumerate::new(&a).iter() {
        check!(r, i == count);
        check!(r, *v == count as i32 + 1);
        count += 1;
    }
    count = 0;
    let vec = vec![1, 2, 3, 4];
    for (i, v) in SkEnumerate::new(&vec).iter() {
        check!(r, i == count);
        check!(r, *v == count as i32 + 1);
        count += 1;
    }
    check!(r, count == 4);
    count = 0;
    let span = SkSpan::new(&vec);
    for (i, v) in SkEnumerate::new(span.as_slice()).iter() {
        check!(r, i == count);
        check!(r, *v == count as i32 + 1);
        count += 1;
    }
    {
        let e = SkEnumerate::new(span.as_slice()).first(2);
        for (i, v) in e.iter() {
            check!(r, *v == i as i32 + 1);
        }
        check!(r, e.size() == 2);
    }
    {
        let e = SkEnumerate::new(span.as_slice()).last(2);
        for (i, v) in e.iter() {
            check!(r, *v == i as i32 + 1);
        }
        check!(r, e.size() == 2);
    }
    {
        let e = SkEnumerate::new(span.as_slice()).subspan(1, 2);
        for (i, v) in e.iter() {
            check!(r, *v == i as i32 + 1);
        }
        check!(r, e.size() == 2);
    }
    {
        #[derive(Default, Clone)]
        struct I {
            i: i32,
        }
        impl Drop for I {
            fn drop(&mut self) {}
        }
        let mut values: [I; 10] = std::array::from_fn(|_| I::default());
        let mut s = SkSpanMut::new(&mut values);
        for (i, v) in SkEnumerate::new_mut(s.as_mut_slice()).iter_mut() {
            *v = I { i: i as i32 };
        }
        for i in 0..s.size() {
            check!(r, s.as_slice()[i].i == i as i32);
        }
        // The span borrows values; inspect the same backing array after ending that borrow.
        for i in 0..values.len() {
            check!(r, values[i].i == i as i32);
        }
    }
    {
        let mut input: [Option<Box<i32>>; 10] = std::array::from_fn(|_| None);
        let mut output: [Option<Box<i32>>; 10] = std::array::from_fn(|_| None);
        for (i, v) in SkEnumerate::new_mut(&mut input).iter_mut() {
            *v = Some(Box::new(i as i32));
        }
        for (i, v) in SkEnumerate::new_mut(&mut output).iter_mut() {
            *v = input[i].take();
        }
        for i in 0..10 {
            check!(r, **output[i].as_ref().unwrap() == i as i32);
            check!(r, input[i].is_none());
        }
    }
    {
        let mut input: [Option<Box<i32>>; 10] = std::array::from_fn(|_| None);
        let mut output: [Option<Box<i32>>; 10] = std::array::from_fn(|_| None);
        for (i, v) in SkEnumerate::new_mut(&mut input).iter_mut() {
            *v = Some(Box::new(i as i32));
        }
        let mut zipped = SkZip::new_mut((&mut output[..], &mut input[..]));
        for (_i, (ov, iv)) in zipped.enumerate_mut() {
            *ov = iv.take();
        }
        for i in 0..10 {
            check!(r, **output[i].as_ref().unwrap() == i as i32);
            check!(r, input[i].is_none());
        }
    }
    r.finish();
}
// tests/UtilsTest.cpp:306. Cells preserve the original overlapping C/S mutable columns safely.
#[test]
fn SkZip() {
    let mut r = Reporter::default();
    let a = [1u16, 2, 3, 4].map(Cell::new);
    let b = [10f32, 20., 30., 40.];
    let c: Vec<Cell<i32>> = [20, 30, 40, 50].map(Cell::new).into();
    let d = [100, 200, 300, 400].map(Cell::new);
    let s = SkSpan::new(&c);
    let z = SkZip::new((&a[..], &b[..], &c[..], &d[..], s.as_slice()));
    check!(r, z.size() == 4);
    check!(r, !z.empty());
    {
        let t = z.front();
        check!(r, t.0.get() == 1);
        check!(r, *t.1 == 10.);
        check!(r, t.2.get() == 20);
        check!(r, t.3.get() == 100);
        check!(r, t.4.get() == 20);
    }
    {
        let t = z.back();
        check!(r, t.0.get() == 4);
        check!(r, *t.1 == 40.);
    }
    {
        let mut i = 0;
        for (av, bv, cv, dv, sv) in z.iter() {
            check!(r, av.get() == a[i].get());
            check!(r, *bv == b[i]);
            check!(r, cv.get() == c[i].get());
            check!(r, dv.get() == d[i].get());
            check!(r, sv.get() == s[i].get());
            i += 1;
        }
        // Original assertion is assignment (i = 4), not equality.
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let mut i = 0;
        for (av, bv, cv, dv, sv) in z.first(2).iter() {
            check!(r, av.get() == a[i].get());
            check!(r, *bv == b[i]);
            check!(r, cv.get() == c[i].get());
            check!(r, dv.get() == d[i].get());
            check!(r, sv.get() == s[i].get());
            i += 1;
        }
        check!(r, {
            i = 2;
            i != 0
        });
    }
    {
        let mut i = 0;
        for t in z.last(2).iter() {
            let (av, bv, cv, dv, sv) = t;
            check!(r, av.get() == a[i + 2].get());
            check!(r, *bv == b[i + 2]);
            check!(r, cv.get() == c[i + 2].get());
            check!(r, dv.get() == d[i + 2].get());
            check!(r, sv.get() == s[i + 2].get());
            i += 1;
        }
        check!(r, {
            i = 2;
            i != 0
        });
    }
    {
        let mut i = 0;
        for t in z.subspan(1, 2).iter() {
            let (av, bv, cv, dv, sv) = t;
            check!(r, av.get() == a[i + 1].get());
            check!(r, *bv == b[i + 1]);
            check!(r, cv.get() == c[i + 1].get());
            check!(r, dv.get() == d[i + 1].get());
            check!(r, sv.get() == s[i + 1].get());
            i += 1;
        }
        check!(r, {
            i = 2;
            i != 0
        });
    }
    {
        let zz = z.clone();
        let mut i = 0;
        for (av, bv, cv, dv, sv) in zz.iter() {
            check!(r, av.get() == a[i].get());
            check!(r, *bv == b[i]);
            check!(r, cv.get() == c[i].get());
            check!(r, dv.get() == d[i].get());
            check!(r, sv.get() == s[i].get());
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let cz = z.as_const();
        let mut i = 0;
        for (av, bv, cv, dv, sv) in cz.iter() {
            check!(r, av.get() == a[i].get());
            check!(r, *bv == b[i]);
            check!(r, cv.get() == c[i].get());
            check!(r, dv.get() == d[i].get());
            check!(r, sv.get() == s[i].get());
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let ptrs = z.data();
        check!(
            r,
            ptrs == (a.as_ptr(), b.as_ptr(), c.as_ptr(), d.as_ptr(), s.data())
        );
    }
    {
        let span = z.column::<1>();
        check!(r, span[1] == 20.);
    }
    {
        let (av, bv, cv, dv, ev) = z.at(1);
        check!(r, av.get() == 2);
        check!(r, *bv == 20.);
        check!(r, cv.get() == 30);
        check!(r, dv.get() == 200);
        check!(r, ev.get() == 30);
        check!(r, std::ptr::eq(av, &a[1]));
        check!(r, std::ptr::eq(bv, &b[1]));
        check!(r, std::ptr::eq(cv, &c[1]));
        check!(r, std::ptr::eq(dv, &d[1]));
        check!(r, std::ptr::eq(ev, &s[1]));
        av.set(20);
        cv.set(300);
        dv.set(2000);
        ev.set(300);
        let t = z.at(1);
        check!(r, t.0.get() == 20);
        check!(r, *t.1 == 20.);
        check!(r, t.2.get() == 300);
        check!(r, t.3.get() == 2000);
        check!(r, t.4.get() == 300);
    }
    r.finish();
}
// tests/UtilsTest.cpp:479. Each tuple form preserves the original column order and sizing source.
#[test]
fn SkMakeZip() {
    use rust_skia::extras::sk_make_zip;
    let mut r = Reporter::default();
    let a = [1u16, 2, 3, 4];
    const B: [f32; 4] = [10., 20., 30., 40.];
    let c = vec![20, 30, 40, 50];
    let d = [100, 200, 300, 400];
    let s = SkSpan::new(&c);
    let p = &a[..];
    {
        let zz = sk_make_zip((&a[..], &B, c.as_slice(), &d, s.as_slice(), p));
        let mut i = 0;
        for (av, bv, cv, dv, sv, pv) in zz.iter() {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
            check!(r, *pv == p[i]);
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let mut i = 0;
        for (av, bv, cv, dv, sv) in sk_make_zip((&a[..], &B, c.as_slice(), &d, s.as_slice())).iter()
        {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let mut i = 0;
        for (av, bv, cv, dv, sv) in
            sk_make_zip((&a[..], &B[..], c.as_slice(), &d, s.as_slice())).iter()
        {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let mut i = 0;
        for (sv, av, bv, cv, dv) in sk_make_zip((s.as_slice(), &a, &B, c.as_slice(), &d)).iter() {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let mut i = 0;
        for (cv, sv, av, bv, dv) in sk_make_zip((c.as_slice(), s.as_slice(), &a, &B, &d)).iter() {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
            i += 1;
        }
        check!(r, {
            i = 4;
            i != 0
        });
    }
    {
        let zz = sk_make_zip((&a, &B, c.as_slice(), &d, s.as_slice()));
        for (i, (av, bv, cv, dv, sv)) in zz.enumerate() {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
        }
    }
    {
        let zz = &sk_make_zip((&a, &B, c.as_slice(), &d, s.as_slice()));
        for (i, (av, bv, cv, dv, sv)) in zz.enumerate() {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
        }
    }
    {
        for (i, (av, bv, cv, dv, sv)) in
            sk_make_zip((&a, &B, c.as_slice(), &d, s.as_slice())).enumerate()
        {
            check!(r, *av == a[i]);
            check!(r, *bv == B[i]);
            check!(r, *cv == c[i]);
            check!(r, *dv == d[i]);
            check!(r, *sv == s[i]);
        }
    }
    {
        let v: Vec<i32> = vec![];
        let z = sk_make_zip((v.as_slice(),));
        check!(r, z.empty());
    }
    {
        const CA: [u16; 4] = [1, 2, 3, 4];
        const CP: &[u16] = &CA;
        const Z: SkZip<(&[u16], &[u16])> = SkZip::new_const((&CA, CP));
        check!(r, !Z.empty());
    }
    r.finish();
}
// tests/UtilsTest.cpp:615
#[test]
fn UtilsPreserveBitPatterns() {
    let mut r = Reporter::default();
    let patterns = [
        0x00400000u32,
        0x80000000,
        0x3f800000,
        0x7f800000,
        0x7fa00000,
        0x7fe00000,
    ];
    for src in patterns {
        {
            let f: f32 = sk_unaligned_load(&src.to_ne_bytes());
            let dst: u32 = sk_unaligned_load(&f.to_ne_bytes());
            check!(r, dst == src);
        }
        {
            let mut f = [0u8; 4];
            sk_unaligned_store(&mut f, src);
            let f: f32 = f32::from_ne_bytes(f);
            let mut dst = [0u8; 4];
            sk_unaligned_store(&mut dst, f);
            let dst = u32::from_ne_bytes(dst);
            check!(r, dst == src);
        }
        check!(
            r,
            sk_bit_cast::<u32, _>(&sk_bit_cast::<f32, _>(&src)) == src
        );
    }
    r.finish();
}
