// Supplemental M05 contracts; independent of the faithful original registrations.
use super::*;
use rust_skia::{
    bitmap_proc_state_matrix_procs as tile,
    camera::{Camera3D, Patch3D, View3D},
    matrix_invert,
};

#[test]
fn supplemental_serialization_boundaries_byte_order_and_ownership() {
    // Native-endian row-major scalar payload, including signed zero and NaN payload bits.
    let values = [
        1.25,
        -0.0,
        f32::from_bits(0x7fc12345),
        -7.5,
        2.0,
        6.0,
        0.125,
        -0.25,
        1.0,
    ];
    let mut m = Matrix::I();
    m.set9(&values);
    let mut buffer = [0xa5; 140];
    let n = matrix_priv::write_to_memory(&m, &mut buffer[4..136]);
    assert_eq!(n, 36);
    assert_eq!(&buffer[..4], &[0xa5; 4]);
    assert!(buffer[40..].iter().all(|b| *b == 0xa5));
    for i in 0..9 {
        assert_eq!(&buffer[4 + i * 4..8 + i * 4], &values[i].to_ne_bytes());
    }
    for len in 0..36 {
        let mut dst = Matrix::Translate(17., 19.);
        let before = dst.get9_array();
        assert_eq!(
            matrix_priv::read_from_memory(&mut dst, &buffer[4..4 + len]),
            0
        );
        assert_eq!(dst.get9_array(), before);
        let mut short = [0x5a; 36];
        assert_eq!(matrix_priv::write_to_memory(&m, &mut short[..len]), 0);
        assert_eq!(short, [0x5a; 36]);
    }
    let mut decoded = Matrix::I();
    assert_eq!(
        matrix_priv::read_from_memory(&mut decoded, &buffer[4..40]),
        36
    );
    for i in 0..9 {
        assert_eq!(decoded[i].to_bits(), values[i].to_bits());
    }
    buffer.fill(0);
    m.reset();
    for i in 0..9 {
        assert_eq!(decoded[i].to_bits(), values[i].to_bits());
    }
    let persisted = {
        let temporary = Matrix::ScaleTranslate(2., 3., 4., 5.);
        let mut bytes = [0; 36];
        temporary.write_to_memory(&mut bytes);
        bytes
    };
    let mut restored = Matrix::I();
    assert_eq!(restored.read_from_memory(&persisted), 36);
    assert_eq!(restored, Matrix::ScaleTranslate(2., 3., 4., 5.));
}

#[test]
fn supplemental_inversion_singular_nonfinite_and_near_singular() {
    for sx in [0., -0., f32::from_bits(1), f32::NAN] {
        let m = Matrix::Scale(sx, 1.);
        assert!(!m.invert(None));
        let mut out = Matrix::I();
        assert!(!m.invert(Some(&mut out)));
    }
    // Pinned SkMatrix accepts infinite diagonal scales: their reciprocals are signed zero.
    for sx in [f32::INFINITY, f32::NEG_INFINITY] {
        let m = Matrix::Scale(sx, 1.);
        assert!(m.invert(None));
        let mut out = Matrix::I();
        assert!(m.invert(Some(&mut out)));
        assert!(out.is_finite());
        let expected = [1. / sx, 0., -0. / sx, 0., 1., -0., 0., 0., 1.];
        for i in 0..9 {
            assert_eq!(out[i].to_bits(), expected[i].to_bits());
        }
    }
    let m = Matrix::Scale(1e-6, 2.);
    let inv = m.inverted().expect("finite near-singular diagonal");
    let p = Point::new(3., -4.);
    let q = inv.map_point(m.map_point(p));
    assert!(near(p.x, q.x) && near(p.y, q.y));
    for m in [
        M44::Scale(0., 1., 1.),
        M44::Scale(f32::INFINITY, 1., 1.),
        M44::nan(),
    ] {
        assert!(!m.invert(None));
        let mut out = M44::I();
        assert!(!m.invert(Some(&mut out)));
    }
    let mut o2 = [0.; 4];
    assert_eq!(
        matrix_invert::invert_2x2_matrix(&[2., 0., 0., 4.], &mut o2),
        8.
    );
    assert_eq!(o2, [0.5, 0., 0., 0.25]);
    assert_eq!(
        matrix_invert::invert_2x2_matrix(&[1., 2., 2., 4.], &mut o2),
        0.
    );
    let mut o3 = [0.; 9];
    assert_eq!(
        matrix_invert::invert_3x3_matrix(&[2., 0., 0., 0., 4., 0., 0., 0., 8.], &mut o3),
        64.
    );
    assert_eq!(o3, [0.5, 0., 0., 0., 0.25, 0., 0., 0., 0.125]);
    let mut o4 = [0.; 16];
    let diag = [
        2., 0., 0., 0., 0., 4., 0., 0., 0., 0., 8., 0., 0., 0., 0., 16.,
    ];
    assert_eq!(matrix_invert::invert_4x4_matrix(&diag, &mut o4), 1024.);
    assert_eq!(
        o4,
        [
            0.5, 0., 0., 0., 0., 0.25, 0., 0., 0., 0., 0.125, 0., 0., 0., 0., 0.0625
        ]
    );
}

