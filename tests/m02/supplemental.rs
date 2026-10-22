// Supplemental contracts, not original registration translations.
use super::fake_streams::*;
use super::*;
#[test]
fn supplemental_stream_boundaries_and_byte_order() {
    let mut r = Reporter::default();
    let mut s = SkMemoryStream::new(SkData::make_with_copy(&[1, 2, 3, 4]));
    let mut b = [0u8; 8];
    check!(r, s.peek(&mut b) == 4);
    check!(r, s.position() == 0);
    check!(r, s.read(&mut b[..2]) == 2);
    check!(r, s.position() == 2);
    check!(r, s.rewind());
    check!(r, s.position() == 0);
    check!(r, s.seek(99));
    check!(r, s.position() == 4);
    check!(r, s.is_at_end());
    check!(r, s.read(&mut b) == 0);
    check!(r, s.rewind());
    let mut w = SkDynamicMemoryWStream::new();
    check!(r, w.write_u16(0x1234));
    check!(r, w.write_u32(0x89abcdef));
    let d = w.detach_as_data();
    check!(r, d.bytes() == [0x34, 0x12, 0xef, 0xcd, 0xab, 0x89]);
    r.finish();
}
#[test]
fn supplemental_packed_truncation() {
    let mut r = Reporter::default();
    for b in [&[][..], &[0xfe], &[0xfe, 1], &[0xff, 1, 2, 3]] {
        let mut s = SkMemoryStream::new(SkData::make_with_copy(b));
        let actual = s.read_packed_u32();
        eprintln!("packed input={b:02x?} expected=None actual={actual:?}");
        check!(r, actual.is_none());
    }
    r.finish();
}
#[test]
fn supplemental_without_copy_identity() {
    let mut r = Reporter::default();
    static INPUT: [u8; 4] = [1, 2, 3, 4];
    let d = SkData::make_without_copy(&INPUT);
    eprintln!(
        "without-copy expected={:p} actual={:p}",
        INPUT.as_ptr(),
        d.data()
    );
    check!(r, d.data() == INPUT.as_ptr());
    r.finish();
}
#[test]
fn supplemental_empty_data_release() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let mut r = Reporter::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    {
        let _d = SkData::make_with_proc(Vec::new(), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
    }
    let actual = calls.load(Ordering::SeqCst);
    eprintln!("empty data release expected=1 actual={actual}");
    check!(r, actual == 1);
    r.finish();
}
#[test]
fn supplemental_arena_external_storage() {
    let mut r = Reporter::default();
    let mut storage = [0u8; 1000];
    let mut a = rust_skia::ArenaAlloc::with_storage(storage.as_mut_ptr(), 1000, 1000);
    let p = a.make(3i32) as usize;
    let start = storage.as_ptr() as usize;
    eprintln!(
        "external storage range={start:#x}..{:#x} actual={p:#x}",
        start + 1000
    );
    check!(r, p >= start && p < start + 1000);
    r.finish();
}
#[test]
fn supplemental_fake_streams() {
    let mut r = Reporter::default();
    let d = SkData::make_with_copy(b"abcd");
    let mut a = NotAssetMemStream::new(d.clone());
    check!(r, !a.has_position());
    check!(r, !a.has_length());
    let mut b = [0; 4];
    check!(r, a.peek(&mut b) == 4);
    check!(r, a.read(&mut b[..1]) == 1);
    check!(r, a.rewind());
    let mut n = NonseekableStream::new(d.clone());
    check!(r, !n.rewind());
    check!(r, !n.seek(0));
    let mut h = HaltingStream::new(d, 2);
    check!(r, h.read(&mut b) == 2);
    check!(r, !h.is_at_end());
    check!(r, !h.is_all_data_received());
    h.add_new_data(99);
    check!(r, h.is_all_data_received());
    check!(r, h.read(&mut b) == 2);
    check!(r, h.is_at_end());
    r.finish();
}
#[test]
fn supplemental_data_overflow_rejection() {
    let mut r = Reporter::default();
    let d = SkData::make_with_copy(b"abcd");
    check!(r, d.share_subset(usize::MAX, 1).is_none());
    check!(r, d.copy_subset(1, usize::MAX).is_none());
    check!(r, d.copy_range(usize::MAX, 1) == 0);
    r.finish();
}
#[test]
fn supplemental_checksum_reference_vectors() {
    let mut r = Reporter::default();
    let cases: [(&[u8], [u32; 3]); 4] = [
        (b"", [3804095577, 638533687, 2821084191]),
        (b"a", [164791601, 459268265, 4117487878]),
        (b"abc", [3411111026, 2184336823, 1582327983]),
        (
            b"abcdefghijklmnopqrstuvwxyz",
            [517385426, 3575925044, 3468900993],
        ),
    ];
    for (bytes, expected) in cases {
        for (i, seed) in [0, 1, u32::MAX].into_iter().enumerate() {
            let actual = sk_checksum_hash32(bytes, seed);
            eprintln!(
                "checksum bytes={bytes:?} seed={seed} expected={} actual={actual}",
                expected[i]
            );
            check!(r, actual == expected[i]);
        }
    }
    r.finish();
}
#[test]
fn supplemental_smart_pointer_size() {
    let mut r = Reporter::default();
    let e = sk_make_sp(SkRefCnt::new());
    let actual = std::mem::size_of_val(&e);
    let expected = std::mem::size_of::<*const ()>();
    eprintln!("RefCntTest.cpp:190 owning-pointer width expected={expected} actual={actual}");
    check!(r, actual == expected);
    r.finish();
}
#[test]
fn supplemental_executor_background_workers() {
    use std::time::Duration;
    let mut r = Reporter::default();
    let factories: [fn() -> SkFunctionalExecutor; 3] = [
        || SkExecutors::<SkFunctionalExecutor>::make_multi_list_fifo_thread_pool(2, 4, false),
        || SkExecutors::<SkFunctionalExecutor>::make_multi_list_lifo_thread_pool(2, 4, false),
        || SkExecutors::<SkFunctionalExecutor>::make_fifo_thread_pool(4, false),
    ];
    for (i, factory) in factories.into_iter().enumerate() {
        let e = factory();
        let (tx, rx) = std::sync::mpsc::channel();
        e.add_high(Box::new(move || {
            tx.send(()).unwrap();
        }));
        let completed = rx.recv_timeout(Duration::from_millis(200)).is_ok();
        eprintln!(
            "factory {i} workers=4 allow_borrowing=false completed_without_borrow={completed}"
        );
        check!(r, completed);
    }
    r.finish();
}

