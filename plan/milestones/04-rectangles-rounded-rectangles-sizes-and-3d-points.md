# Milestone 04: Rectangles, rounded rectangles, sizes and 3D points

## Outcome and entry gate

Complete Rect, IRect, RRect, sizes, Point3, rectangle helpers and scale-to-sides operations. Preserve empty/inverted conventions, integer overflow rules, bounds failures and radius normalization. M01's rectangle subset must remain compatible.

Begin implementation only after milestone 03 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Complete Rect, IRect, RRect, sizes, Point3, rectangle helpers and scale-to-sides operations. Preserve empty/inverted conventions, integer overflow rules, bounds failures and radius normalization. M01's rectangle subset must remain compatible.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkPoint3.h` — port to Rust.
- `include/core/SkRRect.h` — port to Rust.
- `include/core/SkRect.h` — port to Rust.
- `include/core/SkSize.h` — port to Rust.
- `src/core/SkIPoint16.h` — port to Rust.
- `src/core/SkPoint3.cpp` — port to Rust.
- `src/core/SkRRect.cpp` — port to Rust.
- `src/core/SkRRectPriv.h` — port to Rust.
- `src/core/SkRect.cpp` — port to Rust.
- `src/core/SkRectMemcpy.h` — port to Rust.
- `src/core/SkRectPriv.h` — port to Rust.
- `src/core/SkScaleToSides.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate geometry-only registrations now. RectTest.cpp rendering registrations belong to M14, Rect_path_nan to M07 and Rect_QuadContainsRect to M05; the test-case inventory records these splits. No file may be claimed fully covered while these registrations remain pending.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/Point3Test.cpp`.
- `tests/RectTest.cpp`.
- `tests/RoundRectTest.cpp`.
- `tests/ScaleToSidesTest.cpp`.
- `tests/SizeTest.cpp`.

### Original test registrations assigned to this gate

