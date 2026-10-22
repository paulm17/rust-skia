use super::*;
fn total(p: &BlockAllocator) -> usize {
    p.total_size() - p.scratch_block_size()
}
fn block(p: &BlockAllocator, i: usize) -> *mut Block {
    p.blocks().nth(i).unwrap() as *const Block as *mut Block
}
fn add_block(p: &mut BlockAllocator) -> usize {
    let size = total(p);
    let current = block(p, p.blocks().count() - 1);
    while std::ptr::eq(p.current_block(), current) {
        p.allocate::<4, 0>(p.prealloc_size() / 2);
    }
    total(p) - size
}
fn byte(p: &mut BlockAllocator) -> *mut u8 {
    p.allocate::<1, 0>(1).ptr()
}
fn validate(p: &BlockAllocator) {
    if cfg!(debug_assertions) {
        p.validate();
    }
}
#[test]
fn SkBlockAllocatorPreallocSize() {
    let mut r = Reporter::default();
    let p = BlockAllocator::new(GrowthPolicy::Fixed, 2048);
    validate(&p);
    check!(
        r,
        p.prealloc_size() == std::mem::size_of::<BlockAllocator>()
    );
    check!(
        r,
        p.prealloc_usable_space() == p.current_block().avail::<1>()
    );
    let p = BlockAllocator::with_prealloc(
        GrowthPolicy::Linear,
        1024,
        1024 - std::mem::size_of::<BlockAllocator>(),
    );
    check!(r, p.prealloc_size() == 1024);
    check!(
        r,
        p.prealloc_usable_space() < 1024
            && p.prealloc_usable_space() >= 1024 - std::mem::size_of::<BlockAllocator>()
    );
    let p = SkSBlockAllocator::<2048>::new(GrowthPolicy::Fixed);
    validate(&p);
    check!(r, p.prealloc_size() == 2048);
    check!(
        r,
        p.prealloc_usable_space() < 2048
            && p.prealloc_usable_space() >= 2048 - std::mem::size_of::<BlockAllocator>()
    );
    r.finish();
}
fn validate_ptr(
    r: &mut Reporter,
    p: &BlockAllocator,
    align: usize,
    size: usize,
    b: ByteRange,
    prev: Option<ByteRange>,
) {
    let pt = b.ptr() as usize;
    check!(r, pt % align == 0);
    check!(r, b.end - b.aligned_offset >= size);
    check!(r, b.aligned_offset >= b.start);
    check!(r, b.aligned_offset - b.start <= align - 1);
    check!(r, std::ptr::eq(p.current_block(), b.block));
    if let Some(prev) = prev {
        let end = unsafe { (*prev.block).ptr(prev.end - 1) } as usize;
        check!(r, pt > end);
    }
    unsafe {
        b.ptr().write_bytes(0xff, b.end - b.aligned_offset);
    }
}
#[test]
fn SkBlockAllocatorAlloc() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Fixed);
    validate(&p);
    let mut prev = None;
    for (a, s) in [(1, 14), (2, 24), (4, 28), (8, 40), (16, 64), (32, 96)] {
        let b = p.allocate_aligned(s, a, 0);
        validate_ptr(&mut r, &p, a, s, b, prev);
        prev = Some(b);
    }
    check!(r, total(&p) == p.prealloc_size());
    validate(&p);
    let avail = p.current_block().avail::<4>();
    let b = p.allocate::<4, 0>(avail);
    validate_ptr(&mut r, &p, 4, avail, b, prev);
    check!(r, p.current_block().avail::<4>() < 4);
    let b = p.allocate::<4, 0>(4);
    validate_ptr(&mut r, &p, 4, 4, b, None);
    check!(r, total(&p) > p.prealloc_size());
    let current = total(&p);
    let big = p.current_block().avail::<4>() * 2;
    let b = p.allocate::<4, 0>(big);
    validate_ptr(&mut r, &p, 4, big, b, None);
    check!(r, total(&p) > current);
    check!(r, total(&p) - current < 4096);
    let current = total(&p);
    let b = p.allocate::<4, 0>(4096);
    validate_ptr(&mut r, &p, 4, 4096, b, None);
    check!(r, total(&p) >= current + 4096);
    validate(&p);
    r.finish();
}
#[test]
fn SkBlockAllocatorResize() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Fixed);
    validate(&p);
    let mut b = p.allocate::<4, 0>(16);
    unsafe {
        check!(r, (*b.block).avail::<4>() > 16);
        check!(r, (*b.block).resize(b.start, b.end, 16));
        b.end += 16;
        b.ptr().write_bytes(0x11, b.end - b.aligned_offset);
        let mut next = p.allocate::<4, 0>(16);
        check!(
            r,
            (*next.block).ptr(next.aligned_offset) as usize
                - (*next.block).ptr(b.aligned_offset) as usize
                == 32
        );
        check!(r, b.block == next.block);
        check!(r, !(*b.block).resize(b.start, b.end, 48));
        check!(r, (*b.block).release(next.start, next.end));
        let fill = (*b.block).avail::<4>();
        check!(r, (*b.block).resize(b.start, b.end, fill as i32));
        b.end += fill;
        b.ptr().write_bytes(0x22, b.end - b.aligned_offset);
        check!(r, (*b.block).avail::<4>() < fill);
        check!(r, !(*b.block).resize(b.start, b.end, fill as i32));
        let shrink = b.start as i32 - b.end as i32 + 32;
        check!(r, (*b.block).resize(b.start, b.end, shrink));
        b.end = (b.end as i32 + shrink) as usize;
        check!(r, b.end - b.start == 32);
        b.ptr().write_bytes(0x33, b.end - b.aligned_offset);
        next = p.allocate::<4, 0>(16);
        check!(
            r,
            (*next.block).ptr(next.aligned_offset) as usize
                - (*next.block).ptr(b.aligned_offset) as usize
                == 32
        );
        validate(&p);
        let shrink = next.start as i32 - next.end as i32;
        if !cfg!(debug_assertions) {
            check!(r, !(*next.block).resize(next.start, next.end, shrink - 1));
        }
        check!(r, (*next.block).resize(next.start, next.end, shrink));
    }
    r.finish();
}
#[test]
fn SkBlockAllocatorRelease() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Fixed);
    validate(&p);
    unsafe {
        let mut b = p.allocate::<8, 0>(32);
        check!(r, p.current_block_mut().release(b.start, b.end));
        let b2 = p.allocate::<8, 0>(32);
        check!(r, b.start == b2.start);
        let b3 = p.allocate::<8, 0>(64);
        check!(r, !(*b2.block).release(b2.start, b2.end));
        let b4 = p.allocate::<8, 0>(16);
        let b5 = p.allocate::<8, 0>(96);
        check!(r, (*b5.block).release(b5.start, b5.end));
        check!(r, (*b4.block).release(b4.start, b4.end));
        check!(r, (*b2.block).release(b2.start, b3.end));
        b = p.allocate::<8, 0>(32);
        check!(r, !(*b.block).release(b.start, b.end - 16));
        check!(r, !(*b.block).release(b.start, b.end + 16));
        check!(r, (*b.block).release(b.start, b.end));
    }
    validate(&p);
    r.finish();
}
#[test]
fn SkBlockAllocatorRewind() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Fixed);
    validate(&p);
    let mut ptrs = Vec::with_capacity(32);
    for _ in 0..32 {
        ptrs.push(p.allocate::<4, 0>(16));
    }
    validate(&p);
    for i in (0..32).rev() {
        let b = ptrs[i];
        check!(r, unsafe { (*b.block).release(b.start, b.end) });
    }
    validate(&p);
    check!(r, p.allocate::<4, 0>(16).start == ptrs[0].start);
    r.finish();
}
const POLICIES: [GrowthPolicy; 4] = [
    GrowthPolicy::Fixed,
    GrowthPolicy::Linear,
    GrowthPolicy::Fibonacci,
    GrowthPolicy::Exponential,
];
#[test]
fn SkBlockAllocatorGrowthPolicy() {
    let mut r = Reporter::default();
    let expected = [
        [1, 1, 1, 1, 1],
        [1, 2, 3, 4, 5],
        [1, 1, 2, 3, 5],
        [1, 2, 4, 8, 16],
    ];
    for (gp, policy) in POLICIES.into_iter().enumerate() {
        let mut p = SkSBlockAllocator::<128>::new(policy);
        validate(&p);
        check!(r, expected[gp][0] * 128 == total(&p));
        for i in 1..5 {
            let actual = add_block(&mut p);
            eprintln!(
                "policy={gp} block={i} expected={} actual={actual}",
                expected[gp][i] * 128
            );
            check!(r, expected[gp][i] * 128 == actual);
        }
        validate(&p);
    }
    r.finish();
}
#[test]
fn SkBlockAllocatorReset() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Linear);
    validate(&p);
    let first = byte(&mut p);
    for _ in 0..3 {
        add_block(&mut p);
    }
    validate(&p);
    check!(r, p.blocks().count() == 4);
    unsafe {
        (*block(&p, 0)).set_metadata(2);
    }
    p.reset();
    validate(&p);
    check!(r, p.blocks().count() == 1);
    check!(r, p.prealloc_size() == p.total_size());
    check!(r, p.head_block().metadata() == 0);
    check!(r, first == byte(&mut p));
    check!(r, 2 * 1024 == add_block(&mut p));
    check!(r, 3 * 1024 == add_block(&mut p));
    validate(&p);
    r.finish();
}
#[test]
fn SkBlockAllocatorReleaseBlock() {
    let mut r = Reporter::default();
    for policy in POLICIES {
        let mut p = SkSBlockAllocator::<1024>::new(policy);
        validate(&p);
        let first = byte(&mut p);
        let s1 = total(&p);
        let s2 = add_block(&mut p);
        let s3 = add_block(&mut p);
        let s4 = add_block(&mut p);
        validate(&p);
        for i in 0..4 {
            unsafe {
                (*block(&p, i)).set_metadata(i as i32 + 1);
            }
        }
        check!(r, total(&p) == s1 + s2 + s3 + s4);
        {
            let b = block(&p, 1);
            p.release_block(b);
        }
        check!(r, p.blocks().count() == 3);
        check!(r, unsafe { (*block(&p, 1)).metadata() } == 3);
        check!(r, total(&p) == s1 + s3 + s4);
        {
            let b = block(&p, 1);
            p.release_block(b);
        }
        check!(r, p.blocks().count() == 2);
        check!(r, unsafe { (*block(&p, 1)).metadata() } == 4);
        check!(r, total(&p) == s1 + s4);
        {
            let b = block(&p, 1);
            p.release_block(b);
        }
        check!(r, p.blocks().count() == 1);
        check!(r, total(&p) == s1);
        p.reset_scratch_space();
        let size = add_block(&mut p);
        check!(r, size == s2);
        {
            let b = block(&p, 1);
            p.release_block(b);
        }
        {
            let b = block(&p, 0);
            p.release_block(b);
        }
        check!(r, total(&p) == p.prealloc_size());
        check!(r, p.blocks().count() == 1);
        check!(r, first == byte(&mut p));
        check!(r, p.head_block().metadata() == 0);
        add_block(&mut p);
        add_block(&mut p);
        {
            let b = block(&p, 0);
            p.release_block(b);
        }
        check!(r, p.blocks().count() == 3);
        validate(&p);
    }
    r.finish();
}
#[test]
fn SkBlockAllocatorIterateAndRelease() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<256>::new(GrowthPolicy::Fixed);
    for reverse in [false, true] {
        p.head_block_mut().set_metadata(1);
        for _ in 0..3 {
            add_block(&mut p);
        }
        let blocks: Vec<_> = if reverse {
            p.rblocks()
                .map(|b| b as *const Block as *mut Block)
                .collect()
        } else {
            p.blocks()
                .map(|b| b as *const Block as *mut Block)
                .collect()
        };
        let mut count = 0;
        for b in blocks {
            p.release_block(b);
            count += 1;
        }
        check!(r, count == 4);
        check!(r, p.head_block().metadata() == 0);
        check!(r, p.blocks().count() == 1);
    }
    r.finish();
}
#[test]
fn SkBlockAllocatorScratchBlockReserve() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<256>::new(GrowthPolicy::Fixed);
    let added = add_block(&mut p);
    check!(r, p.scratch_block_size() == 0);
    let total = p.total_size();
    let tail = p.current_block() as *const Block as *mut Block;
    p.release_block(tail);
    check!(r, p.total_size() == total);
    check!(r, p.scratch_block_size() == added);
    p.reset();
    check!(r, p.scratch_block_size() == 0);
    let avail = p.current_block().avail::<1>();
    let reserve = avail + 1;
    p.reserve(reserve, false, false);
    check!(r, p.current_block().avail::<1>() == avail);
    check!(
        r,
        p.scratch_block_size() >= reserve && p.scratch_block_size() % 256 == 0
    );
    let pre = p.total_size();
    p.allocate::<1, 0>(avail + 1);
    check!(r, p.scratch_block_size() == 0);
    check!(r, p.total_size() == pre);
    let avail = p.current_block().avail::<1>();
    p.reserve(avail, false, false);
    check!(r, p.scratch_block_size() == 0);
    p.reserve(avail, false, true);
    check!(r, p.scratch_block_size() > 0);
    p.reset_scratch_space();
    p.reserve(32, true, false);
    check!(
        r,
        p.scratch_block_size() > 0 && p.scratch_block_size() < 256
    );
    let old = p.current_block() as *const Block;
    let avail = p.current_block().avail::<1>();
    let scratch = 2 * avail;
    p.reserve(scratch, false, false);
    check!(r, p.scratch_block_size() >= scratch);
    let scratch = p.scratch_block_size();
    p.allocate::<1, 0>(scratch + 1);
    check!(r, !std::ptr::eq(p.current_block(), old));
    check!(r, p.scratch_block_size() == scratch);
    r.finish();
}
#[test]
fn SkBlockAllocatorStealBlocks() {
    let mut r = Reporter::default();
    let mut a = SkSBlockAllocator::<256>::new(GrowthPolicy::Fixed);
    let mut b = SkSBlockAllocator::<128>::new(GrowthPolicy::Fixed);
    for _ in 0..3 {
        add_block(&mut a);
    }
    for _ in 0..2 {
        add_block(&mut b);
    }
    let alloc = byte(&mut b);
    unsafe {
        *alloc = b't';
    }
    let owner = b.find_owning_block(alloc);
    check!(r, a.blocks().count() == 4);
    check!(r, b.blocks().count() == 3);
    let asize = a.total_size();
    let theft = b.total_size() - b.prealloc_size();
    a.steal_heap_blocks(&mut b);
    check!(r, a.blocks().count() == 6);
    check!(r, b.blocks().count() == 1);
    check!(r, b.prealloc_size() == b.total_size());
    check!(r, a.total_size() == asize + theft);
    check!(r, unsafe { *alloc } == b't');
    check!(r, a.find_owning_block(alloc) == owner);
    check!(r, b.find_owning_block(alloc).is_null());
    a.steal_heap_blocks(&mut b);
    check!(r, a.blocks().count() == 6);
    check!(r, b.blocks().count() == 1);
    r.finish();
}
#[repr(C)]
struct Meta {
    x: i32,
    y: i32,
}
#[repr(C, align(32))]
struct BigMeta {
    x: i32,
    y: i32,
}
#[test]
fn SkBlockAllocatorMetadata() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Fixed);
    validate(&p);
    assert!(std::mem::align_of::<Meta>() < 16);
    let b = p.allocate::<16, 8>(16);
    validate(&p);
    check!(r, b.aligned_offset - b.start >= 8);
    let meta = unsafe { b.ptr().sub(8).cast::<Meta>() };
    check!(r, meta as usize % std::mem::align_of::<Meta>() == 0);
    check!(r, b.ptr() as usize % 16 == 0);
    unsafe {
        meta.write(Meta { x: 2, y: 5 });
    }
    assert!(std::mem::align_of::<BigMeta>() == 32);
    let b = p.allocate::<32, 32>(16);
    validate(&p);
    check!(r, b.aligned_offset - b.start >= 32);
    let big = unsafe { b.ptr().sub(32).cast::<BigMeta>() };
    check!(r, big as usize % 32 == 0);
    check!(r, b.ptr() as usize % 16 == 0);
    unsafe {
        big.write(BigMeta { x: 3, y: 6 });
        check!(r, (*meta).x == 2 && (*meta).y == 5);
        check!(r, (*big).x == 3 && (*big).y == 6);
    }
    r.finish();
}
#[test]
fn SkBlockAllocatorAllocatorMetadata() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<256>::new(GrowthPolicy::Fixed);
    validate(&p);
    check!(r, p.metadata() == 0);
    p.set_metadata(4);
    check!(r, p.metadata() == 4);
    let head = block(&p, 0);
    p.release_block(head);
    check!(r, p.metadata() == 4);
    p.reset();
    check!(r, p.metadata() == 0);
    r.finish();
}
fn owning<const A: usize, const P: usize>(r: &mut Reporter, p: &mut BlockAllocator) {
    let b = p.allocate::<A, P>(1);
    let user = b.ptr();
    let meta = unsafe { user.sub(P) };
    check!(r, p.owning_block::<A, P>(user, b.start) == b.block);
    check!(r, p.owning_block::<A, 0>(meta, b.start) == b.block);
    let computed = (user as usize - b.aligned_offset) as *mut Block;
    check!(r, computed == b.block);
}
fn owning_all<const P: usize>(r: &mut Reporter, p: &mut BlockAllocator) {
    owning::<1, P>(r, p);
    owning::<2, P>(r, p);
    owning::<4, P>(r, p);
    owning::<8, P>(r, p);
    owning::<16, P>(r, p);
    owning::<32, P>(r, p);
    owning::<64, P>(r, p);
    owning::<128, P>(r, p);
}
#[test]
fn SkBlockAllocatorOwningBlock() {
    let mut r = Reporter::default();
    let mut p = SkSBlockAllocator::<1024>::new(GrowthPolicy::Fixed);
    validate(&p);
    owning_all::<1>(&mut r, &mut p);
    owning_all::<2>(&mut r, &mut p);
    owning_all::<4>(&mut r, &mut p);
    owning_all::<8>(&mut r, &mut p);
    owning_all::<16>(&mut r, &mut p);
    owning_all::<32>(&mut r, &mut p);
    owning_all::<3>(&mut r, &mut p);
    owning_all::<9>(&mut r, &mut p);
    owning_all::<17>(&mut r, &mut p);
    r.finish();
}
