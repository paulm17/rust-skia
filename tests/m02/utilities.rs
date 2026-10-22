use super::*;
#[test]
fn memory_calloc() {
    let mut r = Reporter::default();
    unsafe {
        let p = sk_calloc_throw(200, 1) as *mut u8;
        for i in 0..200 {
            check!(r, *p.add(i) == 0);
        }
        sk_free(p.cast());
    }
    r.finish();
}
#[test]
fn SkFree_SafeToPassNull() {
    unsafe {
        sk_free(std::ptr::null_mut());
    }
}
#[test]
fn SkCalloc_DataIsZeroInitializedAndWriteable() {
    let mut r = Reporter::default();
    unsafe {
        let p = sk_calloc_throw(100, 1) as *mut u8;
        assert!(!p.is_null());
        for i in 0..100 {
            check!(r, *p.add(i) == 0);
            *p.add(i) = i as u8;
        }
        sk_free(p.cast());
        let p = sk_calloc_throw(100, 4) as *mut i32;
        assert!(!p.is_null());
        for i in 0..100 {
            check!(r, *p.add(i) == 0);
            *p.add(i) = i as i32;
        }
        sk_free(p.cast());
    }
    r.finish();
}
#[test]
fn SkMalloc_DataIsWriteable() {
    unsafe {
        let p = sk_malloc_throw(100) as *mut u8;
        assert!(!p.is_null());
        for i in 0..100 {
            *p.add(i) = i as u8;
        }
        sk_free(p.cast());
        let p = sk_malloc_throw(100 * 4) as *mut i32;
        assert!(!p.is_null());
        for i in 0..100 {
            *p.add(i) = i as i32;
        }
        sk_free(p.cast());
    }
}
#[test]
fn SkRealloc_DataIsWriteableAndEventuallyFreed() {
    let mut r = Reporter::default();
    unsafe {
        let mut p = sk_realloc_throw(std::ptr::null_mut(), 10) as *mut u8;
        assert!(!p.is_null());
        for i in 0..10 {
            *p.add(i) = i as u8;
        }
        p = sk_realloc_throw(p.cast(), 5).cast();
        assert!(!p.is_null());
        for i in 0..5 {
            check!(r, *p.add(i) == i as u8);
            *p.add(i) = i as u8 + 17;
        }
        p = sk_realloc_throw(p.cast(), 20).cast();
        assert!(!p.is_null());
        for i in 0..5 {
            check!(r, *p.add(i) == i as u8 + 17);
            *p.add(i) = i as u8 + 43;
        }
        for i in 5..20 {
            *p.add(i) = i as u8 + 43;
        }
        p = sk_realloc_throw(p.cast(), 0).cast();
        check!(r, p.is_null());
    }
    r.finish();
}
#[test]
fn SkContainerAllocatorBasic() {
    let mut r = Reporter::default();
    for i in 1..33 {
        let max = (usize::MAX / i).min(i32::MAX as usize);
        let a = SkContainerAllocator::new(i, max);
        check!(r, a.round_up_capacity(1) == K_CAPACITY_MULTIPLE);
        check!(r, a.round_up_capacity(i32::MAX as i64) == max);
        check!(r, a.round_up_capacity(max as i64) == max);
    }
    for i in 1..33 {
        let max = (usize::MAX / i).min(i32::MAX as usize);
        let a = SkContainerAllocator::new(i, max);
        check!(r, a.growth_factor_capacity(1, 1.5) == K_CAPACITY_MULTIPLE);
        check!(r, a.growth_factor_capacity(i32::MAX as usize, 1.5) == max);
        check!(r, a.growth_factor_capacity(max, 1.5) == max);
    }
    r.finish();
}
#[test]
fn TLazy_copy() {
    let mut r = Reporter::default();
    let mut lazy = SkTLazy::<i32>::new();
    check!(r, !lazy.is_valid());
    check!(r, lazy.get().is_none());
    {
        let c = lazy.clone();
        check!(r, !c.is_valid());
        check!(r, c.get().is_none());
    }
    lazy.init(42);
    check!(r, lazy.is_valid());
    check!(r, *lazy.get().unwrap() == 42);
    {
        let c = lazy.clone();
        check!(r, c.is_valid());
        check!(r, *c.get().unwrap() == 42);
        check!(r, !std::ptr::eq(lazy.get().unwrap(), c.get().unwrap()));
    }
    r.finish();
}
// SkTCopyOnFirstWrite has no Clone implementation: complete source body is
// retained in the correspondence ledger as an API failure, not replaced by Cow.
#[test]
fn SkOnce_Singlethreaded() {
    let mut r = Reporter::default();
    let mut x = 0;
    let once = SkOnce::new();
    for _ in 0..5 {
        once.call_once(|| x += 5);
    }
    check!(r, x == 5);
    r.finish();
}
#[test]
fn SkOnce_Multithreaded() {
    let mut r = Reporter::default();
    let x = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let once = std::sync::Arc::new(SkOnce::new());
    let g = SkTaskGroup::new();
    for _ in 0..1021 {
        let x = x.clone();
        let once = once.clone();
        g.add(move || {
            once.call_once(|| {
                x.fetch_add(6, std::sync::atomic::Ordering::Relaxed);
            })
        });
    }
    g.wait();
    check!(r, x.load(std::sync::atomic::Ordering::Relaxed) == 6);
    r.finish();
}
#[test]
fn SkOnce_NoArg() {
    let mut r = Reporter::default();
    let mut x = 0;
    let once = SkOnce::new();
    for _ in 0..3 {
        once.call_once(|| x += 1);
    }
    check!(r, x == 1);
    r.finish();
}
#[test]
fn SkSharedMutexBasic() {
    let sm = SkSharedMutex::new();
    sm.acquire_exclusive();
    sm.assert_exclusive_held();
    sm.release_exclusive();
    sm.acquire();
    sm.assert_held();
    sm.release();
}

