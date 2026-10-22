// Copyright 2025 Google LLC. BSD license: tests/LICENSE.skia.
// ExecutorTest.cpp: both full registrations and the three original factories.
use super::*;
use std::ffi::c_void;
use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};
use std::time::Duration;
struct CollectorData {
    count: usize,
    data: [i32; 200],
}
struct Collector {
    lock: SkSpinlock,
    data: std::cell::UnsafeCell<CollectorData>,
}
// As in the original helper, every access is protected by the production spinlock.
unsafe impl Sync for Collector {}
impl Default for Collector {
    fn default() -> Self {
        Self {
            lock: SkSpinlock::new(),
            data: std::cell::UnsafeCell::new(CollectorData {
                count: 0,
                data: [0; 200],
            }),
        }
    }
}
struct SpinGuard<'a>(&'a SkSpinlock);
impl<'a> SpinGuard<'a> {
    fn new(lock: &'a SkSpinlock) -> Self {
        lock.acquire();
        Self(lock)
    }
}
impl Drop for SpinGuard<'_> {
    fn drop(&mut self) {
        self.0.release();
    }
}
impl Collector {
    fn insert(&self, i: i32) {
        let _guard = SpinGuard::new(&self.lock);
        let d = unsafe { &mut *self.data.get() };
        if d.count >= 200 {
            return;
        }
        d.data[d.count] = i;
        d.count += 1;
    }
    fn count(&self) -> usize {
        let _guard = SpinGuard::new(&self.lock);
        unsafe { (*self.data.get()).count }
    }
    fn data_is_in_order(&self) -> bool {
        let _guard = SpinGuard::new(&self.lock);
        let d = unsafe { &*self.data.get() };
        for i in 0..d.count {
            if i < 100 {
                if d.data[i] >= 100 {
                    return false;
                }
            } else if d.data[i] < 100 {
                return false;
            }
        }
        true
    }
}
fn functional_priority_test(r: &mut Reporter, e: SkFunctionalExecutor) {
    let group = SkTaskGroup::with_executor(&e);
    let collector = Arc::new(Collector::default());
    for i in 0..100 {
        let c = collector.clone();
        group.add_priority(
            move || {
                c.insert(i);
                std::thread::sleep(Duration::from_micros(50));
            },
            0,
        );
    }
    for i in 100..200 {
        let c = collector.clone();
        group.add_priority(
            move || {
                std::thread::sleep(Duration::from_millis(1));
                c.insert(i);
            },
            1,
        );
    }
    group.wait();
    check!(r, collector.count() == 200);
    check!(r, collector.data_is_in_order());
}
struct CallbackContext {
    collector: Arc<Collector>,
    pending: Arc<AtomicI32>,
    count: i32,
    priority: i32,
}
unsafe fn callback(context: *mut c_void) {
    let context = unsafe { &*context.cast::<CallbackContext>() };
    if context.priority == 1 {
        std::thread::sleep(Duration::from_millis(1));
    }
    context.collector.insert(context.count);
    if context.priority == 0 {
        std::thread::sleep(Duration::from_micros(50));
    }
    context.pending.fetch_add(-1, Ordering::Release);
}
fn callback_priority_test(r: &mut Reporter, e: SkCallbackExecutor) {
    let collector = Arc::new(Collector::default());
    let pending = Arc::new(AtomicI32::new(0)); // Fixed storage retains all contexts until executor destruction.
    let mut contexts: Box<[CallbackContext; 200]> =
        Box::new(std::array::from_fn(|i| CallbackContext {
            collector: collector.clone(),
            pending: pending.clone(),
            count: i as i32,
            priority: if i < 100 { 0 } else { 1 },
        }));
    for i in 0..100 {
        pending.fetch_add(1, Ordering::Relaxed);
        e.add_priority(
            SkExecutorCallbackWork::new(
                callback,
                (&mut contexts[i] as *mut CallbackContext).cast(),
            ),
            0,
        );
    }
    for i in 100..200 {
        pending.fetch_add(1, Ordering::Relaxed);
        e.add_priority(
            SkExecutorCallbackWork::new(
                callback,
                (&mut contexts[i] as *mut CallbackContext).cast(),
            ),
            1,
        );
    }
    while pending.load(Ordering::Acquire) > 0 {
        e.borrow();
    }
    drop(e);
    check!(r, collector.count() == 200);
    check!(r, collector.data_is_in_order());
}
#[test]
fn ExecutorTest_StdFunction() {
    let mut r = Reporter::default();
    let factories: [fn() -> SkFunctionalExecutor; 3] = [
        || SkFunctionalExecutors::make_multi_list_fifo_thread_pool(2, 4, false),
        || SkFunctionalExecutors::make_multi_list_lifo_thread_pool(2, 4, false),
        || SkFunctionalExecutors::make_fifo_thread_pool(4, false),
    ];
    for (i, make) in factories.into_iter().enumerate() {
        eprintln!(
            "original functional factory={i} lists={} threads=4 borrowing=false",
            if i == 2 { 1 } else { 2 }
        );
        functional_priority_test(&mut r, make());
    }
    r.finish();
}
#[test]
fn ExecutorTest_Callback() {
    let mut r = Reporter::default();
    let factories: [fn() -> SkCallbackExecutor; 3] = [
        || SkCallbackExecutors::make_multi_list_fifo_thread_pool(2, 4, false),
        || SkCallbackExecutors::make_multi_list_lifo_thread_pool(2, 4, false),
        || SkCallbackExecutors::make_fifo_thread_pool(4, false),
    ];
    for (i, make) in factories.into_iter().enumerate() {
        eprintln!(
            "original callback factory={i} lists={} threads=4 borrowing=false",
            if i == 2 { 1 } else { 2 }
        );
        callback_priority_test(&mut r, make());
    }
    r.finish();
}
