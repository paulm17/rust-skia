//! Allocation, container allocation, safety helpers, and the no-op `raw_ptr`
//! implementation.

use std::alloc::{Layout, alloc, dealloc, realloc};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub const SK_MALLOC_ZERO_INITIALIZE: u32 = 1 << 0;
pub const SK_MALLOC_THROW: u32 = 1 << 1;

pub fn sk_malloc_good_size(size: usize) -> usize {
    if size == 0 { 1 } else { (size + 15) & !15 }
}

pub unsafe fn sk_malloc_flags(size: usize, flags: u32) -> *mut libc::c_void {
    let size = sk_malloc_good_size(size);
    let ptr = if flags & SK_MALLOC_ZERO_INITIALIZE != 0 {
        unsafe { libc::calloc(1, size) }
    } else {
        unsafe { libc::malloc(size) }
    };
    if ptr.is_null() && flags & SK_MALLOC_THROW != 0 {
        std::alloc::handle_alloc_error(Layout::from_size_align(size, 16).unwrap());
    }
    ptr
}

pub unsafe fn sk_malloc_throw(size: usize) -> *mut libc::c_void {
    unsafe { sk_malloc_flags(size, SK_MALLOC_THROW) }
}

pub unsafe fn sk_malloc_canfail(size: usize) -> *mut libc::c_void {
    unsafe { sk_malloc_flags(size, 0) }
}

pub unsafe fn sk_calloc_throw(num: usize, elem_size: usize) -> *mut libc::c_void {
    let size = match num.checked_mul(elem_size) {
        Some(size) => size,
        None => std::alloc::handle_alloc_error(Layout::from_size_align(1, 1).unwrap()),
    };
    unsafe { sk_malloc_flags(size, SK_MALLOC_THROW | SK_MALLOC_ZERO_INITIALIZE) }
}

pub unsafe fn sk_realloc_throw(ptr: *mut libc::c_void, size: usize) -> *mut libc::c_void {
    if ptr.is_null() {
        return unsafe { sk_malloc_throw(size) };
    }
    if size == 0 {
        unsafe { libc::free(ptr) };
        return std::ptr::null_mut();
    }
    let new_ptr = unsafe { libc::realloc(ptr, sk_malloc_good_size(size)) };
    if new_ptr.is_null() {
        std::alloc::handle_alloc_error(Layout::from_size_align(size, 16).unwrap());
    }
    new_ptr
}

pub unsafe fn sk_free(ptr: *mut libc::c_void) {
    if !ptr.is_null() {
        unsafe { libc::free(ptr) };
    }
}

/// Rust-side allocation helper used by containers. The returned pointer is
/// allocated with the global allocator and must be released with `dealloc`.
pub fn sk_allocate_bytes(size: usize) -> (*mut u8, usize) {
    if size == 0 {
        return (std::ptr::null_mut(), 0);
    }
    let adjusted = sk_malloc_good_size(size);
    let layout = Layout::from_size_align(adjusted, 16).expect("valid allocation layout");
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    (ptr, adjusted)
}

pub unsafe fn sk_deallocate_bytes(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size != 0 {
        let adjusted = sk_malloc_good_size(size);
        let layout = Layout::from_size_align(adjusted, 16).expect("valid allocation layout");
        unsafe { dealloc(ptr, layout) };
    }
}

pub unsafe fn sk_reallocate_bytes(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if ptr.is_null() {
        return sk_allocate_bytes(new_size).0;
    }
    if new_size == 0 {
        unsafe { sk_deallocate_bytes(ptr, old_size) };
        return std::ptr::null_mut();
    }
    let old = sk_malloc_good_size(old_size);
    let new = sk_malloc_good_size(new_size);
    let layout = Layout::from_size_align(old, 16).expect("valid allocation layout");
    let new_ptr = unsafe { realloc(ptr, layout, new) };
    if new_ptr.is_null() {
        std::alloc::handle_alloc_error(Layout::from_size_align(new, 16).unwrap());
    }
    new_ptr
}

pub const K_CAPACITY_MULTIPLE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct SkContainerAllocator {
    size_of_t: usize,
    max_capacity: usize,
}

impl SkContainerAllocator {
    pub const fn new(size_of_t: usize, max_capacity: usize) -> Self {
        Self {
            size_of_t,
            max_capacity,
        }
    }

