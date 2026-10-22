// Supplemental M07 contracts, separate from original registrations.
use super::*;
#[test]
fn supplemental_measure_boundaries_and_output_ownership() {
    let mut r = Reporter::default();
    let path = SkPath::line(p(2.0, 3.0), p(5.0, 7.0));
    let mut meas = SkPathMeasure::from_path(&path, false, 1.0);
    let mut pos = p(99.0, 98.0);
    let mut tan = p(97.0, 96.0);
    for (d, expected) in [
        (-f32::INFINITY, p(2.0, 3.0)),
        (-1.0, p(2.0, 3.0)),
        (0.0, p(2.0, 3.0)),
        (5.0, p(5.0, 7.0)),
        (6.0, p(5.0, 7.0)),
        (f32::INFINITY, p(5.0, 7.0)),
    ] {
        check!(r, meas.get_pos_tan(d, Some(&mut pos), Some(&mut tan)));
        check!(r, pos == expected);
        check!(r, near(tan.x, 0.6));
        check!(r, near(tan.y, 0.8));
    }
    pos = p(99.0, 98.0);
    tan = p(97.0, 96.0);
    check!(
        r,
        !meas.get_pos_tan(f32::NAN, Some(&mut pos), Some(&mut tan))
    );
    check!(r, pos == p(99.0, 98.0));
    check!(r, tan == p(97.0, 96.0));
    check!(r, meas.get_pos_tan(2.5, None, None));
    let mut matrix = Matrix::new();
    check!(
        r,
        meas.get_matrix(2.5, Some(&mut matrix), MatrixFlags::GET_POS_AND_TAN)
    );
    let origin = matrix.map_point(p(0.0, 0.0));
    let axis = matrix.map_vector(p(1.0, 0.0));
    check!(r, near(origin.x, 3.5));
    check!(r, near(origin.y, 5.0));
    check!(r, near(axis.x, 0.6));
    check!(r, near(axis.y, 0.8));
    let mut segment = SkPathBuilder::new();
    check!(r, meas.get_segment(-1.0, 6.0, &mut segment, true));
    let extracted = segment.detach(None);
    check!(r, extracted == path);
    segment.move_to_xy(9.0, 9.0).line_to_xy(10.0, 10.0);
    let before = segment.snapshot(None);
    for (a, b) in [(4.0, 1.0), (f32::NAN, 2.0), (0.0, f32::NAN)] {
        check!(r, !meas.get_segment(a, b, &mut segment, true));
        check!(r, segment.snapshot(None) == before);
    }
    meas.set_path(None, false);
    check!(r, meas.get_length() == 0.0);
    check!(r, !meas.is_closed());
    check!(r, !meas.next_contour());
    check!(r, !meas.get_pos_tan(0.0, Some(&mut pos), Some(&mut tan)));
    r.finish();
}
#[test]
fn supplemental_t_value_width_and_overflow_growth() {
    let mut r = Reporter::default();
    check!(r, K_MAX_T_VALUE == 0x3fff_ffff);
    check!(r, t_value_to_scalar(0) == 0.0);
    check!(r, t_value_to_scalar(K_MAX_T_VALUE) == 1.0);
    let mut previous = 0.0;
    for t in [
        1,
        2,
        3,
        0xffff,
        0x1fff_ffff,
        0x2000_0000,
        K_MAX_T_VALUE - 1,
        K_MAX_T_VALUE,
    ] {
        let value = t_value_to_scalar(t);
        check!(r, value >= previous);
        check!(r, value <= 1.0);
        previous = value;
    }
    // A positive short edge after a large accumulated length must not create equal-distance segments.
    let pts = [p(100000.0, 100000.0), p(1.0005, 0.9999), p(1.0, 1.0)];
    let path = SkPath::polygon(&pts, false, Winding, false);
    let mut iter = SkContourMeasureIter::from_path(&path, false, 1.0);
    let cm = iter.next().unwrap();
    let mut distance = 0.0;
    for seg in &cm.f_segments {
        check!(r, seg.f_distance > distance);
        check!(r, seg.f_t_value <= K_MAX_T_VALUE);
        distance = seg.f_distance;
    }
    check!(r, distance == cm.length());
    // Finite coordinates whose subtraction/length overflows cannot produce a usable measure.
    let path = SkPath::line(p(-f32::MAX, 0.0), p(f32::MAX, 0.0));
    let mut iter = SkContourMeasureIter::from_path(&path, false, 1.0);
    check!(r, iter.next().is_none());
    r.finish();
}
#[test]
fn supplemental_path_copy_builder_snapshot_and_lifetimes() {
    let mut r = Reporter::default();
    let mut builder = SkPathBuilder::new();
    builder
        .move_to_xy(1.0, 2.0)
        .conic_to_xy(3.0, 4.0, 5.0, 6.0, 0.5);
    let first = builder.snapshot(None);
    let copy = first.clone();
    let id = first.get_generation_id();
    let points = first.points().to_vec();
    let weights = first.conic_weights().to_vec();
    check!(r, copy.get_generation_id() == id);
    builder.line_to_xy(100.0, 200.0);
    let second = builder.detach(None);
    check!(r, second.get_generation_id() != id);
    check!(r, first.points() == points);
    check!(r, copy.points() == points);
    check!(r, copy.conic_weights() == weights);
    builder.reset().line_to_xy(-5.0, -6.0);
    drop(builder);
    check!(r, first == copy);
    check!(r, first.get_generation_id() == id);
    let mut changed = copy.clone();
    changed.set_fill_type(SkPathFillType::EvenOdd);
    changed.set_is_volatile(true);
    check!(r, first.get_fill_type() == Winding);
    check!(r, !first.is_volatile());
    check!(r, changed.get_fill_type() == SkPathFillType::EvenOdd);
    let (cm, raw) = {
        let path = SkPath::line(p(0.0, 0.0), p(3.0, 4.0));
        let mut iter = SkContourMeasureIter::from_path(&path, false, 1.0);
        let cm = iter.next().unwrap();
        let raw = path.raw(SkResolveConvexity::No).unwrap();
        iter.reset(&SkPath::new(), false, 1.0);
        drop(iter);
        (cm, raw)
    };
    let mut pos = Point::default();
    check!(r, cm.length() == 5.0);
    check!(r, cm.get_pos_tan(2.5, Some(&mut pos), None));
    check!(r, pos == p(1.5, 2.0));
    check!(r, raw.points() == [p(0.0, 0.0), p(3.0, 4.0)]);
    let data = {
        let mut b = SkPathBuilder::new();
        b.line_to_xy(3.0, 4.0);
        b.detach_data()
    };
    let weak = Arc::downgrade(&data);
    let path = SkPath::from_data(data.clone(), Winding, false);
    drop(data);
    check!(r, weak.upgrade().is_some());
    drop(path);
    check!(r, weak.upgrade().is_none());
    r.finish();
}
fn general_fixture() -> (SkPath, Vec<u8>) {
    // Native little-endian v5 general-path format on the assigned Apple Silicon target.
    // Header: fill=EvenOdd, 4 points, 1 conic, 4 verbs; payload M,L,K,X plus padding.
    let path = SkPathBuilder::new_with_fill_type(SkPathFillType::EvenOdd)
        .move_to_xy(1.0, -2.0)
        .line_to_xy(3.0, 4.0)
        .conic_to_xy(5.0, 6.0, 7.0, 8.0, 0.5)
        .close()
        .detach(None);
    let bytes = vec![
        5, 1, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 192, 0, 0, 64, 64,
        0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 0, 0, 0, 63, 0, 1,
        3, 5,
    ];
    (path, bytes)
}
#[test]
fn supplemental_serialization_byte_order_and_v4_compatibility() {
    let mut r = Reporter::default();
    let (path, expected) = general_fixture();
    check!(r, cfg!(target_endian = "little"));
    check!(r, path.write_to_memory(None) == expected.len());
    let mut storage = vec![0xa5; expected.len() + 8];
    let size = path.write_to_memory(Some(&mut storage[..expected.len()]));
    check!(r, size == expected.len());
    check!(r, storage[..size] == expected);
    check!(r, storage[size..] == [0xa5; 8]);
    check!(r, path.serialize().as_bytes() == expected);
    let mut n = usize::MAX;
    let read = SkPath::read_from_memory(&expected, Some(&mut n));
    check!(r, read.is_some());
    check!(r, read.unwrap() == path);
    check!(r, n == expected.len());
    let mut v4 = expected.clone();
    v4[0] = 4;
    v4[52..56].reverse();
    let mut n = 0;
    let read = SkPath::read_from_memory(&v4, Some(&mut n));
    check!(r, read.is_some());
    check!(r, read.unwrap() == path);
    check!(r, n == v4.len());
    let mut extra = expected.clone();
    extra.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
    let mut n = 0;
    let read = SkPath::read_from_memory(&extra, Some(&mut n));
    check!(r, read.is_some());
    check!(r, read.unwrap() == path);
    check!(r, n == expected.len());
    // Input memory can die after decoding; decoded path must own its points and weights.
    let decoded = {
        let input = expected.clone();
        SkPath::read_from_memory(&input, None).unwrap()
    };
    check!(r, decoded == path);
    r.finish();
}
#[test]
fn supplemental_serialization_truncation_overflow_and_invalid_payloads() {
    let mut r = Reporter::default();
    let (_, bytes) = general_fixture();
    for cut in 0..bytes.len() {
        let mut n = usize::MAX;
        check!(
            r,
            SkPath::read_from_memory(&bytes[..cut], Some(&mut n)).is_none()
        );
        check!(r, n == 0);
    }
    // Signed/unsigned count boundaries must be rejected before allocation or reading payload.
    for field in [4, 8, 12] {
        for count in [0x7fff_ffffu32, 0x8000_0000, 0xffff_ffff] {
            let mut malformed = bytes[..16].to_vec();
            malformed[field..field + 4].copy_from_slice(&count.to_le_bytes());
            let mut n = usize::MAX;
            check!(
                r,
                SkPath::read_from_memory(&malformed, Some(&mut n)).is_none()
            );
            check!(r, n == 0);
        }
    }
    for version in [0, 3, 6, 255] {
        let mut bad = bytes.clone();
        bad[0] = version;
        check!(r, SkPath::read_from_memory(&bad, None).is_none());
    }
    let mut bad = bytes.clone();
    bad[3] = 0xf0;
    check!(r, SkPath::read_from_memory(&bad, None).is_none());
    let mut bad = bytes.clone();
    bad[52] = 255;
    check!(r, SkPath::read_from_memory(&bad, None).is_none());
    let mut bad = bytes.clone();
    bad[20..24].copy_from_slice(&f32::INFINITY.to_bits().to_le_bytes());
    check!(r, SkPath::read_from_memory(&bad, None).is_none());
    let mut bad = bytes;
    bad[48..52].copy_from_slice(&(-1.0f32).to_bits().to_le_bytes());
    check!(r, SkPath::read_from_memory(&bad, None).is_none());
    r.finish();
}
#[test]
fn supplemental_raw_counts_and_generation_boundaries() {
    let mut r = Reporter::default();
    for (pts, v, w) in [
        (vec![], vec![M], vec![]),
        (vec![p(0.0, 0.0)], vec![M, K], vec![0.5]),
        (vec![p(0.0, 0.0); 3], vec![M, K], vec![]),
        (vec![p(0.0, 0.0); 3], vec![M, K], vec![0.5, 0.5]),
    ] {
        check!(r, SkPathData::make(&pts, &v, &w).is_none());
    }
    let mut b = SkPathBuilder::new();
    b.inc_reserve(0, 0, 0);
    check!(r, b.is_empty());
    b.line_to_xy(1.0, 1.0);
    let a = b.snapshot(None);
    let id = a.get_generation_id();
    let c = a.clone();
    check!(r, c.get_generation_id() == id);
    let mut dst = [p(-99.0, -99.0); 4];
    let n = a.get_points(&mut dst[..1]);
    check!(r, n == 1);
    check!(r, dst[0] == p(0.0, 0.0));
    check!(r, dst[1..] == [p(-99.0, -99.0); 3]);
    r.finish();
}
#[cfg(target_os = "macos")]
#[test]
fn supplemental_executable_path() {
    let mut r = Reporter::default();
    let path = rust_skia::sk_get_executable_path::sk_get_executable_path();
    check!(r, !path.is_empty());
    check!(r, !path.as_bytes().contains(&0));
    check!(r, std::path::Path::new(&path).is_absolute());
    r.finish();
}
