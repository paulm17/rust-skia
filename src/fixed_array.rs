//! Fixed-capacity array from `src/core/SkFixedArray.h`.

use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedArray<const N: usize, T> {
    data: [T; N],
    size: usize,
}

impl<const N: usize, T: Default + Copy> Default for FixedArray<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, T: Default + Copy> FixedArray<N, T> {
    pub fn new() -> Self {
        debug_assert!(N < 256);
        Self {
            data: std::array::from_fn(|_| T::default()),
            size: 0,
        }
    }

    pub fn with_capacity(reserve_count: usize) -> Self {
        assert!(reserve_count <= N);
        Self::new()
    }

    pub fn from_slice(src: &[T]) -> Self {
        let mut result = Self::new();
        result.reset(src);
        result
    }

    pub fn reset(&mut self, src: &[T]) {
        assert!(src.len() <= N);
        self.data[..src.len()].copy_from_slice(src);
        self.size = src.len();
    }

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    pub fn capacity(&self) -> usize {
        N
    }
    pub const fn const_capacity() -> usize {
        N
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn resize(&mut self, new_size: usize) {
        assert!(new_size <= N);
        if new_size > self.size {
            for slot in &mut self.data[self.size..new_size] {
                *slot = T::default();
            }
        }
        self.size = new_size;
    }

    pub fn push_back(&mut self) -> &mut T {
        assert!(self.size < N);
        let index = self.size;
        self.data[index] = T::default();
        self.size += 1;
        &mut self.data[index]
    }

    pub fn push(&mut self, value: T) -> &mut T {
        assert!(self.size < N);
        let index = self.size;
        self.data[index] = value;
        self.size += 1;
        &mut self.data[index]
    }

    pub fn push_back_n(&mut self, n: usize) -> &mut [T] {
        assert!(self.size + n <= N);
        let start = self.size;
        for slot in &mut self.data[start..start + n] {
            *slot = T::default();
        }
        self.size += n;
        &mut self.data[start..self.size]
    }

    pub fn push_back_n_with(&mut self, n: usize, value: T) -> &mut [T] {
        assert!(self.size + n <= N);
        let start = self.size;
        for slot in &mut self.data[start..start + n] {
            *slot = value;
        }
        self.size += n;
        &mut self.data[start..self.size]
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        self.size -= 1;
        Some(self.data[self.size])
    }

    pub fn pop_back_n(&mut self, n: usize) {
        assert!(n <= self.size);
        self.size -= n;
    }

    pub fn remove_shuffle(&mut self, n: usize) -> T {
        assert!(n < self.size);
        let old = self.data[n];
        let last = self.size - 1;
        self.data[n] = self.data[last];
        self.size = last;
        old
    }

    pub fn remove_shuffle_void(&mut self, n: usize) {
        let _ = self.remove_shuffle(n);
    }

    pub fn data(&self) -> &[T] {
        &self.data[..self.size]
    }
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data[..self.size]
    }
    pub fn front(&self) -> Option<&T> {
        self.data().first()
    }
    pub fn back(&self) -> Option<&T> {
        self.data().last()
    }
    pub fn as_slice(&self) -> &[T] {
        self.data()
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data_mut()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data().iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data_mut().iter_mut()
    }

    pub fn reserve(&mut self, reserve_count: usize) {
        assert!(reserve_count <= N);
    }
}

impl<const N: usize, T> Deref for FixedArray<N, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data[..self.size]
    }
}

impl<const N: usize, T> DerefMut for FixedArray<N, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.data[..self.size]
    }
}

impl<const N: usize, T> std::ops::Index<usize> for FixedArray<N, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        assert!(index < self.size);
        &self.data[index]
    }
}

impl<const N: usize, T> std::ops::IndexMut<usize> for FixedArray<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.size);
        &mut self.data[index]
    }
}

impl<const N: usize, T: Default + Copy> FromIterator<T> for FixedArray<N, T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut result = Self::new();
        for value in iter {
            result.push(value);
        }
        result
    }
}