    pub fn allocate(&self, capacity: usize, growth_factor: f64) -> (*mut u8, usize) {
        assert!(capacity <= self.max_capacity);
        let capacity = if growth_factor > 1.0 && capacity > 0 {
            self.growth_factor_capacity(capacity, growth_factor)
        } else {
            capacity
        };
        sk_allocate_bytes(capacity.saturating_mul(self.size_of_t))
    }

    pub fn round_up_capacity(&self, capacity: i64) -> usize {
        assert!(capacity >= 0);
        let capacity = capacity as usize;
        if capacity < self.max_capacity.saturating_sub(K_CAPACITY_MULTIPLE) {
            (capacity + K_CAPACITY_MULTIPLE - 1) & !(K_CAPACITY_MULTIPLE - 1)
        } else {
            self.max_capacity
        }
    }

    pub fn growth_factor_capacity(&self, capacity: usize, growth_factor: f64) -> usize {
        assert!(growth_factor >= 1.0);
        let grown = (capacity as f64 * growth_factor) as i64;
        self.round_up_capacity(grown)
    }
}

/// No-op `raw_ptr` with the same pointer operations used by M02.
pub struct RawPtr<T> {
    ptr: Option<NonNull<T>>,
    _marker: PhantomData<*mut T>,
}

impl<T> Default for RawPtr<T> {
    fn default() -> Self {
        Self {
            ptr: None,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for RawPtr<T> {}

impl<T> std::fmt::Debug for RawPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RawPtr").field(&self.get()).finish()
    }
}

impl<T> RawPtr<T> {
    pub const fn null() -> Self {
        Self {
            ptr: None,
            _marker: PhantomData,
        }
    }

    pub fn new(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new(ptr),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> *mut T {
        self.ptr.map_or(std::ptr::null_mut(), |p| p.as_ptr())
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_none()
    }

    pub fn as_ref(&self) -> Option<&T> {
        self.ptr.map(|p| unsafe { p.as_ref() })
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.ptr.map(|mut p| unsafe { p.as_mut() })
    }

    /// Explicit Rust replacement for C++ raw_ptr's "zero on move".
    pub fn take(&mut self) -> *mut T {
        let ptr = self.ptr.take();
        ptr.map_or(std::ptr::null_mut(), |p| p.as_ptr())
    }

    pub fn replace(&mut self, ptr: *mut T) -> *mut T {
        let old = self.get();
        self.ptr = NonNull::new(ptr);
        old
    }

    pub fn offset(&self, count: isize) -> *mut T {
        if self.ptr.is_some() {
            unsafe { self.get().offset(count) }
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn add(&self, count: usize) -> Self {
        Self::new(if self.ptr.is_some() {
            unsafe { self.get().add(count) }
        } else {
            std::ptr::null_mut()
        })
    }

    pub fn sub(&self, count: usize) -> Self {
        Self::new(if self.ptr.is_some() {
            unsafe { self.get().sub(count) }
        } else {
            std::ptr::null_mut()
        })
    }

    pub fn cast<U>(self) -> RawPtr<U> {
        RawPtr::new(self.get().cast::<U>())
    }

    pub fn map<U, F: FnOnce(*mut T) -> *mut U>(self, f: F) -> RawPtr<U> {
        RawPtr::new(f(self.get()))
    }

    pub fn to_address(self) -> *mut T {
        self.get()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.get()
    }
}

impl<T> From<*mut T> for RawPtr<T> {
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

impl<T> From<*const T> for RawPtr<T> {
    fn from(ptr: *const T) -> Self {
        Self::new(ptr as *mut T)
    }
}

impl<T> PartialEq for RawPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
impl<T> Eq for RawPtr<T> {}

impl<T> Ord for RawPtr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.cmp(&other.ptr)
    }
}

impl<T> std::hash::Hash for RawPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl<T> PartialEq<*mut T> for RawPtr<T> {
    fn eq(&self, other: &*mut T) -> bool {
        self.get() == *other
    }
}

impl<T> PartialEq<RawPtr<T>> for *mut T {
    fn eq(&self, other: &RawPtr<T>) -> bool {
        *self == other.get()
    }
}

impl<T> PartialEq<*const T> for RawPtr<T> {
    fn eq(&self, other: &*const T) -> bool {
        self.get() as *const T == *other
    }
}

impl<T> PartialEq<RawPtr<T>> for *const T {
    fn eq(&self, other: &RawPtr<T>) -> bool {
        *self == other.get() as *const T
    }
}

impl<T> PartialOrd for RawPtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialOrd<*mut T> for RawPtr<T> {
    fn partial_cmp(&self, other: &*mut T) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other)
    }
}

impl<T> PartialOrd<*const T> for RawPtr<T> {
    fn partial_cmp(&self, other: &*const T) -> Option<std::cmp::Ordering> {
        (self.get() as *const T).partial_cmp(other)
    }
}

impl<T> std::ops::Add<usize> for RawPtr<T> {
    type Output = RawPtr<T>;
    fn add(self, rhs: usize) -> Self::Output {
        RawPtr::add(&self, rhs)
    }
}

impl<T> std::ops::Sub<usize> for RawPtr<T> {
    type Output = RawPtr<T>;
    fn sub(self, rhs: usize) -> Self::Output {
        RawPtr::sub(&self, rhs)
    }
}

impl<T> std::ops::AddAssign<usize> for RawPtr<T> {
    fn add_assign(&mut self, rhs: usize) {
        *self = RawPtr::add(self, rhs);
    }
}

impl<T> std::ops::SubAssign<usize> for RawPtr<T> {
    fn sub_assign(&mut self, rhs: usize) {
        *self = RawPtr::sub(self, rhs);
    }
}

impl<T> std::ops::Sub for RawPtr<T> {
    type Output = isize;
    fn sub(self, rhs: Self) -> isize {
        unsafe { self.get().offset_from(rhs.get()) }
    }
}

impl<T> From<RawPtr<T>> for *mut T {
    fn from(value: RawPtr<T>) -> Self {
        value.get()
    }
}

impl<T> Deref for RawPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.as_ref().expect("dereference of null raw_ptr")
    }
}

