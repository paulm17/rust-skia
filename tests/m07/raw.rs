// Copyright 2025 Google LLC. BSD: tests/LICENSE.skia.
// PathRawShapesTest.cpp and PathRawTest.cpp.
use super::*;
fn path_from_raw(raw: &SkPathRaw) -> SkPath {
    SkPath::Raw(
        raw.points(),
        raw.verbs(),
        raw.conics(),
        raw.fill_type(),
        false,
    )
}
fn check_path_is_raw(r: &mut Reporter, path: &SkPath, raw: &SkPathRaw) {
    let praw = path.raw(SkResolveConvexity::No);
    check!(r, praw.is_some());
    let praw = praw.unwrap();
    check!(r, praw.f_points == raw.f_points);
    check!(r, praw.f_verbs == raw.f_verbs);
    check!(r, praw.f_conics == raw.f_conics);
    check!(r, praw.f_bounds == raw.f_bounds);
}
#[test]
fn pathrawshapes_rect() {
    let mut r = Reporter::default();
    let bounds = rect(1.0, 2.0, 3.0, 4.0);
    for d in DIRS {
        let shape = SkPathRawShapes::rect(&bounds, d, 0);
        check!(r, shape.bounds() == bounds);
        check!(r, shape.is_known_to_be_convex());
        check!(
            r,
            shape.segment_masks() == K_LINE_SK_PATH_SEGMENT_MASK as u32
        );
        let path = path_from_raw(&shape);
        check_path_is_raw(&mut r, &path, &shape);
    }
    r.finish();
}
#[test]
fn pathrawshapes_oval() {
    let mut r = Reporter::default();
    let bounds = rect(1.0, 2.0, 3.0, 4.0);
    for d in DIRS {
        let shape = SkPathRawShapes::oval(&bounds, d, 1);
        check!(r, shape.bounds() == bounds);
        check!(r, shape.is_known_to_be_convex());
        check!(
            r,
            shape.segment_masks() == K_CONIC_SK_PATH_SEGMENT_MASK as u32
        );
        let path = SkPath::oval_dir(&bounds, d);
        check_path_is_raw(&mut r, &path, &shape);
    }
    r.finish();
}
#[test]
fn pathrawshapes_rrect() {
    let mut r = Reporter::default();
    let bounds = rect(0.0, 0.0, 4.0, 4.0);
    let rr = RRect::make_rect_xy(&bounds, 1.0, 1.0);
    for d in DIRS {
        let shape = SkPathRawShapes::rrect_dirl(&rr, d);
        check!(r, shape.bounds() == bounds);
        check!(r, shape.is_known_to_be_convex());
        check!(
            r,
            shape.segment_masks()
                == (K_LINE_SK_PATH_SEGMENT_MASK | K_CONIC_SK_PATH_SEGMENT_MASK) as u32
        );
        let path = SkPath::rrect_dirl(&rr, d);
        check_path_is_raw(&mut r, &path, &shape);
    }
    r.finish();
}
// Original fixture builder uses caller-provided fixed-capacity storage. Keep explicit capacities
// and extension assertions; this helper does no production path normalization.
struct SkSPathRawBuilder {
    pts: Vec<Point>,
    vbs: Vec<SkPathVerb>,
    cns: Vec<f32>,
    pc: usize,
    vc: usize,
    cc: usize,
}
impl SkSPathRawBuilder {
    fn new(pc: usize, vc: usize, cc: usize) -> Self {
        Self {
            pts: Vec::new(),
            vbs: Vec::new(),
            cns: Vec::new(),
            pc,
            vc,
            cc,
        }
    }
    fn add(&mut self, v: SkPathVerb, pts: &[Point], w: Option<f32>) {
        debug_assert!(self.pts.len() + pts.len() <= self.pc);
        debug_assert!(self.vbs.len() + 1 <= self.vc);
        if let Some(w) = w {
            debug_assert!(self.cns.len() + 1 <= self.cc);
            self.cns.push(w);
        }
        self.pts.extend_from_slice(pts);
        self.vbs.push(v);
    }
    fn move_to(&mut self, p: Point) {
        self.add(M, &[p], None);
    }
    fn line_to(&mut self, p: Point) {
        self.add(L, &[p], None);
    }
    fn quad_to(&mut self, a: Point, b: Point) {
        self.add(Q, &[a, b], None);
    }
    fn conic_to(&mut self, a: Point, b: Point, w: f32) {
        self.add(K, &[a, b], Some(w));
    }
    fn cubic_to(&mut self, a: Point, b: Point, c: Point) {
        self.add(C, &[a, b, c], None);
    }
    fn close(&mut self) {
        self.add(X, &[], None);
    }
    fn raw(&self, ft: SkPathFillType, cv: SkPathConvexity) -> SkPathRaw {
        let mut bounds = Rect::make_empty();
        bounds.set_bounds_check(&self.pts);
        SkPathRaw::new(
            self.pts.clone(),
            self.vbs.clone(),
            self.cns.clone(),
            bounds,
            ft,
            cv,
            compute_segment_mask(&self.vbs),
        )
    }
}
fn check_iter(r: &mut Reporter, raw: &SkPathRaw, pts: &[Point], vbs: &[SkPathVerb], cns: &[f32]) {
    let (mut pi, mut vi, mut ci, mut mi) = (0, 0, 0, 0);
    let mut it = raw.iter();
    while let Some(rec) = it.next() {
        check!(r, vi < vbs.len());
        check!(r, vbs[vi] == rec.f_verb);
        vi += 1;
        match rec.f_verb {
            M => {
                mi = pi;
                check!(r, pi < pts.len());
                check!(r, pts[pi] == rec.f_points[0]);
                pi += 1;
            }
            L => {
                check!(r, pi < pts.len());
                check!(r, pts[pi - 1] == rec.f_points[0]);
                check!(r, pts[pi] == rec.f_points[1]);
                pi += 1;
            }
            Q | K => {
                check!(r, pi + 1 < pts.len());
                check!(r, pts[pi - 1] == rec.f_points[0]);
                check!(r, pts[pi] == rec.f_points[1]);
                pi += 1;
                check!(r, pts[pi] == rec.f_points[2]);
                pi += 1;
                if rec.f_verb == K {
                    check!(r, ci < cns.len());
                    check!(r, cns[ci] == rec.f_conic_weight);
                    ci += 1;
                }
            }
            C => {
                check!(r, pi + 2 < pts.len());
                check!(r, pts[pi - 1] == rec.f_points[0]);
                check!(r, pts[pi] == rec.f_points[1]);
                pi += 1;
                check!(r, pts[pi] == rec.f_points[2]);
                pi += 1;
                check!(r, pts[pi] == rec.f_points[3]);
                pi += 1;
            }
            X => {
                check!(r, pts[pi - 1] == rec.f_points[0]);
                check!(r, pts[mi] == rec.f_points[1]);
            }
            _ => unreachable!(),
        }
    }
    check!(r, it.next().is_none());
}
#[test]
fn pathraw_iter() {
    let mut r = Reporter::default();
    let pts: Vec<_> = (0..11).map(|i| p(i as f32, i as f32)).collect();
    let v = [M, L, Q, C, X, M, L, K];
    let mut bu = SkSPathRawBuilder::new(11, 8, 1);
    bu.move_to(pts[0]);
    bu.line_to(pts[1]);
    bu.quad_to(pts[2], pts[3]);
    bu.cubic_to(pts[4], pts[5], pts[6]);
    bu.close();
    bu.move_to(pts[7]);
    bu.line_to(pts[8]);
    bu.conic_to(pts[9], pts[10], 2.0);
    let raw = bu.raw(Winding, SkPathConvexity::Unknown);
    check!(r, raw.f_points.len() == 11);
    check!(r, raw.f_verbs.len() == 8);
    check!(r, raw.f_conics.len() == 1);
    check_iter(&mut r, &raw, &pts, &v, &bu.cns);
    let mut pb = SkPathBuilder::new();
    pb.move_to(pts[0])
        .line_to(pts[1])
        .quad_to(pts[2], pts[3])
        .cubic_to(pts[4], pts[5], pts[6])
        .close()
        .move_to(pts[7])
        .line_to(pts[8])
        .conic_to(pts[9], pts[10], 2.0);
    let path = pb.detach(None);
    let raw = path.raw(SkResolveConvexity::No).unwrap();
    check_iter(&mut r, &raw, &pts, &v, &bu.cns);
    r.finish();
}
fn check_mask(r: &mut Reporter, desc: &str, mask: u8, build: fn(&mut SkSPathRawBuilder)) {
    eprintln!("{desc}");
    let mut bu = SkSPathRawBuilder::new(20, 20, 10);
    build(&mut bu);
    let raw = bu.raw(Winding, SkPathConvexity::Unknown);
    check!(r, raw.f_segment_mask == mask);
}
#[test]
fn pathraw_segmentmask() {
    let mut r = Reporter::default();
    check_mask(&mut r, "move-only", 0, |b| b.move_to(p(0.0, 0.0)));
    check_mask(&mut r, "line", 1, |b| {
        b.move_to(p(0.0, 0.0));
        b.line_to(p(1.0, 1.0));
    });
    check_mask(&mut r, "quad", 2, |b| {
        b.move_to(p(0.0, 0.0));
        b.quad_to(p(1.0, 1.0), p(2.0, 2.0));
    });
    check_mask(&mut r, "conic", 4, |b| {
        b.move_to(p(0.0, 0.0));
        b.conic_to(p(1.0, 1.0), p(2.0, 2.0), 0.5);
    });
    check_mask(&mut r, "cubic", 8, |b| {
        b.move_to(p(0.0, 0.0));
        b.cubic_to(p(1.0, 1.0), p(2.0, 2.0), p(3.0, 3.0));
    });
    check_mask(&mut r, "line-quad", 1 | 2, |b| {
        b.move_to(p(0.0, 0.0));
        b.line_to(p(1.0, 1.0));
        b.quad_to(p(2.0, 2.0), p(3.0, 3.0));
    });
    check_mask(&mut r, "conic-cubic", 4 | 8, |b| {
        b.move_to(p(0.0, 0.0));
        b.conic_to(p(1.0, 1.0), p(2.0, 2.0), 0.5);
        b.cubic_to(p(3.0, 3.0), p(4.0, 4.0), p(5.0, 5.0));
    });
    check_mask(&mut r, "all", 1 | 2 | 4 | 8, |b| {
        b.move_to(p(0.0, 0.0));
        b.line_to(p(1.0, 1.0));
        b.quad_to(p(2.0, 2.0), p(3.0, 3.0));
        b.conic_to(p(4.0, 4.0), p(5.0, 5.0), 0.5);
        b.cubic_to(p(6.0, 6.0), p(7.0, 7.0), p(8.0, 8.0));
        b.close();
    });
    check_mask(&mut r, "empty path", 0, |b| {
        b.move_to(p(0.0, 0.0));
        b.close();
    });
    r.finish();
}
