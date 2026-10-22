use super::*;
#[test]
fn HashMapCtor() {
    let mut r = Reporter::default();
    let mut m = THashMap::new();
    for (k, v) in [(1, "one"), (2, "two"), (3, "three"), (4, "four")] {
        m.set(k, v);
    }
    check!(r, m.count() == 4);
    check!(
        r,
        m.approx_bytes_used() >= 4 * (std::mem::size_of::<i32>() + std::mem::size_of::<&str>())
    );
    for (k, v) in [(1, "one"), (2, "two"), (3, "three"), (4, "four")] {
        let found = m.find(&k);
        check!(r, found.is_some());
        check!(r, *found.unwrap() == v);
    }
    check!(r, m.find(&5).is_none());
    check!(r, m.find(&6).is_none());
    r.finish();
}
#[test]
fn HashMapCtorOneElem() {
    let mut r = Reporter::default();
    let mut m = THashMap::new();
    m.set(1, "one");
    check!(r, m.count() == 1);
    check!(
        r,
        m.approx_bytes_used() >= std::mem::size_of::<i32>() + std::mem::size_of::<&str>()
    );
    let f = m.find(&1);
    check!(r, f.is_some());
    check!(r, *f.unwrap() == "one");
    check!(r, m.find(&2).is_none());
    m.set(2, "two");
    let f = m.find(&2);
    check!(r, f.is_some());
    check!(r, *f.unwrap() == "two");
    check!(r, m.find(&3).is_none());
    r.finish();
}
#[test]
fn HashMapSwap() {
    let mut r = Reporter::default();
    let mut a = THashMap::new();
    let mut b = THashMap::new();
    for (k, v) in [(1, "one"), (2, "two"), (3, "three"), (4, "four")] {
        a.set(k, v);
    }
    for (k, v) in [(1, "ichi"), (2, "ni"), (3, "san")] {
        b.set(k, v);
    }
    a.swap(&mut b);
    check!(r, a.count() == 3);
    check!(r, a.find(&1) == Some(&"ichi"));
    check!(r, a.find(&2) == Some(&"ni"));
    check!(r, a.find(&3) == Some(&"san"));
    check!(r, b.count() == 4);
    check!(r, b.find(&1) == Some(&"one"));
    check!(r, b.find(&2) == Some(&"two"));
    check!(r, b.find(&3) == Some(&"three"));
    check!(r, b.find(&4) == Some(&"four"));
    a.swap(&mut THashMap::new());
    check!(r, a.empty());
    r.finish();
}
#[test]
fn HashSetSwap() {
    let mut r = Reporter::default();
    let mut a = THashSet::new();
    let mut b = THashSet::new();
    for v in ["one", "two", "three", "four"] {
        a.add(&v);
    }
    for v in ["ichi", "ni", "san"] {
        b.add(&v);
    }
    a.swap(&mut b);
    check!(r, a.count() == 3);
    check!(r, a.contains(&"ichi"));
    check!(r, a.contains(&"ni"));
    check!(r, a.contains(&"san"));
    check!(r, b.count() == 4);
    check!(r, b.contains(&"one"));
    check!(r, b.contains(&"two"));
    check!(r, b.contains(&"three"));
    check!(r, b.contains(&"four"));
    a.swap(&mut THashSet::new());
    check!(r, a.empty());
    r.finish();
}
#[derive(Eq)]
struct Counter {
    id: u32,
    count: std::rc::Rc<std::cell::Cell<u32>>,
}
impl PartialEq for Counter {
    fn eq(&self, o: &Self) -> bool {
        self.id == o.id
    }
}
impl std::hash::Hash for Counter {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        h.write_u32(0);
    }
}
impl Clone for Counter {
    fn clone(&self) -> Self {
        self.count.set(self.count.get() + 1);
        Self {
            id: self.id,
            count: self.count.clone(),
        }
    }
}
#[test]
fn HashSetCopyCounter() {
    let mut r = Reporter::default();
    let c = std::rc::Rc::new(std::cell::Cell::new(0));
    let c1 = Counter {
        id: 1,
        count: c.clone(),
    };
    let c2 = Counter {
        id: 2,
        count: c.clone(),
    };
    let mut s = THashSet::new();
    check!(r, c.get() == 0);
    s.add(&c1);
    check!(r, c.get() == 1);
    check!(r, s.contains(&c1));
    check!(r, c.get() == 1);
    s.add(&c1);
    check!(r, c.get() == 2);
    s.add(&c2);
    check!(r, c.get() == 3);
    check!(r, s.contains(&c1));
    check!(r, s.contains(&c2));
    check!(r, c.get() == 3);
    s.add(&c1);
    s.add(&c2);
    check!(r, c.get() == 5);
    r.finish();
}

