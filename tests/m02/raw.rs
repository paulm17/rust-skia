use super::*;
#[test]
fn RawPtr_DisableDanglingPtrDetection() {
    let mut owner = Box::new(42);
    let ptr = RawPtr::new(&mut *owner);
    let _ = ptr;
    drop(owner);
}
#[test]
fn RawPtr_DefaultConstructor() {
    let mut r = Reporter::default();
    let p = RawPtr::<i32>::default();
    check!(r, p == std::ptr::null_mut());
    check!(r, p.is_null());
    r.finish();
}
#[test]
fn RawPtr_NullptrConstructor() {
    let mut r = Reporter::default();
    let p = RawPtr::<i32>::null();
    check!(r, p == std::ptr::null_mut());
    check!(r, p.is_null());
    r.finish();
}
#[test]
fn RawPtr_RawPointerConstructor() {
    let mut r = Reporter::default();
    let mut val = 42;
    let p = RawPtr::new(&mut val);
    check!(r, p == std::ptr::addr_of_mut!(val));
    check!(r, !p.is_null());
    check!(r, *p == 42);
    r.finish();
}
#[test]
fn RawPtr_CopyAndMove() {
    let mut r = Reporter::default();
    let mut val = 42;
    let p1 = RawPtr::new(&mut val);
    let mut p2 = p1;
    check!(r, p2 == std::ptr::addr_of_mut!(val));
    check!(r, p1 == std::ptr::addr_of_mut!(val));
    let p3 = RawPtr::new(p2.take());
    check!(r, p3 == std::ptr::addr_of_mut!(val));
    check!(r, p2.is_null());
    r.finish();
}
#[test]
fn RawPtr_Assignment() {
    let mut r = Reporter::default();
    let mut v1 = 42;
    let mut v2 = 84;
    let mut p1 = RawPtr::new(&mut v1);
    let p2 = RawPtr::new(&mut v2);
    std::hint::black_box(p1);
    p1 = p2;
    check!(r, p1 == std::ptr::addr_of_mut!(v2));
    p1 = RawPtr::null();
    check!(r, p1.is_null());
    p1 = RawPtr::new(&mut v1);
    check!(r, p1 == std::ptr::addr_of_mut!(v1));
    r.finish();
}
#[test]
fn RawPtr_SelfAssignment() {
    let mut r = Reporter::default();
    let mut val = 42;
    let mut p = RawPtr::new(&mut val);
    let pp = &p as *const RawPtr<i32>;
    p = unsafe { *pp };
    check!(r, p == std::ptr::addr_of_mut!(val));
    p = unsafe { *pp };
    check!(r, p == std::ptr::addr_of_mut!(val));
    r.finish();
}
#[repr(C)]
struct Base {
    base_val: i32,
}
#[repr(C)]
struct Derived {
    base: Base,
    derived_val: i32,
}
#[test]
fn RawPtr_Upcasting() {
    let mut r = Reporter::default();
    let mut d = Derived {
        base: Base { base_val: 1 },
        derived_val: 2,
    };
    let pd = RawPtr::new(&mut d);
    let pb = pd.cast::<Base>();
    check!(r, pb.get() == std::ptr::addr_of_mut!(d.base));
    check!(r, pb.base_val == 1);
    let pb2 = pd.cast::<Base>();
    check!(r, pb2.get() == std::ptr::addr_of_mut!(d.base));
    let mut pm = RawPtr::new(&mut d);
    let bm = RawPtr::new(pm.take()).cast::<Base>();
    check!(r, bm.get() == std::ptr::addr_of_mut!(d.base));
    check!(r, pm.is_null());
    let mut pm2 = RawPtr::new(&mut d);
    let bm2 = RawPtr::new(pm2.take()).cast::<Base>();
    check!(r, bm2.get() == std::ptr::addr_of_mut!(d.base));
    check!(r, pm2.is_null());
    r.finish();
}
#[test]
fn RawPtr_DereferenceAndMemberAccess() {
    struct Foo {
        x: i32,
    }
    let mut r = Reporter::default();
    let mut foo = Foo { x: 42 };
    let p = RawPtr::new(&mut foo);
    check!(r, p.x == 42);
    check!(r, (*p).x == 42);
    check!(r, p == std::ptr::addr_of_mut!(foo));
    r.finish();
}
#[test]
fn RawPtr_PointerArithmetic() {
    let mut r = Reporter::default();
    let mut arr = [10, 20, 30];
    let mut expected = arr.as_mut_ptr();
    let mut p = RawPtr::new(expected);
    check!(r, p == expected);
    unsafe {
        expected = expected.add(1);
    }
    let old_address = &p as *const _;
    p += 1;
    let p_ref = &p;
    check!(r, std::ptr::eq(p_ref, old_address));
    check!(r, p == expected);
    let expected_old = expected;
    unsafe {
        expected = expected.add(1);
    }
    let mut p_old = p;
    p += 1;
    check!(r, p_old == expected_old);
    check!(r, p == expected);
    unsafe {
        expected = expected.sub(1);
    }
    p -= 1;
    let p_ref2 = &p;
    check!(r, std::ptr::eq(p_ref2, old_address));
    check!(r, p == expected);
    let expected_old = expected;
    unsafe {
        expected = expected.sub(1);
    }
    std::hint::black_box(p_old);
    p_old = p;
    p -= 1;
    check!(r, p_old == expected_old);
    check!(r, p == expected);
    unsafe {
        check!(r, (p + 1) == expected.add(1));
        check!(r, (p + 2) == expected.add(2));
        let p2 = p + 2;
        let e2 = expected.add(2);
        check!(r, (p2 - 1) == e2.sub(1));
        check!(r, (p2 - 2) == e2.sub(2));
        check!(r, p2 - p == e2.offset_from(expected));
    }
    r.finish();
}
#[test]
fn RawPtr_EqualityComparisons() {
    let mut r = Reporter::default();
    let mut v1 = 42;
    let mut v2 = 84;
    let p1 = RawPtr::new(&mut v1);
    let p2 = RawPtr::new(&mut v2);
    let n = RawPtr::<i32>::null();
    let a = std::ptr::addr_of_mut!(v1);
    let b = std::ptr::addr_of_mut!(v2);
    check!(r, p1 == p1);
    check!(r, !(p1 == p2));
    check!(r, p1 != p2);
    check!(r, p1 == a);
    check!(r, a == p1);
    check!(r, !(p1 == b));
    check!(r, !(b == p1));
    check!(r, n == std::ptr::null_mut());
    check!(r, std::ptr::null_mut() == n);
    check!(r, !(p1 == std::ptr::null_mut()));
    check!(r, !(std::ptr::null_mut() == p1));
    r.finish();
}
#[test]
fn RawPtr_RelationalComparisons() {
    let mut r = Reporter::default();
    let mut arr = [42, 84];
    let p0 = RawPtr::new(&mut arr[0]);
    let p1 = RawPtr::new(&mut arr[1]);
    check!(r, p0 < p1);
    check!(r, p0 <= p1);
    check!(r, p1 > p0);
    check!(r, p1 >= p0);
    check!(r, !(p1 < p0));
    check!(r, !(p1 <= p0));
    check!(r, !(p0 > p1));
    check!(r, !(p0 >= p1));
    let a = std::ptr::addr_of_mut!(arr[1]);
    check!(r, p0 < a);
    check!(r, p0 <= a);
    check!(r, a > p0.get());
    check!(r, a >= p0.get());
    r.finish();
}
#[test]
fn RawPtr_Void() {
    let mut r = Reporter::default();
    let mut val = 42;
    let a = std::ptr::addr_of_mut!(val).cast::<std::ffi::c_void>();
    let p = RawPtr::new(a);
    check!(r, p == a);
    check!(r, !p.is_null());
    let cp = RawPtr::new(a);
    check!(r, cp == a);
    r.finish();
}
#[test]
fn RawPtr_Stl() {
    let mut r = Reporter::default();
    let mut a = 1;
    let mut b = 2;
    let pa = RawPtr::new(&mut a);
    let pb = RawPtr::new(&mut b);
    let mut set = std::collections::BTreeSet::new();
    set.insert(pa);
    set.insert(pb);
    check!(r, set.len() == 2);
    check!(r, set.contains(&pa));
    check!(r, set.contains(&RawPtr::new(std::ptr::addr_of_mut!(a))));
    let mut uset = std::collections::HashSet::new();
    uset.insert(pa);
    uset.insert(pb);
    check!(r, uset.len() == 2);
    check!(r, uset.contains(&pa));
    check!(r, pa.to_address() == std::ptr::addr_of_mut!(a));
    r.finish();
}
