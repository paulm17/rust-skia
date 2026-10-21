# Milestone 36: Graphite Metal backend

## Outcome and entry gate

Implement Graphite Metal resource creation, render/compute pipelines, command submission, synchronization, recording replay and readback. Complete native Metal capability discovery and validation integration.

Begin implementation only after milestone 35 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement Graphite Metal resource creation, render/compute pipelines, command submission, synchronization, recording replay and readback. Complete native Metal capability discovery and validation integration.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/gpu/graphite/mtl/MtlBackendContext.h` — port to Rust.
- `include/gpu/graphite/mtl/MtlGraphiteTypes.h` — port to Rust.
- `include/gpu/graphite/mtl/MtlGraphiteTypesUtils.h` — port to Rust.
- `include/gpu/graphite/mtl/MtlGraphiteTypes_cpp.h` — port to Rust.
- `include/gpu/graphite/mtl/MtlGraphiteUtils.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlBackendSemaphore.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlBackendTexture.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlBlitCommandEncoder.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlBuffer.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlBuffer.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlCaps.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlCaps.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlCommandBuffer.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlCommandBuffer.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlComputeCommandEncoder.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlComputePipeline.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlComputePipeline.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlGraphicsPipeline.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlGraphicsPipeline.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlGraphiteUtils.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlGraphiteUtils.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlQueueManager.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlQueueManager.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlRenderCommandEncoder.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlResourceProvider.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlResourceProvider.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlSampler.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlSampler.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlSharedContext.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlSharedContext.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlTexture.h` — port to Rust.
- `src/gpu/graphite/mtl/MtlTexture.mm` — port to Rust.
- `src/gpu/graphite/mtl/MtlTextureInfo.mm` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/gpu/graphite/mtl/BUILD.bazel`.
- `src/gpu/graphite/mtl/BUILD.bazel`.
- `tools/graphite/mtl/BUILD.bazel`.

## Codex: acceptance-test contract

Translate Metal-specific Graphite tests and execute all applicable generic Graphite tests on Metal. Exercise rendering, compute where supported, resource reuse, async reads, precompile and replay. Record adapter/OS/capability metadata with results.

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
- `tests/graphite/MtlBackendTextureTest.cc`.
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
- `tools/graphite/mtl/GraphiteMtlTestContext.h`.
- `tools/graphite/mtl/GraphiteMtlTestContext.mm`.

### Original test registrations assigned to this gate

- `Compute_NativeShaderSourceMetal` — `tests/graphite/ComputeTest.cpp:2342`; Graphite; `DEF_GRAPHITE_TEST_FOR_METAL_CONTEXT`.
- `Compute_WorkgroupBufferDescMetal` — `tests/graphite/ComputeTest.cpp:2464`; Graphite; `DEF_GRAPHITE_TEST_FOR_METAL_CONTEXT`.
- `MtlBackendTextureTest` — `tests/graphite/MtlBackendTextureTest.cc:23`; Graphite; `DEF_GRAPHITE_TEST_FOR_METAL_CONTEXT`.

### Backend variants required at this gate

These variants must be translated/run here even if their cross-backend consolidation is assigned to M38. Apply the original context filter; never execute an ineligible backend just to inflate the count.

