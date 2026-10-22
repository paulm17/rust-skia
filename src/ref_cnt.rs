//! Intrusive reference counting corresponding to `SkRefCnt.h`,
//! `SkWeakRefCnt.h`, and the owning `sk_sp` smart pointer.

use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug, Default)]
pub struct RefCntBase {
    count: AtomicI32,
}

impl RefCntBase {
    pub const fn new() -> Self {
        Self {
            count: AtomicI32::new(1),
        }
    }

    pub fn from_count(count: i32) -> Self {
        Self {
            count: AtomicI32::new(count),
        }
    }

    pub fn ref_cnt(&self) -> i32 {
        self.count.load(Ordering::Acquire)
    }

    pub fn unique(&self) -> bool {
        self.ref_cnt() == 1
    }

    pub fn ref_(&self) {
        let old = self.count.fetch_add(1, Ordering::Relaxed);
        debug_assert!(old > 0, "reference to a disposed SkRefCnt");
    }

    /// Returns true when this was the final strong reference.
    pub fn unref(&self) -> bool {
        let old = self.count.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(old > 0, "unref of a disposed SkRefCnt");
        old == 1
    }
}

/// Base for non-virtual intrusive reference counting (`SkNVRefCnt`).
pub type NVRefCntBase = RefCntBase;

/// A type that owns an intrusive strong reference count.
///
/// The default `internal_dispose` deletes a heap-allocated `Self`. This mirrors
/// the C++ requirement that ref-counted objects are not stack allocated.
pub trait RefCnt {
    fn ref_cnt_base(&self) -> &RefCntBase;

    fn ref_(&self) {
        self.ref_cnt_base().ref_();
    }

    fn unref(&self)
    where
        Self: Sized,
    {
        if self.ref_cnt_base().unref() {
            unsafe { self.internal_dispose() };
        }
    }

    fn unique(&self) -> bool {
        self.ref_cnt_base().unique()
    }

    fn get_ref_cnt(&self) -> i32 {
        self.ref_cnt_base().ref_cnt()
    }

    unsafe fn internal_dispose(&self)
    where
        Self: Sized,
    {
        unsafe {
            drop(Box::from_raw(self as *const Self as *mut Self));
        }
    }
}

/// Marker for the non-virtual reference-counted contract. It has the same
/// observable ref/unref behavior as `RefCnt`.
pub trait NVRefCnt: RefCnt {}

/// Weak intrusive counting base from `SkWeakRefCnt`.
#[derive(Debug)]
pub struct WeakRefCntBase {
    strong: RefCntBase,
    weak: AtomicI32,
}

impl Default for WeakRefCntBase {
    fn default() -> Self {
        Self::new()
    }
}

impl WeakRefCntBase {
    pub const fn new() -> Self {
        Self {
            strong: RefCntBase::new(),
            weak: AtomicI32::new(1),
        }
    }

    pub fn strong_base(&self) -> &RefCntBase {
        &self.strong
    }

    pub fn weak_cnt(&self) -> i32 {
        self.weak.load(Ordering::Acquire)
    }

    pub fn strong_cnt(&self) -> i32 {
        self.strong.ref_cnt()
    }

    pub fn weak_expired(&self) -> bool {
        self.strong.ref_cnt() == 0
    }

    pub fn try_ref(&self) -> bool {
        let mut prev = self.strong.ref_cnt();
        loop {
            if prev == 0 {
                return false;
            }
            match self.strong.count.compare_exchange_weak(
                prev,
                prev + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => prev = actual,
            }
        }
    }

    pub fn weak_ref(&self) {
        self.weak.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns true when the final weak reference was released.
    pub fn weak_unref(&self) -> bool {
        let old = self.weak.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(old > 0, "weak_unref of a disposed SkWeakRefCnt");
        old == 1
    }
}

pub trait WeakRefCnt: Sized {
    fn weak_ref_cnt_base(&self) -> &WeakRefCntBase;

    fn ref_(&self) {
        self.weak_ref_cnt_base().strong_base().ref_();
    }

    fn unref(&self) {
        let base = self.weak_ref_cnt_base();
        if base.strong_base().unref() {
            self.weak_dispose();
            if base.weak_unref() {
                unsafe { self.internal_dispose() };
            }
        }
    }

    fn unique(&self) -> bool {
        self.weak_ref_cnt_base().strong_base().unique()
    }

    fn get_ref_cnt(&self) -> i32 {
        self.weak_ref_cnt_base().strong_cnt()
    }

    fn get_weak_cnt(&self) -> i32 {
        self.weak_ref_cnt_base().weak_cnt()
    }

    fn weak_ref(&self) {
        self.weak_ref_cnt_base().weak_ref();
    }

    fn weak_unref(&self) {
        if self.weak_ref_cnt_base().weak_unref() {
            unsafe { self.internal_dispose() };
        }
    }

