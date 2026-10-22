//! Small M02 contracts that do not warrant a dedicated module.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};

use crate::containers::SkTArray;
use crate::memory::{sk_free, sk_malloc_throw};
use crate::rect::Rect;

pub const HEX_UPPER: [u8; 16] = *b"0123456789ABCDEF";
pub const HEX_LOWER: [u8; 16] = *b"0123456789abcdef";

pub fn sk_unaligned_load<T: Copy>(bytes: &[u8]) -> T {
    assert!(bytes.len() >= std::mem::size_of::<T>());
    unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const T) }
}

pub fn sk_unaligned_store<T: Copy>(bytes: &mut [u8], value: T) {
    assert!(bytes.len() >= std::mem::size_of::<T>());
    unsafe { std::ptr::write_unaligned(bytes.as_mut_ptr() as *mut T, value) };
}

pub fn sk_bit_cast<Dst: Copy, Src: Copy>(src: &Src) -> Dst {
    assert_eq!(std::mem::size_of::<Dst>(), std::mem::size_of::<Src>());
    unsafe { std::ptr::read_unaligned(src as *const Src as *const Dst) }
}

pub struct SkSafeRange {
    ok: bool,
}

impl Default for SkSafeRange {
    fn default() -> Self {
        Self::new()
    }
}

impl SkSafeRange {
    pub const fn new() -> Self {
        Self { ok: true }
    }
    pub fn ok(&self) -> bool {
        self.ok
    }
    pub fn check_le(&mut self, value: u64, max: u64) -> u64 {
        if value > max {
            self.ok = false;
            0
        } else {
            value
        }
    }
    pub fn check_ge(&mut self, value: i32, min: i32) -> i32 {
        if value < min {
            self.ok = false;
            min
        } else {
            value
        }
    }
}

impl std::ops::Not for SkSafeRange {
    type Output = bool;
    fn not(self) -> bool {
        self.ok
    }
}

pub enum SkAutoMallocOnShrink {
    Alloc,
    Reuse,
}

pub struct SkAutoMalloc {
    ptr: *mut u8,
    size: usize,
}

impl Default for SkAutoMalloc {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SkAutoMalloc {
    pub fn new(size: usize) -> Self {
        Self {
            ptr: if size == 0 {
                std::ptr::null_mut()
            } else {
                unsafe { sk_malloc_throw(size) as *mut u8 }
            },
            size,
        }
    }
    pub fn reset(&mut self, size: usize, shrink: SkAutoMallocOnShrink) -> *mut u8 {
        let change = size != self.size
            && (size > self.size || matches!(shrink, SkAutoMallocOnShrink::Alloc));
        if change {
            unsafe { sk_free(self.ptr as *mut _) };
            self.ptr = if size == 0 {
                std::ptr::null_mut()
            } else {
                unsafe { sk_malloc_throw(size) as *mut u8 }
            };
            self.size = size;
        }
        self.ptr
    }
    pub fn get(&self) -> *mut u8 {
        self.ptr
    }
    pub fn release(&mut self) -> *mut u8 {
        self.size = 0;
        std::mem::take(&mut self.ptr)
    }
}

impl Drop for SkAutoMalloc {
    fn drop(&mut self) {
        unsafe { sk_free(self.ptr as *mut _) };
    }
}

pub struct SkNextID;
static NEXT_ID: AtomicU32 = AtomicU32::new(1);
impl SkNextID {
    pub fn image_id() -> u32 {
        NEXT_ID.fetch_add(1, AtomicOrdering::Relaxed)
    }
}

pub fn sk_is_valid_i_rect(rect: &[i32; 4]) -> bool {
    rect[2] - rect[0] >= 0 && rect[3] - rect[1] >= 0
}

pub fn sk_is_valid_rect(rect: &Rect) -> bool {
    rect.left <= rect.right
        && rect.top <= rect.bottom
        && crate::floating_point::is_finite2(rect.right - rect.left, rect.bottom - rect.top)
}

pub fn sk_tile_mode_to_str(mode: u32) -> &'static str {
    match mode {
        0 => "Clamp",
        1 => "Repeat",
        2 => "Mirror",
        3 => "Decal",
        _ => "Unknown",
    }
}

pub fn sk_dump_tag(tag: u32) -> String {
    String::from_utf8_lossy(&[
        (tag >> 24) as u8,
        (tag >> 16) as u8,
        (tag >> 8) as u8,
        tag as u8,
    ])
    .into_owned()
}

