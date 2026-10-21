# Milestone 37: Graphite Dawn with Metal

## Outcome and entry gate

Implement the Graphite Dawn adapter using Dawn's Metal backend on macOS. Preserve capability negotiation, device callbacks, resources, render/compute pipelines and error propagation. Treat Dawn and its transitives as external dependencies.

Begin implementation only after milestone 36 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement the Graphite Dawn adapter using Dawn's Metal backend on macOS. Preserve capability negotiation, device callbacks, resources, render/compute pipelines and error propagation. Treat Dawn and its transitives as external dependencies.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/gpu/graphite/dawn/DawnBackendContext.h` — port to Rust.
- `include/gpu/graphite/dawn/DawnGraphiteTypes.h` — port to Rust.
- `include/gpu/graphite/dawn/DawnTypes.h` — port to Rust.
- `include/gpu/graphite/dawn/DawnUtils.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnAsyncWait.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnAsyncWait.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnBackendTexture.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnBuffer.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnBuffer.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnCaps.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnCaps.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnCommandBuffer.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnCommandBuffer.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnComputePipeline.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnComputePipeline.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnErrorChecker.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnErrorChecker.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnGraphicsPipeline.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnGraphicsPipeline.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnGraphiteUtils.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnGraphiteUtils.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnQueueManager.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnQueueManager.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnResourceProvider.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnResourceProvider.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnSampler.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnSampler.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnSharedContext.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnSharedContext.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnTexture.cpp` — port to Rust.
- `src/gpu/graphite/dawn/DawnTexture.h` — port to Rust.
- `src/gpu/graphite/dawn/DawnTextureInfo.cpp` — port to Rust.
- `tools/graphite/dawn/GraphiteDawnToggles.cpp` — port to Rust.
- `tools/graphite/dawn/GraphiteDawnToggles.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/gpu/graphite/dawn/BUILD.bazel`.
- `src/gpu/graphite/dawn/BUILD.bazel`.
- `tools/graphite/dawn/BUILD.bazel`.

## Codex: acceptance-test contract

