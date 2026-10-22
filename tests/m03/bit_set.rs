// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// Reference: b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
// tests/BitSetTest.cpp:14
#[test]
fn BitSet() {
    let mut r = Reporter::default();
    let mut set0 = SkBitSet::new(65536);
    check!(r, set0.count() == 65536);
    check!(r, set0.test(0) == false);
    check!(r, set0.test(32767) == false);
    check!(r, set0.test(65535) == false);
    check!(r, set0.find_first().is_none());

    set0.set(22);
    check!(r, set0.test(22) == true);
    check!(r, set0.find_first().is_some());
    check!(r, set0.find_first().unwrap() == 22);
    set0.set(24);
    check!(r, set0.test(24) == true);
    check!(r, set0.find_first().unwrap() == 22);
    set0.set(35); // on a different DWORD
    check!(r, set0.test(35) == true);
    check!(r, set0.find_first().unwrap() == 22);
    check!(r, set0.test(24) == true);
    check!(r, set0.test(35) == true);
    set0.set(21);
    check!(r, set0.test(21) == true);
    check!(r, set0.find_first().unwrap() == 21);
    set0.clear(21);
    check!(r, set0.test(21) == false);
    check!(r, set0.find_first().unwrap() == 22);

    let mut data = Vec::new();
    set0.for_each_set_index(|v| data.push(v));

    check!(r, data.len() == 3);
    check!(r, data[0] == 22);
    check!(r, data[1] == 24);
    check!(r, data[2] == 35);

    let mut set1 = SkBitSet::new(65536);
    set1.set(12345);
    check!(r, set0.test(12345) == false);
    check!(r, set1.test(12345) == true);
    check!(r, set1.test(22) == false);
    check!(r, set0.test(35) == true);

    set0.reset();
    check!(r, set0.find_first().is_none());
    check!(r, set0.test(1234) == false);

    set0.set_all();
    check!(r, set0.find_first_unset().is_none());
    check!(r, set0.test(5678) == true);
    r.finish();
}

// tests/BitSetTest.cpp:65
#[test]
fn BitSetComparisonSizeMatching() {
    let mut r = Reporter::default();
    // Verify that bitsets of differing sizes are never equal, even if the contents match.
    let mut bitsetA = SkBitSet::new(123);
    let mut bitsetB = SkBitSet::new(123);
    check!(r, bitsetA == bitsetB);
    check!(r, !(bitsetA != bitsetB));

    let mut bitsetC = SkBitSet::new(345);
    check!(r, bitsetA != bitsetC);
    check!(r, !(bitsetA == bitsetC));

    let mut bitsetD = SkBitSet::new(67);
    check!(r, bitsetA != bitsetD);
    check!(r, !(bitsetA == bitsetD));
    r.finish();
}

// tests/BitSetTest.cpp:81
#[test]
fn BitSetComparisonBitMatching() {
    let mut r = Reporter::default();
    let mut random = SkRandom::new();
    for _trial in 0..500 {
        // Make two identical bitsets.
        let size = random.next_range_u(1, 1000) as usize;
        let mut bitsetA = SkBitSet::new(size);
        let mut bitsetB = SkBitSet::new(size);

        for _index in 0..size {
            if random.next_bool() {
                bitsetA.set(_index);
                bitsetB.set(_index);
            }
        }

        // Verify that the sets are equal.
        check!(r, bitsetA == bitsetB);
        check!(r, !(bitsetA != bitsetB));

        // Flip some bits randomly. Always flip an odd number of bits to ensure that our random bit
        // flips don't all cancel each other out.
        let numBitsToFlip = random.next_u_less_than(size as u32) | 1;
        for _index in 0..numBitsToFlip {
            let bitToFlip = random.next_u_less_than(size as u32) as usize;
            if bitsetA.test(bitToFlip) {
                bitsetA.clear(bitToFlip);
            } else {
                bitsetA.set(bitToFlip);
            }
        }

        // Verify that the sets are no longer equal.
        check!(r, bitsetA != bitsetB);
        check!(r, !(bitsetA == bitsetB));
    }
    r.finish();
}
