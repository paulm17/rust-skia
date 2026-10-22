//! Pointer-stable block list corresponding to `src/core/SkTBlockList.h`.

use std::ops::Index;

pub struct TBlockList<T, const STARTING_ITEMS: usize = 1> {
    items: Vec<Box<T>>,
    reserved: usize,
    block_size: usize,
    scratch_bytes: usize,
}

impl<T, const STARTING_ITEMS: usize> Default for TBlockList<T, STARTING_ITEMS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const STARTING_ITEMS: usize> TBlockList<T, STARTING_ITEMS> {
    pub fn new() -> Self {
        let block_size = STARTING_ITEMS.max(1);
        Self {
            items: Vec::new(),
            reserved: block_size,
            block_size,
            scratch_bytes: Self::rounded_scratch(block_size),
        }
    }

    /// Runtime block-size constructor corresponding to `SkTBlockList<T>(blockSize)`.
    pub fn with_block_size(block_size: usize) -> Self {
        let block_size = block_size.max(1);
        Self {
            items: Vec::new(),
            reserved: block_size,
            block_size,
            scratch_bytes: Self::rounded_scratch(block_size),
        }
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn total_size(&self) -> usize {
        self.reserved * std::mem::size_of::<T>() + std::mem::size_of::<Self>()
    }

    pub fn scratch_block_size(&self) -> usize {
        self.scratch_bytes
    }

    fn rounded_scratch(items: usize) -> usize {
        let bytes = items.saturating_mul(std::mem::size_of::<T>());
        let rounded = bytes.div_ceil(16) * 16;
        let clamped = rounded.min(65535 * 16);
        if clamped > 32 * 1024 {
            clamped.div_ceil(4096) * 4096
        } else {
            clamped
        }
    }

    fn note_allocation(&mut self, items: usize) {
        self.scratch_bytes = self.scratch_bytes.max(Self::rounded_scratch(items));
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn back(&self) -> Option<&T> {
        self.items.last().map(|x| x.as_ref())
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut().map(|x| x.as_mut())
    }

    pub fn push_back(&mut self, value: T) -> &T {
        if self.items.len() >= self.reserved {
            self.reserved += self.block_size.max(1);
        }
        self.items.push(Box::new(value));
        self.items.last().unwrap().as_ref()
    }

    pub fn emplace_back<F: FnOnce() -> T>(&mut self, f: F) -> &T {
        self.push_back(f())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.items.pop().map(|b| *b)
    }

    pub fn item(&self, index: usize) -> Option<&T> {
        self.items.get(index).map(|x| x.as_ref())
    }

    pub fn item_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).map(|x| x.as_mut())
    }

    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|x| x.as_ref())
    }

    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().map(|x| x.as_mut())
    }

    pub fn ritems(&self) -> impl Iterator<Item = &T> {
        self.items.iter().rev().map(|x| x.as_ref())
    }

    pub fn ritems_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().rev().map(|x| x.as_mut())
    }

    /// Append a default-constructed value and return a mutable reference.
    pub fn push_back_default(&mut self) -> &mut T
    where
        T: Default,
    {
        self.push_back(T::default());
        self.items.last_mut().unwrap().as_mut()
    }

    /// Append a value and return a mutable reference.
    pub fn push_back_mut(&mut self, value: T) -> &mut T {
        self.items.push(Box::new(value));
        self.items.last_mut().unwrap().as_mut()
    }

    pub fn reserve(&mut self, count: usize) {
        let available = self.reserved.saturating_sub(self.items.len());
        if count > available {
            self.reserved += count;
        }
        self.note_allocation(count.max(self.block_size));
        self.items.reserve(count.saturating_sub(self.items.len()));
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.reserved = self.block_size;
        self.scratch_bytes = Self::rounded_scratch(self.block_size);
    }

    pub fn concat<const M: usize>(&mut self, other: &mut TBlockList<T, M>) {
        for item in other.items.drain(..) {
            self.items.push(item);
        }
        other.reserved = other.block_size;
    }
}

impl<T, const STARTING_ITEMS: usize> Index<usize> for TBlockList<T, STARTING_ITEMS> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.items[index].as_ref()
    }
}

impl<T, const STARTING_ITEMS: usize> IntoIterator for TBlockList<T, STARTING_ITEMS> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.items
            .into_iter()
            .map(|b| *b)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_tblocklist_pointer_stability_and_concat() {
        let mut a = TBlockList::<i32, 4>::new();
        let p = a.push_back(1) as *const i32;
        a.push_back(2);
        assert_eq!(unsafe { *p }, 1);
        let mut b = TBlockList::<i32, 4>::new();
        b.push_back(3);
        a.concat(&mut b);
        assert_eq!(a.count(), 3);
        assert!(b.empty());
        assert_eq!(unsafe { *p }, 1);
    }

    #[test]
    fn dev_tblocklist_reserve_accounting() {
        let block = 5usize;
        let mut list = TBlockList::<i32>::with_block_size(block);
        let initial = list.total_size();
        for i in 0..block {
            list.push_back(i as i32);
        }
        assert_eq!(list.total_size(), initial);
        list.reserve(2 * block);
        let reserved = list.total_size();
        assert!(reserved >= initial + 2 * block * std::mem::size_of::<i32>());
        for i in 0..2 * block {
            list.push_back(i as i32);
        }
        assert_eq!(list.total_size(), reserved);
        for i in 0..block {
            list.push_back(i as i32);
        }
        list.reserve(2 * block);
        assert!(list.scratch_block_size() >= (block + block) * std::mem::size_of::<i32>());
    }

    #[test]
    fn dev_tblocklist_runtime_block_size_and_iterators() {
        let mut list = TBlockList::<i32>::with_block_size(5);
        assert_eq!(list.block_size(), 5);
        list.push_back(1);
        *list.push_back_default() = 2;
        list.push_back_mut(3);
        let seen: Vec<_> = list.items_mut().map(|v| *v).collect();
        assert_eq!(seen, vec![1, 2, 3]);
        let rev: Vec<_> = list.ritems().copied().collect();
        assert_eq!(rev, vec![3, 2, 1]);
        assert!(list.scratch_block_size() >= 5 * std::mem::size_of::<i32>());
    }
}
