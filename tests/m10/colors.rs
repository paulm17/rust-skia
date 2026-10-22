// Copyright Google LLC. BSD: tests/LICENSE.skia.
// Pinned Skia b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use crate::support::{Reporter, reporter_assert};
use rust_skia::sk_color::*;
use rust_skia::sk_color_data::*;
use rust_skia::sk_color_priv::{
    SK_R32_SHIFT, sk_get_packed_a32, sk_get_packed_b32, sk_get_packed_g32, sk_get_packed_r32,
};
use rust_skia::sk_un_pre_multiply::SkUnPreMultiply;
use rust_skia::{SkRandom, sk_mul_div255_ceiling};

// tests/ColorPrivTest.cpp:18
#[test]
fn Splay() {
    let mut r = Reporter::default();
    let color = 0xA1B2C3D4;
    let (ag, rb) = sk_splay(color);
    reporter_assert!(r, ag == 0x00A100C3);
    reporter_assert!(r, rb == 0x00B200D4);
    reporter_assert!(r, sk_unsplay(ag << 8, rb << 8) == color);
    let agrb = sk_splay_64(color);
    reporter_assert!(r, agrb == 0x00A100C300B200D4);
    reporter_assert!(r, sk_unsplay_64(agrb << 8) == color);
    r.finish();
}

// tests/ColorPrivTest.cpp:32
#[test]
fn FourByteInterp() {
    let mut r = Reporter::default();
    let (src, dst) = (0xAB998877, 0x66334455);
    for scale in 0..=256 {
        reporter_assert!(
            r,
            sk_four_byte_interp256(src, dst, scale as i32)
                == sk_fast_four_byte_interp256(src, dst, scale)
        );
    }
    for scale in 0..256 {
        let slow = sk_four_byte_interp(src, dst, scale as u8);
        let fast = sk_fast_four_byte_interp(src, dst, scale as u8);
        let deltaA = sk_get_packed_a32(slow) as i32 - sk_get_packed_a32(fast) as i32;
        let deltaR = sk_get_packed_r32(slow) as i32 - sk_get_packed_r32(fast) as i32;
        let deltaG = sk_get_packed_g32(slow) as i32 - sk_get_packed_g32(fast) as i32;
        let deltaB = sk_get_packed_b32(slow) as i32 - sk_get_packed_b32(fast) as i32;
        reporter_assert!(r, deltaA == 0 || deltaA == 1);
        reporter_assert!(r, deltaR == 0 || deltaR == 1);
        reporter_assert!(r, deltaG == 0 || deltaG == 1);
        reporter_assert!(r, deltaB == 0 || deltaB == 1);
    }
    r.finish();
}

// tests/ColorPrivTest.cpp:56
#[test]
fn SkBlendARGB32_SameOpaqueSrcAndDstWithAnyAlpha_ProducesSameColor() {
    let mut r = Reporter::default();
    let colors = [
        SK_COLOR_WHITE,
        SK_COLOR_BLACK,
        SK_COLOR_GRAY,
        SK_COLOR_RED,
        SK_COLOR_GREEN,
        SK_COLOR_BLUE,
        SK_COLOR_YELLOW,
        SK_COLOR_CYAN,
        SK_COLOR_MAGENTA,
        sk_color_set_argb(255, 255 / 4, 255 / 3, 255 / 2),
        0xFF2E3338,
    ];
    for c in colors {
        assert_eq!(
            sk_color_get_a(c),
            0xFF,
            "this is only true for opaque colors"
        );
        let srcAndDst = sk_pre_multiply_color(c);
        for scale in 0..256 {
            let alpha = scale as u8;
            let output = sk_blend_argb32(srcAndDst, srcAndDst, alpha);
            r.record(
                srcAndDst == output,
                &format!(
                    "SkBlendARGB32({srcAndDst:08x}, {srcAndDst:08x}, {alpha:02x}) = {output:08x}"
                ),
                file!(),
                line!(),
            );
        }
    }
    r.finish();
}

// tests/ColorTest.cpp:19
#[test]
fn ColorPremul() {
    let mut reporter = Reporter::default();
    for a in 0..=255u8 {
        for x in 0..=255u8 {
            let c0 = sk_color_set_argb(a, x, x, x);
            let p0 = sk_pre_multiply_color(c0);
            let c1 = SkUnPreMultiply::pm_color_to_color(p0);
            let p1 = sk_pre_multiply_color(c1);
            reporter_assert!(reporter, p0 == p1);
            let ax = sk_mul_div255_ceiling(x, a);
            reporter_assert!(reporter, ax <= a as u32);
        }
    }
    reporter.finish();
}

