// RectTest.cpp:281. BSD: tests/LICENSE.skia. Other cases retain their M04/M07/M14 gates.
use super::*;
#[test]
fn Rect_QuadContainsRect() {
    let mut r = Reporter::default();
    let mut epsilon = Matrix::MakeAll(
        0.984808, 0.173648, -98.4808, -0.173648, 0.984808, 17.3648, 0., 0., 1.,
    );
    epsilon.pre_translate(65., 0.);
    let tests = [
        (
            "Identity matrix contains success",
            true,
            Matrix::I(),
            [0, 0, 15, 15],
            [2, 2, 10, 10],
            0.,
        ),
        (
            "Identity matrix contains failure",
            false,
            Matrix::I(),
            [0, 0, 15, 15],
            [-2, -2, 10, 10],
            0.,
        ),
        (
            "Identity mapped rect contains itself",
            true,
            Matrix::I(),
            [0, 0, 10, 10],
            [0, 0, 10, 10],
            0.,
        ),
        (
            "Scaled rect contains success",
            true,
            Matrix::Scale(2., 3.4),
            [0, 0, 4, 4],
            [1, 1, 6, 6],
            0.,
        ),
        (
            "Scaled rect contains failure",
            false,
            Matrix::Scale(0.25, 0.3),
            [0, 0, 8, 8],
            [0, 0, 5, 5],
            0.,
        ),
        (
            "Rotate rect contains success",
            true,
            Matrix::rotate_deg_pivot(45., 10., 10.),
            [0, 0, 20, 20],
            [3, 3, 17, 17],
            0.,
        ),
        (
            "Rotate rect contains failure",
            false,
            Matrix::rotate_deg_pivot(45., 10., 10.),
            [0, 0, 20, 20],
            [2, 2, 18, 18],
            0.,
        ),
        (
            "Negative scale contains success",
            true,
            Matrix::Scale(-1., 1.),
            [0, 0, 10, 10],
            [-9, 1, -1, 9],
            0.,
        ),
        (
            "Empty rect contains nothing",
            false,
            Matrix::rotate_deg_pivot(45., 0., 0.),
            [10, 10, 10, 20],
            [10, 14, 10, 16],
            0.,
        ),
        (
            "MakeEmpty() contains nothing",
            false,
            Matrix::rotate_deg_pivot(45., 0., 0.),
            [0, 0, 0, 0],
            [0, 0, 1, 1],
            0.,
        ),
        (
            "Unsorted rect contains nothing",
            false,
            Matrix::I(),
            [10, 10, 0, 0],
            [2, 2, 8, 8],
            0.,
        ),
        (
            "Unsorted rect is contained",
            true,
            Matrix::I(),
            [0, 0, 10, 10],
            [8, 8, 2, 2],
            0.,
        ),
        (
            "Epsilon not contained",
            true,
            epsilon,
            [0, 0, 134, 215],
            [0, 0, 100, 200],
            0.001,
        ),
    ];
    for (label, expect, m, a, b, tol) in tests {
        eprintln!("{label}");
        let a = IRect::new(a[0], a[1], a[2], a[3]);
        let b = IRect::new(b[0], b[1], b[2], b[3]);
        check!(r, RectPriv::quad_contains_rect(&m, &a, &b, tol) == expect);
        let mut offset = m.map_vector(Point::new(0.5, 0.5));
        let mut af = Rect::from(a);
        af.offset(0.5, 0.5);
        let mut bf = Rect::from(b);
        bf.offset(offset.x, offset.y);
        check!(
            r,
            RectPriv::quad_contains_rect_m44(&M44::from(m), &af, &bf, tol) == expect
        );
        if tol != 0. {
            check!(r, RectPriv::quad_contains_rect(&m, &a, &b, 0.) == !expect);
            offset = m.map_vector(Point::new(0.5, 0.5));
            af = Rect::from(a);
            af.offset(0.5, 0.5);
            bf = Rect::from(b);
            bf.offset(offset.x, offset.y);
            check!(
                r,
                RectPriv::quad_contains_rect_m44(&M44::from(m), &af, &bf, 0.) == !expect
            );
        }
    }
    let a = Rect::new(1.83, -0.48, 15.53, 30.68);
    {
        eprintln!("Perspective, W > 0");
        let mut p = M44::Perspective(0.01, 10., std::f32::consts::PI / 3.);
        p.pre_translate(0., 5., -0.1);
        p.pre_concat(&M44::Rotate(V3::new(0., 1., 0.), 0.008));
        check!(
            r,
            RectPriv::quad_contains_rect_m44(&p, &a, &Rect::new(4., 10., 20., 45.), 0.)
        );
        check!(
            r,
            !RectPriv::quad_contains_rect_m44(&p, &a, &Rect::new(2., 6., 23., 50.), 0.)
        );
    }
    {
        eprintln!("Perspective, some W > 0");
        let mut p = M44::I();
        p.set_row(3, V4::new(-0.2, -0.6, 0., 8.));
        check!(
            r,
            RectPriv::quad_contains_rect_m44(&p, &a, &Rect::new(10., 50., 20., 60.), 0.)
        );
        check!(
            r,
            !RectPriv::quad_contains_rect_m44(&p, &a, &Rect::new(0., 1., 10., 10.), 0.)
        );
    }
    {
        eprintln!("Perspective, no W > 0");
        let mut p = M44::I();
        p.set_row(3, V4::new(-0.2, -0.6, 0., 8.));
        let mut na = a;
        na.offset(16., 31.);
        check!(
            r,
            !RectPriv::quad_contains_rect_m44(&p, &na, &Rect::new(10., 50., 20., 60.), 0.)
        );
        check!(
            r,
            !RectPriv::quad_contains_rect_m44(&p, &na, &Rect::new(-1.1, -1.8, -1., -1.79), 0.)
        );
        check!(
            r,
            !RectPriv::quad_contains_rect_m44(&p, &na, &Rect::new(-1.9, -2.3, -0.4, -1.6), 0.)
        );
    }
    r.finish();
}
