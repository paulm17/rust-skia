// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// Reference: b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
// tests/SafeMathTest.cpp:14
#[test]
fn SafeMath_SizeT_Success() {
    let mut r = Reporter::default();
    let max: usize = usize::MAX;
    let halfMax: usize = max >> 1;
    let halfMaxPlus1: usize = halfMax + 1;
    let mut safe = SkSafeMath::new();

    check!(r, safe.add(halfMax, halfMax) == 2 * halfMax);
    check!(r, safe.add(halfMax, halfMaxPlus1) == max);
    check!(r, safe.ok());

    let bits: usize = std::mem::size_of::<usize>() * 8;
    let halfBits: usize = bits / 2;
    let sqrtMax: usize = max >> halfBits;
    let sqrtMaxPlus1: usize = sqrtMax + 1;
    safe = SkSafeMath::new();
    check!(r, safe.mul(sqrtMax, sqrtMax) == sqrtMax * sqrtMax);
    check!(r, safe.mul(sqrtMax, sqrtMaxPlus1) == sqrtMax << halfBits);
    check!(r, safe.ok());
    r.finish();
}

// tests/SafeMathTest.cpp:34
#[test]
fn SafeMath_SizeT_Failure() {
    let mut r = Reporter::default();
    let max: usize = usize::MAX;
    let mut safe = SkSafeMath::new();

    let _ = safe.add(max, 1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.add(max, max);
    check!(r, !safe.ok());

    let bits: usize = std::mem::size_of::<usize>() * 8;
    let halfBits: usize = bits / 2;
    let sqrtMax: usize = max >> halfBits;
    let sqrtMaxPlus1: usize = sqrtMax + 1;
    safe = SkSafeMath::new();
    let _ = safe.mul(sqrtMaxPlus1, sqrtMaxPlus1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul(max, max);
    check!(r, !safe.ok());
    r.finish();
}

// tests/SafeMathTest.cpp:58
#[test]
fn SafeMath_Int_Success() {
    let mut r = Reporter::default();
    let maxInt: i32 = i32::MAX;
    let minInt: i32 = i32::MIN;
    let mut safe = SkSafeMath::new();

    check!(r, safe.add_int(maxInt, 0) == maxInt);
    check!(r, safe.add_int(minInt, 1) == minInt + 1);
    check!(r, safe.sub_int(minInt, 0) == minInt);
    check!(r, safe.sub_int(-1, minInt) == maxInt);
    check!(r, safe.mul_int(maxInt, 1) == maxInt);
    check!(r, safe.mul_int(maxInt, 0) == 0);
    check!(r, safe.div_int(6, 3) == 2);
    check!(r, safe.mod_int(6, 3) == 0);
    check!(r, safe.mod_int(9, 4) == 1);
    check!(r, safe.ok());
    r.finish();
}

// tests/SafeMathTest.cpp:75
#[test]
fn SafeMath_Int_Failure() {
    let mut r = Reporter::default();
    let maxInt: i32 = i32::MAX;
    let minInt: i32 = i32::MIN;
    let sqrtMaxInt: i32 = 46340; // floor(sqrt(i32::MAX))
    let mut safe = SkSafeMath::new();

    let _ = safe.add_int(maxInt, 1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.add_int(minInt, -1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.sub_int(minInt, 1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.sub_int(maxInt, -1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul_int(maxInt, 2);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul_int(sqrtMaxInt + 1, sqrtMaxInt + 1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul_int(maxInt, -2);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul_int(minInt, 2);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul_int(minInt, minInt);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mul_int(minInt, -1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.div_int(6, 0);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.div_int(minInt, -1);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mod_int(6, 0);
    check!(r, !safe.ok());

    safe = SkSafeMath::new();
    let _ = safe.mod_int(minInt, -1);
    check!(r, !safe.ok());
    r.finish();
}
