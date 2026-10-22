//! Generic containers, spans, hashes, sorting, and related helpers from M02.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::sync::{Arc, Mutex, Weak};

pub struct SkSpan<'a, T> {
    data: &'a [T],
}

impl<'a, T> Clone for SkSpan<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T> Copy for SkSpan<'a, T> {}

impl<'a, T> SkSpan<'a, T> {
    pub const fn new(data: &'a [T]) -> Self {
        Self { data }
    }

    pub const fn empty() -> Self {
        Self { data: &[] }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn size_bytes(&self) -> usize {
        std::mem::size_of_val(self.data)
    }

    pub fn data(&self) -> *const T {
        self.data.as_ptr()
    }

    pub fn as_slice(&self) -> &'a [T] {
        self.data
    }

    pub fn subspan(&self, offset: usize, length: usize) -> Self {
        let end = offset.saturating_add(length).min(self.data.len());
        Self {
            data: &self.data[offset.min(end)..end],
        }
    }

    pub fn subspan_from(&self, offset: usize) -> Self {
        self.subspan(offset, self.data.len().saturating_sub(offset))
    }

    pub fn first(&self) -> Option<&'a T> {
        self.data.first()
    }

    pub fn last(&self) -> Option<&'a T> {
        self.data.last()
    }
}

impl<'a, T> PartialEq for SkSpan<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.size() == other.size() && self.data() == other.data()
    }
}
impl<'a, T> Eq for SkSpan<'a, T> {}

impl<'a, T> Deref for SkSpan<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> From<&'a [T]> for SkSpan<'a, T> {
    fn from(data: &'a [T]) -> Self {
        Self::new(data)
    }
}

pub struct SkSpanMut<'a, T> {
    data: &'a mut [T],
}

impl<'a, T> SkSpanMut<'a, T> {
    pub fn new(data: &'a mut [T]) -> Self {
        Self { data }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[T] {
        self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data
    }

    pub fn subspan(&mut self, offset: usize, length: usize) -> SkSpan<'_, T> {
        let end = offset.saturating_add(length).min(self.data.len());
        SkSpan::new(&self.data[offset.min(end)..end])
    }
}

impl<'a, T> Deref for SkSpanMut<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> DerefMut for SkSpanMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

pub fn sk_str_starts_with(haystack: &str, needle: &str) -> bool {
    haystack.starts_with(needle)
}
pub fn sk_str_ends_with(haystack: &str, needle: &str) -> bool {
    haystack.ends_with(needle)
}
pub fn sk_str_contains(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}
pub fn sk_str_starts_with_char(haystack: &str, needle: char) -> bool {
    haystack.starts_with(needle)
}
pub fn sk_str_ends_with_char(haystack: &str, needle: char) -> bool {
    haystack.ends_with(needle)
}
pub fn sk_str_contains_char(haystack: &str, needle: char) -> bool {
    haystack.contains(needle)
}

#[derive(Clone, Debug)]
pub struct SkTArray<T> {
    data: Vec<T>,
}

