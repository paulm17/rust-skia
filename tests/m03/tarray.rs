// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/TArrayTest.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
use rust_skia::containers::STArray;
use std::fmt::Debug;
// Test fixture: dynamic dispatch and destruction retain the nontrivial TestClass case.
trait ClassValue: Debug {
    fn value(&self) -> i32;
}
#[derive(Debug)]
struct ClassInt(i32);
impl ClassValue for ClassInt {
    fn value(&self) -> i32 {
        self.0
    }
}
impl Drop for ClassInt {
    fn drop(&mut self) {}
}
#[derive(Debug)]
struct TestClass(Box<dyn ClassValue>);
impl Clone for TestClass {
    fn clone(&self) -> Self {
        Self::value(self.0.value())
    }
}
impl Default for TestClass {
    fn default() -> Self {
        Self::value(0)
    }
}
impl PartialEq for TestClass {
    fn eq(&self, other: &Self) -> bool {
        self.0.value() == other.0.value()
    }
}
trait Value: Clone + Default + PartialEq + Debug {
    fn value(n: i32) -> Self;
}
macro_rules! primitive_value { ($($t:ty),*) => { $(impl Value for $t { fn value(n:i32)->Self { n as Self } })* }; }
primitive_value!(i32, i64, u8, i8, u16, i16, f32, f64);
impl Value for TestClass {
    fn value(n: i32) -> Self {
        Self(Box::new(ClassInt(n)))
    }
}
// This adapter only forwards production calls. It must never implement container behavior.
trait Array<T: Value>: Default + Clone {
    fn reserved(n: usize) -> Self;
    fn from_values(s: &[T]) -> Self;
    fn size(&self) -> usize;
    fn empty(&self) -> bool;
    fn cap(&self) -> usize;
    fn add_value(&mut self, t: T);
    fn pop(&mut self);
    fn erase_shuffle(&mut self, i: usize);
    fn clear_values(&mut self);
    fn reset_values(&mut self, s: &[T]);
    fn push_n(&mut self, n: usize, t: T);
    fn pop_n(&mut self, n: usize);
    fn reserve_count(&mut self, n: usize);
    fn slice(&self) -> &[T];
    fn slice_mut(&mut self) -> &mut [T];
    fn same(&self, other: &Self) -> bool;
    fn different(&self, other: &Self) -> bool;
}
macro_rules! dynamic_array {
    ([$($generic:tt)*]; $a:ty) => {
        impl<$($generic)* T:Value> Array<T> for $a {
            fn reserved(n:usize)->Self { Self::with_capacity(n) }
            fn from_values(s:&[T])->Self { Self::from_slice(s) }
            fn size(&self)->usize { self.count() } fn empty(&self)->bool { self.empty() }
            fn cap(&self)->usize { self.capacity() }
            fn add_value(&mut self,t:T) { self.push_back(t); } fn pop(&mut self) { self.pop_back(); }
            fn erase_shuffle(&mut self,i:usize) { self.remove_shuffle(i); }
            fn clear_values(&mut self) { self.clear(); }
            fn reset_values(&mut self,s:&[T]) { self.reset_from_slice(s); }
            fn push_n(&mut self,n:usize,t:T) { self.push_back_n_with(n,t); }
            fn pop_n(&mut self,n:usize) { self.pop_back_n(n); } fn reserve_count(&mut self,n:usize) { self.reserve(n); }
            fn slice(&self)->&[T] { self.as_slice() } fn slice_mut(&mut self)->&mut [T] { self.as_mut_slice() }
            fn same(&self,other:&Self)->bool { self == other } fn different(&self,other:&Self)->bool { self != other }
        }
    };
}
dynamic_array!([]; SkTArray<T>);
dynamic_array!([const N:usize,]; STArray<N,T>);
impl<T: Value + Copy, const N: usize> Array<T> for FixedArray<N, T> {
    fn reserved(n: usize) -> Self {
        Self::with_capacity(n)
    }
    fn from_values(s: &[T]) -> Self {
        Self::from_slice(s)
    }
    fn size(&self) -> usize {
        self.size()
    }
    fn empty(&self) -> bool {
        self.is_empty()
    }
    fn cap(&self) -> usize {
        self.capacity()
    }
    fn add_value(&mut self, t: T) {
        self.push(t);
    }
    fn pop(&mut self) {
        self.pop_back();
    }
    fn erase_shuffle(&mut self, i: usize) {
        self.remove_shuffle(i);
    }
    fn clear_values(&mut self) {
        self.clear();
    }
    fn reset_values(&mut self, s: &[T]) {
        self.reset(s);
    }
    fn push_n(&mut self, n: usize, t: T) {
        self.push_back_n_with(n, t);
    }
    fn pop_n(&mut self, n: usize) {
        self.pop_back_n(n);
    }
    fn reserve_count(&mut self, n: usize) {
        self.reserve(n);
    }
    fn slice(&self) -> &[T] {
        self.as_slice()
    }
    fn slice_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
    fn same(&self, other: &Self) -> bool {
        self == other
    }
    fn different(&self, other: &Self) -> bool {
        self != other
    }
}
fn basic<T: Value, A: Array<T>>(r: &mut Reporter) {
    let mut a = A::default();
    check!(r, a.empty());
    check!(r, a.size() == 0);
    a.add_value(T::default());
    a.slice_mut()[0] = T::value(0);
    check!(r, !a.empty());
    check!(r, a.size() == 1);
    a.erase_shuffle(0);
    check!(r, a.empty());
    check!(r, a.size() == 0);
    a.add_value(T::default());
    a.slice_mut()[0] = T::value(0);
    a.add_value(T::default());
    a.slice_mut()[1] = T::value(1);
    a.erase_shuffle(0);
    check!(r, !a.empty());
    check!(r, a.size() == 1);
    check!(r, a.slice()[0] == T::value(1));
    let b = [
        T::value(0),
        T::value(1),
        T::value(2),
        T::value(3),
        T::value(4),
    ];
    a.reset_values(&b);
    check!(r, a.size() == b.len());
    check!(r, a.slice()[2] == T::value(2));
    check!(r, a.slice()[4] == T::value(4));
    a.erase_shuffle(4);
    check!(r, a.size() == b.len() - 1);
    check!(r, a.slice()[3] == T::value(3));
    a.erase_shuffle(1);
    check!(r, a.size() == b.len() - 2);
    check!(r, a.slice()[0] == T::value(0));
    check!(r, a.slice()[1] == T::value(3));
    check!(r, a.slice()[2] == T::value(2));
    a.pop();
    check!(r, a.size() == b.len() - 3);
    a.push_n(3, T::value(5));
    check!(r, a.size() == b.len());
    check!(r, a.slice()[2] == T::value(5));
    check!(r, a.slice()[3] == T::value(5));
    check!(r, a.slice()[4] == T::value(5));
    a.pop_n(4);
    check!(r, a.size() == b.len() - 4);
    check!(r, a.slice()[0] == T::value(0));
}
fn construction<T: Value, A: Array<T>>(r: &mut Reporter, has_move: bool) {
    let array_no_args = A::default();
    check!(r, array_no_args.empty());
    let array_reserve = A::reserved(15);
    check!(r, array_reserve.empty());
    check!(r, array_reserve.cap() >= 15 && array_reserve.cap() < 50);
    let mut initial = A::default();
    initial.add_value(T::value(1));
    initial.add_value(T::value(2));
    initial.add_value(T::value(3));
    let copy = initial.clone();
    check!(r, initial.size() == 3);
    check!(r, initial.slice()[0] == T::value(1));
    check!(r, initial.slice()[1] == T::value(2));
    check!(r, initial.slice()[2] == T::value(3));
    check!(r, copy.size() == 3);
    check!(r, copy.slice()[0] == T::value(1));
    check!(r, copy.slice()[1] == T::value(2));
    check!(r, copy.slice()[2] == T::value(3));
    // mem::take is Rust's defined moved-from state. FixedArray's C++ copy-on-rvalue remains a copy.
    let moved = if has_move {
        std::mem::take(&mut initial)
    } else {
        initial.clone()
    };
    if has_move {
        check!(r, initial.empty());
    }
    check!(r, moved.size() == 3);
    check!(r, moved.slice()[0] == T::value(1));
    check!(r, moved.slice()[1] == T::value(2));
    check!(r, moved.slice()[2] == T::value(3));
    let data = [T::value(7), T::value(8), T::value(9)];
    let ptr_count = A::from_values(&data);
    check!(r, ptr_count.size() == 3);
    check!(r, ptr_count.slice()[0] == T::value(7));
    check!(r, ptr_count.slice()[1] == T::value(8));
    check!(r, ptr_count.slice()[2] == T::value(9));
    let init = A::from_values(&[
        T::value(8),
        T::value(6),
        T::value(7),
        T::value(5),
        T::value(3),
        T::value(0),
        T::value(9),
    ]);
    check!(r, init.size() == 7);
    check!(r, init.slice()[0] == T::value(8));
    check!(r, init.slice()[1] == T::value(6));
    check!(r, init.slice()[2] == T::value(7));
    check!(r, init.slice()[3] == T::value(5));
    check!(r, init.slice()[4] == T::value(3));
    check!(r, init.slice()[5] == T::value(0));
    check!(r, init.slice()[6] == T::value(9));
}
fn self_assignment<T: Value, A: Array<T>>(r: &mut Reporter) {
    let mut a = A::default();
    a.add_value(T::value(1));
    check!(r, !a.empty());
    check!(r, a.size() == 1);
    check!(r, a.slice()[0] == T::value(1));
    // Defined Rust self-move assignment; creating aliasing &mut/& to invoke clone_from would be UB.
    a = a;
    check!(r, !a.empty());
    check!(r, a.size() == 1);
    check!(r, a.slice()[0] == T::value(1));
}
fn comparison<T: Value, A: Array<T>>(r: &mut Reporter) {
    let mut a = A::default();
    let mut b = A::default();
    check!(r, a.same(&b));
    check!(r, !a.different(&b));
    for x in 0..10 {
        a.add_value(T::value(x));
        b.add_value(T::value(x));
        check!(r, a.same(&b));
        check!(r, !a.different(&b));
    }
    for _ in 0..10 {
        a.pop();
        check!(r, a.different(&b));
        check!(r, b.different(&a));
        check!(r, !a.same(&b));
        check!(r, !b.same(&a));
    }
    a.clone_from(&b);
    for x in 0..10 {
        a.slice_mut()[x] = T::value(x as i32 + 100);
        check!(r, a.different(&b));
        check!(r, b.different(&a));
        check!(r, !a.same(&b));
        check!(r, !b.same(&a));
        a.slice_mut()[x] = T::value(x as i32);
    }
}
fn array_reserve<T: Value, A: Array<T>>(r: &mut Reporter, a: &mut A, n: usize) {
    let mut random = SkRandom::new();
    check!(r, a.cap() >= n);
    a.add_value(T::default());
    check!(r, a.cap() >= n);
    a.pop();
    check!(r, a.cap() >= n);
    while a.size() < n {
        if random.next_u_less_than(3) < 2 {
            a.add_value(T::default());
        } else if a.size() > 0 {
            a.pop();
        }
        check!(r, a.cap() >= n);
    }
}
fn reserve<T: Value, A: Array<T>>(r: &mut Reporter) {
    for n in [1, 2, 10, 100] {
        let mut a1 = A::reserved(n);
        array_reserve(r, &mut a1, n);
        let mut a2 = A::default();
        a2.reserve_count(n);
        array_reserve(r, &mut a2, n);
        let mut a3 = A::reserved(n / 2);
        a3.reserve_count(n);
        array_reserve(r, &mut a3, n);
        let mut a4 = A::default();
        a4.push_n(n, T::default());
        a4.reserve_count(2 * n);
        a4.pop_n(n);
        array_reserve(r, &mut a4, 2 * n);
    }
}
fn inner_push<T: Value, A: Array<T>>(r: &mut Reporter) {
    let mut a = A::default();
    a.add_value(T::value(12345));
    // A local copied/cloned element ends the borrow before a potentially reallocating push.
    for _ in 0..50 {
        let v = a.slice().first().unwrap().clone();
        a.add_value(v);
    }
    for _ in 0..50 {
        let v = a.slice().last().unwrap().clone();
        a.add_value(v);
    }
    check!(r, a.size() == 101);
    check!(
        r,
        a.slice().iter().filter(|v| **v == T::value(12345)).count() == a.size()
    );
}
struct MoveOnlyInt(i32);
trait MoveValue {
    fn make(v: i32) -> Self;
    fn equals(&self, v: i32) -> bool;
}
impl MoveValue for i32 {
    fn make(v: i32) -> Self {
        v
    }
    fn equals(&self, v: i32) -> bool {
        *self == v
    }
}
impl MoveValue for MoveOnlyInt {
    fn make(v: i32) -> Self {
        Self(v)
    }
    fn equals(&self, v: i32) -> bool {
        self.0 == v
    }
}
// STArray exposes its TArray base exactly as in the upstream helper.
fn swap_and_move<T: MoveValue>(r: &mut Reporter, do_move: bool) {
    let mut arr = SkTArray::<T>::new();
    let mut arr5 = STArray::<5, T>::new();
    let mut arr10 = STArray::<10, T>::new();
    let mut arr20 = STArray::<20, T>::new();
    let mut arrays: [&mut SkTArray<T>; 4] = [
        &mut arr,
        arr5.as_tarray_mut(),
        arr10.as_tarray_mut(),
        arr20.as_tarray_mut(),
    ];
    let sizes = [0, 1, 5, 10, 15, 20, 25];
    if do_move {
        for a in arrays.iter_mut() {
            for size_a in sizes {
                a.clear();
                for i in 0..size_a {
                    a.push_back(T::make(i as i32));
                }
                a.move_back_self();
                check!(r, a.count() == size_a);
                for i in 0..size_a {
                    check!(r, a[i].equals(i as i32));
                }
            }
        }
    }
    for ai in 0..4 {
        for bi in 0..4 {
            if ai == bi {
                continue;
            }
            for size_a in sizes {
                for size_b in sizes {
                    // Distinct indices provide disjoint mutable references without aliasing UB.
                    let (a, b) = if ai < bi {
                        let (l, h) = arrays.split_at_mut(bi);
                        (&mut *l[ai], &mut *h[0])
                    } else {
                        let (l, h) = arrays.split_at_mut(ai);
                        (&mut *h[0], &mut *l[bi])
                    };
                    a.clear();
                    b.clear();
                    let mut curr = 0;
                    for _ in 0..size_a {
                        a.push_back(T::make(curr));
                        curr += 1;
                    }
                    for _ in 0..size_b {
                        b.push_back(T::make(curr));
                        curr += 1;
                    }
                    if do_move {
                        a.move_back(b);
                        check!(r, b.count() == 0);
                        check!(r, a.count() == size_a + size_b);
                        curr = 0;
                        for x in a.iter() {
                            check!(r, x.equals(curr));
                            curr += 1;
                        }
                    } else {
                        a.swap(b);
                        check!(r, b.count() == size_a);
                        check!(r, a.count() == size_b);
                        curr = 0;
                        for x in b.iter() {
                            check!(r, x.equals(curr));
                            curr += 1;
                        }
                        for x in a.iter() {
                            check!(r, x.equals(curr));
                            curr += 1;
                        }
                        a.swap_self();
                        curr = size_a as i32;
                        for x in a.iter() {
                            check!(r, x.equals(curr));
                            curr += 1;
                        }
                    }
                }
            }
        }
    }
}
// tests/TArrayTest.cpp:261
#[test]
fn TArray_Swap() {
    let mut r = Reporter::default();
    swap_and_move::<i32>(&mut r, false);
    swap_and_move::<MoveOnlyInt>(&mut r, false);
    r.finish();
}
// tests/TArrayTest.cpp:320
#[test]
fn TArray_Move() {
    let mut r = Reporter::default();
    swap_and_move::<i32>(&mut r, true);
    swap_and_move::<MoveOnlyInt>(&mut r, true);
    r.finish();
}
// tests/TArrayTest.cpp:338
#[test]
fn TArray_NoUnnecessaryAllocs() {
    let mut r = Reporter::default();
    {
        let a = SkTArray::<i32>::new();
        check!(r, a.capacity() == 0);
    }
    {
        let a = STArray::<10, i32>::new();
        check!(r, a.capacity() == 10);
    }
    {
        let a = SkTArray::<i32>::with_capacity(1);
        check!(r, a.capacity() >= 1);
    }
    {
        let a = SkTArray::<i32>::new();
        let mut b = SkTArray::new();
        b.clone_from(&a);
        check!(r, b.capacity() == 0);
    }
    {
        let a = STArray::<10, i32>::new();
        let mut b = SkTArray::new();
        b.assign_from(&a);
        check!(r, b.capacity() == 0);
    }
    {
        let a = SkTArray::<i32>::new();
        let b = a.clone();
        check!(r, b.capacity() == 0);
    }
    {
        let a = STArray::<10, i32>::new();
        let b = SkTArray::from_array(&a);
        check!(r, b.capacity() == 0);
    }
    {
        let a = SkTArray::<i32>::new();
        let b = a;
        check!(r, b.capacity() == 0);
    }
    {
        let a = STArray::<10, i32>::new();
        let b = SkTArray::move_from(a);
        check!(r, b.capacity() == 0);
    }
    {
        let a = SkTArray::<i32>::new();
        let mut b = SkTArray::new();
        b = a;
        check!(r, b.capacity() == 0);
    }
    {
        let a = STArray::<10, i32>::new();
        let mut b = SkTArray::new();
        b.move_assign(a);
        check!(r, b.capacity() == 0);
    }
    r.finish();
}
// tests/TArrayTest.cpp:410
#[test]
fn TArray_SelfAssignment() {
    let mut r = Reporter::default();
    self_assignment::<i32, SkTArray<i32>>(&mut r);
    self_assignment::<u16, STArray<3, u16>>(&mut r);
    r.finish();
}
// tests/TArrayTest.cpp:415
#[test]
fn FixedArray_SelfAssignment() {
    let mut r = Reporter::default();
    self_assignment::<i32, FixedArray<1, i32>>(&mut r);
    self_assignment::<u16, FixedArray<4, u16>>(&mut r);
    r.finish();
}
macro_rules! dynamic_variants {
    ($r:expr,$helper:ident) => {
        $helper::<i32, SkTArray<i32>>($r);
        $helper::<f64, SkTArray<f64>>($r);
        $helper::<TestClass, SkTArray<TestClass>>($r);
        $helper::<i32, STArray<1, i32>>($r);
        $helper::<i8, STArray<5, i8>>($r);
        $helper::<TestClass, STArray<7, TestClass>>($r);
        $helper::<f32, STArray<10, f32>>($r);
    };
}
// tests/TArrayTest.cpp:458
#[test]
fn TArray_Comparison() {
    let mut r = Reporter::default();
    dynamic_variants!(&mut r, comparison);
    r.finish();
}
// tests/TArrayTest.cpp:468
#[test]
fn FixedArray_Comparison() {
    let mut r = Reporter::default();
    comparison::<i32, FixedArray<15, i32>>(&mut r);
    comparison::<i8, FixedArray<20, i8>>(&mut r);
    comparison::<f32, FixedArray<25, f32>>(&mut r);
    r.finish();
}
// tests/TArrayTest.cpp:557; Rust move semantics replace both MEM_MOVE specializations, each still instantiated.
#[test]
fn TArray_Basic() {
    let mut r = Reporter::default();
    basic::<i32, SkTArray<i32>>(&mut r);
    basic::<i32, SkTArray<i32>>(&mut r);
    basic::<TestClass, SkTArray<TestClass>>(&mut r);
    r.finish();
}
// tests/TArrayTest.cpp:566
#[test]
fn FixedArray_Basic() {
    let mut r = Reporter::default();
    basic::<i8, FixedArray<5, i8>>(&mut r);
    basic::<i32, FixedArray<7, i32>>(&mut r);
    basic::<f64, FixedArray<100, f64>>(&mut r);
    r.finish();
}
// tests/TArrayTest.cpp:572
#[test]
fn TArray_Reserve() {
    let mut r = Reporter::default();
    reserve::<i32, SkTArray<i32>>(&mut r);
    reserve::<i32, STArray<1, i32>>(&mut r);
    reserve::<i32, STArray<2, i32>>(&mut r);
    reserve::<i32, STArray<16, i32>>(&mut r);
    reserve::<TestClass, SkTArray<TestClass>>(&mut r);
    reserve::<TestClass, STArray<1, TestClass>>(&mut r);
    reserve::<TestClass, STArray<2, TestClass>>(&mut r);
    reserve::<TestClass, STArray<16, TestClass>>(&mut r);
    r.finish();
}
fn construction_move<T: Value, A: Array<T>>(r: &mut Reporter) {
    construction::<T, A>(r, true);
}
// tests/TArrayTest.cpp:584
#[test]
fn TArray_Construction() {
    let mut r = Reporter::default();
    dynamic_variants!(&mut r, construction_move);
    r.finish();
}
// tests/TArrayTest.cpp:594
#[test]
fn FixedArray_Construction() {
    let mut r = Reporter::default();
    construction::<i32, FixedArray<15, i32>>(&mut r, false);
    construction::<i8, FixedArray<20, i8>>(&mut r, false);
    construction::<f32, FixedArray<25, f32>>(&mut r, false);
    r.finish();
}
// tests/TArrayTest.cpp:600
#[test]
fn TArray_InnerPush() {
    let mut r = Reporter::default();
    inner_push::<i32, SkTArray<i32>>(&mut r);
    inner_push::<i32, STArray<1, i32>>(&mut r);
    inner_push::<i32, STArray<99, i32>>(&mut r);
    inner_push::<i32, STArray<200, i32>>(&mut r);
    r.finish();
}
// tests/TArrayTest.cpp:607
#[test]
fn FixedArray_InnerPush() {
    let mut r = Reporter::default();
    inner_push::<i32, FixedArray<101, i32>>(&mut r);
    inner_push::<i16, FixedArray<150, i16>>(&mut r);
    inner_push::<f64, FixedArray<250, f64>>(&mut r);
    r.finish();
}
#[derive(Default)]
struct EmplaceStruct {
    value: i32,
}
fn inner_emplace<A: EmplaceArray>(r: &mut Reporter) {
    let mut a = A::new();
    a.emplace(12345);
    for _ in 0..50 {
        let value = a.slice().first().unwrap().value;
        a.emplace(value);
    }
    for _ in 0..50 {
        let value = a.slice().last().unwrap().value;
        a.emplace(value);
    }
    check!(r, a.slice().len() == 101);
    check!(r, a.slice().iter().all(|s| s.value == 12345));
}
trait EmplaceArray {
    fn new() -> Self;
    fn emplace(&mut self, v: i32);
    fn slice(&self) -> &[EmplaceStruct];
}
impl EmplaceArray for SkTArray<EmplaceStruct> {
    fn new() -> Self {
        Self::new()
    }
    fn emplace(&mut self, v: i32) {
        self.emplace_back(EmplaceStruct { value: v });
    }
    fn slice(&self) -> &[EmplaceStruct] {
        self.as_slice()
    }
}
impl<const N: usize> EmplaceArray for STArray<N, EmplaceStruct> {
    fn new() -> Self {
        Self::new()
    }
    fn emplace(&mut self, v: i32) {
        self.emplace_back(EmplaceStruct { value: v });
    }
    fn slice(&self) -> &[EmplaceStruct] {
        self.as_slice()
    }
}
// tests/TArrayTest.cpp:613
#[test]
fn TArray_InnerEmplace() {
    let mut r = Reporter::default();
    inner_emplace::<SkTArray<EmplaceStruct>>(&mut r);
    inner_emplace::<STArray<1, EmplaceStruct>>(&mut r);
    inner_emplace::<STArray<99, EmplaceStruct>>(&mut r);
    inner_emplace::<STArray<200, EmplaceStruct>>(&mut r);
    r.finish();
}
// Cross-type conversions are production APIs, never copies implemented in test support.
trait Compatible<T: Value, A: Array<T>>: Array<T> {
    fn copy_from_array(a: &A) -> Self;
    fn assign_array(&mut self, a: &A);
    fn move_array(a: &mut A) -> Self;
    fn move_assign_array(&mut self, a: &mut A);
}
macro_rules! compatible {
    ($t:ty,$a:ty,$b:ty) => {
        impl Compatible<$t, $a> for $b {
            fn copy_from_array(a: &$a) -> Self {
                Self::from_array(a)
            }
            fn assign_array(&mut self, a: &$a) {
                self.assign_from(a);
            }
            fn move_array(a: &mut $a) -> Self {
                Self::move_from(std::mem::take(a))
            }
            fn move_assign_array(&mut self, a: &mut $a) {
                self.move_assign(std::mem::take(a));
            }
        }
    };
}
compatible!(i32,STArray<1,i32>,SkTArray<i32>);
compatible!(i8,STArray<5,i8>,SkTArray<i8>);
compatible!(f32,STArray<10,f32>,SkTArray<f32>);
compatible!(i32,SkTArray<i32>,STArray<1,i32>);
compatible!(i8,SkTArray<i8>,STArray<5,i8>);
compatible!(f32,SkTArray<f32>,STArray<10,f32>);
compatible!(u8,STArray<10,u8>,STArray<1,u8>);
compatible!(i64,STArray<1,i64>,STArray<10,i64>);
compatible!(f64,STArray<3,f64>,STArray<4,f64>);
compatible!(i16,STArray<2,i16>,STArray<1,i16>);
fn compatibility<T: Value, A: Array<T>, B: Compatible<T, A>>(r: &mut Reporter) {
    let mut a = A::default();
    a.add_value(T::value(1));
    a.add_value(T::value(2));
    a.add_value(T::value(3));
    let mut a2 = a.clone();
    let copy = B::copy_from_array(&a);
    check!(r, a.size() == 3);
    check!(r, a.slice()[0] == T::value(1));
    check!(r, a.slice()[1] == T::value(2));
    check!(r, a.slice()[2] == T::value(3));
    check!(r, copy.size() == 3);
    check!(r, copy.slice()[0] == T::value(1));
    check!(r, copy.slice()[1] == T::value(2));
    check!(r, copy.slice()[2] == T::value(3));
    let mut assignment = B::default();
    assignment.assign_array(&a);
    check!(r, a.size() == 3);
    check!(r, a.slice()[0] == T::value(1));
    check!(r, a.slice()[1] == T::value(2));
    check!(r, a.slice()[2] == T::value(3));
    check!(r, assignment.size() == 3);
    check!(r, assignment.slice()[0] == T::value(1));
    check!(r, assignment.slice()[1] == T::value(2));
    check!(r, assignment.slice()[2] == T::value(3));
    let moved = B::move_array(&mut a);
    check!(r, a.empty());
    check!(r, moved.size() == 3);
    check!(r, moved.slice()[0] == T::value(1));
    check!(r, moved.slice()[1] == T::value(2));
    check!(r, moved.slice()[2] == T::value(3));
    let mut assign_moved = B::default();
    assign_moved.move_assign_array(&mut a2);
    check!(r, a2.empty());
    check!(r, assign_moved.size() == 3);
    check!(r, assign_moved.slice()[0] == T::value(1));
    check!(r, assign_moved.slice()[1] == T::value(2));
    check!(r, assign_moved.slice()[2] == T::value(3));
}
// tests/TArrayTest.cpp:620
#[test]
fn TArray_STArrayCompatibility() {
    let mut r = Reporter::default();
    compatibility::<i32, STArray<1, i32>, SkTArray<i32>>(&mut r);
    compatibility::<i8, STArray<5, i8>, SkTArray<i8>>(&mut r);
    compatibility::<f32, STArray<10, f32>, SkTArray<f32>>(&mut r);
    compatibility::<i32, SkTArray<i32>, STArray<1, i32>>(&mut r);
    compatibility::<i8, SkTArray<i8>, STArray<5, i8>>(&mut r);
    compatibility::<f32, SkTArray<f32>, STArray<10, f32>>(&mut r);
    compatibility::<u8, STArray<10, u8>, STArray<1, u8>>(&mut r);
    compatibility::<i64, STArray<1, i64>, STArray<10, i64>>(&mut r);
    compatibility::<f64, STArray<3, f64>, STArray<4, f64>>(&mut r);
    compatibility::<i16, STArray<2, i16>, STArray<1, i16>>(&mut r);
    r.finish();
}
// tests/TArrayTest.cpp:633; entire original body is #if 0 and remains disabled.
#[test]
fn TArray_BoundsCheck() {}
// tests/TArrayTest.cpp:643..691; SK_SANITIZE_ADDRESS must be selected explicitly by the runner.
// rustc does not expose a stable sanitizer cfg: use RUSTFLAGS='--cfg m03_asan -Zsanitizer=address'.
#[cfg(m03_asan)]
mod asan {
    use super::*;
    unsafe extern "C" {
        fn __asan_address_is_poisoned(p: *const u8) -> i32;
    }
    fn verify<T: Value, A: Array<T>>(r: &mut Reporter, a: &A) {
        let allocated = a.size() * std::mem::size_of::<T>();
        let capacity = a.cap() * std::mem::size_of::<T>();
        let data = a.slice().as_ptr().cast::<u8>();
        for index in 0..allocated {
            check!(r, unsafe {
                __asan_address_is_poisoned(data.add(index)) == 0
            });
        }
        check!(r, capacity >= 8);
        for index in allocated..capacity {
            check!(r, unsafe {
                __asan_address_is_poisoned(data.add(index)) != 0
            });
        }
    }
    fn poison<T: Value, A: Array<T>>(r: &mut Reporter) {
        let mut a = A::default();
        for _ in 0..20 {
            a.add_value(T::default());
            verify(r, &a);
        }
        for _ in 0..20 {
            a.pop();
            verify(r, &a);
        }
        for _ in 0..20 {
            a.reserve_count(a.cap() + 3);
            verify(r, &a);
        }
        a.clear_values();
        verify(r, &a);
    }
    #[test]
    fn TArray_ASANPoisoning() {
        let mut r = Reporter::default();
        poison::<i32, SkTArray<i32>>(&mut r);
        poison::<f64, STArray<1, f64>>(&mut r);
        poison::<i8, STArray<2, i8>>(&mut r);
        poison::<TestClass, STArray<16, TestClass>>(&mut r);
        r.finish();
    }
}