pub struct SkAutoLocaleSetter;
impl SkAutoLocaleSetter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SkAutoLocaleSetter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::type_complexity)]
pub struct SkTDPQueue<T> {
    heap: Vec<T>,
    cmp: Box<dyn Fn(&T, &T) -> Ordering>,
    index: Option<Box<dyn Fn(&T) -> *mut i32>>,
}

impl<T> SkTDPQueue<T> {
    pub fn new<F: Fn(&T, &T) -> Ordering + 'static>(cmp: F) -> Self {
        Self {
            heap: Vec::new(),
            cmp: Box::new(cmp),
            index: None,
        }
    }
    pub fn new_with_index<F, G>(cmp: F, index: G) -> Self
    where
        F: Fn(&T, &T) -> Ordering + 'static,
        G: Fn(&T) -> *mut i32 + 'static,
    {
        Self {
            heap: Vec::new(),
            cmp: Box::new(cmp),
            index: Some(Box::new(index)),
        }
    }
    pub fn count(&self) -> usize {
        self.heap.len()
    }
    pub fn empty(&self) -> bool {
        self.heap.is_empty()
    }
    pub fn peek(&self) -> Option<&T> {
        self.heap.first()
    }
    pub fn at(&self, index: usize) -> &T {
        &self.heap[index]
    }
    fn refresh_indices(&mut self) {
        if let Some(index) = &self.index {
            for (i, value) in self.heap.iter().enumerate() {
                unsafe {
                    *index(value) = i as i32;
                }
            }
        }
    }
    pub fn insert(&mut self, value: T) {
        let mut i = self.heap.len();
        self.heap.push(value);
        while i > 0 {
            let parent = (i - 1) / 2;
            if (self.cmp)(&self.heap[i], &self.heap[parent]) == Ordering::Less {
                self.heap.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
        self.refresh_indices();
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.heap.is_empty() {
            return None;
        }
        let last = self.heap.pop().unwrap();
        let out = if self.heap.is_empty() {
            last
        } else {
            let out = std::mem::replace(&mut self.heap[0], last);
            let mut i = 0;
            loop {
                let left = i * 2 + 1;
                let right = left + 1;
                if left >= self.heap.len() {
                    break;
                }
                let child = if right < self.heap.len()
                    && (self.cmp)(&self.heap[right], &self.heap[left]) == Ordering::Less
                {
                    right
                } else {
                    left
                };
                if (self.cmp)(&self.heap[child], &self.heap[i]) == Ordering::Less {
                    self.heap.swap(i, child);
                    i = child;
                } else {
                    break;
                }
            }
            out
        };
        self.refresh_indices();
        Some(out)
    }
    pub fn remove(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let Some(position) = self.heap.iter().position(|v| v == value) else {
            return false;
        };
        self.heap.swap_remove(position);
        self.heap.sort_by(|a, b| (self.cmp)(a, b));
        self.refresh_indices();
        true
    }
    pub fn priority_did_change(&mut self) {
        self.heap.sort_by(|a, b| (self.cmp)(a, b));
        self.refresh_indices();
    }
    pub fn priority_did_change_for(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        if self.heap.iter().any(|v| v == value) {
            self.priority_did_change();
            true
        } else {
            false
        }
    }
    pub fn sort(&mut self) {
        self.heap.sort_by(|a, b| (self.cmp)(a, b));
        self.refresh_indices();
    }
}

pub struct SkTDynamicHash<K, V> {
    map: HashMap<K, V>,
}
impl<K: std::hash::Hash + Eq, V> Default for SkTDynamicHash<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: std::hash::Hash + Eq, V> SkTDynamicHash<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn count(&self) -> usize {
        self.map.len()
    }
    pub fn find(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    pub fn add(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }
    pub fn foreach<F: FnMut(&K, &V)>(&self, mut f: F) {
        for (k, v) in &self.map {
            f(k, v);
        }
    }
    pub fn reset(&mut self) {
        self.map.clear();
    }
}

pub struct SkTMultiMap<K, V> {
    map: HashMap<K, Vec<V>>,
}
impl<K: std::hash::Hash + Eq, V> Default for SkTMultiMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: std::hash::Hash + Eq, V> SkTMultiMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: K, value: V) {
        self.map.entry(key).or_default().push(value);
    }
    pub fn find(&self, key: &K) -> Option<&[V]> {
        self.map.get(key).map(Vec::as_slice)
    }
    pub fn remove(&mut self, key: &K, value: &V) -> bool
    where
        V: PartialEq,
    {
        if let Some(values) = self.map.get_mut(key) {
            if let Some(index) = values.iter().position(|v| v == value) {
                values.swap_remove(index);
                if values.is_empty() {
                    self.map.remove(key);
                }
                return true;
            }
        }
        false
    }
    pub fn count(&self) -> usize {
        self.map.values().map(Vec::len).sum()
    }
}