- `Point3` — `tests/Point3Test.cpp:126`; CPU; `DEF_TEST`.
- `Rect_largest` — `tests/RectTest.cpp:119`; CPU; `DEF_TEST`.
- `Rect_setbounds` — `tests/RectTest.cpp:133`; CPU; `DEF_TEST`.
- `Rect_whOverflow` — `tests/RectTest.cpp:163`; CPU; `DEF_TEST`.
- `Rect_subtract` — `tests/RectTest.cpp:182`; CPU; `DEF_TEST`.
- `Rect_subtract_overflow` — `tests/RectTest.cpp:255`; CPU; `DEF_TEST`.
- `Rect_ClosestDisjointEdge` — `tests/RectTest.cpp:398`; CPU; `DEF_TEST`.
- `RRect_fuzzer_regressions` — `tests/RoundRectTest.cpp:1652`; CPU; `DEF_TEST`.
- `RRect_b511391129` — `tests/RoundRectTest.cpp:1684`; CPU; `DEF_TEST`.
- `RRect_b527765132` — `tests/RoundRectTest.cpp:1714`; CPU; `DEF_TEST`.
- `RRect_b547198215` — `tests/RoundRectTest.cpp:1731`; CPU; `DEF_TEST`.
- `ScaleToSides` — `tests/ScaleToSidesTest.cpp:16`; CPU; `DEF_TEST`.
- `ISize` — `tests/SizeTest.cpp:12`; CPU; `DEF_TEST`.
- `Size` — `tests/SizeTest.cpp:34`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/sksg/tests/SGTest.cpp` — M29.
- `modules/svg/tests/PaintServer.cpp` — M28.
- `tests/AAClipTest.cpp` — M14.
- `tests/AnimatedImageTest.cpp` — M22.
- `tests/AnnotationTest.cpp` — M27.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/BackendAllocationTest.cpp` — M32, M34.
- `tests/BigImageTest.cpp` — M38.
- `tests/BitmapCopyTest.cpp` — M12.
- `tests/BitmapGetColorTest.cpp` — M11.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlitMaskClip.cpp` — M14.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/BulkRectTest.cpp` — M38.
- `tests/CPUContextRecorderTest.cpp` — M22.
- `tests/CanvasStateTest.cpp` — M17.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/ClearTest.cpp` — M38.
- `tests/ClipCubicTest.cpp` — M14.
- `tests/ClipStackTest.cpp` — M09.
- `tests/ClipperTest.cpp` — M14.
- `tests/CodecAnimTest.cpp` — M29.
- `tests/CodecTest.cpp` — M17.
- `tests/CompressedBackendAllocationTest.cpp` — M38.
- `tests/ConvertPixelsTest.cpp` — M14.
- `tests/CopySurfaceTest.cpp` — M38.
- `tests/CullTestTest.cpp` — M31.
- `tests/DMSAATest.cpp` — M38.
- `tests/DashPathEffectTest.cpp` — M14.
- `tests/DebugLayerManagerTest.cpp` — M40.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/DeviceTest.cpp` — M32, M38.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/DrawPathTest.cpp` — M14.
- `tests/DrawTextTest.cpp` — M18.
- `tests/ExifTest.cpp` — M17.
- `tests/FillPathTest.cpp` — M14.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FlattenDrawableTest.cpp` — M22.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/FontHostTest.cpp` — M19.
- `tests/GLFramebufferDeletionTest.cpp` — M34.
- `tests/GainmapShaderTest.cpp` — M16.
- `tests/GifTest.cpp` — M16.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GpuRectanizerTest.cpp` — M32.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrContextOOM.cpp` — M32.
- `tests/GrDDLImageTest.cpp` — M32.
- `tests/GrFinishedFlushTest.cpp` — M38.
- `tests/GrGpuBufferTest.cpp` — M38.
- `tests/GrMeshTest.cpp` — M38.
- `tests/GrMipMappedTest.cpp` — M32, M38.
- `tests/GrOpListFlushTest.cpp` — M38.
- `tests/GrPipelineDynamicStateTest.cpp` — M38.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/GrSurfaceResolveTest.cpp` — M32, M38.
- `tests/GrSurfaceTest.cpp` — M32, M38.
- `tests/GrTextureMipMapInvalidationTest.cpp` — M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/ImageBitmapTest.cpp` — M14.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/InfRectTest.cpp` — M14.
- `tests/JpegGainmapTest.cpp` — M17.
- `tests/LazyProxyTest.cpp` — M32.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/M44Test.cpp` — M05, M07.
- `tests/MaskCacheTest.cpp` — M15.
- `tests/MatrixTest.cpp` — M05.
- `tests/MeshTest.cpp` — M26.
- `tests/MipMapTest.cpp` — M14.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/OffsetSimplePolyTest.cpp` — M06.
- `tests/OpChainTest.cpp` — M32.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/ParsePathTest.cpp` — M07.
- `tests/PathBuilderTest.cpp` — M07.
- `tests/PathOpsAsWindingTest.cpp` — M23.
- `tests/PathOpsBoundsTest.cpp` — M23.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathOpsBuilderTest.cpp` — M23.
- `tests/PathOpsOpTest.cpp` — M23.
- `tests/PathOpsSimplifyTest.cpp` — M23.
- `tests/PathOpsTightBoundsTest.cpp` — M23.
- `tests/PathRawShapesTest.cpp` — M07.
- `tests/PathRawTest.cpp` — M07.
- `tests/PathRendererCacheTests.cpp` — M32.
- `tests/PathTest.cpp` — M22.
- `tests/PictureBBHTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/PngGainmapTest.cpp` — M17.
- `tests/PrimitiveProcessorTest.cpp` — M38.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/ProgramsTest.cpp` — M32.
- `tests/PromiseImageTest.cpp` — M32, M38.
- `tests/ProxyConversionTest.cpp` — M38.
- `tests/ProxyTest.cpp` — M38.
- `tests/QuickRejectTest.cpp` — M15.
- `tests/RRectInPathTest.cpp` — M07.
- `tests/RTreeTest.cpp` — M09.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RecordPatternTest.cpp` — M22.
- `tests/RecordTest.cpp` — M22.
- `tests/RecorderTest.cpp` — M22.
- `tests/RecordingXfermodeTest.cpp` — M22.
- `tests/RectTest.cpp` — M05, M07, M14.
- `tests/RectangleTextureTest.cpp` — M34.
- `tests/RegionTest.cpp` — M14.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RoundRectTest.cpp` — M07, M23.
- `tests/RuntimeBlendTest.cpp` — M35, M38.
- `tests/SRGBReadWritePixelsTest.cpp` — M38.
- `tests/SaveLayerOriginTest.cpp` — M38.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/ShadowTest.cpp` — M15.
- `tests/SimplifyPaintTest.cpp` — M35, M38.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkIcoRustDecoderTest.cpp` — M16.
- `tests/SkImageTest.cpp` — M14.
- `tests/SkPngRustDecoderTest.cpp` — M16.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkResourceCacheTest.cpp` — M22.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkSLES2ConformanceTest.cpp` — M32, M38.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/Skbug6653.cpp` — M38.
- `tests/SlugTest.cpp` — M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/SrcSrcOverBatchTest.cpp` — M38.
- `tests/StrokeTest.cpp` — M13.
- `tests/SurfaceSemaphoreTest.cpp` — M32, M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureBindingsResetTest.cpp` — M34.
- `tests/TextureOpTest.cpp` — M38.
- `tests/TextureProxyTest.cpp` — M38.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TraceMemoryDumpTest.cpp` — M34.
- `tests/TracingTest.cpp` — M11.
- `tests/TransferPixelsTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/TypefaceTest.cpp` — M19.
- `tests/WangsFormulaTest.cpp` — M31.
- `tests/WindowRectanglesTest.cpp` — M32.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp` — M38.
- `tests/WritePixelsTest.cpp` — M35, M38.
- `tests/Writer32Test.cpp` — M22.
- `tests/YUVTest.cpp` — M17.
- `tests/crbug_500172224.cpp` — M38.
- `tests/graphite/ImageOriginTest.cpp` — M38.
- `tests/graphite/NotifyInUseTest.cpp` — M38.
- `tests/graphite/PersistentPipelineStorageTest.cpp` — M38.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp` — M38.
- `tests/graphite/crbug_513836996.cpp` — M38.
- `tests/graphite/sparse_strips/CoverageTest.cpp` — M38.
- `tests/graphite/sparse_strips/FlattenTest.cpp` — M35.
- `tests/graphite/sparse_strips/OracleTest.cpp` — M35.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 5 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=4`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M04 ACCEPTED may the user hand Deepseek M05.
