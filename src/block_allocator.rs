//! Block allocator and owning block-list support from `SkBlockAllocator`.

use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ops::{Deref, DerefMut};

pub const K_MAX_ALLOCATION_SIZE: usize = 1 << 29;
pub const K_ADDRESS_ALIGN: usize = 16;
pub const HEADER_SIZE: usize = std::mem::size_of::<Block>();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GrowthPolicy {
    Fixed = 0,
    Linear = 1,
    Fibonacci = 2,
    Exponential = 3,
}

pub const K_GROWTH_POLICY_COUNT: usize = 4;

#[repr(C)]
#[derive(Debug)]
pub struct Block {
    next: *mut Block,
    prev: *mut Block,
    data_start: i32,
    size: i32,
    cursor: i32,
    metadata: i32,
    allocator_metadata: i32,
}

impl Block {
    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn cursor(&self) -> i32 {
        self.cursor
    }

    pub fn avail<const ALIGN: usize>(&self) -> usize {
        self.avail_with_padding::<ALIGN, 0>()
    }

    pub fn avail_with_padding<const ALIGN: usize, const PADDING: usize>(&self) -> usize {
        if self.cursor < 0 {
            return 0;
        }
        let aligned = self.aligned_offset::<ALIGN, PADDING>(self.cursor as usize);
        self.size.saturating_sub(aligned as i32) as usize
    }

    pub fn first_aligned_offset<const ALIGN: usize, const PADDING: usize>(&self) -> usize {
        self.aligned_offset::<ALIGN, PADDING>(self.data_start as usize)
    }

    fn aligned_offset<const ALIGN: usize, const PADDING: usize>(&self, offset: usize) -> usize {
        if ALIGN <= 1 {
            return offset + PADDING;
        }
        if ALIGN <= K_ADDRESS_ALIGN {
            (offset + PADDING + ALIGN - 1) & !(ALIGN - 1)
        } else {
            let base = self as *const Block as usize;
            let addr = base + offset + PADDING;
            ((addr + ALIGN - 1) & !(ALIGN - 1)) - base
        }
    }

    fn aligned_offset_runtime(&self, offset: usize, align: usize, padding: usize) -> usize {
        if align <= 1 {
            return offset + padding;
        }
        if align <= K_ADDRESS_ALIGN {
            (offset + padding + align - 1) & !(align - 1)
        } else {
            let base = self as *const Block as usize;
            let addr = base + offset + padding;
            ((addr + align - 1) & !(align - 1)) - base
        }
    }

    pub fn ptr(&self, offset: usize) -> *const u8 {
        debug_assert!(offset <= self.size as usize);
        unsafe { (self as *const Block as *mut u8).add(offset) }
    }

    pub fn ptr_mut(&mut self, offset: usize) -> *mut u8 {
        debug_assert!(offset <= self.size as usize);
        unsafe { (self as *mut Block as *mut u8).add(offset) }
    }

    pub fn metadata(&self) -> i32 {
        self.metadata
    }

    pub fn set_metadata(&mut self, value: i32) {
        self.metadata = value;
    }

    pub fn release(&mut self, start: usize, end: usize) -> bool {
        if start < self.data_start as usize || end > self.size as usize || start >= end {
            return false;
        }
        if self.cursor as usize == end {
            self.cursor = start as i32;
            true
        } else {
            false
        }
    }

    pub fn resize(&mut self, start: usize, end: usize, delta: i32) -> bool {
        if start < self.data_start as usize || end > self.size as usize || start >= end {
            return false;
        }
        if delta > K_MAX_ALLOCATION_SIZE as i32 || delta < -(K_MAX_ALLOCATION_SIZE as i32) {
            return false;
        }
        if self.cursor as usize != end {
            return false;
        }
        let next = end as i64 + delta as i64;
        if next < start as i64 || next > self.size as i64 {
            return false;
        }
        self.cursor = next as i32;
        true
    }

    fn is_scratch(&self) -> bool {
        self.cursor < 0
    }

    fn mark_scratch(&mut self) {
        self.cursor = -1;
    }

