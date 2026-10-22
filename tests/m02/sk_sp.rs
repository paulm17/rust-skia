// Copyright 2012 Google Inc. BSD license: tests/LICENSE.skia.
// RefCntTest.cpp::sk_sp and its entire reached fixture/helper closure.
#![allow(clippy::disallowed_names)] // Preserve original helper/local names.
use super::*;
use std::cell::Cell;
use std::mem::MaybeUninit;
thread_local! {static COUNTERS:Cell<[i32;4]>=const{Cell::new([0;4])};}
fn bump(i: usize) {
    COUNTERS.with(|c| {
        let mut v = c.get();
        v[i] += 1;
        c.set(v);
    });
}
fn reset_counters() {
    COUNTERS.with(|c| c.set([0; 4]));
}
fn counts(r: &mut Reporter, refs: i32, unrefs: i32, new: i32, delete: i32) {
    let actual = COUNTERS.with(|c| c.get());
    eprintln!(
        "refcount expected={:?} actual={actual:?}",
        [refs, unrefs, new, delete]
    );
    check!(r, actual[0] == refs);
    check!(r, actual[1] == unrefs);
    check!(r, actual[2] == new);
    check!(r, actual[3] == delete);
}
// A repr(C) prefix represents the original single C++ base. The fixture retains
// its own integer count and virtual destructor; production SkSp does ownership.
#[repr(C)]
struct Effect {
    marker: RefCntBase,
    f_ref_cnt: Cell<i32>,
    dispose: unsafe fn(*mut Effect),
}
unsafe fn delete_effect(p: *mut Effect) {
    unsafe {
        drop(Box::from_raw(p));
    }
}
impl Effect {
    fn new() -> Self {
        bump(2);
        Self {
            marker: RefCntBase::new(),
            f_ref_cnt: Cell::new(1),
            dispose: delete_effect,
        }
    }
    fn method(&self) -> Box<MaybeUninit<i32>> {
        Box::new(MaybeUninit::uninit())
    }
}
impl RefCnt for Effect {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.marker
    }
    fn ref_(&self) {
        bump(0);
        self.f_ref_cnt.set(self.f_ref_cnt.get() + 1);
    }
    fn unref(&self) {
        bump(1);
        assert!(self.f_ref_cnt.get() > 0);
        let n = self.f_ref_cnt.get() - 1;
        self.f_ref_cnt.set(n);
        if n == 0 {
            bump(3);
            unsafe {
                (self.dispose)(self as *const Self as *mut Self);
            }
        }
    }
}
fn create() -> SkSp<Effect> {
    rust_skia::sk_make_sp(Effect::new())
}
#[derive(Default)]
struct Paint {
    effect: SkSp<Effect>,
}
impl Paint {
    fn get(&self) -> &SkSp<Effect> {
        &self.effect
    }
    fn set(&mut self, value: SkSp<Effect>) {
        self.effect = value;
    }
}
#[repr(C)]
struct EffectImpl {
    base: Effect,
    value: i32,
}
unsafe fn delete_impl(p: *mut Effect) {
    unsafe {
        drop(Box::from_raw(p.cast::<EffectImpl>()));
    }
}
impl EffectImpl {
    fn create() -> SkSp<Self> {
        let mut base = Effect::new();
        base.dispose = delete_impl;
        rust_skia::sk_make_sp(Self { base, value: 0 })
    }
}
impl RefCnt for EffectImpl {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base.marker
    }
    fn ref_(&self) {
        self.base.ref_();
    }
    fn unref(&self) {
        self.base.unref();
    }
}
fn as_base(e: &EffectImpl) -> &Effect {
    &e.base
}
fn make_effect() -> SkSp<Effect> {
    let mut foo = EffectImpl::create();
    foo.get_mut().unwrap().value = 42;
    foo.cast(as_base)
}
#[test]
fn sk_sp() {
    let mut r = Reporter::default();
    reset_counters();
    let mut paint = Paint::default();
    check!(r, paint.effect.get().is_none());
    check!(r, paint.get().is_null());
    counts(&mut r, 0, 0, 0, 0);
    paint.set(create());
    counts(&mut r, 0, 0, 1, 0);
    check!(r, paint.effect.f_ref_cnt.get() == 1);
    if !paint.get().is_null() {
        check!(r, true);
    } else {
        check!(r, false);
    }
    if paint.get().is_null() {
        check!(r, false);
    } else {
        check!(r, true);
    }
    paint.set(SkSp::null());
    counts(&mut r, 0, 1, 1, 1);
    if !paint.get().is_null() {
        check!(r, false);
    } else {
        check!(r, true);
    }
    if paint.get().is_null() {
        check!(r, true);
    } else {
        check!(r, false);
    }
    let mut e = create();
    check!(
        r,
        std::mem::size_of_val(&e) == std::mem::size_of::<*const ()>()
    );
    counts(&mut r, 0, 1, 2, 1);
    paint.set(e.clone());
    counts(&mut r, 1, 1, 2, 1);
    check!(r, paint.effect.f_ref_cnt.get() == 2);
    let mut paint2 = Paint::default();
    paint2.set(paint.get().clone());
    counts(&mut r, 2, 1, 2, 1);
    check!(r, paint.effect.f_ref_cnt.get() == 3);
    drop(paint.get().method());
    counts(&mut r, 2, 1, 2, 1);
    drop((*paint.get()).method());
    counts(&mut r, 2, 1, 2, 1);
    paint.set(SkSp::null());
    e.reset(None);
    paint2.set(SkSp::null());
    counts(&mut r, 2, 4, 2, 2);
    reset_counters();
    {
        counts(&mut r, 0, 0, 0, 0);
        let mut foo: SkSp<Effect> = SkSp::null();
        check!(r, foo.is_null());
        foo = make_effect();
        check!(r, !foo.is_null());
        counts(&mut r, 0, 0, 1, 0);
    }
    counts(&mut r, 0, 1, 1, 1);
    reset_counters();
    paint.set(EffectImpl::create().cast(as_base));
    counts(&mut r, 0, 0, 1, 0);
    paint.set(SkSp::null());
    counts(&mut r, 0, 1, 1, 1);
    reset_counters();
    let mut baz = EffectImpl::create();
    counts(&mut r, 0, 0, 1, 0);
    paint.set(std::mem::take(&mut baz).cast(as_base));
    counts(&mut r, 0, 0, 1, 0);
    check!(r, baz.is_null());
    paint.set(SkSp::null());
    counts(&mut r, 0, 1, 1, 1);
    reset_counters();
    {
        let bar1 = EffectImpl::create();
        let bar2: SkSp<Effect> = bar1.clone_as(as_base);
        counts(&mut r, 1, 0, 1, 0);
        check!(r, !bar1.is_null());
        check!(r, !bar2.is_null());
        check!(r, bar1 == bar2);
        check!(r, bar2 == bar1);
        check!(r, !(bar1 != bar2));
        check!(r, !(bar2 != bar1));
        let mut bar3: SkSp<Effect> = SkSp::null();
        std::hint::black_box(&bar3);
        bar3 = bar1.clone_as(as_base);
        std::hint::black_box(&bar3);
        counts(&mut r, 2, 0, 1, 0);
    }
    counts(&mut r, 2, 3, 1, 1);
    reset_counters();
    baz = EffectImpl::create();
    counts(&mut r, 0, 0, 1, 0);
    paint.set(baz.clone_as(as_base));
    counts(&mut r, 1, 0, 1, 0);
    baz.reset(None);
    counts(&mut r, 1, 1, 1, 0);
    paint.set(SkSp::null());
    counts(&mut r, 1, 2, 1, 1);
    {
        let empty: SkSp<SkRefCnt> = SkSp::null();
        let not_empty = rust_skia::sk_make_sp(SkRefCnt::new());
        check!(r, empty == SkSp::<SkRefCnt>::null());
        check!(r, not_empty != empty);
        check!(r, empty != not_empty);
        check!(r, std::ptr::null::<SkRefCnt>() == empty.as_ptr());
        check!(r, empty == std::ptr::null::<SkRefCnt>());
        check!(r, empty == empty);
    }
    {
        let a = rust_skia::sk_make_sp(SkRefCnt::new());
        let b = rust_skia::sk_make_sp(SkRefCnt::new());
        check!(r, a != b);
        check!(r, a == a);
    }
    // Original self-owning foo and the A/B cycle delete the containing object
    // during reset. Raw allocation is necessary to preserve those lifetimes.
    {
        struct Foo {
            base: RefCntBase,
            bar: SkSp<Foo>,
        }
        impl RefCnt for Foo {
            fn ref_cnt_base(&self) -> &RefCntBase {
                &self.base
            }
        }
        let p = Box::into_raw(Box::new(Foo {
            base: RefCntBase::new(),
            bar: SkSp::null(),
        }));
        unsafe {
            (*p).bar = SkSp::adopt(p);
            (*p).bar.reset(None);
        }
    }
    {
        struct A {
            base: RefCntBase,
            b: SkSp<B>,
        }
        struct B {
            base: RefCntBase,
            a: SkSp<A>,
        }
        impl RefCnt for A {
            fn ref_cnt_base(&self) -> &RefCntBase {
                &self.base
            }
        }
        impl RefCnt for B {
            fn ref_cnt_base(&self) -> &RefCntBase {
                &self.base
            }
        }
        let a = Box::into_raw(Box::new(A {
            base: RefCntBase::new(),
            b: SkSp::null(),
        }));
        let b = Box::into_raw(Box::new(B {
            base: RefCntBase::new(),
            a: SkSp::null(),
        }));
        unsafe {
            (*a).b.reset_raw(b);
            (*b).a.reset_raw(a);
            (*a).b.reset(None);
        }
    }
    r.finish();
}