    fn try_ref(&self) -> bool {
        self.weak_ref_cnt_base().try_ref()
    }

    fn weak_expired(&self) -> bool {
        self.weak_ref_cnt_base().weak_expired()
    }

    fn weak_dispose(&self) {}

    unsafe fn internal_dispose(&self) {
        unsafe {
            drop(Box::from_raw(self as *const Self as *mut Self));
        }
    }
}

/// `sk_sp<T>`: an owning intrusive strong reference.
///
/// The handle itself is one pointer. A separate heap header stores the erased
/// object pointer and its concrete ref/unref operations, which permits
/// `SkSp<dyn Trait>` while keeping `size_of::<SkSp<T>>() == size_of::<*mut ()>()`.
/// Sized values additionally expose `Deref`, `get`, and `as_ptr`.
struct SkSpRepr {
    /// The concrete owner pointer used by the ref/unref operations and disposal.
    ptr: *const (),
    /// The current view pointer exposed by `get`/`as_ptr`; may be a base
    /// subobject or trait-object projection at a nonzero offset.
    view: *const (),
    ref_fn: unsafe fn(*const ()),
    unref_fn: unsafe fn(*const ()),
}

pub struct SkSp<T: ?Sized + RefCnt> {
    repr: Option<NonNull<SkSpRepr>>,
    _marker: PhantomData<*mut T>,
}

unsafe fn sp_ref<T: RefCnt>(ptr: *const ()) {
    unsafe { (&*(ptr as *const T)).ref_() };
}

unsafe fn sp_unref<T: RefCnt>(ptr: *const ()) {
    unsafe { (&*(ptr as *const T)).unref() };
}

impl<T: ?Sized + RefCnt> SkSp<T> {
    pub const fn null() -> Self {
        Self {
            repr: None,
            _marker: PhantomData,
        }
    }

    pub fn is_null(&self) -> bool {
        self.repr.is_none()
    }

    fn from_repr(
        ptr: *const (),
        ref_fn: unsafe fn(*const ()),
        unref_fn: unsafe fn(*const ()),
    ) -> Self {
        Self::from_repr_view(ptr, ptr, ref_fn, unref_fn)
    }

    fn from_repr_view(
        ptr: *const (),
        view: *const (),
        ref_fn: unsafe fn(*const ()),
        unref_fn: unsafe fn(*const ()),
    ) -> Self {
        let repr = Box::into_raw(Box::new(SkSpRepr {
            ptr,
            view,
            ref_fn,
            unref_fn,
        }));
        Self {
            repr: Some(unsafe { NonNull::new_unchecked(repr) }),
            _marker: PhantomData,
        }
    }

    fn repr_ptr(&self) -> *const () {
        self.repr
            .map_or(std::ptr::null(), |repr| unsafe { repr.as_ref().view })
    }

    pub fn from_raw_parts(
        ptr: Option<NonNull<()>>,
        ref_fn: unsafe fn(*const ()),
        unref_fn: unsafe fn(*const ()),
    ) -> Self {
        match ptr {
            Some(ptr) => Self::from_repr(ptr.as_ptr(), ref_fn, unref_fn),
            None => Self::null(),
        }
    }
}

impl<T: RefCnt> SkSp<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Self::adopt(ptr)
    }

    /// Adopt a raw reference whose initial count is already owned by the caller.
    pub fn adopt(ptr: *mut T) -> Self {
        match NonNull::new(ptr) {
            Some(ptr) => Self::from_repr(ptr.as_ptr() as *const (), sp_ref::<T>, sp_unref::<T>),
            None => Self::null(),
        }
    }

    /// Construct a strong reference by incrementing `ptr`'s reference count.
    pub fn from_ref(ptr: *mut T) -> Self {
        if let Some(non_null) = NonNull::new(ptr) {
            unsafe { non_null.as_ref().ref_() };
            Self::adopt(non_null.as_ptr())
        } else {
            Self::null()
        }
    }

    pub fn get(&self) -> Option<&T> {
        let repr = self.repr?;
        Some(unsafe { &*(repr.as_ref().view as *const T) })
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        let repr = self.repr?;
        Some(unsafe { &mut *(repr.as_ref().view as *mut T) })
    }

    pub fn as_ptr(&self) -> *const T {
        self.repr_ptr() as *const T
    }

    /// Move this owning handle to a base or trait-object view of the same
    /// object. The coercion function is checked by the compiler, so `U` must be
    /// a base trait/type implemented by `T`; no reference count is changed.
    pub fn cast<U: ?Sized + RefCnt>(self, coerce: fn(&T) -> &U) -> SkSp<U> {
        let view = match self.get() {
            Some(value) => coerce(value) as *const U as *const (),
            None => return SkSp::null(),
        };
        let repr = self.repr;
        std::mem::forget(self);
        if let Some(repr) = repr {
            unsafe { (*repr.as_ptr()).view = view };
        }
        SkSp {
            repr,
            _marker: PhantomData,
        }
    }

    /// Clone and move the new reference to a base or trait-object view.
    pub fn clone_as<U: ?Sized + RefCnt>(&self, coerce: fn(&T) -> &U) -> SkSp<U> {
        self.clone().cast(coerce)
    }

    pub fn as_erased_ptr(&self) -> *const () {
        self.repr_ptr()
    }

    fn take_raw(&mut self) -> *mut T {
        match self.repr.take() {
            Some(repr) => {
                let header = unsafe { Box::from_raw(repr.as_ptr()) };
                header.view as *mut T
            }
            None => std::ptr::null_mut(),
        }
    }

    pub fn into_raw(mut self) -> *mut T {
        self.take_raw()
    }

    /// Rust adaptation of a moved-from `sk_sp`: explicitly take the pointer.
    pub fn take(&mut self) -> *mut T {
        self.take_raw()
    }

    pub fn reset(&mut self, new_value: Option<Self>) {
        let old = std::mem::replace(self, new_value.unwrap_or_else(Self::null));
        drop(old);
    }

    pub fn reset_raw(&mut self, ptr: *mut T) {
        let old = std::mem::replace(self, Self::adopt(ptr));
        drop(old);
    }
}

