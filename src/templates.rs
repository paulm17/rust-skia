//! Small RAII/template helpers: AutoTArray, AutoTMalloc, scope exit, etc.

use std::ops::{Deref, DerefMut};

use crate::containers::SkTArray;

#[derive(Clone, Debug)]
pub struct AutoTArray<T, const N: usize> {
    data: SkTArray<T>,
}

impl<T, const N: usize> Default for AutoTArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> AutoTArray<T, N> {
    pub fn new() -> Self {
        Self {
            data: SkTArray::new(),
        }
    }
    pub fn count(&self) -> usize {
        self.data.count()
    }
    pub fn empty(&self) -> bool {
        self.data.empty()
    }
    pub fn with_count(count: usize) -> Self
    where
        T: Default,
    {
        let mut data = SkTArray::new();
        data.resize_with(count, T::default);
        Self { data }
    }
    pub fn get(&self) -> *const T {
        if self.data.count() == 0 {
            std::ptr::null()
        } else {
            self.data.data()
        }
    }
    pub fn data(&self) -> *const T {
        self.get()
    }
    pub fn data_mut(&mut self) -> *mut T {
        if self.data.count() == 0 {
            std::ptr::null_mut()
        } else {
            self.data.data_mut()
        }
    }
    pub fn size(&self) -> usize {
        self.data.count()
    }
    pub fn clear(&mut self) {
        self.data.reset();
    }
    pub fn reset(&mut self, count: usize)
    where
        T: Default,
    {
        self.data.reset();
        self.data.resize_with(count, T::default);
    }
    pub fn trim_to(&mut self, count: usize) {
        self.data.trim_to(count);
    }
    pub fn push_back(&mut self, value: T) {
        self.data.push_back(value);
    }
}

impl<T, const N: usize> Deref for AutoTArray<T, N> {
    type Target = SkTArray<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T, const N: usize> DerefMut for AutoTArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Clone, Debug)]
pub struct AutoSTArray<T, const N: usize> {
    data: SkTArray<T>,
}

impl<T, const N: usize> Default for AutoSTArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> AutoSTArray<T, N> {
    pub fn new() -> Self {
        Self {
            data: SkTArray::new(),
        }
    }
    pub fn count(&self) -> usize {
        self.data.count()
    }
    pub fn empty(&self) -> bool {
        self.data.empty()
    }
    pub fn with_count(count: usize) -> Self
    where
        T: Default,
    {
        let mut data = SkTArray::new();
        data.resize_with(count, T::default);
        Self { data }
    }
    pub fn get(&self) -> *const T {
        if self.data.count() == 0 {
            std::ptr::null()
        } else {
            self.data.data()
        }
    }
    pub fn data(&self) -> *const T {
        self.get()
    }
    pub fn data_mut(&mut self) -> *mut T {
        if self.data.count() == 0 {
            std::ptr::null_mut()
        } else {
            self.data.data_mut()
        }
    }
    pub fn size(&self) -> usize {
        self.data.count()
    }
    pub fn clear(&mut self) {
        self.data.reset();
    }
    pub fn reset(&mut self, count: usize)
    where
        T: Default,
    {
        self.data.reset();
        self.data.resize_with(count, T::default);
    }
    pub fn trim_to(&mut self, count: usize) {
        self.data.trim_to(count);
    }
    pub fn push_back(&mut self, value: T) {
        self.data.push_back(value);
    }
}

impl<T, const N: usize> Deref for AutoSTArray<T, N> {
    type Target = SkTArray<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T, const N: usize> DerefMut for AutoSTArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Clone, Debug)]
pub struct AutoTMalloc<T> {
    data: Vec<T>,
}