#[test]
fn MD5() {
    let mut r = Reporter::default();
    let cases: &[(&[u8], [u8; 16])] = &[
        (
            b"",
            [
                0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8,
                0x42, 0x7e,
            ],
        ),
        (
            b"a",
            [
                0x0c, 0xc1, 0x75, 0xb9, 0xc0, 0xf1, 0xb6, 0xa8, 0x31, 0xc3, 0x99, 0xe2, 0x69, 0x77,
                0x26, 0x61,
            ],
        ),
        (
            b"abc",
            [
                0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0, 0xd6, 0x96, 0x3f, 0x7d, 0x28, 0xe1,
                0x7f, 0x72,
            ],
        ),
        (
            b"message digest",
            [
                0xf9, 0x6b, 0x69, 0x7d, 0x7c, 0xb7, 0x93, 0x8d, 0x52, 0x5a, 0x2f, 0x31, 0xaa, 0xf1,
                0x61, 0xd0,
            ],
        ),
        (
            b"abcdefghijklmnopqrstuvwxyz",
            [
                0xc3, 0xfc, 0xd3, 0xd7, 0x61, 0x92, 0xe4, 0x00, 0x7d, 0xfb, 0x49, 0x6c, 0xca, 0x67,
                0xe1, 0x3b,
            ],
        ),
        (
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            [
                0xd1, 0x74, 0xab, 0x98, 0xd2, 0x77, 0xd9, 0xf5, 0xa5, 0x61, 0x1c, 0x2c, 0x9f, 0x41,
                0x9d, 0x9f,
            ],
        ),
        (
            b"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
            [
                0x57, 0xed, 0xf4, 0xa2, 0x2b, 0xe3, 0xc9, 0x55, 0xac, 0x49, 0xda, 0x2e, 0x21, 0x07,
                0xb6, 0x7a,
            ],
        ),
    ];
    for (message, expected) in cases {
        let mut c = SkMD5::new();
        c.write(message);
        let actual = c.finish().data;
        eprintln!("MD5 input={message:?} expected={expected:02x?} actual={actual:02x?}");
        check!(r, actual == *expected);
        let mut c = SkMD5::new();
        for b in *message {
            c.write(&[*b]);
        }
        let actual = c.finish().data;
        eprintln!("MD5 input={message:?} expected={expected:02x?} actual={actual:02x?}");
        check!(r, actual == *expected);
    }
    r.finish();
}
#[test]
fn SkStringViewStartsAndEnds() {
    let mut r = Reporter::default();
    let mut empty = "";
    check!(r, empty.is_empty());
    check!(r, !sk_str_starts_with_char(empty, 'x'));
    check!(r, !sk_str_ends_with_char(empty, 'x'));
    check!(r, !sk_str_starts_with(empty, "x"));
    check!(r, !sk_str_ends_with(empty, "x"));
    check!(r, sk_str_starts_with(empty, ""));
    check!(r, sk_str_ends_with(empty, ""));

    let mut xyz = "xyz";
    check!(r, !xyz.is_empty());
    check!(r, xyz.as_bytes()[0] == b'x');
    check!(r, xyz.as_bytes()[2] == b'z');
    check!(r, xyz.len() == 3);

    check!(r, sk_str_starts_with_char(xyz, 'x'));
    check!(r, !sk_str_starts_with_char(xyz, 'y'));
    check!(r, sk_str_ends_with_char(xyz, 'z'));
    check!(r, !sk_str_ends_with_char(xyz, 'y'));

    check!(r, sk_str_starts_with(xyz, ""));
    check!(r, sk_str_ends_with(xyz, ""));
    check!(r, sk_str_starts_with(xyz, "x"));
    check!(r, sk_str_ends_with(xyz, "z"));
    check!(r, !sk_str_starts_with(xyz, "xa"));
    check!(r, !sk_str_ends_with(xyz, "az"));
    check!(r, sk_str_starts_with(xyz, "xy"));
    check!(r, sk_str_ends_with(xyz, "yz"));
    check!(r, sk_str_starts_with(xyz, "xyz"));
    check!(r, sk_str_ends_with(xyz, "xyz"));
    check!(r, !sk_str_starts_with(xyz, "wxyz"));
    check!(r, !sk_str_ends_with(xyz, "wxyz"));

    std::mem::swap(&mut xyz, &mut empty);
    check!(r, xyz == "");
    check!(r, empty == "xyz");
    r.finish();
}

