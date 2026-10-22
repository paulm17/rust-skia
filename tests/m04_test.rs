// Faithful M04 translations. BSD license: tests/LICENSE.skia.
#![allow(non_snake_case, unused_mut, unused_assignments, dead_code, clippy::all)]
mod support;
use rust_skia::*;
use support::{Reporter, reporter_assert as check};
#[path = "m04/point3.rs"]
mod point3;
#[path = "m04/rect.rs"]
mod rect;
#[path = "m04/rrect.rs"]
mod rrect;
#[path = "m04/scale_to_sides.rs"]
mod scale_to_sides;
#[path = "m04/size.rs"]
mod size;
#[path = "m04/supplemental.rs"]
mod supplemental;
