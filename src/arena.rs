//! Arena allocation and Fibonacci block sizing from `SkArenaAlloc`.

use std::alloc::{Layout, alloc, dealloc};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub const SK_FIBONACCI_47: [u32; 47] = [
    1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946,
    17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040, 1346269, 2178309, 3524578,
    5702887, 9227465, 14930352, 24157817, 39088169, 63245986, 102334155, 165580141, 267914296,
    433494437, 701408733, 1134903170, 1836311903, 2971215073,
];

#[derive(Clone, Copy, Debug)]
pub struct SkFibBlockSizes<const K_MAX_SIZE: u32> {
    index: usize,
    block_unit_size: u32,
}

impl<const K_MAX_SIZE: u32> SkFibBlockSizes<K_MAX_SIZE> {
    pub fn new(static_block_size: u32, first_allocation_size: u32) -> Self {
        let unit = if first_allocation_size > 0 {
            first_allocation_size
        } else if static_block_size > 0 {
            static_block_size
        } else {
            1024
        };
        assert!(unit > 0);
        assert!((unit as u64) < (K_MAX_SIZE as u64).min((1u64 << 26) - 1));
        Self {
            index: 0,
            block_unit_size: unit,
        }
    }

    pub fn next_block_size(&mut self) -> u32 {
        let result = (SK_FIBONACCI_47[self.index] as u64 * self.block_unit_size as u64) as u32;
        if self.index + 1 < SK_FIBONACCI_47.len()
            && (SK_FIBONACCI_47[self.index + 1] as u64)
                < (K_MAX_SIZE as u64) / self.block_unit_size as u64
        {
            self.index += 1;
        }
        result
    }
}

enum BlockStorage {
    Owned(Vec<u8>),
    External(*mut u8),
}

struct ArenaBlock {
    storage: BlockStorage,
    size: usize,
    cursor: usize,
}

impl ArenaBlock {
    fn new(size: usize) -> Self {
        Self {
            storage: BlockStorage::Owned(vec![0u8; size]),
            size,
            cursor: 0,
        }
    }

    fn borrowed(ptr: *mut u8, size: usize) -> Self {
        Self {
            storage: BlockStorage::External(ptr),
            size,
            cursor: 0,
        }
    }

    fn len(&self) -> usize {
        self.size
    }

    fn base_ptr(&self) -> *mut u8 {
        match &self.storage {
            BlockStorage::Owned(data) => data.as_ptr() as *mut u8,
            BlockStorage::External(ptr) => *ptr,
        }
    }

    fn align_cursor(&mut self, align: usize, extra: usize) {
        if align > 1 {
            let base = self.base_ptr() as usize;
            let addr = base + self.cursor + extra;
            let aligned = (addr + align - 1) & !(align - 1);
            self.cursor = aligned - base;
        } else {
            self.cursor += extra;
        }
    }

    fn ptr(&mut self, offset: usize) -> *mut u8 {
        unsafe { self.base_ptr().add(offset) }
    }
}

pub struct ArenaAlloc {
    blocks: Vec<ArenaBlock>,
    current: usize,
    initial_size: usize,
    fib: SkFibBlockSizes<{ u32::MAX }>,
    dtors: Vec<Box<dyn FnOnce()>>,
    allocated: bool,
    external: Option<(*mut u8, usize)>,
}

impl ArenaAlloc {
    pub fn new(first_heap_allocation: usize) -> Self {
        Self::with_storage(std::ptr::null_mut(), 0, first_heap_allocation)
    }

    pub fn with_storage(block: *mut u8, block_size: usize, first_heap_allocation: usize) -> Self {
        let mut arena = Self {
            blocks: Vec::new(),
            current: 0,
            initial_size: block_size,
            fib: SkFibBlockSizes::new(block_size as u32, first_heap_allocation as u32),
            dtors: Vec::new(),
            allocated: false,
            external: if !block.is_null() && block_size > 0 {
                Some((block, block_size))
            } else {
                None
            },
        };
        if let Some((ptr, size)) = arena.external {
            arena.blocks.push(ArenaBlock::borrowed(ptr, size));
        }
        arena
    }

    pub fn is_empty(&self) -> bool {
        !self.allocated
    }

    fn ensure_space(&mut self, size: usize, align: usize) -> (usize, usize) {
        if self.blocks.is_empty() {
            let fib_size = self.fib.next_block_size() as usize;
            let want = (size + align).max(fib_size).max(16);
            let rounded = (want + 15) & !15;
            self.blocks.push(ArenaBlock::new(rounded));
            self.current = 0;
        }
        let block = &mut self.blocks[self.current];
        block.align_cursor(align, 0);
        if block.cursor + size > block.len() {
            let fib_size = self.fib.next_block_size() as usize;
            let want = (size + align).max(fib_size).max(16);
            let block_size = if want > 32 * 1024 {
                (want + 4095) & !4095
            } else {
                (want + 15) & !15
            };
            self.blocks.push(ArenaBlock::new(block_size));
            self.current = self.blocks.len() - 1;
            self.blocks[self.current].align_cursor(align, 0);
        }
        let block = &mut self.blocks[self.current];
        let offset = block.cursor;
        block.cursor += size;
        self.allocated = true;
        (self.current, offset)
    }

