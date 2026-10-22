# Milestone 01: Scalar and point baseline

Status: **ACCEPTED** — 2026-10-21. [Codex acceptance evidence](../acceptance/M01.md).

## Outcome and entry gate

Retain the implemented scalar, floating-point, float-bit and point APIs. Reconcile every declaration with the pinned sources; keep the minimal rectangle bounds support required by Scalar. Deliver an API map for existing names rather than rewriting passing code. This is the first acceptance checkpoint and preserves the existing three tests.

This baseline has prior local arm64 macOS test evidence. Reconcile it with the pinned source and record the acceptance report; do not discard the existing tests.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Retain the implemented scalar, floating-point, float-bit and point APIs. Reconcile every declaration with the pinned sources; keep the minimal rectangle bounds support required by Scalar. Deliver an API map for existing names rather than rewriting passing code. This is the first acceptance checkpoint and preserves the existing three tests.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkPoint.h` — port to Rust.
- `include/core/SkScalar.h` — port to Rust.
- `include/private/SkFloatingPoint.h` — port to Rust.
- `src/core/SkFloatBits.h` — port to Rust.
- `src/core/SkFloatingPoint.cpp` — port to Rust.
- `src/core/SkPoint.cpp` — port to Rust.
- `src/core/SkPointPriv.h` — port to Rust.
- `src/core/SkScalar.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Run Scalar, Point and Point_setLengthFast and reporter self-tests in debug and release. Preserve finite classification Cartesian products, rounding regressions, pointer identity, overflow normalization and fast-path tolerances. Prior local passage is evidence, not evidence for unrelated APIs.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/CtsEnforcement.cpp`.
- `tests/PointTest.cpp`.
- `tests/ScalarTest.cpp`.
- `tests/TestHarness.cpp`.
- `tests/TestHarness.h`.
- `tests/TestTest.cpp`.
- `tests/TestType.h`.
- `tools/Registry.h`.

### Original test registrations assigned to this gate

- `Point` — `tests/PointTest.cpp:133`; CPU; `DEF_TEST`.
- `Point_setLengthFast` — `tests/PointTest.cpp:154`; CPU; `DEF_TEST`.
- `Scalar` — `tests/ScalarTest.cpp:212`; CPU; `DEF_TEST`.
- `TestNormal` — `tests/TestTest.cpp:15`; CPU; `DEF_TEST`.
- `TestTrueCondition` — `tests/TestTest.cpp:20`; CPU; `DEF_CONDITIONAL_TEST`.
- `TestFalseCondition` — `tests/TestTest.cpp:25`; CPU; `DEF_CONDITIONAL_TEST`.
- `TestCtsEnforcement` — `tests/TestTest.cpp:97`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/skshaper/tests/ShaperTest.cpp` — M20.
- `tests/AAClipTest.cpp` — M14.
- `tests/AnnotationTest.cpp` — M27.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/AsADashTest.cpp` — M08.
- `tests/BezierCurveTest.cpp` — M06.
- `tests/BigImageTest.cpp` — M38.
- `tests/BitmapCopyTest.cpp` — M12.
- `tests/BitmapTest.cpp` — M11.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/CanvasStateTest.cpp` — M17.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/ClipCubicTest.cpp` — M14.
- `tests/ClipStackTest.cpp` — M09.
- `tests/ClipperTest.cpp` — M14.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/ColorSpaceTest.cpp` — M16.
- `tests/CopySurfaceTest.cpp` — M38.
- `tests/CubicChopTest.cpp` — M23.
- `tests/CubicMapTest.cpp` — M06.
- `tests/CubicRootsTest.cpp` — M23.
- `tests/CullTestTest.cpp` — M31.
- `tests/DMSAATest.cpp` — M38.
- `tests/DashPathEffectTest.cpp` — M14.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawPathTest.cpp` — M14.
- `tests/DrawTextTest.cpp` — M18.
- `tests/EdgeTest.cpp` — M14.
- `tests/EmptyPathTest.cpp` — M14.
- `tests/FillPathTest.cpp` — M14.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FindCubicConvex180ChopsTest.cpp` — M31.
- `tests/FloatingPointTest.cpp` — M03.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/FontHostTest.cpp` — M19.
- `tests/GLBackendSurfaceTest.cpp` — M34.
- `tests/GeometryTest.cpp` — M06.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrGpuBufferTest.cpp` — M38.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/HSVRoundTripTest.cpp` — M10.
- `tests/HdrMetadataTest.cpp` — M16.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/InfRectTest.cpp` — M14.
- `tests/InsetConvexPolyTest.cpp` — M06.
- `tests/JSONTest.cpp` — M42.
- `tests/M44Test.cpp` — M05, M07.
- `tests/MaskCacheTest.cpp` — M15.
- `tests/MathTest.cpp` — M03.
- `tests/MatrixTest.cpp` — M05.
- `tests/MetaDataTest.cpp` — M02.
- `tests/MipMapTest.cpp` — M14.
- `tests/OffsetSimplePolyTest.cpp` — M06.
- `tests/OpChainTest.cpp` — M32.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/ParsePathTest.cpp` — M07.
- `tests/PathBuilderTest.cpp` — M07.
- `tests/PathCoverageTest.cpp` — M07.
- `tests/PathMeasureTest.cpp` — M07.
- `tests/PathOpsAngleIdeas.cpp` — M23.
- `tests/PathOpsAngleTest.cpp` — M23.
- `tests/PathOpsAsWindingTest.cpp` — M23.
- `tests/PathOpsBattles.cpp` — M23.
- `tests/PathOpsBoundsTest.cpp` — M23.
- `tests/PathOpsBuildUseTest.cpp` — M23.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathOpsBuilderTest.cpp` — M23.
- `tests/PathOpsChalkboardTest.cpp` — M23.
- `tests/PathOpsConicIntersectionTest.cpp` — M23.
- `tests/PathOpsConicLineIntersectionTest.cpp` — M23.
- `tests/PathOpsCubicIntersectionTest.cpp` — M23.
- `tests/PathOpsDCubicTest.cpp` — M23.
- `tests/PathOpsDLineTest.cpp` — M23.
- `tests/PathOpsDPointTest.cpp` — M23.
- `tests/PathOpsDVectorTest.cpp` — M23.
- `tests/PathOpsFuzz763Test.cpp` — M23.
- `tests/PathOpsIssue3651.cpp` — M23.
- `tests/PathOpsOpCircleThreadedTest.cpp` — M23.
- `tests/PathOpsOpCubicThreadedTest.cpp` — M23.
- `tests/PathOpsOpLoopThreadedTest.cpp` — M23.
- `tests/PathOpsOpRectThreadedTest.cpp` — M23.
- `tests/PathOpsOpTest.cpp` — M23.
- `tests/PathOpsQuadIntersectionTest.cpp` — M23.
- `tests/PathOpsSimplifyDegenerateThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyFailTest.cpp` — M23.
- `tests/PathOpsSimplifyQuadThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyQuadralateralsThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyRectThreadedTest.cpp` — M23.
- `tests/PathOpsSimplifyTest.cpp` — M23.
- `tests/PathOpsSimplifyTrianglesThreadedTest.cpp` — M23.
- `tests/PathOpsSkpTest.cpp` — M23.
- `tests/PathOpsTigerTest.cpp` — M23.
- `tests/PathRawShapesTest.cpp` — M07.
- `tests/PathRawTest.cpp` — M07.
- `tests/PathTest.cpp` — M22.
- `tests/PictureBBHTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/Point3Test.cpp` — M04.
- `tests/PolyUtilsTest.cpp` — M06.
- `tests/PrimitiveProcessorTest.cpp` — M38.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/QuadRootsTest.cpp` — M23.
- `tests/QuickRejectTest.cpp` — M15.
- `tests/RRectInPathTest.cpp` — M07.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RecorderTest.cpp` — M22.
- `tests/RecordingXfermodeTest.cpp` — M22.
- `tests/RectTest.cpp` — M04, M05, M07, M14.
- `tests/RegionTest.cpp` — M14.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/RoundRectTest.cpp` — M04, M07, M23.
- `tests/SRGBReadWritePixelsTest.cpp` — M38.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/ScaleToSidesTest.cpp` — M04.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderOpacityTest.cpp` — M13.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/ShadowTest.cpp` — M15.
- `tests/SizeTest.cpp` — M04.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkPathRangeIterTest.cpp` — M07.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/SkVxTest.cpp` — M03.
- `tests/SmallPathRendererTest.cpp` — M32, M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/StringTest.cpp` — M02.
- `tests/StrokeTest.cpp` — M13.
- `tests/StrokerTest.cpp` — M23.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TracingTest.cpp` — M11.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/TypefaceTest.cpp` — M19.
- `tests/VerticesTest.cpp` — M22.
- `tests/WangsFormulaTest.cpp` — M31.
- `tests/Writer32Test.cpp` — M22.
- `tests/YUVTest.cpp` — M17.
- `tests/crbug_500172224.cpp` — M38.
- `tests/graphite/RecordingSurfacesTest.cpp` — M38.
- `tests/graphite/StorageContextTest.cpp` — M38.
- `tests/graphite/TextureFormatTest.cpp` — M38.
- `tests/graphite/UniformManagerTest.cpp` — M35.
- `tests/graphite/crbug_513836996.cpp` — M38.
- `tests/graphite/sparse_strips/CoverageTest.cpp` — M38.
- `tests/graphite/sparse_strips/FlattenTest.cpp` — M35.
- `tests/graphite/sparse_strips/LUTTest.cpp` — M35.
- `tests/graphite/sparse_strips/OracleTest.cpp` — M35.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 17 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=1`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M01 ACCEPTED may the user hand Deepseek M02.
