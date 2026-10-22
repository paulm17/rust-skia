// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// Complete helpers of tests/RandomTest.cpp:186, reference b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
fn anderson_darling_test(p: &mut [f64; 32]) -> bool {
    sk_tqsort(p, |a, b| a.partial_cmp(b).unwrap());
    let mut s = 0.;
    for k in 0..32 {
        let mut v = p[k] * (1. - p[31 - k]);
        if v < 1.0e-30 {
            v = 1.0e-30;
        }
        s += (2. * (k + 1) as f64 - 1.) * v.ln();
    }
    let a2 = -32. - 0.03125 * s;
    0.202 < a2 && a2 < 3.89
}
fn chi_square_test(bins: &[i32; 256], e: i32) -> bool {
    let mut chi2 = 0.;
    for j in 0..256 {
        let delta = (bins[j] - e) as f64;
        chi2 += delta * delta / e as f64;
    }
    206.3179 < chi2 && chi2 < 311.5603
}
fn normal_cdf(z: f64) -> f64 {
    // Preserve the upstream `z*z* + 0.0418198` expression (unary plus, not addition).
    let mut t = ((-0.0004406 * z * z * 0.0418198) * z * z + 0.9) * z;
    t *= -1.77245385091;
    1. / (1. + t.exp())
}
fn test_random_byte(r: &mut Reporter, shift: u32) {
    let mut bins = [0; 256];
    let mut rand = SkRandom::new();
    for _ in 0..256 * 10000 {
        bins[((rand.next_u() >> shift) & 0xff) as usize] += 1;
    }
    check!(r, chi_square_test(&bins, 10000));
}
fn test_random_float(r: &mut Reporter) {
    let mut bins = [0; 256];
    let mut rand = SkRandom::new();
    for _ in 0..256 * 10000 {
        let f = rand.next_f();
        check!(r, 0. <= f && f < 1.);
        bins[(f * 256.) as usize] += 1;
    }
    check!(r, chi_square_test(&bins, 10000));
    let mut p = [0.; 32];
    for j in 0..32 {
        let f = rand.next_f();
        check!(r, 0. <= f && f < 1.);
        p[j] = f as f64;
    }
    check!(r, anderson_darling_test(&mut p));
}
fn test_single_gorilla(r: &mut Reporter, shift: u32) -> f64 {
    let k_word_width = 16;
    let k_n = 1 << k_word_width;
    let k_num_entries = k_n >> 5;
    let mut entries = [0u32; 2048];
    let mut rand = SkRandom::new();
    let mut value = 0i32;
    for _ in 0..k_word_width - 1 {
        value <<= 1;
        let rnd = rand.next_u();
        value |= ((rnd >> shift) & 1) as i32;
    }
    for _ in 0..k_n {
        value = left_shift_i32(value, 1);
        let rnd = rand.next_u();
        value |= ((rnd >> shift) & 1) as i32;
        let index = (value & (k_num_entries - 1)) as usize;
        debug_assert!(index < k_num_entries as usize);
        let entry_shift = (value >> (k_word_width - 5)) & 0x1f;
        entries[index] |= 1u32 << entry_shift;
    }
    let mut total = 0;
    for i in 0..k_num_entries as usize {
        let mut entry = entries[i];
        while entry != 0 {
            total += (entry & 1) as i32;
            entry >>= 1;
        }
    }
    let z = ((k_n - total) as f64 - 24108.) / 127.;
    let p = normal_cdf(z);
    check!(r, 0.01 < p && p < 0.99);
    p
}
fn test_gorilla(r: &mut Reporter) {
    let mut p = [0.; 32];
    for bit_position in 0..32 {
        p[bit_position] = test_single_gorilla(r, bit_position as u32);
    }
    check!(r, anderson_darling_test(&mut p));
}
fn test_range(r: &mut Reporter) {
    let mut rand = SkRandom::new();
    let _ = rand.next_range_u(0, 0xffffffff);
    let mut bins = [0; 256];
    for _ in 0..256 * 10000 {
        let u = rand.next_range_u(17, 17 + 255);
        check!(r, 17 <= u && u <= 17 + 255);
        bins[(u - 17) as usize] += 1;
    }
    check!(r, chi_square_test(&bins, 10000));
}
#[test]
fn Random() {
    let mut r = Reporter::default();
    test_random_byte(&mut r, 0);
    test_random_byte(&mut r, 8);
    test_random_byte(&mut r, 16);
    test_random_byte(&mut r, 24);
    test_random_float(&mut r);
    test_gorilla(&mut r);
    test_range(&mut r);
    r.finish();
}
