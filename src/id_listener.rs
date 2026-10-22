//! `SkIDChangeListener` collection.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub trait SkIDChangeListener: Send + Sync {
    fn changed(&self);
    fn should_deregister(&self) -> bool {
        false
    }
}

pub struct SkIDChangeListenerFunc<F: Fn() + Send + Sync> {
    func: F,
    deregister: AtomicBool,
}

impl<F: Fn() + Send + Sync> SkIDChangeListenerFunc<F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            deregister: AtomicBool::new(false),
        }
    }
}

impl<F: Fn() + Send + Sync> SkIDChangeListener for SkIDChangeListenerFunc<F> {
    fn changed(&self) {
        if !self.deregister.load(Ordering::Acquire) {
            (self.func)();
        }
    }
    fn should_deregister(&self) -> bool {
        self.deregister.load(Ordering::Acquire)
    }
}

#[derive(Default)]
pub struct SkIDChangeListenerList {
    listeners: Vec<Arc<dyn SkIDChangeListener>>,
}

impl SkIDChangeListenerList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, listener: Option<Arc<dyn SkIDChangeListener>>) {
        self.listeners.retain(|l| !l.should_deregister());
        if let Some(listener) = listener {
            assert!(!listener.should_deregister());
            self.listeners.push(listener);
        }
    }
    pub fn changed(&mut self) {
        let listeners = std::mem::take(&mut self.listeners);
        for listener in listeners {
            if !listener.should_deregister() {
                listener.changed();
            }
        }
    }
    pub fn reset(&mut self) {
        self.listeners.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
    pub fn len(&self) -> usize {
        self.listeners.len()
    }
}

impl Drop for SkIDChangeListenerList {
    fn drop(&mut self) {
        let listeners = std::mem::take(&mut self.listeners);
        for listener in listeners {
            if !listener.should_deregister() {
                listener.changed();
            }
        }
    }
}
