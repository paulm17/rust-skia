//! Executors, thread pools, callback pools, and task groups.

use std::collections::VecDeque;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread::{self, JoinHandle};

pub trait SkExecutor {
    fn add(&self, task: Box<dyn FnOnce() + Send + 'static>);
    fn add_high(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.add(task);
    }
    fn add_low(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.add(task);
    }
    fn drain(&self) {}
}

pub type SkExecutorCallbackContext = *mut c_void;
pub type SkExecutorCallbackFunction = unsafe fn(SkExecutorCallbackContext);

#[derive(Clone, Copy)]
pub struct SkExecutorCallbackWork {
    pub callback: Option<SkExecutorCallbackFunction>,
    pub context: SkExecutorCallbackContext,
}

unsafe impl Send for SkExecutorCallbackWork {}

impl SkExecutorCallbackWork {
    pub fn new(callback: SkExecutorCallbackFunction, context: SkExecutorCallbackContext) -> Self {
        Self {
            callback: Some(callback),
            context,
        }
    }

    pub fn null() -> Self {
        Self {
            callback: None,
            context: std::ptr::null_mut(),
        }
    }
}

enum WorkItem {
    Functional(Box<dyn FnOnce() + Send + 'static>),
    Callback(SkExecutorCallbackWork),
}

impl WorkItem {
    fn execute(self) {
        match self {
            WorkItem::Functional(task) => {
                let _ = catch_unwind(AssertUnwindSafe(task));
            }
            WorkItem::Callback(work) => {
                if let Some(callback) = work.callback {
                    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
                        callback(work.context);
                    }));
                }
            }
        }
    }
}

struct PoolState {
    lists: Vec<VecDeque<WorkItem>>,
    pending: usize,
    shutdown: bool,
}

struct Pool {
    state: Mutex<PoolState>,
    cond: Condvar,
    allow_borrowing: bool,
    fifo: bool,
}

impl Pool {
    fn create(
        num_work_lists: usize,
        threads: usize,
        allow_borrowing: bool,
        fifo: bool,
    ) -> (Arc<Self>, Arc<ThreadSet>) {
        let num_work_lists = num_work_lists.max(1);
        let worker_count = if threads == 0 {
            thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            threads
        };
        let pool = Arc::new(Self {
            state: Mutex::new(PoolState {
                lists: (0..num_work_lists).map(|_| VecDeque::new()).collect(),
                pending: 0,
                shutdown: false,
            }),
            cond: Condvar::new(),
            allow_borrowing,
            fifo,
        });
        let thread_set = ThreadSet::new(&pool, worker_count);
        (pool, thread_set)
    }

    fn pop_locked(&self, state: &mut PoolState) -> Option<WorkItem> {
        for list in &mut state.lists {
            if !list.is_empty() {
                return if self.fifo {
                    list.pop_front()
                } else {
                    list.pop_back()
                };
            }
        }
        None
    }

    fn add(&self, item: WorkItem, work_list: i32) {
        let mut state = self.state.lock().unwrap();
        let index = work_list.clamp(0, state.lists.len() as i32 - 1) as usize;
        state.lists[index].push_back(item);
        state.pending += 1;
        self.cond.notify_one();
    }

    fn complete_one(&self) {
        let mut state = self.state.lock().unwrap();
        state.pending = state.pending.saturating_sub(1);
        if state.pending == 0 {
            self.cond.notify_all();
        }
    }

    fn execute_one(&self, item: WorkItem) {
        item.execute();
        self.complete_one();
    }

    fn borrow_one(&self) {
        if !self.allow_borrowing {
            return;
        }
        let item = {
            let mut state = self.state.lock().unwrap();
            self.pop_locked(&mut state)
        };
        if let Some(item) = item {
            self.execute_one(item);
        }
    }

    fn drain(&self) {
        loop {
            let item = {
                let state = self.state.lock().unwrap();
                if state.pending == 0 {
                    break;
                }
                let mut state = state;
                self.pop_locked(&mut state)
            };
            match item {
                Some(item) => self.execute_one(item),
                None => break,
            }
        }
    }