impl<T> DerefMut for RawPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut().expect("dereference of null raw_ptr")
    }
}

impl<T> std::ops::Not for RawPtr<T> {
    type Output = bool;
    fn not(self) -> bool {
        self.is_null()
    }
}

pub struct RawRef<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<*mut T>,
}

impl<T> RawRef<T> {
    pub fn new(value: &T) -> Self {
        Self {
            ptr: NonNull::from(value),
            _marker: PhantomData,
        }
    }

    pub fn from_ptr(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("raw_ref cannot be null"),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Clone for RawRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for RawRef<T> {}

impl<T> Deref for RawRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

pub trait DisableDanglingPtrDetection {}

pub const SK_USE_PARTITION_ALLOC: bool = false;
pub const ENABLE_DANGLING_RAW_PTR_CHECKS: bool = false;

pub fn sk_asan_poison_memory_region(_addr: *const u8, _size: usize) {}
pub fn sk_asan_unpoison_memory_region(_addr: *mut u8, _size: usize) {}
pub fn sk_msan_poison(_addr: *const u8, _size: usize) {}
pub fn sk_msan_unpoison(_addr: *const u8, _size: usize) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_malloc_and_calloc_contract() {
        unsafe {
            let p = sk_calloc_throw(100, 1) as *mut u8;
            for i in 0..100 {
                assert_eq!(*p.add(i), 0);
            }
            sk_free(p as *mut _);

            let p = sk_malloc_throw(10) as *mut u8;
            for i in 0..10 {
                *p.add(i) = i as u8;
            }
            let p = sk_realloc_throw(p as *mut _, 5) as *mut u8;
            for i in 0..5 {
                assert_eq!(*p.add(i), i as u8);
            }
            let p = sk_realloc_throw(p as *mut _, 0) as *mut u8;
            assert!(p.is_null());
            sk_free(std::ptr::null_mut());
        }
    }

    #[test]
    fn dev_raw_ptr_arithmetic_and_comparisons() {
        let mut arr = [10, 20, 30];
        let mut p = RawPtr::new(arr.as_mut_ptr());
        assert_eq!(&*p, &10);
        p += 1;
        assert_eq!(&*p, &20);
        assert_eq!(
            p + 1,
            RawPtr::new(unsafe { arr.as_ptr().add(2) } as *mut i32)
        );
        assert!(p < RawPtr::new(unsafe { arr.as_ptr().add(2) } as *mut i32));
        assert!(!RawPtr::<i32>::null());
        let _ = p.take();
        assert!(p.is_null());
    }

    #[test]
    fn dev_container_capacity_rounding() {
        let a = SkContainerAllocator::new(4, i32::MAX as usize);
        assert_eq!(a.round_up_capacity(1), 8);
        assert_eq!(a.growth_factor_capacity(1, 1.5), 8);
        let max = SkContainerAllocator::new(4, 100);
        assert_eq!(max.round_up_capacity(100), 100);
        assert_eq!(max.round_up_capacity(i32::MAX as i64), 100);
    }
}