impl<T> Default for SkTArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SkTArray<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn with_capacity(count: usize) -> Self {
        Self {
            data: Vec::with_capacity(count),
        }
    }
    pub fn from_slice(values: &[T]) -> Self
    where
        T: Clone,
    {
        Self {
            data: values.to_vec(),
        }
    }
    pub fn from_array<A: AsRef<SkTArray<T>>>(array: &A) -> Self
    where
        T: Clone,
    {
        Self::from_slice(array.as_ref().as_slice())
    }
    pub fn move_from<A: Into<SkTArray<T>>>(array: A) -> Self {
        array.into()
    }
    pub fn move_assign<A: Into<SkTArray<T>>>(&mut self, array: A) {
        *self = array.into();
    }
    pub fn assign_from<A: AsRef<SkTArray<T>>>(&mut self, array: &A)
    where
        T: Clone,
    {
        self.data.clone_from(&array.as_ref().data);
    }
    pub fn count(&self) -> usize {
        self.data.len()
    }
    pub fn size(&self) -> usize {
        self.data.len()
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn data(&self) -> *const T {
        self.data.as_ptr()
    }
    pub fn data_mut(&mut self) -> *mut T {
        self.data.as_mut_ptr()
    }
    pub fn resize_with<F: FnMut() -> T>(&mut self, count: usize, mut f: F) {
        self.data.resize_with(count, &mut f);
    }
    pub fn front(&self) -> Option<&T> {
        self.data.first()
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.data.first_mut()
    }
    pub fn back(&self) -> Option<&T> {
        self.data.last()
    }
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }
    pub fn push_back(&mut self, value: T) {
        self.data.push(value);
    }
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }
    pub fn emplace_back(&mut self, value: T) -> &mut T {
        self.data.push(value);
        self.data.last_mut().unwrap()
    }
    pub fn push_back_n_with(&mut self, count: usize, value: T) -> &mut [T]
    where
        T: Clone,
    {
        let start = self.data.len();
        self.data.resize(start + count, value);
        &mut self.data[start..]
    }
    pub fn pop_back(&mut self) -> Option<T> {
        self.data.pop()
    }
    pub fn pop_back_n(&mut self, count: usize) {
        let new_len = self.data.len().saturating_sub(count);
        self.data.truncate(new_len);
    }
    pub fn remove_shuffle(&mut self, index: usize) -> T {
        self.data.swap_remove(index)
    }
    pub fn remove(&mut self, index: usize) {
        self.data.remove(index);
    }
    pub fn clear(&mut self) {
        self.data.clear();
    }
    pub fn reset(&mut self) {
        self.data.clear();
    }
    pub fn reset_from_slice(&mut self, values: &[T])
    where
        T: Clone,
    {
        self.data.clear();
        self.data.extend_from_slice(values);
    }
    pub fn reserve(&mut self, count: usize) {
        self.data.reserve(count);
    }
    pub fn trim_to(&mut self, count: usize) {
        if count < self.data.len() {
            self.data.truncate(count);
        }
    }
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.data, &mut other.data);
    }
    pub fn swap_self(&mut self) {}
    pub fn move_back(&mut self, other: &mut Self) {
        self.data.append(&mut other.data);
    }
    pub fn move_back_self(&mut self) {}
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }
}

impl<T> AsRef<SkTArray<T>> for SkTArray<T> {
    fn as_ref(&self) -> &SkTArray<T> {
        self
    }
}

impl<T: PartialEq> PartialEq for SkTArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl<T: Eq> Eq for SkTArray<T> {}

impl<T> Index<usize> for SkTArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T> IndexMut<usize> for SkTArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

pub type TArray<T> = SkTArray<T>;

#[derive(Clone, Debug)]
pub struct STArray<const N: usize, T> {
    data: SkTArray<T>,
}

impl<const N: usize, T> Default for STArray<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, T> STArray<N, T> {
    pub fn new() -> Self {
        Self {
            data: SkTArray::new(),
        }
    }
    pub fn with_capacity(count: usize) -> Self {
        Self {
            data: SkTArray::with_capacity(count.max(N)),
        }
    }
    pub fn from_slice(values: &[T]) -> Self
    where
        T: Clone,
    {
        Self {
            data: SkTArray::from_slice(values),
        }
    }
    pub fn from_array<A: AsRef<SkTArray<T>>>(array: &A) -> Self
    where
        T: Clone,
    {
        Self::from_slice(array.as_ref().as_slice())
    }
    pub fn move_from<A: Into<SkTArray<T>>>(array: A) -> Self {
        Self { data: array.into() }
    }
    pub fn move_assign<A: Into<SkTArray<T>>>(&mut self, array: A) {
        self.data = array.into();
    }
    pub fn assign_from<A: AsRef<SkTArray<T>>>(&mut self, array: &A)
    where
        T: Clone,
    {
        self.data.assign_from(array);
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity().max(N)
    }
    pub fn empty(&self) -> bool {
        self.data.empty()
    }
    pub fn as_tarray(&self) -> &SkTArray<T> {
        &self.data
    }
    pub fn as_tarray_mut(&mut self) -> &mut SkTArray<T> {
        &mut self.data
    }
}

impl<const N: usize, T> AsRef<SkTArray<T>> for STArray<N, T> {
    fn as_ref(&self) -> &SkTArray<T> {
        &self.data
    }
}

impl<const N: usize, T> From<STArray<N, T>> for SkTArray<T> {
    fn from(value: STArray<N, T>) -> Self {
        value.data
    }
}

impl<const N: usize, T: PartialEq> PartialEq for STArray<N, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl<const N: usize, T: Eq> Eq for STArray<N, T> {}

