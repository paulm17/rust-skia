//! Mutex, once, semaphore, spinlock, and shared mutex primitives.

use std::sync::{Condvar, Mutex, Once};

pub type SkMutex = Mutex<()>;

pub struct SkOnce {
    once: Once,
}

impl Default for SkOnce {
    fn default() -> Self {
        Self::new()
    }
}

impl SkOnce {
    pub const fn new() -> Self {
        Self { once: Once::new() }
    }
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        self.once.call_once(f);
    }
    pub fn is_done(&self) -> bool {
        self.once.is_completed()
    }
}

pub struct SkSemaphore {
    count: Mutex<usize>,
    cond: Condvar,
}

impl SkSemaphore {
    pub const fn new(count: usize) -> Self {
        Self {
            count: Mutex::new(count),
            cond: Condvar::new(),
        }
    }
    pub fn signal(&self, count: usize) {
        let mut guard = self.count.lock().unwrap();
        *guard += count;
        self.cond.notify_all();
    }
    pub fn wait(&self) {
        let mut guard = self.count.lock().unwrap();
        while *guard == 0 {
            guard = self.cond.wait(guard).unwrap();
        }
        *guard -= 1;
    }
    pub fn try_wait(&self) -> bool {
        let mut guard = self.count.lock().unwrap();
        if *guard > 0 {
            *guard -= 1;
            true
        } else {
            false
        }
    }
    pub fn count(&self) -> usize {
        *self.count.lock().unwrap()
    }
}

#[derive(Default)]
pub struct SkSpinlock {
    locked: std::sync::atomic::AtomicBool,
}

impl SkSpinlock {
    pub const fn new() -> Self {
        Self {
            locked: std::sync::atomic::AtomicBool::new(false),
        }
    }
    pub fn acquire(&self) {
        while self
            .locked
            .compare_exchange_weak(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            std::hint::spin_loop();
        }
    }
    pub fn release(&self) {
        self.locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
    pub fn try_acquire(&self) -> bool {
        self.locked
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
    }
}

struct SharedState {
    readers: usize,
    writer: bool,
    waiting_writers: usize,
}

pub struct SkSharedMutex {
    state: Mutex<SharedState>,
    cond: Condvar,
}

impl Default for SkSharedMutex {
    fn default() -> Self {
        Self::new()
    }
}

impl SkSharedMutex {
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(SharedState {
                readers: 0,
                writer: false,
                waiting_writers: 0,
            }),
            cond: Condvar::new(),
        }
    }

    pub fn acquire(&self) {
        let mut state = self.state.lock().unwrap();
        while state.writer || state.waiting_writers > 0 {
            state = self.cond.wait(state).unwrap();
        }
        state.readers += 1;
    }

    pub fn release(&self) {
        let mut state = self.state.lock().unwrap();
        if state.readers > 0 {
            state.readers -= 1;
        }
        self.cond.notify_all();
    }

    pub fn acquire_exclusive(&self) {
        let mut state = self.state.lock().unwrap();
        state.waiting_writers += 1;
        while state.writer || state.readers > 0 {
            state = self.cond.wait(state).unwrap();
        }
        state.waiting_writers -= 1;
        state.writer = true;
    }

    pub fn release_exclusive(&self) {
        let mut state = self.state.lock().unwrap();
        state.writer = false;
        self.cond.notify_all();
    }

    pub fn assert_held(&self) {
        let state = self.state.lock().unwrap();
        debug_assert!(state.readers > 0 || state.writer);
    }

    pub fn assert_exclusive_held(&self) {
        let state = self.state.lock().unwrap();
        debug_assert!(state.writer);
    }
}
