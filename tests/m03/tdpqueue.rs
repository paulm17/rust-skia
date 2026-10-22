// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/TDPQueueTest.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
fn simple_test(r: &mut Reporter) {
    let mut heap = SkTDPQueue::new(|a: &i32, b: &i32| a.cmp(b));
    check!(r, 0 == heap.count());
    heap.insert(0);
    check!(r, 1 == heap.count());
    check!(r, 0 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 0 == heap.count());
    heap.insert(0);
    heap.insert(1);
    check!(r, 2 == heap.count());
    check!(r, 0 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 1 == heap.count());
    check!(r, 1 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 0 == heap.count());
    heap.insert(2);
    heap.insert(1);
    heap.insert(0);
    check!(r, 3 == heap.count());
    check!(r, 0 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 2 == heap.count());
    check!(r, 1 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 1 == heap.count());
    check!(r, 2 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 0 == heap.count());
    heap.insert(2);
    heap.insert(3);
    heap.insert(0);
    heap.insert(1);
    check!(r, 4 == heap.count());
    check!(r, 0 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 3 == heap.count());
    check!(r, 1 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 2 == heap.count());
    check!(r, 2 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 1 == heap.count());
    check!(r, 3 == *heap.peek().unwrap());
    heap.pop();
    check!(r, 0 == heap.count());
}
#[derive(Clone, Copy)]
struct Mock {
    value: i32,
    priority: i32,
    index: i32,
}
impl PartialEq for Mock {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.priority == other.priority
    }
}
fn mock_queue() -> SkTDPQueue<*mut Mock> {
    // Both callbacks are part of the production queue contract; tests do not maintain heap indices.
    SkTDPQueue::new_with_index(
        |a: &*mut Mock, b: &*mut Mock| unsafe { (**a).priority.cmp(&(**b).priority) },
        |mock: &*mut Mock| unsafe { std::ptr::addr_of_mut!((**mock).index) },
    )
}
fn random_test(r: &mut Reporter) {
    let mut random = SkRandom::new();
    let sentinel = Mock {
        value: -1,
        priority: -1,
        index: -1,
    };
    for _ in 0..100 {
        let count = random.next_u_less_than(100) as usize;
        let mut array = SkTDArray::new();
        array.reserve(count);
        let mut j = 0;
        while j < count {
            let mock = Mock {
                priority: random.next_s(),
                value: random.next_s(),
                index: -1,
            };
            array.push_back(mock);
            if mock == sentinel {
                array.pop_back();
            } else {
                j += 1;
            }
        }
        let mut pq = mock_queue();
        for j in 0..count {
            pq.insert(unsafe { array.as_mut_slice().as_mut_ptr().add(j) });
        }
        check!(r, pq.count() == array.count());
        for j in 0..count {
            check!(r, -1 != array[j].index);
        }
        while pq.count() != 0 {
            let mut top = *pq.peek().unwrap();
            for k in 0..count {
                check!(
                    r,
                    array[k] == sentinel || array[k].priority >= unsafe { (*top).priority }
                );
            }
            let action = random.next_u_less_than(3);
            match action {
                0 => {
                    top = *pq.peek().unwrap();
                    let begin = array.data();
                    let end = unsafe { begin.add(count) };
                    check!(r, begin <= top.cast_const() && top.cast_const() < end);
                    pq.pop();
                    unsafe {
                        *top = sentinel;
                    }
                }
                1 => {
                    let mut item;
                    loop {
                        item = random.next_u_less_than(count as u32) as usize;
                        if array[item] != sentinel {
                            break;
                        }
                    }
                    let ptr = unsafe { array.as_mut_slice().as_mut_ptr().add(item) };
                    pq.remove(&ptr);
                    array[item] = sentinel;
                }
                2 => {
                    let mut item;
                    loop {
                        item = random.next_u_less_than(count as u32) as usize;
                        if array[item] != sentinel {
                            break;
                        }
                    }
                    array[item].priority = random.next_s();
                    let ptr = unsafe { array.as_mut_slice().as_mut_ptr().add(item) };
                    pq.priority_did_change_for(&ptr);
                }
                _ => unreachable!(),
            }
        }
    }
}
fn sort_test(r: &mut Reporter) {
    let mut random = SkRandom::new();
    let mut test = mock_queue();
    let mut control = mock_queue();
    let count = random.next_u_less_than(100) as usize;
    let mut test_array = SkTDArray::new();
    test_array.reserve(count);
    for i in 0..count {
        let mock = Mock {
            priority: random.next_s(),
            value: random.next_s(),
            index: -1,
        };
        test_array.push_back(mock);
        test.insert(unsafe { test_array.as_mut_slice().as_mut_ptr().add(i) });
    }
    let mut control_array = SkTDArray::new();
    control_array.reserve(count);
    for i in 0..count {
        control_array.push_back(Mock {
            priority: test_array[i].priority,
            value: test_array[i].value,
            index: -1,
        });
        control.insert(unsafe { control_array.as_mut_slice().as_mut_ptr().add(i) });
    }
    test.sort();
    let mut prev = unsafe { (**test.peek().unwrap()).priority };
    for i in 0..count {
        let item = *test.at(i);
        check!(r, i as i32 <= unsafe { (*item).index });
        check!(r, prev <= unsafe { (*item).priority });
        prev = unsafe { (*item).priority };
    }
    for _ in 0..count {
        check!(r, unsafe {
            **control.peek().unwrap() == **test.peek().unwrap()
        });
        control.pop();
        test.pop();
    }
}
// tests/TDPQueueTest.cpp:202
#[test]
fn TDPQueueTest() {
    let mut r = Reporter::default();
    simple_test(&mut r);
    random_test(&mut r);
    sort_test(&mut r);
    r.finish();
}
