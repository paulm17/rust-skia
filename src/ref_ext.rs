//! Concrete refcount types and small ownership helpers.

use crate::ref_cnt::{RefCnt, RefCntBase, WeakRefCnt, WeakRefCntBase};

pub struct SkRefCnt {
    base: RefCntBase,
}

impl Default for SkRefCnt {
    fn default() -> Self {
        Self::new()
    }
}

impl SkRefCnt {
    pub const fn new() -> Self {
        Self {
            base: RefCntBase::new(),
        }
    }
    pub fn base(&self) -> &RefCntBase {
        &self.base
    }
    pub fn ref_(&self) {
        self.base.ref_();
    }
    pub fn unref(&self) {
        if self.base.unref() {
            unsafe {
                self.internal_dispose();
            }
        }
    }
    pub fn unique(&self) -> bool {
        self.base.unique()
    }
    pub fn get_ref_cnt(&self) -> i32 {
        self.base.ref_cnt()
    }
    unsafe fn internal_dispose(&self) {
        unsafe { drop(Box::from_raw(self as *const Self as *mut Self)) };
    }
}

impl RefCnt for SkRefCnt {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
    unsafe fn internal_dispose(&self) {
        unsafe { drop(Box::from_raw(self as *const Self as *mut Self)) };
    }
}

pub struct SkNVRefCnt {
    base: RefCntBase,
}

impl Default for SkNVRefCnt {
    fn default() -> Self {
        Self::new()
    }
}

impl SkNVRefCnt {
    pub const fn new() -> Self {
        Self {
            base: RefCntBase::new(),
        }
    }
    pub fn base(&self) -> &RefCntBase {
        &self.base
    }
    pub fn ref_(&self) {
        self.base.ref_();
    }
    pub fn unref(&self) {
        if self.base.unref() {
            unsafe {
                self.internal_dispose();
            }
        }
    }
    pub fn unique(&self) -> bool {
        self.base.unique()
    }
    pub fn get_ref_cnt(&self) -> i32 {
        self.base.ref_cnt()
    }
}

impl RefCnt for SkNVRefCnt {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
    unsafe fn internal_dispose(&self) {
        unsafe { drop(Box::from_raw(self as *const Self as *mut Self)) };
    }
}

pub struct SkWeakRefCnt {
    base: WeakRefCntBase,
}

impl Default for SkWeakRefCnt {
    fn default() -> Self {
        Self::new()
    }
}

impl SkWeakRefCnt {
    pub const fn new() -> Self {
        Self {
            base: WeakRefCntBase::new(),
        }
    }
    pub fn ref_(&self) -> bool {
        self.base.try_ref()
    }
    pub fn unref(&self) {
        WeakRefCnt::unref(self);
    }
    pub fn weak_ref(&self) {
        self.base.weak_ref();
    }
    pub fn weak_unref(&self) {
        self.base.weak_unref();
    }
}

impl WeakRefCnt for SkWeakRefCnt {
    fn weak_ref_cnt_base(&self) -> &WeakRefCntBase {
        &self.base
    }
}

pub struct SingleOwner {
    _private: (),
}
impl SingleOwner {
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

pub struct SkAnySubclass<'a, T> {
    value: &'a mut T,
}
impl<'a, T> SkAnySubclass<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        Self { value }
    }
    pub fn get(&self) -> &T {
        self.value
    }
    pub fn get_mut(&mut self) -> &mut T {
        self.value
    }
}
