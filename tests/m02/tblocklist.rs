use super::*;
thread_local! {static INSTANCES:std::cell::Cell<i32>=const{std::cell::Cell::new(0)};}
fn count() -> i32 {
    INSTANCES.with(|c| c.get())
}
#[repr(C)]
struct C {
    id: i32,
    padding: [i32; 4],
}
impl C {
    fn new(id: i32) -> Self {
        INSTANCES.with(|c| c.set(c.get() + 1));
        Self {
            id,
            padding: [0; 4],
        }
    }
}
impl Default for C {
    fn default() -> Self {
        Self::new(-1)
    }
}
impl Clone for C {
    fn clone(&self) -> Self {
        Self::new(self.id)
    }
}
impl Drop for C {
    fn drop(&mut self) {
        INSTANCES.with(|c| c.set(c.get() - 1));
    }
}
#[derive(Clone, Copy)]
struct D {
    id: i32,
}
fn check_allocator_helper<const N: usize>(
    r: &mut Reporter,
    a: &mut TBlockList<C, N>,
    cnt: usize,
    pop: usize,
) {
    check!(r, (cnt == 0) == a.empty());
    check!(r, cnt == a.count());
    check!(r, cnt as i32 == count());
    let mut i = 0;
    for c in a.items() {
        check!(r, i as i32 == c.id);
        check!(r, a.item(i).unwrap().id == i as i32);
        i += 1;
    }
    check!(r, i == cnt);
    if cnt > 0 {
        check!(r, a.back().unwrap().id == cnt as i32 - 1);
    }
    if pop > 0 {
        for _ in 0..pop {
            a.pop_back();
        }
        check_allocator_helper(r, a, cnt - pop, 0);
    }
}
fn check_iterator_helper<const N: usize>(
    r: &mut Reporter,
    a: &mut TBlockList<C, N>,
    expected: &[*const C],
) {
    check!(r, a.count() == expected.len());
    let mut i = 0;
    for c in a.items() {
        check!(r, std::ptr::eq(c, expected[i]));
        i += 1;
    }
    check!(r, i == expected.len());
    i = 0;
    for c in a.items_mut() {
        check!(r, std::ptr::eq(c, expected[i]));
        i += 1;
    }
    check!(r, i == expected.len());
    let mut i = expected.len() as isize - 1;
    for c in a.ritems() {
        check!(r, std::ptr::eq(c, expected[i as usize]));
        i -= 1;
    }
    check!(r, i == -1);
    i = expected.len() as isize - 1;
    for c in a.ritems_mut() {
        check!(r, std::ptr::eq(c, expected[i as usize]));
        i -= 1;
    }
    check!(r, i == -1);
    for i in 0..a.count() {
        check!(r, std::ptr::eq(a.item(i).unwrap(), expected[i]));
        check!(r, std::ptr::eq(a.item(i).unwrap(), expected[i]));
    }
}
fn check_allocator<const N: usize>(
    r: &mut Reporter,
    a: &mut TBlockList<C, N>,
    cnt: usize,
    pop: usize,
) {
    assert!(a.empty());
    let mut items = Vec::new();
    for i in 0..cnt {
        match i % 5 {
            0 => {
                let c = C::new(i as i32);
                a.push_back(c.clone());
            }
            1 => {
                a.push_back(C::new(i as i32));
            }
            2 => {
                *a.push_back_default() = C::new(i as i32);
            }
            3 => {
                *a.push_back_default() = C::new(i as i32);
            }
            _ => {
                a.emplace_back(|| C::new(i as i32));
            }
        }
        items.push(a.back().unwrap() as *const C);
    }
    check_iterator_helper(r, a, &items);
    check_allocator_helper(r, a, cnt, pop);
    a.reset();
    check_iterator_helper(r, a, &[]);
    check_allocator_helper(r, a, 0, 0);
}
fn run_allocator_test<const N: usize>(r: &mut Reporter, a: &mut TBlockList<C, N>) {
    for (cnt, pop) in [
        (0, 0),
        (1, 1),
        (2, 2),
        (10, 1),
        (10, 5),
        (10, 10),
        (100, 10),
    ] {
        check_allocator(r, a, cnt, pop);
    }
}
fn run_concat_test<const A: usize, const B: usize>(r: &mut Reporter, ac: usize, bc: usize) {
    let mut a = TBlockList::<C, A>::new();
    let mut b = TBlockList::<C, B>::new();
    for i in 0..ac {
        a.emplace_back(|| C::new(i as i32));
    }
    for i in 0..bc {
        b.emplace_back(|| C::new((ac + i) as i32));
    }
    check!(r, a.count() == ac && b.count() == bc);
    check!(r, count() as usize == ac + bc);
    a.concat(&mut b);
    check!(r, a.count() == ac + bc);
    check!(r, b.count() == 0);
    check!(r, count() as usize == ac + bc);
    let mut i = 0;
    for item in a.items() {
        check!(r, i == item.id);
        i += 1;
    }
    check!(r, i as usize == ac + bc);
}
fn run_concat_trivial_test<const A: usize, const B: usize>(r: &mut Reporter, ac: usize, bc: usize) {
    let mut a = TBlockList::<D, A>::new();
    let mut b = TBlockList::<D, B>::new();
    for i in 0..ac {
        a.push_back(D { id: i as i32 });
    }
    for i in 0..bc {
        b.push_back(D {
            id: (ac + i) as i32,
        });
    }
    check!(r, a.count() == ac && b.count() == bc);
    a.concat(&mut b);
    check!(r, a.count() == ac + bc);
    check!(r, b.count() == 0);
    let mut i = 0;
    for item in a.items() {
        check!(r, i == item.id);
        i += 1;
    }
    check!(r, i as usize == ac + bc);
}
fn run_reserve_test<const N: usize>(r: &mut Reporter) {
    let per = N + 4;
    let mut list = TBlockList::<C, N>::with_block_size(per);
    let initial = list.total_size();
    for i in 0..N {
        list.push_back(C::new(i as i32));
    }
    check!(r, initial == list.total_size());
    list.reserve(2 * per);
    check!(r, list.count() == N);
    let reserved = list.total_size();
    eprintln!(
        "reserve N={N} initial={initial} reserved={reserved} min={}",
        initial + 2 * per * std::mem::size_of::<C>()
    );
    check!(r, reserved >= initial + 2 * per * std::mem::size_of::<C>());
    for i in 0..2 * per {
        list.push_back(C::new(i as i32));
    }
    check!(r, reserved == list.total_size());
    for i in 0..N {
        list.push_back(C::new(i as i32));
    }
    list.reserve(2 * per);
    check!(
        r,
        list.scratch_block_size() >= (per + N) * std::mem::size_of::<C>()
    );
    let reserved = list.total_size();
    for i in 0..2 * per {
        list.push_back(C::new(i as i32));
    }
    check!(r, reserved == list.total_size());
    list.reserve(2);
    check!(
        r,
        list.scratch_block_size() >= per * std::mem::size_of::<C>()
    );
    check!(r, count() as usize == 2 * (N + 2 * per));
    list.reset();
    check!(r, count() == 0);
}
#[test]
fn SkTBlockList() {
    let mut r = Reporter::default();
    for n in [1, 2, 5] {
        let mut a = TBlockList::<C>::with_block_size(n);
        run_allocator_test(&mut r, &mut a);
    }
    run_allocator_test(&mut r, &mut TBlockList::<C, 1>::new());
    run_allocator_test(&mut r, &mut TBlockList::<C, 3>::new());
    run_allocator_test(&mut r, &mut TBlockList::<C, 4>::new());
    run_reserve_test::<1>(&mut r);
    run_reserve_test::<2>(&mut r);
    run_reserve_test::<3>(&mut r);
    run_reserve_test::<4>(&mut r);
    run_reserve_test::<5>(&mut r);
    run_concat_test::<1, 1>(&mut r, 10, 10);
    run_concat_test::<5, 1>(&mut r, 50, 10);
    run_concat_test::<1, 5>(&mut r, 10, 50);
    run_concat_test::<5, 5>(&mut r, 100, 100);
    run_concat_trivial_test::<1, 1>(&mut r, 10, 10);
    run_concat_trivial_test::<5, 1>(&mut r, 50, 10);
    run_concat_trivial_test::<1, 5>(&mut r, 10, 50);
    run_concat_trivial_test::<5, 5>(&mut r, 100, 100);
    let item_count = std::mem::size_of::<C>() * u16::MAX as usize / 16 + 1;
    let mut large = TBlockList::<C>::with_block_size(item_count);
    large.reserve(10);
    let expected = (u16::MAX as usize * 16).div_ceil(4096) * 4096;
    check!(r, large.scratch_block_size() == expected);
    r.finish();
}