impl<T: Default> Default for AutoTMalloc<T> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<T: Default> AutoTMalloc<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn reset(&mut self, count: usize) {
        if count == 0 {
            self.data = Vec::new();
        } else {
            self.data.resize_with(count, T::default);
        }
    }
    pub fn realloc(&mut self, count: usize) {
        self.data.resize_with(count, T::default);
    }
    pub fn get(&self) -> *const T {
        if self.data.is_empty() {
            std::ptr::null()
        } else {
            self.data.as_ptr()
        }
    }
    pub fn data(&self) -> *const T {
        self.get()
    }
    pub fn data_mut(&mut self) -> *mut T {
        if self.data.is_empty() {
            std::ptr::null_mut()
        } else {
            self.data.as_mut_ptr()
        }
    }
    pub fn count(&self) -> usize {
        self.data.len()
    }
}

impl<T: Default> Deref for AutoTMalloc<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T: Default> DerefMut for AutoTMalloc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Clone, Debug)]
pub struct AutoSTMalloc<const N: usize, T> {
    data: Vec<T>,
}

impl<const N: usize, T: Default> Default for AutoSTMalloc<N, T> {
    fn default() -> Self {
        Self {
            data: (0..N).map(|_| T::default()).collect(),
        }
    }
}

impl<const N: usize, T: Default> AutoSTMalloc<N, T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn reset(&mut self, count: usize) {
        if count == 0 {
            self.data = Vec::new();
        } else {
            self.data.resize_with(count, T::default);
        }
    }
    pub fn realloc(&mut self, count: usize) {
        self.data.resize_with(count, T::default);
    }
    pub fn get(&self) -> *const T {
        if self.data.is_empty() {
            std::ptr::null()
        } else {
            self.data.as_ptr()
        }
    }
    pub fn data(&self) -> *const T {
        self.get()
    }
    pub fn data_mut(&mut self) -> *mut T {
        if self.data.is_empty() {
            std::ptr::null_mut()
        } else {
            self.data.as_mut_ptr()
        }
    }
    pub fn count(&self) -> usize {
        self.data.len()
    }
}

impl<const N: usize, T: Default> Deref for AutoSTMalloc<N, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<const N: usize, T: Default> DerefMut for AutoSTMalloc<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct SkScopeExit<F: FnOnce()> {
    func: Option<F>,
}

impl<F: FnOnce()> SkScopeExit<F> {
    pub fn new(func: F) -> Self {
        Self { func: Some(func) }
    }
    pub fn release(&mut self) {
        self.func = None;
    }
}

impl<F: FnOnce()> Drop for SkScopeExit<F> {
    fn drop(&mut self) {
        if let Some(func) = self.func.take() {
            func();
        }
    }
}

pub struct SkNoDestructor<T> {
    value: std::mem::ManuallyDrop<T>,
}

impl<T> SkNoDestructor<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: std::mem::ManuallyDrop::new(value),
        }
    }
}
impl<T> Deref for SkNoDestructor<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> DerefMut for SkNoDestructor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_auto_arrays_expose_null_storage_when_empty() {
        let mut a = AutoTArray::<i32, 4>::new();
        assert!(a.get().is_null() && a.data().is_null() && a.data_mut().is_null());
        a.reset(2);
        assert!(!a.data().is_null());
        a.reset(0);
        assert!(a.get().is_null() && a.data().is_null() && a.data_mut().is_null());
        a.push_back(7);
        let moved = std::mem::take(&mut a);
        assert!(!moved.data().is_null());
        assert!(a.get().is_null() && a.data().is_null() && a.data_mut().is_null());

        let mut b = AutoSTArray::<i32, 4>::new();
        assert!(b.get().is_null() && b.data().is_null() && b.data_mut().is_null());
        b.reset(2);
        assert!(!b.data().is_null());
        b.reset(0);
        assert!(b.get().is_null() && b.data().is_null() && b.data_mut().is_null());
        b.push_back(7);
        let moved = std::mem::take(&mut b);
        assert!(!moved.data().is_null());
        assert!(b.get().is_null() && b.data().is_null() && b.data_mut().is_null());
    }
}