impl<T: ?Sized + RefCnt> Default for SkSp<T> {
    fn default() -> Self {
        Self::null()
    }
}

impl<T: ?Sized + RefCnt> Clone for SkSp<T> {
    fn clone(&self) -> Self {
        match self.repr {
            Some(repr) => {
                let header = unsafe { repr.as_ref() };
                unsafe { (header.ref_fn)(header.ptr) };
                Self::from_repr_view(header.ptr, header.view, header.ref_fn, header.unref_fn)
            }
            None => Self::null(),
        }
    }
}

impl<T: ?Sized + RefCnt> Drop for SkSp<T> {
    fn drop(&mut self) {
        if let Some(repr) = self.repr.take() {
            let header = unsafe { Box::from_raw(repr.as_ptr()) };
            unsafe { (header.unref_fn)(header.ptr) };
        }
    }
}

impl<T: RefCnt> Deref for SkSp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get().expect("dereference of null SkSp")
    }
}

impl<T: ?Sized + RefCnt, U: ?Sized + RefCnt> PartialEq<SkSp<U>> for SkSp<T> {
    fn eq(&self, other: &SkSp<U>) -> bool {
        self.repr_ptr() == other.repr_ptr()
    }
}

impl<T: ?Sized + RefCnt> Eq for SkSp<T> {}

impl<T: ?Sized + RefCnt> PartialEq<*const T> for SkSp<T> {
    fn eq(&self, other: &*const T) -> bool {
        self.repr_ptr() == *other as *const ()
    }
}

impl<T: RefCnt + fmt::Debug> fmt::Debug for SkSp<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get() {
            Some(value) => value.fmt(f),
            None => f.write_str("null"),
        }
    }
}

unsafe impl<T: RefCnt + Send + Sync> Send for SkSp<T> {}
unsafe impl<T: RefCnt + Send + Sync> Sync for SkSp<T> {}

/// Weak counterpart to `sk_sp`.
pub struct SkWeakRef<T: WeakRefCnt> {
    ptr: Option<NonNull<T>>,
    _marker: PhantomData<T>,
}

impl<T: WeakRefCnt> SkWeakRef<T> {
    pub const fn null() -> Self {
        Self {
            ptr: None,
            _marker: PhantomData,
        }
    }

