# Milestone 31: Shared GPU infrastructure and tessellation

## Outcome and entry gate

Implement backend-independent GPU types, resource keys, swizzles, blend formulas, buffer writers, rectangle allocation, tessellation and shared shader interfaces. Preserve ABI layouts and shader/CPU agreement. Keep GPU algorithmic logic in Rust.

Begin implementation only after milestone 30 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement backend-independent GPU types, resource keys, swizzles, blend formulas, buffer writers, rectangle allocation, tessellation and shared shader interfaces. Preserve ABI layouts and shader/CPU agreement. Keep GPU algorithmic logic in Rust.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/gpu/GpuTypes.h` — port to Rust.
- `include/gpu/MutableTextureState.h` — port to Rust.
- `include/gpu/ShaderErrorHandler.h` — port to Rust.
- `include/gpu/mtl/MtlMemoryAllocator.h` — port to Rust.
- `src/gpu/AsyncReadTypes.h` — port to Rust.
- `src/gpu/Blend.cpp` — port to Rust.
- `src/gpu/Blend.h` — port to Rust.
- `src/gpu/BlendFormula.cpp` — port to Rust.
- `src/gpu/BlendFormula.h` — port to Rust.
- `src/gpu/BlurUtils.cpp` — port to Rust.
- `src/gpu/BlurUtils.h` — port to Rust.
- `src/gpu/BufferWriter.h` — port to Rust.
- `src/gpu/DataUtils.cpp` — port to Rust.
- `src/gpu/DataUtils.h` — port to Rust.
- `src/gpu/DitherUtils.cpp` — port to Rust.
- `src/gpu/DitherUtils.h` — port to Rust.
- `src/gpu/GlobalResourceStats.cpp` — port to Rust.
- `src/gpu/GlobalResourceStats.h` — port to Rust.
- `src/gpu/GpuRefCnt.h` — port to Rust.
- `src/gpu/GpuTypesPriv.h` — port to Rust.
- `src/gpu/GradientBitmap.cpp` — port to Rust.
- `src/gpu/GradientBitmap.h` — port to Rust.
- `src/gpu/KeyBuilder.h` — port to Rust.
- `src/gpu/MaskFormat.h` — port to Rust.
- `src/gpu/MutableTextureState.cpp` — port to Rust.
- `src/gpu/MutableTextureStatePriv.h` — port to Rust.
- `src/gpu/Rectanizer.h` — port to Rust.
- `src/gpu/RectanizerPow2.cpp` — port to Rust.
- `src/gpu/RectanizerPow2.h` — port to Rust.
- `src/gpu/RectanizerSkyline.cpp` — port to Rust.
- `src/gpu/RectanizerSkyline.h` — port to Rust.
- `src/gpu/RefCntedCallback.h` — port to Rust.
- `src/gpu/ResourceKey.cpp` — port to Rust.
- `src/gpu/ResourceKey.h` — port to Rust.
- `src/gpu/ShaderErrorHandler.cpp` — port to Rust.
- `src/gpu/SkBackingFit.cpp` — port to Rust.
- `src/gpu/SkBackingFit.h` — port to Rust.
- `src/gpu/SkRenderEngineAbortf.h` — port to Rust.
- `src/gpu/SkSLToBackend.cpp` — port to Rust.
- `src/gpu/SkSLToBackend.h` — port to Rust.
- `src/gpu/Swizzle.cpp` — port to Rust.
- `src/gpu/Swizzle.h` — port to Rust.
- `src/gpu/SwizzlePriv.h` — port to Rust.
- `src/gpu/TiledTextureUtils.cpp` — port to Rust.
- `src/gpu/TiledTextureUtils.h` — port to Rust.
- `src/gpu/Token.h` — port to Rust.
- `src/gpu/mtl/MtlMemoryAllocatorImpl.h` — port to Rust.
- `src/gpu/mtl/MtlMemoryAllocatorImpl.mm` — port to Rust.
- `src/gpu/mtl/MtlUtils.mm` — port to Rust.
- `src/gpu/mtl/MtlUtilsPriv.h` — port to Rust.
- `src/gpu/tessellate/AffineMatrix.h` — port to Rust.
- `src/gpu/tessellate/CullTest.h` — port to Rust.
- `src/gpu/tessellate/FixedCountBufferUtils.cpp` — port to Rust.
- `src/gpu/tessellate/FixedCountBufferUtils.h` — port to Rust.
- `src/gpu/tessellate/LinearTolerances.h` — port to Rust.
- `src/gpu/tessellate/MiddleOutPolygonTriangulator.h` — port to Rust.
- `src/gpu/tessellate/MidpointContourParser.h` — port to Rust.
- `src/gpu/tessellate/PatchWriter.h` — port to Rust.
- `src/gpu/tessellate/StrokeIterator.h` — port to Rust.
- `src/gpu/tessellate/Tessellation.cpp` — port to Rust.
- `src/gpu/tessellate/Tessellation.h` — port to Rust.
- `src/gpu/tessellate/WangsFormula.h` — port to Rust.
- `tools/gpu/BackendSurfaceFactory.cpp` — port to Rust.
- `tools/gpu/BackendSurfaceFactory.h` — port to Rust.
- `tools/gpu/BackendTextureImageFactory.cpp` — port to Rust.
- `tools/gpu/BackendTextureImageFactory.h` — port to Rust.
- `tools/gpu/CompressedTexture.cpp` — port to Rust.
- `tools/gpu/CompressedTexture.h` — port to Rust.
- `tools/gpu/ContextType.cpp` — port to Rust.
- `tools/gpu/ContextType.h` — port to Rust.
- `tools/gpu/FlushFinishTracker.cpp` — port to Rust.
- `tools/gpu/FlushFinishTracker.h` — port to Rust.
- `tools/gpu/ManagedBackendTexture.cpp` — port to Rust.
- `tools/gpu/ManagedBackendTexture.h` — port to Rust.
- `tools/gpu/ProtectedUtils.h` — port to Rust.
- `tools/gpu/YUVUtils.cpp` — port to Rust.
- `tools/gpu/YUVUtils.h` — port to Rust.
- `tools/text/gpu/TextBlobTools.cpp` — port to Rust.
- `tools/text/gpu/TextBlobTools.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/gpu/BUILD.bazel`.
- `include/gpu/mtl/BUILD.bazel`.
- `include/private/gpu/BUILD.bazel`.
- `src/gpu/BUILD.bazel`.
- `src/gpu/mtl/BUILD.bazel`.
- `src/gpu/tessellate/BUILD.bazel`.
- `tools/gpu/BUILD.bazel`.
- `tools/text/gpu/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate CPU-executable GPU geometry/tessellation/key tests, including curve chopping and Wang's formula. Verify byte layouts, alignment, index limits and original numerical thresholds. Actual driver execution begins at the concrete backend milestones.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/CullTestTest.cpp`.
- `tests/FindCubicConvex180ChopsTest.cpp`.
- `tests/GpuRectanizerTest.cpp`.
- `tests/PreChopPathCurvesTest.cpp`.
- `tests/SkRasterPipelineTest.cpp`.
- `tests/WangsFormulaTest.cpp`.

