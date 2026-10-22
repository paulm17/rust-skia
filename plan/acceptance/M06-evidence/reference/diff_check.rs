// Temporary M06 differential check (Deepseek development check, not a Codex acceptance test).
// It prints the Rust results in the same layout as target/m06-evidence/ref_dump.cpp so the two
// dumps can be compared numerically.
#![allow(clippy::all)]
use rust_skia::*;

use std::fs;

struct Out {
    buf: String,
}

impl Out {
    fn line(&mut self, s: String) {
        self.buf.push_str(&s);
        self.buf.push('\n');
    }

    fn floats(&mut self, tag: &str, values: &[f32]) {
        let mut s = format!("{tag} {}", values.len());
        for v in values {
            s.push_str(&format!(" {v:e}"));
        }
        self.line(s);
    }

    fn points(&mut self, tag: &str, values: &[Point]) {
        let mut s = format!("{tag} {}", values.len());
        for p in values {
            s.push_str(&format!(" {:e} {:e}", p.x, p.y));
        }
        self.line(s);
    }

    fn doubles(&mut self, tag: &str, values: &[f64]) {
        let mut s = tag.to_string();
        for v in values {
            s.push_str(&format!(" {v:e}"));
        }
        self.line(s);
    }
}

fn verb_len(verb: SkPathVerb) -> usize {
    match verb {
        SkPathVerb::Move => 1,
        SkPathVerb::Line => 2,
        SkPathVerb::Quad => 3,
        SkPathVerb::Cubic => 4,
        _ => 0,
    }
}