// Remaining original HashTest.cpp bodies, including all iteration passes.
#[test]
fn HashMap() {
    let mut r = Reporter::default();
    let mut map = THashMap::new();
    map.set(3, 4.0);
    check!(r, map.count() == 1);
    check!(r, map.approx_bytes_used() > 0);
    let f = map.find(&3);
    check!(r, f.is_some());
    check!(r, *f.unwrap() == 4.0);
    map.foreach_mut(|key, d| *d = -*key as f64);
    let mut count = 0;
    map.foreach(|_, _| count += 1);
    check!(r, count == 1);
    let f = map.find(&3);
    check!(r, f.is_some());
    check!(r, *f.unwrap() == -3.0);
    check!(r, map.find(&2).is_none());
    for i in 0..20 {
        map.set(i, 2.0 * i as f64);
    }
    map.foreach(|k, v| check!(r, (*k * 2) as f64 == *v));
    map.foreach_mut(|k, v| check!(r, (*k * 2) as f64 == *v));
    map.foreach(|k, v| check!(r, (*k * 2) as f64 == *v));
    for (k, v) in map.iter() {
        check!(r, (*k * 2) as f64 == *v);
    }
    for (k, v) in map.iter() {
        check!(r, (*k * 2) as f64 == *v);
    }
    let cmap = &map;
    for (k, v) in cmap.iter() {
        check!(r, (*k * 2) as f64 == *v);
    }
    for (k, v) in cmap.iter() {
        check!(r, (*k * 2) as f64 == *v);
    }
    for (number, times_two) in cmap.iter() {
        check!(r, (*number * 2) as f64 == *times_two);
    }
    let mut clone = map.clone();
    for i in 0..20 {
        let f = map.find(&i);
        check!(r, f.is_some());
        check!(r, *f.unwrap() == i as f64 * 2.0);
        let f = clone.find(&i);
        check!(r, f.is_some());
        check!(r, *f.unwrap() == i as f64 * 2.0);
    }
    for i in 20..40 {
        check!(r, map.find(&i).is_none());
        check!(r, clone.find(&i).is_none());
    }
    check!(r, map.count() == 20);
    check!(r, clone.count() == 20);
    for i in 0..10 {
        map.remove(&i);
    }
    for i in 0..20 {
        check!(r, map.find(&i).is_none() == (i < 10));
        check!(r, *clone.find(&i).unwrap() == i as f64 * 2.0);
    }
    check!(r, map.count() == 10);
    check!(r, clone.count() == 20);
    map.reset();
    check!(r, map.count() == 0);
    check!(r, clone.count() == 20);
    std::hint::black_box(&clone);
    clone = map;
    check!(r, clone.count() == 0);
    {
        let mut m = THashMap::new();
        let rc = sk_make_sp(SkRefCnt::new());
        check!(r, rc.unique());
        m.set(0, rc.clone());
        check!(r, m.count() == 1);
        check!(r, !rc.unique());
        m.remove(&0);
        check!(r, m.count() == 0);
        check!(r, rc.unique());
    }
    r.finish();
}
#[test]
fn HashSetCtor() {
    let mut r = Reporter::default();
    let mut s = THashSet::new();
    for v in ["one", "two", "three", "four"] {
        s.add(&v);
    }
    check!(r, s.count() == 4);
    check!(r, s.approx_bytes_used() >= 4 * std::mem::size_of::<&str>());
    for v in ["one", "two", "three", "four"] {
        check!(r, s.contains(&v));
    }
    check!(r, !s.contains(&"five"));
    check!(r, !s.contains(&"six"));
    r.finish();
}
#[test]
fn HashSetCtorOneElem() {
    let mut r = Reporter::default();
    let mut s = THashSet::new();
    s.add(&"one");
    check!(r, s.count() == 1);
    check!(r, s.approx_bytes_used() >= std::mem::size_of::<&str>());
    check!(r, s.contains(&"one"));
    check!(r, !s.contains(&"two"));
    s.add(&"two");
    check!(r, s.contains(&"one"));
    check!(r, s.contains(&"two"));
    check!(r, !s.contains(&"three"));
    r.finish();
}
fn test_hash_set<T: Clone + Eq + std::hash::Hash>(
    r: &mut Reporter,
    make: impl Fn(&'static str) -> T,
) {
    let mut s = THashSet::new();
    s.add(&make("Hello"));
    s.add(&make("World"));
    check!(r, s.count() == 2);
    check!(r, s.contains(&make("Hello")));
    check!(r, s.contains(&make("World")));
    check!(r, !s.contains(&make("Goodbye")));
    check!(r, s.find(&make("Hello")).is_some());
    check!(r, *s.find(&make("Hello")).unwrap() == make("Hello"));
    for _ in 0..5 {
        for entry in s.iter() {
            check!(r, *entry == make("Hello") || *entry == make("World"));
        }
    }
    let mut clone = s.clone();
    check!(r, clone.count() == 2);
    check!(r, clone.contains(&make("Hello")));
    check!(r, clone.contains(&make("World")));
    check!(r, !clone.contains(&make("Goodbye")));
    check!(r, clone.find(&make("Hello")).is_some());
    check!(r, *clone.find(&make("Hello")).unwrap() == make("Hello"));
    s.remove(&make("Hello"));
    check!(r, !s.contains(&make("Hello")));
    check!(r, s.count() == 1);
    check!(r, clone.contains(&make("Hello")));
    check!(r, clone.count() == 2);
    s.reset();
    check!(r, s.count() == 0);
    std::hint::black_box(&clone);
    clone = s;
    check!(r, clone.count() == 0);
}
#[test]
fn HashSetWithSkString() {
    let mut r = Reporter::default();
    test_hash_set(&mut r, SkString::from_str);
    r.finish();
}
#[test]
fn HashSetWithStdString() {
    let mut r = Reporter::default();
    test_hash_set(&mut r, |s| s.to_owned());
    r.finish();
}
#[test]
fn HashSetWithStdStringView() {
    let mut r = Reporter::default();
    test_hash_set(&mut r, |s| s);
    r.finish();
}
#[test]
fn HashFindOrNull() {
    #[allow(dead_code)]
    struct Entry {
        key: i32,
        val: i32,
    }
    let mut r = Reporter::default();
    let mut table = THashTable::<*mut Entry, i32>::new();
    check!(r, table.find_or_null(&7).is_none());
    let mut seven = Entry { key: 7, val: 24 };
    table.set(seven.key, &mut seven);
    check!(
        r,
        table.find_or_null(&7) == Some(&std::ptr::addr_of_mut!(seven))
    );
    r.finish();
}
fn count_cap(r: &mut Reporter, s: &THashSet<i32>, count: usize, cap: usize) {
    eprintln!(
        "hash expected count={count} capacity={cap} actual count={} bytes={}",
        s.count(),
        s.approx_bytes_used()
    );
    check!(r, s.count() == count);
    check!(r, s.approx_bytes_used() == 8 * cap);
}
#[test]
fn HashTableGrowsAndShrinks() {
    let mut r = Reporter::default();
    let mut s = THashSet::new();
    // Add and remove some elements to test basic growth and shrink patterns.
    count_cap(&mut r, &s, 0, 0);
    s.add(&1);
    count_cap(&mut r, &s, 1, 4);
    s.add(&2);
    count_cap(&mut r, &s, 2, 4);
    s.add(&3);
    count_cap(&mut r, &s, 3, 4);
    s.add(&4);
    count_cap(&mut r, &s, 4, 8);

    s.remove(&4);
    count_cap(&mut r, &s, 3, 8);
    s.remove(&3);
    count_cap(&mut r, &s, 2, 4);
    s.remove(&2);
    count_cap(&mut r, &s, 1, 4);
    s.remove(&1);
    count_cap(&mut r, &s, 0, 4);

    s.add(&1);
    count_cap(&mut r, &s, 1, 4);
    s.add(&2);
    count_cap(&mut r, &s, 2, 4);
    s.add(&3);
    count_cap(&mut r, &s, 3, 4);
    s.add(&4);
    count_cap(&mut r, &s, 4, 8);

    // Test that reserve() will preallocate space, rounded up to 2^n.
    s.reserve(40);
    count_cap(&mut r, &s, 4, 64);
    s.reserve(48);
    count_cap(&mut r, &s, 4, 64);
    s.reserve(49);
    count_cap(&mut r, &s, 4, 128);

    // Test that reserve() will not shrink the capacity.
    s.reserve(48);
    count_cap(&mut r, &s, 4, 128);
    s.reserve(12);
    count_cap(&mut r, &s, 4, 128);
    s.reserve(1);
    count_cap(&mut r, &s, 4, 128);

    // Test that adding elements will maintain the reserved capacity.
    s.add(&5);
    count_cap(&mut r, &s, 5, 128);
    s.add(&6);
    count_cap(&mut r, &s, 6, 128);
    s.add(&7);
    count_cap(&mut r, &s, 7, 128);
    s.add(&8);
    count_cap(&mut r, &s, 8, 128);

    // Removing elements, on the other hand, _will_ allow the container to shrink.
    s.remove(&4);
    count_cap(&mut r, &s, 7, 64);
    s.remove(&5);
    count_cap(&mut r, &s, 6, 32);
    s.remove(&6);
    count_cap(&mut r, &s, 5, 16);
    s.remove(&7);
    count_cap(&mut r, &s, 4, 8);
    s.remove(&8);
    count_cap(&mut r, &s, 3, 8);
    s.add(&4);
    count_cap(&mut r, &s, 4, 8);

    // Add and remove single elements repeatedly to test hysteresis
    // avoids reallocating these small tables all the time.
    for _ in 0..10 {
        s.add(&5);
        count_cap(&mut r, &s, 5, 8);
        s.remove(&5);
        count_cap(&mut r, &s, 4, 8);
    }

    s.remove(&4);
    count_cap(&mut r, &s, 3, 8);
    for _ in 0..10 {
        s.add(&4);
        count_cap(&mut r, &s, 4, 8);
        s.remove(&4);
        count_cap(&mut r, &s, 3, 8);
    }

    s.remove(&3);
    count_cap(&mut r, &s, 2, 4);
    for _ in 0..10 {
        s.add(&4);
        count_cap(&mut r, &s, 3, 4);
        s.remove(&4);
        count_cap(&mut r, &s, 2, 4);
    }

    s.remove(&2);
    count_cap(&mut r, &s, 1, 4);
    for _ in 0..10 {
        s.add(&2);
        count_cap(&mut r, &s, 2, 4);
        s.remove(&2);
        count_cap(&mut r, &s, 1, 4);
    }

    r.finish();
}
