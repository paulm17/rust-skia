// MatrixTest.cpp:363-660, reached by Matrix (:868). BSD: tests/LICENSE.skia.
use super::*;
fn test_matrix_preserve_shape(r: &mut Reporter) {
    let mut mat = Matrix::I();

    mat.set_identity();
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_translate(100.0, 100.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_scale(15.0, 15.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_scale(-15.0, 15.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_scale(15.0, 20.0);
    check!(r, !mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_scale_pivot(15.0, 15.0, 2.0, 2.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_scale_pivot(15.0, 20.0, 2.0, 2.0);
    check!(r, !mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_skew(15.0, 15.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.set_skew(15.0, 20.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.set_skew_pivot(15.0, 15.0, 2.0, 2.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.set_skew_pivot(15.0, 20.0, 2.0, 2.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.reset().set_persp_x(1.0 / 2.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.reset().set_persp_y(1.0 / 2.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    for angle in 0..360 {
        mat.set_rotate(angle as f32);
        check!(r, mat.is_similarity());
        check!(r, mat.preserves_right_angles());
    }

    mat.reset();
    for _ in 1..360 {
        mat.post_rotate(1.0);
    }
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_rotate(30.0).post_translate(10.0, 20.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_rotate(30.0).post_scale(2.0, 2.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_rotate(30.0).post_scale(3.0, 2.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.set_scale(3.0, 2.0).post_rotate(30.0);
    check!(r, !mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_all(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.set_all(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    check!(r, !mat.is_similarity());
    check!(r, !mat.preserves_right_angles());

    mat.set_all(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());

    mat.set_all(0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    check!(r, mat.is_similarity());
    check!(r, mat.preserves_right_angles());
}
fn test_matrix_decomposition(r: &mut Reporter) {
    let mut mat = Matrix::I();
    let (mut rotation1, mut scale, mut rotation2) =
        (Point::default(), Point::default(), Point::default());

    let kRotation0: f32 = 15.5;
    let kRotation1: f32 = -50.;
    let kScale0: f32 = 5000.;
    let kScale1: f32 = 0.001;

    mat.reset();
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    check!(r, matrix_utils::decompose_upper2x2(&mat, None, None, None));

    mat.set_rotate(kRotation0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_scale(kScale0, kScale0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_scale(kScale1, kScale0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(kRotation1).post_scale(kScale0, kScale0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_scale(kScale0, kScale0).post_rotate(kRotation1);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(kRotation0).post_scale(kScale1, -kScale1);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_scale(kScale0, -kScale0).post_rotate(kRotation1);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(kRotation1).post_scale(kScale1, kScale0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(90.0).post_scale(kScale1, kScale0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_scale(kScale1, kScale0).post_rotate(kRotation0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_scale(kScale1, kScale0).post_rotate(90.0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(kRotation1)
        .post_scale(kScale0, kScale0)
        .post_rotate(kRotation0);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(kRotation0)
        .post_scale(kScale1, kScale0)
        .post_rotate(kRotation1);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.set_rotate(kRotation0)
        .post_scale(-kScale1, kScale0)
        .post_rotate(kRotation1);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    let mut rand = SkRandom::new();
    for _ in 0..1000 {
        let rot0 = rand.next_range_f(-180.0, 180.0);
        let sx = rand.next_range_f(-3000., 3000.);
        let sy = rand.next_range_f(-3000., 3000.);
        let rot1 = rand.next_range_f(-180.0, 180.0);
        mat.set_rotate(rot0).post_scale(sx, sy).post_rotate(rot1);

        if matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2),
        ) {
            check!(r, recompose(&mat, rotation1, scale, rotation2));
        } else {
            let perpdot = mat[Matrix::M_SCALE_X] * mat[Matrix::M_SCALE_Y]
                - mat[Matrix::M_SKEW_X] * mat[Matrix::M_SKEW_Y];
            check!(r, perpdot.abs() <= NZ);
        }
    }

    mat.post_translate(-1000., 1000.);
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat[Matrix::M_PERSP_0] = 12.;
    mat[Matrix::M_PERSP_1] = 4.;
    mat[Matrix::M_PERSP_2] = 1872.;
    check!(
        r,
        matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    check!(r, recompose(&mat, rotation1, scale, rotation2));

    mat.reset();
    mat[Matrix::M_SCALE_X] = 0.;
    check!(
        r,
        !matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    mat.reset();
    mat[Matrix::M_SCALE_Y] = 0.;
    check!(
        r,
        !matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
    mat.reset();

    mat[Matrix::M_SCALE_X] = 1.;
    mat[Matrix::M_SKEW_X] = 2.;
    mat[Matrix::M_SKEW_Y] = 4.;
    mat[Matrix::M_SCALE_Y] = 8.;
    check!(
        r,
        !matrix_utils::decompose_upper2x2(
            &mat,
            Some(&mut rotation1),
            Some(&mut scale),
            Some(&mut rotation2)
        )
    );
}
fn nearly_equal(a: &Matrix, b: &Matrix) -> bool {
    (0..9).all(|i| (a[i] - b[i]).abs() <= 1.0 / 200000.0)
}
fn is_identity(m: &Matrix) -> bool {
    let mut id = Matrix::I();
    id.reset();
    nearly_equal(m, &id)
}
fn are_equal(r: &mut Reporter, a: &Matrix, b: &Matrix) -> bool {
    let equal = a == b;
    let cheap = matrix_priv::cheap_equal(a, b);
    if equal != cheap {
        if equal {
            let mut found = false;
            for i in 0..9 {
                let av = a.get(i);
                let bv = b.get(i);
                if av == 0. && bv == 0. && av.to_bits() != bv.to_bits() {
                    found = true;
                } else {
                    check!(r, av == bv && av.to_bits() == bv.to_bits());
                }
            }
            check!(r, found);
        } else {
            let mut found = false;
            for i in 0..9 {
                let av = a.get(i);
                let bv = b.get(i);
                if av.is_nan() && av.to_bits() == bv.to_bits() {
                    found = true;
                } else {
                    check!(r, av == bv && av.to_bits() == bv.to_bits());
                }
            }
            check!(r, found);
        }
    }
    equal
}
fn assert9(r: &mut Reporter, m: &Matrix, expected: [f32; 9]) {
    let mut buffer = [0.; 9];
    m.get9(&mut buffer);
    for i in 0..9 {
        check!(r, buffer[i] == expected[i]);
    }
    for i in 0..9 {
        check!(r, m.rc(i / 3, i % 3) == expected[i]);
    }
}
fn test_set9(r: &mut Reporter) {
    let mut m = Matrix::I();
    m.reset();
    assert9(r, &m, [1., 0., 0., 0., 1., 0., 0., 0., 1.]);
    m.set_scale(2., 3.);
    assert9(r, &m, [2., 0., 0., 0., 3., 0., 0., 0., 1.]);
    m.post_translate(4., 5.);
    assert9(r, &m, [2., 0., 4., 0., 3., 5., 0., 0., 1.]);
    let mut buffer = [0.; 9];
    buffer[Matrix::M_SCALE_X] = 1.;
    buffer[Matrix::M_SCALE_Y] = 1.;
    buffer[Matrix::M_PERSP_2] = 1.;
    check!(r, !m.is_identity());
    m.set9(&buffer);
    check!(r, m.is_identity());
}
fn test_matrix_recttorect(r: &mut Reporter) {
    let mut src = Rect::new(0., 0., 10., 10.);
    let mut dst = src;
    let mut matrix = Matrix::Rect2Rect(&src, &dst);
    check!(r, matrix.is_some());
    check!(r, matrix.unwrap().get_type() == Matrix::IDENTITY_MASK);
    check!(r, matrix.unwrap().rect_stays_rect());
    dst.offset(1., 1.);
    matrix = Matrix::Rect2Rect(&src, &dst);
    check!(r, matrix.is_some());
    check!(r, matrix.unwrap().get_type() == Matrix::TRANSLATE_MASK);
    check!(r, matrix.unwrap().rect_stays_rect());
    dst.right += 1.;
    matrix = Matrix::Rect2Rect(&src, &dst);
    check!(r, matrix.is_some());
    check!(
        r,
        matrix.unwrap().get_type() == (Matrix::TRANSLATE_MASK | Matrix::SCALE_MASK)
    );
    check!(r, matrix.unwrap().rect_stays_rect());
    dst = src;
    dst.right = src.right * 2.;
    matrix = Matrix::Rect2Rect(&src, &dst);
    check!(r, matrix.is_some());
    check!(r, matrix.unwrap().get_type() == Matrix::SCALE_MASK);
    check!(r, matrix.unwrap().rect_stays_rect());
    src = Rect::make_xywh(10., 20., 0., 0.);
    matrix = Matrix::Rect2Rect(&src, &dst);
    check!(r, matrix.is_none());
    let mx = Matrix::RectToRectOrIdentity(&src, &dst);
    check!(r, mx.get_type() == Matrix::IDENTITY_MASK);
    let mut m = Matrix::Translate(20., 20.);
    check!(
        r,
        !m.set_rect_to_rect(&Rect::default(), &Rect::make_wh(10., 20.))
    );
    check!(r, m.is_identity());
}
fn test_flatten(r: &mut Reporter, m: &Matrix) {
    let mut buffer = [0u8; matrix_priv::MAX_FLATTEN_SIZE + 100];
    // Safe slice API has no null size-query argument. The source payload is nine SkScalars.
    let size1 = 9 * std::mem::size_of::<f32>();
    let size2 = matrix_priv::write_to_memory(m, &mut buffer);
    check!(r, size1 == size2);
    check!(r, size1 <= matrix_priv::MAX_FLATTEN_SIZE);
    let mut m2 = Matrix::I();
    let mut size3 = matrix_priv::read_from_memory(&mut m2, &buffer);
    check!(r, size1 == size3);
    let equal = are_equal(r, m, &m2);
    check!(r, equal);
    let mut buffer2 = [0u8; matrix_priv::MAX_FLATTEN_SIZE + 100];
    size3 = matrix_priv::write_to_memory(&m2, &mut buffer2);
    check!(r, size1 == size3);
    check!(r, buffer[..size1] == buffer2[..size1]);
}
fn recompose(mat: &Matrix, r1: Point, scale: Point, r2: Point) -> bool {
    let (c1, s1, sx, sy, c2, s2) = (r1.x, r1.y, scale.x, scale.y, r2.x, r2.y);
    relative(mat[0], sx * c1 * c2 - sy * s1 * s2)
        && relative(mat[1], -sx * s1 * c2 - sy * c1 * s2)
        && relative(mat[3], sx * c1 * s2 + sy * s1 * c2)
        && relative(mat[4], -sx * s1 * s2 + sy * c1 * c2)
}
fn test_matrix_min_max_scale(r: &mut Reporter) {
    let mut scales = [0.; 2];
    let mut identity = Matrix::I();
    identity.reset();
    check!(r, 1. == identity.get_min_scale());
    check!(r, 1. == identity.get_max_scale());
    let mut success = identity.get_min_max_scales(&mut scales);
    check!(r, success && scales == [1., 1.]);
    let mut scale = Matrix::I();
    scale.set_scale(2., 4.);
    check!(r, 2. == scale.get_min_scale());
    check!(r, 4. == scale.get_max_scale());
    success = scale.get_min_max_scales(&mut scales);
    check!(r, success && scales == [2., 4.]);
    let mut rot90 = Matrix::I();
    rot90.set_rotate(90.).post_scale(1. / 4., 1. / 2.);
    check!(r, 1. / 4. == rot90.get_min_scale());
    check!(r, 1. / 2. == rot90.get_max_scale());
    success = rot90.get_min_max_scales(&mut scales);
    check!(r, success && scales == [1. / 4., 1. / 2.]);
    let mut rotate = Matrix::I();
    rotate.set_rotate(128.);
    check!(r, near(1., rotate.get_min_scale()));
    check!(r, near(1., rotate.get_max_scale()));
    success = rotate.get_min_max_scales(&mut scales);
    check!(r, success);
    check!(r, near(1., scales[0]));
    check!(r, near(1., scales[1]));
    let mut translate = Matrix::I();
    translate.set_translate(10., -5.);
    check!(r, 1. == translate.get_min_scale());
    check!(r, 1. == translate.get_max_scale());
    success = translate.get_min_max_scales(&mut scales);
    check!(r, success && scales == [1., 1.]);
    let mut px = Matrix::I();
    px.reset().set_persp_x(1. / 1000.);
    check!(r, -1. == px.get_min_scale());
    check!(r, -1. == px.get_max_scale());
    success = px.get_min_max_scales(&mut scales);
    check!(r, !success);
    let big = Matrix::MakeAll(
        2.39394089e36,
        8.85347779e36,
        9.26526204e36,
        3.9159619e36,
        1.44823453e37,
        1.51559342e37,
        0.,
        0.,
        1.,
    );
    success = big.get_min_max_scales(&mut scales);
    check!(r, !success);
    let negative = Matrix::MakeAll(
        0.00436534, 0.114138, 0.37141, 0.00358857, 0.0936228, -0.0174198, 0., 0., 1.,
    );
    success = negative.get_min_max_scales(&mut scales);
    check!(r, success && scales[0] == 0.);
    let mut py = Matrix::I();
    py.reset().set_persp_y(-1. / 500.);
    check!(r, -1. == py.get_min_scale());
    check!(r, -1. == py.get_max_scale());
    scales = [-5., -5.];
    success = py.get_min_max_scales(&mut scales);
    check!(r, !success && scales == [-5., -5.]);
    let base = [scale, rot90, rotate, translate, px, py];
    let mut mats = [Matrix::I(); 12];
    for i in 0..6 {
        mats[i] = base[i];
        let mut inv = Matrix::I();
        let invertible = mats[i].invert(Some(&mut inv));
        mats[i + 6] = inv;
        check!(r, invertible);
    }
    let mut rand = SkRandom::new();
    let mut m = 0;
    while m < 1000 {
        let mut mat = Matrix::I();
        mat.reset();
        for _ in 0..4 {
            let x = rand.next_u() as usize % mats.len();
            mat.post_concat(&mats[x]);
        }
        let min_scale = mat.get_min_scale();
        let max_scale = mat.get_max_scale();
        check!(r, (min_scale < 0.) == (max_scale < 0.));
        check!(r, (max_scale < 0.) == mat.has_perspective());
        success = mat.get_min_max_scales(&mut scales);
        check!(r, success == !mat.has_perspective());
        check!(r, !success || scales == [min_scale, max_scale]);
        if mat.has_perspective() {
            continue;
        }
        let mut max = 0.;
        let mut min = f32::MAX;
        let mut vectors = [Point::default(); 1000];
        let mut i = 0;
        while i < vectors.len() {
            vectors[i].x = rand.next_s_scalar1();
            vectors[i].y = rand.next_s_scalar1();
            if !vectors[i].normalize() {
                continue;
            }
            i += 1;
        }
        let source = vectors;
        mat.map_vectors(&mut vectors, &source);
        for v in vectors {
            let d = v.length();
            check!(r, d / max_scale < 105. / 100.);
            check!(r, min_scale / d < 105. / 100.);
            if max < d {
                max = d;
            }
            if min > d {
                min = d;
            }
        }
        check!(r, max / max_scale >= 97. / 100.);
        check!(r, min_scale / min >= 97. / 100.);
        m += 1;
    }
}
fn point3_equal(a: &[Point3], b: &[Point3]) -> bool {
    a.iter()
        .zip(b)
        .all(|(a, b)| relative(a.x, b.x) && relative(a.y, b.y) && relative(a.z, b.z))
}
fn naive_homogeneous(m: &Matrix, src: Point3, dst: Point3) -> bool {
    let ms = m.get9_array();
    let res = Point3::new(
        src.x * ms[0] + src.y * ms[1] + src.z * ms[2],
        src.x * ms[3] + src.y * ms[4] + src.z * ms[5],
        src.x * ms[6] + src.y * ms[7] + src.z * ms[8],
    );
    point3_equal(&[res], &[dst])
}
fn test_matrix_homogeneous(r: &mut Reporter) {
    let mut mat = Matrix::I();
    let mut rand = SkRandom::new();
    let mut triples = [Point3::default(); 1000];
    for p in &mut triples {
        p.x = rand.next_range_f(-3000., 3000.);
        p.y = rand.next_range_f(-3000., 3000.);
        p.z = rand.next_range_f(-3000., 3000.);
    }
    let mut mats = [Matrix::I(); 1000];
    for m in &mut mats {
        for j in 0..9 {
            m.set(j, rand.next_range_f(-3000., 3000.));
        }
    }
    mat.reset();
    let mut dst = [Point3::default(); 1000];
    mat.map_homogeneous_points(&mut dst, &triples);
    check!(r, point3_equal(&triples, &dst));
    let zero = Point3::new(0., 0., 0.);
    mat.set_all(0., 0., 0., 0., 0., 0., 0., 0., 0.);
    mat.map_homogeneous_points(&mut dst, &triples);
    for p in dst {
        check!(r, point3_equal(&[p], &[zero]));
    }
    for m in &mats {
        let dst = m.map_homogeneous_point(zero);
        check!(r, point3_equal(&[dst], &[zero]));
    }
    mats[0].map_homogeneous_points(&mut [], &[]);
    mat.set_scale(5000., 5000.);
    for stage in 0..3 {
        if stage == 1 {
            mat.set_rotate(15.5);
        }
        if stage == 2 {
            mat.set_rotate(-50.)
                .post_scale(5000., 5000.)
                .post_rotate(15.5);
        }
        let src = Point3::new(triples[0].x, triples[0].y, 1.);
        let mut p = Point::new(src.x, src.y);
        let dst = mat.map_homogeneous_point(src);
        p = mat.map_point(p);
        check!(r, near(dst.x, p.x));
        check!(r, near(dst.y, p.y));
        check!(r, near(dst.z, 1.));
    }
    for m in &mats {
        for src in triples {
            let dst = m.map_homogeneous_point(src);
            check!(r, naive_homogeneous(m, src, dst));
        }
    }
}
fn check_decomp_scale(original: &Matrix) -> bool {
    let mut scale = Size::default();
    let mut remaining = Matrix::I();
    if !original.decompose_scale(&mut scale, &mut remaining) {
        return false;
    }
    if scale.width <= 0. || scale.height <= 0. {
        return false;
    }
    let mut reconstituted = remaining;
    reconstituted.pre_scale(scale.width, scale.height);
    if !nearly_equal(original, &reconstituted) {
        return false;
    }
    let pts = [
        Point::new(0., 0.),
        Point::new(1., 1.),
        Point::new(1., 0.5),
        Point::new(-1., -0.5),
        Point::new(-1., 2.),
    ];
    let mut v1 = [Point::default(); 5];
    original.map_points(&mut v1, &pts);
    let mut v2 = [Point::default(); 5];
    let scale_mat = Matrix::Scale(scale.width, scale.height);
    scale_mat.map_points(&mut v2, &pts);
    remaining.map_points_in_place(&mut v2);
    for i in 0..5 {
        if !((v1[i].x - v2[i].x).abs() <= 0.00001 && (v1[i].y - v2[i].y).abs() <= 0.00001) {
            return false;
        }
    }
    true
}
fn test_decompScale(r: &mut Reporter) {
    let mut m = Matrix::I();
    m.reset();
    check!(r, check_decomp_scale(&m));
    m.set_scale(2., 3.);
    check!(r, check_decomp_scale(&m));
    m.set_rotate_pivot(35., 0., 0.);
    check!(r, check_decomp_scale(&m));
    m.set_scale(1., 0.);
    check!(r, !check_decomp_scale(&m));
    m.set_rotate_pivot(35., 0., 0.).pre_scale(2., 3.);
    check!(r, check_decomp_scale(&m));
    m.set_rotate_pivot(35., 0., 0.).post_scale(2., 3.);
    check!(r, check_decomp_scale(&m));
}
// Original registrations MatrixTest.cpp:868,1036,1050,1090,1104,1108,1114,1133.
#[test]
fn Matrix() {
    let mut r = Reporter::default();
    let (mut mat, mut inverse, mut iden1, mut iden2) =
        (Matrix::I(), Matrix::I(), Matrix::I(), Matrix::I());
    mat.reset();
    mat.set_translate(1., 1.);
    check!(r, mat.invert(Some(&mut inverse)));
    iden1.set_concat(&mat, &inverse);
    check!(r, is_identity(&iden1));
    mat.set_scale(2., 4.);
    check!(r, mat.invert(Some(&mut inverse)));
    iden1.set_concat(&mat, &inverse);
    check!(r, is_identity(&iden1));
    test_flatten(&mut r, &mat);
    mat.set_scale(1. / 2., 2.);
    check!(r, mat.invert(Some(&mut inverse)));
    iden1.set_concat(&mat, &inverse);
    check!(r, is_identity(&iden1));
    test_flatten(&mut r, &mat);
    mat.set_scale_pivot(3., 5., 20., 0.).post_rotate(25.);
    check!(r, mat.invert(None));
    check!(r, mat.invert(Some(&mut inverse)));
    iden1.set_concat(&mat, &inverse);
    check!(r, is_identity(&iden1));
    iden2.set_concat(&inverse, &mat);
    check!(r, is_identity(&iden2));
    test_flatten(&mut r, &mat);
    test_flatten(&mut r, &iden2);
    mat.set_scale(0., 1.);
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    mat.set_scale(1., 0.);
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    mat.set_all(0., 1., 2., 0., 1., -3.40277175e38, 1.00003040, 1., 0.);
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    mat.set_all(f32::from_bits(1), 0., 0., 0., 1., 0., 0., 0., 1.);
    check!(r, mat.is_scale_translate());
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    mat.set_all(2., 0., f32::NAN, 0., 2., 0., 0., 0., 1.);
    check!(r, mat.is_scale_translate());
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    mat.set_all(1., 0., f32::NAN, 0., 1., 0., 0., 0., 1.);
    check!(r, mat.is_translate());
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    mat.set_all(f32::MIN_POSITIVE, 0., f32::MAX, 0., 1., 0., 0., 0., 1.);
    check!(r, mat.is_scale_translate());
    check!(r, mat.is_finite());
    check!(r, !mat.invert(None));
    check!(r, !mat.invert(Some(&mut inverse)));
    let samples = [
        (0., 0., 0., 0., false),
        (0., 0., 0., 1., false),
        (0., 0., 1., 0., false),
        (0., 0., 1., 1., false),
        (0., 1., 0., 0., false),
        (0., 1., 0., 1., false),
        (0., 1., 1., 0., true),
        (0., 1., 1., 1., false),
        (1., 0., 0., 0., false),
        (1., 0., 0., 1., true),
        (1., 0., 1., 0., false),
        (1., 0., 1., 1., false),
        (1., 1., 0., 0., false),
        (1., 1., 0., 1., false),
        (1., 1., 1., 0., false),
        (1., 1., 1., 1., false),
    ];
    for (a, b, c, d, expected) in samples {
        let mut m = Matrix::I();
        m.reset();
        m.set(Matrix::M_SCALE_X, a)
            .set(Matrix::M_SKEW_X, b)
            .set(Matrix::M_SKEW_Y, c)
            .set(Matrix::M_SCALE_Y, d);
        check!(r, m.rect_stays_rect() == expected);
    }
    mat.reset();
    mat.set(Matrix::M_SCALE_X, 1.)
        .set(Matrix::M_SKEW_X, 2.)
        .set(Matrix::M_TRANS_X, 3.)
        .set(Matrix::M_SKEW_Y, 4.)
        .set(Matrix::M_SCALE_Y, 5.)
        .set(Matrix::M_TRANS_Y, 6.);
    let mut affine = [0.; 6];
    check!(r, mat.as_affine(&mut affine));
    for (a, m) in [
        (Matrix::A_SCALE_X, Matrix::M_SCALE_X),
        (Matrix::A_SKEW_Y, Matrix::M_SKEW_Y),
        (Matrix::A_SKEW_X, Matrix::M_SKEW_X),
        (Matrix::A_SCALE_Y, Matrix::M_SCALE_Y),
        (Matrix::A_TRANS_X, Matrix::M_TRANS_X),
        (Matrix::A_TRANS_Y, Matrix::M_TRANS_Y),
    ] {
        check!(r, affine[a] == mat.get(m));
    }
    mat.set(Matrix::M_PERSP_1, 1. / 2.);
    check!(r, !mat.as_affine(&mut affine));
    let mut mat2 = Matrix::I();
    mat2.reset();
    mat.reset();
    let zero = 0.;
    mat.set(Matrix::M_SKEW_X, -zero);
    let equal = are_equal(&mut r, &mat, &mat2);
    check!(r, equal);
    mat2.reset();
    mat.reset();
    mat.set(Matrix::M_SKEW_X, f32::NAN);
    mat2.set(Matrix::M_SKEW_X, f32::NAN);
    let equal = are_equal(&mut r, &mat, &mat2);
    check!(r, !equal);
    test_matrix_min_max_scale(&mut r);
    test_matrix_preserve_shape(&mut r);
    test_matrix_recttorect(&mut r);
    test_matrix_decomposition(&mut r);
    test_matrix_homogeneous(&mut r);
    test_set9(&mut r);
    test_decompScale(&mut r);
    mat.set_scale_translate(2., 3., 1., 4.);
    mat2.set_scale(2., 3.).post_translate(1., 4.);
    check!(r, mat == mat2);
    r.finish();
}
#[test]
fn Matrix_Concat() {
    let mut r = Reporter::default();
    let mut a = Matrix::I();
    a.set_translate(10., 20.);
    let mut b = Matrix::I();
    b.set_scale(3., 5.);
    let mut expected = Matrix::I();
    expected.set_concat(&a, &b);
    check!(r, expected == Matrix::Concat(&a, &b));
    r.finish();
}
#[test]
fn Matrix_maprects() {
    let mut r = Reporter::default();
    let scale = 1000.;
    let mut mat = Matrix::I();
    mat.set_scale(2., 3.).post_translate(1., 4.);
    let mut rand = SkRandom::new();
    for _ in 0..10000 {
        let src = Rect::new(
            rand.next_s_scalar1() * scale,
            rand.next_s_scalar1() * scale,
            rand.next_s_scalar1() * scale,
            rand.next_s_scalar1() * scale,
        );
        let mut dst = [Rect::default(); 4];
        let mut mapped = [Point::default(); 2];
        mat.map_points(
            &mut mapped,
            &[
                Point::new(src.left, src.top),
                Point::new(src.right, src.bottom),
            ],
        );
        dst[0] = Rect::new(mapped[0].x, mapped[0].y, mapped[1].x, mapped[1].y);
        dst[0].sort();
        mat.map_rect_into(&mut dst[1], &src);
        dst[2] = mat.map_rect_scale_translate(&src);
        dst[3] = mat.map_rect(&src);
        check!(r, dst[0] == dst[1]);
        check!(r, dst[0] == dst[2]);
        check!(r, dst[0] == dst[3]);
    }
    let m0 = Matrix::Scale(1e20, 1e20);
    let mut m1 = Matrix::I();
    m1.set_rotate(30.).post_scale(1e20, 1e20);
    for m in [m0, m1] {
        let mut rect = Rect::new(0., 0., 1e20, 1e20);
        check!(r, rect.is_finite());
        rect = m.map_rect(&rect);
        check!(r, !rect.is_finite());
    }
    r.finish();
}
#[test]
fn Matrix_mapRect_skbug12335() {
    let mut r = Reporter::default();
    let rect = Rect::new(0., 0., 319., 620.);
    let m = Matrix::MakeAll(
        0.000152695269,
        0.,
        -6.53848401e-05,
        -1.75697533e-05,
        0.000157153074,
        -1.10847975e-06,
        -6.00415362e-08,
        0.,
        0.000169880834,
    );
    let out = m.map_rect(&rect);
    check!(r, !out.is_empty());
    r.finish();
}
#[test]
fn Matrix_Ctor() {
    let mut r = Reporter::default();
    check!(r, Matrix::default() == Matrix::I());
    r.finish();
}
#[test]
fn Matrix_LookAt() {
    let mut r = Reporter::default();
    let m = M44::LookAt(
        V3::new(0., 0., 0.),
        V3::new(0., 0., 0.),
        V3::new(0., 0., 0.),
    );
    check!(r, m == M44::I());
    r.finish();
}
#[test]
fn Matrix_SetRotateSnap() {
    let mut r = Reporter::default();
    let mut m = Matrix::I();
    let mut deg = 90.;
    while deg <= 1080. {
        m.set_rotate(deg);
        check!(r, m.rect_stays_rect());
        m.set_rotate(-deg);
        check!(r, m.rect_stays_rect());
        deg += 90.;
    }
    m.set_rotate(0.01);
    check!(r, !m.rect_stays_rect());
    r.finish();
}
#[test]
fn Matrix_rectStaysRect_zeroScale() {
    let mut r = Reporter::default();
    for (x, y) in [(0., 0.), (0., 2.), (2., 0.)] {
        check!(r, !Matrix::Scale(x, y).rect_stays_rect());
    }
    let src = Rect::new(0., 0., 10., 10.);
    for dst in [
        Rect::new(0., 0., 0., 0.),
        Rect::new(0., 0., 0., 20.),
        Rect::new(0., 0., 20., 0.),
    ] {
        check!(r, !Matrix::Rect2Rect(&src, &dst).unwrap().rect_stays_rect());
    }
    let rect = Matrix::I();
    check!(r, rect.rect_stays_rect());
    for (x, y) in [(0., 0.), (0., 2.), (2., 0.)] {
        let mut non_rect = rect;
        non_rect.pre_scale(x, y);
        check!(r, !non_rect.rect_stays_rect());
    }
    for (x, y) in [(0., 0.), (0., 2.), (2., 0.)] {
        let mut m = Matrix::I();
        m.set_scale(x, y);
        check!(r, !m.rect_stays_rect());
    }
    let mut m = Matrix::I();
    m.set_scale_translate(0., 0., 10., 10.);
    check!(r, !m.rect_stays_rect());
    r.finish();
}