pub fn sk_enumerate<T>(slice: &[T]) -> impl Iterator<Item = (usize, &T)> {
    slice.iter().enumerate()
}

pub fn sk_zip2<'a, A, B>(a: &'a [A], b: &'a [B]) -> impl Iterator<Item = (&'a A, &'a B)> {
    a.iter().zip(b.iter())
}

pub fn sk_zip3<'a, A, B, C>(
    a: &'a [A],
    b: &'a [B],
    c: &'a [C],
) -> impl Iterator<Item = (&'a A, &'a B, &'a C)> {
    a.iter()
        .zip(b.iter())
        .zip(c.iter())
        .map(|((a, b), c)| (a, b, c))
}

pub struct SkSpanPriv;
impl SkSpanPriv {
    pub fn eq<T>(a: &[T], b: &[T]) -> bool {
        a.len() == b.len() && a.as_ptr() == b.as_ptr()
    }
    pub fn copy<T: Clone>(dst: &mut [T], src: &[T]) {
        dst[..src.len()].clone_from_slice(src);
    }
}

pub struct SkStringView;
impl SkStringView {
    pub fn starts_with(haystack: &str, needle: &str) -> bool {
        haystack.starts_with(needle)
    }
    pub fn ends_with(haystack: &str, needle: &str) -> bool {
        haystack.ends_with(needle)
    }
    pub fn contains(haystack: &str, needle: &str) -> bool {
        haystack.contains(needle)
    }
}

pub type SkArenaAllocList<T> = SkTArray<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_extras_safe_range_heap_and_next_id() {
        let mut range = SkSafeRange::new();
        assert_eq!(range.check_le(5, 4), 0);
        assert!(!range.ok());
        let mut q = SkTDPQueue::new(|a: &i32, b: &i32| a.cmp(b));
        q.insert(3);
        q.insert(1);
        q.insert(2);
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_ne!(SkNextID::image_id(), SkNextID::image_id());
    }
}

pub struct SkEnumerate<'a, T> {
    data: &'a [T],
    base: usize,
}

impl<'a, T> SkEnumerate<'a, T> {
    pub fn new<D: AsRef<[T]> + ?Sized>(data: &'a D) -> Self {
        Self {
            data: data.as_ref(),
            base: 0,
        }
    }
    pub fn size(&self) -> usize {
        self.data.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, &'a T)> + '_ {
        let base = self.base;
        self.data
            .iter()
            .enumerate()
            .map(move |(i, v)| (base + i, v))
    }
    pub fn first(&self, count: usize) -> Self {
        Self {
            data: &self.data[..count.min(self.data.len())],
            base: self.base,
        }
    }
    pub fn last(&self, count: usize) -> Self {
        let start = self.data.len().saturating_sub(count);
        Self {
            data: &self.data[start..],
            base: self.base + start,
        }
    }
    pub fn subspan(&self, offset: usize, count: usize) -> Self {
        let end = (offset + count).min(self.data.len());
        Self {
            data: &self.data[offset.min(self.data.len())..end],
            base: self.base + offset.min(self.data.len()),
        }
    }
}

impl<T> SkEnumerate<'static, T> {
    pub fn new_mut<'a, D: AsMut<[T]> + ?Sized>(data: &'a mut D) -> SkEnumerateMut<'a, T> {
        SkEnumerateMut::new(data)
    }
}

pub struct SkEnumerateMut<'a, T> {
    data: &'a mut [T],
    base: usize,
}

