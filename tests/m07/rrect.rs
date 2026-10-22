// Copyright 2015 Google Inc. / 2025 Google LLC. BSD: tests/LICENSE.skia.
// RRectInPathTest.cpp; RoundRectTest.cpp:1786,1805.
use super::*;
fn path_contains_rrect(r: &mut Reporter, path: &SkPath) -> SkPathRRectInfo {
    let info = path.rrect_info();
    check!(r, info.is_some());
    let info = info.unwrap();
    let mut recreated = SkPath::rrect(&info.f_rrect, info.f_direction, info.f_start_index as u32);
    check!(r, *path == recreated);
    for m in [
        scale(1.0, 1.0),
        scale(-1.0, 1.0),
        scale(1.0, -1.0),
        scale(-1.0, -1.0),
    ] {
        let xformed = path.make_transform(&m);
        let xi = xformed.rrect_info();
        check!(r, xi.is_some());
        let xi = xi.unwrap();
        recreated = SkPath::rrect(&xi.f_rrect, xi.f_direction, xi.f_start_index as u32);
        check!(r, recreated == xformed);
    }
    info
}
fn inner_path_contains_rrect(
    r: &mut Reporter,
    input: &RRect,
    d: SkPathDirection,
    start: u32,
) -> RRect {
    if input.is_empty() || input.is_rect() || input.is_oval() {
        return *input;
    }
    let path = SkPath::rrect(input, d, start);
    let rr = path_contains_rrect(r, &path);
    check!(r, rr.f_direction == d && rr.f_start_index as u32 == start);
    rr.f_rrect
}
fn path_contains_rrect_check(r: &mut Reporter, input: &RRect, d: SkPathDirection, start: u32) {
    let out = inner_path_contains_rrect(r, input, d, start);
    if *input != out {
        eprint!("");
    }
    check!(r, *input == out);
}
fn path_contains_rrect_nocheck(r: &mut Reporter, input: &RRect, d: SkPathDirection, start: u32) {
    let out = inner_path_contains_rrect(r, input, d, start);
    if *input == out {
        eprint!("");
    }
}
fn radii_check(
    r: &mut Reporter,
    bounds: &Rect,
    radii: &[Point; 4],
    d: SkPathDirection,
    start: u32,
) {
    let mut rr = RRect::make_empty();
    rr.set_rect_radii(bounds, radii);
    path_contains_rrect_check(r, &rr, d, start);
}
fn test_tricky_radii(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            {
                let bounds = rect(3709.0, 3709.0, 3709.0 + 7402.0, 3709.0 + 29825.0);
                let rad = 12814.0;
                let v = [p(rad, rad), p(0.0, rad), p(rad, rad), p(0.0, rad)];
                let mut rr = RRect::make_empty();
                rr.set_rect_radii(&bounds, &v);
                path_contains_rrect_check(r, &rr, d, start);
            }
            {
                let bounds = rect(0.0, 0.0, 1009.0, 33554432.0);
                let v = [
                    p(13.0, 8.0),
                    p(170.0, 2.0),
                    p(256.0, 33554432.0),
                    p(110.0, 5.0),
                ];
                let mut rr = RRect::make_empty();
                rr.set_rect_radii(&bounds, &v);
                path_contains_rrect_nocheck(r, &rr, d, start);
            }
        }
    }
}
fn test_empty_crbug_458524(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(3709.0, 3709.0, 3709.0 + 7402.0, 3709.0 + 29825.0);
            let mut rr = RRect::make_empty();
            rr.set_rect_xy(&bounds, 40.0, 40.0);
            path_contains_rrect_check(r, &rr, d, start);
            let _ = rr.transform(&scale(0.0, 1.0));
            path_contains_rrect_check(r, &rr, d, start);
        }
    }
}
fn test_inset(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(0.0, 0.0, 100.0, 100.0);
            let mut rr = RRect::make_empty();
            let mut rr2 = RRect::make_empty();
            rr.set_rect(&bounds);
            rr.inset(-20.0, -20.0, &mut rr2);
            path_contains_rrect_check(r, &rr, d, start);
            rr.inset(20.0, 20.0, &mut rr2);
            path_contains_rrect_check(r, &rr, d, start);
            rr.inset(bounds.width() / 2.0, bounds.height() / 2.0, &mut rr2);
            path_contains_rrect_check(r, &rr, d, start);
            rr.set_rect_xy(&bounds, 20.0, 20.0);
            rr.inset(19.0, 19.0, &mut rr2);
            path_contains_rrect_check(r, &rr, d, start);
            rr.inset(20.0, 20.0, &mut rr2);
            path_contains_rrect_check(r, &rr, d, start);
        }
    }
}
fn test_9patch_rrect(
    r: &mut Reporter,
    bounds: &Rect,
    l: f32,
    t: f32,
    right: f32,
    b: f32,
    check_radii: bool,
) {
    for d in DIRS {
        for start in 0..8 {
            let mut rr = RRect::make_empty();
            rr.set_nine_patch(bounds, l, t, right, b);
            if check_radii {
                path_contains_rrect_check(r, &rr, d, start);
            } else {
                path_contains_rrect_nocheck(r, &rr, d, start);
            }
            let mut rr2 = RRect::make_empty();
            let v = [p(l, t), p(right, t), p(right, b), p(l, b)];
            rr2.set_rect_radii(bounds, &v);
            // Upstream deliberately calls with rr again, not rr2.
            if check_radii {
                path_contains_rrect_check(r, &rr, d, start);
            } else {
                path_contains_rrect_nocheck(r, &rr, d, start);
            }
        }
    }
}
fn test_round_rect_basic(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(0.0, 0.0, 100.0, 100.0);
            let mut rr1 = RRect::make_empty();
            rr1.set_rect(&bounds);
            path_contains_rrect_check(r, &rr1, d, start);
            let mut rr1_2 = RRect::make_empty();
            rr1_2.set_rect_radii(&bounds, &[p(0.0, 0.0); 4]);
            path_contains_rrect_check(r, &rr1_2, d, start);
            let mut rr1_3 = RRect::make_empty();
            rr1_3.set_nine_patch(&bounds, 0.0, 0.0, 0.0, 0.0);
            path_contains_rrect_check(r, &rr1_2, d, start);
            let half = p(100.0 / 2.0, 100.0 / 2.0);
            let mut rr2 = RRect::make_empty();
            rr2.set_oval(&bounds);
            path_contains_rrect_check(r, &rr2, d, start);
            let mut rr2_2 = RRect::make_empty();
            rr2_2.set_rect_radii(&bounds, &[p(half.x, half.y); 4]);
            path_contains_rrect_check(r, &rr2_2, d, start);
            let mut rr2_3 = RRect::make_empty();
            rr2_3.set_nine_patch(&bounds, half.x, half.y, half.x, half.y);
            path_contains_rrect_check(r, &rr2_3, d, start);
            let point = p(5.0, 5.0);
            let mut rr3 = RRect::make_empty();
            rr3.set_rect_xy(&bounds, point.x, point.y);
            path_contains_rrect_check(r, &rr3, d, start);
            let mut rr3_2 = RRect::make_empty();
            rr3_2.set_rect_radii(&bounds, &[p(5.0, 5.0); 4]);
            path_contains_rrect_check(r, &rr3_2, d, start);
            let mut rr3_3 = RRect::make_empty();
            rr3_3.set_nine_patch(&bounds, 5.0, 5.0, 5.0, 5.0);
            path_contains_rrect_check(r, &rr3_3, d, start);
            test_9patch_rrect(r, &bounds, 10.0, 9.0, 8.0, 7.0, true);
            {
                let rect2 = rect(0.358211994, 0.755430222, 0.872866154, 0.806214333);
                test_9patch_rrect(
                    r,
                    &rect2,
                    0.926942348,
                    0.642850280,
                    0.529063463,
                    0.587844372,
                    false,
                );
            }
            let radii = [p(0.0, 0.0), p(0.0, 0.0), p(50.0, 50.0), p(20.0, 50.0)];
            let mut rr5 = RRect::make_empty();
            rr5.set_rect_radii(&bounds, &radii);
            path_contains_rrect_check(r, &rr5, d, start);
        }
    }
}
fn test_round_rect_rects(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(0.0, 0.0, 100.0, 100.0);
            let mut rr1 = RRect::make_empty();
            rr1.set_rect_xy(&bounds, 0.0, 0.0);
            path_contains_rrect_check(r, &rr1, d, start);
            let mut rr2 = RRect::make_empty();
            rr2.set_rect_radii(&bounds, &[p(0.0, 0.0); 4]);
            path_contains_rrect_check(r, &rr2, d, start);
            let mut rr3 = RRect::make_empty();
            rr3.set_rect_radii(
                &bounds,
                &[p(0.0, 0.0), p(20.0, 20.0), p(50.0, 50.0), p(20.0, 50.0)],
            );
            path_contains_rrect_check(r, &rr3, d, start);
        }
    }
}
fn test_round_rect_ovals(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(0.0, 0.0, 100.0, 100.0);
            let mut rr1 = RRect::make_empty();
            rr1.set_rect_xy(&bounds, 100.0 / 2.0, 100.0 / 2.0);
            path_contains_rrect_check(r, &rr1, d, start);
        }
    }
}
fn test_round_rect_general(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(0.0, 0.0, 100.0, 100.0);
            let mut rr1 = RRect::make_empty();
            rr1.set_rect_xy(&bounds, 20.0, 20.0);
            path_contains_rrect_check(r, &rr1, d, start);
            let mut rr2 = RRect::make_empty();
            rr2.set_rect_radii(
                &bounds,
                &[p(0.0, 0.0), p(20.0, 20.0), p(50.0, 50.0), p(20.0, 50.0)],
            );
            path_contains_rrect_check(r, &rr2, d, start);
        }
    }
}
fn test_round_rect_iffy_parameters(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let bounds = rect(0.0, 0.0, 100.0, 100.0);
            let mut rr1 = RRect::make_empty();
            rr1.set_rect_radii(
                &bounds,
                &[
                    p(50.0, 100.0),
                    p(100.0, 50.0),
                    p(50.0, 100.0),
                    p(100.0, 50.0),
                ],
            );
            path_contains_rrect_nocheck(r, &rr1, d, start);
        }
    }
}
fn set_radii(radii: &mut [Point; 4], index: usize, rad: f32) {
    *radii = [Point::default(); 4];
    radii[index].set(rad, rad);
}
fn test_skbug_3239(r: &mut Reporter) {
    let min = f32::from_bits(0xcb7f16c8);
    let max = f32::from_bits(0x4b7f1c1d);
    let big = f32::from_bits(0x4b7f1bd7);
    let rad = 33436320.0;
    let rectx = rect(min, min, max, big);
    let recty = rect(min, min, big, max);
    for d in DIRS {
        for start in 0..8 {
            let mut radii = [Point::default(); 4];
            for i in 0..4 {
                set_radii(&mut radii, i, rad);
                radii_check(r, &rectx, &radii, d, start);
                radii_check(r, &recty, &radii, d, start);
            }
        }
    }
}
fn test_mix(r: &mut Reporter) {
    for d in DIRS {
        for start in 0..8 {
            let radii = [p(0.0, 0.0), p(0.0, 0.0), p(0.0, 0.0), p(100.0, 100.0)];
            let bounds = Rect::make_wh(100.0, 100.0);
            let mut rr = RRect::make_empty();
            rr.set_rect_radii(&bounds, &radii);
            path_contains_rrect_check(r, &rr, d, start);
        }
    }
}
#[test]
fn RoundRectInPath() {
    let mut r = Reporter::default();
    test_tricky_radii(&mut r);
    test_empty_crbug_458524(&mut r);
    test_inset(&mut r);
    test_round_rect_basic(&mut r);
    test_round_rect_rects(&mut r);
    test_round_rect_ovals(&mut r);
    test_round_rect_general(&mut r);
    test_round_rect_iffy_parameters(&mut r);
    test_skbug_3239(&mut r);
    test_mix(&mut r);
    r.finish();
}
#[test]
fn RRect_fragile() {
    let bounds = rect(
        f32::from_bits(0x1f800000),
        f32::from_bits(0x1400001C),
        f32::from_bits(0x3F000004),
        f32::from_bits(0x3F000004),
    );
    let radii = [
        p(f32::from_bits(1), f32::from_bits(1)),
        p(f32::from_bits(0x20), f32::from_bits(1)),
        p(f32::from_bits(0), f32::from_bits(0)),
        p(f32::from_bits(0x3F000004), f32::from_bits(0x3F000004)),
    ];
    let mut rr = RRect::make_empty();
    if false {
        eprintln!("{} 0x{:08X}", bounds.left, bounds.left.to_bits());
        rr.set_rect_radii(&bounds, &radii);
    } // Original disabled body.
}
#[test]
fn RRect_b561770646_part1() {
    let mut r = Reporter::default();
    let bounds = rect(0.0, 1e-30, 100.0, 100.0);
    let radii = [p(0.0, 0.0), p(0.0, 0.0), p(0.0, 0.0), p(10.0, 100.0)];
    let mut rr = RRect::make_empty();
    rr.set_rect_radii(&bounds, &radii);
    check!(r, rr.is_valid());
    let path = SkPath::rrect(&rr, Cw, 0);
    let mut out = RRect::make_empty();
    let _ = path.is_rrect(Some(&mut out));
    r.finish();
}
#[test]
fn RRect_b561770646_part2() {
    let mut r = Reporter::default();
    let rr = RRect::make_rect_xy(&Rect::make_wh(100.0, 100.0), 10.0, 10.0);
    let mx = translate(1e20, 0.0);
    let transformed = rr.transform(&mx);
    check!(r, transformed.is_none());
    let path = SkPath::rrect_default(&rr);
    let transformed = path.make_transform(&mx);
    let mut out = RRect::make_empty();
    check!(r, !transformed.is_rrect(Some(&mut out)));
    r.finish();
}
