// Copyright 2020–2025 Google LLC. BSD: tests/LICENSE.skia. PathBuilderTest.cpp.
use super::*;
fn is_empty(r: &mut Reporter, path: &SkPath) {
    check!(r, path.get_bounds().is_empty());
    check!(r, path.count_points() == 0);
}
fn is_eq(a: &SkPath, b: &SkPath) -> bool {
    if a != b {
        return false;
    }
    let mut ra = Rect::make_empty();
    let mut rb = ra;
    let ia = a.is_oval(Some(&mut ra));
    let ib = b.is_oval(Some(&mut rb));
    if ia != ib || (ia && ra != rb) {
        return false;
    }
    let mut ra = RRect::make_empty();
    let mut rb = ra;
    let ia = a.is_rrect(Some(&mut ra));
    let ib = b.is_rrect(Some(&mut rb));
    if ia != ib || (ia && ra != rb) {
        return false;
    }
    a.get_convexity_or_unknown() == b.get_convexity_or_unknown()
}
fn check_points(path: &SkPath, expected: &[Point]) -> bool {
    let mut pts = Vec::new();
    let mut it = range(path);
    while let Some((v, p, _)) = it.next() {
        match v {
            M => pts.push(p[0]),
            L => pts.push(p[1]),
            Q | K => pts.extend_from_slice(&p[1..3]),
            C => pts.extend_from_slice(&p[1..4]),
            _ => {}
        }
    }
    if pts.len() != expected.len() {
        return false;
    }
    for i in 0..expected.len() {
        if pts[i] != expected[i] {
            return false;
        }
    }
    true
}
#[test]
fn pathbuilder() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    check!(r, b.is_empty());
    is_empty(&mut r, &b.snapshot(None));
    is_empty(&mut r, &b.detach(None));
    b.move_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 20.0)
        .quad_to_xy(30.0, 10.0, 10.0, 20.0);
    check!(r, b.count_points() == 4);
    let p0 = b.snapshot(None);
    let p1 = b.snapshot(None);
    let p2 = b.detach(None);
    check!(r, *p0.get_bounds() == rect(10.0, 10.0, 30.0, 20.0));
    check!(r, p0.count_points() == 4);
    check!(r, p0 == p1);
    check!(r, p0 == p2);
    check!(r, b.is_empty());
    is_empty(&mut r, &b.snapshot(None));
    is_empty(&mut r, &b.detach(None));
    r.finish();
}
#[test]
fn pathbuilder_filltype() {
    let mut r = Reporter::default();
    for ft in FILL_TYPES {
        let mut b = SkPathBuilder::new_with_fill_type(ft);
        check!(r, b.get_fill_type() == ft);
        check!(r, b.is_inverse_fill_type() == ft.is_inverse());
        for path in [b.snapshot(None), b.detach(None)] {
            check!(r, path.get_fill_type() == ft);
            is_empty(&mut r, &path);
        }
    }
    r.finish();
}
#[test]
fn pathbuilder_missing_move() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    b.line_to_xy(10.0, 10.0).line_to_xy(20.0, 30.0);
    check!(
        r,
        check_points(
            &b.snapshot(None),
            &[p(0.0, 0.0), p(10.0, 10.0), p(20.0, 30.0)]
        )
    );
    b.reset()
        .move_to_xy(20.0, 20.0)
        .line_to_xy(10.0, 10.0)
        .line_to_xy(20.0, 30.0)
        .close()
        .line_to_xy(60.0, 60.0);
    check!(
        r,
        check_points(
            &b.snapshot(None),
            &[
                p(20.0, 20.0),
                p(10.0, 10.0),
                p(20.0, 30.0),
                p(20.0, 20.0),
                p(60.0, 60.0)
            ]
        )
    );
    r.finish();
}
#[test]
fn pathbuilder_addRect() {
    let mut r = Reporter::default();
    let bounds = rect(10.0, 20.0, 30.0, 40.0);
    for i in 0..4 {
        for d in DIRS {
            let mut b = SkPathBuilder::new();
            b.add_rect(&bounds, d, i);
            let mut bp = b.detach(None);
            let mut r2 = Rect::make_empty();
            let mut closed = false;
            let mut d2 = Cw;
            check!(r, bp.is_convex());
            check!(
                r,
                bp.is_rect(Some(&mut r2), Some(&mut closed), Some(&mut d2))
            );
            check!(r, r2 == bounds);
            check!(r, closed);
            check!(r, d == d2);
            let path = SkPath::rect_dir(&bounds, d, i);
            check!(r, path == bp);
            b.add_rect(&bounds, d, i)
                .move_to_xy(3.0, 4.0)
                .line_to_xy(4.0, 5.0);
            bp = b.detach(None);
            check!(r, !bp.is_convex());
            check!(
                r,
                !bp.is_rect(Some(&mut r2), Some(&mut closed), Some(&mut d2))
            );
        }
    }
    r.finish();
}
#[test]
fn pathbuilder_addOval() {
    let mut r = Reporter::default();
    let bounds = rect(10.0, 20.0, 30.0, 40.0);
    let mut tmp = Rect::make_empty();
    for d in DIRS {
        for i in 0..4 {
            let bp = SkPathBuilder::new().add_oval(&bounds, d, i).detach(None);
            let path = SkPath::oval(&bounds, d, i);
            check!(r, is_eq(&path, &bp));
            let mut b = Rect::make_empty();
            check!(r, path.is_oval(Some(&mut b)));
            check!(r, bp.is_oval(Some(&mut b)));
            check!(r, path.is_convex());
            check!(r, bp.is_convex());
        }
        let bp = SkPathBuilder::new().add_oval(&bounds, d, 1).detach(None);
        let path = SkPath::oval_dir(&bounds, d);
        check!(r, is_eq(&path, &bp));
        let bp = SkPathBuilder::new()
            .add_oval(&bounds, d, 1)
            .line_to_xy(10.0, 10.0)
            .detach(None);
        check!(r, !bp.is_oval(Some(&mut tmp)));
        let bp = SkPathBuilder::new()
            .line_to_xy(10.0, 10.0)
            .add_oval(&bounds, d, 1)
            .detach(None);
        check!(r, !bp.is_oval(Some(&mut tmp)));
    }
    r.finish();
}
#[test]
fn pathbuilder_addRRect() {
    let mut r = Reporter::default();
    let rr = RRect::make_rect_xy(&rect(10.0, 20.0, 30.0, 40.0), 5.0, 6.0);
    for d in DIRS {
        for i in 0..4 {
            let mut b = SkPathBuilder::new();
            b.add_rrect(&rr, d, i);
            let bp = b.detach(None);
            let path = SkPath::rrect(&rr, d, i);
            check!(r, is_eq(&path, &bp));
        }
        let start = if d == Cw { 6 } else { 7 };
        let bp = SkPathBuilder::new().add_rrect(&rr, d, start).detach(None);
        let path = SkPath::rrect_dirl(&rr, d);
        check!(r, is_eq(&path, &bp));
        let mut tmp = RRect::make_empty();
        let bp = SkPathBuilder::new()
            .add_rrect(&rr, d, start)
            .line_to_xy(10.0, 10.0)
            .detach(None);
        check!(r, !bp.is_rrect(Some(&mut tmp)));
        let bp = SkPathBuilder::new()
            .line_to_xy(10.0, 10.0)
            .add_rrect(&rr, d, start)
            .detach(None);
        check!(r, !bp.is_rrect(Some(&mut tmp)));
    }
    r.finish();
}
#[test]
fn pathbuilder_make() {
    let mut r = Reporter::default();
    let mut v = [M; 100];
    let mut pts = [p(0.0, 0.0); 100];
    let mut rand = SkRandom::new();
    let mut b = SkPathBuilder::new();
    b.move_to_xy(0.0, 0.0);
    for i in 1..100 {
        let x = rand.next_f();
        let y = rand.next_f();
        b.line_to_xy(x, y);
        pts[i] = p(x, y);
        v[i] = L;
    }
    let p0 = b.detach(None);
    let p1 = SkPath::Raw(&pts, &v, &[], p0.get_fill_type(), false);
    check!(r, p0 == p1);
    r.finish();
}
#[test]
fn pathbuilder_genid() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    b.line_to_xy(10.0, 10.0);
    let p1 = b.snapshot(None);
    b.line_to_xy(10.0, 20.0);
    let p2 = b.snapshot(None);
    check!(r, p1.get_generation_id() != p2.get_generation_id());
    r.finish();
}
#[test]
fn pathbuilder_addPolygon() {
    let mut r = Reporter::default();
    let pts = [p(1.0, 2.0), p(3.0, 4.0), p(5.0, 6.0), p(7.0, 8.0)];
    for closed in [false, true] {
        for n in 0..=pts.len() {
            let p0 = SkPathBuilder::new()
                .add_polygon(&pts[..n], closed)
                .detach(None);
            let mut b = SkPathBuilder::new();
            if n > 0 {
                b.move_to(pts[0]);
                for i in 1..n {
                    b.line_to(pts[i]);
                }
                if closed {
                    b.close();
                }
            }
            let p1 = b.detach(None);
            check!(r, p0 == p1);
        }
    }
    r.finish();
}
fn test_addPath(r: &mut Reporter) {
    let mut b = SkPathBuilder::new();
    let mut q = SkPathBuilder::new();
    b.line_to_xy(1.0, 2.0);
    q.move_to_xy(4.0, 4.0)
        .line_to_xy(7.0, 8.0)
        .conic_to_xy(8.0, 7.0, 6.0, 5.0, 0.5)
        .quad_to_xy(6.0, 7.0, 8.0, 6.0)
        .cubic_to_xy(5.0, 6.0, 7.0, 8.0, 7.0, 5.0)
        .close();
    b.add_path_offset(&q.snapshot(None), -4.0, -4.0, AddPathMode::Append);
    check!(
        r,
        *b.snapshot(None).get_bounds() == rect(0.0, 0.0, 4.0, 4.0)
    );
    b.reset();
    b.private_reverse_add_path(&q.snapshot(None));
    check!(
        r,
        *b.snapshot(None).get_bounds() == rect(4.0, 4.0, 8.0, 8.0)
    );
}
fn test_addPathMode(r: &mut Reporter, explicit: bool, extend: bool) {
    let mut b = SkPathBuilder::new();
    let mut q = SkPathBuilder::new();
    if explicit {
        b.move_to_xy(1.0, 1.0);
    }
    b.line_to_xy(1.0, 2.0);
    if explicit {
        q.move_to_xy(2.0, 1.0);
    }
    q.line_to_xy(2.0, 2.0);
    b.add_path(
        &q.snapshot(None),
        if extend {
            AddPathMode::Extend
        } else {
            AddPathMode::Append
        },
    );
    let v = b.verbs();
    check!(r, v.len() == 4);
    check!(r, v[0] == M);
    check!(r, v[1] == L);
    check!(r, v[2] == if extend { L } else { M });
    check!(r, v[3] == L);
}
fn get_point(b: &SkPathBuilder, i: i32) -> Option<Point> {
    b.points().get(i as usize).copied()
}
fn test_extendClosedPath(r: &mut Reporter) {
    let mut b = SkPathBuilder::new();
    let mut q = SkPathBuilder::new();
    b.move_to_xy(1.0, 1.0)
        .line_to_xy(1.0, 2.0)
        .line_to_xy(2.0, 2.0)
        .close();
    q.move_to_xy(2.0, 1.0).line_to_xy(2.0, 3.0);
    b.add_path(&q.detach(None), AddPathMode::Extend);
    let v = b.verbs();
    check!(r, v.len() == 7);
    for (i, expected) in [M, L, L, X, M, L, L].into_iter().enumerate() {
        check!(r, v[i] == expected);
    }
    let pt = b.get_last_pt();
    check!(r, pt.is_some());
    check!(r, pt.unwrap() == p(2.0, 3.0));
    let pt = get_point(&b, 3);
    check!(r, pt.is_some());
    check!(r, pt == Some(p(1.0, 1.0)));
}
fn test_addEmptyPath(r: &mut Reporter, mode: AddPathMode) {
    let mut b = SkPathBuilder::new();
    let mut q = SkPathBuilder::new();
    let empty = SkPathBuilder::new();
    b.move_to_xy(2.0, 1.0).line_to_xy(2.0, 3.0);
    q.add_path(&b.snapshot(None), mode);
    check!(r, q.snapshot(None) == b.snapshot(None));
    b.add_path(&empty.snapshot(None), mode);
    check!(r, q.snapshot(None) == b.snapshot(None));
    q.reset();
    q.add_path(&empty.snapshot(None), mode);
    check!(r, q.is_empty());
}
fn test_before_after_lineto(r: &mut Reporter, b: &mut SkPathBuilder, last: Point, mov: Point) {
    let pt = get_point(b, b.count_points() - 1);
    check!(r, pt.is_some());
    check!(r, pt.unwrap() == last);
    let new = p(1234.0, 5678.0);
    b.line_to(new);
    let pt = get_point(b, b.count_points() - 2);
    check!(r, pt.is_some());
    check!(r, pt.unwrap() == mov);
    let pt = get_point(b, b.count_points() - 1);
    check!(r, pt.is_some());
    check!(r, pt.unwrap() == new);
}
fn test_addPath_and_injected_moveTo(r: &mut Reporter) {
    let mut p1 = SkPathBuilder::new();
    p1.move_to_xy(230.0, 230.0)
        .move_to_xy(20.0, 30.0)
        .line_to_xy(40.0, 30.0)
        .line_to_xy(40.0, 50.0)
        .line_to_xy(20.0, 50.0);
    let mut p1c = SkPathBuilder::from_path(&p1.snapshot(None));
    p1c.close();
    let mut p2 = SkPathBuilder::new();
    p2.move_to_xy(144.0, 72.0).line_to_xy(146.0, 72.0);
    let mut p2c = SkPathBuilder::from_path(&p2.snapshot(None));
    p2c.close();
    let mut p3 = SkPathBuilder::from_path(&p2.snapshot(None));
    let mut p3c = SkPathBuilder::from_path(&p2c.snapshot(None));
    p2.add_path(&p1c.snapshot(None), AddPathMode::Append);
    p2c.add_path(&p1c.snapshot(None), AddPathMode::Append);
    for b in [&mut p1c, &mut p2, &mut p2c] {
        test_before_after_lineto(r, b, p(20.0, 50.0), p(20.0, 30.0));
    }
    p3.add_path(&p1.snapshot(None), AddPathMode::Append);
    p3c.add_path(&p1.snapshot(None), AddPathMode::Append);
    for b in [&mut p1, &mut p3, &mut p3c] {
        test_before_after_lineto(r, b, p(20.0, 50.0), p(20.0, 50.0));
    }
}
fn test_addPath_convexity(r: &mut Reporter) {
    let circle = SkPath::circle(10.0, 10.0, 10.0, Cw);
    check!(r, circle.is_convex());
    for (mov, mode, expected) in [
        (false, AddPathMode::Append, true),
        (true, AddPathMode::Append, true),
        (false, AddPathMode::Extend, true),
        (true, AddPathMode::Extend, false),
    ] {
        let mut b = SkPathBuilder::new();
        if mov {
            b.move_to_xy(0.0, 0.0);
        }
        b.add_path(&circle, mode);
        let path = b.detach(None);
        check!(r, path.is_convex() == expected);
    }
    let mut b = SkPathBuilder::new();
    check!(r, b.snapshot(None).is_convex());
    b.add_path(&circle, AddPathMode::Append);
    check!(r, b.snapshot(None).is_convex());
    b.add_path(&circle, AddPathMode::Append);
    check!(r, !b.snapshot(None).is_convex());
}
#[test]
fn pathbuilder_addPath() {
    let mut r = Reporter::default();
    let path = SkPathBuilder::new()
        .move_to_xy(10.0, 10.0)
        .line_to_xy(100.0, 10.0)
        .quad_to_xy(200.0, 100.0, 100.0, 200.0)
        .close()
        .move_to_xy(200.0, 200.0)
        .cubic_to_xy(210.0, 200.0, 210.0, 300.0, 200.0, 300.0)
        .conic_to_xy(150.0, 250.0, 100.0, 200.0, 1.4)
        .detach(None);
    check!(
        r,
        path == SkPathBuilder::new()
            .add_path(&path, AddPathMode::Append)
            .detach(None)
    );
    test_addPath(&mut r);
    test_addPathMode(&mut r, false, false);
    test_addPathMode(&mut r, true, false);
    test_addPathMode(&mut r, false, true);
    test_addPathMode(&mut r, true, true);
    test_extendClosedPath(&mut r);
    test_addEmptyPath(&mut r, AddPathMode::Extend);
    test_addEmptyPath(&mut r, AddPathMode::Append);
    test_addPath_and_injected_moveTo(&mut r);
    test_addPath_convexity(&mut r);
    r.finish();
}
#[test]
fn pathbuilder_addpath_crbug_1153516() {
    let mut r = Reporter::default();
    let mut p1 = SkPathBuilder::new();
    let mut p2 = SkPathBuilder::new();
    p2.line_to_xy(10.0, 20.0);
    p1.add_rect(&rect(143.0, 226.0, 200.0, 241.0), Cw, 0);
    p2.add_path(&p1.snapshot(None), AddPathMode::Append);
    p2.line_to_xy(262.0, 513.0);
    let start = p(143.0, 226.0);
    let end = p(262.0, 513.0);
    let mov = get_point(&p2, p2.count_points() - 2);
    check!(r, mov.is_some());
    check!(r, mov.unwrap() == start);
    let line = get_point(&p2, p2.count_points() - 1);
    check!(r, line.is_some());
    check!(r, line.unwrap() == end);
    let snap = p1.snapshot(None);
    p1.add_path(&snap, AddPathMode::Append)
        .line_to_xy(262.0, 513.0);
    let mov = get_point(&p1, p1.count_points() - 2);
    check!(r, mov.is_some());
    check!(r, mov.unwrap() == start);
    let line = get_point(&p1, p1.count_points() - 1);
    check!(r, line.is_some());
    check!(r, line.unwrap() == end);
    r.finish();
}
fn assertIsMoveTo(r: &mut Reporter, it: &mut RangeIter<'_>, x: f32, y: f32) {
    let (v, pts, _) = it.next().unwrap();
    check!(r, v == M);
    check!(r, pts[0].x == x);
    check!(r, pts[0].y == y);
}
fn assertIsLineTo(r: &mut Reporter, it: &mut RangeIter<'_>, x: f32, y: f32) {
    let (v, pts, _) = it.next().unwrap();
    check!(r, v == L);
    check!(r, pts[1].x == x);
    check!(r, pts[1].y == y);
}
fn assertIsDone(r: &mut Reporter, it: &RangeIter<'_>, _path: &SkPath) {
    check!(r, it.at_end());
}
#[test]
fn SkPathBuilder_multipleMoveTos() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    check!(r, b.is_empty());
    b.move_to_xy(1.0, 2.0);
    check!(r, b.points().len() == 1);
    let pt = b.get_last_pt();
    check!(r, pt.is_some());
    check!(r, pt.unwrap() == p(1.0, 2.0));
    check!(r, b.compute_bounds() == Rect::make_xywh(1.0, 2.0, 0.0, 0.0));
    b.move_to_xy(3.0, 4.0)
        .move_to_xy(5.0, 6.0)
        .move_to_xy(7.0, 8.0);
    check!(r, b.points().len() == 1);
    let pt = b.get_last_pt();
    check!(r, pt.is_some());
    check!(r, pt.unwrap() == p(7.0, 8.0));
    check!(r, b.compute_bounds() == Rect::make_xywh(7.0, 8.0, 0.0, 0.0));
    r.finish();
}
#[test]
fn SkPathBuilder_lineToMoveTo() {
    let mut r = Reporter::default();
    let path = SkPathBuilder::new()
        .move_to_xy(20.0, 3.0)
        .line_to_xy(7.0, 11.0)
        .line_to_xy(8.0, 12.0)
        .move_to_xy(2.0, 3.0)
        .line_to_xy(20.0, 30.0)
        .detach(None);
    let mut it = range(&path);
    assertIsMoveTo(&mut r, &mut it, 20.0, 3.0);
    assertIsLineTo(&mut r, &mut it, 7.0, 11.0);
    assertIsLineTo(&mut r, &mut it, 8.0, 12.0);
    assertIsMoveTo(&mut r, &mut it, 2.0, 3.0);
    assertIsLineTo(&mut r, &mut it, 20.0, 30.0);
    assertIsDone(&mut r, &it, &path);
    r.finish();
}
#[test]
fn SkPathBuilder_arcToPtPtRad_invalidInputsResultInALine() {
    let mut r = Reporter::default();
    for (name, start, end, rad, expected) in [
        (
            "first point equals previous point",
            p(0.0, 0.0),
            p(1.0, 2.0),
            1.0,
            p(0.0, 0.0),
        ),
        (
            "two points equal",
            p(5.0, 7.0),
            p(5.0, 7.0),
            1.0,
            p(5.0, 7.0),
        ),
        (
            "radius is zero",
            p(-3.0, 5.0),
            p(-7.0, 11.0),
            0.0,
            p(-3.0, 5.0),
        ),
        (
            "second point equals previous point",
            p(5.0, 4.0),
            p(0.0, 0.0),
            1.0,
            p(5.0, 4.0),
        ),
    ] {
        let path = SkPathBuilder::new()
            .arc_to_pts(start, end, rad)
            .detach(None);
        eprintln!("{name}");
        let mut it = range(&path);
        assertIsMoveTo(&mut r, &mut it, 0.0, 0.0);
        assertIsLineTo(&mut r, &mut it, expected.x, expected.y);
        assertIsDone(&mut r, &it, &path);
    }
    r.finish();
}
#[test]
fn SkPathBuilder_assign() {
    let mut r = Reporter::default();
    let pts = [p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0)];
    for vol in [false, true] {
        let src = SkPath::polygon(&pts, false, Winding, vol);
        let mut b = SkPathBuilder::from_path(&src);
        let dst = b.detach(None);
        check!(r, src == dst);
        check!(r, src.is_volatile() == dst.is_volatile());
    }
    r.finish();
}
#[test]
fn SkPathBuilder_getLastPt() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    check!(r, b.get_last_pt() == None);
    b.move_to_xy(10.0, 10.0);
    let pt = b.get_last_pt();
    check!(r, pt.is_some());
    check!(r, pt == Some(p(10.0, 10.0)));
    b.r_line_to_xy(10.0, 10.0);
    let pt = b.get_last_pt();
    check!(r, pt == Some(p(20.0, 20.0)));
    r.finish();
}
#[test]
fn SkPathBuilder_transform() {
    let mut r = Reporter::default();
    let pts = [
        p(0.0, 0.0),
        p(10.0, 10.0),
        p(20.0, 10.0),
        p(20.0, 0.0),
        p(0.0, 0.0),
        p(0.0, 10.0),
        p(1.0, 10.0),
    ];
    // Original CONIC_PERSPECTIVE_BUG_FIXED is 0: the two conic points and conic call stay excluded.
    let mut b = SkPathBuilder::new();
    b.move_to(pts[0])
        .line_to(pts[1])
        .quad_to(pts[2], pts[3])
        .cubic_to(pts[4], pts[5], pts[6])
        .close();
    let p1 = SkPathBuilder::from_path(&b.snapshot(None))
        .transform(&Matrix::new())
        .detach(None);
    check!(r, b.snapshot(None) == p1);
    let p1 = SkPathBuilder::from_path(&b.snapshot(None))
        .transform(&scale(2.0, 3.0))
        .detach(None);
    let pts1 = p1.points();
    check!(r, pts.len() == pts1.len());
    for i in 0..pts1.len() {
        check!(r, p(pts[i].x * 2.0, pts[i].y * 3.0) == pts1[i]);
    }
    let mut mx = Matrix::new();
    mx.set_persp_x(4.0);
    let mut b1 = SkPathBuilder::from_path(&b.snapshot(None));
    b1.move_to_xy(0.0, 0.0).transform(&mx);
    let mut inv = Matrix::new();
    check!(r, mx.invert(Some(&mut inv)));
    mx = inv;
    b1.transform(&mx);
    let pb = *b.snapshot(None).get_bounds();
    let p1b = *b1.detach(None).get_bounds();
    check!(r, near(pb.left, p1b.left));
    check!(r, near(pb.top, p1b.top));
    check!(r, near(pb.right, p1b.right));
    check!(r, near(pb.bottom, p1b.bottom));
    b.reset().add_circle(p(0.0, 0.0), 1.0, Cw);
    for (mx, expected) in [
        (Matrix::new(), SkPathFirstDirection::Cw),
        (scale(-1.0, 1.0), SkPathFirstDirection::Ccw),
        (
            Matrix::make_all(1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            SkPathFirstDirection::Unknown,
        ),
    ] {
        let path = SkPathBuilder::from_path(&b.snapshot(None))
            .move_to_xy(0.0, 0.0)
            .transform(&mx)
            .detach(None);
        check!(
            r,
            compute_first_direction(&path.raw(SkResolveConvexity::No).unwrap()) == expected
        );
    }
    r.finish();
}
#[test]
fn SkPathBuilder_cleaning() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    b.move_to_xy(1.0, 2.0).close().close();
    let v = b.verbs();
    check!(r, v.len() == 2);
    check!(r, v[0] == M);
    check!(r, v[1] == X);
    let pts = b.points();
    check!(r, pts.len() == 1);
    check!(r, pts[0] == p(1.0, 2.0));
    r.finish();
}
fn check_roundtrip(r: &mut Reporter, path: &SkPath) {
    let rpath = SkPathBuilder::from_path(path).detach(None);
    check!(r, *path == rpath);
    check!(r, path.is_convex() == rpath.is_convex());
    let mx = rotate(30.0);
    let bupath = SkPathBuilder::from_path(path).transform(&mx).detach(None);
    let copy = path.make_transform(&mx);
    let mut bounds = Rect::make_empty();
    let ovals = [
        path.is_oval(Some(&mut bounds)),
        rpath.is_oval(Some(&mut bounds)),
        copy.is_oval(Some(&mut bounds)),
        bupath.is_oval(Some(&mut bounds)),
    ];
    check!(r, ovals[0] == ovals[1]);
    check!(r, ovals[2] == false);
    check!(r, ovals[3] == false);
    check!(r, bupath.is_convex() == copy.is_convex());
    let a = path.oval_info();
    let b = rpath.oval_info();
    check!(r, a.is_some() == b.is_some());
    if let (Some(a), Some(b)) = (a, b) {
        check!(r, a.f_bounds == b.f_bounds);
        check!(r, a.f_direction == b.f_direction);
        check!(r, a.f_start_index == b.f_start_index);
    }
    let a = path.rrect_info();
    let b = rpath.rrect_info();
    check!(r, a.is_some() == b.is_some());
    if let (Some(a), Some(b)) = (a, b) {
        check!(r, a.f_rrect == b.f_rrect);
        check!(r, a.f_direction == b.f_direction);
        check!(r, a.f_start_index == b.f_start_index);
    }
}
#[test]
fn SkPathBuilder_path_roundtrip() {
    let mut r = Reporter::default();
    let bounds = rect(10.0, 20.0, 30.0, 40.0);
    for path in [
        SkPath::new(),
        SkPath::circle(10.0, 20.0, 30.0, Ccw),
        SkPath::oval(&bounds, Ccw, 2),
        SkPath::rect_dir(&bounds, Ccw, 2),
        SkPath::rrect_xy(&bounds, 1.0, 2.0, Ccw),
        SkPathBuilder::new()
            .line_to_xy(100.0, 0.0)
            .quad_to(p(0.0, 0.0), p(0.0, 100.0))
            .close()
            .detach(None),
    ] {
        check_roundtrip(&mut r, &path);
    }
    r.finish();
}
fn check_move(r: &mut Reporter, it: &mut SkPathIter<'_>, x: f32, y: f32) {
    let rec = it.next().unwrap();
    check!(r, rec.f_verb == M);
    check!(r, rec.f_points[0].x == x);
    check!(r, rec.f_points[0].y == y);
}
fn check_line(r: &mut Reporter, it: &mut SkPathIter<'_>, x: f32, y: f32) {
    let rec = it.next().unwrap();
    check!(r, rec.f_verb == L);
    check!(r, rec.f_points[1].x == x);
    check!(r, rec.f_points[1].y == y);
}
fn check_close(r: &mut Reporter, it: &mut SkPathIter<'_>) {
    let rec = it.next().unwrap();
    check!(r, rec.f_verb == X);
}
#[test]
fn SkPathBuilder_rMoveTo() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    b.move_to_xy(10.0, 11.0)
        .line_to_xy(20.0, 21.0)
        .close()
        .r_move_to(p(30.0, 31.0))
        .line_to_xy(30.0, 40.0);
    {
        let mut it = SkPathIter::new(b.points(), b.verbs(), &[]);
        check_move(&mut r, &mut it, 10.0, 11.0);
        check_line(&mut r, &mut it, 20.0, 21.0);
        check_close(&mut r, &mut it);
        check_move(&mut r, &mut it, 10.0 + 30.0, 11.0 + 31.0);
        check_line(&mut r, &mut it, 30.0, 40.0);
        check!(r, it.next().is_none());
    }
    b.reset();
    b.move_to_xy(10.0, 11.0)
        .line_to_xy(20.0, 21.0)
        .r_move_to_xy(30.0, 31.0)
        .line_to_xy(30.0, 40.0);
    {
        let mut it = b.iter();
        check_move(&mut r, &mut it, 10.0, 11.0);
        check_line(&mut r, &mut it, 20.0, 21.0);
        check_move(&mut r, &mut it, 20.0 + 30.0, 21.0 + 31.0);
        check_line(&mut r, &mut it, 30.0, 40.0);
        check!(r, it.next().is_none());
    }
    b.reset();
    b.r_move_to(p(30.0, 31.0));
    {
        let mut it = b.iter();
        check!(r, it.next().is_none());
    }
    b.reset();
    r.finish();
}
fn check_filltype_eq(r: &mut Reporter, a: &SkPathBuilder) {
    let mut copy = a.clone();
    check!(r, *a == copy);
    for ft in FILL_TYPES {
        if ft != a.get_fill_type() {
            copy.set_fill_type(ft);
            check!(r, *a != copy);
        }
    }
}
fn set_segments(b: &mut SkPathBuilder) {
    b.reset()
        .move_to_xy(1.0, 2.0)
        .line_to_xy(3.0, 4.0)
        .quad_to_xy(5.0, 6.0, 7.0, 8.0)
        .conic_to_xy(9.0, 10.0, 11.0, 12.0, 0.5)
        .cubic_to_xy(13.0, 14.0, 15.0, 16.0, 17.0, 18.0)
        .close();
}
#[test]
fn SkPathBuilder_equality() {
    let mut r = Reporter::default();
    let mut a = SkPathBuilder::new();
    let mut b = SkPathBuilder::new();
    check!(r, a == b);
    check_filltype_eq(&mut r, &a);
    a.move_to_xy(0.0, 0.0);
    check!(r, a != b);
    b.move_to_xy(0.0, 0.0);
    check!(r, a == b);
    check_filltype_eq(&mut r, &a);
    b.close();
    check!(r, a != b);
    a.close();
    check!(r, a == b);
    check_filltype_eq(&mut r, &a);
    set_segments(&mut a);
    set_segments(&mut b);
    check!(r, a == b);
    check_filltype_eq(&mut r, &a);
    a.set_last_point(p(-1.0, -2.0));
    check!(r, a != b);
    check_filltype_eq(&mut r, &a);
    r.finish();
}
#[test]
fn SkPathBuilder_dump() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    b.move_to_xy(1.0, 2.0)
        .line_to_xy(3.0, 4.0)
        .quad_to_xy(5.0, 6.0, 7.0, 8.0)
        .conic_to_xy(9.0, 10.0, 11.0, 12.0, 0.5)
        .cubic_to_xy(13.0, 14.0, 15.0, 16.0, 17.0, 18.0)
        .close()
        .move_to_xy(1.0, 2.0)
        .line_to_xy(3.0, 4.0);
    let s = b.dump_to_string(DumpFormat::Decimal);
    let expected = "SkPathBuilder(SkPathFillType::kWinding)\n.moveTo(1, 2)\n.lineTo(3, 4)\n.quadTo(5, 6, 7, 8)\n.conicTo(9, 10, 11, 12, 0.5f)\n.cubicTo(13, 14, 15, 16, 17, 18)\n.close()\n.moveTo(1, 2)\n.lineTo(3, 4)\n";
    check!(r, s.as_str() == expected);
    r.finish();
}
#[test]
fn SkPathBuilder_trailingmove_addpath() {
    let mut r = Reporter::default();
    for mx in [
        Matrix::new(),
        Matrix::make_all(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.005, 0.001, 1.0),
    ] {
        {
            let src = SkPath::new();
            let mut b = SkPathBuilder::new();
            b.line_to_xy(10.0, 10.0).move_to_xy(20.0, 20.0);
            let res = b
                .add_path_matrix(&src, &mx, AddPathMode::Append)
                .detach(None);
            check!(r, !res.is_empty());
            check!(r, res.points().last() == Some(&p(20.0, 20.0)));
        }
        {
            let src = SkPathBuilder::new().line_to_xy(100.0, 100.0).detach(None);
            let mut b = SkPathBuilder::new();
            b.line_to_xy(10.0, 10.0).move_to_xy(20.0, 20.0);
            let res = b
                .add_path_matrix(&src, &mx, AddPathMode::Append)
                .detach(None);
            check!(r, !res.is_empty());
            check!(r, res.points().last() != Some(&p(20.0, 20.0)));
        }
        {
            let src = SkPathBuilder::new().move_to_xy(100.0, 100.0).detach(None);
            let mut b = SkPathBuilder::new();
            b.line_to_xy(10.0, 10.0).move_to_xy(20.0, 20.0);
            let res = b
                .add_path_matrix(&src, &mx, AddPathMode::Append)
                .detach(None);
            check!(r, !res.is_empty());
            check!(r, res.points().last() != Some(&p(20.0, 20.0)));
        }
    }
    r.finish();
}
#[test]
fn SkPathBuilder_b_463584612() {
    let mut r = Reporter::default();
    let mut b = SkPathBuilder::new();
    // SK_SUPPORT_LEGACY_PATHBUILDER_SETLASTPT is not defined in the pinned default configuration.
    b.move_to_xy(0.0, 0.0).close().r_move_to_xy(1.0, 1.0);
    let mut builder = SkPathBuilder::new();
    builder
        .add_path_matrix(&b.snapshot(None), &Matrix::new(), AddPathMode::Extend)
        .r_move_to_xy(3.0, 4.0);
    let path = builder.detach(None);
    check!(r, !path.is_empty());
    r.finish();
}
#[test]
fn SkPathBuilder_conic_semantics() {
    let mut r = Reporter::default();
    {
        let path = SkPathBuilder::new()
            .conic_to(p(10.0, 5.0), p(10.0, 10.0), 0.5)
            .detach(None);
        check!(r, !path.is_empty());
        check!(r, path.verbs().len() == 2);
        check!(r, path.points().len() == 3);
        check!(r, path.conic_weights().len() == 1);
        check!(r, path.get_segment_masks() == 4);
    }
    {
        let path = SkPathBuilder::new()
            .conic_to(p(10.0, 5.0), p(10.0, 10.0), 1.0)
            .detach(None);
        check!(r, !path.is_empty());
        check!(r, path.verbs().len() == 2);
        check!(r, path.points().len() == 3);
        check!(r, path.conic_weights().is_empty());
        check!(r, path.get_segment_masks() == 2);
    }
    {
        let path = SkPathBuilder::new()
            .conic_to(p(10.0, 5.0), p(10.0, 10.0), f32::INFINITY)
            .detach(None);
        check!(r, !path.is_empty());
        check!(r, path.verbs().len() == 3);
        check!(r, path.points().len() == 3);
        check!(r, path.conic_weights().is_empty());
        check!(r, path.get_segment_masks() == 1);
    }
    {
        let path = SkPathBuilder::new()
            .conic_to(p(10.0, 5.0), p(10.0, 10.0), 0.0)
            .detach(None);
        check!(r, !path.is_empty());
        check!(r, path.verbs().len() == 2);
        check!(r, path.points().len() == 2);
        check!(r, path.conic_weights().is_empty());
        check!(r, path.get_segment_masks() == 1);
    }
    r.finish();
}
#[test]
fn SkPathBuilder_b520944501() {
    let path = SkPathBuilder::new_with_fill_type(SkPathFillType::InverseEvenOdd)
        .move_to_xy(0.0, 0.0)
        .cubic_to_xy(
            8.51330895e-18,
            4.80215861e30,
            4.80215347e30,
            4.73888368e-38,
            2.36942783e-38,
            7.54830852e-30,
        )
        .move_to_xy(4.80215831e30, 1.03969141e-21)
        .conic_to_xy(
            4.80215831e30,
            4.73888368e-38,
            2.40107915e30,
            4.73888368e-38,
            0.707106769,
        )
        .conic_to_xy(
            2.0779294e-21,
            4.73888368e-38,
            2.0779294e-21,
            1.03969141e-21,
            0.707106769,
        )
        .conic_to_xy(
            2.0779294e-21,
            2.07938282e-21,
            2.40107915e30,
            2.07938282e-21,
            0.707106769,
        )
        .conic_to_xy(
            4.80215831e30,
            2.07938282e-21,
            4.80215831e30,
            1.03969141e-21,
            0.707106769,
        )
        .close()
        .detach(None);
    let mx = Matrix::make_all(
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        10159839284371128320.0,
        36261335138304000.0,
        0.0,
        -0.0,
    );
    let transformed = path.make_transform(&mx);
    let meas = SkPathMeasure::from_path(&transformed, false, 1.0);
    let _ = meas.get_length();
}