// tests/ColorTest.cpp:40. Preserve the upstream packing conditional via the public shift.
#[test]
fn SkPMColor_SetAndRetrieveChannels() {
    let mut reporter = Reporter::default();
    let pmc = sk_pm_color_set_argb(0xFE, 0xDC, 0xBA, 0x98);
    if SK_R32_SHIFT == 0 {
        reporter_assert!(reporter, pmc == 0xFE98BADC);
    } else {
        reporter_assert!(reporter, pmc == 0xFEDCBA98);
    }
    reporter_assert!(reporter, sk_pm_color_get_a(pmc) == 0xFE);
    reporter_assert!(reporter, sk_pm_color_get_r(pmc) == 0xDC);
    reporter_assert!(reporter, sk_pm_color_get_g(pmc) == 0xBA);
    reporter_assert!(reporter, sk_pm_color_get_b(pmc) == 0x98);
    reporter.finish();
}

// tests/ColorTest.cpp:60. The original false branch remains disabled.
#[test]
fn ColorInterp() {
    let mut reporter = Reporter::default();
    let mut r = SkRandom::new();
    let (a0, a255) = (0, 255);
    for _i in 0..200 {
        let colorSrc = r.next_u();
        let colorDst = r.next_u();
        let src = sk_pre_multiply_color(colorSrc);
        let dst = sk_pre_multiply_color(colorDst);
        if false {
            reporter_assert!(reporter, sk_four_byte_interp(src, dst, a0) == dst);
            reporter_assert!(reporter, sk_four_byte_interp(src, dst, a255) == src);
        }
    }
    reporter.finish();
}

// tests/ColorTest.cpp:78
#[test]
fn ColorFastIterp() {
    let mut reporter = Reporter::default();
    let mut r = SkRandom::new();
    let (a0, a255) = (0, 255);
    for _i in 0..200 {
        let colorSrc = r.next_u();
        let colorDst = r.next_u();
        let src = sk_pre_multiply_color(colorSrc);
        let dst = sk_pre_multiply_color(colorDst);
        reporter_assert!(reporter, sk_fast_four_byte_interp(src, dst, a0) == dst);
        reporter_assert!(reporter, sk_fast_four_byte_interp(src, dst, a255) == src);
    }
    reporter.finish();
}

// tests/HSVRoundTripTest.cpp:14. All 256^3 inputs; ERRORF remains nonfatal.
#[test]
fn ColorToHSVRoundTrip() {
    let mut reporter = Reporter::default();
    for r in 0..=255u8 {
        for g in 0..=255u8 {
            for b in 0..=255u8 {
                let color = sk_color_set_rgb(r, g, b);
                let hsv = sk_color_to_hsv(color);
                let result = sk_hsv_to_color(0xFF, &hsv);
                if result != color {
                    reporter.record(false, &format!("HSV roundtrip mismatch! original: {color:X}, HSV: {}, {}, {}, result: {result:X}", hsv[0], hsv[1], hsv[2]), file!(), line!());
                }
            }
        }
    }
    reporter.finish();
}

// tests/SkColor4fTest.cpp:11
#[test]
fn SkColor4f_FromColor() {
    let mut reporter = Reporter::default();
    let recs = [
        (SK_COLOR_BLACK, SkColor4f::new(0.0, 0.0, 0.0, 1.0)),
        (SK_COLOR_WHITE, SkColor4f::new(1.0, 1.0, 1.0, 1.0)),
        (SK_COLOR_RED, SkColor4f::new(1.0, 0.0, 0.0, 1.0)),
        (SK_COLOR_GREEN, SkColor4f::new(0.0, 1.0, 0.0, 1.0)),
        (SK_COLOR_BLUE, SkColor4f::new(0.0, 0.0, 1.0, 1.0)),
        (0, SkColor4f::new(0.0, 0.0, 0.0, 0.0)),
    ];
    for (c, expected) in recs {
        let c4 = SkColor4f::from_color(c);
        reporter_assert!(reporter, c4 == expected);
    }
    reporter.finish();
}