### Original test registrations assigned to this gate

- `CullTestTest` — `tests/CullTestTest.cpp:35`; CPU; `DEF_TEST`.
- `FindCubicConvex180Chops` — `tests/FindCubicConvex180ChopsTest.cpp:76`; CPU; `DEF_TEST`.
- `PreChopPathCurves` — `tests/PreChopPathCurvesTest.cpp:16`; CPU; `DEF_TEST`.
- `SkRasterPipeline_swizzle` — `tests/SkRasterPipelineTest.cpp:3322`; CPU; `DEF_TEST`.
- `wangs_formula_log2` — `tests/WangsFormulaTest.cpp:130`; CPU; `DEF_TEST`.
- `wangs_formula_vectorXforms` — `tests/WangsFormulaTest.cpp:292`; CPU; `DEF_TEST`.
- `wangs_formula_worst_case_cubic` — `tests/WangsFormulaTest.cpp:326`; CPU; `DEF_TEST`.
- `wangs_formula_quad_within_tol` — `tests/WangsFormulaTest.cpp:364`; CPU; `DEF_TEST`.
- `wangs_formula_rational_quad_reduces` — `tests/WangsFormulaTest.cpp:416`; CPU; `DEF_TEST`.
- `wangs_formula_conic_within_tol` — `tests/WangsFormulaTest.cpp:431`; CPU; `DEF_TEST`.
- `wangs_formula_conic_matches_reference` — `tests/WangsFormulaTest.cpp:493`; CPU; `DEF_TEST`.
- `wangs_formula_conic_vectorXforms` — `tests/WangsFormulaTest.cpp:510`; CPU; `DEF_TEST`.
- `wangs_formula_nextlog2` — `tests/WangsFormulaTest.cpp:538`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/AdvancedBlendTest.cpp` — M38.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/AtlasOobTest.cpp` — M38.
- `tests/BackendAllocationTest.cpp` — M32, M34.
- `tests/BigImageTest.cpp` — M38.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/BulkRectTest.cpp` — M38.
- `tests/ClearTest.cpp` — M38.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/CompressedBackendAllocationTest.cpp` — M38.
- `tests/CopySurfaceTest.cpp` — M38.
- `tests/DMSAATest.cpp` — M38.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/DeviceTest.cpp` — M32, M38.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/ExtendedSkColorTypeTests.cpp` — M32, M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FloatingPointTextureTest.cpp` — M38.
- `tests/GLBackendSurfaceTest.cpp` — M34.
- `tests/GLFramebufferDeletionTest.cpp` — M34.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GpuRectanizerTest.cpp` — M32.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrContextAbandonTest.cpp` — M32.
- `tests/GrContextFactoryTest.cpp` — M32, M38.
- `tests/GrContextOOM.cpp` — M32.
- `tests/GrDDLImageTest.cpp` — M32.
- `tests/GrFinishedFlushTest.cpp` — M38.
- `tests/GrGpuBufferTest.cpp` — M38.
- `tests/GrMeshTest.cpp` — M38.
- `tests/GrMipMappedTest.cpp` — M32, M38.
- `tests/GrOpListFlushTest.cpp` — M38.
- `tests/GrPipelineDynamicStateTest.cpp` — M38.
- `tests/GrPorterDuffTest.cpp` — M32.
- `tests/GrSubmittedFlushTest.cpp` — M38.
- `tests/GrSurfaceResolveTest.cpp` — M32, M38.
- `tests/GrSurfaceTest.cpp` — M32, M38.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/GrTextureMipMapInvalidationTest.cpp` — M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GrUniqueProxyTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageIsOpaqueTest.cpp` — M32, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/LazyProxyTest.cpp` — M32.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/MatrixColorFilterTest.cpp` — M38.
- `tests/MtlBackendAllocationTest.cc` — M33.
- `tests/OpChainTest.cpp` — M32.
- `tests/PathRendererCacheTests.cpp` — M32.
- `tests/PinnedImageTest.cpp` — M38.
- `tests/PremulAlphaRoundTripTest.cpp` — M32, M38.
- `tests/PrimitiveProcessorTest.cpp` — M38.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/ProgramsTest.cpp` — M32.
- `tests/PromiseImageTest.cpp` — M32, M38.
- `tests/ProtectedTest.cpp` — M38.
- `tests/ProxyConversionTest.cpp` — M38.
- `tests/ProxyRefTest.cpp` — M38.
- `tests/ProxyTest.cpp` — M38.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RectangleTextureTest.cpp` — M34.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/ResourceAllocatorTest.cpp` — M38.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RuntimeBlendTest.cpp` — M35, M38.
- `tests/SRGBReadWritePixelsTest.cpp` — M38.
- `tests/SaveLayerOriginTest.cpp` — M38.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkSLES2ConformanceTest.cpp` — M32, M38.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/Skbug12214.cpp` — M38.
- `tests/Skbug5221.cpp` — M32, M38.
- `tests/Skbug6653.cpp` — M38.
- `tests/SlugTest.cpp` — M38.
- `tests/SmallPathRendererTest.cpp` — M32, M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/SrcSrcOverBatchTest.cpp` — M38.
- `tests/SurfaceDrawContextTest.cpp` — M38.
- `tests/SurfaceSemaphoreTest.cpp` — M32, M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/Test.h` — M32.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextureBindingsResetTest.cpp` — M34.
- `tests/TextureOpTest.cpp` — M38.
- `tests/TextureProxyTest.cpp` — M38.
- `tests/TextureSizeTest.cpp` — M38.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TraceMemoryDumpTest.cpp` — M34.
- `tests/TransferPixelsTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp` — M38.
- `tests/WritePixelsTest.cpp` — M35, M38.
- `tests/graphite/BackendTextureTest.cpp` — M38.
- `tests/graphite/DrawAtlasTest.cpp` — M35, M38.
- `tests/graphite/ImageOriginTest.cpp` — M38.
- `tests/graphite/ImageWrapTextureMipmapsTest.cpp` — M38.
- `tests/graphite/InnerFillTest.cpp` — M38.
- `tests/graphite/KeyTest.cpp` — M38.
- `tests/graphite/MutableImagesTest.cpp` — M38.
- `tests/graphite/ProxyCacheTest.cpp` — M38.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp` — M38.
- `tests/graphite/RecorderTest.cpp` — M38.
- `tests/graphite/SwizzleTest.cpp` — M35.
- `tests/graphite/TextureFormatTest.cpp` — M38.

### Preserve these fixtures, metadata and references

- `src/gpu/gpu_workaround_list.txt` — preserve.

## Dependency and integration notes

The initial scan found 35 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=31`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M31 ACCEPTED may the user hand Deepseek M32.