fn dump_cubic(out: &mut Out, p: &[Point], index: usize) {
    out.line(format!("C {index}"));
    let mut t = [0.0; 2];
    let mut s = [0.0; 2];
    let mut d = [0.0; 4];
    let cubic_type = sk_classify_cubic(p, Some(&mut t), Some(&mut s), Some(&mut d));
    out.line(format!("type {}", cubic_type as i32));
    out.doubles("ts", &[t[0], t[1], s[0], s[1]]);
    out.doubles("d", &d);

    let mut t_values = [0.0; 3];
    let mut inf_arr = [0.0f32; 2];
    let inflections = sk_find_cubic_inflections(p, &mut inf_arr);
    out.floats("inf", &inf_arr[..inflections as usize]);

    let max_curvature = sk_find_cubic_max_curvature(p, &mut t_values);
    out.floats("mc", &t_values[..max_curvature as usize]);

    out.line(format!("cusp {:e}", sk_find_cubic_cusp(p)));

    let mut dst = [Point::default(); 13];
    let chopped = sk_chop_cubic_at_inflections(p, &mut dst);
    out.points("ichop", &dst[..(chopped as usize) * 3 + 1]);
    let chopped = sk_chop_cubic_at_max_curvature(p, Some(&mut dst), None);
    out.points("mchop", &dst[..(chopped as usize) * 3 + 1]);

    let y_extrema = sk_chop_cubic_at_y_extrema(p, &mut dst);
    out.points("yext", &dst[..(y_extrema as usize) * 3 + 1]);
    let x_extrema = sk_chop_cubic_at_x_extrema(p, &mut dst);
    out.points("xext", &dst[..(x_extrema as usize) * 3 + 1]);

    let bounds = Rect::bounds_or_empty(p);
    let mut mono = [Point::default(); 7];
    let chopped_y = sk_chop_mono_cubic_at_y(p, bounds.center_y(), &mut mono);
    let mut s = format!("monoy {}", chopped_y as i32);
    if chopped_y {
        for point in &mono {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
    }
    out.line(s);
    let chopped_x = sk_chop_mono_cubic_at_x(p, bounds.center_x(), &mut mono);
    let mut s = format!("monox {}", chopped_x as i32);
    if chopped_x {
        for point in &mono {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
    }
    out.line(s);

    let clip = Rect::new(0.0, 0.0, 10.0, 10.0);
    let mut lines = [Point::default(); sk_line_clipper::K_MAX_POINTS];
    let endpoints = [p[0], p[3]];
    let mut intersection = [Point::default(); 2];
    let intersects = intersect_line(&endpoints, &clip, &mut intersection);
    let mut s = format!("intersect {}", intersects as i32);
    if intersects {
        for point in &intersection {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
    }
    out.line(s);

    let segment_count = clip_line(&endpoints, &clip, &mut lines, false);
    if segment_count > 0 {
        out.points("clip", &lines[..segment_count as usize + 1]);
    } else {
        out.line("clip 0".to_string());
    }

    let mut clipper = SkEdgeClipper::new(false);
    let clipped = clipper.clip_cubic(&[p[0], p[1], p[2], p[3]], &clip);
    let mut s = format!("edge {}", clipped as i32);
    if clipped {
        let mut verb_pts = [Point::default(); 4];
        while let Some(verb) = clipper.next(&mut verb_pts) {
            s.push_str(&format!(" {}", verb as i32));
            for point in verb_pts.iter().take(verb_len(verb)) {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
        }
    }
    out.line(s);

    let mut line_clipper = SkEdgeClipper::new(false);
    let line_clipped = line_clipper.clip_line(p[0], p[1], &clip);
    let mut s = format!("edgeline {}", line_clipped as i32);
    if line_clipped {
        let mut verb_pts = [Point::default(); 4];
        while let Some(verb) = line_clipper.next(&mut verb_pts) {
            s.push_str(&format!(" {}", verb as i32));
            for point in verb_pts.iter().take(verb_len(verb)) {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
        }
    }
    out.line(s);

    match SkCubicClipper::chop_mono_at_y(&[p[0], p[1], p[2], p[3]], bounds.center_y()) {
        Some(t) => out.line(format!("cubicchop 1 {t:e}")),
        None => out.line("cubicchop 0 -1e0".to_string()),
    }

    let mut cubic_clipper = SkCubicClipper::new();
    cubic_clipper.set_clip(&IRect::new(0, 0, 10, 10));
    let mut clipped_cubic = [Point::default(); 4];
    let ok = cubic_clipper.clip_cubic(&[p[0], p[1], p[2], p[3]], &mut clipped_cubic);
    let mut s = format!("cubicclip {}", ok as i32);
    if ok {
        for point in &clipped_cubic {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
    }
    out.line(s);

    let curve = [
        p[0].x as f64,
        p[0].y as f64,
        p[1].x as f64,
        p[1].y as f64,
        p[2].x as f64,
        p[2].y as f64,
        p[3].x as f64,
        p[3].y as f64,
    ];
    let eval = SkBezierCubic::eval_at(&curve, 0.37);
    out.doubles("beval", &eval);
    let mut two = [0.0; 14];
    SkBezierCubic::subdivide(&curve, 0.37, &mut two);
    out.doubles("bsub", &two);
    let poly_x = SkBezierCubic::convert_to_polynomial(&curve, false);
    let poly_y = SkBezierCubic::convert_to_polynomial(&curve, true);
    out.doubles(
        "bpoly",
        &[
            poly_x[0], poly_x[1], poly_x[2], poly_x[3], poly_y[0], poly_y[1], poly_y[2],
            poly_y[3],
        ],
    );
    out.line(format!(
        "angle {:e}",
        sk_measure_angle_between_vectors(p[1] - p[0], p[2] - p[1])
    ));
    let bisector = sk_find_bisector(p[1] - p[0], p[2] - p[1]);
    out.line(format!("bisector {:e} {:e}", bisector.x, bisector.y));
    out.line(format!(
        "rot {:e}",
        sk_measure_non_inflect_cubic_rotation(p)
    ));
    out.line(format!(
        "midtangent {:e}",
        sk_find_cubic_mid_tangent(p)
    ));
    let mut mid_chopped = [Point::default(); 7];
    sk_chop_cubic_at_mid_tangent(p, &mut mid_chopped);
    out.points("midchop", &mid_chopped);

    let cubic_map = SkCubicMap::new(p[0], p[1]);
    out.line(format!(
        "cmap {:e} {:e} {:e}",
        cubic_map.compute_y_from_x(0.1),
        cubic_map.compute_y_from_x(0.5),
        cubic_map.compute_y_from_x(0.9)
    ));
    let from_t = cubic_map.compute_from_t(0.37);
    out.line(format!("cmapfromt {:e} {:e}", from_t.x, from_t.y));

    let hits = SkBezierCubic::intersect_with_horizontal_line(p, 0.0);
    out.floats("bhit", &hits);
}

fn dump_quad(out: &mut Out, p: &[Point], index: usize) {
    out.line(format!("Q {index}"));
    out.line(format!("maxcurv {:e}", sk_find_quad_max_curvature(p)));
    out.line(format!("midtan {:e}", sk_find_quad_mid_tangent(p)));
    let mut dst = [Point::default(); 5];
    let chopped = sk_chop_quad_at_y_extrema(p, &mut dst);
    out.points("ychop", &dst[..(chopped as usize) * 2 + 1]);
    let chopped = sk_chop_quad_at_x_extrema(p, &mut dst);
    out.points("xchop", &dst[..(chopped as usize) * 2 + 1]);
    let chopped = sk_chop_quad_at_max_curvature(p, &mut dst);
    out.points("mchop", &dst[..(chopped as usize) * 2 + 1]);
    let mut mid_chopped = [Point::default(); 5];
    sk_chop_quad_at_mid_tangent(p, &mut mid_chopped);
    out.points("qmidchop", &mid_chopped);

    let clip = Rect::new(0.0, 0.0, 10.0, 10.0);
    let mut quad_clipper = SkQuadClipper::new();
    quad_clipper.set_clip(&IRect::new(0, 0, 10, 10));
    let mut clipped = [Point::default(); 3];
    let ok = quad_clipper.clip_quad(&[p[0], p[1], p[2]], &mut clipped);
    let mut s = format!("qclip {}", ok as i32);
    if ok {
        for point in &clipped {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
    }
    out.line(s);

    let mut clipper = SkEdgeClipper::new(false);
    let edge_clipped = clipper.clip_quad(&[p[0], p[1], p[2]], &clip);
    let mut s = format!("qedge {}", edge_clipped as i32);
    if edge_clipped {
        let mut verb_pts = [Point::default(); 4];
        while let Some(verb) = clipper.next(&mut verb_pts) {
            s.push_str(&format!(" {}", verb as i32));
            for point in verb_pts.iter().take(verb_len(verb)) {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
        }
    }
    out.line(s);

    let hits = SkBezierQuad::intersect_with_horizontal_line(p, 0.0);
    out.floats("qhit", &hits);
}

fn dump_conic(out: &mut Out, p: &[Point], w: f32, index: usize) {
    out.line(format!("K {index}"));
    let conic = SkConic::new(p[0], p[1], p[2], w);
    out.line(format!("w {:e}", conic.w));
    out.line(format!("pow2_1 {}", conic.compute_quad_pow2(1.0)));
    out.line(format!(
        "pow2_tenth {}",
        conic.compute_quad_pow2(0.1)
    ));
    let mut pos = Point::default();
    let mut tangent = Point::default();
    conic.eval_at_with(0.37, Some(&mut pos), Some(&mut tangent));
    out.line(format!(
        "eval {:e} {:e} {:e} {:e}",
        pos.x, pos.y, tangent.x, tangent.y
    ));

    let mut halves = [SkConic::default(); 2];
    let chopped = conic.chop_at(0.37, &mut halves);
    let mut s = format!("chop {}", chopped as i32);
    if chopped {
        for half in &halves {
            for point in &half.pts {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
            s.push_str(&format!(" {:e}", half.w));
        }
    }
    out.line(s);

    let pow2 = conic.compute_quad_pow2(1.0);
    if pow2 <= 10 {
        let mut quads = vec![Point::default(); 1 + 2 * (1usize << pow2)];
        let quad_count = conic.chop_into_quads_pow2(&mut quads, pow2);
        let mut s = format!("qpow {quad_count}");
        for point in quads.iter().take(1 + 2 * quad_count as usize) {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
        out.line(s);
    } else {
        out.line("qpow skipped".to_string());
    }

    let mut chopped_twice = SkConic::default();
    conic.chop_at_two(0.3, 0.6, &mut chopped_twice);
    let mut s = "chop2".to_string();
    for point in &chopped_twice.pts {
        s.push_str(&format!(" {:e} {:e}", point.x, point.y));
    }
    s.push_str(&format!(" {:e}", chopped_twice.w));
    out.line(s);

    let mut x_halves = [SkConic::default(); 2];
    let x_chop = conic.chop_at_x_extrema(&mut x_halves);
    let mut s = format!("xchop {}", x_chop as i32);
    if x_chop {
        for half in &x_halves {
            for point in &half.pts {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
        }
    }
    out.line(s);
    let mut y_halves = [SkConic::default(); 2];
    let y_chop = conic.chop_at_y_extrema(&mut y_halves);
    let mut s = format!("ychopc {}", y_chop as i32);
    if y_chop {
        for half in &y_halves {
            for point in &half.pts {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
        }
    }
    out.line(s);

    let mut quad_error = Point::default();
    conic.compute_as_quad_error(&mut quad_error);
    out.line(format!("quaderr {:e} {:e}", quad_error.x, quad_error.y));
    out.line(format!("quadtol1 {}", conic.as_quad_tol(1.0) as i32));
    out.line(format!("quadtol01 {}", conic.as_quad_tol(0.1) as i32));
    out.line(format!(
        "transformw {:e}",
        SkConic::transform_w(p, conic.w, &Matrix::Scale(2.0, 3.0))
    ));

    out.line(format!("midtan {:e}", conic.find_mid_tangent()));
    let x_ext = conic.find_x_extrema();
    out.line(format!(
        "xext {} {:e}",
        x_ext.is_some() as i32,
        x_ext.unwrap_or(-1.0)
    ));
    let y_ext = conic.find_y_extrema();
    out.line(format!(
        "yext {} {:e}",
        y_ext.is_some() as i32,
        y_ext.unwrap_or(-1.0)
    ));
    let tight = conic.compute_tight_bounds();
    out.line(format!(
        "tight {:e} {:e} {:e} {:e}",
        tight.left, tight.top, tight.right, tight.bottom
    ));
}

fn dump_poly(out: &mut Out, pts: &[Point], index: usize) {
    out.line(format!("P {index}"));
    out.line(format!("winding {}", sk_get_polygon_winding(pts)));
    out.line(format!("convex {}", sk_is_convex_polygon(pts) as i32));
    out.line(format!("simple {}", sk_is_simple_polygon(pts) as i32));
    let radial = sk_compute_radial_steps(
        Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
        5.0,
    );
    match radial {
        Some((rot_sin, rot_cos, steps)) => {
            out.line(format!("radial 1 {rot_sin:e} {rot_cos:e} {steps}"))
        }
        None => out.line("radial 0 0e0 0e0 0".to_string()),
    }

    let mut inset = Vec::new();
    let inset_ok = sk_inset_convex_polygon(pts, 5.0, &mut inset);
    let mut s = format!("inset {}", inset_ok as i32);
    if inset_ok {
        for point in &inset {
            s.push_str(&format!(" {:e} {:e}", point.x, point.y));
        }
    }
    out.line(s);

    let bounds = Rect::bounds_or_empty(pts);
    for offset in [5.0f32, -5.0] {
        let mut offset_poly = Vec::new();
        let mut indices = Vec::new();
        let offset_ok =
            sk_offset_simple_polygon(pts, &bounds, offset, &mut offset_poly, Some(&mut indices));
        let mut s = format!("offset {}", offset_ok as i32);
        if offset_ok {
            for point in &offset_poly {
                s.push_str(&format!(" {:e} {:e}", point.x, point.y));
            }
            s.push_str(" idx");
            for index in &indices {
                s.push_str(&format!(" {index}"));
            }
        }
        out.line(s);
    }

    let index_map: Vec<u16> = (0..pts.len() as u16).collect();
    let mut triangles = Vec::new();
    let tri_ok = sk_triangulate_simple_polygon(pts, &index_map, &mut triangles);
    let mut s = format!("tri {}", tri_ok as i32);
    if tri_ok {
        s.push_str(&format!(" {}", triangles.len()));
        for index in &triangles {
            s.push_str(&format!(" {index}"));
        }
    }
    out.line(s);
}

#[test]
fn dump_all() {
    let text = fs::read_to_string("target/m06-evidence/fixtures.txt").expect("fixtures");
    let mut tokens = text.split_whitespace();
    let mut out = Out { buf: String::new() };
    let mut index = 0usize;
    while let Some(kind) = tokens.next() {
        match kind {
            "cubic" => {
                let mut p = [Point::default(); 4];
                for point in &mut p {
                    point.x = tokens.next().unwrap().parse().unwrap();
                    point.y = tokens.next().unwrap().parse().unwrap();
                }
                dump_cubic(&mut out, &p, index);
            }
            "quad" => {
                let mut p = [Point::default(); 3];
                for point in &mut p {
                    point.x = tokens.next().unwrap().parse().unwrap();
                    point.y = tokens.next().unwrap().parse().unwrap();
                }
                dump_quad(&mut out, &p, index);
            }
            "conic" => {
                let mut p = [Point::default(); 3];
                for point in &mut p {
                    point.x = tokens.next().unwrap().parse().unwrap();
                    point.y = tokens.next().unwrap().parse().unwrap();
                }
                let w: f32 = tokens.next().unwrap().parse().unwrap();
                dump_conic(&mut out, &p, w, index);
            }
            "poly" => {
                let n: usize = tokens.next().unwrap().parse().unwrap();
                let mut pts = vec![Point::default(); n];
                for point in &mut pts {
                    point.x = tokens.next().unwrap().parse().unwrap();
                    point.y = tokens.next().unwrap().parse().unwrap();
                }
                dump_poly(&mut out, &pts, index);
            }
            other => panic!("unknown record {other}"),
        }
        index += 1;
    }
    fs::write("target/m06-evidence/rust.txt", &out.buf).expect("write rust dump");
}