Translate Dawn and applicable generic Graphite tests with Metal selected explicitly. Exercise device loss, callbacks, uploads, compute and precompile. Vulkan/D3D Dawn adapters are excluded; backend selection must not silently fall back to another platform.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/AtlasOobTest.cpp`.
- `tests/BigImageTest.cpp`.
- `tests/BlurTest.cpp`.
- `tests/CanvasTest.cpp`.
- `tests/DirectMaskLimitTest.cpp`.
- `tests/F16DrawTest.cpp`.
- `tests/FilterResultTest.cpp`.
- `tests/ImageFilterTest.cpp`.
- `tests/RuntimeBlendTest.cpp`.
- `tests/ShaderTest.cpp`.
- `tests/SimplifyPaintTest.cpp`.
- `tests/SkColorSpaceXformStepsTest.cpp`.
- `tests/SkRuntimeEffectTest.cpp`.
- `tests/SkSLTest.cpp`.
- `tests/TextureSizeTest.cpp`.
- `tests/WritePixelsTest.cpp`.
- `tests/graphite/BackendTextureTest.cpp`.
- `tests/graphite/BufferManagerTest.cpp`.
- `tests/graphite/CacheBudgetTest.cpp`.
- `tests/graphite/CacheKeyTest.cpp`.
- `tests/graphite/ComputeTest.cpp`.
- `tests/graphite/DawnBackendTextureTest.cpp`.
- `tests/graphite/DeviceTest.cpp`.
- `tests/graphite/DrawAtlasTest.cpp`.
- `tests/graphite/GraphiteContextRecorderTest.cpp`.
- `tests/graphite/GraphitePromiseImageTest.cpp`.
- `tests/graphite/GraphiteResourceCacheTest.cpp`.
- `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`.
- `tests/graphite/ImageOriginTest.cpp`.
- `tests/graphite/ImageProviderTest.cpp`.
- `tests/graphite/ImageShaderTest.cpp`.
- `tests/graphite/ImageWrapTextureMipmapsTest.cpp`.
- `tests/graphite/InnerFillTest.cpp`.
- `tests/graphite/KeyTest.cpp`.
- `tests/graphite/MultisampleTest.cpp`.
- `tests/graphite/MutableImagesTest.cpp`.
- `tests/graphite/NotifyInUseTest.cpp`.
- `tests/graphite/PersistentPipelineStorageTest.cpp`.
- `tests/graphite/PipelineCallbackTest.cpp`.
- `tests/graphite/PipelineDataCacheTest.cpp`.
- `tests/graphite/ProxyCacheTest.cpp`.
- `tests/graphite/RTEffectTest.cpp`.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp`.
- `tests/graphite/RecorderTest.cpp`.
- `tests/graphite/RecordingOrderTest.cpp`.
- `tests/graphite/RecordingSurfacesTest.cpp`.
- `tests/graphite/StorageContextTest.cpp`.
- `tests/graphite/SubmitWithFinishProcTest.cpp`.
- `tests/graphite/TextureFallbackTest.cpp`.
- `tests/graphite/TextureFormatTest.cpp`.
- `tests/graphite/TextureProxyTest.cpp`.
- `tests/graphite/UpdateBackendTextureTest.cpp`.
- `tests/graphite/UploadBufferManagerTest.cpp`.
- `tests/graphite/VerticesPaddingTest.cpp`.
- `tests/graphite/crbug_513836996.cpp`.
- `tests/graphite/precompile/ChromePrecompileTest.cpp`.
- `tests/graphite/precompile/CombinationBuilderTest.cpp`.
- `tests/graphite/precompile/PaintParamsKeyTest.cpp`.
- `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`.
- `tests/graphite/precompile/PipelineManagerThreaded.cpp`.
- `tests/graphite/precompile/PrecompileStatsTest.cpp`.
- `tests/graphite/precompile/ThreadedPrecompileTest.cpp`.
- `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`.
- `tests/graphite/sparse_strips/AtlasTests.cpp`.
- `tests/graphite/sparse_strips/CoverageTest.cpp`.
- `tools/graphite/dawn/GraphiteDawnTestContext.cpp`.
- `tools/graphite/dawn/GraphiteDawnTestContext.h`.

### Original test registrations assigned to this gate

- `SkSL` — `tests/SkSLTest.cpp:1028`; CPU; `DEF_CONDITIONAL_TEST`.
- `SkSL` — `tests/SkSLTest.cpp:1029`; CPU; `DEF_TEST`.
- `SkSL` — `tests/SkSLTest.cpp:1030`; CPU; `DEF_TEST`.
- `Compute_NativeShaderSourceWGSL` — `tests/graphite/ComputeTest.cpp:2593`; Graphite; `DEF_GRAPHITE_TEST_FOR_DAWN_CONTEXT`.
- `DawnBackendTextureSimpleCreationTest` — `tests/graphite/DawnBackendTextureTest.cpp:24`; Graphite; `DEF_GRAPHITE_TEST_FOR_DAWN_CONTEXT`.
- `DawnBackendTextureCopyVariableTest` — `tests/graphite/DawnBackendTextureTest.cpp:48`; Graphite; `DEF_GRAPHITE_TEST_FOR_DAWN_CONTEXT`.
- `StaticVerticesPaddingTest` — `tests/graphite/VerticesPaddingTest.cpp:133`; Graphite; `DEF_GRAPHITE_TEST_FOR_DAWN_AND_VULKAN_CONTEXTS`.
- `DynamicVerticesPaddingTest` — `tests/graphite/VerticesPaddingTest.cpp:204`; Graphite; `DEF_GRAPHITE_TEST_FOR_DAWN_AND_VULKAN_CONTEXTS`.

### Backend variants required at this gate

These variants must be translated/run here even if their cross-backend consolidation is assigned to M38. Apply the original context filter; never execute an ineligible backend just to inflate the count.

