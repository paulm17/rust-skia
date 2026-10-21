# Milestone 09: Regions, clip storage and spatial indexes

## Outcome and entry gate

Implement integer regions, clip stacks, raster clip state, RTree/BBH and supporting edge/clip data structures. Separate clip representation from the later canvas execution. Preserve save/restore, inversion and intersection semantics.

Begin implementation only after milestone 08 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement integer regions, clip stacks, raster clip state, RTree/BBH and supporting edge/clip data structures. Separate clip representation from the later canvas execution. Preserve save/restore, inversion and intersection semantics.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkBBHFactory.h` — port to Rust.
- `include/core/SkClipOp.h` — port to Rust.
- `include/core/SkRegion.h` — port to Rust.
- `src/core/SkAAClip.cpp` — port to Rust.
- `src/core/SkAAClip.h` — port to Rust.
- `src/core/SkBBHFactory.cpp` — port to Rust.
- `src/core/SkClipStack.cpp` — port to Rust.
- `src/core/SkClipStack.h` — port to Rust.
- `src/core/SkClipStackDevice.cpp` — port to Rust.
- `src/core/SkClipStackDevice.h` — port to Rust.
- `src/core/SkRTree.cpp` — port to Rust.
- `src/core/SkRTree.h` — port to Rust.
- `src/core/SkRasterClip.cpp` — port to Rust.
- `src/core/SkRasterClip.h` — port to Rust.
- `src/core/SkRasterClipStack.h` — port to Rust.
- `src/core/SkRegion.cpp` — port to Rust.
- `src/core/SkRegionPriv.h` — port to Rust.
- `src/utils/SkClipStackUtils.cpp` — port to Rust.
- `src/utils/SkClipStackUtils.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: acceptance-test contract

Translate region and spatial-index tests; exercise empty, large and overflowing bounds and original randomized operations. Clip tests that call Canvas are assigned to M14 or M15. Retain all original operation sequences and expected region contents.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/ClipStackTest.cpp`.
- `tests/RTreeTest.cpp`.

### Original test registrations assigned to this gate

- `ClipStack` — `tests/ClipStackTest.cpp:828`; CPU; `DEF_TEST`.
- `RTree` — `tests/RTreeTest.cpp:65`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/AAClipTest.cpp` — M14.
- `tests/CanvasStateTest.cpp` — M17.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathTest.cpp` — M22.
- `tests/PictureBBHTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordingXfermodeTest.cpp` — M22.
- `tests/RegionTest.cpp` — M14.
- `tests/SerializationTest.cpp` — M22.
- `tests/SurfaceTest.cpp` — M32, M34, M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 15 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=9`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M09 ACCEPTED may the user hand Deepseek M10.
