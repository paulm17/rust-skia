use super::*;
use std::cell::Cell;
use std::rc::Rc;
#[derive(Clone)]
struct Counts {
    created: Rc<Cell<usize>>,
    destroyed: Rc<Cell<usize>>,
}
impl Counts {
    fn new() -> Self {
        Self {
            created: Rc::new(Cell::new(0)),
            destroyed: Rc::new(Cell::new(0)),
        }
    }
}
struct Foo {
    x: i32,
    y: f32,
    counts: Counts,
}
impl Foo {
    fn new(x: i32, y: f32, c: &Counts) -> Self {
        c.created.set(c.created.get() + 1);
        Self {
            x,
            y,
            counts: c.clone(),
        }
    }
}
impl Drop for Foo {
    fn drop(&mut self) {
        self.counts.destroyed.set(self.counts.destroyed.get() + 1);
    }
}
#[repr(align(8))]
struct OddAlignment {
    buf: [u8; 10],
}
#[test]
fn ArenaAlloc() {
    let mut r = Reporter::default();
    for mode in 0..3 {
        let c = Counts::new();
        let mut storage = [0u8; 1024];
        {
            let mut dynamic = rust_skia::ArenaAlloc::new(0);
            let mut stack = SkSTArenaAlloc::<64>::new();
            let mut supplied = rust_skia::ArenaAlloc::with_storage(storage.as_mut_ptr(), 1024, 0);
            let a: &mut rust_skia::ArenaAlloc = match mode {
                0 => &mut dynamic,
                1 => &mut stack,
                _ => &mut supplied,
            };
            unsafe {
                check!(r, *a.make(3i32) == 3);
                let f = a.make(Foo::new(3, 4.0, &c));
                check!(r, (*f).x == 3);
                check!(r, (*f).y == 4.0);
                check!(r, c.created.get() == 1);
                check!(r, c.destroyed.get() == 0);
                a.alloc_uninitialized::<i32>(10);
                let z = a.make_array_default::<i32>(10);
                for i in 0..10 {
                    check!(r, *z.add(i) == 0);
                }
                // Constructor closure preserves ten distinct default constructions, not ten clones.
                let f = a.make_initialized_array(10, |_| Foo::new(-2, -3.0, &c));
                check!(r, (*f.add(3)).x == -2);
                check!(r, (*f.add(4)).y == -3.0);
                check!(r, c.created.get() == 11);
                check!(r, c.destroyed.get() == 0);
                a.make(OddAlignment { buf: [0; 10] });
            }
        }
        check!(r, c.created.get() == 11);
        check!(r, c.destroyed.get() == 11);
    }
    r.finish();
}
#[test]
fn ArenaAllocReset() {
    let mut a = SkSTArenaAllocWithReset::<64>::new();
    a.make_array_default::<u8>(256);
    a.reset();
    a.reset();
}
#[test]
fn ArenaAllocIsEmpty() {
    let mut r = Reporter::default();
    let mut storage = [0u8; 1000];
    for size in [1, 2, 3, 10, 100, 1000] {
        for s1 in [1, 10, 100, 1000] {
            for s2 in [1, 10, 100, 1000] {
                let mut a = rust_skia::ArenaAlloc::with_storage(storage.as_mut_ptr(), size, 1000);
                check!(r, a.is_empty());
                a.make_array_default::<u8>(s1);
                check!(r, !a.is_empty());
                a.make_array_default::<u8>(s2);
                check!(r, !a.is_empty());
                a.reset();
                check!(r, a.is_empty());
            }
        }
    }
    r.finish();
}
struct Node {
    next: *mut Node,
    filler: [u8; 64],
    counts: Counts,
}
impl Drop for Node {
    fn drop(&mut self) {
        self.counts.destroyed.set(self.counts.destroyed.get() + 1);
    }
}
#[test]
fn ArenaAllocWithMultipleBlocks() {
    let mut r = Reporter::default();
    let c = Counts::new();
    {
        let mut a = SkSTArenaAlloc::<64>::new();
        let mut cur = std::ptr::null_mut();
        for _ in 0..128 {
            c.created.set(c.created.get() + 1);
            cur = a.make(Node {
                next: cur,
                filler: [0; 64],
                counts: c.clone(),
            });
        }
    }
    check!(r, c.created.get() == 128);
    check!(r, c.destroyed.get() == 128);
    r.finish();
}
#[test]
fn ArenaAllocDestructionOrder() {
    // Upstream placement-new nodes are not registered for arena destruction. A recursive
    // explicit destructor traverses storage while all blocks are alive.
    struct N {
        next: *mut N,
    }
    unsafe fn destroy(n: *mut N, c: &Counts) {
        c.destroyed.set(c.destroyed.get() + 1);
        let next = unsafe { (*n).next };
        if !next.is_null() {
            unsafe { destroy(next, c) }
        }
    }
    let mut r = Reporter::default();
    let c = Counts::new();
    {
        let mut a = SkSTArenaAlloc::<64>::new();
        let mut cur = std::ptr::null_mut();
        for _ in 0..128 {
            let p = a.alloc_uninitialized::<u64>(
                std::mem::size_of::<N>() / std::mem::size_of::<*mut N>(),
            ) as *mut N;
            unsafe {
                p.write(N { next: cur });
            }
            cur = p;
            c.created.set(c.created.get() + 1);
        }
        unsafe { destroy(cur, &c) };
    }
    check!(r, c.created.get() == 128);
    check!(r, c.destroyed.get() == 128);
    {
        let mut a = SkSTArenaAlloc::<64>::new();
        let p = a.make_initialized_array(8, |i| i as i32);
        for i in 0..8 {
            check!(r, unsafe { *p.add(i) } == i as i32);
        }
    }
    r.finish();
}
#[test]
fn ArenaAllocUnusualAlignment() {
    let mut r = Reporter::default();
    let mut a = rust_skia::ArenaAlloc::new(4096);
    a.make(0u8);
    let p = a.make_bytes_aligned_to(4081, 8);
    check!(r, (p as usize & 7) == 0);
    r.finish();
}
#[test]
fn SkFibBlockSizes() {
    let mut r = Reporter::default();
    let mut a = rust_skia::SkFibBlockSizes::<{ u32::MAX }>::new(1, 1);
    let mut last = 1;
    for _ in 0..64 {
        let s = a.next_block_size();
        check!(r, last <= s);
        last = s;
    }
    check!(r, last == 2971215073);
    let mut a = rust_skia::SkFibBlockSizes::<{ u32::MAX }>::new(0, 1024);
    last = 1;
    for _ in 0..64 {
        let s = a.next_block_size();
        check!(r, last <= s);
        last = s;
        check!(r, last <= u32::MAX);
    }
    check!(r, last == 3524578 * 1024);
    let mut a = rust_skia::SkFibBlockSizes::<{ u32::MAX / 2 }>::new(1024, 0);
    last = 1;
    for _ in 0..64 {
        let s = a.next_block_size();
        check!(r, last <= s);
        last = s;
        check!(r, last <= u32::MAX / 2);
    }
    check!(r, last == 1346269 * 1024);
    r.finish();
}