impl<const N: usize, T> Deref for STArray<N, T> {
    type Target = SkTArray<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<const N: usize, T> DerefMut for STArray<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkTDArray<T: Copy> {
    data: Vec<T>,
}

impl<T: Copy> SkTDArray<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn count(&self) -> usize {
        self.data.len()
    }
    pub fn empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn push_back(&mut self, value: T) {
        self.data.push(value);
    }
    pub fn append(&mut self, values: &[T]) {
        self.data.extend_from_slice(values);
    }
    pub fn pop_back(&mut self) -> Option<T> {
        self.data.pop()
    }
    pub fn remove_shuffle(&mut self, index: usize) -> T {
        self.data.swap_remove(index)
    }
    pub fn reset(&mut self) {
        self.data.clear();
    }
    pub fn reserve(&mut self, count: usize) {
        self.data.reserve(count);
    }
    pub fn data(&self) -> *const T {
        self.data.as_ptr()
    }
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }
}

impl<T: Copy> Index<usize> for SkTDArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T: Copy> IndexMut<usize> for SkTDArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

pub type TDArray<T> = SkTDArray<T>;

#[derive(Clone, Debug, Default)]
pub struct SkTDeque<T> {
    data: VecDeque<T>,
    block_capacity: usize,
}

impl<T> SkTDeque<T> {
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
            block_capacity: 1,
        }
    }
    pub fn with_capacity(block_capacity: usize) -> Self {
        Self {
            data: VecDeque::new(),
            block_capacity: block_capacity.max(1),
        }
    }
    pub fn count(&self) -> usize {
        self.data.len()
    }
    pub fn empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn elem_size(&self) -> usize {
        std::mem::size_of::<T>()
    }
    pub fn push_front(&mut self, value: T) {
        self.data.push_front(value);
    }
    pub fn push_back(&mut self, value: T) {
        self.data.push_back(value);
    }
    pub fn pop_front(&mut self) -> Option<T> {
        self.data.pop_front()
    }
    pub fn pop_back(&mut self) -> Option<T> {
        self.data.pop_back()
    }
    pub fn front(&self) -> Option<&T> {
        self.data.front()
    }
    pub fn back(&self) -> Option<&T> {
        self.data.back()
    }
    pub fn num_blocks_allocated(&self) -> usize {
        if self.data.is_empty() {
            1
        } else {
            self.data.len().div_ceil(self.block_capacity)
        }
    }
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.data.iter()
    }
    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<'_, T> {
        self.data.iter_mut()
    }
}

pub type TDeque<T> = SkTDeque<T>;

#[derive(Clone, Debug)]
struct LListNode<T> {
    value: Option<T>,
    prev: Option<usize>,
    next: Option<usize>,
    alive: bool,
}

#[derive(Clone, Debug)]
pub struct TInternalLList<T> {
    nodes: Vec<LListNode<T>>,
    head: Option<usize>,
    tail: Option<usize>,
}