impl<'a, T> SkEnumerateMut<'a, T> {
    pub fn new<D: AsMut<[T]> + ?Sized>(data: &'a mut D) -> Self {
        Self {
            data: data.as_mut(),
            base: 0,
        }
    }
    pub fn size(&self) -> usize {
        self.data.len()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> + '_ {
        let base = self.base;
        self.data
            .iter_mut()
            .enumerate()
            .map(move |(i, v)| (base + i, v))
    }
    pub fn into_iter_mut(self) -> impl Iterator<Item = (usize, &'a mut T)> {
        let base = self.base;
        self.data
            .iter_mut()
            .enumerate()
            .map(move |(i, v)| (base + i, v))
    }
}

pub trait ZipColumn {
    type Elem;
    fn zip_slice(&self) -> &[Self::Elem];
    fn zip_len(&self) -> usize {
        self.zip_slice().len()
    }
}
impl<T> ZipColumn for &[T] {
    type Elem = T;
    fn zip_slice(&self) -> &[T] {
        self
    }
}
impl<T, const N: usize> ZipColumn for &[T; N] {
    type Elem = T;
    fn zip_slice(&self) -> &[T] {
        &self[..]
    }
}
impl<T> ZipColumn for &Vec<T> {
    type Elem = T;
    fn zip_slice(&self) -> &[T] {
        self.as_slice()
    }
}

pub trait ZipLen {
    fn zip_len(&self) -> usize;
}

pub trait ZipAt<const I: usize> {
    type Elem;
    fn zip_at(&self) -> &[Self::Elem];
}

#[derive(Clone, Copy)]
pub struct SkZip<C> {
    columns: C,
    len: usize,
}

impl<C: ZipLen> SkZip<C> {
    pub fn new(columns: C) -> Self {
        let len = columns.zip_len();
        Self { columns, len }
    }

    pub fn size(&self) -> usize {
        self.len
    }
    pub fn empty(&self) -> bool {
        self.len == 0
    }
    pub fn as_const(self) -> Self {
        self
    }
}

impl<'a, T> SkZip<(&'a [T], &'a [T])> {
    pub const fn new_const(columns: (&'a [T], &'a [T])) -> Self {
        let len = columns.0.len();
        Self { columns, len }
    }
}

impl SkZip<()> {
    pub fn new_mut<C: ZipMutLen>(mut columns: C) -> SkZipMut<C> {
        let len = columns.zip_mut_len();
        SkZipMut { columns, len }
    }
}

macro_rules! zip_impl {
    ($($C:ident : $idx:tt),+ $(,)?) => {
        impl<$($C: ZipColumn),+> ZipLen for ($($C,)+) {
            fn zip_len(&self) -> usize { self.0.zip_slice().len() }
        }
        impl<$($C: ZipColumn),+> SkZip<($($C,)+)> {
            pub fn front(&self) -> ($(&$C::Elem,)+) {
                ($(self.columns.$idx.zip_slice().first().unwrap(),)+)
            }
            pub fn back(&self) -> ($(&$C::Elem,)+) {
                ($(self.columns.$idx.zip_slice().last().unwrap(),)+)
            }
            pub fn at(&self, index: usize) -> ($(&$C::Elem,)+) {
                ($(&self.columns.$idx.zip_slice()[index],)+)
            }
            pub fn iter(&self) -> impl Iterator<Item = ($(&$C::Elem,)+)> + '_ {
                (0..self.len).map(move |i| ($(&self.columns.$idx.zip_slice()[i],)+))
            }
            pub fn enumerate(&self) -> impl Iterator<Item = (usize, ($(&$C::Elem,)+))> + '_ {
                (0..self.len).map(move |i| (i, ($(&self.columns.$idx.zip_slice()[i],)+)))
            }
            pub fn data(&self) -> ($(*const $C::Elem,)+) {
                ($(self.columns.$idx.zip_slice().as_ptr(),)+)
            }
            pub fn first(&self, count: usize) -> SkZip<($(&[$C::Elem],)+)> {
                let n = count.min(self.len);
                SkZip { columns: ($(&self.columns.$idx.zip_slice()[..n],)+), len: n }
            }
            pub fn last(&self, count: usize) -> SkZip<($(&[$C::Elem],)+)> {
                let n = count.min(self.len);
                let start = self.len - n;
                SkZip { columns: ($(&self.columns.$idx.zip_slice()[start..],)+), len: n }
            }
            pub fn subspan(&self, offset: usize, count: usize) -> SkZip<($(&[$C::Elem],)+)> {
                let start = offset.min(self.len);
                let end = (start + count).min(self.len);
                SkZip { columns: ($(&self.columns.$idx.zip_slice()[start..end],)+), len: end - start }
            }
            pub fn column<const I: usize>(&self) -> &[<($($C,)+) as ZipAt<I>>::Elem]
            where
                ($($C,)+): ZipAt<I>,
            {
                <($($C,)+) as ZipAt<I>>::zip_at(&self.columns)
            }
        }
    };
}

