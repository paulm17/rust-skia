// Copyright 2013-2025 Google LLC and original Skia authors.
// Faithful M02 translations; BSD license: tests/LICENSE.skia.
// Keep original assertion syntax, including tautological width bounds and operator checks.
#![allow(
    non_snake_case,
    dead_code,
    clippy::needless_range_loop,
    clippy::int_plus_one,
    clippy::absurd_extreme_comparisons,
    clippy::manual_is_multiple_of,
    clippy::len_zero,
    clippy::cmp_null,
    clippy::explicit_auto_deref,
    clippy::nonminimal_bool,
    clippy::neg_cmp_op_on_partial_ord,
    clippy::comparison_to_empty,
    clippy::useless_vec,
    clippy::approx_constant
)]
mod support;
use rust_skia::*;
use support::{Reporter, reporter_assert as check};
#[path = "m02/arena.rs"]
mod arena;
#[path = "m02/block.rs"]
mod block;
#[path = "m02/containers.rs"]
mod containers;
#[path = "m02/cow.rs"]
mod cow;
#[path = "m02/data.rs"]
mod data;
#[path = "m02/deque.rs"]
mod deque;
#[path = "m02/executor.rs"]
mod executor;
#[path = "m02/fake_streams.rs"]
mod fake_streams;
#[path = "m02/full_string.rs"]
mod full_string;
#[path = "m02/graph.rs"]
mod graph;
#[path = "m02/hash.rs"]
mod hash;
#[path = "m02/llist.rs"]
mod llist;
#[path = "m02/metadata.rs"]
mod metadata;
#[path = "m02/more_templates.rs"]
mod more_templates;
#[path = "m02-api/polymorphic.rs"]
mod polymorphic;
#[path = "m02/raw.rs"]
mod raw;
#[path = "m02/refcount.rs"]
mod refcount;
#[path = "m02/sk_sp.rs"]
mod smart_pointer;
#[path = "m02/string.rs"]
mod strings;
#[path = "m02/supplemental.rs"]
mod supplemental;
#[path = "m02/tblocklist.rs"]
mod tblocklist;
#[path = "m02/templates.rs"]
mod templates;
#[path = "m02/utf.rs"]
mod utf;
#[path = "m02/utilities.rs"]
mod utilities;
