// Copyright 2025 Google LLC. BSD: tests/LICENSE.skia. PathDataTest.cpp.
use super::*;
type PD = Arc<SkPathData>;
fn check_asA_transforms(r: &mut Reporter, orig: PD, pred: fn(&SkPathData) -> bool) {
    debug_assert!(pred(&orig));
    for (m, expected) in [
        (Matrix::new(), true),
        (translate(1.0, 2.0), true),
        (scale(2.0, 3.0), true),
        (rotate(30.0), false),
    ] {
        let data = orig.make_transform_from(&m).unwrap();
        check!(r, pred(&data) == expected);
    }
}
// The six original RectMaker variants; the explicit fifth point is retained.
fn make_rect(r: &Rect, d: SkPathDirection, variant: usize) -> PD {
    let pts = if d == Cw {
        vec![
            p(r.left, r.top),
            p(r.right, r.top),
            p(r.right, r.bottom),
            p(r.left, r.bottom),
        ]
    } else {
        vec![
            p(r.left, r.top),
            p(r.left, r.bottom),
            p(r.right, r.bottom),
            p(r.right, r.top),
        ]
    };
    let mut pts = pts;
    if variant == 2 || variant == 5 {
        pts.push(pts[0]);
    }
    match variant {
        0 => SkPathData::rect(r, d, 0).unwrap(),
        1 | 2 => SkPathData::polygon(&pts, true),
        3 => SkPathBuilder::new().add_rect(r, d, 0).detach_data(),
        _ => SkPathBuilder::new().add_polygon(&pts, true).detach_data(),
    }
}
fn make_oval(r: &Rect, d: SkPathDirection, start: u32, builder: bool) -> PD {
    if builder {
        SkPathBuilder::new().add_oval(r, d, start).detach_data()
    } else {
        SkPathData::oval(r, d, start).unwrap()
    }
}
fn make_rrect(rr: &RRect, d: SkPathDirection, start: u32, builder: bool) -> PD {
    if builder {
        SkPathBuilder::new().add_rrect(rr, d, start).detach_data()
    } else {
        SkPathData::rrect(rr, d, start).unwrap()
    }
}
#[test]
fn pathdata_empty() {
    let mut r = Reporter::default();
    let data = SkPathData::empty();
    check!(r, data.is_empty());
    check!(r, data.points().is_empty());
    check!(r, data.conics().is_empty());
    check!(r, data.verbs().is_empty());
    check!(r, *data.bounds() == Rect::make_empty());
    check!(r, data.segment_mask() == 0);
    check!(r, data.as_line().is_none());
    check!(r, data.as_oval().is_none());
    check!(r, data.as_rect().is_none());
    check!(r, data.as_rrect().is_none());
    let x = data.make_transform_from(&scale(2.0, 3.0)).unwrap();
    check!(r, *data == *x);
    r.finish();
}
#[test]
fn pathdata_rect() {
    let mut r = Reporter::default();
    let bounds = rect(1.0, 2.0, 3.0, 4.0);
    for maker in 0..6 {
        for d in DIRS {
            let data = make_rect(&bounds, d, maker);
            check!(r, bounds == *data.bounds());
            let pts = data.points();
            let mut b = Rect::make_empty();
            b.set_bounds_check(pts);
            check!(r, bounds == b);
            let cross_sign = if d == Cw { 1.0 } else { -1.0 };
            for i in 1..3 {
                let u = pts[i] - pts[i - 1];
                let v = pts[i + 1] - pts[i];
                let cross = u.cross(&v);
                let dot = u.dot(&v);
                check!(r, dot == 0.0);
                let sign = if cross == 0.0 {
                    0.0
                } else if cross > 0.0 {
                    1.0
                } else {
                    -1.0
                };
                check!(r, cross_sign == sign);
            }
            let isa = data.as_rect();
            check!(r, isa.is_some());
            let isa = isa.unwrap();
            check!(r, isa.f_rect == bounds);
            check!(r, isa.f_direction == d);
            check!(r, isa.f_start_index == 0);
            check_asA_transforms(&mut r, data, |pd| pd.as_rect().is_some());
        }
    }
    r.finish();
}
#[test]
fn pathdata_polygon() {
    let mut r = Reporter::default();
    let points = [
        p(0.0, 1.0),
        p(2.0, 3.0),
        p(4.0, 5.0),
        p(6.0, 7.0),
        p(8.0, 9.0),
    ];
    for builder in [false, true] {
        for closed in [false, true] {
            for n in 0..=points.len() {
                let pts = &points[..n];
                let data = if builder {
                    SkPathBuilder::new().add_polygon(pts, closed).detach_data()
                } else {
                    SkPathData::polygon(pts, closed)
                };
                let empty = if closed { n == 0 } else { n <= 1 };
                if empty {
                    check!(r, data.is_empty());
                    continue;
                }
                check!(r, data.points() == pts);
                check!(r, data.conics().is_empty());
                let line = data.as_line();
                if n == 2 && !closed {
                    check!(r, line.is_some());
                    let line = line.unwrap();
                    check!(r, line[0] == points[0]);
                    check!(r, line[1] == points[1]);
                    let pline = SkPathData::line(points[0], points[1]);
                    check!(r, *pline == *data);
                } else {
                    check!(r, line.is_none());
                }
                let v = data.verbs();
                check!(r, v.len() == pts.len() + usize::from(closed));
                check!(r, v[0] == M);
                for i in 1..pts.len() {
                    check!(r, v[i] == L);
                }
                if closed {
                    check!(r, v.last() == Some(&X));
                }
            }
        }
    }
    r.finish();
}
#[test]
fn pathdata_oval() {
    let mut r = Reporter::default();
    let bounds = rect(1.0, 2.0, 3.0, 4.0);
    for builder in [false, true] {
        for d in DIRS {
            for start in 0..4 {
                let data = make_oval(&bounds, d, start, builder);
                check!(r, *data.bounds() == bounds);
                let oval = data.as_oval();
                check!(r, oval.is_some());
                let oval = oval.unwrap();
                check!(r, oval.f_bounds == bounds);
                check!(r, oval.f_direction == d);
                check!(r, oval.f_start_index as u32 == start);
                check_asA_transforms(&mut r, data, |pd| pd.as_oval().is_some());
            }
        }
    }
    r.finish();
}
#[test]
fn pathdata_rrect() {
    let mut r = Reporter::default();
    let bounds = rect(0.0, 0.0, 20.0, 30.0);
    let rr = RRect::make_rect_xy(&bounds, 2.0, 3.0);
    for builder in [false, true] {
        for d in DIRS {
            for start in 0..8 {
                let data = make_rrect(&rr, d, start, builder);
                check!(r, *data.bounds() == bounds);
                let info = data.as_rrect();
                check!(r, info.is_some());
                let info = info.unwrap();
                check!(r, info.f_rrect == rr);
                check!(r, info.f_direction == d);
                check!(r, info.f_start_index as u32 == start);
                check_asA_transforms(&mut r, data, |pd| pd.as_rrect().is_some());
            }
        }
    }
    r.finish();
}
#[test]
fn pathdata_make_edgecases() {
    let mut r = Reporter::default();
    let pts: Vec<_> = (0..20).map(|i| p(i as f32 * 1.0, i as f32 * 1.0)).collect();
    let weights = [1.5, 2.0, 3.0];
    check!(r, SkPathData::make(&[], &[], &[]).unwrap().is_empty());
    check!(
        r,
        !SkPathData::make(&pts[..1], &[M], &[]).unwrap().is_empty()
    );
    check!(r, SkPathData::make(&pts[..1], &[L], &[]).is_none());
    check!(r, SkPathData::make(&pts[..3], &[M, M, L], &[]).is_none());
    check!(r, SkPathData::make(&pts[..2], &[M, L, X, X], &[]).is_none());
    check!(r, SkPathData::make(&pts[..4], &[M, L, M, M], &[]).is_none());
    let data = SkPathData::make(&pts[..3], &[M, L, M], &[]).unwrap();
    check!(r, data.points().len() == 3);
    check!(r, data.verbs().len() == 3);
    let mut b = Rect::make_empty();
    b.set_bounds_check(&pts[..2]);
    check!(r, *data.bounds() == b);
    let v = [M, L, Q, K, C, X, M];
    for (n, w, success) in [
        (10, 1, true),
        (9, 1, false),
        (11, 1, false),
        (10, 0, false),
        (10, 2, false),
        (0, 1, false),
    ] {
        let data = SkPathData::make(&pts[..n], &v, &weights[..w]);
        if success {
            check!(r, data.is_some());
        } else {
            check!(r, data.is_none());
        }
    }
    r.finish();
}
fn make_bad_rrect() -> Option<RRect> {
    let big = f32::MAX;
    let mut rr = RRect::make_rect_xy(&rect(0.0, 0.0, big, big), 4.0, 4.0);
    rr.offset(big, big);
    if !rr.rect().is_finite() {
        Some(rr)
    } else {
        None
    }
}
#[test]
fn pathdata_make_nonfinite() {
    let mut r = Reporter::default();
    let inf = f32::INFINITY;
    let mut pts = [p(0.0, 0.0), p(inf, 1.0), p(2.0, 4.0)];
    let v = [M, K];
    let mut weights = [2.0];
    check!(r, SkPathData::make(&pts, &v, &weights).is_none());
    pts[1].x = 3.0;
    for bad in [-1.0, inf, -inf, inf * 0.0] {
        weights[0] = bad;
        check!(r, SkPathData::make(&pts, &v, &weights).is_none());
    }
    let bounds = rect(1.0, 2.0, inf, 4.0);
    check!(r, SkPathData::rect(&bounds, Cw, 0).is_none());
    check!(r, SkPathData::oval(&bounds, Cw, 1).is_none());
    if let Some(rr) = make_bad_rrect() {
        check!(r, SkPathData::rrect(&rr, Cw, 6).is_none());
    }
    pts[1].x = inf;
    // Polygon's Rust API returns an Arc; failure is represented by the error singleton.
    check!(
        r,
        Arc::ptr_eq(
            &SkPathData::polygon(&pts, false),
            &SkPathData::peek_error_singleton()
        )
    );
    check!(
        r,
        SkPathData::make(&[p(0.0, 0.0), p(10.0, 10.0), p(inf, 20.0)], &[M, L, M], &[]).is_none()
    );
    r.finish();
}
#[test]
fn pathdata_transform() {
    let mut r = Reporter::default();
    let bounds = rect(10.0, 20.0, 30.0, 40.0);
    let data = SkPathData::oval(&bounds, Cw, 1).unwrap();
    let newd = data.make_transform_from(&Matrix::new()).unwrap();
    check!(r, *newd == *data);
    let newd = data.make_transform_from(&translate(5.0, 6.0)).unwrap();
    check!(r, *newd.bounds() == bounds.make_offset(5.0, 6.0));
    let newd = data.make_transform_from(&scale(0.5, 2.0)).unwrap();
    check!(
        r,
        *newd.bounds()
            == rect(
                bounds.left * 0.5,
                bounds.top * 2.0,
                bounds.right * 0.5,
                bounds.bottom * 2.0
            )
    );
    check!(
        r,
        data.make_transform_from(&scale(f32::INFINITY, 2.0))
            .is_none()
    );
    check!(r, data.make_transform_from(&scale(f32::NAN, 2.0)).is_none());
    r.finish();
}
#[test]
fn pathdata_transform_convexity() {
    let mut r = Reporter::default();
    let pts = [p(0.0, 0.0), p(100.0, 0.0), p(200.0, 0.0), p(200.0, 200.0)];
    check!(r, is_axis_aligned(&pts));
    let src = SkPathData::polygon(&pts, true);
    check!(
        r,
        src.get_convexity_or_unknown() == SkPathConvexity::Unknown
    );
    let mut raw = src.raw(Winding, SkResolveConvexity::No);
    check!(r, raw.f_convexity == SkPathConvexity::Unknown);
    raw = src.raw(Winding, SkResolveConvexity::Yes);
    check!(r, raw.is_known_to_be_convex());
    let cs = [
        SkPathConvexity::Unknown,
        SkPathConvexity::ConvexCw,
        SkPathConvexity::Concave,
    ];
    for mx in [Matrix::new(), translate(1.0, 2.0), scale(2.0, 3.0)] {
        for conv in cs {
            raw.f_convexity = conv;
            let dst = SkPathData::make_transform(&raw, &mx).unwrap();
            check!(r, dst.get_convexity_or_unknown() == conv);
        }
    }
    let mx = rotate(30.0);
    for conv in cs {
        raw.f_convexity = conv;
        let dst = SkPathData::make_transform(&raw, &mx).unwrap();
        let expected = if conv.is_convex() {
            SkPathConvexity::Unknown
        } else {
            conv
        };
        check!(r, dst.get_convexity_or_unknown() == expected);
    }
    r.finish();
}
#[test]
fn pathdata_inverted_bounds() {
    let mut r = Reporter::default();
    let bounds = rect(-10.0, -10.0, 10.0, 10.0);
    let inverted = rect(10.0, 10.0, -10.0, -10.0);
    for maker in [0, 3] {
        for d in DIRS {
            for b in [bounds, inverted] {
                check!(r, *make_rect(&b, d, maker).bounds() == bounds);
            }
        }
    }
    for builder in [false, true] {
        for d in DIRS {
            for start in 0..4 {
                for b in [bounds, inverted] {
                    check!(r, *make_oval(&b, d, start, builder).bounds() == bounds);
                }
            }
        }
    }
    for builder in [false, true] {
        for d in DIRS {
            for start in 0..8 {
                for sign in [1.0, -1.0] {
                    for b in [bounds, inverted] {
                        let rr = RRect::make_rect_xy(&b, sign * 2.0, sign * 3.0);
                        check!(r, *make_rrect(&rr, d, start, builder).bounds() == bounds);
                    }
                }
            }
        }
    }
    r.finish();
}
