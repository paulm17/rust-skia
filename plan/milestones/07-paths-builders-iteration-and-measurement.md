# Milestone 07: Paths, builders, iteration and measurement

## Outcome and entry gate

Implement path storage, verbs, conics, fill types, builders, bounds, raw paths, interpolation, iteration and contour/path measurement. Preserve copy-on-write observables, generation IDs and error handling. Provide the path portion of serialization using existing byte primitives.

Begin implementation only after milestone 06 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement path storage, verbs, conics, fill types, builders, bounds, raw paths, interpolation, iteration and contour/path measurement. Preserve copy-on-write observables, generation IDs and error handling. Provide the path portion of serialization using existing byte primitives.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkArc.h` — port to Rust.
- `include/core/SkContourMeasure.h` — port to Rust.
- `include/core/SkPath.h` — port to Rust.
- `include/core/SkPathBuilder.h` — port to Rust.
- `include/core/SkPathIter.h` — port to Rust.
- `include/core/SkPathMeasure.h` — port to Rust.
- `include/core/SkPathTypes.h` — port to Rust.
- `include/core/SkPathUtils.h` — port to Rust.
- `include/private/SkPathRef.h` — port to Rust.
- `include/utils/SkParsePath.h` — port to Rust.
- `src/core/SkContourMeasure.cpp` — port to Rust.
- `src/core/SkPath.cpp` — port to Rust.
- `src/core/SkPathBuilder.cpp` — port to Rust.
- `src/core/SkPathData.cpp` — port to Rust.
- `src/core/SkPathData.h` — port to Rust.
- `src/core/SkPathDump.cpp` — port to Rust.
- `src/core/SkPathEnums.h` — port to Rust.
- `src/core/SkPathIter.cpp` — port to Rust.
- `src/core/SkPathMakers.h` — port to Rust.
- `src/core/SkPathMeasure.cpp` — port to Rust.
- `src/core/SkPathMeasurePriv.h` — port to Rust.
- `src/core/SkPathPriv.cpp` — port to Rust.
- `src/core/SkPathPriv.h` — port to Rust.
- `src/core/SkPathRaw.cpp` — port to Rust.
- `src/core/SkPathRaw.h` — port to Rust.
- `src/core/SkPathRawShapes.cpp` — port to Rust.
- `src/core/SkPathRawShapes.h` — port to Rust.
- `src/core/SkPathUtils.cpp` — port to Rust.
- `src/core/SkPath_interpolate.cpp` — port to Rust.
- `src/core/SkPath_serial.cpp` — port to Rust.
- `src/core/SkRegion_path.cpp` — port to Rust.
- `src/core/SkScan_AAAPath.cpp` — port to Rust.
- `src/core/SkScan_AntiPath.cpp` — port to Rust.
- `src/core/SkScan_Path.cpp` — port to Rust.
- `src/utils/SkGetExecutablePath.h` — port to Rust.
- `src/utils/SkGetExecutablePath_mac.cpp` — port to Rust.
- `src/utils/SkParsePath.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate path and path-builder tests, path measurement, raw shapes and Rect_path_nan. Preserve verb streams, closure behavior, cached bounds, non-finite points and iterator results. Rendering assertions remain scheduled with the renderer; every registration must be accounted for.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/IsClosedSingleContourTest.cpp`.
- `tests/M44Test.cpp`.
- `tests/OSPathTest.cpp`.
- `tests/ParsePathTest.cpp`.
- `tests/PathBuilderTest.cpp`.
- `tests/PathCoverageTest.cpp`.
- `tests/PathDataTest.cpp`.
- `tests/PathMeasureTest.cpp`.
- `tests/PathRawShapesTest.cpp`.
- `tests/PathRawTest.cpp`.
- `tests/RRectInPathTest.cpp`.
- `tests/RectTest.cpp`.
- `tests/RoundRectTest.cpp`.
- `tests/SkPathRangeIterTest.cpp`.
- `tests/SubsetPath.cpp`.
- `tests/SubsetPath.h`.

### Original test registrations assigned to this gate

- `IsClosedSingleContourTest` — `tests/IsClosedSingleContourTest.cpp:11`; CPU; `DEF_TEST`.
- `M44_mapRect` — `tests/M44Test.cpp:262`; CPU; `DEF_TEST`.
- `OSPath` — `tests/OSPathTest.cpp:69`; CPU; `DEF_TEST`.
- `ParsePath` — `tests/ParsePathTest.cpp:49`; CPU; `DEF_TEST`.
- `ParsePath_InvalidDoesNotCrash` — `tests/ParsePathTest.cpp:78`; CPU; `DEF_TEST`.
- `ParsePathOptionalCommand` — `tests/ParsePathTest.cpp:91`; CPU; `DEF_TEST`.
- `ParsePathArcFlags` — `tests/ParsePathTest.cpp:132`; CPU; `DEF_TEST`.
- `pathbuilder` — `tests/PathBuilderTest.cpp:34`; CPU; `DEF_TEST`.
- `pathbuilder_filltype` — `tests/PathBuilderTest.cpp:59`; CPU; `DEF_TEST`.
- `pathbuilder_missing_move` — `tests/PathBuilderTest.cpp:112`; CPU; `DEF_TEST`.
- `pathbuilder_addRect` — `tests/PathBuilderTest.cpp:129`; CPU; `DEF_TEST`.
- `pathbuilder_addOval` — `tests/PathBuilderTest.cpp:203`; CPU; `DEF_TEST`.
- `pathbuilder_addRRect` — `tests/PathBuilderTest.cpp:231`; CPU; `DEF_TEST`.
- `pathbuilder_make` — `tests/PathBuilderTest.cpp:256`; CPU; `DEF_TEST`.
- `pathbuilder_genid` — `tests/PathBuilderTest.cpp:276`; CPU; `DEF_TEST`.
- `pathbuilder_addPolygon` — `tests/PathBuilderTest.cpp:288`; CPU; `DEF_TEST`.
- `pathbuilder_addPath` — `tests/PathBuilderTest.cpp:515`; CPU; `DEF_TEST`.
- `pathbuilder_addpath_crbug_1153516` — `tests/PathBuilderTest.cpp:541`; CPU; `DEF_TEST`.
- `SkPathBuilder_multipleMoveTos` — `tests/PathBuilderTest.cpp:592`; CPU; `DEF_TEST`.
- `SkPathBuilder_lineToMoveTo` — `tests/PathBuilderTest.cpp:615`; CPU; `DEF_TEST`.
- `SkPathBuilder_arcToPtPtRad_invalidInputsResultInALine` — `tests/PathBuilderTest.cpp:634`; CPU; `DEF_TEST`.
- `SkPathBuilder_assign` — `tests/PathBuilderTest.cpp:659`; CPU; `DEF_TEST`.
- `SkPathBuilder_getLastPt` — `tests/PathBuilderTest.cpp:679`; CPU; `DEF_TEST`.
- `SkPathBuilder_transform` — `tests/PathBuilderTest.cpp:691`; CPU; `DEF_TEST`.
- `SkPathBuilder_cleaning` — `tests/PathBuilderTest.cpp:793`; CPU; `DEF_TEST`.
- `SkPathBuilder_path_roundtrip` — `tests/PathBuilderTest.cpp:810`; CPU; `DEF_TEST`.
- `SkPathBuilder_rMoveTo` — `tests/PathBuilderTest.cpp:908`; CPU; `DEF_TEST`.
- `SkPathBuilder_equality` — `tests/PathBuilderTest.cpp:947`; CPU; `DEF_TEST`.
- `SkPathBuilder_dump` — `tests/PathBuilderTest.cpp:998`; CPU; `DEF_TEST`.
- `SkPathBuilder_trailingmove_addpath` — `tests/PathBuilderTest.cpp:1025`; CPU; `DEF_TEST`.
- `SkPathBuilder_b_463584612` — `tests/PathBuilderTest.cpp:1063`; CPU; `DEF_TEST`.
- `SkPathBuilder_conic_semantics` — `tests/PathBuilderTest.cpp:1081`; CPU; `DEF_TEST`.
- `SkPathBuilder_b520944501` — `tests/PathBuilderTest.cpp:1127`; CPU; `DEF_TEST`.
- `PathCoverage` — `tests/PathCoverageTest.cpp:165`; CPU; `DEF_TEST`.
- `pathdata_empty` — `tests/PathDataTest.cpp:32`; CPU; `DEF_TEST`.
- `pathdata_rect` — `tests/PathDataTest.cpp:121`; CPU; `DEF_TEST`.
- `pathdata_polygon` — `tests/PathDataTest.cpp:176`; CPU; `DEF_TEST`.
- `pathdata_oval` — `tests/PathDataTest.cpp:233`; CPU; `DEF_TEST`.
- `pathdata_rrect` — `tests/PathDataTest.cpp:267`; CPU; `DEF_TEST`.
- `pathdata_make_edgecases` — `tests/PathDataTest.cpp:293`; CPU; `DEF_TEST`.
- `pathdata_make_nonfinite` — `tests/PathDataTest.cpp:375`; CPU; `DEF_TEST`.
- `pathdata_transform` — `tests/PathDataTest.cpp:417`; CPU; `DEF_TEST`.
- `pathdata_transform_convexity` — `tests/PathDataTest.cpp:458`; CPU; `DEF_TEST`.
- `pathdata_inverted_bounds` — `tests/PathDataTest.cpp:509`; CPU; `DEF_TEST`.
- `PathMeasure` — `tests/PathMeasureTest.cpp:107`; CPU; `DEF_TEST`.
- `PathMeasureConic` — `tests/PathMeasureTest.cpp:236`; CPU; `DEF_TEST`.
- `PathMeasure_nextctr` — `tests/PathMeasureTest.cpp:255`; CPU; `DEF_TEST`.
- `contour_measure` — `tests/PathMeasureTest.cpp:310`; CPU; `DEF_TEST`.
- `contour_measure_verbs` — `tests/PathMeasureTest.cpp:338`; CPU; `DEF_TEST`.
- `pathrawshapes_rect` — `tests/PathRawShapesTest.cpp:47`; CPU; `DEF_TEST`.
- `pathrawshapes_oval` — `tests/PathRawShapesTest.cpp:63`; CPU; `DEF_TEST`.
- `pathrawshapes_rrect` — `tests/PathRawShapesTest.cpp:79`; CPU; `DEF_TEST`.
- `pathraw_iter` — `tests/PathRawTest.cpp:171`; CPU; `DEF_TEST`.
- `pathraw_segmentmask` — `tests/PathRawTest.cpp:231`; CPU; `DEF_TEST`.
- `RoundRectInPath` — `tests/RRectInPathTest.cpp:380`; CPU; `DEF_TEST`.
- `RRect_fragile` — `tests/RRectInPathTest.cpp:393`; CPU; `DEF_TEST`.
- `Rect_path_nan` — `tests/RectTest.cpp:113`; CPU; `DEF_TEST`.
- `RRect_b561770646_part1` — `tests/RoundRectTest.cpp:1786`; CPU; `DEF_TEST`.
- `RRect_b561770646_part2` — `tests/RoundRectTest.cpp:1805`; CPU; `DEF_TEST`.
- `SkPath_RangeIter` — `tests/SkPathRangeIterTest.cpp:17`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/bentleyottmann/tests/ContourTest.cpp` — M42.
- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/sksg/tests/SGTest.cpp` — M29.
- `tests/AAClipTest.cpp` — M14.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/CPUBackendTest.cpp` — M15.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/CappedHairlinesTest.cpp` — M14.
- `tests/ClipCubicTest.cpp` — M14.
- `tests/ClipStackTest.cpp` — M09.
- `tests/DashPathEffectTest.cpp` — M14.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DirectMaskLimitTest.cpp` — M38.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawPathTest.cpp` — M14.
- `tests/EdgeTest.cpp` — M14.
- `tests/EmptyPathTest.cpp` — M14.
- `tests/FillPathTest.cpp` — M14.
- `tests/FontationsFtCompTest.cpp` — M20.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/PathOpsAsWindingTest.cpp` — M23.
- `tests/PathOpsBattles.cpp` — M23.
- `tests/PathOpsBuildUseTest.cpp` — M23.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathOpsBuilderTest.cpp` — M23.
- `tests/PathOpsChalkboardTest.cpp` — M23.
- `tests/PathOpsConicLineIntersectionTest.cpp` — M23.
- `tests/PathOpsFuzz763Test.cpp` — M23.
- `tests/PathOpsInverseTest.cpp` — M23.
- `tests/PathOpsIssue3651.cpp` — M23.
- `tests/PathOpsOpCircleThreadedTest.cpp` — M23.
- `tests/PathOpsOpCubicThreadedTest.cpp` — M23.
- `tests/PathOpsOpLoopThreadedTest.cpp` — M23.
- `tests/PathOpsOpRectThreadedTest.cpp` — M23.
- `tests/PathOpsOpTest.cpp` — M23.
- `tests/PathOpsSimplifyDegenerateThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyFailTest.cpp` — M23.
- `tests/PathOpsSimplifyQuadThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyQuadralateralsThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyRectThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyTest.cpp` — M23.
- `tests/PathOpsSimplifyTrianglesThreadedTest.cpp` — M23.
- `tests/PathOpsSkpTest.cpp` — M23.
- `tests/PathOpsTigerTest.cpp` — M23.
- `tests/PathOpsTightBoundsTest.cpp` — M23.
- `tests/PathRendererCacheTests.cpp` — M32.
- `tests/PathTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/PreChopPathCurvesTest.cpp` — M31.
- `tests/RectTest.cpp` — M14.
- `tests/RegionTest.cpp` — M14.
- `tests/RoundRectTest.cpp` — M23.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShadowTest.cpp` — M15.
- `tests/SimplifyPaintTest.cpp` — M35, M38.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/SmallPathRendererTest.cpp` — M32, M38.
- `tests/StencilClearTest.cpp` — M38.
- `tests/StrokeTest.cpp` — M13.
- `tests/StrokerTest.cpp` — M23.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/graphite/MultisampleTest.cpp` — M38.
- `tests/graphite/sparse_strips/CoverageTest.cpp` — M38.
- `tests/graphite/sparse_strips/FlattenTest.cpp` — M35.
- `tests/graphite/sparse_strips/OracleTest.cpp` — M35.
- `tests/graphite/sparse_strips/PolylineTest.cpp` — M35.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 30 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=7`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M07 ACCEPTED may the user hand Deepseek M08.