impl<T> Default for TInternalLList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TInternalLList<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn head(&self) -> Option<&T> {
        self.head.and_then(|i| self.nodes[i].value.as_ref())
    }

    pub fn tail(&self) -> Option<&T> {
        self.tail.and_then(|i| self.nodes[i].value.as_ref())
    }

    pub fn head_handle(&self) -> Option<usize> {
        self.head
    }

    pub fn tail_handle(&self) -> Option<usize> {
        self.tail
    }

    pub fn count_entries(&self) -> usize {
        self.nodes.iter().filter(|n| n.alive).count()
    }

    pub fn add_to_head(&mut self, value: T) -> usize {
        let index = self.push_node(value);
        self.link_head(index);
        index
    }

    pub fn add_to_tail(&mut self, value: T) -> usize {
        let index = self.push_node(value);
        self.link_tail(index);
        index
    }

    pub fn add_after(&mut self, value: T, after: Option<usize>) -> usize {
        let Some(after) = after else {
            return self.add_to_head(value);
        };
        let index = self.push_node(value);
        let next = self.nodes[after].next;
        self.nodes[index].prev = Some(after);
        self.nodes[index].next = next;
        self.nodes[after].next = Some(index);
        if let Some(next) = next {
            self.nodes[next].prev = Some(index);
        } else {
            self.tail = Some(index);
        }
        index
    }

    pub fn add_before(&mut self, value: T, before: Option<usize>) -> usize {
        let Some(before) = before else {
            return self.add_to_tail(value);
        };
        let index = self.push_node(value);
        let prev = self.nodes[before].prev;
        self.nodes[index].next = Some(before);
        self.nodes[index].prev = prev;
        self.nodes[before].prev = Some(index);
        if let Some(prev) = prev {
            self.nodes[prev].next = Some(index);
        } else {
            self.head = Some(index);
        }
        index
    }

    fn push_node(&mut self, value: T) -> usize {
        let index = self.nodes.len();
        self.nodes.push(LListNode {
            value: Some(value),
            prev: None,
            next: None,
            alive: true,
        });
        index
    }

    fn link_head(&mut self, index: usize) {
        self.nodes[index].prev = None;
        self.nodes[index].next = self.head;
        if let Some(head) = self.head {
            self.nodes[head].prev = Some(index);
        } else {
            self.tail = Some(index);
        }
        self.head = Some(index);
    }

    fn link_tail(&mut self, index: usize) {
        self.nodes[index].next = None;
        self.nodes[index].prev = self.tail;
        if let Some(tail) = self.tail {
            self.nodes[tail].next = Some(index);
        } else {
            self.head = Some(index);
        }
        self.tail = Some(index);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.nodes.len() || !self.nodes[index].alive {
            return None;
        }
        let prev = self.nodes[index].prev;
        let next = self.nodes[index].next;
        if let Some(prev) = prev {
            self.nodes[prev].next = next;
        } else {
            self.head = next;
        }
        if let Some(next) = next {
            self.nodes[next].prev = prev;
        } else {
            self.tail = prev;
        }
        self.nodes[index].alive = false;
        self.nodes[index].prev = None;
        self.nodes[index].next = None;
        self.nodes[index].value.take()
    }

    pub fn is_in_list(&self, index: usize) -> bool {
        self.nodes.get(index).is_some_and(|n| n.alive)
    }

    pub fn concat(&mut self, other: &mut Self) {
        while let Some(head) = other.head {
            let value = other.remove(head).unwrap();
            self.add_to_tail(value);
        }
    }

    pub fn iter(&self) -> TInternalLListIter<'_, T> {
        TInternalLListIter {
            list: self,
            current: self.head,
        }
    }

    pub fn iter_rev(&self) -> TInternalLListIterRev<'_, T> {
        TInternalLListIterRev {
            list: self,
            current: self.tail,
        }
    }

    pub fn validate(&self) {
        let mut count = 0;
        let mut current = self.head;
        let mut prev = None;
        while let Some(index) = current {
            assert!(self.nodes[index].alive);
            assert_eq!(self.nodes[index].prev, prev);
            prev = Some(index);
            current = self.nodes[index].next;
            count += 1;
        }
        assert_eq!(prev, self.tail);
        assert_eq!(count, self.count_entries());
    }
}

pub struct TInternalLListIter<'a, T> {
    list: &'a TInternalLList<T>,
    current: Option<usize>,
}

impl<'a, T> Iterator for TInternalLListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current?;
        self.current = self.list.nodes[index].next;
        self.list.nodes[index].value.as_ref()
    }
}

pub struct TInternalLListIterRev<'a, T> {
    list: &'a TInternalLList<T>,
    current: Option<usize>,
}

impl<'a, T> Iterator for TInternalLListIterRev<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current?;
        self.current = self.list.nodes[index].prev;
        self.list.nodes[index].value.as_ref()
    }
}

pub struct SkTLazy<T> {
    value: Option<T>,
}

impl<T> Default for SkTLazy<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SkTLazy<T> {
    pub const fn new() -> Self {
        Self { value: None }
    }
    pub fn is_valid(&self) -> bool {
        self.value.is_some()
    }
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
    pub fn init(&mut self, value: T) -> &mut T {
        self.value = Some(value);
        self.value.as_mut().unwrap()
    }
    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }
    pub fn reset(&mut self) {
        self.value = None;
    }
}

impl<T: Clone> Clone for SkTLazy<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

pub struct SkTCopyOnFirstWrite<'a, T: Clone> {
    initial: &'a T,
    copy: Option<T>,
}

impl<'a, T: Clone> Clone for SkTCopyOnFirstWrite<'a, T> {
    fn clone(&self) -> Self {
        Self {
            initial: self.initial,
            copy: self.copy.clone(),
        }
    }
}

