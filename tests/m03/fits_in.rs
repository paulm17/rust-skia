// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// Reference: b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
// tests/FitsInTest.cpp:23; enum fits use the declared underlying representation.
#[repr(u8)]
enum TestEnum {
    Foo,
    Bar,
    Baz,
}
#[repr(i32)]
enum E {
    N1 = -1,
    P1 = 1,
}
#[test]
fn FitsIn() {
    let mut r = Reporter::default();
    check!(r, sk_tfits_in_i8((257) as u16) == false);
    check!(r, sk_tfits_in_i8((1) as i32) == true);
    check!(r, sk_tfits_in_i8((-1) as i32) == true);
    check!(r, sk_tfits_in_i8((i8::MAX) as i32) == true);
    check!(r, sk_tfits_in_i8((i8::MAX as i32) + 1) == false);
    check!(r, sk_tfits_in_i8((i8::MIN) as i32) == true);
    check!(r, sk_tfits_in_i8((i8::MIN as i32) - 1) == false);
    check!(r, sk_tfits_in_u8((1) as i32) == true);
    check!(r, sk_tfits_in_u8((-1) as i32) == false);
    check!(r, sk_tfits_in_u8((u8::MAX) as i32) == true);
    check!(r, sk_tfits_in_u8((u8::MAX as i32) + 1) == false);
    check!(r, sk_tfits_in_u8((0) as i32) == true);
    check!(r, sk_tfits_in_u8((-1) as i32) == false);
    check!(r, sk_tfits_in_u8((-127) as i32) == false);
    check!(r, sk_tfits_in_u8((-128) as i32) == false);
    check!(r, sk_tfits_in_u8((2) as u8) == true);
    check!(r, sk_tfits_in_u8((TestEnum::Bar as u8) as u8) == true);
    check!(r, sk_tfits_in_i8((1000) as i32) == false);
    check!(r, sk_tfits_in_u8((1000) as i32) == false);
    check!(r, sk_tfits_in_i32((1) as i32) == true);
    check!(r, sk_tfits_in_i32((-1) as i32) == true);
    check!(r, sk_tfits_in_u32((1) as i32) == true);
    check!(r, sk_tfits_in_u32((-1) as i32) == false);
    check!(r, sk_tfits_in_i64((1) as i32) == true);
    check!(r, sk_tfits_in_i64((-1) as i32) == true);
    check!(r, sk_tfits_in_u64((1) as i32) == true);
    check!(r, sk_tfits_in_u64((-1) as i32) == false);
    check!(r, sk_tfits_in_i8((1) as u32) == true);
    check!(r, sk_tfits_in_u8((1) as u32) == true);
    check!(r, sk_tfits_in_i32((1) as u32) == true);
    check!(r, sk_tfits_in_u32((1) as u32) == true);
    check!(r, sk_tfits_in_i64((1) as u32) == true);
    check!(r, sk_tfits_in_u64((1) as u32) == true);
    check!(r, sk_tfits_in_i8((u32::MAX) as u32) == false);
    check!(r, sk_tfits_in_u8((u32::MAX) as u32) == false);
    check!(r, sk_tfits_in_i32((u32::MAX) as u32) == false);
    check!(r, sk_tfits_in_u32((u32::MAX) as u32) == true);
    check!(r, sk_tfits_in_i64((u32::MAX) as u32) == true);
    check!(r, sk_tfits_in_u64((u32::MAX) as u32) == true);
    check!(r, sk_tfits_in_i8((1) as u64) == true);
    check!(r, sk_tfits_in_u8((1) as u64) == true);
    check!(r, sk_tfits_in_i32((1) as u64) == true);
    check!(r, sk_tfits_in_u32((1) as u64) == true);
    check!(r, sk_tfits_in_i64((1) as u64) == true);
    check!(r, sk_tfits_in_u64((1) as u64) == true);
    check!(r, sk_tfits_in_i32((E::P1 as i32) as i32) == true);
    check!(r, sk_tfits_in_i32((E::N1 as i32) as i32) == true);
    check!(r, sk_tfits_in_i32((-1) as i32) == true);
    check!(r, sk_tfits_in_i32((1) as i32) == true);
    check!(r, sk_tfits_in_u32((E::P1 as i32) as i32) == true);
    check!(r, sk_tfits_in_u32((E::N1 as i32) as i32) == false);
    r.finish();
}
// Original commented-out float -> integer compile-failure example remains disabled.
