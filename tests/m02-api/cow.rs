#[path = "../support/mod.rs"]
mod support;
use rust_skia::*;
use support::{Reporter, reporter_assert as check};
#[test]
fn TCopyOnFirstWrite_copy() {
    let mut r = Reporter::default();
    let v = 42;
    let mut cow = SkTCopyOnFirstWrite::new(&v);
    check!(r, *cow.get() == 42);
    check!(r, std::ptr::eq(&v, cow.get()));
    {
        let mut copy = cow.clone();
        check!(r, *copy.get() == 42);
        check!(r, std::ptr::eq(&v, copy.get()));
        check!(r, std::ptr::eq(cow.get(), copy.get()));
        *copy.writable() = 43;
        check!(r, *cow.get() == 42);
        check!(r, std::ptr::eq(&v, cow.get()));
        check!(r, *copy.get() == 43);
        check!(r, !std::ptr::eq(&v, copy.get()));
        check!(r, !std::ptr::eq(cow.get(), copy.get()));
    }
    *cow.writable() = 43;
    check!(r, *cow.get() == 43);
    check!(r, !std::ptr::eq(&v, cow.get()));
    {
        let mut copy = cow.clone();
        check!(r, *copy.get() == 43);
        check!(r, !std::ptr::eq(&v, copy.get()));
        check!(r, !std::ptr::eq(cow.get(), copy.get()));
        *copy.writable() = 44;
        check!(r, *cow.get() == 43);
        check!(r, !std::ptr::eq(&v, cow.get()));
        check!(r, *copy.get() == 44);
        check!(r, !std::ptr::eq(&v, copy.get()));
        check!(r, !std::ptr::eq(cow.get(), copy.get()));
    }
    r.finish();
}
