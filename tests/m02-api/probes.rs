// Minimal compile probes for required API observations. These are NOT faithful
// original bodies or runtime PASS results. See API correspondence in M02.md.
use rust_skia::*;
fn main() {
    let map = THashMap::<i32, f64>::new();
    let _copy = map.clone();
    let mut set = THashSet::<i32>::new();
    let _ = set.approx_bytes_used();
    let _ = set.find(&1);
    let _copy = set.clone();
    set.reserve(40);
    let _deque = SkTDeque::<i32>::with_capacity(10);
    let list = TInternalLList::<i32>::new();
    let _ = list.iter();
    let _ = list.iter_rev();
    let p = RawPtr::<i32>::null();
    let mut ordered = std::collections::BTreeSet::new();
    ordered.insert(p);
    let mut hashed = std::collections::HashSet::new();
    hashed.insert(p);
    let mut a = TBlockList::<i32, 1>::new();
    let mut b = TBlockList::<i32, 5>::new();
    a.concat(&mut b);
    let mut a = AutoSTArray::<i32, 10>::new();
    a.reset(1);
    let _ = a.size();
    let mut s = SkString::new();
    s.append_s64(0x1000000, 15);
    s.append_u64(0x1000000, 15);
    let _pool = SkExecutors::<SkFunctionalExecutor>::make_multi_list_fifo_thread_pool(2, 4, false);
    let group = SkTaskGroup::new();
    group.add_priority(|| {}, 0);
}
fn pointer_table() {
    let _table = THashTable::<*mut i32, i32>::new();
}
fn skstring_hash_set() {
    let _ = THashSet::<SkString>::new();
}
