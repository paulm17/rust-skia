// Translated from Skia. BSD license: tests/LICENSE.skia.
#![allow(non_snake_case, unused_mut, unused_assignments, dead_code, clippy::all)]
mod support;
use rust_skia::*;
use rust_skia::{matrix_priv, matrix_utils, random::SkRandom};
use support::{Reporter, reporter_assert as check};
const NZ: f32 = 1.0 / 4096.0;
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= NZ
}
fn relative(a: f32, b: f32) -> bool {
    let diff = (a - b).abs();
    diff < NZ || diff <= (if b.abs() > a.abs() { b.abs() } else { a.abs() }) * NZ
}
#[path = "m05/m44.rs"]
mod m44_tests;
#[path = "m05/matrix.rs"]
mod matrix_tests;
#[path = "m05/rect.rs"]
mod rect_tests;
#[path = "m05/supplemental.rs"]
mod supplemental;
