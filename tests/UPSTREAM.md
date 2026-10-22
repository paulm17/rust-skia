# Skia test correspondence

Reference repository: https://skia.googlesource.com/skia

Pinned C++ commit: `b81c69985fd14bb7bfbd356b7618633ce07dc1a4`

| Reference file | SHA-256 |
| --- | --- |
| `tests/ScalarTest.cpp` | `a0bf3e0ff1d29f64642e438bb0c3221009a2754a0c68098ef231ac10dfa3f51c` |
| `tests/PointTest.cpp` | `630ed39ff0f64f48f73a34fb38263491ba742cdf331553ee82d6e0dc2f3d04a1` |
| `tests/Test.h` | `0dfcdc5be1b0398af9cb0ecf9b268c2d61bc4677012234a3c8131d75f26b6103` |

## Test and helper mapping

M01 was validated on 2026-10-21 against the independently compiled pinned C++ CPU registry in debug and release. See [the acceptance report](../plan/acceptance/M01.md) for all seven assigned registration sites, source hashes, supplemental cases and complete results. Six original registrations execute; TestFalseCondition is excluded by its original false condition.

| C++ entry | Rust file / entry |
| --- | --- |
| `DEF_TEST(Scalar)` | `scalar_test.rs` / `Scalar` |
| `test_roundtoint` | `scalar_test.rs` / `test_roundtoint` |
| `test_isRectFinite` | `scalar_test.rs` / `test_isRectFinite` |
| `isFinite_int`, `isFinite_float`, `isFinite_mulzero` | `scalar_test.rs` / same names |
| `isFinite2_and`, `isFinite2_mulzeroadd` | `scalar_test.rs` / same names |
| `test_floatclass`, `test_isfinite` | `scalar_test.rs` / same names |
| `DEF_TEST(Point)` | `point_test.rs` / `Point` |
| `test_casts`, `test_Normalize` | `point_test.rs` / same names |
| `test_normalize_cannormalize_consistent` | `point_test.rs` / same name |
| `test_length`, `get_value`, `force_as_float`, `test_overflow` | `point_test.rs` / same names |
| `DEF_TEST(Point_setLengthFast)` | `point_test.rs` / `Point_setLengthFast` |
| `REPORTER_ASSERT`, reporter failure accumulation | `support/mod.rs` / `reporter_assert!`, `Reporter` |

## Translation audit

- Scalar preserves all four rounding assertions, seven classification inputs with three assertions each, ten finite-value records, three single-value procedures, two paired procedures, and all 100 pairs with all six procedure combinations. All six rectangle fixtures and assertions are retained. Total: 661 executed assertions when all cases complete normally.
- Point preserves both scalar-address comparisons; both length and both normalization assertions for each of the two input records; overflow diagnostics and its conditional failure assertion; successful set-length and normalized-length assertions; and all 6 × 4 × 5 normalization consistency cases. Total on success: 132 assertions, plus the original conditional assertion if overflow unexpectedly returns a finite length.
- Point_setLengthFast preserves all six inputs, both ignored operation return values, the original short-circuit subnormal skip, and the strict `> 0.999f` and `< 1.001f` assertions. The number of assertions depends on the original skip condition.
- Each reporter assertion evaluates its condition once and continues after failure. The Cargo test panics only at reporter completion. Unexpected Rust panics still terminate a test. The isolated reporter self-test verifies evaluation count, multiple failures, continued execution, diagnostics, and final failure; it runs in each integration test binary that includes the shared support module.

## Language adaptations

- Rust slices carry the C++ point-array pointer and count together. Iterator loops preserve input order and the complete Cartesian products. Tuples represent local record structs. The unused C++ `PointSet` type is omitted.
- Rust point construction initializes storage before `set`; the C++ tests leave that storage uninitialized before setting it. C++ aggregate point copies are represented by constructing points with the same components.
- `SkPoint::Normalize` maps to `normalize_returning_length`; static `Length` maps to `length_xy`. Other library calls use the corresponding existing Rust declarations. Production code was not changed during acceptance; Deepseek’s symbol mapping and working-tree hashes were reconciled.
- C++ integer literals in floating-point contexts become f32 literals. Scalar constants, near-equality, invert, integer-to-scalar conversion, rounding, finite/NaN checks, and float-to-bits calls still exercise the library APIs. C++ `std::isinf` and `floorf` reference calculations use Rust's native f32 operations.
- C++ `!!` and `SkToBool` are unnecessary for Rust boolean results. The independent exponent-bit and multiply-by-zero reference calculations are retained, including `y == y` for NaN detection.
- C++ `memcpy` in `force_as_float` becomes native `to_bits` / `from_bits`, with `black_box` to preserve a runtime storage boundary. The null-reporter branch is unreachable with Rust references and is omitted; `get_value` likewise preserves its always-non-null path using `black_box`.
- `FLT_MIN` becomes `f32::MIN_POSITIVE`. Overflow-producing arithmetic is intentional. No fast-math flags or relaxed tolerances are introduced. Windows warning pragmas have no Rust equivalent and are omitted.
- Failure locations refer to the translated Rust assertions. Overflow diagnostic formatting uses Rust formatting. Original test names are retained with a local naming-lint allowance. Source copyright notices are retained; the reference license is included as `LICENSE.skia`.

## Reproduce

From the Rust repository:

```sh
cargo test --tests --locked
cargo test --tests --release --locked
cargo fmt --all -- --check
cargo clippy --tests --locked -- -D warnings
```

To run only the original tests, excluding reporter self-tests:

```sh
cargo test --test scalar_test Scalar -- --exact
cargo test --test point_test Point -- --exact
cargo test --test point_test Point_setLengthFast -- --exact
```

Validation: all six enabled M01 original registrations, reporter self-tests, and labeled supplemental cases passed in both profiles; the false-condition registration was excluded as upstream requires. Formatting and Clippy with warnings denied passed. See the acceptance report for counts and independent C++ results. No failures require implementation changes.

## M01 harness correspondence

`TestNormal`, `TestTrueCondition`, and `TestCtsEnforcement` map to identically named tests in `harness_test.rs`. `TestFalseCondition` maps to the retained `false_condition` body, excluded by the original condition and checked separately by `supplemental_registry_excludes_false_condition_and_preserves_head_order`. Reached Registry and CTS helpers live under `support/harness.rs`. Unreached TestHarness classification and non-CPU TestType scheduling are explicitly dispositioned in the acceptance report. Source hashes for all assigned files and reached support are in `plan/acceptance/M01-evidence/reference.sha256`.

The five `supplemental_m01.rs` cases are supplemental contracts, not upstream registrations. The independent reference builder is `python3 tests/reference/run_m01.py --source /Volumes/Data/Users/paul/development/src/github/skia`. It compiles original registrations/macros and production bodies, checks the exact enabled registry, and executes debug/release outside upstream.

## M02 validation (not accepted)

See [M02 acceptance evidence](../plan/acceptance/M02.md) and its 111-row correspondence ledger for source hashes, adaptations and both profile results. All 110 enabled originals now execute and pass in Rust and independently built unchanged C++ in debug and release. The original PartitionAlloc/dangling-check registration remains NOT_APPLICABLE. The final three translations preserve both executor registrations' three factory variants and the complete sk_sp fixture/counter/self-reset/cycle closure. Eleven supplemental runtime checks include one failure: projected base-pointer conversion. A separate compile-negative sized erasure contract also fails. M02 remains NEEDS_FIX; M01 remains ACCEPTED and M03 remains unopened.