#[test]
fn supplemental_matrix_order_cache_layout_and_lifetime() {
    let mut m = Matrix::Translate(3., 5.);
    m.pre_scale(2., 4.);
    assert_eq!(m.map_point(Point::new(1., 1.)), Point::new(5., 9.));
    let mut post = Matrix::Translate(3., 5.);
    post.post_scale(2., 4.);
    assert_eq!(post.map_point(Point::new(1., 1.)), Point::new(8., 24.));
    assert!(m.rect_stays_rect());
    m[Matrix::M_PERSP_0] = 0.25;
    assert!(m.has_perspective());
    m[Matrix::M_PERSP_0] = 0.;
    assert!(!m.has_perspective());
    let mut scaled = Matrix::MakeAll(4., 0., 8., 0., 6., 10., 0., 0., 2.);
    scaled.normalize_perspective();
    assert_eq!(scaled, Matrix::ScaleTranslate(2., 3., 4., 5.));
    let mut m44 = M44::RowMajor(&[
        1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15., 16.,
    ]);
    assert_eq!(std::mem::size_of::<M44>(), 64);
    assert_eq!(std::mem::size_of::<Matrix>(), 40);
    assert_eq!(
        matrix_priv::m44_col_major(&m44),
        &[
            1., 5., 9., 13., 2., 6., 10., 14., 3., 7., 11., 15., 4., 8., 12., 16.
        ]
    );
    let owned = m44.get_col_major();
    m44.set_identity();
    assert_eq!(owned[1], 5.);
    let mapped = {
        let local = Matrix::Translate(7., 9.);
        local.map_point(Point::new(1., 2.))
    };
    assert_eq!(mapped, Point::new(8., 11.));
    let src = Rect::new(-1., -2., 3., 4.);
    let mut dst = Rect::new(99., 98., 100., 101.);
    let saved = dst;
    assert!(!matrix_priv::inverse_map_rect(
        &Matrix::Scale(0., 1.),
        &mut dst,
        &src
    ));
    assert_eq!(dst, saved);
    assert!(matrix_priv::inverse_map_rect(
        &Matrix::Scale(-2., 4.),
        &mut dst,
        &src
    ));
    assert_eq!(dst, Rect::new(-1.5, -0.5, 0.5, 1.));
    let sentinel = Point::new(99., 101.);
    let mut strided = [Point::new(1., 2.), sentinel, Point::new(3., 4.), sentinel];
    matrix_priv::map_points_in_place_with_stride(&Matrix::Translate(7., 9.), &mut strided, 2, 2);
    assert_eq!(
        strided,
        [
            Point::new(8., 11.),
            sentinel,
            Point::new(10., 13.),
            sentinel
        ]
    );
    Matrix::I().map_points(&mut [], &[]);
    Matrix::I().map_vectors(&mut [], &[]);
}

#[test]
fn supplemental_fixed_tiling_overflow_and_packed_byte_order() {
    for n in [1, 2, 3, 255, 32767] {
        for x in [i32::MIN, -65537, -1, 0, 1, 65535, i32::MAX] {
            let expected = (x as i64).rem_euclid(n as i64) as u16;
            assert_eq!(tile::int_repeat(x, n), expected);
            let folded = (x as i64).rem_euclid(2 * n as i64);
            let expected = if folded < n as i64 {
                folded
            } else {
                2 * n as i64 - 1 - folded
            } as u16;
            assert_eq!(tile::int_mirror(x, n), expected);
            assert_eq!(
                tile::int_clamp(x, n),
                (x as i64).clamp(0, n as i64 - 1) as u16
            );
        }
    }
    for max in [0, 1, 255, 16383] {
        for f in [
            i32::MIN,
            -65537,
            -65536,
            -1,
            0,
            1,
            4095,
            4096,
            65535,
            65536,
            i32::MAX,
        ] {
            assert_eq!(tile::clamp(f, max), (f >> 16).clamp(0, max) as u32);
            assert_eq!(
                tile::repeat(f, max),
                (((f as u32 & 65535) as u64 * (max + 1) as u64) >> 16) as u32
            );
            let parity = (f >> 16) & 1;
            let fraction = (f as u32 & 65535) ^ if parity != 0 { 65535 } else { 0 };
            assert_eq!(
                tile::mirror(f, max),
                ((fraction as u64 * (max + 1) as u64) >> 16) as u32
            );
        }
    }
    let packed = tile::pack_clamp((3 << 16) + (5 << 12), 100);
    assert_eq!(packed, (((3u32 << 4) | 5) << 14) | 4);
    // Validate the packed word's native bytes explicitly for each target byte order.
    #[cfg(target_endian = "little")]
    assert_eq!(packed.to_ne_bytes(), [0x04, 0x40, 0x0d, 0x00]);
    #[cfg(target_endian = "big")]
    assert_eq!(packed.to_ne_bytes(), [0x00, 0x0d, 0x40, 0x04]);
    for count in 0..=7 {
        let mut dst = [0xdead; 9];
        tile::decal_nofilter_scale(&mut dst[1..8], 1 << 16, 1 << 15, count);
        assert_eq!(dst[0], 0xdead);
        for i in 0..count {
            assert_eq!(dst[i + 1], ((65536 + i * 32768) >> 16) as u16);
        }
        assert!(dst[count + 1..].iter().all(|x| *x == 0xdead));
    }
    assert!(!tile::can_truncate_to_fixed_for_decal(
        0,
        tile::DECAL_DX_MIN,
        1,
        10
    ));
    assert!(tile::can_truncate_to_fixed_for_decal(
        0,
        tile::DECAL_DX_MIN + 1,
        1,
        10
    ));
    assert!(!tile::can_truncate_to_fixed_for_decal(-1, 65536, 1, 10));
    assert!(!tile::can_truncate_to_fixed_for_decal(0, 65536, 11, 10));
    assert!(!tile::can_truncate_to_fixed_for_decal(
        i32::MAX - 65536,
        65536,
        3,
        65535
    ));
    for (x, expected) in [
        (i64::MIN, i32::MIN),
        (-32768i64 << 32, i32::MIN),
        (-1i64 << 32, -65536),
        (0, 0),
        (1i64 << 32, 65536),
        (32767i64 << 32, 32767 << 16),
        (i64::MAX, 32767 << 16),
    ] {
        assert_eq!(tile::sk_fixed3232_saturate2fixed(x), expected);
    }
    assert_eq!(tile::fixed3232_floor_to_int(-1), -1);
}

