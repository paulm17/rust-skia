// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// RoundRectTest.cpp:1652,1684,1714,1731; later registrations remain M07/M23.
#[test]
fn RRect_fuzzer_regressions() {
    let mut r = Reporter::default();
    {
        let buf: [u8; 48] = [
            0x0a, 0x00, 0x00, 0xff, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x7f, 0x7f,
            0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
            0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x20, 0x00, 0x02, 0x00,
        ];
        check!(r, buf.len() == RRect::new().read_from_memory(&buf));
    }

    {
        let buf: [u8; 48] = [
            0x5d, 0xff, 0xff, 0x5d, 0x0a, 0x60, 0x0a, 0x0a, 0x0a, 0x7e, 0x0a, 0x5a, 0x0a, 0x12,
            0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a,
            0x3a, 0x3a, 0x3a, 0x3a, 0x00, 0x00, 0x00, 0x0a, 0x0a, 0x0a, 0x0a, 0x26, 0x0a, 0x0a,
            0x0a, 0x0a, 0xff, 0xff, 0x0a, 0x0a,
        ];
        check!(r, buf.len() == RRect::new().read_from_memory(&buf));
    }

    {
        let buf: [u8; 48] = [
            0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x04, 0xdd, 0xdd, 0x15, 0xfe, 0x00,
            0x00, 0x04, 0x05, 0x7e, 0x00, 0x00, 0x00, 0xff, 0x08, 0x04, 0xff, 0xff, 0xfe, 0xfe,
            0xff, 0x32, 0x32, 0x32, 0x32, 0x00, 0x32, 0x32, 0x04, 0xdd, 0x3d, 0x1c, 0xfe, 0x89,
            0x04, 0x0a, 0x0e, 0x05, 0x7e, 0x0a,
        ];
        check!(r, buf.len() == RRect::new().read_from_memory(&buf));
    }
    r.finish();
}

#[test]
fn RRect_b511391129() {
    let mut r = Reporter::default();
    let kValA: f32 = f32::from_bits(0x4b4b4b4a); // 13323074.0f32
    let kValB: f32 = f32::from_bits(0x4b4b4b4b); // 13323083.0f32

    {
        let rect = Rect::make_ltrb(0.0f32, 0.0f32, kValB, kValB);
        let mut rrect = RRect::new();
        rrect.set_rect_xy(&rect, kValA, kValA);
        check!(r, rrect.is_valid());
    }

    {
        let rect = Rect::make_ltrb(0.0f32, 0.0f32, kValB, kValB);
        let mut rrect = RRect::new();
        rrect.set_nine_patch(&rect, kValA, kValA, kValA, kValA);
        check!(r, rrect.is_valid());
    }

    {
        let rect = Rect::make_ltrb(0.0f32, 0.0f32, kValB, kValB);
        let radii = [
            Point::new(kValA, kValA),
            Point::new(kValA, kValA),
            Point::new(kValA, kValA),
            Point::new(kValA, kValA),
        ];
        let mut rrect = RRect::new();
        rrect.set_rect_radii(&rect, &radii);
        check!(r, rrect.is_valid());
    }
    r.finish();
}

#[test]
fn RRect_b527765132() {
    let mut r = Reporter::default();
    // These values were found via fuzzing to trigger a validation due to ULPs only not being a
    // good way to compare floats
    let width: f32 = 37.408840f32;
    let height: f32 = 73.093933f32;
    let dx: f32 = -5430.867188f32;
    let dy: f32 = -26967.046875f32;

    let rect = Rect::make_wh(width, height);
    let rrect = RRect::make_oval(&rect);

    let offsetRRect = rrect.make_offset(dx, dy);

    check!(r, offsetRRect.is_valid());
    check!(r, offsetRRect.is_oval());
    r.finish();
}

#[test]
fn RRect_b547198215() {
    let mut r = Reporter::default();
    // Overlapping corner radii on any side must be rejected by AreRectAndRadiiValid.
    let rect = Rect::make_wh(100.0f32, 100.0f32);

    // Overlapping right side (UR.y + LR.y > height)
    {
        let radii = [
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 60.0f32),
            Point::new(0.0f32, 60.0f32),
            Point::new(0.0f32, 0.0f32),
        ];
        check!(r, !RRectPriv::are_rect_and_radii_valid(&rect, &radii));

        // Constructing via setRectRadii scales radii to fit, making it valid.
        let mut rrect = RRect::new();
        rrect.set_rect_radii(&rect, &radii);
        check!(r, rrect.is_valid());
    }

    // Overlapping top side (UL.x + UR.x > width)
    {
        let radii = [
            Point::new(70.0f32, 0.0f32),
            Point::new(70.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
        ];
        check!(r, !RRectPriv::are_rect_and_radii_valid(&rect, &radii));
    }

    // Overlapping bottom side (LL.x + LR.x > width)
    {
        let radii = [
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(60.0f32, 0.0f32),
            Point::new(60.0f32, 0.0f32),
        ];
        check!(r, !RRectPriv::are_rect_and_radii_valid(&rect, &radii));
    }

    // Overlapping left side (UL.y + LL.y > height)
    {
        let radii = [
            Point::new(0.0f32, 55.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 55.0f32),
        ];
        check!(r, !RRectPriv::are_rect_and_radii_valid(&rect, &radii));
    }

    // Negative radii must be rejected by AreRectAndRadiiValid.
    {
        let radii = [
            Point::new(-5.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
        ];
        check!(r, !RRectPriv::are_rect_and_radii_valid(&rect, &radii));
    }
    {
        let radii = [
            Point::new(0.0f32, -5.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
            Point::new(0.0f32, 0.0f32),
        ];
        check!(r, !RRectPriv::are_rect_and_radii_valid(&rect, &radii));
    }

    // Test the specific testcase geometry from b/547198215
    {
        let fuzzedRect = Rect::make_ltrb(
            1.35631564e-19f32,
            1.35631564e-19f32,
            163968.5f32,
            800.501953f32,
        );
        let fuzzedRadii = [
            Point::new(1.35631564e-19f32, 1.35631564e-19f32),
            Point::new(1.35631564e-19f32, 640.501953f32),
            Point::new(1.35631564e-19f32, 640.501953f32),
            Point::new(1.35631564e-19f32, 1.35631564e-19f32),
        ];
        check!(
            r,
            !RRectPriv::are_rect_and_radii_valid(&fuzzedRect, &fuzzedRadii)
        );
    }
    r.finish();
}