impl<'a, T: Clone> SkTCopyOnFirstWrite<'a, T> {
    pub fn new(initial: &'a T) -> Self {
        Self {
            initial,
            copy: None,
        }
    }
    pub fn get(&self) -> &T {
        self.copy.as_ref().unwrap_or(self.initial)
    }
    pub fn writable(&mut self) -> &mut T {
        if self.copy.is_none() {
            self.copy = Some(self.initial.clone());
        }
        self.copy.as_mut().unwrap()
    }
}

pub struct THashMap<K, V> {
    map: HashMap<K, V>,
    capacity: usize,
}

impl<K: Eq + Hash, V> Default for THashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq + Hash, V: Clone> Clone for THashMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            capacity: self.capacity,
        }
    }
}

impl<K: Eq + Hash, V> THashMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            capacity: 0,
        }
    }
    pub fn count(&self) -> usize {
        self.map.len()
    }
    pub fn empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn reserve(&mut self, count: usize) {
        let new_cap = if count == 0 {
            0
        } else {
            let mut cap = count.next_power_of_two().max(4);
            if count.saturating_mul(4) > cap.saturating_mul(3) {
                cap *= 2;
            }
            cap
        };
        self.capacity = self.capacity.max(new_cap);
        self.map.reserve(count.saturating_sub(self.map.capacity()));
    }
    pub fn set(&mut self, key: K, value: V) -> Option<V> {
        if !self.map.contains_key(&key) {
            self.grow_for_insert();
        }
        self.map.insert(key, value)
    }
    pub fn find(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn find_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }
    pub fn find_or_null(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let value = self.map.remove(key);
        if value.is_some() {
            self.shrink_for_remove();
        }
        value
    }
    pub fn reset(&mut self) {
        self.map.clear();
        self.capacity = 0;
    }
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    pub fn foreach<F: FnMut(&K, &V)>(&self, mut f: F) {
        for (k, v) in &self.map {
            f(k, v);
        }
    }
    pub fn foreach_mut<F: FnMut(&K, &mut V)>(&mut self, mut f: F) {
        for (k, v) in self.map.iter_mut() {
            f(k, v);
        }
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.map.iter()
    }
    pub fn approx_bytes_used(&self) -> usize {
        self.map.capacity() * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
    }
    fn grow_for_insert(&mut self) {
        if self.capacity == 0 {
            self.capacity = 4;
        } else if self.map.len() + 1 >= self.capacity {
            self.capacity = self.capacity.saturating_mul(2);
        }
    }
    fn shrink_for_remove(&mut self) {
        if self.capacity >= 8 && self.map.len() <= self.capacity / 4 {
            self.capacity /= 2;
        }
    }
}

pub type TMap<K, V> = THashMap<K, V>;

pub struct THashSet<T> {
    set: HashSet<T>,
    capacity: usize,
}

impl<T: Eq + Hash + Clone> Default for THashSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Eq + Hash> Clone for THashSet<T> {
    fn clone(&self) -> Self {
        Self {
            set: self.set.clone(),
            capacity: self.capacity,
        }
    }
}

