// Faithful M03 translations. BSD license: tests/LICENSE.skia.
#![allow(
    non_snake_case,
    dead_code,
    unused_imports,
    unused_mut,
    unused_assignments,
    clippy::all,
    unexpected_cfgs
)]
mod support;
use rust_skia::*;
use support::{Reporter, reporter_assert as check};
#[path = "m03/base64.rs"]
mod base64;
#[path = "m03/bit_set.rs"]
mod bit_set;
#[path = "m03/checksum.rs"]
mod checksum;
#[path = "m03/fits_in.rs"]
mod fits_in;
#[path = "m03/float16.rs"]
mod float16;
#[path = "m03/floating_point.rs"]
mod floating_point;
#[path = "m03/math.rs"]
mod math;
#[path = "m03/over_aligned.rs"]
mod over_aligned;
#[path = "m03/random.rs"]
mod random;
#[path = "m03/safe_math.rs"]
mod safe_math;
#[path = "m03/sort.rs"]
mod sort;
#[path = "m03/stream.rs"]
mod stream;
#[path = "m03/supplemental.rs"]
mod supplemental;
#[path = "m03/tarray.rs"]
mod tarray;
#[path = "m03/tdpqueue.rs"]
mod tdpqueue;
#[path = "m03/utils.rs"]
mod utils;
#[path = "m03/vx.rs"]
mod vx;