    /// Adopt a weak reference whose count is already owned by the caller.
    pub fn adopt(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new(ptr),
            _marker: PhantomData,
        }
    }

    pub fn from_ref(ptr: *mut T) -> Self {
        if let Some(non_null) = NonNull::new(ptr) {
            unsafe { non_null.as_ref().weak_ref() };
            Self {
                ptr: Some(non_null),
                _marker: PhantomData,
            }
        } else {
            Self::null()
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.ptr.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn upgrade(&self) -> Option<SkSp<T>>
    where
        T: RefCnt,
    {
        match self.get() {
            Some(value) if value.try_ref() => Some(SkSp::adopt(value as *const T as *mut T)),
            _ => None,
        }
    }

    pub fn expired(&self) -> bool {
        self.get().is_none_or(|value| value.weak_expired())
    }
}

impl<T: WeakRefCnt> Clone for SkWeakRef<T> {
    fn clone(&self) -> Self {
        if let Some(value) = self.get() {
            value.weak_ref();
        }
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: WeakRefCnt> Drop for SkWeakRef<T> {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr.take() {
            unsafe { ptr.as_ref().weak_unref() };
        }
    }
}

pub fn sk_make_sp<T: RefCnt>(value: T) -> SkSp<T> {
    SkSp::new(value)
}

/// Construct an erased owning reference, preserving the concrete intrusive
/// strong count. Used for derived-to-abstract `SkSp<dyn Trait>` ownership.
pub fn sk_make_sp_as<U: ?Sized + RefCnt, T: RefCnt + Send + Sync + 'static>(value: T) -> SkSp<U> {
    let ptr = Box::into_raw(Box::new(value)) as *mut ();
    SkSp::from_raw_parts(NonNull::new(ptr), sp_ref::<T>, sp_unref::<T>)
}

pub fn sk_ref_sp<T: RefCnt>(value: &T) -> SkSp<T> {
    SkSp::from_ref(value as *const T as *mut T)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    static DROPS: AtomicI32 = AtomicI32::new(0);

    struct Counter {
        base: RefCntBase,
        refs: AtomicI32,
        unrefs: AtomicI32,
    }

    impl Counter {
        fn new() -> Self {
            Self {
                base: RefCntBase::new(),
                refs: AtomicI32::new(0),
                unrefs: AtomicI32::new(0),
            }
        }
    }

    impl RefCnt for Counter {
        fn ref_cnt_base(&self) -> &RefCntBase {
            &self.base
        }

        fn ref_(&self) {
            self.refs.fetch_add(1, Ordering::Relaxed);
            self.base.ref_();
        }

        fn unref(&self) {
            self.unrefs.fetch_add(1, Ordering::Relaxed);
            if self.base.unref() {
                unsafe { self.internal_dispose() };
            }
        }

        unsafe fn internal_dispose(&self) {
            DROPS.fetch_add(1, Ordering::Relaxed);
            unsafe { drop(Box::from_raw(self as *const Self as *mut Self)) };
        }
    }

    #[test]
    fn dev_refcnt_adopts_clones_and_disposes_once() {
        let before = DROPS.load(Ordering::Relaxed);
        let sp = SkSp::new(Counter::new());
        let clone = sp.clone();
        assert_eq!(sp.get_ref_cnt(), 2);
        assert!(!sp.unique());
        drop(clone);
        assert!(sp.unique());
        assert_eq!(sp.get().unwrap().refs.load(Ordering::Relaxed), 1);
        assert_eq!(sp.get().unwrap().unrefs.load(Ordering::Relaxed), 1);
        drop(sp);
        assert_eq!(DROPS.load(Ordering::Relaxed) - before, 1);
    }

    #[test]
    fn dev_sk_sp_is_one_pointer_and_erases_unsized() {
        assert_eq!(
            std::mem::size_of::<SkSp<Counter>>(),
            std::mem::size_of::<*mut ()>()
        );

        trait Abstract: RefCnt {}
        struct Concrete {
            base: RefCntBase,
            drops: Arc<AtomicI32>,
        }
        impl RefCnt for Concrete {
            fn ref_cnt_base(&self) -> &RefCntBase {
                &self.base
            }
        }
        impl Abstract for Concrete {}
        impl Drop for Concrete {
            fn drop(&mut self) {
                self.drops.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(AtomicI32::new(0));
        let sp: SkSp<dyn Abstract> = sk_make_sp_as(Concrete {
            base: RefCntBase::new(),
            drops: drops.clone(),
        });
        assert_eq!(std::mem::size_of_val(&sp), std::mem::size_of::<*mut ()>());
        let clone = sp.clone();
        drop(clone);
        drop(sp);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn dev_sk_sp_casts_existing_handle_without_ref_changes() {
        trait Base: RefCnt {}
        struct Derived {
            base: RefCntBase,
        }
        impl RefCnt for Derived {
            fn ref_cnt_base(&self) -> &RefCntBase {
                &self.base
            }
        }
        impl Base for Derived {}

        let sp = sk_make_sp(Derived {
            base: RefCntBase::new(),
        });
        assert_eq!(sp.get_ref_cnt(), 1);
        let cloned = sp.clone_as(|value| value as &dyn Base);
        let moved = sp.cast(|value| value as &dyn Base);
        assert_eq!(
            std::mem::size_of_val(&cloned),
            std::mem::size_of::<*mut ()>()
        );
        assert_eq!(
            std::mem::size_of_val(&moved),
            std::mem::size_of::<*mut ()>()
        );
        drop(cloned);
        drop(moved);
    }

    #[test]
    fn dev_sk_sp_null_and_take() {
        let mut sp: SkSp<Counter> = SkSp::null();
        assert!(sp.is_null());
        assert_eq!(sp.take(), std::ptr::null_mut());
        let mut sp = sk_make_sp(Counter::new());
        let raw = sp.take();
        assert!(sp.is_null());
        assert!(!raw.is_null());
        let adopted = SkSp::adopt(raw);
        drop(adopted);
    }
}
