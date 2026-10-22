use super::*;
use std::rc::Rc;
type List = TInternalLList<Rc<i32>>;
fn check_list(
    r: &mut Reporter,
    list: &List,
    empty: bool,
    n: usize,
    members: [bool; 4],
    elems: &[Rc<i32>; 4],
) {
    check!(r, empty == list.is_empty());
    if cfg!(debug_assertions) {
        list.validate();
        check!(r, n == list.count_entries());
        for i in 0..4 {
            check!(
                r,
                members[i] == list.iter().any(|e| Rc::ptr_eq(e, &elems[i]))
            );
        }
    }
}
#[test]
fn InternalLList() {
    let mut r = Reporter::default();
    let mut list = List::new();
    let e = std::array::from_fn(|i| Rc::new(i as i32));
    let mut h = [usize::MAX; 4];
    check_list(&mut r, &list, true, 0, [false; 4], &e);
    h[0] = list.add_to_head(e[0].clone());
    check_list(&mut r, &list, false, 1, [true, false, false, false], &e);
    for i in 1..4 {
        h[i] = list.add_to_head(e[i].clone());
    }
    check_list(&mut r, &list, false, 4, [true; 4], &e);
    for (i, cur) in list.iter().enumerate() {
        check!(r, **cur == 3 - i as i32);
    }
    for (i, cur) in list.iter_rev().enumerate() {
        check!(r, **cur == i as i32);
    }
    for i in [1, 3, 0] {
        list.remove(h[i]);
    }
    check_list(&mut r, &list, false, 1, [false, false, true, false], &e);
    list.remove(h[2]);
    check_list(&mut r, &list, true, 0, [false; 4], &e);
    h[1] = list.add_after(e[1].clone(), None);
    check_list(&mut r, &list, false, 1, [false, true, false, false], &e);
    list.remove(h[1]);
    h[1] = list.add_before(e[1].clone(), None);
    check_list(&mut r, &list, false, 1, [false, true, false, false], &e);
    h[0] = list.add_before(e[0].clone(), Some(h[1]));
    check_list(&mut r, &list, false, 2, [true, true, false, false], &e);
    h[3] = list.add_after(e[3].clone(), Some(h[1]));
    check_list(&mut r, &list, false, 3, [true, true, false, true], &e);
    list.add_before(e[2].clone(), Some(h[3]));
    check_list(&mut r, &list, false, 4, [true; 4], &e);
    for (i, cur) in list.iter().enumerate() {
        check!(r, **cur == i as i32);
    }
    while !list.is_empty() {
        list.remove(list.tail_handle().unwrap());
    }
    let mut a = List::new();
    let mut b = List::new();
    a.concat(&mut b);
    check_list(&mut r, &a, true, 0, [false; 4], &e);
    check_list(&mut r, &b, true, 0, [false; 4], &e);
    b.add_to_tail(e[0].clone());
    a.concat(&mut b);
    check_list(&mut r, &a, false, 1, [true, false, false, false], &e);
    check_list(&mut r, &b, true, 0, [false; 4], &e);
    b.add_to_tail(e[1].clone());
    a.concat(&mut b);
    check_list(&mut r, &a, false, 2, [true, true, false, false], &e);
    check_list(&mut r, &b, true, 0, [false; 4], &e);
    a.concat(&mut b);
    check_list(&mut r, &a, false, 2, [true, true, false, false], &e);
    check_list(&mut r, &b, true, 0, [false; 4], &e);
    b.add_to_tail(e[2].clone());
    b.add_to_tail(e[3].clone());
    a.concat(&mut b);
    check_list(&mut r, &a, false, 4, [true; 4], &e);
    check_list(&mut r, &b, true, 0, [false; 4], &e);
    for (i, cur) in a.iter().enumerate() {
        check!(r, **cur == i as i32);
    }
    r.finish();
}
