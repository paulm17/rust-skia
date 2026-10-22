// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// GeometryTest.cpp: GeometryChopMonoCubicAtX_Successful; complete original fixture sequence.
#[test]
fn GeometryChopMonoCubicAtX_Successful() {
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
        true,
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
        true,
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
        true,
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
        true,
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
            Point::new(1.348275, 1.087069),
            Point::new(1.681389, 1.181719),
            Point::new(2.000000, 1.283949),
            Point::new(5.340694, 2.355856),
            Point::new(7.087069, 4.261207),
            Point::new(8.000000, 7.000000),
        ],
        true,
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
            Point::new(2.650396, 1.412599),
            Point::new(3.960316, 1.995436),
            Point::new(5.000000, 2.748511),
            Point::new(6.480158, 3.820634),
            Point::new(7.412599, 5.237797),
            Point::new(8.000000, 7.000000),
        ],
        true,
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
            Point::new(2.500000, 4.500000),
            Point::new(3.500000, 3.500000),
            Point::new(5.000000, 3.000000),
            Point::new(6.500000, 2.500000),
            Point::new(8.500000, 2.500000),
            Point::new(11.000000, 2.000000),
        ],
        true,
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
            Point::new(2.228714, 5.856432),
            Point::new(2.562047, 5.026724),
            Point::new(3.000000, 4.415163),
            Point::new(4.476901, 2.352807),
            Point::new(7.143568, 2.771286),
            Point::new(11.000000, 2.000000),
        ],
        true,
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
            Point::new(2.131881, 6.340593),
            Point::new(2.298548, 5.785543),
            Point::new(2.500000, 5.316498),
            Point::new(3.826073, 2.228977),
            Point::new(6.659407, 2.868119),
            Point::new(11.000000, 2.000000),
        ],
        true,
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
            Point::new(8.500000, 2.500000),
            Point::new(6.500000, 2.500000),
            Point::new(5.000000, 3.000000),
            Point::new(3.500000, 3.500000),
            Point::new(2.500000, 4.500000),
            Point::new(2.000000, 7.000000),
        ],
        true,
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
            Point::new(7.143568, 2.771286),
            Point::new(4.476901, 2.352807),
            Point::new(3.000000, 4.415163),
            Point::new(2.562047, 5.026724),
            Point::new(2.228714, 5.856432),
            Point::new(2.000000, 7.000000),
        ],
        true,
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
            Point::new(6.659407, 2.868119),
            Point::new(3.826073, 2.228977),
            Point::new(2.500000, 5.316498),
            Point::new(2.298548, 5.785543),
            Point::new(2.131881, 6.340593),
            Point::new(2.000000, 7.000000),
        ],
        true,
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
            Point::new(-0.069021, 3.451032),
            Point::new(-0.002382, 0.119096),
            Point::new(90.000000, -1.795892),
            Point::new(93.217033, -1.864341),
            Point::new(96.548965, -1.930979),
            Point::new(100.000000, -2.000000),
        ],
        true,
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
            Point::new(-1.062495, 53.124729),
            Point::new(-0.564447, 28.222368),
            Point::new(10.000000, 14.787060),
            Point::new(21.972910, -0.439458),
            Point::new(46.875271, -0.937505),
            Point::new(100.000000, -2.000000),
        ],
        true,
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
            Point::new(-1.573017, 78.650871),
            Point::new(-1.237192, 61.859592),
            Point::new(0.000000, 48.633648),
            Point::new(4.557854, -0.091157),
            Point::new(21.349131, -0.426983),
            Point::new(100.000000, -2.000000),
        ],
        true,
    );
    r.finish();
}
#[test]
fn GeometryChopMonoCubicAtX_OutOfRangeReturnFalse() {
    let mut r = Reporter::default();
    let inputs = [p(0., 0.), p(0., 0.), p(10., 10.), p(10., 10.)];
    let mut outputs = [Point::default(); 7];
    check!(r, !sk_chop_mono_cubic_at_x(&inputs, -10., &mut outputs));
    check!(r, !sk_chop_mono_cubic_at_x(&inputs, 20., &mut outputs));
    r.finish();
}