    fn reset_for_reuse(&mut self) {
        self.cursor = self.data_start;
        self.metadata = 0;
        self.allocator_metadata = 0;
        self.next = std::ptr::null_mut();
        self.prev = std::ptr::null_mut();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ByteRange {
    pub block: *mut Block,
    pub start: usize,
    pub aligned_offset: usize,
    pub end: usize,
}

impl ByteRange {
    pub fn ptr(&self) -> *mut u8 {
        unsafe { (*self.block).ptr_mut(self.aligned_offset) }
    }

    pub fn block(&self) -> &Block {
        unsafe { &*self.block }
    }

    pub fn block_mut(&mut self) -> &mut Block {
        unsafe { &mut *self.block }
    }
}

pub struct BlockAllocator {
    blocks: Vec<*mut Block>,
    tail: *mut Block,
    scratch: *mut Block,
    policy: GrowthPolicy,
    block_increment: usize,
    n0: i64,
    n1: i64,
    prealloc_size: usize,
}

impl BlockAllocator {
    pub fn new(policy: GrowthPolicy, block_increment_bytes: usize) -> Self {
        Self::with_total_prealloc(policy, block_increment_bytes, std::mem::size_of::<Self>())
    }

    pub fn with_prealloc(
        policy: GrowthPolicy,
        block_increment_bytes: usize,
        additional_prealloc_bytes: usize,
    ) -> Self {
        Self::with_total_prealloc(
            policy,
            block_increment_bytes,
            std::mem::size_of::<Self>() + additional_prealloc_bytes,
        )
    }

    fn with_total_prealloc(
        policy: GrowthPolicy,
        block_increment_bytes: usize,
        total_prealloc: usize,
    ) -> Self {
        let increment = ((block_increment_bytes + K_ADDRESS_ALIGN - 1) & !(K_ADDRESS_ALIGN - 1))
            / K_ADDRESS_ALIGN;
        let increment = increment.clamp(1, u16::MAX as usize);
        let (n0, n1) = match policy {
            GrowthPolicy::Linear | GrowthPolicy::Exponential => (1, 1),
            GrowthPolicy::Fixed | GrowthPolicy::Fibonacci => (0, 1),
        };
        let head_start = 64usize.min(total_prealloc).max(HEADER_SIZE);
        let head = unsafe { alloc_block(total_prealloc.max(head_start), std::ptr::null_mut()) };
        unsafe {
            (*head).data_start = head_start as i32;
            (*head).cursor = head_start as i32;
        }
        Self {
            blocks: vec![head],
            tail: head,
            scratch: std::ptr::null_mut(),
            policy,
            block_increment: increment,
            n0,
            n1,
            prealloc_size: total_prealloc.max(head_start),
        }
    }

    pub fn prealloc_size(&self) -> usize {
        self.prealloc_size
    }

    pub fn prealloc_usable_space(&self) -> usize {
        let head = self.head_block();
        head.size.saturating_sub(head.data_start) as usize
    }

    pub fn metadata(&self) -> i32 {
        self.head_block().allocator_metadata
    }

    pub fn set_metadata(&mut self, value: i32) {
        self.head_block_mut().allocator_metadata = value;
    }

    pub fn current_block(&self) -> &Block {
        unsafe { &*self.tail }
    }

    pub fn current_block_mut(&mut self) -> &mut Block {
        unsafe { &mut *self.tail }
    }

    pub fn head_block(&self) -> &Block {
        unsafe { &*self.blocks[0] }
    }

    pub fn head_block_mut(&mut self) -> &mut Block {
        unsafe { &mut *self.blocks[0] }
    }

    pub fn scratch_block_size(&self) -> usize {
        if self.scratch.is_null() {
            0
        } else {
            unsafe { (*self.scratch).size as usize }
        }
    }

    pub fn total_size(&self) -> usize {
        self.blocks
            .iter()
            .map(|b| unsafe { (**b).size as usize })
            .sum::<usize>()
            + self.scratch_block_size()
    }

    pub fn total_usable_space(&self) -> usize {
        self.blocks
            .iter()
            .map(|b| {
                let b = unsafe { &**b };
                b.size.saturating_sub(b.data_start) as usize
            })
            .sum::<usize>()
            + if self.scratch.is_null() {
                0
            } else {
                let b = unsafe { &*self.scratch };
                b.size.saturating_sub(b.data_start) as usize
            }
    }

    pub fn total_space_in_use(&self) -> usize {
        self.blocks
            .iter()
            .map(|b| {
                let b = unsafe { &**b };
                if b.cursor < b.data_start {
                    0
                } else {
                    (b.cursor - b.data_start) as usize
                }
            })
            .sum()
    }

    pub fn allocate<const ALIGN: usize, const PADDING: usize>(&mut self, size: usize) -> ByteRange {
        assert!(size <= K_MAX_ALLOCATION_SIZE);
        let overhead = block_overhead::<ALIGN, PADDING>();
        let tail = self.tail_mut();
        let mut offset = tail.aligned_offset::<ALIGN, PADDING>(tail.cursor as usize);
        let mut end = offset + size;
        if end > tail.size as usize {
            self.add_block(size + overhead, max_block_size::<ALIGN, PADDING>());
            let tail = self.tail_mut();
            offset = tail.aligned_offset::<ALIGN, PADDING>(tail.cursor as usize);
            end = offset + size;
        }
        let tail = self.tail_mut();
        let start = tail.cursor as usize;
        tail.cursor = end as i32;
        ByteRange {
            block: self.tail,
            start,
            aligned_offset: offset,
            end,
        }
    }

    pub fn allocate_aligned(&mut self, size: usize, align: usize, padding: usize) -> ByteRange {
        assert!(align.is_power_of_two());
        assert!(size <= K_MAX_ALLOCATION_SIZE);
        let overhead = block_overhead_runtime(align, padding);
        let block = self.tail_mut();
        let mut offset = block.aligned_offset_runtime(block.cursor as usize, align, padding);
        let mut end = offset + size;
        if end > block.size as usize {
            self.add_block(size + overhead, K_MAX_ALLOCATION_SIZE + overhead);
            let block = self.tail_mut();
            offset = block.aligned_offset_runtime(block.cursor as usize, align, padding);
            end = offset + size;
        }
        let block = self.tail_mut();
        let start = block.cursor as usize;
        block.cursor = end as i32;
        ByteRange {
            block: self.tail,
            start,
            aligned_offset: offset,
            end,
        }
    }

    pub fn reserve(&mut self, size: usize, ignore_growth: bool, ignore_existing: bool) {
        assert!(size <= K_MAX_ALLOCATION_SIZE);
        if !ignore_existing && self.current_block().avail::<1>() >= size {
            return;
        }
        let block_size = HEADER_SIZE + size;
        let max_size = if ignore_growth {
            block_size
        } else {
            HEADER_SIZE + K_MAX_ALLOCATION_SIZE
        };
        self.add_block(block_size, max_size);
        self.release_block(self.tail);
    }

    pub fn reset_scratch_space(&mut self) {
        if !self.scratch.is_null() {
            unsafe { free_block(self.scratch) };
            self.scratch = std::ptr::null_mut();
        }
    }

    pub fn release_block(&mut self, block: *mut Block) {
        if block.is_null() {
            return;
        }
        if block == self.blocks[0] {
            let head = unsafe { &mut *block };
            head.cursor = head.data_start;
            head.metadata = 0;
            self.decrement_growth();
            return;
        }
        let Some(index) = self.blocks.iter().position(|&b| b == block) else {
            return;
        };
        let b = unsafe { &mut *block };
        unsafe {
            if !b.prev.is_null() {
                (*b.prev).next = b.next;
            }
            if !b.next.is_null() {
                (*b.next).prev = b.prev;
            }
        }
        if self.tail == block {
            self.tail = b.prev;
        }
        self.blocks.remove(index);
        if self.scratch_block_size() < b.size as usize {
            self.reset_scratch_space();
            b.mark_scratch();
            self.scratch = block;
        } else {
            unsafe { free_block(block) };
        }
        self.decrement_growth();
    }

    pub fn steal_heap_blocks(&mut self, other: &mut BlockAllocator) {
        if other.blocks.len() <= 1 {
            return;
        }
        let stolen = other.blocks.split_off(1);
        let first = stolen[0];
        let last = *stolen.last().unwrap();
        unsafe {
            (*first).prev = self.tail;
            (*self.tail).next = first;
        }
        self.tail = last;
        self.blocks.extend(stolen);
        let other_head = other.blocks[0];
        unsafe {
            (*other_head).next = std::ptr::null_mut();
        }
        other.tail = other_head;
    }

    pub fn reset(&mut self) {
        self.reset_scratch_space();
        for &block in self.blocks.iter().skip(1) {
            unsafe { free_block(block) };
        }
        self.blocks.truncate(1);
        let head = unsafe { &mut *self.blocks[0] };
        head.next = std::ptr::null_mut();
        head.cursor = head.data_start;
        head.metadata = 0;
        head.allocator_metadata = 0;
        self.tail = self.blocks[0];
        self.n0 = match self.policy {
            GrowthPolicy::Linear | GrowthPolicy::Exponential => 1,
            _ => 0,
        };
        self.n1 = 1;
    }

    pub fn find_owning_block(&self, ptr: *const u8) -> *mut Block {
        let addr = ptr as usize;
        for &block in self.blocks.iter().rev() {
            let b = unsafe { &*block };
            let lower = block as usize + b.data_start as usize;
            let upper = block as usize + b.size as usize;
            if (lower..upper).contains(&addr) {
                return block;
            }
        }
        std::ptr::null_mut()
    }

    pub fn owning_block<const ALIGN: usize, const PADDING: usize>(
        &mut self,
        ptr: *const u8,
        start: usize,
    ) -> *mut Block {
        if ALIGN <= K_ADDRESS_ALIGN {
            ((ptr as usize).wrapping_sub(start).wrapping_sub(PADDING) & !(ALIGN - 1)) as *mut Block
        } else {
            self.find_owning_block(ptr)
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter().map(|&b| unsafe { &*b })
    }

    pub fn rblocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter().rev().map(|&b| unsafe { &*b })
    }

    pub fn blocks_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        self.blocks.iter_mut().map(|&mut b| unsafe { &mut *b })
    }

    pub fn validate(&self) {
        assert!(!self.blocks.is_empty());
        let head = self.blocks[0];
        assert!(self.tail == head || self.blocks.contains(&self.tail));
        for (i, &block) in self.blocks.iter().enumerate() {
            let b = unsafe { &*block };
            assert!(!b.is_scratch());
            assert!(b.data_start as usize >= HEADER_SIZE);
            assert!(b.cursor >= b.data_start && b.cursor <= b.size);
            if i > 0 {
                assert_eq!(b.prev, self.blocks[i - 1]);
            }
            if i + 1 < self.blocks.len() {
                assert_eq!(b.next, self.blocks[i + 1]);
            } else {
                assert!(b.next.is_null());
                assert_eq!(block, self.tail);
            }
        }
        if !self.scratch.is_null() {
            assert!(unsafe { (*self.scratch).is_scratch() });
            assert!(!self.blocks.contains(&self.scratch));
        }
    }

    fn tail_mut(&mut self) -> &mut Block {
        unsafe { &mut *self.tail }
    }

    fn add_block(&mut self, min_size: usize, max_size: usize) {
        assert!(min_size > HEADER_SIZE && min_size <= max_size);
        if self.scratch_block_size() >= min_size {
            let scratch = self.scratch;
            self.scratch = std::ptr::null_mut();
            unsafe {
                (*scratch).reset_for_reuse();
            }
            self.insert_heap_block(scratch);
            return;
        }
        let alloc_size = if min_size < max_size {
            let next_n1 = self.n0 + self.n1;
            let next_n0 = match self.policy {
                GrowthPolicy::Fixed | GrowthPolicy::Linear => self.n0,
                GrowthPolicy::Fibonacci => self.n1,
                GrowthPolicy::Exponential => next_n1,
            };
            const K_MAX_N: i64 = (1 << 23) - 1;
            self.n0 = next_n0.min(K_MAX_N);
            self.n1 = next_n1.min(K_MAX_N);
            let size_increment = (self.block_increment * K_ADDRESS_ALIGN) as i64;
            if size_increment > 0 && max_size as i64 / size_increment < next_n1 {
                max_size
            } else {
                let wanted = ((size_increment * next_n1) as usize).max(min_size);
                align_alloc_size(wanted).min(max_size)
            }
        } else {
            align_alloc_size(min_size)
        };
        let block = unsafe { alloc_block(alloc_size, self.tail) };
        self.insert_heap_block(block);
    }

    fn insert_heap_block(&mut self, block: *mut Block) {
        unsafe {
            (*block).prev = self.tail;
            (*self.tail).next = block;
        }
        self.tail = block;
        self.blocks.push(block);
    }

    fn decrement_growth(&mut self) {
        if self.n0 > 0 && (self.n1 > 1 || self.policy == GrowthPolicy::Fibonacci) {
            match self.policy {
                GrowthPolicy::Linear => self.n1 -= self.n0,
                GrowthPolicy::Fibonacci => {
                    let temp = self.n1 - self.n0;
                    self.n1 -= temp;
                    self.n0 = temp;
                }
                GrowthPolicy::Exponential => {
                    self.n1 >>= 1;
                    self.n0 = self.n1;
                }
                GrowthPolicy::Fixed => {}
            }
        }
        self.n1 = self.n1.max(1);
        self.n0 = self.n0.max(0);
    }
}

impl Drop for BlockAllocator {
    fn drop(&mut self) {
        self.reset_scratch_space();
        for &block in self.blocks.iter() {
            unsafe { free_block(block) };
        }
        self.blocks.clear();
    }
}

const fn block_overhead<const ALIGN: usize, const PADDING: usize>() -> usize {
    let value = HEADER_SIZE + PADDING;
    if ALIGN <= 1 {
        value
    } else {
        (value + ALIGN - 1) & !(ALIGN - 1)
    }
}

fn block_overhead_runtime(align: usize, padding: usize) -> usize {
    if align <= 1 {
        HEADER_SIZE + padding
    } else {
        (HEADER_SIZE + padding + align - 1) & !(align - 1)
    }
}

const fn max_block_size<const ALIGN: usize, const PADDING: usize>() -> usize {
    block_overhead::<ALIGN, PADDING>() + K_MAX_ALLOCATION_SIZE
}

fn align_alloc_size(size: usize) -> usize {
    let mask = if size > (1 << 15) {
        (1 << 12) - 1
    } else {
        K_ADDRESS_ALIGN - 1
    };
    (size + mask) & !mask
}

unsafe fn alloc_block(size: usize, prev: *mut Block) -> *mut Block {
    let layout = Layout::from_size_align(size.max(HEADER_SIZE), K_ADDRESS_ALIGN).unwrap();
    let ptr = unsafe { alloc(layout) } as *mut Block;
    if ptr.is_null() {
        handle_alloc_error(layout);
    }
    unsafe {
        ptr.write(Block {
            next: std::ptr::null_mut(),
            prev,
            data_start: HEADER_SIZE as i32,
            size: size.max(HEADER_SIZE) as i32,
            cursor: HEADER_SIZE as i32,
            metadata: 0,
            allocator_metadata: 0,
        });
    }
    ptr
}

unsafe fn free_block(block: *mut Block) {
    let size = unsafe { (*block).size as usize };
    let layout = Layout::from_size_align(size.max(HEADER_SIZE), K_ADDRESS_ALIGN).unwrap();
    unsafe { dealloc(block as *mut u8, layout) };
}

pub struct SkSBlockAllocator<const N: usize> {
    allocator: BlockAllocator,
}

impl<const N: usize> SkSBlockAllocator<N> {
    pub fn new(policy: GrowthPolicy) -> Self {
        Self {
            allocator: BlockAllocator::with_total_prealloc(policy, N, N),
        }
    }

