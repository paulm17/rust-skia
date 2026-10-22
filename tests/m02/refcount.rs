use super::*;
#[test]
fn RefCnt() {
    let mut r = Reporter::default();
    let rc = Box::new(SkRefCnt::new());
    std::thread::scope(|s| {
        let a = s.spawn(|| {
            for _ in 0..100000 {
                rc.ref_();
                rc.unref();
            }
        });
        let b = s.spawn(|| {
            for _ in 0..100000 {
                rc.ref_();
                rc.unref();
            }
        });
        a.join().unwrap();
        b.join().unwrap();
    });
    check!(r, rc.unique());
    let ptr = Box::into_raw(rc);
    unsafe {
        (*ptr).unref();
    }
    let rc = Box::new(SkWeakRefCnt::new());
    std::thread::scope(|s| {
        let a = s.spawn(|| {
            for _ in 0..100000 {
                WeakRefCnt::ref_(&*rc);
                rc.unref();
            }
        });
        let b = s.spawn(|| {
            for _ in 0..100000 {
                WeakRefCnt::ref_(&*rc);
                rc.unref();
            }
        });
        let c = s.spawn(|| {
            for _ in 0..100000 {
                if rc.weak_ref_cnt_base().try_ref() {
                    rc.unref();
                }
            }
        });
        let d = s.spawn(|| {
            for _ in 0..100000 {
                rc.weak_ref();
                rc.weak_unref();
            }
        });
        a.join().unwrap();
        b.join().unwrap();
        c.join().unwrap();
        d.join().unwrap();
    });
    check!(r, rc.unique());
    if cfg!(debug_assertions) {
        check!(r, rc.get_weak_cnt() == 1);
    }
    let ptr = Box::into_raw(rc);
    unsafe {
        (*ptr).unref();
    }
    r.finish();
}
#[test]
fn sk_sp_reset() {
    let mut r = Reporter::default();
    let rc = Box::into_raw(Box::new(SkRefCnt::new()));
    unsafe {
        check!(r, (*rc).unique());
        let mut sp: SkSp<SkRefCnt> = SkSp::null();
        sp.reset_raw(rc);
        check!(r, (*rc).unique());
        (*rc).ref_();
        check!(r, !(*rc).unique());
        sp.reset_raw(rc);
        check!(r, (*rc).unique());
    }
    r.finish();
}
#[test]
fn sk_sp_ref() {
    let mut r = Reporter::default();
    let rc = Box::into_raw(Box::new(SkRefCnt::new()));
    unsafe {
        check!(r, (*rc).unique());
        {
            let _sp = sk_ref_sp(&*rc);
            check!(r, !(*rc).unique());
        }
        check!(r, (*rc).unique());
        (*rc).unref();
    }
    r.finish();
}
