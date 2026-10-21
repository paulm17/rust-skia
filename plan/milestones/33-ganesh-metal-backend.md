# Milestone 33: Ganesh Metal backend

## Outcome and entry gate

Implement Ganesh Metal buffers, textures, pipelines, shader compilation, command encoding, render targets, readback and synchronization. Bind Apple Metal APIs through minimal FFI; keep Skia resource policy and rendering decisions in Rust.

Begin implementation only after milestone 32 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement Ganesh Metal buffers, textures, pipelines, shader compilation, command encoding, render targets, readback and synchronization. Bind Apple Metal APIs through minimal FFI; keep Skia resource policy and rendering decisions in Rust.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/gpu/ganesh/mtl/GrMtlBackendContext.h` — port to Rust.
- `include/gpu/ganesh/mtl/GrMtlBackendSemaphore.h` — port to Rust.
- `include/gpu/ganesh/mtl/GrMtlBackendSurface.h` — port to Rust.
- `include/gpu/ganesh/mtl/GrMtlDirectContext.h` — port to Rust.
- `include/gpu/ganesh/mtl/GrMtlTypes.h` — port to Rust.
- `include/gpu/ganesh/mtl/SkSurfaceMetal.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlAttachment.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlAttachment.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlBackendSemaphore.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlBackendSurface.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlBuffer.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlBuffer.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlCaps.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlCaps.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlCommandBuffer.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlCommandBuffer.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlCppUtil.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlDepthStencil.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlDepthStencil.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlDirectContext.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlFramebuffer.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlFramebuffer.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlGpu.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlGpu.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlOpsRenderPass.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlOpsRenderPass.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipeline.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipelineState.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipelineState.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipelineStateBuilder.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipelineStateBuilder.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipelineStateDataManager.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlPipelineStateDataManager.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlRenderCommandEncoder.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlRenderTarget.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlRenderTarget.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlResourceProvider.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlResourceProvider.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlSampler.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlSampler.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlSemaphore.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlSemaphore.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTexture.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTexture.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTextureRenderTarget.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTextureRenderTarget.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTrampoline.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTrampoline.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTypesPriv.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlTypesPriv.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlUniformHandler.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlUniformHandler.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlUtil.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlUtil.mm` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlVaryingHandler.h` — port to Rust.
- `src/gpu/ganesh/mtl/GrMtlVaryingHandler.mm` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/gpu/ganesh/mtl/BUILD.bazel`.
- `src/gpu/ganesh/mtl/BUILD.bazel`.
- `tools/ganesh/mtl/BUILD.bazel`.

## Codex: acceptance-test contract

Translate Metal-specific tests and run all applicable Ganesh rendering tests on actual Metal hardware. Verify readback, MSAA, texture formats, context abandon, asynchronous completion and resource destruction. Missing hardware is BLOCKED, never PASS.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/AdvancedBlendTest.cpp`.
- `tests/ApplyGammaTest.cpp`.
- `tests/AtlasOobTest.cpp`.
- `tests/BigImageTest.cpp`.
- `tests/BlendTest.cpp`.
- `tests/BlurTest.cpp`.
- `tests/BulkRectTest.cpp`.
- `tests/CanvasTest.cpp`.
- `tests/ClearTest.cpp`.
- `tests/ColorFilterTest.cpp`.
- `tests/CompressedBackendAllocationTest.cpp`.
- `tests/CopySurfaceTest.cpp`.
- `tests/DMSAATest.cpp`.
- `tests/DashPathEffectTestGanesh.cpp`.
- `tests/DefaultPathRendererTest.cpp`.
- `tests/DeferredDisplayListTest.cpp`.
- `tests/DeviceTest.cpp`.
- `tests/DirectMaskLimitTest.cpp`.
- `tests/DrawOpAtlasTest.cpp`.
- `tests/ExtendedSkColorTypeTests.cpp`.
- `tests/F16DrawTest.cpp`.
- `tests/FilterResultTest.cpp`.
- `tests/FloatingPointTextureTest.cpp`.
- `tests/GaneshContextRecorderTest.cpp`.
- `tests/GpuDrawPathTest.cpp`.
- `tests/GrClipStackTest.cpp`.
- `tests/GrContextFactoryTest.cpp`.
- `tests/GrFinishedFlushTest.cpp`.
- `tests/GrGpuBufferTest.cpp`.
- `tests/GrMeshTest.cpp`.
- `tests/GrMipMappedTest.cpp`.
- `tests/GrOpListFlushTest.cpp`.
- `tests/GrPipelineDynamicStateTest.cpp`.
- `tests/GrSubmittedFlushTest.cpp`.
- `tests/GrSurfaceResolveTest.cpp`.
- `tests/GrSurfaceTest.cpp`.
- `tests/GrTextBlobTest.cpp`.
- `tests/GrTextureMipMapInvalidationTest.cpp`.
- `tests/GrThreadSafeCacheTest.cpp`.
- `tests/GrUniqueProxyTest.cpp`.
- `tests/GradientTest.cpp`.
- `tests/ImageFilterCacheTest.cpp`.
- `tests/ImageFilterTest.cpp`.
- `tests/ImageIsOpaqueTest.cpp`.
- `tests/ImageNewShaderTest.cpp`.
- `tests/ImageTest.cpp`.
- `tests/LazyStencilAttachmentTest.cpp`.
- `tests/MatrixColorFilterTest.cpp`.
- `tests/MtlBackendAllocationTest.cc`.
- `tests/MtlCopySurfaceTest.cc`.
- `tests/MultiPictureDocumentTest.cpp`.
- `tests/PinnedImageTest.cpp`.
- `tests/PremulAlphaRoundTripTest.cpp`.
- `tests/PrimitiveProcessorTest.cpp`.
- `tests/ProcessorTest.cpp`.
- `tests/PromiseImageTest.cpp`.
- `tests/ProtectedTest.cpp`.
- `tests/ProxyConversionTest.cpp`.
- `tests/ProxyRefTest.cpp`.
- `tests/ProxyTest.cpp`.
- `tests/ReadWritePixelsGpuTest.cpp`.
- `tests/RepeatedClippedBlurTest.cpp`.
- `tests/ResourceAllocatorTest.cpp`.
- `tests/ResourceCacheTest.cpp`.
- `tests/RuntimeBlendTest.cpp`.
- `tests/SRGBReadWritePixelsTest.cpp`.
- `tests/SaveLayerOriginTest.cpp`.
- `tests/ShaderImageFilterTest.cpp`.
- `tests/ShaderTest.cpp`.
- `tests/SimplifyPaintTest.cpp`.
- `tests/SkColorSpaceXformStepsTest.cpp`.
- `tests/SkRemoteGlyphCacheTest.cpp`.
- `tests/SkRuntimeEffectTest.cpp`.
- `tests/SkSLCross.cpp`.
- `tests/SkSLES2ConformanceTest.cpp`.
- `tests/SkSLTest.cpp`.
- `tests/Skbug12214.cpp`.
- `tests/Skbug5221.cpp`.
- `tests/Skbug6653.cpp`.
- `tests/SlugTest.cpp`.
- `tests/SmallPathRendererTest.cpp`.
- `tests/SpecialImageTest.cpp`.
- `tests/SrcSrcOverBatchTest.cpp`.
- `tests/StencilClearTest.cpp`.
- `tests/SurfaceDrawContextTest.cpp`.
- `tests/SurfaceSemaphoreTest.cpp`.
- `tests/SurfaceTest.cpp`.
- `tests/TestTest.cpp`.
- `tests/TextBlobCacheTest.cpp`.
- `tests/TextureOpTest.cpp`.
- `tests/TextureProxyTest.cpp`.
- `tests/TextureSizeTest.cpp`.
- `tests/TextureStripAtlasManagerTest.cpp`.
- `tests/TransferPixelsTest.cpp`.
- `tests/TriangulatingPathRendererTests.cpp`.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp`.
- `tests/WritePixelsTest.cpp`.
- `tests/crbug_500172224.cpp`.
- `tools/ganesh/mtl/MtlTestContext.h`.
- `tools/ganesh/mtl/MtlTestContext.mm`.

### Original test registrations assigned to this gate

- `MtlBackendAllocationTest` — `tests/MtlBackendAllocationTest.cc:50`; Ganesh; `DEF_GANESH_TEST_FOR_METAL_CONTEXT`.
- `MtlCopySurfaceTest` — `tests/MtlCopySurfaceTest.cc:21`; Ganesh; `DEF_GANESH_TEST_FOR_METAL_CONTEXT`.

### Backend variants required at this gate

These variants must be translated/run here even if their cross-backend consolidation is assigned to M38. Apply the original context filter; never execute an ineligible backend just to inflate the count.

- `AdvancedBlendTest` — `tests/AdvancedBlendTest.cpp`; Ganesh Metal.
- `ApplyGamma` — `tests/ApplyGammaTest.cpp`; Ganesh Metal.
- `Atlas_Oob_ganesh` — `tests/AtlasOobTest.cpp`; Ganesh Metal.
- `BigImageTest_Ganesh` — `tests/BigImageTest.cpp`; Ganesh Metal.
- `TiledDrawCacheTest_Ganesh` — `tests/BigImageTest.cpp`; Ganesh Metal.
- `BlendRequiringDstReadWithLargeCoordinates` — `tests/BlendTest.cpp`; Ganesh Metal.
- `SmallBoxBlurBug` — `tests/BlurTest.cpp`; Ganesh Metal.
- `BlurMaskBiggerThanDest` — `tests/BlurTest.cpp`; Ganesh Metal.
- `BlurDegenerateAffineFuzzer` — `tests/BlurTest.cpp`; Ganesh Metal.
- `BulkFillRectTest` — `tests/BulkRectTest.cpp`; Ganesh Metal.
- `BulkTextureRectTest` — `tests/BulkRectTest.cpp`; Ganesh Metal.
- `TestManyDrawsGanesh` — `tests/CanvasTest.cpp`; Ganesh Metal.
- `ClearOp` — `tests/ClearTest.cpp`; Ganesh Metal.
- `FullScreenClearWithLayers` — `tests/ClearTest.cpp`; Ganesh Metal.
- `ComposeFailureWithInputElision` — `tests/ColorFilterTest.cpp`; Ganesh Metal.
- `CompressedBackendAllocationTest` — `tests/CompressedBackendAllocationTest.cpp`; Ganesh Metal.
- `CopySurface` — `tests/CopySurfaceTest.cpp`; Ganesh Metal.
- `DMSAA_preserve_contents` — `tests/DMSAATest.cpp`; Ganesh Metal.
- `DMSAA_dst_read` — `tests/DMSAATest.cpp`; Ganesh Metal.
- `DMSAA_aa_dst_read_after_dmsaa` — `tests/DMSAATest.cpp`; Ganesh Metal.
- `DMSAA_dst_read_with_existing_barrier` — `tests/DMSAATest.cpp`; Ganesh Metal.
- `DMSAA_dual_source_blend_disable` — `tests/DMSAATest.cpp`; Ganesh Metal.
- `DashPathEffectTest_2PiInterval` — `tests/DashPathEffectTestGanesh.cpp`; Ganesh Metal.
- `DefaultPathRendererTest` — `tests/DefaultPathRendererTest.cpp`; Ganesh Metal.
- `DDLOperatorEqTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLSurfaceCharacterizationTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLNonTextureabilityTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLMakeRenderTargetTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLWrapBackendTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLInvalidRecorder` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLCreateCharacterizationFailures` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLSkSurfaceFlush` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `DDLMultipleDDLs` — `tests/DeferredDisplayListTest.cpp`; Ganesh Metal.
- `SpecialImage_GPUDevice` — `tests/DeviceTest.cpp`; Ganesh Metal.
- `DirectMaskLimitTest_Ganesh` — `tests/DirectMaskLimitTest.cpp`; Ganesh Metal.
- `BasicDrawOpAtlas` — `tests/DrawOpAtlasTest.cpp`; Ganesh Metal.
- `GrAtlasTextOpPreparation` — `tests/DrawOpAtlasTest.cpp`; Ganesh Metal.
- `ExtendedSkColorTypeTests_gpu` — `tests/ExtendedSkColorTypeTests.cpp`; Ganesh Metal.
- `F16DrawTest_Ganesh` — `tests/F16DrawTest.cpp`; Ganesh Metal.
- `FilterResult_ganesh_` — `tests/FilterResultTest.cpp`; Ganesh Metal.
- `HalfFloatAlphaTextureTest` — `tests/FloatingPointTextureTest.cpp`; Ganesh Metal.
- `HalfFloatRGBATextureTest` — `tests/FloatingPointTextureTest.cpp`; Ganesh Metal.
- `CPUSurface_UsesGaneshContextAndRasterRecorderToDraw_DrawsPixels` — `tests/GaneshContextRecorderTest.cpp`; Ganesh Metal.
- `ImageMakeColorSpace_GaneshImageWithContext_Success` — `tests/GaneshContextRecorderTest.cpp`; Ganesh Metal.
- `ImageMakeColorSpace_GaneshImageWithRecorder_Success` — `tests/GaneshContextRecorderTest.cpp`; Ganesh Metal.
- `ImageMakeScaled_GaneshImageWithRecorder_Success` — `tests/GaneshContextRecorderTest.cpp`; Ganesh Metal.
- `CanvasBaseRecorder_GaneshBasedCanvas_IsLinkedToDirectContext` — `tests/GaneshContextRecorderTest.cpp`; Ganesh Metal.
- `GrDrawCollapsedPath` — `tests/GpuDrawPathTest.cpp`; Ganesh Metal.
- `PathTest_CrBug1232834` — `tests/GpuDrawPathTest.cpp`; Ganesh Metal.
- `StrokeCircle_Bug356182429` — `tests/GpuDrawPathTest.cpp`; Ganesh Metal.
- `ClipStack_SWMask` — `tests/GrClipStackTest.cpp`; Ganesh Metal.
- `ClipStack_MixedAA` — `tests/GrClipStackTest.cpp`; Ganesh Metal.
- `GrContextDump` — `tests/GrContextFactoryTest.cpp`; Ganesh Metal.
- `FlushFinishedProcTest` — `tests/GrFinishedFlushTest.cpp`; Ganesh Metal.
- `FinishedAsyncProcWhenAbandonedTest` — `tests/GrFinishedFlushTest.cpp`; Ganesh Metal.
- `GrGpuBufferTransferTest` — `tests/GrGpuBufferTest.cpp`; Ganesh Metal.
- `GrGpuBufferUpdateDataTest` — `tests/GrGpuBufferTest.cpp`; Ganesh Metal.
- `GrMeshTest` — `tests/GrMeshTest.cpp`; Ganesh Metal.
- `GrMeshTest` — `tests/GrMeshTest.cpp`; Ganesh Metal.
- `GrWrappedMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh Metal.
- `GrBackendTextureImageMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh Metal.
- `GrImageSnapshotMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh Metal.
- `Gr1x1TextureMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh Metal.
- `OpsTaskFlushCount` — `tests/GrOpListFlushTest.cpp`; Ganesh Metal.
- `GrPipelineDynamicStateTest` — `tests/GrPipelineDynamicStateTest.cpp`; Ganesh Metal.
- `FlushSubmittedProcTest` — `tests/GrSubmittedFlushTest.cpp`; Ganesh Metal.
- `SurfaceResolveTest` — `tests/GrSurfaceResolveTest.cpp`; Ganesh Metal.
- `NonmippedDrawBeforeMippedDraw` — `tests/GrSurfaceResolveTest.cpp`; Ganesh Metal.
- `SurfaceResolveAfterFailedFlush` — `tests/GrSurfaceResolveTest.cpp`; Ganesh Metal.
- `GrSurfaceRenderability` — `tests/GrSurfaceTest.cpp`; Ganesh Metal.
- `ReadOnlyTexture` — `tests/GrSurfaceTest.cpp`; Ganesh Metal.
- `GrTextBlobScaleAnimation` — `tests/GrTextBlobTest.cpp`; Ganesh Metal.
- `GrTextBlobMoveAround` — `tests/GrTextBlobTest.cpp`; Ganesh Metal.
- `GrTextureMipMapInvalidationTest` — `tests/GrTextureMipMapInvalidationTest.cpp`; Ganesh Metal.
- `ReimportImageTextureWithMipLevels` — `tests/GrTextureMipMapInvalidationTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache1View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache1Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache2View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache2Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache3View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache3Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache4View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache4Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache4_5View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache4_5Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache4_75View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache4_75Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache5View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache5Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache6View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache6Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache7View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache7Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache8View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache8Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache9View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache9Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache10View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache11View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache11Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache12View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache12Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache13View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache13Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache14` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache15View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache15Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrThreadSafeCache16Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh Metal.
- `GrUniqueProxyTest` — `tests/GrUniqueProxyTest.cpp`; Ganesh Metal.
- `TestSweepGradientZeroXGanesh` — `tests/GradientTest.cpp`; Ganesh Metal.
- `TestManyStopLinearHardstopsGanesh` — `tests/GradientTest.cpp`; Ganesh Metal.
- `ImageFilterCache_ImageBackedGPU` — `tests/ImageFilterCacheTest.cpp`; Ganesh Metal.
- `ImageFilterCache_GPUBacked` — `tests/ImageFilterCacheTest.cpp`; Ganesh Metal.
- `ImageFilterNegativeBlurSigma_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `MorphologyFilterRadiusWithMirrorCTM_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterZeroBlurSigma_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterFailAffectsTransparentBlack_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterMergeResultSize_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterMatrixConvolutionBigKernel_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterCropRect_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterClippedPictureImageFilter_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ComposedImageFilterOffset_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ComposedImageFilterBounds_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterMakeWithFilter_Ganesh` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterHugeBlur_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `XfermodeImageFilterCroppedInput_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageFilterBlurLargeImage_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh Metal.
- `ImageIsOpaqueTest_Gpu` — `tests/ImageIsOpaqueTest.cpp`; Ganesh Metal.
- `ImageNewShader_GPU` — `tests/ImageNewShaderTest.cpp`; Ganesh Metal.
- `ImageEncode_Gpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `SkImage_Ganesh2Cpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `SkImage_makeTextureImage` — `tests/ImageTest.cpp`; Ganesh Metal.
- `SkImage_makeNonTextureImage` — `tests/ImageTest.cpp`; Ganesh Metal.
- `GrContext_colorTypeSupportedAsImage` — `tests/ImageTest.cpp`; Ganesh Metal.
- `UnpremulTextureImage` — `tests/ImageTest.cpp`; Ganesh Metal.
- `ImageReadPixels_Gpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `ImageLegacyBitmap_Gpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `ImagePeek_Gpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `ImageBackendAccessAbandoned_Gpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `ImageScalePixels_Gpu` — `tests/ImageTest.cpp`; Ganesh Metal.
- `ImageFlush` — `tests/ImageTest.cpp`; Ganesh Metal.
- `crbug_1271431` — `tests/LazyStencilAttachmentTest.cpp`; Ganesh Metal.
- `MatrixColorFilter_TransparentBlack` — `tests/MatrixColorFilterTest.cpp`; Ganesh Metal.
- `MtlBackendAllocationTest` — `tests/MtlBackendAllocationTest.cc`; Ganesh Metal.
- `MtlCopySurfaceTest` — `tests/MtlCopySurfaceTest.cc`; Ganesh Metal.
- `SkMultiPictureDocument_AHardwarebuffer` — `tests/MultiPictureDocumentTest.cpp`; Ganesh Metal.
- `PinnedImageTest` — `tests/PinnedImageTest.cpp`; Ganesh Metal.
- `PinnedImageTest_AsGaneshView` — `tests/PinnedImageTest.cpp`; Ganesh Metal.
- `PinnedImageTest_AsFragmentProcessor` — `tests/PinnedImageTest.cpp`; Ganesh Metal.
- `PremulAlphaRoundTrip_Gpu` — `tests/PremulAlphaRoundTripTest.cpp`; Ganesh Metal.
- `VertexAttributeCount` — `tests/PrimitiveProcessorTest.cpp`; Ganesh Metal.
- `ProcessorRefTest` — `tests/ProcessorTest.cpp`; Ganesh Metal.
- `PromiseImageTest` — `tests/PromiseImageTest.cpp`; Ganesh Metal.
- `PromiseImageTextureFullCache` — `tests/PromiseImageTest.cpp`; Ganesh Metal.
- `PromiseImageNullFulfill` — `tests/PromiseImageTest.cpp`; Ganesh Metal.
- `Protected_SmokeTest` — `tests/ProtectedTest.cpp`; Ganesh Metal.
- `Protected_readPixelsFromSurfaces` — `tests/ProtectedTest.cpp`; Ganesh Metal.
- `Protected_CopyOnWrite` — `tests/ProtectedTest.cpp`; Ganesh Metal.
- `Protected_asyncRescaleAndReadPixelsFromSurfaces` — `tests/ProtectedTest.cpp`; Ganesh Metal.
- `Protected_pictureShader` — `tests/ProtectedTest.cpp`; Ganesh Metal.
- `PreinstantiatedProxyConversionTest` — `tests/ProxyConversionTest.cpp`; Ganesh Metal.
- `DefferredProxyConversionTest` — `tests/ProxyConversionTest.cpp`; Ganesh Metal.
- `ProxyRefTest` — `tests/ProxyRefTest.cpp`; Ganesh Metal.
- `DeferredProxyTest` — `tests/ProxyTest.cpp`; Ganesh Metal.
- `WrappedProxyTest` — `tests/ProxyTest.cpp`; Ganesh Metal.
- `ZeroSizedProxyTest` — `tests/ProxyTest.cpp`; Ganesh Metal.
- `SurfaceContextReadPixels` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `ReadPixels_InvalidRowBytes_Gpu` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `WritePixels_InvalidRowBytes_Gpu` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `SurfaceAsyncReadPixels` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `ImageAsyncReadPixels_NonRenderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `ImageAsyncReadPixels_NonRenderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `ImageAsyncReadPixels_Renderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `ImageAsyncReadPixels_Renderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `SurfaceContextWritePixels_NonRenderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `SurfaceContextWritePixels_NonRenderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `SurfaceContextWritePixels_Renderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `SurfaceContextWritePixels_Renderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `SurfaceContextWritePixelsMipped` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `ReadPixelsIntermediateFailedFlush` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `WritePixelsFailedFlushLeaksScratch` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh Metal.
- `RepeatedClippedBlurTest` — `tests/RepeatedClippedBlurTest.cpp`; Ganesh Metal.
- `ResourceAllocatorTest` — `tests/ResourceAllocatorTest.cpp`; Ganesh Metal.
- `ResourceAllocatorStressTest` — `tests/ResourceAllocatorTest.cpp`; Ganesh Metal.
- `ResourceAllocatorMemoryBudgetTest` — `tests/ResourceAllocatorTest.cpp`; Ganesh Metal.
- `ResourceCacheCache` — `tests/ResourceCacheTest.cpp`; Ganesh Metal.
- `ResourceCacheStencilBuffers` — `tests/ResourceCacheTest.cpp`; Ganesh Metal.
- `ResourceCacheWrappedResources` — `tests/ResourceCacheTest.cpp`; Ganesh Metal.
- `ResourceMessagesAfterAbandon` — `tests/ResourceCacheTest.cpp`; Ganesh Metal.
- `GPUMemorySize` — `tests/ResourceCacheTest.cpp`; Ganesh Metal.
- `PurgeToMakeHeadroom` — `tests/ResourceCacheTest.cpp`; Ganesh Metal.
- `SkRuntimeBlender_Ganesh` — `tests/RuntimeBlendTest.cpp`; Ganesh Metal.
- `SRGBReadWritePixels` — `tests/SRGBReadWritePixelsTest.cpp`; Ganesh Metal.
- `SaveLayerOrigin` — `tests/SaveLayerOriginTest.cpp`; Ganesh Metal.
- `SkRuntimeShaderImageFilter_GPU` — `tests/ShaderImageFilterTest.cpp`; Ganesh Metal.
- `ShaderTestNestedBlendsGanesh` — `tests/ShaderTest.cpp`; Ganesh Metal.
- `SimplifyPaintTest_Ganesh` — `tests/SimplifyPaintTest.cpp`; Ganesh Metal.
- `SkColorSpaceXform_Ganesh` — `tests/SkColorSpaceXformStepsTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_StrikeSerialization` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_StrikeSerializationSlug` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_StrikeSerializationSlugForcePath` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_SlugSerialization` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_SlugSerialization_LargeOffset` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_SubRunBoundsReconstruction` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_ReleaseTypeFace` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_DrawTextAsPath` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_DrawTextAsMaskWithPathFallback` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_DrawTextXY` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_DrawTextAsDFT` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_CacheMissReporting` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_TypefaceWithNoPaths` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_TypefaceWithPaths_MaskThenPath` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRemoteGlyphCache_TypefaceWithPaths_PathThenMask` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh Metal.
- `SkRuntimeEffectSimple_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `SkRuntimeEffectObeysCapabilities_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `SkRuntimeColorFilterReturningInvalidAlpha_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `SkRuntimeEffect_Blender_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `SkRuntimeStructNameReuse_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `SkRuntimeShader_TransformedCoords_Ganesh` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `GrSkSLFP_Specialized` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `GrSkSLFP_UniformArray` — `tests/SkRuntimeEffectTest.cpp`; Ganesh Metal.
- `SkSLCross` — `tests/SkSLCross.cpp`; Ganesh Metal.
- `SkSL_ES2Conformance_Pass_GPU` — `tests/SkSLES2ConformanceTest.cpp`; Ganesh Metal.
- `SkSL` — `tests/SkSLTest.cpp`; Ganesh Metal.
- `flags` — `tests/SkSLTest.cpp`; Ganesh Metal.
- `skbug12214` — `tests/Skbug12214.cpp`; Ganesh Metal.
- `skbug5221_GPU` — `tests/Skbug5221.cpp`; Ganesh Metal.
- `skbug6653` — `tests/Skbug6653.cpp`; Ganesh Metal.
- `Slug_empty` — `tests/SlugTest.cpp`; Ganesh Metal.
- `Slug_b520113415` — `tests/SlugTest.cpp`; Ganesh Metal.
- `Slug_b520571816` — `tests/SlugTest.cpp`; Ganesh Metal.
- `Slug_draw_then_serialize` — `tests/SlugTest.cpp`; Ganesh Metal.
- `Slug_draw_then_serialize_PathSubRun` — `tests/SlugTest.cpp`; Ganesh Metal.
- `Slug_draw_then_serialize_DrawableSubRun` — `tests/SlugTest.cpp`; Ganesh Metal.
- `SmallPathRenderer_crbug505876830` — `tests/SmallPathRendererTest.cpp`; Ganesh Metal.
- `SpecialImage_Gpu` — `tests/SpecialImageTest.cpp`; Ganesh Metal.
- `SrcSrcOverBatchTest` — `tests/SrcSrcOverBatchTest.cpp`; Ganesh Metal.
- `StencilClearTest` — `tests/StencilClearTest.cpp`; Ganesh Metal.
- `StencilClearTest_PreservedAcrossSplit` — `tests/StencilClearTest.cpp`; Ganesh Metal.
- `SurfaceDrawContextTest` — `tests/SurfaceDrawContextTest.cpp`; Ganesh Metal.
- `EmptySurfaceSemaphoreTest` — `tests/SurfaceSemaphoreTest.cpp`; Ganesh Metal.
- `SurfaceEmpty_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `GrContext_colorTypeSupportedAsSurface` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `GrContext_maxSurfaceSamplesForColorType` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceCanvasPeek_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceSnapshotAlphaType_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceBackendSurfaceAccessCopyOnWrite_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceBackendHandleAccessIDs_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceAbandonPostFlush_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceBackendAccessAbandoned_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceCopyOnWrite_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceWriteableAfterSnapshotRelease_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceCRBug263329_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfacepeekTexture_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceBudget` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceNoCanvas_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfacePartialDraw_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `SurfaceWrappedWithRelease_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `DrawSnapshotBackIntoWappedSurface` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `ReplaceSurfaceBackendTexture` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `OverdrawSurface_Gpu` — `tests/SurfaceTest.cpp`; Ganesh Metal.
- `TestGpuAllContexts` — `tests/TestTest.cpp`; Ganesh Metal.
- `TestGpuRenderingContexts` — `tests/TestTest.cpp`; Ganesh Metal.
- `TestGpuAllContextsWithTrueCondition` — `tests/TestTest.cpp`; Ganesh Metal.
- `TestGpuAllContextsWithFalseCondition` — `tests/TestTest.cpp`; Ganesh Metal.
- `TestGpuRenderingContextsWithTrueCondition` — `tests/TestTest.cpp`; Ganesh Metal.
- `TestGpuRenderingContextsWithFalseCondition` — `tests/TestTest.cpp`; Ganesh Metal.
- `TextBlobIntegerOverflowTest` — `tests/TextBlobCacheTest.cpp`; Ganesh Metal.
- `TextBlobJaggedGlyph` — `tests/TextBlobCacheTest.cpp`; Ganesh Metal.
- `TextBlobSmoothScroll` — `tests/TextBlobCacheTest.cpp`; Ganesh Metal.
- `TextBlob_draw_releases_strike_from_cache` — `tests/TextBlobCacheTest.cpp`; Ganesh Metal.
- `TextureOpTest` — `tests/TextureOpTest.cpp`; Ganesh Metal.
- `TextureProxyTest` — `tests/TextureProxyTest.cpp`; Ganesh Metal.
- `ImageSizeTest_Ganesh` — `tests/TextureSizeTest.cpp`; Ganesh Metal.
- `TextureStripAtlasManagerGradientTest` — `tests/TextureStripAtlasManagerTest.cpp`; Ganesh Metal.
- `TextureStripAtlasManagerColorFilterTest` — `tests/TextureStripAtlasManagerTest.cpp`; Ganesh Metal.
- `TransferPixelsToTextureTest` — `tests/TransferPixelsTest.cpp`; Ganesh Metal.
- `TransferPixelsFromTextureTest` — `tests/TransferPixelsTest.cpp`; Ganesh Metal.
- `TriangulatingPathRendererTests` — `tests/TriangulatingPathRendererTests.cpp`; Ganesh Metal.
- `WrappedSurfaceCopyOnWrite` — `tests/WrappedSurfaceCopyOnWriteTest.cpp`; Ganesh Metal.
- `SkipCopyTaskTest` — `tests/WrappedSurfaceCopyOnWriteTest.cpp`; Ganesh Metal.
- `SkipOpsTaskTest` — `tests/WrappedSurfaceCopyOnWriteTest.cpp`; Ganesh Metal.
- `WritePixels_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh Metal.
- `WritePixelsMSAA_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh Metal.
- `WritePixelsNonTexture_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh Metal.
- `WritePixelsNonTextureMSAA_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh Metal.
- `WritePixelsPendingIO` — `tests/WritePixelsTest.cpp`; Ganesh Metal.
- `crbug_500172224_aa_double` — `tests/crbug_500172224.cpp`; Ganesh Metal.
- `crbug_500172224_non_aa_double` — `tests/crbug_500172224.cpp`; Ganesh Metal.
- `crbug_500172224_aa` — `tests/crbug_500172224.cpp`; Ganesh Metal.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/BackendAllocationTest.cpp` — M34.
- `tests/GrMipMappedTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=33`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M33 ACCEPTED may the user hand Deepseek M34.