// A safe cast's supplied projection must be honored even when the base view
// is not at byte zero. Both subobjects have valid Base representations, so a
// wrong result can be observed without invoking undefined behavior.
#[test]
fn supplemental_smart_pointer_projected_base() {
    #[repr(C)]
    struct Base {
        count: RefCntBase,
        value: i32,
    }
    impl RefCnt for Base {
        fn ref_cnt_base(&self) -> &RefCntBase {
            &self.count
        }
    }
    #[repr(C)]
    struct Derived {
        other: Base,
        base: Base,
    }
    impl RefCnt for Derived {
        fn ref_cnt_base(&self) -> &RefCntBase {
            &self.base.count
        }
    }
    let make = || {
        rust_skia::sk_make_sp(Derived {
            other: Base {
                count: RefCntBase::new(),
                value: 11,
            },
            base: Base {
                count: RefCntBase::new(),
                value: 42,
            },
        })
    };
    let mut r = Reporter::default();
    let original = make();
    let expected = std::ptr::addr_of!(original.base);
    let copy: SkSp<Base> = original.clone_as(|d| &d.base);
    eprintln!(
        "clone_as expected base={expected:p} value=42 actual={:p} value={}",
        copy.as_ptr(),
        copy.value
    );
    check!(r, copy.as_ptr() == expected);
    check!(r, copy.value == 42);
    let moved: SkSp<Base> = original.cast(|d| &d.base);
    eprintln!(
        "cast expected base={expected:p} value=42 actual={:p} value={}",
        moved.as_ptr(),
        moved.value
    );
    check!(r, moved.as_ptr() == expected);
    check!(r, moved.value == 42);
    r.finish();
}