    fn wait(&self) {
        let mut state = self.state.lock().unwrap();
        while state.pending > 0 {
            state = self.cond.wait(state).unwrap();
        }
    }

    fn shutdown(&self) {
        self.state.lock().unwrap().shutdown = true;
        self.cond.notify_all();
    }
}

struct ThreadSet {
    pool: Weak<Pool>,
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl ThreadSet {
    fn new(pool: &Arc<Pool>, count: usize) -> Arc<Self> {
        let set = Arc::new(Self {
            pool: Arc::downgrade(pool),
            handles: Mutex::new(Vec::with_capacity(count)),
        });
        for _ in 0..count {
            let weak = Arc::downgrade(pool);
            let handle = thread::Builder::new()
                .name("rust-skia-executor".to_string())
                .spawn(move || worker_loop(weak))
                .expect("failed to spawn SkExecutor worker");
            set.handles.lock().unwrap().push(handle);
        }
        set
    }
}

impl Drop for ThreadSet {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            pool.shutdown();
        }
        let handles = std::mem::take(&mut *self.handles.lock().unwrap());
        for handle in handles {
            let _ = handle.join();
        }
    }
}

fn worker_loop(weak: Weak<Pool>) {
    loop {
        let pool = match weak.upgrade() {
            Some(pool) => pool,
            None => return,
        };
        let item = {
            let mut state = pool.state.lock().unwrap();
            loop {
                if state.shutdown {
                    return;
                }
                if let Some(item) = pool.pop_locked(&mut state) {
                    break item;
                }
                state = pool.cond.wait(state).unwrap();
            }
        };
        pool.execute_one(item);
        drop(pool);
    }
}

#[derive(Clone)]
pub struct SkFunctionalExecutor {
    pool: Arc<Pool>,
    _threads: Arc<ThreadSet>,
}

impl Default for SkFunctionalExecutor {
    fn default() -> Self {
        Self::new(1)
    }
}

impl SkFunctionalExecutor {
    pub fn new(threads: usize) -> Self {
        Self::with_mode(2, threads, false, true)
    }

    pub fn with_mode(
        num_work_lists: usize,
        threads: usize,
        allow_borrowing: bool,
        fifo: bool,
    ) -> Self {
        let (pool, _threads) = Pool::create(num_work_lists, threads, allow_borrowing, fifo);
        Self { pool, _threads }
    }

    pub fn add_with_priority<F>(&self, f: F, work_list: i32)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.add(WorkItem::Functional(Box::new(f)), work_list);
    }

    pub fn pending(&self) -> usize {
        self.pool.state.lock().unwrap().pending
    }

    pub fn wait(&self) {
        self.pool.wait();
    }

    pub fn shutdown(&self) {
        self.pool.shutdown();
    }

    pub fn borrow(&self) {
        self.pool.borrow_one();
    }

    pub fn run(&self) {
        self.pool.drain();
    }
}

impl SkExecutor for SkFunctionalExecutor {
    fn add(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.add_high(task);
    }

    fn add_high(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.pool.add(WorkItem::Functional(task), 0);
    }

    fn add_low(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.pool.add(WorkItem::Functional(task), i32::MAX);
    }

    fn drain(&self) {
        self.pool.drain();
    }
}

#[derive(Clone)]
pub struct SkCallbackExecutor {
    pool: Arc<Pool>,
    _threads: Arc<ThreadSet>,
}

impl SkCallbackExecutor {
    pub fn add(&self, work: SkExecutorCallbackWork) {
        self.add_priority(work, 0);
    }

    pub fn add_priority(&self, work: SkExecutorCallbackWork, work_list: i32) {
        self.pool.add(WorkItem::Callback(work), work_list);
    }

    pub fn add_high(&self, work: SkExecutorCallbackWork) {
        self.pool.add(WorkItem::Callback(work), 0);
    }

    pub fn add_low(&self, work: SkExecutorCallbackWork) {
        self.pool.add(WorkItem::Callback(work), i32::MAX);
    }

    pub fn borrow(&self) {
        self.pool.borrow_one();
    }

    pub fn pending(&self) -> usize {
        self.pool.state.lock().unwrap().pending
    }

