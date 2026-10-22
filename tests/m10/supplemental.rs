// Supplemental M10 contracts; these do not replace upstream registrations.
use rust_skia::sk_codecs_icc_profile_chromium::IccProfileChromium;
use rust_skia::sk_color::*;
use rust_skia::sk_color_data::*;
use rust_skia::sk_color_space::SkColorSpace;
use rust_skia::sk_color_table::SkColorTable;
use rust_skia::skcms::*;
use rust_skia::skcms_transform::skcms_transform;

#[test]
fn supplemental_packed_boundaries_and_widened_interpolation() {
    // Independent u64 arithmetic catches carry/overflow in the packed implementations.
    for src in [0, 1, 0x00ff00ff, 0xff00ff00, 0x7f80fe01, u32::MAX] {
        let (ag, rb) = sk_splay(src);
        assert_eq!(sk_unsplay(ag << 8, rb << 8), src);
        assert_eq!(sk_unsplay_64(sk_splay_64(src) << 8), src);
        for dst in [0, 0x01020304, 0x80ff017f, u32::MAX] {
            for scale in 0..=256u32 {
                let mut expected = 0u32;
                for shift in [0, 8, 16, 24] {
                    let s = ((src >> shift) & 255) as u64;
                    let d = ((dst >> shift) & 255) as u64;
                    expected |=
                        (((s * scale as u64 + d * (256 - scale) as u64) / 256) as u32) << shift;
                }
                assert_eq!(sk_four_byte_interp256(src, dst, scale as i32), expected);
                assert_eq!(sk_fast_four_byte_interp256_32(src, dst, scale), expected);
                assert_eq!(sk_fast_four_byte_interp256_64(src, dst, scale), expected);
            }
        }
    }
    for a in 0..=255u8 {
        for x in 0..=255u8 {
            let packed = sk_pre_multiply_argb(a, x, 255 - x, 255);
            assert_eq!(sk_pm_color_get_a(packed), a);
            assert_eq!(
                sk_pm_color_get_r(packed) as u32,
                (a as u32 * x as u32 + 127) / 255
            );
            assert_eq!(
                sk_pm_color_get_g(packed) as u32,
                (a as u32 * (255 - x) as u32 + 127) / 255
            );
            assert_eq!(sk_pm_color_get_b(packed), a);
        }
    }
    // Apple Silicon's native PMColor is RGBA in little-endian memory.
    assert_eq!(
        sk_pm_color_set_argb(0xfe, 0xdc, 0xba, 0x98).to_le_bytes(),
        [0xdc, 0xba, 0x98, 0xfe]
    );
}

#[test]
fn supplemental_transform_byte_order_and_size_overflow() {
    use SkcmsAlphaFormat::Unpremul;
    use SkcmsPixelFormat::*;
    let profile = skcms_srgb_profile();
    let src = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
    let mut dst = [0xa5; 10];
    assert!(skcms_transform(
        &src,
        Rgba16161616Be,
        Unpremul,
        Some(profile),
        &mut dst[1..9],
        Rgba16161616Le,
        Unpremul,
        Some(profile),
        1
    ));
    assert_eq!(
        dst,
        [0xa5, 0x34, 0x12, 0x78, 0x56, 0xbc, 0x9a, 0xf0, 0xde, 0xa5]
    );
    let mut restored = [0; 8];
    assert!(skcms_transform(
        &dst[1..9],
        Rgba16161616Le,
        Unpremul,
        Some(profile),
        &mut restored,
        Rgba16161616Be,
        Unpremul,
        Some(profile),
        1
    ));
    assert_eq!(restored, src);
    for count in [usize::MAX, usize::MAX / 4 + 1, i32::MAX as usize] {
        let mut output = [0xa5; 4];
        assert!(!skcms_transform(
            &[],
            Rgba8888,
            Unpremul,
            Some(profile),
            &mut output,
            Rgba8888,
            Unpremul,
            Some(profile),
            count
        ));
        assert_eq!(output, [0xa5; 4]);
    }
    for (src, len) in [(&[][..], 4), (&[0; 4][..], 3)] {
        let mut output = [0xa5; 4];
        assert!(!skcms_transform(
            src,
            Rgba8888,
            Unpremul,
            Some(profile),
            &mut output[..len],
            Rgba8888,
            Unpremul,
            Some(profile),
            1
        ));
        assert_eq!(output, [0xa5; 4]);
    }
}

