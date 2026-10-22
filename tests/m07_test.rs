// Copyright 2011–2025 Google Inc. / Google LLC. BSD: tests/LICENSE.skia.
// Translated from Skia b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
// Source correspondence and language adaptations: m07/README.md.
#![allow(
    non_snake_case,
    dead_code,
    unused_imports,
    unused_mut,
    unused_assignments,
    clippy::all
)]
mod support;
use SkPathDirection::{Ccw, Cw};
use SkPathFillType::Winding;
use SkPathVerb::{Close as X, Conic as K, Cubic as C, Line as L, Move as M, Quad as Q};
use rust_skia::random::SkRandom;
use rust_skia::sk_contour_measure::*;
use rust_skia::sk_parse_path::*;
use rust_skia::sk_path::*;
use rust_skia::sk_path_builder::*;
use rust_skia::sk_path_data::*;
use rust_skia::sk_path_iter::*;
use rust_skia::sk_path_measure::*;
use rust_skia::sk_path_priv::*;
use rust_skia::sk_path_raw::*;
use rust_skia::sk_path_raw_shapes::*;
use rust_skia::sk_path_types::*;
use rust_skia::*;
use std::sync::Arc;
use support::{Reporter, reporter_assert as check};
const DIRS: [SkPathDirection; 2] = [Cw, Ccw];
const FILL_TYPES: [SkPathFillType; 4] = [
    Winding,
    SkPathFillType::EvenOdd,
    SkPathFillType::InverseWinding,
    SkPathFillType::InverseEvenOdd,
];
const fn p(x: f32, y: f32) -> Point {
    Point::new(x, y)
}
fn rect(l: f32, t: f32, r: f32, b: f32) -> Rect {
    Rect::make_ltrb(l, t, r, b)
}
fn scale(x: f32, y: f32) -> Matrix {
    let mut m = Matrix::new();
    m.set_scale(x, y);
    m
}
fn translate(x: f32, y: f32) -> Matrix {
    let mut m = Matrix::new();
    m.set_translate(x, y);
    m
}
fn rotate(d: f32) -> Matrix {
    let mut m = Matrix::new();
    m.set_rotate(d);
    m
}
fn range(path: &SkPath) -> RangeIter<'_> {
    RangeIter::from_slices(path.verbs(), path.points(), path.conic_weights())
}
fn near(a: f32, b: f32) -> bool {
    rust_skia::scalar::scalar_nearly_equal_within(a, b, 1.0 / 4096.0)
}
#[path = "m07/builder.rs"]
mod builder;
#[path = "m07/data.rs"]
mod data;
#[path = "m07/measure.rs"]
mod measure;
#[path = "m07/misc.rs"]
mod misc;
#[path = "m07/parse_os.rs"]
mod parse_os;
#[path = "m07/raw.rs"]
mod raw;
#[path = "m07/rrect.rs"]
mod rrect;
#[path = "m07/subset.rs"]
mod subset;
#[path = "m07/supplemental.rs"]
mod supplemental;
