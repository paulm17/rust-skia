use super::*;
fn assert_count(r: &mut Reporter, d: &SkTDeque<i32>, count: usize) {
    if count == 0 {
        check!(r, d.empty());
        check!(r, d.count() == 0);
        check!(r, d.elem_size() == std::mem::size_of::<i32>());
        check!(r, d.front().is_none());
        check!(r, d.back().is_none());
    } else {
        check!(r, !d.empty());
        check!(r, count == d.count());
        check!(r, d.elem_size() == std::mem::size_of::<i32>());
        check!(r, d.front().is_some());
        check!(r, d.back().is_some());
        if count == 1 {
            check!(r, std::ptr::eq(d.front().unwrap(), d.back().unwrap()));
        } else {
            check!(r, !std::ptr::eq(d.front().unwrap(), d.back().unwrap()));
        }
    }
}
fn assert_iter(r: &mut Reporter, d: &SkTDeque<i32>, max: i32, min: i32) {
    let mut value = max;
    for p in d.iter() {
        check!(r, value == *p);
        value -= 1;
    }
    check!(r, value + 1 == min);
    value = min;
    for p in d.iter().rev() {
        check!(r, value == *p);
        value += 1;
    }
    check!(r, value - 1 == max);
    value = max;
    let halfway = d.count() / 2;
    for p in d.iter().take(halfway) {
        check!(r, value == *p);
        value -= 1;
    } // Original prev() returns the cursor's current (next-unread) item before moving backward.
    for p in d.iter().take(halfway + 1).rev() {
        check!(r, value == *p);
        value += 1;
    }
    check!(r, value - 1 == max);
}
fn assert_blocks(r: &mut Reporter, d: &SkTDeque<i32>, count: usize) {
    let blocks = d.num_blocks_allocated();
    if d.count() == 0 {
        check!(r, blocks == 1);
    } else {
        let expected = d.count().div_ceil(count);
        check!(r, expected == blocks || expected + 1 == blocks);
    }
}
fn test_sub(r: &mut Reporter, count: usize) {
    let mut d = SkTDeque::with_capacity(count);
    assert_count(r, &d, 0);
    for i in 1..=10 {
        d.push_front(i);
    }
    assert_count(r, &d, 10);
    assert_iter(r, &d, 10, 1);
    assert_blocks(r, &d, count);
    for _ in 0..5 {
        d.pop_front();
    }
    assert_count(r, &d, 5);
    assert_iter(r, &d, 5, 1);
    assert_blocks(r, &d, count);
    for _ in 0..5 {
        d.pop_front();
    }
    assert_count(r, &d, 0);
    assert_blocks(r, &d, count);
    for i in (1..=10).rev() {
        d.push_back(i);
    }
    assert_count(r, &d, 10);
    assert_iter(r, &d, 10, 1);
    assert_blocks(r, &d, count);
    for _ in 0..5 {
        d.pop_back();
    }
    assert_count(r, &d, 5);
    assert_iter(r, &d, 10, 6);
    assert_blocks(r, &d, count);
    for _ in 0..5 {
        d.pop_back();
    }
    assert_count(r, &d, 0);
    assert_blocks(r, &d, count);
    d.push_front(5);
    d.push_back(4);
    d.push_front(6);
    d.push_back(3);
    d.push_front(7);
    d.push_back(2);
    d.push_front(8);
    d.push_back(1);
    assert_count(r, &d, 8);
    assert_iter(r, &d, 8, 1);
    assert_blocks(r, &d, count);
}
#[test]
fn Deque() {
    let mut r = Reporter::default();
    test_sub(&mut r, 1);
    test_sub(&mut r, 10);
    r.finish();
}