#[test]
fn supplemental_table_ownership_lifetime_and_serialization_boundaries() {
    assert!(SkColorTable::make(None).is_none());
    let (table, expected) = {
        let mut input: [u8; 256] = std::array::from_fn(|i| 255 - i as u8);
        let expected = input;
        let original = SkColorTable::make_rgba(Some(&input), None, Some(&input), None).unwrap();
        let clone = original.clone();
        input.fill(0);
        drop(original);
        (clone, expected)
    };
    let identity: [u8; 256] = std::array::from_fn(|i| i as u8);
    assert_eq!(table.alpha_table(), &expected);
    assert_eq!(table.red_table(), &identity);
    assert_eq!(table.green_table(), &expected);
    assert_eq!(table.blue_table(), &identity);
    let mut bytes = table.flatten_bytes();
    assert_eq!(&bytes[..256], &expected);
    assert_eq!(&bytes[256..512], &identity);
    let restored = SkColorTable::deserialize(&bytes).unwrap();
    bytes.fill(0);
    drop(table);
    assert_eq!(restored.alpha_table(), &expected);
    assert_eq!(restored.blue_table(), &identity);
    for size in [0, 1, 255, 1023, 1025] {
        assert!(SkColorTable::deserialize(&vec![0; size]).is_none());
    }
    let space = SkColorSpace::make_srgb();
    let serialized = space.serialize();
    let restored = SkColorSpace::deserialize(serialized.bytes()).unwrap();
    drop(serialized);
    drop(space);
    assert!(SkColorSpace::equals(
        Some(&restored),
        Some(&SkColorSpace::make_srgb())
    ));
    let bytes = restored.serialize();
    for len in 0..bytes.bytes().len() {
        assert!(SkColorSpace::deserialize(&bytes.bytes()[..len]).is_none());
    }
    let mut invalid = bytes.bytes().to_vec();
    invalid[0] = 255;
    assert!(SkColorSpace::deserialize(&invalid).is_none());
}

#[test]
fn supplemental_icc_malformed_offsets_and_owned_profile_lifetime() {
    let fixture = include_bytes!("fixtures/AdobeRGB1998.icc");
    for len in [0, 1, 127, 128, 131] {
        assert!(IccProfileChromium::make_from_bytes(&fixture[..len]).is_none());
    }
    // ICC integers are big-endian; reject wrapped tag counts, offsets and sizes.
    for offset in [128, 136, 140] {
        let mut malformed = fixture.to_vec();
        malformed[offset..offset + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(IccProfileChromium::make_from_bytes(&malformed).is_none());
    }
    let owned = {
        let mut source = fixture.to_vec();
        let original = IccProfileChromium::make_from_bytes(&source).unwrap();
        let cloned = original.clone();
        source.fill(0);
        drop(original);
        cloned
    };
    let profile = owned.get_profile();
    assert_eq!(profile.data_color_space, SKCMS_SIGNATURE_RGB);
    assert_eq!(profile.pcs, SKCMS_SIGNATURE_XYZ);
    assert!(profile.has_trc && profile.has_to_xyzd50);
    assert_eq!(&profile.buffer[..fixture.len()], fixture);
    let src = [0, 0, 0, 255, 255, 255, 255, 255];
    let mut output = [0; 8];
    assert!(skcms_transform(
        &src,
        SkcmsPixelFormat::Rgba8888,
        SkcmsAlphaFormat::Unpremul,
        Some(profile),
        &mut output,
        SkcmsPixelFormat::Rgba8888,
        SkcmsAlphaFormat::Unpremul,
        Some(skcms_srgb_profile()),
        2
    ));
    assert_eq!(output, src);
}
