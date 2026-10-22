// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/SkBase64Test.cpp:18 at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
#[test]
fn SkBase64() {
    let mut r = Reporter::default();
    let mut all = [0u8; 256];
    for index in 0..255 {
        all[index] = (index + 1) as u8;
    }
    all[255] = 0;
    for offset in 0..6 {
        let length = 256 - offset;
        let predicted_encode_length = sk_base64_encoded_size(length);
        // Rust's allocating encode/decode APIs replace the C++ null-destination sizing calls.
        let actual_encode_length = sk_base64_encode(&all[offset..]).len();
        check!(r, actual_encode_length == predicted_encode_length);
        let mut src = sk_base64_encode(&all[offset..]);
        let n = src.len();
        check!(r, n == predicted_encode_length);
        src.push(0);
        let decoded = sk_base64_decode(&src[..actual_encode_length]);
        if decoded.is_err() {
            check!(r, false);
            continue;
        }
        let decode_length = decoded.unwrap().len();
        check!(r, decode_length == length);
        let decoded = sk_base64_decode(&src[..actual_encode_length]);
        if decoded.is_err() {
            check!(r, false);
            continue;
        }
        let dst = decoded.unwrap();
        check!(r, dst.len() == length);
        // strcmp includes every byte through the terminating NUL; each input contains one at its end.
        check!(r, all[offset..] == dst);
    }
    r.finish();
}
