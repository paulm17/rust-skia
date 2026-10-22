// Supplemental M04 contracts, separate from original registrations.
use super::*;

#[test]
fn supplemental_boundaries_overflow_and_layout() {
    assert_eq!(core::mem::size_of::<IRect>(), 16);
    assert_eq!(core::mem::size_of::<Rect>(), 16);
    assert_eq!(core::mem::size_of::<Point3>(), 12);
    assert_eq!(core::mem::size_of::<ISize>(), 8);
    assert_eq!(core::mem::size_of::<Size>(), 8);
    assert_eq!(core::mem::size_of::<IPoint16>(), 4);
    assert_eq!(core::mem::offset_of!(IPoint16, y), 2);
    for (x, y) in [(i16::MIN as i32, i16::MAX as i32), (0, -1), (1, 0)] {
        let mut p = IPoint16::make(x, y);
        assert_eq!((p.x(), p.y()), (x as i16, y as i16));
        p.set(y, x);
        assert_eq!((p.x(), p.y()), (y as i16, x as i16));
    }
    let huge = IRect::new(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    assert_eq!(huge.width64(), 4294967295);
    assert_eq!(huge.height64(), 4294967295);
    assert_eq!(huge.width(), -1);
    assert_eq!(huge.height(), -1);
    assert!(huge.is_empty());
    assert!(!huge.is_empty64());
    assert_eq!(ISize::new(i32::MAX, i32::MAX).area(), 4611686014132420609);
    assert_eq!(ISize::new(i32::MIN, i32::MIN).area(), 4611686018427387904);
    let r = IRect::new(-2, -3, 4, 5);
    assert!(r.contains(-2, -3));
    assert!(r.contains(3, 4));
    assert!(!r.contains(4, 0));
    assert!(!r.contains(0, 5));
    let rf = Rect::from(r);
    assert!(rf.contains(-2.0, -3.0));
    assert!(!rf.contains(4.0, 0.0));
    assert!(!Rect::new(4.0, 5.0, -2.0, -3.0).is_sorted());
    assert!(Rect::new(4.0, 5.0, -2.0, -3.0).is_empty());
    assert_eq!(Rect::new(4.0, 5.0, -2.0, -3.0).make_sorted(), rf);
    assert_eq!(
        Rect::from(RectPriv::make_i_large()).round_out(),
        RectPriv::make_i_large()
    );
    let r = Rect::new(-1.5, -2.25, 3.5, 4.25);
    assert_eq!(r.round(), IRect::new(-1, -2, 4, 4));
    assert_eq!(r.round_out(), IRect::new(-2, -3, 4, 5));
    assert_eq!(r.round_in(), IRect::new(-1, -2, 3, 4));
    let s = Size::new(-1.5, 2.5);
    assert_eq!(s.to_round(), ISize::new(-1, 3));
    assert_eq!(s.to_floor(), ISize::new(-2, 2));
    assert_eq!(s.to_ceil(), ISize::new(-1, 3));
    let mut bounds = Rect::make_wh(10.0, 10.0);
    assert!(bounds.set_bounds_check(&[]));
    assert_eq!(bounds, Rect::make_empty());
    assert!(bounds.set_bounds_check(&[Point::new(2.0, 3.0)]));
    assert_eq!(bounds, Rect::new(2.0, 3.0, 2.0, 3.0));
    for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
        assert!(!bounds.set_bounds_check(&[Point::new(1.0, 2.0), Point::new(bad, 3.0)]));
        assert_eq!(bounds, Rect::make_empty());
    }
}

#[test]
fn supplemental_point3_nonfinite_cross_and_value_ownership() {
    let x = Point3::new(1.0, 0.0, 0.0);
    let y = Point3::new(0.0, 1.0, 0.0);
    let z = Point3::new(0.0, 0.0, 1.0);
    assert_eq!(x.cross(&y), z);
    assert_eq!(y.cross(&x), -z);
    assert_eq!(Point3::cross_product(&x, &y), z);
    assert_eq!(Point3::dot_product(&x, &y), 0.0);
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for mut p in [
            Point3::new(bad, 1.0, 2.0),
            Point3::new(1.0, bad, 2.0),
            Point3::new(1.0, 2.0, bad),
        ] {
            assert!(!p.is_finite());
            assert!(!p.normalize());
            assert_eq!(p, Point3::new(0.0, 0.0, 0.0));
        }
    }
    let original = Point3::new(2.0, 3.0, 4.0);
    let mut copy = original;
    copy += x;
    copy -= y;
    assert_eq!(copy, Point3::new(3.0, 2.0, 4.0));
    assert_eq!(original, Point3::new(2.0, 3.0, 4.0));
    assert_eq!(original * 2.0, 2.0 * original);
}

