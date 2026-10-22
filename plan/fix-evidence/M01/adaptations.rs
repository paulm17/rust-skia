use rust_skia::*;
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    assert_eq!(float_interp_func(0., &[], &[]), 0.);
    assert!(is_finite_slice(&[]));
    assert!(PointPriv::are_finite(&[]));
    assert!(scalars_equal_n(&[], &[], 0));
    assert!(scalars_equal_n(&[1., 2.], &[1., 3.], 1));
    assert!(!scalars_equal_n(&[1., 2.], &[1., 3.], 2));
    assert!(!scalars_equal(&[1.], &[1., 2.]));
    assert!(std::panic::catch_unwind(|| scalars_equal_n(&[1.], &[1.], 2)).is_err());
    let mismatched = std::panic::catch_unwind(|| float_interp_func(1., &[0., 1.], &[7.]));
    if cfg!(debug_assertions) { assert!(mismatched.is_err()); }
    else { assert_eq!(mismatched.unwrap(), 7.); }
    assert_eq!(-IPoint::new(i32::MIN, 1), IPoint::new(i32::MIN, -1));
    let mut p = IPoint::new(i32::MIN, 1);
    PointPriv::negate(&mut p);
    assert_eq!(p, IPoint::new(i32::MIN, -1));
    assert_eq!(floor_to_int_no_saturate(f32::NAN), 0);
    assert_eq!(ceil_to_int_no_saturate(f32::INFINITY), i32::MAX);
    assert_eq!(round_to_int_no_saturate(f32::NEG_INFINITY), i32::MIN);
    println!("PASS: empty/mismatched slices, explicit counts, finite arrays, minimum negation, invalid numeric casts");
}