impl<T: Eq + Hash + Clone> THashSet<T> {
    pub fn new() -> Self {
        Self {
            set: HashSet::new(),
            capacity: 0,
        }
    }
    pub fn count(&self) -> usize {
        self.set.len()
    }
    pub fn empty(&self) -> bool {
        self.set.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn reserve(&mut self, count: usize) {
        let new_cap = if count == 0 {
            0
        } else {
            let mut cap = count.next_power_of_two().max(4);
            if count.saturating_mul(4) > cap.saturating_mul(3) {
                cap *= 2;
            }
            cap
        };
        self.capacity = self.capacity.max(new_cap);
        self.set.reserve(count.saturating_sub(self.set.capacity()));
    }
    pub fn find(&self, value: &T) -> Option<&T> {
        self.set.get(value)
    }
    pub fn approx_bytes_used(&self) -> usize {
        (std::mem::size_of::<T>() + std::mem::size_of::<u32>()) * self.capacity
    }
    pub fn add(&mut self, value: &T) -> bool {
        let copy = value.clone();
        if self.set.contains(&copy) {
            return false;
        }
        if self.capacity == 0 {
            self.capacity = 4;
        } else if self.set.len() + 1 >= self.capacity {
            self.capacity *= 2;
        }
        self.set.insert(copy)
    }
    pub fn contains(&self, value: &T) -> bool {
        self.set.contains(value)
    }
    pub fn remove(&mut self, value: &T) -> bool {
        let removed = self.set.remove(value);
        if removed {
            if self.capacity >= 8 && self.set.len() <= self.capacity / 4 {
                self.capacity /= 2;
            }
        }
        removed
    }
    pub fn reset(&mut self) {
        self.set.clear();
        self.capacity = 0;
    }
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, T> {
        self.set.iter()
    }
}

pub type TSet<T> = THashSet<T>;

/// Minimal pointer/key table corresponding to `SkTHashTable`.
///
/// The entry type `V` is stored by value associated with an explicit lookup
/// key `K`. This mirrors the reference contract where a key is derived from an
/// entry pointer; the Rust caller supplies the key when inserting.
pub struct THashTable<V, K> {
    map: HashMap<K, V>,
}

impl<V, K: Eq + Hash> Default for THashTable<V, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V, K: Eq + Hash> THashTable<V, K> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn count(&self) -> usize {
        self.map.len()
    }
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
    pub fn empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn set(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }
    pub fn find(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn find_or_null(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }
    pub fn reserve(&mut self, count: usize) {
        self.map.reserve(count);
    }
    pub fn reset(&mut self) {
        self.map.clear();
    }
    pub fn approx_bytes_used(&self) -> usize {
        self.map.capacity() * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.map.iter()
    }
}

pub struct SkLRUCache<K, V> {
    entries: Vec<(K, V)>,
    max_count: usize,
}

impl<K: Eq, V> SkLRUCache<K, V> {
    pub fn new(max_count: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_count,
        }
    }
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    pub fn max_count(&self) -> usize {
        self.max_count
    }
    pub fn set_max_count(&mut self, max: usize) {
        self.max_count = max;
        self.trim();
    }
    pub fn find(&mut self, key: &K) -> Option<&V> {
        let index = self.entries.iter().position(|(k, _)| k == key)?;
        let entry = self.entries.remove(index);
        self.entries.insert(0, entry);
        Some(&self.entries[0].1)
    }
    pub fn insert(&mut self, key: K, value: V) {
        if let Some(index) = self.entries.iter().position(|(k, _)| *k == key) {
            self.entries.remove(index);
        }
        self.entries.insert(0, (key, value));
        self.trim();
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.entries.iter().position(|(k, _)| k == key)?;
        Some(self.entries.remove(index).1)
    }
    pub fn reset(&mut self) {
        self.entries.clear();
    }
    fn trim(&mut self) {
        while self.entries.len() > self.max_count {
            self.entries.pop();
        }
    }
}

pub fn sk_tqsort<T, F: FnMut(&T, &T) -> std::cmp::Ordering>(slice: &mut [T], mut cmp: F) {
    slice.sort_unstable_by(|a, b| cmp(a, b));
}

pub fn sk_tsearch<T, F: Fn(&T) -> std::cmp::Ordering>(slice: &[T], cmp: F) -> Result<usize, usize> {
    slice.binary_search_by(|probe| cmp(probe))
}

/// Port of `SkTHeapSort`.  Behaviour (and duplicates handling) is that of an
/// in-place heap sort; the public contract is a fully sorted slice.
pub fn sk_theapsort<T, F: FnMut(&T, &T) -> bool>(slice: &mut [T], mut less_than: F) {
    // Explicit binary heap sort keeps the algorithm visible in the port; the
    // observable result is identical to Skia's SkTHeapSort.
    for root in (0..slice.len() / 2).rev() {
        sift_down(slice, root, slice.len(), &mut less_than);
    }
    for end in (1..slice.len()).rev() {
        slice.swap(0, end);
        sift_down(slice, 0, end, &mut less_than);
    }
}

fn sift_down<T, F: FnMut(&T, &T) -> bool>(
    slice: &mut [T],
    mut root: usize,
    bottom: usize,
    less_than: &mut F,
) {
    loop {
        let child = root * 2 + 1;
        if child >= bottom {
            break;
        }
        let mut bigger = child;
        if child + 1 < bottom && less_than(&slice[child], &slice[child + 1]) {
            bigger = child + 1;
        }
        if less_than(&slice[root], &slice[bigger]) {
            slice.swap(root, bigger);
            root = bigger;
        } else {
            break;
        }
    }
}

pub struct SkBitSet {
    bits: Vec<u64>,
    count: usize,
}