#[test]
fn supplemental_rrect_native_byte_order_truncation_and_lifetime() {
    // All twelve serialized scalars are explicit independent expected bytes;
    // no type tag or Rust object padding is serialized.
    let values: [f32; 12] = [
        1.0, 2.0, 101.0, 202.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
    ];
    let mut expected = [0u8; 48];
    for (i, value) in values.into_iter().enumerate() {
        expected[i * 4..i * 4 + 4].copy_from_slice(&value.to_ne_bytes());
    }
    let rr = {
        let bounds = Rect::new(1.0, 2.0, 101.0, 202.0);
        let mut radii = [
            Point::new(3.0, 4.0),
            Point::new(5.0, 6.0),
            Point::new(7.0, 8.0),
            Point::new(9.0, 10.0),
        ];
        let owned = RRect::make_rect_radii(&bounds, &radii);
        radii.fill(Point::new(0.0, 0.0));
        assert_eq!(owned.radii_at(Corner::UpperLeft), Point::new(3.0, 4.0));
        owned
    }; // constructor inputs die here; rr must retain its own geometry.
    assert!(rr.is_valid());
    assert_eq!(RRect::K_SIZE_IN_MEMORY, 48);
    let mut guarded = [0xa5u8; 52];
    assert_eq!(rr.write_to_memory(&mut guarded[1..51]), 48);
    assert_eq!(&guarded[1..49], &expected);
    assert_eq!(guarded[0], 0xa5);
    assert_eq!(&guarded[49..], [0xa5; 3]);
    let decoded = {
        let mut bytes = expected;
        let mut value = RRect::new();
        assert_eq!(value.read_from_memory(&bytes), 48);
        bytes.fill(0);
        value
    }; // serialized buffer dies here; decoded geometry must remain owned.
    assert_eq!(decoded, rr);
    let mut copy = rr;
    copy.offset(20.0, 30.0);
    assert_ne!(copy, rr);
    assert_eq!(*rr.rect(), Rect::new(1.0, 2.0, 101.0, 202.0));
    for n in 0..48 {
        let mut short = [0x5au8; 48];
        assert_eq!(rr.write_to_memory(&mut short[..n]), 0);
        assert_eq!(short, [0x5a; 48]);
        let mut destination = rr;
        assert_eq!(destination.read_from_memory(&expected[..n]), 0);
        assert_eq!(destination, rr);
    }
}

#[test]
fn supplemental_rrect_normalization_and_radius_boundaries() {
    let bounds = Rect::make_wh(100.0, 80.0);
    for (x, y, kind) in [
        (0.0, 0.0, Type::Rect),
        (10.0, 20.0, Type::Simple),
        (50.0, 40.0, Type::Oval),
        (100.0, 80.0, Type::Oval),
        (-1.0, 10.0, Type::Rect),
    ] {
        let rr = RRect::make_rect_xy(&bounds, x, y);
        assert!(rr.is_valid());
        assert_eq!(rr.get_type(), kind);
    }
    for rect in [
        Rect::make_empty(),
        Rect::new(0.0, 0.0, f32::INFINITY, 80.0),
        Rect::new(0.0, f32::NAN, 100.0, 80.0),
    ] {
        let rr = RRect::make_oval(&rect);
        assert!(rr.is_empty());
        assert!(rr.is_valid());
    }
    let exact = [Point::new(50.0, 40.0); 4];
    assert!(RRectPriv::are_rect_and_radii_valid(&bounds, &exact));
    let mut excess = exact;
    excess[0].x = f32::from_bits(50.0f32.to_bits() + 1);
    // SkRRect.cpp:760-789 accepts near-equal sums and corner positions.
    assert!(RRectPriv::are_rect_and_radii_valid(&bounds, &excess));
    let rr = RRect::make_rect_radii(&bounds, &excess);
    assert!(rr.is_valid());
    // A clear overlap exceeds both the absolute and ULP tolerances.
    excess[0].x = 51.0;
    assert!(!RRectPriv::are_rect_and_radii_valid(&bounds, &excess));
    for bad in [f32::NAN, f32::INFINITY, -1.0] {
        let mut radii = exact;
        radii[0].y = bad;
        assert!(!RRectPriv::are_rect_and_radii_valid(&bounds, &radii));
    }
}

#[test]
fn supplemental_rect_memcpy_strides_guards_and_source_lifetime() {
    for (dst_rb, src_rb, trim, rows) in [
        (4, 4, 4, 3),
        (7, 9, 4, 3),
        (7, 9, 0, 3),
        (7, 9, 4, 0),
        (7, 9, 4, 1),
    ] {
        let mut destination = vec![0xa5u8; dst_rb * rows + 2];
        let expected;
        {
            let source: Vec<u8> = (0..src_rb * rows + 2).map(|i| i as u8).collect();
            let source_before = source.clone();
            let mut wanted = destination.clone();
            for row in 0..rows {
                wanted[1 + row * dst_rb..1 + row * dst_rb + trim]
                    .copy_from_slice(&source[1 + row * src_rb..1 + row * src_rb + trim]);
            }
            // Separate allocations, full strides allocated (including the final
            // padding), and valid non-null pointers even for zero copies.
            unsafe {
                sk_rect_memcpy(
                    destination.as_mut_ptr().add(1),
                    dst_rb,
                    source.as_ptr().add(1),
                    src_rb,
                    trim,
                    rows,
                );
            }
            assert_eq!(source, source_before);
            expected = wanted;
        }
        // Output remains valid after the source allocation is released.
        assert_eq!(destination, expected);
    }
}

#[test]
fn supplemental_scale_to_sides_fit_and_symmetry() {
    for (limit, scale, a, b) in [
        (10.0, 0.5, 10.0f32, 10.0f32),
        (1.0, 0.5, 0.0, 2.0),
        (100.0, 0.1, 100.0, 200.0),
        (1.0, 0.5, f32::MIN_POSITIVE, 1.0),
        (33554431.0, 0.5, 33554430.0, 33554430.0),
    ] {
        let (mut x, mut y) = (a, b);
        adjust_radii(limit, scale, &mut x, &mut y);
        assert!(x >= 0.0 && y >= 0.0);
        assert!((x + y) as f64 <= limit);
        assert!(x <= a && y <= b);
        let (mut reverse_x, mut reverse_y) = (b, a);
        adjust_radii(limit, scale, &mut reverse_x, &mut reverse_y);
        assert_eq!((x, y), (reverse_y, reverse_x));
    }
}
