// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// GeometryTest.cpp: GeometryChopMonoCubicAtY_Successful; complete original fixture sequence.
#[test]
fn GeometryChopMonoCubicAtY_Successful() {
    let mut r = Reporter::default();
    mono(
        &mut r,
        "straight, positive slope @ 2.5",
        &[
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 10.0),
        ],
        2.5,
        &[
            Point::new(0.000000, 0.000000),
            Point::new(0.000000, 0.000000),
            Point::new(1.065055, 1.065055),
            Point::new(2.500000, 2.500000),
            Point::new(5.461981, 5.461981),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "straight, positive slope @ 5.0",
        &[
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 10.0),
        ],
        5.0,
        &[
            Point::new(0.000000, 0.000000),
            Point::new(0.000000, 0.000000),
            Point::new(2.500000, 2.500000),
            Point::new(5.000000, 5.000000),
            Point::new(7.500000, 7.500000),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "straight, positive slope @ 9.0",
        &[
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 10.0),
        ],
        9.0,
        &[
            Point::new(0.000000, 0.000000),
            Point::new(0.000000, 0.000000),
            Point::new(6.467375, 6.467375),
            Point::new(9.000000, 9.000000),
            Point::new(9.616623, 9.616623),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "straight, positive slope @ 10.0",
        &[
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 10.0),
        ],
        10.0,
        &[
            Point::new(0.000000, 0.000000),
            Point::new(0.000000, 0.000000),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
            Point::new(10.000000, 10.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "curve, positive slope @ 2.0",
        &[
            Point::new(1.0, 1.0),
            Point::new(5.0, 2.0),
            Point::new(7.0, 4.0),
            Point::new(8.0, 7.0),
        ],
        2.0,
        &[
            Point::new(1.000000, 1.000000),
            Point::new(2.055050, 1.263763),
            Point::new(2.970959, 1.597096),
            Point::new(3.766077, 2.000000),
            Point::new(5.985480, 3.124621),
            Point::new(7.263762, 4.791288),
            Point::new(8.000000, 7.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "curve, positive slope @ 5.0",
        &[
            Point::new(1.0, 1.0),
            Point::new(5.0, 2.0),
            Point::new(7.0, 4.0),
            Point::new(8.0, 7.0),
        ],
        5.0,
        &[
            Point::new(1.000000, 1.000000),
            Point::new(4.033223, 1.758306),
            Point::new(5.916391, 3.091639),
            Point::new(7.085550, 5.000000),
            Point::new(7.458195, 5.608251),
            Point::new(7.758306, 6.274917),
            Point::new(8.000000, 7.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "curve, negative slope @ 5.0",
        &[
            Point::new(2.0, 7.0),
            Point::new(3.0, 2.0),
            Point::new(6.0, 3.0),
            Point::new(11.0, 2.0),
        ],
        5.0,
        &[
            Point::new(2.000000, 7.000000),
            Point::new(2.162856, 6.185719),
            Point::new(2.378757, 5.530570),
            Point::new(2.647702, 5.000000),
            Point::new(4.030182, 2.272668),
            Point::new(6.814281, 2.837144),
            Point::new(11.000000, 2.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "curve, negative slope @ 3.0",
        &[
            Point::new(2.0, 7.0),
            Point::new(3.0, 2.0),
            Point::new(6.0, 3.0),
            Point::new(11.0, 2.0),
        ],
        3.0,
        &[
            Point::new(2.000000, 7.000000),
            Point::new(2.500000, 4.500000),
            Point::new(3.500000, 3.500000),
            Point::new(5.000000, 3.000000),
            Point::new(6.500000, 2.500000),
            Point::new(8.500000, 2.500000),
            Point::new(11.000000, 2.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "curve, negative slope @ 2.5",
        &[
            Point::new(2.0, 7.0),
            Point::new(3.0, 2.0),
            Point::new(6.0, 3.0),
            Point::new(11.0, 2.0),
        ],
        2.5,
        &[
            Point::new(2.000000, 7.000000),
            Point::new(2.750000, 3.250000),
            Point::new(4.625000, 2.875000),
            Point::new(7.625000, 2.500000),
            Point::new(8.625000, 2.375000),
            Point::new(9.750000, 2.250000),
            Point::new(11.000000, 2.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "inverted curve, negative slope @ 5.0",
        &[
            Point::new(11.0, 2.0),
            Point::new(6.0, 3.0),
            Point::new(3.0, 2.0),
            Point::new(2.0, 7.0),
        ],
        5.0,
        &[
            Point::new(11.000000, 2.000000),
            Point::new(6.814281, 2.837144),
            Point::new(4.030182, 2.272668),
            Point::new(2.647702, 5.000000),
            Point::new(2.378757, 5.530570),
            Point::new(2.162856, 6.185719),
            Point::new(2.000000, 7.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "inverted curve, negative slope @ 3.0",
        &[
            Point::new(11.0, 2.0),
            Point::new(6.0, 3.0),
            Point::new(3.0, 2.0),
            Point::new(2.0, 7.0),
        ],
        3.0,
        &[
            Point::new(11.000000, 2.000000),
            Point::new(8.500000, 2.500000),
            Point::new(6.500000, 2.500000),
            Point::new(5.000000, 3.000000),
            Point::new(3.500000, 3.500000),
            Point::new(2.500000, 4.500000),
            Point::new(2.000000, 7.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "inverted curve, negative slope @ 2.5",
        &[
            Point::new(11.0, 2.0),
            Point::new(6.0, 3.0),
            Point::new(3.0, 2.0),
            Point::new(2.0, 7.0),
        ],
        2.5,
        &[
            Point::new(11.000000, 2.000000),
            Point::new(9.750000, 2.250000),
            Point::new(8.625000, 2.375000),
            Point::new(7.625000, 2.500000),
            Point::new(4.625000, 2.875000),
            Point::new(2.750000, 3.250000),
            Point::new(2.000000, 7.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "big curve, negative slope @ 90",
        &[
            Point::new(-2.0, 100.0),
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(100.0, -2.0),
        ],
        90.,
        &[
            Point::new(-2.000000, 100.000000),
            Point::new(-1.930979, 96.548965),
            Point::new(-1.864341, 93.217033),
            Point::new(-1.795892, 90.000000),
            Point::new(0.119096, -0.002382),
            Point::new(3.451032, -0.069021),
            Point::new(100.000000, -2.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "big curve, negative slope @ 10",
        &[
            Point::new(-2.0, 100.0),
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(100.0, -2.0),
        ],
        10.,
        &[
            Point::new(-2.000000, 100.000000),
            Point::new(-0.937505, 46.875271),
            Point::new(-0.439458, 21.972910),
            Point::new(14.787060, 10.000000),
            Point::new(28.222368, -0.564447),
            Point::new(53.124729, -1.062495),
            Point::new(100.000000, -2.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "big curve, negative slope @ 0",
        &[
            Point::new(-2.0, 100.0),
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(100.0, -2.0),
        ],
        0.,
        &[
            Point::new(-2.000000, 100.000000),
            Point::new(-0.426983, 21.349131),
            Point::new(-0.091157, 4.557854),
            Point::new(48.633648, 0.000000),
            Point::new(61.859592, -1.237192),
            Point::new(78.650871, -1.573017),
            Point::new(100.000000, -2.000000),
        ],
        false,
    );
    mono(
        &mut r,
        "ossfuzz:55680 curve barely crosses Y axis",
        &[
            Point::new(-250.121582, -1180.09509),
            Point::new(10.007843, -1180.09509),
            Point::new(20.015685, -786.041259),
            Point::new(40.0313721, 2.0664072),
        ],
        0.,
        &[
            Point::new(-250.121582, -1180.095093),
            Point::new(9.780392, -1180.095093),
            Point::new(19.997992, -786.730042),
            Point::new(39.978889, 0.000000),
            Point::new(39.996376, 0.688501),
            Point::new(40.013870, 1.377304),
            Point::new(40.031372, 2.066407),
        ],
        false,
    );
    r.finish();
}
#[test]
fn GeometryChopMonoCubicAtY_OutOfRangeReturnFalse() {
    let mut r = Reporter::default();
    let inputs = [p(0., 0.), p(0., 0.), p(10., 10.), p(10., 10.)];
    let mut outputs = [Point::default(); 7];
    check!(r, !sk_chop_mono_cubic_at_y(&inputs, -10., &mut outputs));
    check!(r, !sk_chop_mono_cubic_at_y(&inputs, 20., &mut outputs));
    r.finish();
}