impl Default for SkBitSet {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SkBitSet {
    pub fn new(count: usize) -> Self {
        Self {
            bits: vec![0; count.div_ceil(64)],
            count,
        }
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn set(&mut self, index: usize) {
        assert!(index < self.count);
        self.bits[index / 64] |= 1u64 << (index % 64);
    }
    pub fn clear(&mut self, index: usize) {
        assert!(index < self.count);
        self.bits[index / 64] &= !(1u64 << (index % 64));
    }
    pub fn test(&self, index: usize) -> bool {
        index < self.count && (self.bits[index / 64] & (1u64 << (index % 64))) != 0
    }
    pub fn find_first(&self) -> Option<usize> {
        self.bits
            .iter()
            .enumerate()
            .find_map(|(i, &word)| (word != 0).then(|| i * 64 + word.trailing_zeros() as usize))
    }
    pub fn for_each_set_index<F: FnMut(usize)>(&self, mut f: F) {
        for (i, &word) in self.bits.iter().enumerate() {
            let mut bits = word;
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                let index = i * 64 + bit;
                if index < self.count {
                    f(index);
                }
                bits &= bits - 1;
            }
        }
    }
    pub fn reset(&mut self) {
        self.bits.fill(0);
    }
    pub fn set_all(&mut self) {
        self.bits.fill(u64::MAX);
    }
    pub fn find_first_unset(&self) -> Option<usize> {
        (0..self.count).find(|&index| !self.test(index))
    }
}

impl PartialEq for SkBitSet {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.bits == other.bits
    }
}
impl Eq for SkBitSet {}

/// Thread-safe message bus with per-inbox queues.
///
/// The C++ API relies on one process-wide bus per message type. Rust translates that to an
/// explicit bus value (normally held in a `OnceLock`) so no unsafe global monomorphization is
/// required. `post_unique` mirrors `AllowCopyableMessage == false` for `sk_sp` messages by moving
/// the message into at most one inbox and returning whether it was consumed.
#[derive(Debug)]
pub struct SkMessageBus<M, ID = i32> {
    inboxes: Vec<(ID, Weak<Mutex<Vec<M>>>)>,
}

impl<M, ID> Default for SkMessageBus<M, ID> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M, ID> SkMessageBus<M, ID> {
    pub fn new() -> Self {
        Self {
            inboxes: Vec::new(),
        }
    }

    pub fn make_inbox(&mut self, unique_id: ID) -> SkMessageInbox<M, ID>
    where
        ID: Clone,
    {
        let queue = Arc::new(Mutex::new(Vec::new()));
        self.inboxes
            .push((unique_id.clone(), Arc::downgrade(&queue)));
        SkMessageInbox { unique_id, queue }
    }

    pub fn post(&mut self, message: M)
    where
        M: Clone,
    {
        self.inboxes.retain(|(_, weak)| weak.strong_count() != 0);
        for (_, weak) in &self.inboxes {
            if let Some(queue) = weak.upgrade() {
                queue.lock().unwrap().push(message.clone());
            }
        }
    }

    pub fn post_filtered<F>(&mut self, message: M, mut should_post: F)
    where
        M: Clone,
        F: FnMut(&ID) -> bool,
    {
        self.inboxes.retain(|(_, weak)| weak.strong_count() != 0);
        for (id, weak) in &self.inboxes {
            if should_post(id) {
                if let Some(queue) = weak.upgrade() {
                    queue.lock().unwrap().push(message.clone());
                }
            }
        }
    }

    /// Move a message to the first inbox accepted by `should_post`.
    pub fn post_unique<F>(&mut self, message: M, mut should_post: F) -> bool
    where
        F: FnMut(&ID) -> bool,
    {
        self.inboxes.retain(|(_, weak)| weak.strong_count() != 0);
        for (id, weak) in &self.inboxes {
            if should_post(id) {
                if let Some(queue) = weak.upgrade() {
                    queue.lock().unwrap().push(message);
                    return true;
                }
            }
        }
        false
    }

    pub fn inbox_count(&mut self) -> usize {
        self.inboxes.retain(|(_, weak)| weak.strong_count() != 0);
        self.inboxes.len()
    }
}

#[derive(Debug)]
pub struct SkMessageInbox<M, ID> {
    unique_id: ID,
    queue: Arc<Mutex<Vec<M>>>,
}

impl<M, ID> SkMessageInbox<M, ID> {
    pub fn unique_id(&self) -> &ID {
        &self.unique_id
    }

    pub fn poll(&self) -> Vec<M> {
        std::mem::take(&mut *self.queue.lock().unwrap())
    }