- `Atlas_Oob_graphite` — `tests/AtlasOobTest.cpp`; Graphite Dawn-Metal.
- `BigImageTest_Graphite` — `tests/BigImageTest.cpp`; Graphite Dawn-Metal.
- `TiledDrawCacheTest_Graphite` — `tests/BigImageTest.cpp`; Graphite Dawn-Metal.
- `BlurPointCircle` — `tests/BlurTest.cpp`; Graphite Dawn-Metal.
- `TestManyDrawsGraphite` — `tests/CanvasTest.cpp`; Graphite Dawn-Metal.
- `DirectMaskLimitTest_Graphite` — `tests/DirectMaskLimitTest.cpp`; Graphite Dawn-Metal.
- `F16DrawTest_Graphite` — `tests/F16DrawTest.cpp`; Graphite Dawn-Metal.
- `FilterResult_graphite_` — `tests/FilterResultTest.cpp`; Graphite Dawn-Metal.
- `ImageFilterMakeWithFilter_Graphite` — `tests/ImageFilterTest.cpp`; Graphite Dawn-Metal.
- `ImageFilterMakeWithFilter_ScratchReuse_Graphite` — `tests/ImageFilterTest.cpp`; Graphite Dawn-Metal.
- `SkRuntimeBlender_Graphite` — `tests/RuntimeBlendTest.cpp`; Graphite Dawn-Metal.
- `ShaderTestNestedBlendsGraphite` — `tests/ShaderTest.cpp`; Graphite Dawn-Metal.
- `SimplifyPaintTest_Graphite` — `tests/SimplifyPaintTest.cpp`; Graphite Dawn-Metal.
- `SkColorSpaceXform_Graphite` — `tests/SkColorSpaceXformStepsTest.cpp`; Graphite Dawn-Metal.
- `SkRuntimeEffectSimple_Graphite` — `tests/SkRuntimeEffectTest.cpp`; Graphite Dawn-Metal.
- `SkRuntimeShader_TransformedCoords_Graphite` — `tests/SkRuntimeEffectTest.cpp`; Graphite Dawn-Metal.
- `SkSL` — `tests/SkSLTest.cpp`; Graphite Dawn-Metal.
- `flags` — `tests/SkSLTest.cpp`; Graphite Dawn-Metal.
- `ImageSizeTest_Graphite` — `tests/TextureSizeTest.cpp`; Graphite Dawn-Metal.
- `WritePixels_Graphite` — `tests/WritePixelsTest.cpp`; Graphite Dawn-Metal.
- `BackendTextureTest` — `tests/graphite/BackendTextureTest.cpp`; Graphite Dawn-Metal.
- `SurfaceBackendTextureTest` — `tests/graphite/BackendTextureTest.cpp`; Graphite Dawn-Metal.
- `ImageBackendTextureTest` — `tests/graphite/BackendTextureTest.cpp`; Graphite Dawn-Metal.
- `BufferManagerGpuOnlyBufferTest` — `tests/graphite/BufferManagerTest.cpp`; Graphite Dawn-Metal.
- `BufferManagerStaleAllocatorTest` — `tests/graphite/BufferManagerTest.cpp`; Graphite Dawn-Metal.
- `CacheBudgetTest` — `tests/graphite/CacheBudgetTest.cpp`; Graphite Dawn-Metal.
- `CacheKeyTest_Picture` — `tests/graphite/CacheKeyTest.cpp`; Graphite Dawn-Metal.
- `CacheKeyTest_Bitmap` — `tests/graphite/CacheKeyTest.cpp`; Graphite Dawn-Metal.
- `Compute_SingleDispatchTest` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_DispatchGroupTest` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_UniformBufferTest` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_ExternallyAssignedBuffer` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_StorageTexture` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_StorageTextureReadAndWrite` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_ReadOnlyStorageBuffer` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_StorageTextureMultipleComputeSteps` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_SampledTexture` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_AtomicOperationsTest` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_AtomicOperationsOverArrayAndStructTest` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_ClearedBuffer` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_ClearOrdering` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_ClearOrderingScratchBuffers` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_IndirectDispatch` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_NativeShaderSourceWGSL` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `Compute_WorkgroupUniformLoadTest` — `tests/graphite/ComputeTest.cpp`; Graphite Dawn-Metal.
- `DawnBackendTextureSimpleCreationTest` — `tests/graphite/DawnBackendTextureTest.cpp`; Graphite Dawn-Metal.
- `DawnBackendTextureCopyVariableTest` — `tests/graphite/DawnBackendTextureTest.cpp`; Graphite Dawn-Metal.
- `DeviceTestVertexTransparency` — `tests/graphite/DeviceTest.cpp`; Graphite Dawn-Metal.
- `BasicDrawAtlas` — `tests/graphite/DrawAtlasTest.cpp`; Graphite Dawn-Metal.
- `ThrashDrawAtlasCache` — `tests/graphite/DrawAtlasTest.cpp`; Graphite Dawn-Metal.
- `DrawAtlasProxyLifetime` — `tests/graphite/DrawAtlasTest.cpp`; Graphite Dawn-Metal.
- `CPUSurface_UsesGraphiteContextAndRasterRecorderToDraw_DrawsPixels` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Dawn-Metal.
- `ImageMakeColorSpace_GraphiteImageWithRecorder_Success` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Dawn-Metal.
- `ImageMakeScaled_GraphiteImageWithRecorder_Success` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Dawn-Metal.
- `CanvasBaseRecorder_GraphiteBasedCanvas_IsOriginalRecorder` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Dawn-Metal.
- `NonVolatileGraphitePromiseImageTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `NonVolatileGraphitePromiseImageFulfillFailureTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `NonVolatileGraphitePromiseImageCreationFailureTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `VolatileGraphitePromiseImageTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `VolatileGraphitePromiseImageFulfillFailureTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `GraphitePromiseImageRecorderLoss` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `GraphitePromiseImageMultipleImgUses` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Dawn-Metal.
- `GraphiteBudgetedResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphitePurgeAsNeededResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphiteZeroSizedResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphitePurgeNotUsedSinceResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphitePurgeNotUsedOverBudgetTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphitePurgeResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphiteScratchResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `GraphiteTimeLimitedPurgeTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Dawn-Metal.
- `NonVolatileGraphiteYUVAPromiseImageTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `NonVolatileGraphiteYUVAPromiseImageFulfillFailureTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `NonVolatileGraphiteYUVAPromiseImageCreationFailureTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `VolatileGraphiteYUVAPromiseImageTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `VolatileGraphiteYUVAPromiseImageFulfillFailureTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `GraphiteYUVAPromiseImageRecorderLoss` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `GraphiteYUVAPromiseImageMultipleImgUses` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Dawn-Metal.
- `ImageOriginTest_drawImage_Graphite` — `tests/graphite/ImageOriginTest.cpp`; Graphite Dawn-Metal.
- `ImageOriginTest_imageShader_Graphite` — `tests/graphite/ImageOriginTest.cpp`; Graphite Dawn-Metal.
- `ImageProviderTest_Graphite_Default` — `tests/graphite/ImageProviderTest.cpp`; Graphite Dawn-Metal.
- `ImageProviderTest_Graphite_Testing` — `tests/graphite/ImageProviderTest.cpp`; Graphite Dawn-Metal.
- `Make_TextureImage_Subset_Test` — `tests/graphite/ImageProviderTest.cpp`; Graphite Dawn-Metal.
- `MakeColorSpace_Test` — `tests/graphite/ImageProviderTest.cpp`; Graphite Dawn-Metal.
- `ImageShaderTest` — `tests/graphite/ImageShaderTest.cpp`; Graphite Dawn-Metal.
- `ImageWrapTextureMipmapsTest` — `tests/graphite/ImageWrapTextureMipmapsTest.cpp`; Graphite Dawn-Metal.
- `InnerFillTest` — `tests/graphite/InnerFillTest.cpp`; Graphite Dawn-Metal.
- `OptimizeForOpacity` — `tests/graphite/InnerFillTest.cpp`; Graphite Dawn-Metal.
- `KeyWithInvalidCodeSnippetIDTest` — `tests/graphite/KeyTest.cpp`; Graphite Dawn-Metal.
- `KeyEqualityChecksSnippetID` — `tests/graphite/KeyTest.cpp`; Graphite Dawn-Metal.
- `ShaderInfoDetectsFixedFunctionBlend` — `tests/graphite/KeyTest.cpp`; Graphite Dawn-Metal.
- `MultisampleRetainTest` — `tests/graphite/MultisampleTest.cpp`; Graphite Dawn-Metal.
- `MultisampleClearThenLoad` — `tests/graphite/MultisampleTest.cpp`; Graphite Dawn-Metal.
- `MutableImagesTest` — `tests/graphite/MutableImagesTest.cpp`; Graphite Dawn-Metal.
- `NotifyInUseTestSnapshot` — `tests/graphite/NotifyInUseTest.cpp`; Graphite Dawn-Metal.
- `NotifyInUseTestAsImage` — `tests/graphite/NotifyInUseTest.cpp`; Graphite Dawn-Metal.
- `NotifyInUseTestLayer` — `tests/graphite/NotifyInUseTest.cpp`; Graphite Dawn-Metal.
- `PersistentPipelineStorageTest` — `tests/graphite/PersistentPipelineStorageTest.cpp`; Graphite Dawn-Metal.
- `PipelineCallbackTest_Deprecated_Normal` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Dawn-Metal.
- `PipelineCallbackTest_Normal` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Dawn-Metal.
- `PipelineCallbackTest_Deprecated_Precompile` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Dawn-Metal.
- `PipelineCallbackTest_Precompile` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Dawn-Metal.
- `PipelineCallbackTest_UniqueHash` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Dawn-Metal.
- `PipelineDataCacheTest` — `tests/graphite/PipelineDataCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest1` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest2` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest4` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest5` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest6` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest7` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest8` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest9` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `ProxyCacheTest10` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Dawn-Metal.
- `Shader_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Dawn-Metal.
- `ColorFilter_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Dawn-Metal.
- `ShaderUniforms_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Dawn-Metal.
- `ColorFilterUniforms_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Dawn-Metal.
- `ImageAsyncReadPixelsGraphite` — `tests/graphite/ReadWritePixelsGraphiteTest.cpp`; Graphite Dawn-Metal.
- `SurfaceAsyncReadPixelsGraphite` — `tests/graphite/ReadWritePixelsGraphiteTest.cpp`; Graphite Dawn-Metal.
- `ImageAsyncRescaleReadPixelsOriginGraphite` — `tests/graphite/ReadWritePixelsGraphiteTest.cpp`; Graphite Dawn-Metal.
- `RecorderDevicePtrTest` — `tests/graphite/RecorderTest.cpp`; Graphite Dawn-Metal.
- `RecorderOrderingTest` — `tests/graphite/RecorderTest.cpp`; Graphite Dawn-Metal.
- `NullRecordingInsertTest` — `tests/graphite/RecorderTest.cpp`; Graphite Dawn-Metal.
- `InsertRecordingPendingCountsTest` — `tests/graphite/RecorderTest.cpp`; Graphite Dawn-Metal.
- `RecordingOrderTest_Graphite` — `tests/graphite/RecordingOrderTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestClear` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestDraw` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestWritePixels` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestWritePixelsOffscreen` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestDrawWithClip` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestNegativeClip` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestWritePixelsWithClip` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestMipmapped` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestMipmappedWritePixels` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestTwoCanvases` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `RecordingSurfacesTestUnnecessarySurface` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Dawn-Metal.
- `StorageContextAlignmentTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextPaddingAlignmentTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextAppendVertexTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextMultipleRenderStepsTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextLCMVariantsTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextVertexOnlyAndResetTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextMultiStopGradientTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextFallbackStridedCopyTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `StorageContextCapacityLimitsTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Dawn-Metal.
- `SubmitWithFinishProc_PendingCommandBuffer` — `tests/graphite/SubmitWithFinishProcTest.cpp`; Graphite Dawn-Metal.
- `SubmitWithFinishProc_NoPendingCommandBuffer` — `tests/graphite/SubmitWithFinishProcTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackScalarsAndBitcastsTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackAlignmentAndPaddingTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackMatricesTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackVectorBitcastsTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackMultiInstanceMathTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackStressMixedTypesAndAlignmentTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackStressBackToBackComplexStructuresTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFallbackMultiStopGradientsDrawTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Dawn-Metal.
- `TextureFormatTest` — `tests/graphite/TextureFormatTest.cpp`; Graphite Dawn-Metal.
- `GraphiteTextureProxyTest` — `tests/graphite/TextureProxyTest.cpp`; Graphite Dawn-Metal.
- `GraphiteTextureTooLargeTest` — `tests/graphite/TextureProxyTest.cpp`; Graphite Dawn-Metal.
- `GraphiteLazyTextureInvalidDimensions` — `tests/graphite/TextureProxyTest.cpp`; Graphite Dawn-Metal.
- `UpdateImageBackendTextureTest` — `tests/graphite/UpdateBackendTextureTest.cpp`; Graphite Dawn-Metal.
- `UpdateBackendTextureFinishedProcTest` — `tests/graphite/UpdateBackendTextureTest.cpp`; Graphite Dawn-Metal.
- `UploadBufferManagerTest` — `tests/graphite/UploadBufferManagerTest.cpp`; Graphite Dawn-Metal.
- `StaticVerticesPaddingTest` — `tests/graphite/VerticesPaddingTest.cpp`; Graphite Dawn-Metal.
- `DynamicVerticesPaddingTest` — `tests/graphite/VerticesPaddingTest.cpp`; Graphite Dawn-Metal.
- `crbug_513836996` — `tests/graphite/crbug_513836996.cpp`; Graphite Dawn-Metal.
- `ChromePrecompileTest` — `tests/graphite/precompile/ChromePrecompileTest.cpp`; Graphite Dawn-Metal.
- `CombinationBuilderTest` — `tests/graphite/precompile/CombinationBuilderTest.cpp`; Graphite Dawn-Metal.
- `PaintParamsKeyTestReduced` — `tests/graphite/precompile/PaintParamsKeyTest.cpp`; Graphite Dawn-Metal.
- `PaintParamsKeyTest` — `tests/graphite/precompile/PaintParamsKeyTest.cpp`; Graphite Dawn-Metal.
- `PipelineManagerEarlyExitTest_1` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Dawn-Metal.
- `PipelineManagerEarlyExitTest_2` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Dawn-Metal.
- `PipelineManagerEarlyExitTest_3` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Dawn-Metal.
- `PipelineManagerEarlyExitTest_4` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Dawn-Metal.
- `PipelineManagerThreadedTest_1` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Dawn-Metal.
- `PipelineManagerThreadedTest_2` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Dawn-Metal.
- `PipelineManagerThreadedTest_3` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Dawn-Metal.
- `PipelineManagerThreadedTest_4` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Dawn-Metal.
- `PrecompileStatsTest` — `tests/graphite/precompile/PrecompileStatsTest.cpp`; Graphite Dawn-Metal.
- `ThreadedPipelinePrecompileTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Dawn-Metal.
- `ThreadedPipelinePrecompileCompileTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Dawn-Metal.
- `ThreadedPipelineCompilePurgingTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Dawn-Metal.
- `ThreadedPipelinePrecompilePurgingTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Dawn-Metal.
- `ThreadedPipelinePrecompileCompilePurgingTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Dawn-Metal.
- `UserDefinedStableKeyTest` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Dawn-Metal.
- `UserDefinedStableKeyTest_Duplicates` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Dawn-Metal.
- `UserDefinedStableKeyTest_Nullptrs` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Dawn-Metal.
- `UserDefinedStableKeyTest_Overflow` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasDimensionsAndStride` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasDynamicGrowthAndDoubling` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasNullBufferAndNullCaps` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasStraddleToNullBuffer` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasEndCapAtomicity` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasFlipFlopAtFlush` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_AtlasStraddleAndMemmove` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Dawn-Metal.
- `SparseStrips_CoverageScalar_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_CoverageSIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_CoverageSIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_CullingSimplification_SIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_CullingSimplification_SIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_CullingSimplification_Scalar_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_CulledGeometry_SIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_CulledGeometry_SIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_CulledGeometry_Scalar_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_StripGeneratorHistogramReuse` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_SKP_SIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.
- `SparseStrips_Coverage_SKP_SIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Dawn-Metal.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/SkSLTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=37`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M37 ACCEPTED may the user hand Deepseek M38.