zip_impl!(C0:0);
zip_impl!(C0:0, C1:1);
zip_impl!(C0:0, C1:1, C2:2);
zip_impl!(C0:0, C1:1, C2:2, C3:3);
zip_impl!(C0:0, C1:1, C2:2, C3:3, C4:4);
zip_impl!(C0:0, C1:1, C2:2, C3:3, C4:4, C5:5);

macro_rules! zipat_for {
    ($($C:ident),+ ; $idx:tt ; $E:ident) => {
        impl<$($C: ZipColumn),+> ZipAt<{ $idx }> for ($($C,)+) {
            type Elem = $E::Elem;
            fn zip_at(&self) -> &[Self::Elem] { self.$idx.zip_slice() }
        }
    };
}
zipat_for!(C0; 0; C0);
zipat_for!(C0,C1; 0; C0);
zipat_for!(C0,C1; 1; C1);
zipat_for!(C0,C1,C2; 0; C0);
zipat_for!(C0,C1,C2; 1; C1);
zipat_for!(C0,C1,C2; 2; C2);
zipat_for!(C0,C1,C2,C3; 0; C0);
zipat_for!(C0,C1,C2,C3; 1; C1);
zipat_for!(C0,C1,C2,C3; 2; C2);
zipat_for!(C0,C1,C2,C3; 3; C3);
zipat_for!(C0,C1,C2,C3,C4; 0; C0);
zipat_for!(C0,C1,C2,C3,C4; 1; C1);
zipat_for!(C0,C1,C2,C3,C4; 2; C2);
zipat_for!(C0,C1,C2,C3,C4; 3; C3);
zipat_for!(C0,C1,C2,C3,C4; 4; C4);
zipat_for!(C0,C1,C2,C3,C4,C5; 0; C0);
zipat_for!(C0,C1,C2,C3,C4,C5; 1; C1);
zipat_for!(C0,C1,C2,C3,C4,C5; 2; C2);
zipat_for!(C0,C1,C2,C3,C4,C5; 3; C3);
zipat_for!(C0,C1,C2,C3,C4,C5; 4; C4);
zipat_for!(C0,C1,C2,C3,C4,C5; 5; C5);

pub fn sk_make_zip<C: ZipLen>(columns: C) -> SkZip<C> {
    SkZip::new(columns)
}

pub trait ZipMutColumn {
    type Elem;
    fn zip_mut_slice(&mut self) -> &mut [Self::Elem];
}
impl<T> ZipMutColumn for &mut [T] {
    type Elem = T;
    fn zip_mut_slice(&mut self) -> &mut [T] {
        self
    }
}
impl<T, const N: usize> ZipMutColumn for &mut [T; N] {
    type Elem = T;
    fn zip_mut_slice(&mut self) -> &mut [T] {
        &mut self[..]
    }
}
impl<T> ZipMutColumn for &mut Vec<T> {
    type Elem = T;
    fn zip_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

pub trait ZipMutLen {
    fn zip_mut_len(&mut self) -> usize;
}
impl<C0: ZipMutColumn> ZipMutLen for (C0,) {
    fn zip_mut_len(&mut self) -> usize {
        self.0.zip_mut_slice().len()
    }
}
impl<C0: ZipMutColumn, C1: ZipMutColumn> ZipMutLen for (C0, C1) {
    fn zip_mut_len(&mut self) -> usize {
        self.0.zip_mut_slice().len()
    }
}

pub struct SkZipMut<C> {
    columns: C,
    len: usize,
}

impl<'a, A, B> SkZipMut<(&'a mut [A], &'a mut [B])> {
    pub fn size(&self) -> usize {
        self.len
    }
    pub fn enumerate_mut(&mut self) -> impl Iterator<Item = (usize, (&mut A, &mut B))> + '_ {
        let a: &mut [A] = &mut *self.columns.0;
        let b: &mut [B] = &mut *self.columns.1;
        a.iter_mut().zip(b.iter_mut()).enumerate()
    }
}