    pub fn wait(&self) {
        self.pool.wait();
    }

    pub fn shutdown(&self) {
        self.pool.shutdown();
    }
}

/// Factory namespace corresponding to `SkExecutors<E>`.
pub struct SkExecutors<E>(std::marker::PhantomData<fn() -> E>);

impl SkExecutors<SkFunctionalExecutor> {
    pub fn make_fifo_thread_pool(threads: usize, allow_borrowing: bool) -> SkFunctionalExecutor {
        SkFunctionalExecutor::with_mode(1, threads, allow_borrowing, true)
    }

    pub fn make_lifo_thread_pool(threads: usize, allow_borrowing: bool) -> SkFunctionalExecutor {
        SkFunctionalExecutor::with_mode(1, threads, allow_borrowing, false)
    }

    pub fn make_multi_list_fifo_thread_pool(
        num_work_lists: usize,
        threads: usize,
        allow_borrowing: bool,
    ) -> SkFunctionalExecutor {
        SkFunctionalExecutor::with_mode(num_work_lists, threads, allow_borrowing, true)
    }

    pub fn make_multi_list_lifo_thread_pool(
        num_work_lists: usize,
        threads: usize,
        allow_borrowing: bool,
    ) -> SkFunctionalExecutor {
        SkFunctionalExecutor::with_mode(num_work_lists, threads, allow_borrowing, false)
    }
}

impl SkExecutors<SkCallbackExecutor> {
    pub fn make_fifo_thread_pool(threads: usize, allow_borrowing: bool) -> SkCallbackExecutor {
        let (pool, _threads) = Pool::create(1, threads, allow_borrowing, true);
        SkCallbackExecutor { pool, _threads }
    }

    pub fn make_lifo_thread_pool(threads: usize, allow_borrowing: bool) -> SkCallbackExecutor {
        let (pool, _threads) = Pool::create(1, threads, allow_borrowing, false);
        SkCallbackExecutor { pool, _threads }
    }

    pub fn make_multi_list_fifo_thread_pool(
        num_work_lists: usize,
        threads: usize,
        allow_borrowing: bool,
    ) -> SkCallbackExecutor {
        let (pool, _threads) = Pool::create(num_work_lists, threads, allow_borrowing, true);
        SkCallbackExecutor { pool, _threads }
    }

    pub fn make_multi_list_lifo_thread_pool(
        num_work_lists: usize,
        threads: usize,
        allow_borrowing: bool,
    ) -> SkCallbackExecutor {
        let (pool, _threads) = Pool::create(num_work_lists, threads, allow_borrowing, false);
        SkCallbackExecutor { pool, _threads }
    }
}

struct TaskGroupGuard(Arc<AtomicUsize>);

impl Drop for TaskGroupGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}

pub struct SkTaskGroup {
    tasks: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
    executor: Option<SkFunctionalExecutor>,
    pending: Arc<AtomicUsize>,
}

impl Default for SkTaskGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl SkTaskGroup {
    /// Create an unbound task group. Its tasks run when `wait`/`run` is called.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
            executor: None,
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create a task group whose work is submitted to `executor`.
    pub fn with_executor(executor: &SkFunctionalExecutor) -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
            executor: Some(executor.clone()),
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn new_with_executor(executor: &SkFunctionalExecutor) -> Self {
        Self::with_executor(executor)
    }

    pub fn add<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.add_priority(f, 0);
    }

    pub fn add_priority<F: FnOnce() + Send + 'static>(&self, f: F, work_list: i32) {
        if let Some(executor) = &self.executor {
            let pending = self.pending.clone();
            pending.fetch_add(1, Ordering::Relaxed);
            let guard = TaskGroupGuard(pending);
            executor.add_with_priority(
                move || {
                    f();
                    drop(guard);
                },
                work_list,
            );
        } else {
            self.tasks.lock().unwrap().push(Box::new(f));
        }
    }

    pub fn done(&self) -> bool {
        if self.executor.is_some() {
            self.pending.load(Ordering::Acquire) == 0
        } else {
            self.tasks.lock().unwrap().is_empty()
        }
    }

    pub fn wait(&self) {
        if let Some(executor) = &self.executor {
            while !self.done() {
                executor.pool.borrow_one();
                thread::yield_now();
            }
        } else {
            let tasks = std::mem::take(&mut *self.tasks.lock().unwrap());
            for task in tasks {
                task();
            }
        }
    }

    pub fn run(&self) {
        self.wait();
    }
}

