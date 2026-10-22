// Supplemental M08 contracts; not translations of original registrations.
use rust_skia::*;

#[test]
fn dash_validation_boundaries_and_sum_overflow() {
    for intervals in [
        &[][..],
        &[1.0][..],
        &[1.0, 2.0, 3.0][..],
        &[0.0, 0.0][..],
        &[-1.0, 2.0][..],
        &[f32::NAN, 1.0][..],
        &[f32::INFINITY, 1.0][..],
        &[f32::MAX, f32::MAX][..],
    ] {
        assert!(
            SkDashPathEffect::make(intervals, 0.0).is_none(),
            "{intervals:?}"
        );
    }
    for phase in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(SkDashPathEffect::make(&[1.0, 1.0], phase).is_none());
    }
    for intervals in [[0.0, 1.0], [1.0, 0.0], [-0.0, 1.0], [f32::MAX, 0.0]] {
        let info = SkDashPathEffect::make(&intervals, 0.0)
            .unwrap()
            .as_a_dash()
            .unwrap();
        for (actual, expected) in info.f_intervals.iter().zip(intervals) {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        assert_eq!(info.f_phase, 0.0);
    }
}

#[test]
fn dash_phase_wrap_and_negative_rounding_boundary() {
    for (phase, expected) in [
        (0.0, 0.0),
        (10.0, 0.0),
        (20.0, 0.0),
        (-10.0, 0.0),
        (-20.0, 0.0),
        (-7.0, 3.0),
        (27.0, 7.0),
        // len - abs(phase) rounds back to len: crbug.com/124652.
        (-f32::from_bits(1), 0.0),
    ] {
        let info = SkDashPathEffect::make(&[4.0, 2.0, 1.0, 3.0], phase)
            .unwrap()
            .as_a_dash()
            .unwrap();
        assert_eq!(info.f_phase, expected, "phase={phase:?}");
        assert_eq!(info.f_intervals, [4.0, 2.0, 1.0, 3.0]);
    }
}

#[test]
fn corner_radius_boundaries() {
    for radius in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(SkCornerPathEffect::make(radius).is_none());
    }
    for radius in [f32::from_bits(1), 1.0, f32::MAX] {
        assert!(
            SkCornerPathEffect::make(radius)
                .unwrap()
                .base()
                .as_a_dash()
                .is_none()
        );
    }
}

#[test]
fn dash_source_copy_shared_effect_and_owned_info_lifetimes() {
    let effect = {
        let mut source = vec![4.0, 2.0, 1.0, 3.0];
        let effect = SkDashPathEffect::make(&source, -7.0).unwrap();
        source.fill(99.0);
        effect
    };
    let survivor = effect.clone();
    drop(effect);
    let mut info = survivor.base().as_a_dash().unwrap();
    assert_eq!(info.f_intervals, [4.0, 2.0, 1.0, 3.0]);
    info.f_intervals[0] = 100.0;
    let retained = survivor.as_a_dash().unwrap();
    assert_eq!(retained.f_intervals, [4.0, 2.0, 1.0, 3.0]);
    drop(survivor);
    assert_eq!(retained.f_intervals, [4.0, 2.0, 1.0, 3.0]);
    assert_eq!(retained.f_phase, 3.0);
    assert_eq!(info.f_intervals, [100.0, 2.0, 1.0, 3.0]);
}

#[test]
fn stroke_rec_dense_layout_and_native_byte_order() {
    assert_eq!(std::mem::size_of::<SkStrokeRec>(), 16);
    assert_eq!(std::mem::align_of::<SkStrokeRec>(), 4);
    for cap in [SkPaintCap::Butt, SkPaintCap::Round, SkPaintCap::Square] {
        for join in [SkPaintJoin::Miter, SkPaintJoin::Round, SkPaintJoin::Bevel] {
            for filled in [false, true] {
                let style = if filled {
                    SkPaintStyle::StrokeAndFill
                } else {
                    SkPaintStyle::Stroke
                };
                let rec = SkStrokeRec::from_paint_params(style, 2.0, 4.0, cap, join, 3.0);
                // SAFETY: repr(C), three initialized f32s and an initialized u32,
                // no padding; size/alignment above assert the dense layout contract.
                let bytes: [u8; 16] = unsafe { std::mem::transmute(rec) };
                assert_eq!(&bytes[0..4], &3.0_f32.to_ne_bytes());
                assert_eq!(&bytes[4..8], &2.0_f32.to_ne_bytes());
                assert_eq!(&bytes[8..12], &4.0_f32.to_ne_bytes());
                let flags = cap as u32 | ((join as u32) << 16) | ((filled as u32) << 31);
                assert_eq!(&bytes[12..16], &flags.to_ne_bytes());
                assert_eq!(rec.get_cap(), cap);
                assert_eq!(rec.get_join(), join);
                assert_eq!(
                    rec.get_style(),
                    if filled {
                        SkStrokeRecStyle::StrokeAndFill
                    } else {
                        SkStrokeRecStyle::Stroke
                    }
                );
            }
        }
    }
}