    pub fn with_increment(policy: GrowthPolicy, block_increment_bytes: usize) -> Self {
        Self {
            allocator: BlockAllocator::with_total_prealloc(policy, block_increment_bytes, N),
        }
    }

    pub fn allocator(&self) -> &BlockAllocator {
        &self.allocator
    }

    pub fn allocator_mut(&mut self) -> &mut BlockAllocator {
        &mut self.allocator
    }
}

impl<const N: usize> Default for SkSBlockAllocator<N> {
    fn default() -> Self {
        Self::new(GrowthPolicy::Fixed)
    }
}

impl<const N: usize> Deref for SkSBlockAllocator<N> {
    type Target = BlockAllocator;
    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

impl<const N: usize> DerefMut for SkSBlockAllocator<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.allocator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_block_allocator_allocate_resize_release() {
        let mut pool = SkSBlockAllocator::<1024>::default();
        assert_eq!(pool.prealloc_size(), 1024);
        let mut p = pool.allocate::<8, 0>(32);
        let (start, end, aligned) = (p.start, p.end, p.aligned_offset);
        assert_eq!(p.block().ptr(aligned) as usize % 8, 0);
        assert!(p.block_mut().resize(start, end, 16));
        assert!(p.block_mut().release(start, end + 16));
        let p2 = pool.allocate::<8, 0>(32);
        assert_eq!(start, p2.start);
        pool.validate();
    }

    #[test]
    fn dev_block_allocator_growth_and_reset() {
        let mut pool = SkSBlockAllocator::<128>::new(GrowthPolicy::Linear);
        let mut sizes = Vec::new();
        for _ in 0..3 {
            let before = pool.total_size();
            loop {
                let avail = pool.current_block().avail::<4>();
                if avail == 0 {
                    break;
                }
                pool.allocate::<4, 0>(avail);
            }
            pool.allocate::<4, 0>(4);
            sizes.push(pool.total_size() - before);
        }
        assert_eq!(sizes, vec![256, 384, 512]);
        pool.reset();
        assert_eq!(pool.total_size(), pool.prealloc_size());
    }
}
