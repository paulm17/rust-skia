# Milestone 15: Image filters, blur, masks and shadows

## Outcome and entry gate

Implement filter graphs, bounds propagation, cache behavior, blur engines, mask filters, special images and shadows. Preserve crop rectangles, filter-space transforms, tile behavior and empty-result semantics. Runtime-effect-backed filters are finished with M26.

Begin implementation only after milestone 14 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement filter graphs, bounds propagation, cache behavior, blur engines, mask filters, special images and shadows. Preserve crop rectangles, filter-space transforms, tile behavior and empty-result semantics. Runtime-effect-backed filters are finished with M26.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkBlurTypes.h` — port to Rust.
- `include/core/SkImageFilter.h` — port to Rust.
- `include/core/SkMaskFilter.h` — port to Rust.
- `include/effects/SkBlurMaskFilter.h` — port to Rust.
- `include/effects/SkImageFilters.h` — port to Rust.
- `include/effects/SkShaderMaskFilter.h` — port to Rust.
- `include/effects/SkTableMaskFilter.h` — port to Rust.
- `include/utils/SkShadowUtils.h` — port to Rust.
- `src/core/SkBlurEngine.cpp` — port to Rust.
- `src/core/SkBlurEngine.h` — port to Rust.
- `src/core/SkBlurMask.cpp` — port to Rust.
- `src/core/SkBlurMask.h` — port to Rust.
- `src/core/SkBlurMaskFilterImpl.cpp` — port to Rust.
- `src/core/SkBlurMaskFilterImpl.h` — port to Rust.
- `src/core/SkDrawShadowInfo.cpp` — port to Rust.
- `src/core/SkDrawShadowInfo.h` — port to Rust.
- `src/core/SkGaussFilter.cpp` — port to Rust.
- `src/core/SkGaussFilter.h` — port to Rust.
- `src/core/SkImageFilter.cpp` — port to Rust.
- `src/core/SkImageFilterCache.cpp` — port to Rust.
- `src/core/SkImageFilterCache.h` — port to Rust.
- `src/core/SkImageFilterTypes.cpp` — port to Rust.
- `src/core/SkImageFilterTypes.h` — port to Rust.
- `src/core/SkImageFilter_Base.h` — port to Rust.
- `src/core/SkMaskBlurFilter.cpp` — port to Rust.
- `src/core/SkMaskBlurFilter.h` — port to Rust.
- `src/core/SkMaskCache.cpp` — port to Rust.
- `src/core/SkMaskCache.h` — port to Rust.
- `src/core/SkMaskFilter.cpp` — port to Rust.
- `src/core/SkMaskFilterBase.cpp` — port to Rust.
- `src/core/SkMaskFilterBase.h` — port to Rust.
- `src/core/SkSpecialImage.cpp` — port to Rust.
- `src/core/SkSpecialImage.h` — port to Rust.
- `src/effects/SkEmbossMask.cpp` — port to Rust.
- `src/effects/SkEmbossMask.h` — port to Rust.
- `src/effects/SkEmbossMaskFilter.cpp` — port to Rust.
- `src/effects/SkEmbossMaskFilter.h` — port to Rust.
- `src/effects/SkShaderMaskFilterImpl.cpp` — port to Rust.
- `src/effects/SkShaderMaskFilterImpl.h` — port to Rust.
- `src/effects/SkTableMaskFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkBlendImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkBlurImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkColorFilterImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkComposeImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkCropImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkDisplacementMapImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkDropShadowImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkImageImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkLightingImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkMagnifierImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkMatrixConvolutionImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkMatrixConvolutionImageFilter.h` — port to Rust.
- `src/effects/imagefilters/SkMatrixTransformImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkMergeImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkMorphologyImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkPictureImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkRuntimeImageFilter.cpp` — port to Rust.
- `src/effects/imagefilters/SkShaderImageFilter.cpp` — port to Rust.
- `src/utils/SkShadowTessellator.cpp` — port to Rust.
- `src/utils/SkShadowTessellator.h` — port to Rust.
- `src/utils/SkShadowUtils.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate image-filter, blur, shadow, cache and special-image tests. Preserve exact crop and bounds regressions and original image fixtures. Compare filter outputs against the C++ CPU reference and rerun affected canvas/clip tests.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/CPUBackendTest.cpp`.
- `tests/MaskCacheTest.cpp`.
- `tests/QuickRejectTest.cpp`.
- `tests/ShadowTest.cpp`.
- `tests/SkGaussFilterTest.cpp`.
- `tests/Skbug6389.cpp`.

### Original test registrations assigned to this gate

- `DrawToMask_RejectsBigPath` — `tests/CPUBackendTest.cpp:21`; CPU; `DEF_TEST`.
- `RRectMaskCache` — `tests/MaskCacheTest.cpp:39`; CPU; `DEF_TEST`.
- `RectsMaskCache` — `tests/MaskCacheTest.cpp:80`; CPU; `DEF_TEST`.
- `QuickReject` — `tests/QuickRejectTest.cpp:102`; CPU; `DEF_TEST`.
- `QuickReject_MatrixState` — `tests/QuickRejectTest.cpp:111`; CPU; `DEF_TEST`.
- `ShadowUtils` — `tests/ShadowTest.cpp:72`; CPU; `DEF_TEST`.
- `ShadowBounds` — `tests/ShadowTest.cpp:213`; CPU; `DEF_TEST`.
- `SkGaussFilterCommon` — `tests/SkGaussFilterTest.cpp:31`; CPU; `DEF_TEST`.
- `SkGaussFilterSweep` — `tests/SkGaussFilterTest.cpp:66`; CPU; `DEF_TEST`.
- `skbug_6389` — `tests/Skbug6389.cpp:20`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/BlurTest.cpp` — M35, M38.
- `tests/CPUContextRecorderTest.cpp` — M22.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/DeviceTest.cpp` — M32, M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FlattenDrawableTest.cpp` — M22.
- `tests/FlattenableFactoryToName.cpp` — M22.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/graphite/NotifyInUseTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 101 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=15`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M15 ACCEPTED may the user hand Deepseek M16.
