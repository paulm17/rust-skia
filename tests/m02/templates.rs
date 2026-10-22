use super::*;
#[test]
fn Templates() {
    let mut r = Reporter::default();
    let mut a = AutoSTMalloc::<1, i32>::new();
    a[0] = 1;
    check!(r, a[0] == 1);
    a.realloc(1);
    a[0] = 1;
    check!(r, a[0] == 1);
    a.realloc(2);
    check!(r, a[0] == 1);
    a.realloc(1);
    check!(r, a[0] == 1);
    a.realloc(0);
    a.realloc(10);
    for i in 0..10 {
        a[i] = 10 - i as i32;
    }
    a.realloc(20);
    for i in 0..10 {
        check!(r, a[i] == 10 - i as i32);
    }
    a.realloc(10);
    for i in 0..10 {
        check!(r, a[i] == 10 - i as i32);
    }
    a.realloc(1);
    check!(r, {
        a[0] = 10;
        a[0] != 0
    });
    a.reset(0);
    a.realloc(1);
    a.reset(1);
    a[0] = 1;
    check!(r, a[0] == 1);
    a.reset(2);
    a.realloc(3);
    a[0] = 1;
    check!(r, a[0] == 1);
    a.realloc(1);
    check!(r, a[0] == 1);
    r.finish();
}
#[test]
fn AutoReallocToZero() {
    let mut r = Reporter::default();
    let mut a = AutoTMalloc::<i32>::new();
    a.reset(10);
    check!(r, !a.get().is_null());
    a.realloc(0);
    check!(r, a.get().is_null());
    a.realloc(11);
    check!(r, !a.get().is_null());
    a.realloc(0);
    check!(r, a.get().is_null());
    a.realloc(10);
    check!(r, !a.get().is_null());
    let mut a = AutoSTMalloc::<10, i32>::new();
    a.reset(10);
    check!(r, !a.get().is_null());
    a.realloc(0);
    check!(r, a.get().is_null());
    a.realloc(11);
    check!(r, !a.get().is_null());
    a.realloc(0);
    check!(r, a.get().is_null());
    a.realloc(10);
    check!(r, !a.get().is_null());
    r.finish();
}
#[test]
fn AutoTMallocSelfMove() {
    let mut r = Reporter::default();
    let mut foo = AutoTMalloc::<i32>::new();
    foo.reset(20);
    check!(r, !foo.get().is_null());
    #[allow(clippy::self_assignment)]
    {
        foo = foo;
    }
    check!(r, !foo.get().is_null());
    r.finish();
}
