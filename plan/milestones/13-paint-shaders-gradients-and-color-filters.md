# Milestone 13: Paint, shaders, gradients and color filters

## Outcome and entry gate

Implement Paint, blend modes, shader composition, gradients, image-shader plumbing and color filters. Preserve local matrices, tile modes, stop interpolation and premultiplied behavior. Runtime-effect-specific shader paths are completed at M26 with a recorded feature boundary.

Begin implementation only after milestone 12 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement Paint, blend modes, shader composition, gradients, image-shader plumbing and color filters. Preserve local matrices, tile modes, stop interpolation and premultiplied behavior. Runtime-effect-specific shader paths are completed at M26 with a recorded feature boundary.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkBlendMode.h` — port to Rust.
- `include/core/SkBlender.h` — port to Rust.
- `include/core/SkColorFilter.h` — port to Rust.
- `include/core/SkPaint.h` — port to Rust.
- `include/core/SkShader.h` — port to Rust.
- `include/core/SkTileMode.h` — port to Rust.
- `include/effects/SkBlenders.h` — port to Rust.
- `include/effects/SkColorMatrix.h` — port to Rust.
- `include/effects/SkColorMatrixFilter.h` — port to Rust.
- `include/effects/SkGradient.h` — port to Rust.
- `include/effects/SkHighContrastFilter.h` — port to Rust.
- `include/effects/SkLumaColorFilter.h` — port to Rust.
- `include/effects/SkOverdrawColorFilter.h` — port to Rust.
- `include/effects/SkPerlinNoiseShader.h` — port to Rust.
- `include/private/SkGainmapShader.h` — port to Rust.
- `include/utils/SkPaintFilterCanvas.h` — port to Rust.
- `src/core/SkBlendMode.cpp` — port to Rust.
- `src/core/SkBlendModeBlender.cpp` — port to Rust.
- `src/core/SkBlendModeBlender.h` — port to Rust.
- `src/core/SkBlendModePriv.h` — port to Rust.
- `src/core/SkBlenderBase.h` — port to Rust.
- `src/core/SkColorFilter.cpp` — port to Rust.
- `src/core/SkColorFilterPriv.h` — port to Rust.
- `src/core/SkEffectPriv.h` — port to Rust.
- `src/core/SkGlyphRunPainter.cpp` — port to Rust.
- `src/core/SkGlyphRunPainter.h` — port to Rust.
- `src/core/SkPaint.cpp` — port to Rust.
- `src/core/SkPaintDefaults.h` — port to Rust.
- `src/core/SkPaintPriv.cpp` — port to Rust.
- `src/core/SkPaintPriv.h` — port to Rust.
- `src/effects/SkBlenders.cpp` — port to Rust.
- `src/effects/SkColorMatrix.cpp` — port to Rust.
- `src/effects/SkColorMatrixFilter.cpp` — port to Rust.
- `src/effects/SkDashImpl.h` — port to Rust.
- `src/effects/SkHighContrastFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkBlendModeColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkBlendModeColorFilter.h` — port to Rust.
- `src/effects/colorfilters/SkColorFilterBase.cpp` — port to Rust.
- `src/effects/colorfilters/SkColorFilterBase.h` — port to Rust.
- `src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkColorSpaceXformColorFilter.h` — port to Rust.
- `src/effects/colorfilters/SkComposeColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkComposeColorFilter.h` — port to Rust.
- `src/effects/colorfilters/SkGaussianColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkGaussianColorFilter.h` — port to Rust.
- `src/effects/colorfilters/SkMatrixColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkMatrixColorFilter.h` — port to Rust.
- `src/effects/colorfilters/SkTableColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkTableColorFilter.h` — port to Rust.
- `src/effects/colorfilters/SkWorkingFormatColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkWorkingFormatColorFilter.h` — port to Rust.
- `src/shaders/SkBitmapProcShader.cpp` — port to Rust.
- `src/shaders/SkBitmapProcShader.h` — port to Rust.
- `src/shaders/SkBlendShader.cpp` — port to Rust.
- `src/shaders/SkBlendShader.h` — port to Rust.
- `src/shaders/SkColorFilterShader.cpp` — port to Rust.
- `src/shaders/SkColorFilterShader.h` — port to Rust.
- `src/shaders/SkColorShader.cpp` — port to Rust.
- `src/shaders/SkColorShader.h` — port to Rust.
- `src/shaders/SkCoordClampShader.cpp` — port to Rust.
- `src/shaders/SkCoordClampShader.h` — port to Rust.
- `src/shaders/SkEmptyShader.cpp` — port to Rust.
- `src/shaders/SkEmptyShader.h` — port to Rust.
- `src/shaders/SkGainmapShader.cpp` — port to Rust.
- `src/shaders/SkImageShader.cpp` — port to Rust.
- `src/shaders/SkImageShader.h` — port to Rust.
- `src/shaders/SkLocalMatrixShader.cpp` — port to Rust.
- `src/shaders/SkLocalMatrixShader.h` — port to Rust.
- `src/shaders/SkPerlinNoiseShaderImpl.cpp` — port to Rust.
- `src/shaders/SkPerlinNoiseShaderImpl.h` — port to Rust.
- `src/shaders/SkPerlinNoiseShaderType.h` — port to Rust.
- `src/shaders/SkRuntimeShader.cpp` — port to Rust.
- `src/shaders/SkRuntimeShader.h` — port to Rust.
- `src/shaders/SkShader.cpp` — port to Rust.
- `src/shaders/SkShaderBase.cpp` — port to Rust.
- `src/shaders/SkShaderBase.h` — port to Rust.
- `src/shaders/SkTransformShader.cpp` — port to Rust.
- `src/shaders/SkTransformShader.h` — port to Rust.
- `src/shaders/SkTriColorShader.cpp` — port to Rust.
- `src/shaders/SkTriColorShader.h` — port to Rust.
- `src/shaders/SkWorkingColorSpaceShader.cpp` — port to Rust.
- `src/shaders/SkWorkingColorSpaceShader.h` — port to Rust.
- `src/shaders/gradients/SkConicalGradient.cpp` — port to Rust.
- `src/shaders/gradients/SkConicalGradient.h` — port to Rust.
- `src/shaders/gradients/SkGradientBaseShader.cpp` — port to Rust.
- `src/shaders/gradients/SkGradientBaseShader.h` — port to Rust.
- `src/shaders/gradients/SkLinearGradient.cpp` — port to Rust.
- `src/shaders/gradients/SkLinearGradient.h` — port to Rust.
- `src/shaders/gradients/SkRadialGradient.cpp` — port to Rust.
- `src/shaders/gradients/SkRadialGradient.h` — port to Rust.
- `src/shaders/gradients/SkSweepGradient.cpp` — port to Rust.
- `src/shaders/gradients/SkSweepGradient.h` — port to Rust.
- `src/utils/SkPaintFilterCanvas.cpp` — port to Rust.
- `src/utils/SkShaderUtils.cpp` — port to Rust.
- `src/utils/SkShaderUtils.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/effects/BUILD.bazel`.
- `src/effects/BUILD.bazel`.
- `src/effects/colorfilters/BUILD.bazel`.
- `src/effects/imagefilters/BUILD.bazel`.
- `src/shaders/BUILD.bazel`.
- `src/shaders/gradients/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate shader, paint, gradient, blend and color-filter arithmetic/serialization tests. Rendering-based tests run at M14 and again after runtime effects. Keep original expected colors, stop ordering, opacity rules and invalid-input outcomes.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/HighContrastFilterTest.cpp`.
- `tests/NonlinearBlendingTest.cpp`.
- `tests/ShaderOpacityTest.cpp`.
- `tests/StrokeTest.cpp`.

### Original test registrations assigned to this gate

- `HighContrastFilter_SmokeTest` — `tests/HighContrastFilterTest.cpp:17`; CPU; `DEF_TEST`.
- `HighContrastFilter_InvalidInputs` — `tests/HighContrastFilterTest.cpp:31`; CPU; `DEF_TEST`.
- `SkColorSpaceXformSteps_vs_skcms` — `tests/NonlinearBlendingTest.cpp:18`; CPU; `DEF_TEST`.
- `ShaderOpacity` — `tests/ShaderOpacityTest.cpp:120`; CPU; `DEF_TEST`.
- `Stroke` — `tests/StrokeTest.cpp:191`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `tests/AdvancedBlendTest.cpp` — M38.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/BigImageTest.cpp` — M38.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/BulkRectTest.cpp` — M38.
- `tests/CPUContextRecorderTest.cpp` — M22.
- `tests/CanvasStateTest.cpp` — M17.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/CappedHairlinesTest.cpp` — M14.
- `tests/ClearTest.cpp` — M38.
- `tests/ClipCubicTest.cpp` — M14.
- `tests/ClipperTest.cpp` — M14.
- `tests/CodecTest.cpp` — M17.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/ColorMatrixTest.cpp` — M14.
- `tests/CompressedBackendAllocationTest.cpp` — M38.
- `tests/CoreBlittersTest.cpp` — M14.
- `tests/DMSAATest.cpp` — M38.
- `tests/DashPathEffectTest.cpp` — M14.
- `tests/DebugLayerManagerTest.cpp` — M40.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/DirectMaskLimitTest.cpp` — M38.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/DrawPathTest.cpp` — M14.
- `tests/DrawTextTest.cpp` — M18.
- `tests/EmptyPathTest.cpp` — M14.
- `tests/F16DrawTest.cpp` — M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FlattenDrawableTest.cpp` — M22.
- `tests/FlattenableFactoryToName.cpp` — M22.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/GLFramebufferDeletionTest.cpp` — M34.
- `tests/GainmapShaderTest.cpp` — M16.
- `tests/GaneshContextRecorderTest.cpp` — M38.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrContextOOM.cpp` — M32.
- `tests/GrGpuBufferTest.cpp` — M38.
- `tests/GrMeshTest.cpp` — M38.
- `tests/GrMipMappedTest.cpp` — M32, M38.
- `tests/GrOpListFlushTest.cpp` — M38.
- `tests/GrPipelineDynamicStateTest.cpp` — M38.
- `tests/GrPorterDuffTest.cpp` — M32.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/GrTextureMipMapInvalidationTest.cpp` — M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/HdrMetadataTest.cpp` — M16.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/JpegGainmapTest.cpp` — M17.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/MatrixColorFilterTest.cpp` — M38.
- `tests/MeshTest.cpp` — M26.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/PathOpsConicIntersectionTest.cpp` — M23.
- `tests/PathOpsTightBoundsTest.cpp` — M23.
- `tests/PathRendererCacheTests.cpp` — M32.
- `tests/PathTest.cpp` — M22.
- `tests/PictureBBHTest.cpp` — M22.
- `tests/PictureShaderTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/PinnedImageTest.cpp` — M38.
- `tests/PngGainmapTest.cpp` — M17.
- `tests/ProgramsTest.cpp` — M32.
- `tests/PromiseImageTest.cpp` — M32, M38.
- `tests/ProtectedTest.cpp` — M38.
- `tests/QuickRejectTest.cpp` — M15.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RecordPatternTest.cpp` — M22.
- `tests/RecordTest.cpp` — M22.
- `tests/RecorderTest.cpp` — M22.
- `tests/RecordingXfermodeTest.cpp` — M22.
- `tests/RectTest.cpp` — M14.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RuntimeBlendTest.cpp` — M35, M38.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SaveLayerOriginTest.cpp` — M38.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/SimplifyPaintTest.cpp` — M35, M38.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkImageTest.cpp` — M14.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkSLES2ConformanceTest.cpp` — M32, M38.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/SkStrikeCacheTest.cpp` — M22.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/Skbug5221.cpp` — M32, M38.
- `tests/Skbug6389.cpp` — M15.
- `tests/Skbug6653.cpp` — M38.
- `tests/SlugTest.cpp` — M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/SrcSrcOverBatchTest.cpp` — M38.
- `tests/StencilClearTest.cpp` — M38.
- `tests/StrokerTest.cpp` — M23.
- `tests/SurfaceSemaphoreTest.cpp` — M32, M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureOpTest.cpp` — M38.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/VerticesTest.cpp` — M22.
- `tests/YUVTest.cpp` — M17.
- `tests/crbug_500172224.cpp` — M38.
- `tests/graphite/CacheKeyTest.cpp` — M38.
- `tests/graphite/GraphiteContextRecorderTest.cpp` — M38.
- `tests/graphite/GraphitePromiseImageTest.cpp` — M38.
- `tests/graphite/GraphiteYUVAPromiseImageTest.cpp` — M38.
- `tests/graphite/ImageOriginTest.cpp` — M38.
- `tests/graphite/ImageShaderTest.cpp` — M38.
- `tests/graphite/InnerFillTest.cpp` — M38.
- `tests/graphite/KeyTest.cpp` — M38.
- `tests/graphite/NotifyInUseTest.cpp` — M38.
- `tests/graphite/PersistentPipelineStorageTest.cpp` — M38.
- `tests/graphite/PipelineCallbackTest.cpp` — M38.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp` — M38.
- `tests/graphite/StorageContextTest.cpp` — M38.
- `tests/graphite/TextureFallbackTest.cpp` — M38.
- `tests/graphite/precompile/PaintParamsKeyTest.cpp` — M38.
- `tests/graphite/precompile/ThreadedPrecompileTest.cpp` — M38.
- `tests/graphite/sparse_strips/FlattenTest.cpp` — M35.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 131 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=13`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M13 ACCEPTED may the user hand Deepseek M14.
