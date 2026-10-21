# Milestone 06: Bezier geometry, roots and clipping primitives

## Outcome and entry gate

Implement quadratic/cubic/conic geometry, root solving, chopping, curve clipping and polygon math independently of drawing. Preserve root ordering, parameter intervals, degenerate cases and chosen floating-point precision.

Begin implementation only after milestone 05 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement quadratic/cubic/conic geometry, root solving, chopping, curve clipping and polygon math independently of drawing. Preserve root ordering, parameter intervals, degenerate cases and chosen floating-point precision.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkCubicMap.h` — port to Rust.
- `src/core/SkBezierCurves.cpp` — port to Rust.
- `src/core/SkBezierCurves.h` — port to Rust.
- `src/core/SkCubicClipper.cpp` — port to Rust.
- `src/core/SkCubicClipper.h` — port to Rust.
- `src/core/SkCubicMap.cpp` — port to Rust.
- `src/core/SkCubics.cpp` — port to Rust.
- `src/core/SkCubics.h` — port to Rust.
- `src/core/SkEdgeClipper.cpp` — port to Rust.
- `src/core/SkEdgeClipper.h` — port to Rust.
- `src/core/SkGeometry.cpp` — port to Rust.
- `src/core/SkGeometry.h` — port to Rust.
- `src/core/SkLineClipper.cpp` — port to Rust.
- `src/core/SkLineClipper.h` — port to Rust.
- `src/core/SkQuadClipper.cpp` — port to Rust.
- `src/core/SkQuadClipper.h` — port to Rust.
- `src/core/SkQuads.cpp` — port to Rust.
- `src/core/SkQuads.h` — port to Rust.
- `src/utils/SkPolyUtils.cpp` — port to Rust.
- `src/utils/SkPolyUtils.h` — port to Rust.
- `src/utils/mac/SkCGGeometry.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: acceptance-test contract

Translate curve, root, geometry, clipping and polygon utility registrations. Keep regression inputs verbatim and compare return counts as well as coordinates. Tests that draw curves receive a later rendering gate rather than simplified replacements.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/BezierCurveTest.cpp`.
- `tests/CubicMapTest.cpp`.
- `tests/GeometryTest.cpp`.
- `tests/InsetConvexPolyTest.cpp`.
- `tests/OffsetSimplePolyTest.cpp`.
- `tests/PolyUtilsTest.cpp`.

### Original test registrations assigned to this gate

- `BezierCubicEvalAt` — `tests/BezierCurveTest.cpp:52`; CPU; `DEF_TEST`.
- `BezierCubicToPolynomials` — `tests/BezierCurveTest.cpp:107`; CPU; `DEF_TEST`.
- `QuadRoots_CheckTRange` — `tests/BezierCurveTest.cpp:121`; CPU; `DEF_TEST`.
- `SkBezierCubic_CheckTRange` — `tests/BezierCurveTest.cpp:201`; CPU; `DEF_TEST`.
- `CubicMap` — `tests/CubicMapTest.cpp:63`; CPU; `DEF_TEST`.
- `Geometry` — `tests/GeometryTest.cpp:632`; CPU; `DEF_TEST`.
- `GeometryChopMonoCubicAtY_Successful` — `tests/GeometryTest.cpp:692`; CPU; `DEF_TEST`.
- `GeometryChopMonoCubicAtY_OutOfRangeReturnFalse` — `tests/GeometryTest.cpp:818`; CPU; `DEF_TEST`.
- `GeometryChopMonoCubicAtX_Successful` — `tests/GeometryTest.cpp:851`; CPU; `DEF_TEST`.
- `GeometryChopMonoCubicAtX_OutOfRangeReturnFalse` — `tests/GeometryTest.cpp:968`; CPU; `DEF_TEST`.
- `ConicsWithCrazyW` — `tests/GeometryTest.cpp:978`; CPU; `DEF_TEST`.
- `InsetConvexPoly` — `tests/InsetConvexPolyTest.cpp:14`; CPU; `DEF_TEST`.
- `OffsetSimplePoly` — `tests/OffsetSimplePolyTest.cpp:15`; CPU; `DEF_TEST`.
- `PolyUtils` — `tests/PolyUtilsTest.cpp:17`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/ClipCubicTest.cpp` — M14.
- `tests/ClipperTest.cpp` — M14.
- `tests/CubicChopTest.cpp` — M23.
- `tests/CubicRootsTest.cpp` — M23.
- `tests/EdgeTest.cpp` — M14.
- `tests/FindCubicConvex180ChopsTest.cpp` — M31.
- `tests/PathOpsConicIntersectionTest.cpp` — M23.
- `tests/PathOpsConicLineIntersectionTest.cpp` — M23.
- `tests/PathOpsCubicIntersectionTest.cpp` — M23.
- `tests/PathOpsOpTest.cpp` — M23.
- `tests/PathTest.cpp` — M22.
- `tests/QuadRootsTest.cpp` — M23.
- `tests/WangsFormulaTest.cpp` — M31.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 4 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=6`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

Interfaces with later consumers may be declared now, but the accepted feature set must execute real implementations for every reached call. Where a source spans several features (CPU/GPU, serialization, runtime shaders), record completed and pending symbols separately. Deferred portions have explicit integration gates M14, M22, M26, M38 and M47; no pending portion may disappear from the ledger.

## Commands, required evidence and stop condition

Codex runs the milestone tests and all previously accepted tests in debug and release with the exact feature list from the handoff. The standard baseline commands are:

```sh
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

For feature-specific crates/backends, record the additional exact package/target/features commands; do not use an unexamined `--all-features` combination. Native backend, GM, browser, fuzz and benchmark commands are added when their harnesses exist and must be copied into the acceptance report.

Acceptance requires: all assigned macOS cases enumerated; every applicable assertion passing; all earlier accepted tests passing; every source row mapped to implemented Rust or a documented non-code disposition; no unexplained missing resources, untested required backends or unsupported required API; and a Codex-authored evidence report. Formatting/build success alone never constitutes acceptance.

Record PASS, FAIL, BLOCKED or NOT_APPLICABLE per test/configuration. NOT_APPLICABLE requires an original platform/capability condition and a reason. M01–M45 require native Apple Silicon macOS evidence; Intel-specific files may remain explicitly pending until M46, whose gate requires native Intel Mac results. This is a scheduled architecture gate, not an assertion that Intel already passed.

Only after Codex marks M06 ACCEPTED may the user hand Deepseek M07.
