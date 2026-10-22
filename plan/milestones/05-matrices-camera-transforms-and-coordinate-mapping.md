# Milestone 05: Matrices, camera transforms and coordinate mapping

## Outcome and entry gate

Implement Matrix, M44, RSXform, matrix inversion and coordinate utilities. Preserve multiplication order, type masks, perspective normalization, singular handling and mapped-rectangle behavior. Expose stable operations used by paths and shaders.

Begin implementation only after milestone 04 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement Matrix, M44, RSXform, matrix inversion and coordinate utilities. Preserve multiplication order, type masks, perspective normalization, singular handling and mapped-rectangle behavior. Expose stable operations used by paths and shaders.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkM44.h` — port to Rust.
- `include/core/SkMatrix.h` — port to Rust.
- `include/core/SkRSXform.h` — port to Rust.
- `include/utils/SkCamera.h` — port to Rust.
- `src/core/SkBitmapProcState_matrixProcs.cpp` — port to Rust.
- `src/core/SkLocalMatrixImageFilter.cpp` — port to Rust.
- `src/core/SkLocalMatrixImageFilter.h` — port to Rust.
- `src/core/SkM44.cpp` — port to Rust.
- `src/core/SkMatrix.cpp` — port to Rust.
- `src/core/SkMatrixInvert.cpp` — port to Rust.
- `src/core/SkMatrixInvert.h` — port to Rust.
- `src/core/SkMatrixPriv.h` — port to Rust.
- `src/core/SkMatrixUtils.h` — port to Rust.
- `src/core/SkRSXform.cpp` — port to Rust.
- `src/utils/SkCamera.cpp` — port to Rust.
- `src/utils/SkMatrix22.cpp` — port to Rust.
- `src/utils/SkMatrix22.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate matrix and transform tests, including Rect_QuadContainsRect. Preserve exact versus approximate comparisons and random input generation. Exercise identity, affine, perspective, near-singular, infinite and NaN matrices with original tolerances.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/M44Test.cpp`.
- `tests/MatrixTest.cpp`.
- `tests/RectTest.cpp`.

### Original test registrations assigned to this gate

- `M44` — `tests/M44Test.cpp:33`; CPU; `DEF_TEST`.
- `M44_v3` — `tests/M44Test.cpp:79`; CPU; `DEF_TEST`.
- `M44_v4` — `tests/M44Test.cpp:103`; CPU; `DEF_TEST`.
- `M44_rotate` — `tests/M44Test.cpp:144`; CPU; `DEF_TEST`.
- `M44_rectToRect` — `tests/M44Test.cpp:198`; CPU; `DEF_TEST`.
- `M44_mapRect_skbug12335` — `tests/M44Test.cpp:374`; CPU; `DEF_TEST`.
- `Matrix` — `tests/MatrixTest.cpp:868`; CPU; `DEF_TEST`.
- `Matrix_Concat` — `tests/MatrixTest.cpp:1036`; CPU; `DEF_TEST`.
- `Matrix_maprects` — `tests/MatrixTest.cpp:1050`; CPU; `DEF_TEST`.
- `Matrix_mapRect_skbug12335` — `tests/MatrixTest.cpp:1090`; CPU; `DEF_TEST`.
- `Matrix_Ctor` — `tests/MatrixTest.cpp:1104`; CPU; `DEF_TEST`.
- `Matrix_LookAt` — `tests/MatrixTest.cpp:1108`; CPU; `DEF_TEST`.
- `Matrix_SetRotateSnap` — `tests/MatrixTest.cpp:1114`; CPU; `DEF_TEST`.
- `Matrix_rectStaysRect_zeroScale` — `tests/MatrixTest.cpp:1133`; CPU; `DEF_TEST`.
- `Rect_QuadContainsRect` — `tests/RectTest.cpp:281`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/AAClipTest.cpp` — M14.
- `tests/BackendAllocationTest.cpp` — M32, M34.
- `tests/BigImageTest.cpp` — M38.
- `tests/BulkRectTest.cpp` — M38.
- `tests/CPUBackendTest.cpp` — M15.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/ClipStackTest.cpp` — M09.
- `tests/CodecAnimTest.cpp` — M29.
- `tests/CullTestTest.cpp` — M31.
- `tests/DMSAATest.cpp` — M38.
- `tests/DashPathEffectTest.cpp` — M14.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/DrawPathTest.cpp` — M14.
- `tests/DrawTextTest.cpp` — M18.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/GeometryTest.cpp` — M06.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrMipMappedTest.cpp` — M32, M38.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/GrSurfaceResolveTest.cpp` — M32, M38.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageGeneratorOrientationTest.cpp` — M16.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/M44Test.cpp` — M07.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathOpsSimplifyTest.cpp` — M23.
- `tests/PathRendererCacheTests.cpp` — M32.
- `tests/PathTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/PinnedImageTest.cpp` — M38.
- `tests/PreChopPathCurvesTest.cpp` — M31.
- `tests/QuickRejectTest.cpp` — M15.
- `tests/RRectInPathTest.cpp` — M07.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RectTest.cpp` — M07, M14.
- `tests/RectangleTextureTest.cpp` — M34.
- `tests/RegionTest.cpp` — M14.
- `tests/RoundRectTest.cpp` — M07, M23.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/ShadowTest.cpp` — M15.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkResourceCacheTest.cpp` — M22.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkStrikeCacheTest.cpp` — M22.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/SlugTest.cpp` — M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureOpTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/WangsFormulaTest.cpp` — M31.
- `tests/graphite/sparse_strips/CoverageTest.cpp` — M38.
- `tests/graphite/sparse_strips/FlattenTest.cpp` — M35.
- `tests/graphite/sparse_strips/PolylineTest.cpp` — M35.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 15 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=5`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

Interfaces with later consumers may be declared now, but the accepted feature set must execute real implementations for every reached call. Where a source spans several features (CPU/GPU, serialization, runtime shaders), record completed and pending symbols separately. Deferred portions have explicit integration gates M14, M22, M26, M38 and M47; no pending portion may disappear from the ledger.

## Commands, required evidence and stop condition

Deepseek runs the milestone tests and all previously accepted tests in debug and release with the exact feature list from the handoff. The standard baseline commands are:

```sh
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

For feature-specific crates/backends, record the additional exact package/target/features commands; do not use an unexamined `--all-features` combination. Native backend, GM, browser, fuzz and benchmark commands are added when their harnesses exist and must be copied into the acceptance report.

Acceptance requires: all assigned macOS cases enumerated; every applicable assertion passing; all earlier accepted tests passing; every source row mapped to implemented Rust or a documented non-code disposition; no unexplained missing resources, untested required backends or unsupported required API; and a Deepseek-authored evidence report. Formatting/build success alone never constitutes acceptance.

Record PASS, FAIL, BLOCKED or NOT_APPLICABLE per test/configuration. NOT_APPLICABLE requires an original platform/capability condition and a reason. All milestones require Apple Silicon macOS evidence only. Intel/x86_64 implementation, builds and execution are excluded by user direction, including M46/M47. Intel-specific inventory paths are reference accounting only, not pending work or acceptance blockers.

Only after Deepseek runs the tests, completes the audit and marks M05 ACCEPTED may the user hand Deepseek M06.
