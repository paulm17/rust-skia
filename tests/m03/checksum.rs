// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// Reference b81c69985fd14bb7bfbd356b7618633ce07dc1a4; tests/ChecksumTest.cpp.
use super::*;
fn hash(data: &[u8]) -> u32 {
    sk_checksum_hash32(data, 0)
}
fn bytes<T: Copy>(data: &[T]) -> &[u8] {
    // Only initialized primitive arrays are used here; native byte order matches the C++ buffers.
    unsafe { std::slice::from_raw_parts(data.as_ptr().cast(), std::mem::size_of_val(data)) }
}
// tests/ChecksumTest.cpp:21
#[test]
fn Checksum() {
    let mut r = Reporter::default();
    let mut rand = SkRandom::new();
    let mut data = [0u32; 128 / 4];
    let mut tweaked = [0u32; 128 / 4];
    for i in 0..tweaked.len() {
        data[i] = rand.next_u();
        tweaked[i] = data[i];
    }
    let h = hash(bytes(&data));
    check!(r, h == hash(bytes(&data)));
    for j in 0..tweaked.len() {
        let saved = tweaked[j];
        tweaked[j] = rand.next_u();
        let tweaked_hash = hash(bytes(&tweaked));
        check!(r, tweaked_hash != h);
        check!(r, tweaked_hash == hash(bytes(&tweaked)));
        tweaked[j] = saved;
    }
    r.finish();
}
// tests/ChecksumTest.cpp:45; SkGoodHash's signed and unsigned 4-byte overloads use Mix.
#[test]
fn GoodHash() {
    let mut r = Reporter::default();
    check!(r, sk_checksum_mix(4i32 as u32) == 614249093);
    check!(r, sk_checksum_mix(4u32) == 614249093);
    r.finish();
}
// tests/ChecksumTest.cpp:51
#[test]
fn ChecksumCollisions() {
    let mut r = Reporter::default();
    {
        let a = [0f32, 1., 2., 3., 4., 5., 6., 7., 8.];
        let b = [1f32, 2., 0., 4., 5., 3., 7., 8., 6.];
        check!(r, hash(bytes(&a)) != hash(bytes(&b)));
    }
    {
        let a = [0f64, 1., 2., 3., 4., 5., 6., 7., 8.];
        let b = [1f64, 2., 0., 4., 5., 3., 7., 8., 6.];
        check!(r, hash(bytes(&a)) != hash(bytes(&b)));
    }
    r.finish();
}
// tests/ChecksumTest.cpp:79
#[test]
fn ChecksumConsistent() {
    let mut r = Reporter::default();
    let bytes: [u8; 256] = std::array::from_fn(|i| i as u8);
    let hash_bytes = |n| hash(&bytes[..n]);
    check!(r, hash_bytes(0) == 0xe2bde459);
    check!(r, hash_bytes(1) == 0xe5f8bd85);
    check!(r, hash_bytes(2) == 0x77acd42a);
    check!(r, hash_bytes(7) == 0x78d0861f);
    check!(r, hash_bytes(32) == 0x4e73df6d);
    check!(r, hash_bytes(63) == 0x5e66a3f4);
    check!(r, hash_bytes(64) == 0x962d6746);
    check!(r, hash_bytes(99) == 0x79e09416);
    check!(r, hash_bytes(255) == 0x85f837f0);
    r.finish();
}
// tests/ChecksumTest.cpp:98
#[test]
fn ChecksumStrings() {
    let mut r = Reporter::default();
    let message = "Checksums are supported for SkString, string, and string_view.";
    let expected = hash(message.as_bytes());
    let sk = SkString::from_str(message);
    check!(r, expected == sk_checksum_hash_string(sk.as_str()));
    let owned = message.to_owned();
    check!(r, expected == sk_checksum_hash_string(&owned));
    check!(r, expected == sk_checksum_hash_string(message));
    r.finish();
}