#[test]
fn SkStringViewContains() {
    let mut r = Reporter::default();
    check!(r, sk_str_contains("ttttest1tttest2tttest3", "test"));
    check!(r, sk_str_contains("ttttest1tttest2tttest3", "test3"));
    check!(r, !sk_str_contains("ttttest1tttest2tttest3", "test4"));
    check!(r, sk_str_contains("", ""));
    check!(r, !sk_str_contains("", "a"));
    check!(r, sk_str_contains("abcabcd", "abcd"));
    check!(r, sk_str_contains("abc", ""));
    check!(r, sk_str_contains("abc", "a"));
    check!(r, sk_str_contains("abc", "b"));
    check!(r, sk_str_contains("abc", "c"));
    check!(r, sk_str_contains("abc", "ab"));
    check!(r, sk_str_contains("abc", "bc"));
    check!(r, !sk_str_contains("abc", "ac"));
    check!(r, !sk_str_contains("abc", "cb"));
    check!(r, !sk_str_contains("abc", "abcd"));
    r.finish();
}
#[test]
fn SkSharedMutexMultiThreaded() {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicI32, Ordering},
    };
    let sm = Arc::new(SkSharedMutex::new());
    let shared = Arc::new(std::array::from_fn::<_, 10, _>(|_| AtomicI32::new(0)));
    let value = Arc::new(AtomicI32::new(0));
    let r = Arc::new(Mutex::new(Reporter::default()));
    let group = SkTaskGroup::new();
    for index in 0..8 {
        let sm = sm.clone();
        let shared = shared.clone();
        let value = value.clone();
        let r = r.clone();
        group.add(move || {
            if index % 4 != 0 {
                for _ in 0..100000 {
                    sm.acquire();
                    sm.assert_held();
                    let v = shared[0].load(Ordering::Relaxed);
                    let mut r = r.lock().unwrap();
                    for i in 1..10 {
                        check!(r, v == shared[i].load(Ordering::Relaxed));
                    }
                    drop(r);
                    sm.release();
                }
            } else {
                for _ in 0..100000 {
                    sm.acquire_exclusive();
                    sm.assert_exclusive_held();
                    let v = value.fetch_add(1, Ordering::Relaxed) + 1;
                    for i in 0..10 {
                        shared[i].store(v, Ordering::Relaxed);
                    }
                    sm.release_exclusive();
                }
            }
        });
    }
    group.wait();
    Arc::try_unwrap(r)
        .ok()
        .unwrap()
        .into_inner()
        .unwrap()
        .finish();
}