impl Drop for SkTaskGroup {
    fn drop(&mut self) {
        self.wait();
    }
}

pub fn sk_executor_default() -> Arc<SkFunctionalExecutor> {
    Arc::new(SkFunctionalExecutor::new(
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    ))
}

pub type SkFunctionalExecutors = SkExecutors<SkFunctionalExecutor>;
pub type SkCallbackExecutors = SkExecutors<SkCallbackExecutor>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    fn functional_run(executor: SkFunctionalExecutor) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let group = SkTaskGroup::with_executor(&executor);
        for i in 0..100usize {
            let seen = seen.clone();
            group.add_priority(
                move || {
                    seen.lock().unwrap().push(i);
                    thread::sleep(Duration::from_micros(50));
                },
                0,
            );
        }
        for i in 0..100usize {
            let seen = seen.clone();
            group.add_priority(
                move || {
                    thread::sleep(Duration::from_millis(1));
                    seen.lock().unwrap().push(100 + i);
                },
                1,
            );
        }
        group.wait();
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 200);
        assert!(seen[..100].iter().all(|&i| i < 100));
        assert!(seen[100..].iter().all(|&i| i >= 100));
    }

    #[test]
    fn dev_functional_pools_run_background_priority_work() {
        functional_run(SkFunctionalExecutors::make_multi_list_fifo_thread_pool(
            2, 4, false,
        ));
        functional_run(SkFunctionalExecutors::make_multi_list_lifo_thread_pool(
            2, 4, false,
        ));
        functional_run(SkFunctionalExecutors::make_fifo_thread_pool(4, false));
    }

    struct CallbackContext {
        seen: Arc<Mutex<Vec<usize>>>,
        pending: Arc<AtomicUsize>,
        index: usize,
        sleep: Duration,
    }

    unsafe fn callback(context: SkExecutorCallbackContext) {
        let context = unsafe { Box::from_raw(context as *mut CallbackContext) };
        thread::sleep(context.sleep);
        context.seen.lock().unwrap().push(context.index);
        context.pending.fetch_sub(1, Ordering::Release);
    }

    fn callback_run(executor: SkCallbackExecutor) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let pending = Arc::new(AtomicUsize::new(200));
        for i in 0..100usize {
            let context = Box::into_raw(Box::new(CallbackContext {
                seen: seen.clone(),
                pending: pending.clone(),
                index: i,
                sleep: if i < 100 {
                    Duration::from_micros(50)
                } else {
                    Duration::from_millis(1)
                },
            })) as SkExecutorCallbackContext;
            executor.add_high(SkExecutorCallbackWork::new(callback, context));
        }
        for i in 100..200usize {
            let context = Box::into_raw(Box::new(CallbackContext {
                seen: seen.clone(),
                pending: pending.clone(),
                index: i,
                sleep: Duration::from_millis(1),
            })) as SkExecutorCallbackContext;
            executor.add_low(SkExecutorCallbackWork::new(callback, context));
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while pending.load(Ordering::Acquire) > 0 {
            assert!(Instant::now() < deadline, "callback pool did not finish");
            executor.borrow();
            thread::yield_now();
        }
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 200);
        assert!(seen[..100].iter().all(|&i| i < 100));
        assert!(seen[100..].iter().all(|&i| i >= 100));
    }

    #[test]
    fn dev_callback_pools_run_background_priority_work() {
        callback_run(SkCallbackExecutors::make_multi_list_fifo_thread_pool(
            2, 4, false,
        ));
        callback_run(SkCallbackExecutors::make_multi_list_lifo_thread_pool(
            2, 4, false,
        ));
        callback_run(SkCallbackExecutors::make_fifo_thread_pool(4, false));
    }
}