- `Atlas_Oob_graphite` — `tests/AtlasOobTest.cpp`; Graphite Metal.
- `BigImageTest_Graphite` — `tests/BigImageTest.cpp`; Graphite Metal.
- `TiledDrawCacheTest_Graphite` — `tests/BigImageTest.cpp`; Graphite Metal.
- `BlurPointCircle` — `tests/BlurTest.cpp`; Graphite Metal.
- `TestManyDrawsGraphite` — `tests/CanvasTest.cpp`; Graphite Metal.
- `DirectMaskLimitTest_Graphite` — `tests/DirectMaskLimitTest.cpp`; Graphite Metal.
- `F16DrawTest_Graphite` — `tests/F16DrawTest.cpp`; Graphite Metal.
- `FilterResult_graphite_` — `tests/FilterResultTest.cpp`; Graphite Metal.
- `ImageFilterMakeWithFilter_Graphite` — `tests/ImageFilterTest.cpp`; Graphite Metal.
- `ImageFilterMakeWithFilter_ScratchReuse_Graphite` — `tests/ImageFilterTest.cpp`; Graphite Metal.
- `SkRuntimeBlender_Graphite` — `tests/RuntimeBlendTest.cpp`; Graphite Metal.
- `ShaderTestNestedBlendsGraphite` — `tests/ShaderTest.cpp`; Graphite Metal.
- `SimplifyPaintTest_Graphite` — `tests/SimplifyPaintTest.cpp`; Graphite Metal.
- `SkColorSpaceXform_Graphite` — `tests/SkColorSpaceXformStepsTest.cpp`; Graphite Metal.
- `SkRuntimeEffectSimple_Graphite` — `tests/SkRuntimeEffectTest.cpp`; Graphite Metal.
- `SkRuntimeShader_TransformedCoords_Graphite` — `tests/SkRuntimeEffectTest.cpp`; Graphite Metal.
- `SkSL` — `tests/SkSLTest.cpp`; Graphite Metal.
- `flags` — `tests/SkSLTest.cpp`; Graphite Metal.
- `ImageSizeTest_Graphite` — `tests/TextureSizeTest.cpp`; Graphite Metal.
- `WritePixels_Graphite` — `tests/WritePixelsTest.cpp`; Graphite Metal.
- `BackendTextureTest` — `tests/graphite/BackendTextureTest.cpp`; Graphite Metal.
- `SurfaceBackendTextureTest` — `tests/graphite/BackendTextureTest.cpp`; Graphite Metal.
- `ImageBackendTextureTest` — `tests/graphite/BackendTextureTest.cpp`; Graphite Metal.
- `BufferManagerGpuOnlyBufferTest` — `tests/graphite/BufferManagerTest.cpp`; Graphite Metal.
- `BufferManagerStaleAllocatorTest` — `tests/graphite/BufferManagerTest.cpp`; Graphite Metal.
- `CacheBudgetTest` — `tests/graphite/CacheBudgetTest.cpp`; Graphite Metal.
- `CacheKeyTest_Picture` — `tests/graphite/CacheKeyTest.cpp`; Graphite Metal.
- `CacheKeyTest_Bitmap` — `tests/graphite/CacheKeyTest.cpp`; Graphite Metal.
- `Compute_SingleDispatchTest` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_DispatchGroupTest` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_UniformBufferTest` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_ExternallyAssignedBuffer` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_StorageTexture` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_StorageTextureReadAndWrite` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_ReadOnlyStorageBuffer` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_StorageTextureMultipleComputeSteps` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_SampledTexture` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_AtomicOperationsTest` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_AtomicOperationsOverArrayAndStructTest` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_ClearedBuffer` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_ClearOrdering` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_ClearOrderingScratchBuffers` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_IndirectDispatch` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_NativeShaderSourceMetal` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_WorkgroupBufferDescMetal` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `Compute_WorkgroupUniformLoadTest` — `tests/graphite/ComputeTest.cpp`; Graphite Metal.
- `DeviceTestVertexTransparency` — `tests/graphite/DeviceTest.cpp`; Graphite Metal.
- `BasicDrawAtlas` — `tests/graphite/DrawAtlasTest.cpp`; Graphite Metal.
- `ThrashDrawAtlasCache` — `tests/graphite/DrawAtlasTest.cpp`; Graphite Metal.
- `DrawAtlasProxyLifetime` — `tests/graphite/DrawAtlasTest.cpp`; Graphite Metal.
- `CPUSurface_UsesGraphiteContextAndRasterRecorderToDraw_DrawsPixels` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Metal.
- `ImageMakeColorSpace_GraphiteImageWithRecorder_Success` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Metal.
- `ImageMakeScaled_GraphiteImageWithRecorder_Success` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Metal.
- `CanvasBaseRecorder_GraphiteBasedCanvas_IsOriginalRecorder` — `tests/graphite/GraphiteContextRecorderTest.cpp`; Graphite Metal.
- `NonVolatileGraphitePromiseImageTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `NonVolatileGraphitePromiseImageFulfillFailureTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `NonVolatileGraphitePromiseImageCreationFailureTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `VolatileGraphitePromiseImageTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `VolatileGraphitePromiseImageFulfillFailureTest` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `GraphitePromiseImageRecorderLoss` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `GraphitePromiseImageMultipleImgUses` — `tests/graphite/GraphitePromiseImageTest.cpp`; Graphite Metal.
- `GraphiteBudgetedResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphitePurgeAsNeededResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphiteZeroSizedResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphitePurgeNotUsedSinceResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphitePurgeNotUsedOverBudgetTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphitePurgeResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphiteScratchResourcesTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `GraphiteTimeLimitedPurgeTest` — `tests/graphite/GraphiteResourceCacheTest.cpp`; Graphite Metal.
- `NonVolatileGraphiteYUVAPromiseImageTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `NonVolatileGraphiteYUVAPromiseImageFulfillFailureTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `NonVolatileGraphiteYUVAPromiseImageCreationFailureTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `VolatileGraphiteYUVAPromiseImageTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `VolatileGraphiteYUVAPromiseImageFulfillFailureTest` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `GraphiteYUVAPromiseImageRecorderLoss` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `GraphiteYUVAPromiseImageMultipleImgUses` — `tests/graphite/GraphiteYUVAPromiseImageTest.cpp`; Graphite Metal.
- `ImageOriginTest_drawImage_Graphite` — `tests/graphite/ImageOriginTest.cpp`; Graphite Metal.
- `ImageOriginTest_imageShader_Graphite` — `tests/graphite/ImageOriginTest.cpp`; Graphite Metal.
- `ImageProviderTest_Graphite_Default` — `tests/graphite/ImageProviderTest.cpp`; Graphite Metal.
- `ImageProviderTest_Graphite_Testing` — `tests/graphite/ImageProviderTest.cpp`; Graphite Metal.
- `Make_TextureImage_Subset_Test` — `tests/graphite/ImageProviderTest.cpp`; Graphite Metal.
- `MakeColorSpace_Test` — `tests/graphite/ImageProviderTest.cpp`; Graphite Metal.
- `ImageShaderTest` — `tests/graphite/ImageShaderTest.cpp`; Graphite Metal.
- `ImageWrapTextureMipmapsTest` — `tests/graphite/ImageWrapTextureMipmapsTest.cpp`; Graphite Metal.
- `InnerFillTest` — `tests/graphite/InnerFillTest.cpp`; Graphite Metal.
- `OptimizeForOpacity` — `tests/graphite/InnerFillTest.cpp`; Graphite Metal.
- `KeyWithInvalidCodeSnippetIDTest` — `tests/graphite/KeyTest.cpp`; Graphite Metal.
- `KeyEqualityChecksSnippetID` — `tests/graphite/KeyTest.cpp`; Graphite Metal.
- `ShaderInfoDetectsFixedFunctionBlend` — `tests/graphite/KeyTest.cpp`; Graphite Metal.
- `MtlBackendTextureTest` — `tests/graphite/MtlBackendTextureTest.cc`; Graphite Metal.
- `MultisampleRetainTest` — `tests/graphite/MultisampleTest.cpp`; Graphite Metal.
- `MultisampleClearThenLoad` — `tests/graphite/MultisampleTest.cpp`; Graphite Metal.
- `MutableImagesTest` — `tests/graphite/MutableImagesTest.cpp`; Graphite Metal.
- `NotifyInUseTestSnapshot` — `tests/graphite/NotifyInUseTest.cpp`; Graphite Metal.
- `NotifyInUseTestAsImage` — `tests/graphite/NotifyInUseTest.cpp`; Graphite Metal.
- `NotifyInUseTestLayer` — `tests/graphite/NotifyInUseTest.cpp`; Graphite Metal.
- `PersistentPipelineStorageTest` — `tests/graphite/PersistentPipelineStorageTest.cpp`; Graphite Metal.
- `PipelineCallbackTest_Deprecated_Normal` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Metal.
- `PipelineCallbackTest_Normal` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Metal.
- `PipelineCallbackTest_Deprecated_Precompile` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Metal.
- `PipelineCallbackTest_Precompile` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Metal.
- `PipelineCallbackTest_UniqueHash` — `tests/graphite/PipelineCallbackTest.cpp`; Graphite Metal.
- `PipelineDataCacheTest` — `tests/graphite/PipelineDataCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest1` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest2` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest4` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest5` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest6` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest7` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest8` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest9` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `ProxyCacheTest10` — `tests/graphite/ProxyCacheTest.cpp`; Graphite Metal.
- `Shader_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Metal.
- `ColorFilter_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Metal.
- `ShaderUniforms_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Metal.
- `ColorFilterUniforms_FindOrCreateSnippetForRuntimeEffect` — `tests/graphite/RTEffectTest.cpp`; Graphite Metal.
- `ImageAsyncReadPixelsGraphite` — `tests/graphite/ReadWritePixelsGraphiteTest.cpp`; Graphite Metal.
- `SurfaceAsyncReadPixelsGraphite` — `tests/graphite/ReadWritePixelsGraphiteTest.cpp`; Graphite Metal.
- `ImageAsyncRescaleReadPixelsOriginGraphite` — `tests/graphite/ReadWritePixelsGraphiteTest.cpp`; Graphite Metal.
- `RecorderDevicePtrTest` — `tests/graphite/RecorderTest.cpp`; Graphite Metal.
- `RecorderOrderingTest` — `tests/graphite/RecorderTest.cpp`; Graphite Metal.
- `NullRecordingInsertTest` — `tests/graphite/RecorderTest.cpp`; Graphite Metal.
- `InsertRecordingPendingCountsTest` — `tests/graphite/RecorderTest.cpp`; Graphite Metal.
- `RecordingOrderTest_Graphite` — `tests/graphite/RecordingOrderTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestClear` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestDraw` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestWritePixels` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestWritePixelsOffscreen` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestDrawWithClip` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestNegativeClip` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestWritePixelsWithClip` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestMipmapped` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestMipmappedWritePixels` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestTwoCanvases` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `RecordingSurfacesTestUnnecessarySurface` — `tests/graphite/RecordingSurfacesTest.cpp`; Graphite Metal.
- `StorageContextAlignmentTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextPaddingAlignmentTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextAppendVertexTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextMultipleRenderStepsTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextLCMVariantsTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextVertexOnlyAndResetTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextMultiStopGradientTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextFallbackStridedCopyTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `StorageContextCapacityLimitsTest` — `tests/graphite/StorageContextTest.cpp`; Graphite Metal.
- `SubmitWithFinishProc_PendingCommandBuffer` — `tests/graphite/SubmitWithFinishProcTest.cpp`; Graphite Metal.
- `SubmitWithFinishProc_NoPendingCommandBuffer` — `tests/graphite/SubmitWithFinishProcTest.cpp`; Graphite Metal.
- `TextureFallbackScalarsAndBitcastsTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackAlignmentAndPaddingTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackMatricesTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackVectorBitcastsTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackMultiInstanceMathTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackStressMixedTypesAndAlignmentTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackStressBackToBackComplexStructuresTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFallbackMultiStopGradientsDrawTest` — `tests/graphite/TextureFallbackTest.cpp`; Graphite Metal.
- `TextureFormatTest` — `tests/graphite/TextureFormatTest.cpp`; Graphite Metal.
- `GraphiteTextureProxyTest` — `tests/graphite/TextureProxyTest.cpp`; Graphite Metal.
- `GraphiteTextureTooLargeTest` — `tests/graphite/TextureProxyTest.cpp`; Graphite Metal.
- `GraphiteLazyTextureInvalidDimensions` — `tests/graphite/TextureProxyTest.cpp`; Graphite Metal.
- `UpdateImageBackendTextureTest` — `tests/graphite/UpdateBackendTextureTest.cpp`; Graphite Metal.
- `UpdateBackendTextureFinishedProcTest` — `tests/graphite/UpdateBackendTextureTest.cpp`; Graphite Metal.
- `UploadBufferManagerTest` — `tests/graphite/UploadBufferManagerTest.cpp`; Graphite Metal.
- `crbug_513836996` — `tests/graphite/crbug_513836996.cpp`; Graphite Metal.
- `ChromePrecompileTest` — `tests/graphite/precompile/ChromePrecompileTest.cpp`; Graphite Metal.
- `CombinationBuilderTest` — `tests/graphite/precompile/CombinationBuilderTest.cpp`; Graphite Metal.
- `PaintParamsKeyTestReduced` — `tests/graphite/precompile/PaintParamsKeyTest.cpp`; Graphite Metal.
- `PaintParamsKeyTest` — `tests/graphite/precompile/PaintParamsKeyTest.cpp`; Graphite Metal.
- `PipelineManagerEarlyExitTest_1` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Metal.
- `PipelineManagerEarlyExitTest_2` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Metal.
- `PipelineManagerEarlyExitTest_3` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Metal.
- `PipelineManagerEarlyExitTest_4` — `tests/graphite/precompile/PipelineManagerEarlyExit.cpp`; Graphite Metal.
- `PipelineManagerThreadedTest_1` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Metal.
- `PipelineManagerThreadedTest_2` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Metal.
- `PipelineManagerThreadedTest_3` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Metal.
- `PipelineManagerThreadedTest_4` — `tests/graphite/precompile/PipelineManagerThreaded.cpp`; Graphite Metal.
- `PrecompileStatsTest` — `tests/graphite/precompile/PrecompileStatsTest.cpp`; Graphite Metal.
- `ThreadedPipelinePrecompileTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Metal.
- `ThreadedPipelinePrecompileCompileTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Metal.
- `ThreadedPipelineCompilePurgingTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Metal.
- `ThreadedPipelinePrecompilePurgingTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Metal.
- `ThreadedPipelinePrecompileCompilePurgingTest` — `tests/graphite/precompile/ThreadedPrecompileTest.cpp`; Graphite Metal.
- `UserDefinedStableKeyTest` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Metal.
- `UserDefinedStableKeyTest_Duplicates` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Metal.
- `UserDefinedStableKeyTest_Nullptrs` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Metal.
- `UserDefinedStableKeyTest_Overflow` — `tests/graphite/precompile/UserdefinedStableKeyTest.cpp`; Graphite Metal.
- `SparseStrips_AtlasDimensionsAndStride` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_AtlasDynamicGrowthAndDoubling` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_AtlasNullBufferAndNullCaps` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_AtlasStraddleToNullBuffer` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_AtlasEndCapAtomicity` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_AtlasFlipFlopAtFlush` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_AtlasStraddleAndMemmove` — `tests/graphite/sparse_strips/AtlasTests.cpp`; Graphite Metal.
- `SparseStrips_CoverageScalar_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_CoverageSIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_CoverageSIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_CullingSimplification_SIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_CullingSimplification_SIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_CullingSimplification_Scalar_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_CulledGeometry_SIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_CulledGeometry_SIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_CulledGeometry_Scalar_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_StripGeneratorHistogramReuse` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_SKP_SIMD_4x4` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.
- `SparseStrips_Coverage_SKP_SIMD_8x8` — `tests/graphite/sparse_strips/CoverageTest.cpp`; Graphite Metal.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=36`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M36 ACCEPTED may the user hand Deepseek M37.