#[test]
fn supplemental_rsxform_camera_stack_and_owned_outputs() {
    let x = RSXform::Make(2., 3., 5., 7.);
    let quad = x.to_quad(4., 6.);
    assert_eq!(
        quad,
        [
            Point::new(5., 7.),
            Point::new(13., 19.),
            Point::new(-5., 31.),
            Point::new(-13., 19.)
        ]
    );
    let mut strip = [Point::default(); 4];
    x.to_tri_strip(4., 6., &mut strip);
    assert_eq!(strip, [quad[0], quad[3], quad[1], quad[2]]);
    let anchored = RSXform::MakeFromRadians(2., 0., 10., 20., 3., 4.);
    let mut m = Matrix::I();
    m.set_rs_xform(&anchored);
    assert_eq!(m.map_point(Point::new(3., 4.)), Point::new(10., 20.));
    assert!(RSXform::Make(0., 3., 0., 0.).rect_stays_rect());
    assert!(!x.rect_stays_rect());
    let owned = {
        let local = RSXform::Make(1., 0., 2., 3.);
        local.to_quad(0., 0.)
    };
    assert_eq!(owned, [Point::new(2., 3.); 4]);
    let patch = Patch3D::new();
    assert_eq!(patch.dot_with(0., 0., 1.), -1.);
    let moved = patch.transformed(&M44::Translate(3., 4., 5.));
    assert_eq!(moved.f_u, patch.f_u);
    assert_eq!(moved.f_v, patch.f_v);
    assert_eq!(moved.f_origin, V3::new(3., 4., 5.));
    let mut camera = Camera3D::new();
    let mut projected = Matrix::I();
    camera.patch_to_matrix(&patch, &mut projected);
    assert_eq!(projected, Matrix::I());
    let mut view = View3D::new();
    let mut original = Matrix::I();
    view.get_matrix(&mut original);
    assert_eq!(original, Matrix::I());
    view.translate(3., 4., 0.);
    let mut translated = Matrix::I();
    view.get_matrix(&mut translated);
    assert_eq!(translated, Matrix::Translate(3., -4.));
    view.save();
    view.rotate_x(30.);
    view.save();
    view.rotate_y(45.);
    view.rotate_z(15.);
    let mut changed = Matrix::I();
    view.get_matrix(&mut changed);
    assert_ne!(changed, translated);
    view.restore();
    view.restore();
    let mut restored = Matrix::I();
    view.get_matrix(&mut restored);
    assert_eq!(restored, translated);
    for _ in 0..64 {
        view.save();
        view.translate(1., 2., 3.);
        view.restore();
    }
    view.get_matrix(&mut restored);
    assert_eq!(restored, translated);
    let retained = {
        let mut local = View3D::new();
        let mut dst = Matrix::I();
        local.get_matrix(&mut dst);
        dst
    };
    assert_eq!(retained, original);
    for h in [
        Point::new(0., 0.),
        Point::new(-3., 0.),
        Point::new(0., -4.),
        Point::new(3., 4.),
        Point::new(-3., 4.),
    ] {
        let g = rust_skia::matrix22::givens_rotation(h);
        let p = g.map_point(h);
        assert!(near(p.y, 0.));
        assert!(near(p.x, h.length()));
    }
}