    pub fn count(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn receive(&self, message: M) {
        self.queue.lock().unwrap().push(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_hash_capacity_sequence() {
        let mut map = THashMap::<i32, i32>::new();
        assert_eq!(map.capacity(), 0);
        map.set(1, 1);
        assert_eq!(map.capacity(), 4);
        map.set(2, 2);
        map.set(3, 3);
        map.set(4, 4);
        assert_eq!(map.capacity(), 8);
        map.remove(&1);
        map.remove(&2);
        assert_eq!(map.capacity(), 4);
    }

    #[test]
    fn dev_hashset_capacity_bytes_and_hysteresis() {
        let mut s = THashSet::<i32>::new();
        let cap = |s: &THashSet<i32>| s.capacity();
        assert_eq!((s.count(), cap(&s)), (0, 0));
        s.add(&1);
        assert_eq!((s.count(), cap(&s)), (1, 4));
        for i in 2..=4 {
            s.add(&i);
        }
        assert_eq!((s.count(), cap(&s)), (4, 8));
        assert_eq!(s.approx_bytes_used(), (4 + 4) * 8);
        s.remove(&4);
        s.remove(&3);
        assert_eq!((s.count(), cap(&s)), (2, 4));
        s.add(&3);
        s.add(&4);
        s.reserve(40);
        assert_eq!(cap(&s), 64);
        s.reserve(49);
        assert_eq!(cap(&s), 128);
        for i in 5..=8 {
            s.add(&i);
        }
        s.remove(&4);
        assert_eq!(cap(&s), 64);
        s.remove(&5);
        assert_eq!(cap(&s), 32);
        s.remove(&6);
        assert_eq!(cap(&s), 16);
        s.remove(&7);
        assert_eq!(cap(&s), 8);
        s.remove(&8);
        assert_eq!(cap(&s), 8);
    }

    #[test]
    fn dev_internal_list_iterators_and_handles() {
        let mut list = TInternalLList::new();
        let a = list.add_to_tail(1);
        let b = list.add_to_tail(2);
        let c = list.add_to_tail(3);
        assert_eq!(list.head_handle(), Some(a));
        assert_eq!(list.tail_handle(), Some(c));
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(list.iter_rev().copied().collect::<Vec<_>>(), vec![3, 2, 1]);
        assert_eq!(list.remove(b), Some(2));
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![1, 3]);
    }

    #[test]
    fn dev_deque_configured_blocks() {
        let mut d = SkTDeque::with_capacity(1);
        for i in 0..10 {
            d.push_back(i);
        }
        assert_eq!(d.num_blocks_allocated(), 10);
        let mut d = SkTDeque::with_capacity(10);
        for i in 0..10 {
            d.push_back(i);
        }
        assert_eq!(d.num_blocks_allocated(), 1);
    }

    #[test]
    fn dev_lru_recency() {
        let mut cache = SkLRUCache::<i32, i32>::new(3);
        cache.insert(1, 1);
        cache.insert(2, 2);
        cache.insert(3, 3);
        assert_eq!(cache.find(&1), Some(&1));
        cache.insert(4, 4);
        assert_eq!(cache.count(), 3);
        assert_eq!(cache.find(&2), None);
    }

    #[test]
    fn dev_message_bus_inboxes_and_filtering() {
        let mut bus = SkMessageBus::<i32, u32>::new();
        let a = bus.make_inbox(1);
        let b = bus.make_inbox(2);
        bus.post(7);
        assert_eq!(a.poll(), vec![7]);
        assert_eq!(b.poll(), vec![7]);
        bus.post_filtered(9, |id| *id == 2);
        assert!(a.poll().is_empty());
        assert_eq!(b.poll(), vec![9]);
        let unique = bus.make_inbox(3);
        assert!(bus.post_unique(11, |id| *id == 3));
        assert_eq!(unique.poll(), vec![11]);
        assert!(!bus.post_unique(13, |id| *id == 99));
    }

    #[test]
    fn dev_deque_retains_boundary_block() {
        let mut d = SkTDeque::<i32>::new();
        assert_eq!(d.num_blocks_allocated(), 1);
        d.push_back(1);
        d.push_front(2);
        assert_eq!(d.pop_front(), Some(2));
        assert_eq!(d.pop_back(), Some(1));
        assert_eq!(d.num_blocks_allocated(), 1);
    }
}
