// Supplemental M06 contract checks, separate from original registrations.
use super::*;
use rust_skia::sk_line_clipper::{clip_line, intersect_line};
#[test]
fn supplemental_roots_boundaries_and_degenerate_curves() {
    let mut r = Reporter::default();
    let mut roots = [-7.; 2];
    let n = sk_find_unit_quad_roots(1., -1., 0., &mut roots);
    check!(r, n == 0); // open unit interval
    let n = sk_find_unit_quad_roots(0., 2., -1., &mut roots);
    check!(r, n == 1);
    check!(r, roots[0] == 0.5);
    let n = sk_find_unit_quad_roots(1., 0., 1., &mut roots);
    check!(r, n == 0);
    let src = [p(3., -4.); 4];
    for t in [0., f32::MIN_POSITIVE, 0.5, 1. - f32::EPSILON, 1.] {
        check!(r, sk_eval_quad_at(&src, t) == src[0]);
        let mut point = Point::default();
        sk_eval_cubic_at(&src, t, Some(&mut point), None, None);
        check!(r, point_near(point, src[0]));
        let mut dst = [p(99., 99.); 7];
        sk_chop_cubic_at(&src, &mut dst, t);
        check!(r, dst.iter().all(|v| point_near(*v, src[0])));
    }
    let mut q = [Point::default(); 5];
    sk_chop_quad_at_half(&src, &mut q);
    check!(r, q.iter().all(|v| *v == src[0]));
    let mut c = [Point::default(); 7];
    sk_chop_cubic_at_half(&src, &mut c);
    check!(r, c.iter().all(|v| *v == src[0]));
    r.finish();
}
#[test]
fn supplemental_clipping_counts_bounds_and_output_ownership() {
    let mut r = Reporter::default();
    let clip = Rect::new(0., 0., 10., 10.);
    let line = [p(-5., 5.), p(15., 5.)];
    let mut dst = [p(99., 99.); 2];
    check!(r, intersect_line(&line, &clip, &mut dst));
    check!(r, dst == [p(0., 5.), p(10., 5.)]);
    let reversed = [line[1], line[0]];
    check!(r, intersect_line(&reversed, &clip, &mut dst));
    check!(r, dst == [p(10., 5.), p(0., 5.)]);
    check!(
        r,
        !intersect_line(&[p(-5., -2.), p(15., -2.)], &clip, &mut dst)
    );
    let mut output = [Point::default(); 4];
    let n = clip_line(&[p(2., 2.), p(8., 8.)], &clip, &mut output, false);
    check!(r, n == 1);
    check!(r, output[..2] == [p(2., 2.), p(8., 8.)]);
    let mut edge = SkEdgeClipper::new(false);
    check!(r, !edge.can_cull_to_the_right());
    check!(r, edge.clip_line(p(2., 2.), p(8., 8.), &clip));
    let mut points = [Point::default(); 4];
    check!(r, edge.next(&mut points) == Some(SkPathVerb::Line));
    let retained = points;
    check!(r, retained[..2] == [p(2., 2.), p(8., 8.)]);
    check!(r, edge.next(&mut points).is_none());
    check!(r, edge.next(&mut points).is_none());
    check!(r, edge.clip_line(p(3., 3.), p(7., 7.), &clip));
    check!(r, edge.next(&mut points) == Some(SkPathVerb::Line));
    check!(r, retained[..2] == [p(2., 2.), p(8., 8.)]);
    let mut qc = SkQuadClipper::new();
    qc.set_clip(&IRect::new(0, 0, 10, 10));
    let quad = [p(2., 2.), p(5., 5.), p(8., 8.)];
    let mut qout = [Point::default(); 3];
    check!(r, qc.clip_quad(&quad, &mut qout));
    check!(r, qout == quad);
    let mut cc = SkCubicClipper::new();
    cc.set_clip(&IRect::new(0, 0, 10, 10));
    let cubic = [p(2., 2.), p(4., 4.), p(6., 6.), p(8., 8.)];
    let mut cout = [Point::default(); 4];
    check!(r, cc.clip_cubic(&cubic, &mut cout));
    check!(r, cout == cubic);
    check!(r, SkCubicClipper::chop_mono_at_y(&cubic, -1.).is_none());
    let t = SkCubicClipper::chop_mono_at_y(&cubic, 5.);
    check!(r, t.is_some());
    if let Some(t) = t {
        check!(r, near(t, 0.5));
    }
    // Double intermediates must keep intersections finite for extreme finite endpoints.
    check!(
        r,
        intersect_line(&[p(-f32::MAX, 5.), p(f32::MAX, 5.)], &clip, &mut dst)
    );
    check!(r, dst == [p(0., 5.), p(10., 5.)]);
    r.finish();
}
#[test]
fn supplemental_polygon_size_overflow_and_index_bytes() {
    let mut r = Reporter::default();
    let mut output = Vec::new();
    let mut indices = Vec::new();
    for pts in [&[][..], &[p(0., 0.)][..], &[p(0., 0.), p(1., 1.)][..]] {
        check!(r, !sk_is_convex_polygon(pts));
        check!(r, !sk_inset_convex_polygon(pts, 1., &mut output));
        check!(r, !sk_triangulate_simple_polygon(pts, &[], &mut indices));
    }
    let huge = vec![p(0., 0.); 65536];
    check!(r, !sk_inset_convex_polygon(&huge, 1., &mut output));
    for len in [65535, 65536] {
        check!(
            r,
            !sk_offset_simple_polygon(
                &huge[..len],
                &Rect::new(0., 0., 1., 1.),
                1.,
                &mut output,
                None
            )
        );
        check!(
            r,
            !sk_triangulate_simple_polygon(&huge[..len], &[], &mut indices)
        );
    }
    check!(
        r,
        sk_compute_radial_steps(p(1., 0.), p(0., 1.), f32::MAX).is_none()
    );
    check!(
        r,
        sk_compute_radial_steps(p(f32::INFINITY, 0.), p(0., 1.), 1.).is_none()
    );
    let tri = [p(0., 0.), p(10., 0.), p(0., 10.)];
    let map = [0x0102u16, 0x0304, 0x0506];
    check!(r, sk_triangulate_simple_polygon(&tri, &map, &mut indices));
    check!(r, indices.len() == 3);
    indices.sort_unstable();
    check!(r, indices == map);
    // Observe the returned u16 storage, including native byte order on Apple Silicon.
    #[cfg(target_endian = "little")]
    {
        let bytes =
            unsafe { std::slice::from_raw_parts(indices.as_ptr().cast::<u8>(), indices.len() * 2) };
        check!(r, bytes == [2, 1, 4, 3, 6, 5]);
    }
    for inset in [-1., f32::INFINITY, f32::NAN] {
        check!(r, !sk_inset_convex_polygon(&tri, inset, &mut output));
    }
    r.finish();
}
#[test]
fn supplemental_conic_storage_ownership_and_lifetime() {
    let mut r = Reporter::default();
    let saved;
    {
        let mut input = [p(0., 1.), p(0., 0.), p(1., 0.)];
        let conic = SkConic::from_pts(&input, 0.70710678);
        input.fill(p(99., 99.));
        check!(r, conic.pts == [p(0., 1.), p(0., 0.), p(1., 0.)]);
        let mut quadder = SkAutoConicToQuads::new();
        {
            let borrowed = quadder.compute_quads(&conic, 0.001);
            check!(r, borrowed[0] == conic.pts[0]);
            check!(r, *borrowed.last().unwrap() == conic.pts[2]);
            saved = borrowed.to_vec();
        }
        // The borrow has ended before reuse; the owned snapshot survives reallocation and drop.
        let other = SkConic::new(p(10., 10.), p(20., 20.), p(30., 30.), 1.);
        let next = quadder.compute_quads(&other, 1.);
        check!(r, next == [p(10., 10.), p(20., 20.), p(30., 30.)]);
        check!(r, quadder.count_quads() == 1);
    }
    check!(r, saved.first() == Some(&p(0., 1.)));
    check!(r, saved.last() == Some(&p(1., 0.)));
    let owned = {
        let input = [p(0., 0.), p(0.5, 1.), p(1., 0.)];
        SkBezierQuad::intersect_with_horizontal_line(&input, 0.5)
    };
    check!(r, owned == [0.5]);
    let mut c = SkConic::default();
    for w in [0., -1., f32::NAN, f32::INFINITY] {
        c.set_w(w);
        check!(r, c.w == 1.);
    }
    c.set_w(0.5);
    check!(r, c.w == 0.5);
    r.finish();
}
#[test]
fn supplemental_unit_arc_and_perspective_weight() {
    let mut r = Reporter::default();
    let mut arcs = [SkConic::default(); SkConic::K_MAX_CONICS_FOR_ARC];
    let n = SkConic::build_unit_arc(p(1., 0.), p(0., 1.), SkPathDirection::Cw, None, &mut arcs);
    check!(r, n == 1);
    check!(r, point_near(arcs[0].pts[0], p(1., 0.)));
    check!(r, point_near(arcs[0].pts[2], p(0., 1.)));
    check!(r, near(arcs[0].w, std::f32::consts::FRAC_1_SQRT_2));
    let affine = Matrix::make_all(2., 0., 3., 0., 4., 5., 0., 0., 1.);
    let n = SkConic::build_unit_arc(
        p(1., 0.),
        p(0., 1.),
        SkPathDirection::Cw,
        Some(&affine),
        &mut arcs,
    );
    check!(r, n == 1);
    check!(r, point_near(arcs[0].pts[0], p(5., 5.)));
    check!(r, point_near(arcs[0].pts[2], p(3., 9.)));
    let pts = [p(0., 0.), p(1., 1.), p(2., 0.)];
    check!(r, SkConic::transform_w(&pts, 0.5, &affine) == 0.5);
    let perspective = Matrix::make_all(1., 0., 0., 0., 1., 0., 1., 0., 1.);
    check!(
        r,
        near(
            SkConic::transform_w(&pts, 0.5, &perspective),
            1. / 3.0f32.sqrt()
        )
    );
    r.finish();
}
#[test]
fn supplemental_verb_discriminants_and_coregraphics_layout_byte_order() {
    let mut r = Reporter::default();
    let verbs = [
        SkPathVerb::Move,
        SkPathVerb::Line,
        SkPathVerb::Quad,
        SkPathVerb::Conic,
        SkPathVerb::Cubic,
        SkPathVerb::Close,
        SkPathVerb::Done,
    ];
    check!(r, std::mem::size_of::<SkPathVerb>() == 1);
    let bytes = unsafe { std::slice::from_raw_parts(verbs.as_ptr().cast::<u8>(), verbs.len()) };
    check!(r, bytes == [0, 1, 2, 3, 4, 5, 6]);
    #[cfg(target_os = "macos")]
    {
        use rust_skia::sk_cg_geometry::*;
        check!(r, std::mem::size_of::<CGRect>() == 32);
        check!(r, std::mem::align_of::<CGRect>() == 8);
        check!(r, std::mem::offset_of!(CGRect, size) == 16);
        let rect = CGRect {
            origin: CGPoint { x: 1., y: -2. },
            size: CGSize {
                width: 3.,
                height: 4.,
            },
        };
        check!(r, !sk_cg_rect_is_empty(&rect));
        check!(r, sk_cg_rect_get_min_x(&rect) == 1.);
        check!(r, sk_cg_rect_get_max_x(&rect) == 4.);
        check!(r, sk_cg_rect_get_min_y(&rect) == -2.);
        check!(r, sk_cg_rect_get_max_y(&rect) == 2.);
        check!(r, sk_cg_rect_get_width(&rect) == 3.);
        // repr(C), four initialized f64s and no padding: inspect actual object bytes.
        let bytes =
            unsafe { std::slice::from_raw_parts((&rect as *const CGRect).cast::<u8>(), 32) };
        #[cfg(target_endian = "little")]
        check!(
            r,
            bytes
                == [
                    0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0, 0, 0, 0, 8, 64,
                    0, 0, 0, 0, 0, 0, 16, 64
                ]
        );
        for width in [0., -1.] {
            let empty = CGRect {
                size: CGSize { width, height: 4. },
                ..rect
            };
            check!(r, sk_cg_rect_is_empty(&empty));
        }
        for height in [0., -1.] {
            let empty = CGRect {
                size: CGSize { width: 3., height },
                ..rect
            };
            check!(r, sk_cg_rect_is_empty(&empty));
        }
    }
    r.finish();
}
