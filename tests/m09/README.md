# M09 test correspondence

Reference: Skia `b81c69985fd14bb7bfbd356b7618633ce07dc1a4`.
License: `tests/LICENSE.skia`.

| Original registration | Rust test | Context | Translation |
| --- | --- | --- | --- |
| `tests/ClipStackTest.cpp:828`, `ClipStack` | `tests/m09/clip_stack.rs`, `ClipStack` | CPU, no feature guards or resources | Entire registration and all reached helpers |
| `tests/RTreeTest.cpp:65`, `RTree` | `tests/m09/rtree.rs`, `RTree` | CPU, no feature guards or resources | Entire registration and all reached helpers; default SkRandom seed, 100 × 200 rectangles, 100 × 50 queries |

Written only; not compiled, formatted, executed, discovered, audited or accepted.
Source hashes remain in the existing immutable `plan/inventory/source-hashes.csv`;
no new hashes or execution evidence were generated.

ClipStack helper closure: `test_assign_and_comparison`, `assert_count`,
`test_iterators`, `test_bounds` (Rect, RRect, Path), `test_isWideOpen`,
`count`, `test_rect_inverse_fill`, `test_rect_replace`, `test_path_replace`,
`test_rect_merging`, `test_quickContains`, `set_region_to_stack`,
`test_invfill_diff_bug`, `test_is_rrect_deep_rect_stack`.
RTree helper closure: `random_rect`, `verify_query`, `run_queries`.

Adaptations: existing Reporter retains nonfatal REPORTER_ASSERT and ERRORF
behavior. Original SkASSERT checks use debug_assert. Dereferenced element pointers
become Option references with expect where C++ requires a non-null pointer.
Assignment uses Clone; identity matrices use Matrix::default; constructors and
borrowed arguments use the Rust APIs. Default path direction is Cw, start index 0.
Repeated independent scopes in merging, replacement and quickContains are encoded
as ordered tables with exactly the same inputs, expected values and operations.
The region helper uses op_region for C++'s in-place two-argument op (the Rust op
method instead takes both input regions). No tolerance or AA coverage assertion
was introduced into the original cases. RTree retains f64 pow and integer cast,
left/right/top/bottom RNG call order, sorted rectangles and ordered hit comparison.
No platform skip was added.

`tests/m09/supplemental.rs` is separate supplemental coverage: half-open and empty
bounds, reserved sentinel coordinates, large bounds, translation overflow policy,
all six region ops with independent per-pixel truth tables, explicit little-endian
empty/rect/complex fixtures, truncated and malformed decode with transactional
state, COW independence, input buffer and geometry lifetimes, AA region copy,
RTree capacity transitions and stable original indices, BBH Arc ownership and
clip shader destruction. The overflow expected values follow SkRegion.cpp's
pin_offset_s32 and setRect sentinel rejection. The wire fixtures encode the
reference writeToMemory layout directly, without using the Rust encoder as oracle.

Later integration obligations remain in the M09 milestone document, including
RegionTest and AAClipTest at M14. These supplemental tests do not claim those
registrations or analytic AA coverage.

Deepseek commands, from the repository root (no optional features):

```sh
cargo test --locked --test m09_test
cargo test --release --locked --test m09_test
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

Deepseek owns execution, any needed formatting, C++ comparison, registration
reconciliation, source/coverage audits, acceptance reports and milestone status.
