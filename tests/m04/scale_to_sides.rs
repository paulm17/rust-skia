// Translated from Skia; BSD license: tests/LICENSE.skia.
use super::*;
// ScaleToSidesTest.cpp:16. The original body has no reporter assertions.
#[test]
fn ScaleToSides() {
    let interesting_values: [f64; 27] = [
        // From sample app - PathFuzzer
        260.01662826538085938,
        63.61007690429687500,
        795.98901367187500000,
        217.71697616577148438,
        686.15960693359375000,
        556.57641601562500000,
        // From skp bitbucket
        111.60000228881836,
        55.800003051757813,
        0.99999996581812677920,
        0.0,
        0.5,
        1.0,
        2.0,
        3.0,
        33.0,
        33554430.0,
        33554431.0,
        33554464.0,
        333333332.0,
        333333333.0,
        333333334.0,
        f32::MAX as f64,
        f32::EPSILON as f64,
        f32::MIN_POSITIVE as f64,
        340282569745034499980078846904281071616.0,
        170141284872517249990039423452140535808.0,
        170141244307698042686698575557637963776.0,
    ];
    let n = interesting_values.len();
    for s in 0..=n {
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    if interesting_values[i] > f32::MAX as f64
                        || interesting_values[j] > f32::MAX as f64
                    {
                        continue;
                    }
                    let mut radius1 = interesting_values[i] as f32;
                    let mut radius2 = interesting_values[j] as f32;
                    let width = interesting_values[k];
                    let mut scale = width / (radius1 as f64 + radius2 as f64);
                    if width > 0.0 {
                        if s != 0 {
                            scale = scale.min(interesting_values[s - 1]);
                        }
                        if scale < 1.0 && scale > 0.0 {
                            ScaleToSides::adjust_radii(width, scale, &mut radius1, &mut radius2);
                        }
                    }
                }
            }
        }
    }
}
