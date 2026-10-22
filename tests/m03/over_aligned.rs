// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// Reference: b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
// tests/OverAlignedTest.cpp:18
#[test]
fn OverAligned() {
    let mut r = Reporter::default();
    let mut rand = SkRandom::new();
    for _ in 0..1000 {
        unsafe {
            let p = sk_malloc_throw(rand.next_range_u(0, 100) as usize);
            check!(r, (p as usize) & 7 == 0);
            sk_free(p);
        }
    }
    r.finish();
}