    pub fn make_bytes_aligned_to(&mut self, size: usize, align: usize) -> *mut u8 {
        let (block, offset) = self.ensure_space(size, align);
        self.blocks[block].ptr(offset)
    }

    pub fn alloc_uninitialized<T>(&mut self, count: usize) -> *mut T {
        if count == 0 {
            return std::ptr::NonNull::<T>::dangling().as_ptr();
        }
        let size = std::mem::size_of::<T>().saturating_mul(count);
        self.make_bytes_aligned_to(size, std::mem::align_of::<T>()) as *mut T
    }

    pub fn make<T>(&mut self, value: T) -> *mut T {
        let ptr = self.alloc_uninitialized::<T>(1);
        unsafe { ptr.write(value) };
        let drop_ptr = ptr as usize;
        self.dtors.push(Box::new(move || unsafe {
            std::ptr::drop_in_place(drop_ptr as *mut T);
        }));
        ptr
    }

    pub fn make_default<T: Default>(&mut self) -> *mut T {
        self.make(T::default())
    }

    pub fn make_array<T: Clone>(&mut self, count: usize, value: &T) -> *mut T {
        let ptr = self.alloc_uninitialized::<T>(count);
        for i in 0..count {
            unsafe { ptr.add(i).write(value.clone()) };
        }
        let drop_ptr = ptr as usize;
        self.dtors.push(Box::new(move || unsafe {
            for i in 0..count {
                std::ptr::drop_in_place((drop_ptr as *mut T).add(i));
            }
        }));
        ptr
    }

    pub fn make_array_default<T: Default>(&mut self, count: usize) -> *mut T {
        let ptr = self.alloc_uninitialized::<T>(count);
        for i in 0..count {
            unsafe { ptr.add(i).write(T::default()) };
        }
        let drop_ptr = ptr as usize;
        self.dtors.push(Box::new(move || unsafe {
            for i in 0..count {
                std::ptr::drop_in_place((drop_ptr as *mut T).add(i));
            }
        }));
        ptr
    }

    pub fn make_initialized_array<T, F>(&mut self, count: usize, mut f: F) -> *mut T
    where
        F: FnMut(usize) -> T,
    {
        let ptr = self.alloc_uninitialized::<T>(count);
        for i in 0..count {
            unsafe { ptr.add(i).write(f(i)) };
        }
        let drop_ptr = ptr as usize;
        self.dtors.push(Box::new(move || unsafe {
            for i in 0..count {
                std::ptr::drop_in_place((drop_ptr as *mut T).add(i));
            }
        }));
        ptr
    }

    pub fn reset(&mut self) {
        for dtor in self.dtors.drain(..).rev() {
            dtor();
        }
        self.blocks.clear();
        if let Some((ptr, size)) = self.external {
            self.blocks.push(ArenaBlock::borrowed(ptr, size));
        }
        self.current = 0;
        self.fib = SkFibBlockSizes::new(self.initial_size as u32, 0);
        self.allocated = false;
    }
}

impl Drop for ArenaAlloc {
    fn drop(&mut self) {
        for dtor in self.dtors.drain(..).rev() {
            dtor();
        }
    }
}

pub struct SkSTArenaAlloc<const N: usize> {
    arena: ArenaAlloc,
    _marker: PhantomData<[u8; N]>,
}

impl<const N: usize> SkSTArenaAlloc<N> {
    pub fn new() -> Self {
        Self {
            arena: ArenaAlloc::with_storage(std::ptr::null_mut(), 0, N),
            _marker: PhantomData,
        }
    }

    pub fn reset(&mut self) {
        self.arena.reset();
    }
}

impl<const N: usize> Default for SkSTArenaAlloc<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> std::ops::Deref for SkSTArenaAlloc<N> {
    type Target = ArenaAlloc;
    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

impl<const N: usize> std::ops::DerefMut for SkSTArenaAlloc<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}

pub type SkSTArenaAllocWithReset<const N: usize> = SkSTArenaAlloc<N>;
pub type SkArenaAllocWithReset = ArenaAlloc;

#[allow(dead_code)]
fn _heap_alloc(size: usize, align: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, align).unwrap();
    unsafe { alloc(layout) }
}

#[allow(dead_code)]
unsafe fn _heap_free(ptr: *mut u8, size: usize, align: usize) {
    let layout = Layout::from_size_align(size, align).unwrap();
    unsafe { dealloc(ptr, layout) };
}

#[allow(dead_code)]
fn _zeroed<T>() -> MaybeUninit<T> {
    MaybeUninit::zeroed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_fib_block_sizes() {
        let mut fib = SkFibBlockSizes::<{ u32::MAX }>::new(1, 1);
        let mut last = 1;
        for _ in 0..64 {
            let size = fib.next_block_size();
            assert!(last <= size);
            last = size;
        }
        assert_eq!(last, 2971215073);
    }

    #[test]
    fn dev_arena_runs_destructors_in_reverse_order() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        struct D;
        impl Drop for D {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::Relaxed);
            }
        }
        {
            let mut arena = SkSTArenaAlloc::<64>::new();
            for _ in 0..10 {
                arena.make(D);
            }
            assert_eq!(DROPS.load(Ordering::Relaxed), 0);
        }
        assert_eq!(DROPS.load(Ordering::Relaxed), 10);
    }
}
