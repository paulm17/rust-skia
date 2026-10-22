# Milestone 34: Ganesh macOS OpenGL backend

## Outcome and entry gate

Implement the native macOS OpenGL/CGL context path and Ganesh GL state, textures, framebuffer, shader and synchronization logic. Track the capabilities exposed by macOS and preserve upstream exclusions for unsupported features.

Begin implementation only after milestone 33 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement the native macOS OpenGL/CGL context path and Ganesh GL state, textures, framebuffer, shader and synchronization logic. Track the capabilities exposed by macOS and preserve upstream exclusions for unsupported features.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/gpu/ganesh/gl/GrGLAssembleHelpers.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLAssembleInterface.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLBackendSurface.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLConfig.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLDirectContext.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLExtensions.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLFunctions.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLInterface.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLMakeWebGLInterface.h` — port to Rust.
- `include/gpu/ganesh/gl/GrGLTypes.h` — port to Rust.
- `include/gpu/ganesh/gl/epoxy/GrGLMakeEpoxyEGLInterface.h` — port to Rust.
- `include/gpu/ganesh/gl/mac/GrGLMakeMacInterface.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAssembleGLESInterfaceAutogen.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAssembleGLInterfaceAutogen.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAssembleHelpers.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAssembleInterface.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAssembleWebGLInterfaceAutogen.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAttachment.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLAttachment.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLBackendSurface.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLBackendSurfacePriv.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLBuffer.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLBuffer.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLCaps.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLCaps.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLContext.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLContext.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLCoreFunctions.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLDefines.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLDirectContext.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLExtensions.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLFinishCallbacks.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLFinishCallbacks.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLGLSL.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLGLSL.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLGpu.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLGpu.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLGpuProgramCache.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLInterfaceAutogen.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLMakeNativeInterface_none.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLOpsRenderPass.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLOpsRenderPass.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLProgram.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLProgram.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLProgramDataManager.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLProgramDataManager.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLRenderTarget.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLRenderTarget.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLSemaphore.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLSemaphore.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLTexture.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLTexture.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLTextureRenderTarget.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLTextureRenderTarget.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLTypesPriv.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLTypesPriv.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLUniformHandler.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLUniformHandler.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLUtil.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLUtil.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLVaryingHandler.h` — port to Rust.
- `src/gpu/ganesh/gl/GrGLVertexArray.cpp` — port to Rust.
- `src/gpu/ganesh/gl/GrGLVertexArray.h` — port to Rust.
- `src/gpu/ganesh/gl/builders/GrGLProgramBuilder.cpp` — port to Rust.
- `src/gpu/ganesh/gl/builders/GrGLProgramBuilder.h` — port to Rust.
- `src/gpu/ganesh/gl/builders/GrGLShaderStringBuilder.cpp` — port to Rust.
- `src/gpu/ganesh/gl/builders/GrGLShaderStringBuilder.h` — port to Rust.
- `src/gpu/ganesh/gl/epoxy/GrGLMakeEpoxyEGLInterface.cpp` — port to Rust.
- `src/gpu/ganesh/gl/mac/GrGLMakeNativeInterface_mac.cpp` — port to Rust.
- `src/gpu/ganesh/gl/webgl/GrGLMakeNativeInterface_webgl.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/gpu/ganesh/gl/BUILD.bazel`.
- `include/gpu/ganesh/gl/epoxy/BUILD.bazel`.
- `include/gpu/ganesh/gl/mac/BUILD.bazel`.
- `src/gpu/ganesh/gl/BUILD.bazel`.
- `src/gpu/ganesh/gl/builders/BUILD.bazel`.
- `src/gpu/ganesh/gl/epoxy/BUILD.bazel`.
- `src/gpu/ganesh/gl/mac/BUILD.bazel`.
- `src/gpu/ganesh/gl/webgl/BUILD.bazel`.
- `tools/ganesh/gl/BUILD.bazel`.
- `tools/ganesh/gl/interface/BUILD.bazel`.
- `tools/ganesh/gl/interface/gen_interface.go`.
- `tools/ganesh/gl/interface/templates.go`.
- `tools/ganesh/gl/mac/BUILD.bazel`.
- `tools/ganesh/gl/none/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate GL-specific tests and rerun generic Ganesh rendering tests with a macOS GL context. Check state reset, framebuffer lifetime, format limits, context loss/abandon and readback. Do not enable EGL/GLX/Windows/iOS context code.

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
- `tests/BackendAllocationTest.cpp`.
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
- `tests/GLBackendSurfaceTest.cpp`.
- `tests/GLFramebufferDeletionTest.cpp`.
- `tests/GaneshContextRecorderTest.cpp`.
- `tests/GpuDrawPathTest.cpp`.
- `tests/GrClipStackTest.cpp`.
- `tests/GrContextFactoryTest.cpp`.
- `tests/GrFinishedFlushTest.cpp`.
- `tests/GrGLExtensionsTest.cpp`.
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
- `tests/MultiPictureDocumentTest.cpp`.
- `tests/PinnedImageTest.cpp`.
- `tests/PremulAlphaRoundTripTest.cpp`.
- `tests/PrimitiveProcessorTest.cpp`.
- `tests/ProcessorTest.cpp`.
- `tests/ProgramsTest.cpp`.
- `tests/PromiseImageTest.cpp`.
- `tests/ProtectedTest.cpp`.
- `tests/ProxyConversionTest.cpp`.
- `tests/ProxyRefTest.cpp`.
- `tests/ProxyTest.cpp`.
- `tests/ReadWritePixelsGpuTest.cpp`.
- `tests/RectangleTextureTest.cpp`.
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
- `tests/TextureBindingsResetTest.cpp`.
- `tests/TextureOpTest.cpp`.
- `tests/TextureProxyTest.cpp`.
- `tests/TextureSizeTest.cpp`.
- `tests/TextureStripAtlasManagerTest.cpp`.
- `tests/TraceMemoryDumpTest.cpp`.
- `tests/TransferPixelsTest.cpp`.
- `tests/TriangulatingPathRendererTests.cpp`.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp`.
- `tests/WritePixelsTest.cpp`.
- `tests/crbug_500172224.cpp`.
- `tools/ganesh/gl/GLTestContext.cpp`.
- `tools/ganesh/gl/GLTestContext.h`.
- `tools/ganesh/gl/angle/GLTestContext_angle.cpp`.
- `tools/ganesh/gl/angle/GLTestContext_angle.h`.
- `tools/ganesh/gl/mac/CreatePlatformGLTestContext_mac.cpp`.
- `tools/ganesh/gl/none/CreatePlatformGLTestContext_none.cpp`.

### Original test registrations assigned to this gate

- `GLBackendAllocationTest` — `tests/BackendAllocationTest.cpp:808`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `ES2BlendWithNoTexture` — `tests/BlendTest.cpp:92`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `CharacterizationFBO0nessTest` — `tests/DeferredDisplayListTest.cpp:690`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `DDLTextureFlagsTest` — `tests/DeferredDisplayListTest.cpp:1325`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `DDLCompatibilityTest` — `tests/DeferredDisplayListTest.cpp:1377`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `GLTextureParameters` — `tests/GLBackendSurfaceTest.cpp:66`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `GLFramebufferDeletion` — `tests/GLFramebufferDeletionTest.cpp:560`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `GpuDrawPath` — `tests/GpuDrawPathTest.cpp:85`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `GrGLExtensionsTest_remove` — `tests/GrGLExtensionsTest.cpp:34`; CPU; `DEF_TEST`.
- `SkImage_NewFromTextureRelease` — `tests/ImageTest.cpp:932`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `makeBackendTexture` — `tests/ImageTest.cpp:1217`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `ProcessorOptimizationValidationTest` — `tests/ProcessorTest.cpp:595`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `ProcessorCloneTest` — `tests/ProcessorTest.cpp:957`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `GLReadPixelsUnbindPBO` — `tests/ReadWritePixelsGpuTest.cpp:1476`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `RectangleTexture` — `tests/RectangleTextureTest.cpp:155`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SurfaceClear_Gpu` — `tests/SurfaceTest.cpp:901`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SurfaceAttachStencil_Gpu` — `tests/SurfaceTest.cpp:1102`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `TextureBindingsResetTest` — `tests/TextureBindingsResetTest.cpp:48`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SkTraceMemoryDump_ownedGLBuffer` — `tests/TraceMemoryDumpTest.cpp:91`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SkTraceMemoryDump_ownedGLTexture` — `tests/TraceMemoryDumpTest.cpp:104`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SkTraceMemoryDump_unownedGLTexture` — `tests/TraceMemoryDumpTest.cpp:128`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SkTraceMemoryDump_ownedGLRenderTarget` — `tests/TraceMemoryDumpTest.cpp:156`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SkTraceMemoryDump_unownedGLRenderTarget` — `tests/TraceMemoryDumpTest.cpp:184`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.
- `SkTraceMemoryDump_ownedGLTextureRenderTarget` — `tests/TraceMemoryDumpTest.cpp:212`; Ganesh; `DEF_GANESH_TEST_FOR_GL_CONTEXT`.

### Backend variants required at this gate

These variants must be translated/run here even if their cross-backend consolidation is assigned to M38. Apply the original context filter; never execute an ineligible backend just to inflate the count.

- `AdvancedBlendTest` — `tests/AdvancedBlendTest.cpp`; Ganesh macOS OpenGL.
- `ApplyGamma` — `tests/ApplyGammaTest.cpp`; Ganesh macOS OpenGL.
- `Atlas_Oob_ganesh` — `tests/AtlasOobTest.cpp`; Ganesh macOS OpenGL.
- `GLBackendAllocationTest` — `tests/BackendAllocationTest.cpp`; Ganesh macOS OpenGL.
- `BigImageTest_Ganesh` — `tests/BigImageTest.cpp`; Ganesh macOS OpenGL.
- `TiledDrawCacheTest_Ganesh` — `tests/BigImageTest.cpp`; Ganesh macOS OpenGL.
- `ES2BlendWithNoTexture` — `tests/BlendTest.cpp`; Ganesh macOS OpenGL.
- `BlendRequiringDstReadWithLargeCoordinates` — `tests/BlendTest.cpp`; Ganesh macOS OpenGL.
- `SmallBoxBlurBug` — `tests/BlurTest.cpp`; Ganesh macOS OpenGL.
- `BlurMaskBiggerThanDest` — `tests/BlurTest.cpp`; Ganesh macOS OpenGL.
- `BlurDegenerateAffineFuzzer` — `tests/BlurTest.cpp`; Ganesh macOS OpenGL.
- `BulkFillRectTest` — `tests/BulkRectTest.cpp`; Ganesh macOS OpenGL.
- `BulkTextureRectTest` — `tests/BulkRectTest.cpp`; Ganesh macOS OpenGL.
- `TestManyDrawsGanesh` — `tests/CanvasTest.cpp`; Ganesh macOS OpenGL.
- `ClearOp` — `tests/ClearTest.cpp`; Ganesh macOS OpenGL.
- `FullScreenClearWithLayers` — `tests/ClearTest.cpp`; Ganesh macOS OpenGL.
- `ComposeFailureWithInputElision` — `tests/ColorFilterTest.cpp`; Ganesh macOS OpenGL.
- `CompressedBackendAllocationTest` — `tests/CompressedBackendAllocationTest.cpp`; Ganesh macOS OpenGL.
- `CopySurface` — `tests/CopySurfaceTest.cpp`; Ganesh macOS OpenGL.
- `DMSAA_preserve_contents` — `tests/DMSAATest.cpp`; Ganesh macOS OpenGL.
- `DMSAA_dst_read` — `tests/DMSAATest.cpp`; Ganesh macOS OpenGL.
- `DMSAA_aa_dst_read_after_dmsaa` — `tests/DMSAATest.cpp`; Ganesh macOS OpenGL.
- `DMSAA_dst_read_with_existing_barrier` — `tests/DMSAATest.cpp`; Ganesh macOS OpenGL.
- `DMSAA_dual_source_blend_disable` — `tests/DMSAATest.cpp`; Ganesh macOS OpenGL.
- `DashPathEffectTest_2PiInterval` — `tests/DashPathEffectTestGanesh.cpp`; Ganesh macOS OpenGL.
- `DefaultPathRendererTest` — `tests/DefaultPathRendererTest.cpp`; Ganesh macOS OpenGL.
- `DDLOperatorEqTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `CharacterizationFBO0nessTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLSurfaceCharacterizationTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLNonTextureabilityTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLMakeRenderTargetTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLWrapBackendTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLInvalidRecorder` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLCreateCharacterizationFailures` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLSkSurfaceFlush` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLMultipleDDLs` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLTextureFlagsTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `DDLCompatibilityTest` — `tests/DeferredDisplayListTest.cpp`; Ganesh macOS OpenGL.
- `SpecialImage_GPUDevice` — `tests/DeviceTest.cpp`; Ganesh macOS OpenGL.
- `DirectMaskLimitTest_Ganesh` — `tests/DirectMaskLimitTest.cpp`; Ganesh macOS OpenGL.
- `BasicDrawOpAtlas` — `tests/DrawOpAtlasTest.cpp`; Ganesh macOS OpenGL.
- `GrAtlasTextOpPreparation` — `tests/DrawOpAtlasTest.cpp`; Ganesh macOS OpenGL.
- `ExtendedSkColorTypeTests_gpu` — `tests/ExtendedSkColorTypeTests.cpp`; Ganesh macOS OpenGL.
- `F16DrawTest_Ganesh` — `tests/F16DrawTest.cpp`; Ganesh macOS OpenGL.
- `FilterResult_ganesh_` — `tests/FilterResultTest.cpp`; Ganesh macOS OpenGL.
- `HalfFloatAlphaTextureTest` — `tests/FloatingPointTextureTest.cpp`; Ganesh macOS OpenGL.
- `HalfFloatRGBATextureTest` — `tests/FloatingPointTextureTest.cpp`; Ganesh macOS OpenGL.
- `GLTextureParameters` — `tests/GLBackendSurfaceTest.cpp`; Ganesh macOS OpenGL.
- `GLFramebufferDeletion` — `tests/GLFramebufferDeletionTest.cpp`; Ganesh macOS OpenGL.
- `CPUSurface_UsesGaneshContextAndRasterRecorderToDraw_DrawsPixels` — `tests/GaneshContextRecorderTest.cpp`; Ganesh macOS OpenGL.
- `ImageMakeColorSpace_GaneshImageWithContext_Success` — `tests/GaneshContextRecorderTest.cpp`; Ganesh macOS OpenGL.
- `ImageMakeColorSpace_GaneshImageWithRecorder_Success` — `tests/GaneshContextRecorderTest.cpp`; Ganesh macOS OpenGL.
- `ImageMakeScaled_GaneshImageWithRecorder_Success` — `tests/GaneshContextRecorderTest.cpp`; Ganesh macOS OpenGL.
- `CanvasBaseRecorder_GaneshBasedCanvas_IsLinkedToDirectContext` — `tests/GaneshContextRecorderTest.cpp`; Ganesh macOS OpenGL.
- `GpuDrawPath` — `tests/GpuDrawPathTest.cpp`; Ganesh macOS OpenGL.
- `GrDrawCollapsedPath` — `tests/GpuDrawPathTest.cpp`; Ganesh macOS OpenGL.
- `PathTest_CrBug1232834` — `tests/GpuDrawPathTest.cpp`; Ganesh macOS OpenGL.
- `StrokeCircle_Bug356182429` — `tests/GpuDrawPathTest.cpp`; Ganesh macOS OpenGL.
- `ClipStack_SWMask` — `tests/GrClipStackTest.cpp`; Ganesh macOS OpenGL.
- `ClipStack_MixedAA` — `tests/GrClipStackTest.cpp`; Ganesh macOS OpenGL.
- `GrContextDump` — `tests/GrContextFactoryTest.cpp`; Ganesh macOS OpenGL.
- `FlushFinishedProcTest` — `tests/GrFinishedFlushTest.cpp`; Ganesh macOS OpenGL.
- `FinishedAsyncProcWhenAbandonedTest` — `tests/GrFinishedFlushTest.cpp`; Ganesh macOS OpenGL.
- `GrGpuBufferTransferTest` — `tests/GrGpuBufferTest.cpp`; Ganesh macOS OpenGL.
- `GrGpuBufferUpdateDataTest` — `tests/GrGpuBufferTest.cpp`; Ganesh macOS OpenGL.
- `GrMeshTest` — `tests/GrMeshTest.cpp`; Ganesh macOS OpenGL.
- `GrMeshTest` — `tests/GrMeshTest.cpp`; Ganesh macOS OpenGL.
- `GrWrappedMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh macOS OpenGL.
- `GrBackendTextureImageMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh macOS OpenGL.
- `GrImageSnapshotMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh macOS OpenGL.
- `Gr1x1TextureMipMappedTest` — `tests/GrMipMappedTest.cpp`; Ganesh macOS OpenGL.
- `OpsTaskFlushCount` — `tests/GrOpListFlushTest.cpp`; Ganesh macOS OpenGL.
- `GrPipelineDynamicStateTest` — `tests/GrPipelineDynamicStateTest.cpp`; Ganesh macOS OpenGL.
- `FlushSubmittedProcTest` — `tests/GrSubmittedFlushTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceResolveTest` — `tests/GrSurfaceResolveTest.cpp`; Ganesh macOS OpenGL.
- `NonmippedDrawBeforeMippedDraw` — `tests/GrSurfaceResolveTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceResolveAfterFailedFlush` — `tests/GrSurfaceResolveTest.cpp`; Ganesh macOS OpenGL.
- `GrSurfaceRenderability` — `tests/GrSurfaceTest.cpp`; Ganesh macOS OpenGL.
- `ReadOnlyTexture` — `tests/GrSurfaceTest.cpp`; Ganesh macOS OpenGL.
- `GrTextBlobScaleAnimation` — `tests/GrTextBlobTest.cpp`; Ganesh macOS OpenGL.
- `GrTextBlobMoveAround` — `tests/GrTextBlobTest.cpp`; Ganesh macOS OpenGL.
- `GrTextureMipMapInvalidationTest` — `tests/GrTextureMipMapInvalidationTest.cpp`; Ganesh macOS OpenGL.
- `ReimportImageTextureWithMipLevels` — `tests/GrTextureMipMapInvalidationTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache1View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache1Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache2View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache2Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache3View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache3Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache4View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache4Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache4_5View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache4_5Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache4_75View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache4_75Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache5View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache5Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache6View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache6Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache7View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache7Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache8View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache8Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache9View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache9Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache10View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache11View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache11Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache12View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache12Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache13View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache13Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache14` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache15View` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache15Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrThreadSafeCache16Verts` — `tests/GrThreadSafeCacheTest.cpp`; Ganesh macOS OpenGL.
- `GrUniqueProxyTest` — `tests/GrUniqueProxyTest.cpp`; Ganesh macOS OpenGL.
- `TestSweepGradientZeroXGanesh` — `tests/GradientTest.cpp`; Ganesh macOS OpenGL.
- `TestManyStopLinearHardstopsGanesh` — `tests/GradientTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterCache_ImageBackedGPU` — `tests/ImageFilterCacheTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterCache_GPUBacked` — `tests/ImageFilterCacheTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterNegativeBlurSigma_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `MorphologyFilterRadiusWithMirrorCTM_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterZeroBlurSigma_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterFailAffectsTransparentBlack_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterMergeResultSize_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterMatrixConvolutionBigKernel_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterCropRect_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterClippedPictureImageFilter_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ComposedImageFilterOffset_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ComposedImageFilterBounds_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterMakeWithFilter_Ganesh` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterHugeBlur_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `XfermodeImageFilterCroppedInput_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageFilterBlurLargeImage_Gpu` — `tests/ImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ImageIsOpaqueTest_Gpu` — `tests/ImageIsOpaqueTest.cpp`; Ganesh macOS OpenGL.
- `ImageNewShader_GPU` — `tests/ImageNewShaderTest.cpp`; Ganesh macOS OpenGL.
- `ImageEncode_Gpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `SkImage_Ganesh2Cpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `SkImage_makeTextureImage` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `SkImage_makeNonTextureImage` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `GrContext_colorTypeSupportedAsImage` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `UnpremulTextureImage` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `ImageReadPixels_Gpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `ImageLegacyBitmap_Gpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `ImagePeek_Gpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `SkImage_NewFromTextureRelease` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `makeBackendTexture` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `ImageBackendAccessAbandoned_Gpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `ImageScalePixels_Gpu` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `ImageFlush` — `tests/ImageTest.cpp`; Ganesh macOS OpenGL.
- `crbug_1271431` — `tests/LazyStencilAttachmentTest.cpp`; Ganesh macOS OpenGL.
- `MatrixColorFilter_TransparentBlack` — `tests/MatrixColorFilterTest.cpp`; Ganesh macOS OpenGL.
- `SkMultiPictureDocument_AHardwarebuffer` — `tests/MultiPictureDocumentTest.cpp`; Ganesh macOS OpenGL.
- `PinnedImageTest` — `tests/PinnedImageTest.cpp`; Ganesh macOS OpenGL.
- `PinnedImageTest_AsGaneshView` — `tests/PinnedImageTest.cpp`; Ganesh macOS OpenGL.
- `PinnedImageTest_AsFragmentProcessor` — `tests/PinnedImageTest.cpp`; Ganesh macOS OpenGL.
- `PremulAlphaRoundTrip_Gpu` — `tests/PremulAlphaRoundTripTest.cpp`; Ganesh macOS OpenGL.
- `VertexAttributeCount` — `tests/PrimitiveProcessorTest.cpp`; Ganesh macOS OpenGL.
- `ProcessorRefTest` — `tests/ProcessorTest.cpp`; Ganesh macOS OpenGL.
- `ProcessorOptimizationValidationTest` — `tests/ProcessorTest.cpp`; Ganesh macOS OpenGL.
- `ProcessorCloneTest` — `tests/ProcessorTest.cpp`; Ganesh macOS OpenGL.
- `PromiseImageTest` — `tests/PromiseImageTest.cpp`; Ganesh macOS OpenGL.
- `PromiseImageTextureFullCache` — `tests/PromiseImageTest.cpp`; Ganesh macOS OpenGL.
- `PromiseImageNullFulfill` — `tests/PromiseImageTest.cpp`; Ganesh macOS OpenGL.
- `Protected_SmokeTest` — `tests/ProtectedTest.cpp`; Ganesh macOS OpenGL.
- `Protected_readPixelsFromSurfaces` — `tests/ProtectedTest.cpp`; Ganesh macOS OpenGL.
- `Protected_CopyOnWrite` — `tests/ProtectedTest.cpp`; Ganesh macOS OpenGL.
- `Protected_asyncRescaleAndReadPixelsFromSurfaces` — `tests/ProtectedTest.cpp`; Ganesh macOS OpenGL.
- `Protected_pictureShader` — `tests/ProtectedTest.cpp`; Ganesh macOS OpenGL.
- `PreinstantiatedProxyConversionTest` — `tests/ProxyConversionTest.cpp`; Ganesh macOS OpenGL.
- `DefferredProxyConversionTest` — `tests/ProxyConversionTest.cpp`; Ganesh macOS OpenGL.
- `ProxyRefTest` — `tests/ProxyRefTest.cpp`; Ganesh macOS OpenGL.
- `DeferredProxyTest` — `tests/ProxyTest.cpp`; Ganesh macOS OpenGL.
- `WrappedProxyTest` — `tests/ProxyTest.cpp`; Ganesh macOS OpenGL.
- `ZeroSizedProxyTest` — `tests/ProxyTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceContextReadPixels` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `ReadPixels_InvalidRowBytes_Gpu` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `WritePixels_InvalidRowBytes_Gpu` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceAsyncReadPixels` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `ImageAsyncReadPixels_NonRenderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `ImageAsyncReadPixels_NonRenderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `ImageAsyncReadPixels_Renderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `ImageAsyncReadPixels_Renderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceContextWritePixels_NonRenderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceContextWritePixels_NonRenderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceContextWritePixels_Renderable_TopLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceContextWritePixels_Renderable_BottomLeft` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceContextWritePixelsMipped` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `GLReadPixelsUnbindPBO` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `ReadPixelsIntermediateFailedFlush` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `WritePixelsFailedFlushLeaksScratch` — `tests/ReadWritePixelsGpuTest.cpp`; Ganesh macOS OpenGL.
- `RectangleTexture` — `tests/RectangleTextureTest.cpp`; Ganesh macOS OpenGL.
- `RepeatedClippedBlurTest` — `tests/RepeatedClippedBlurTest.cpp`; Ganesh macOS OpenGL.
- `ResourceAllocatorTest` — `tests/ResourceAllocatorTest.cpp`; Ganesh macOS OpenGL.
- `ResourceAllocatorStressTest` — `tests/ResourceAllocatorTest.cpp`; Ganesh macOS OpenGL.
- `ResourceAllocatorMemoryBudgetTest` — `tests/ResourceAllocatorTest.cpp`; Ganesh macOS OpenGL.
- `ResourceCacheCache` — `tests/ResourceCacheTest.cpp`; Ganesh macOS OpenGL.
- `ResourceCacheStencilBuffers` — `tests/ResourceCacheTest.cpp`; Ganesh macOS OpenGL.
- `ResourceCacheWrappedResources` — `tests/ResourceCacheTest.cpp`; Ganesh macOS OpenGL.
- `ResourceMessagesAfterAbandon` — `tests/ResourceCacheTest.cpp`; Ganesh macOS OpenGL.
- `GPUMemorySize` — `tests/ResourceCacheTest.cpp`; Ganesh macOS OpenGL.
- `PurgeToMakeHeadroom` — `tests/ResourceCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeBlender_Ganesh` — `tests/RuntimeBlendTest.cpp`; Ganesh macOS OpenGL.
- `SRGBReadWritePixels` — `tests/SRGBReadWritePixelsTest.cpp`; Ganesh macOS OpenGL.
- `SaveLayerOrigin` — `tests/SaveLayerOriginTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeShaderImageFilter_GPU` — `tests/ShaderImageFilterTest.cpp`; Ganesh macOS OpenGL.
- `ShaderTestNestedBlendsGanesh` — `tests/ShaderTest.cpp`; Ganesh macOS OpenGL.
- `SimplifyPaintTest_Ganesh` — `tests/SimplifyPaintTest.cpp`; Ganesh macOS OpenGL.
- `SkColorSpaceXform_Ganesh` — `tests/SkColorSpaceXformStepsTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_StrikeSerialization` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_StrikeSerializationSlug` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_StrikeSerializationSlugForcePath` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_SlugSerialization` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_SlugSerialization_LargeOffset` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_SubRunBoundsReconstruction` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_ReleaseTypeFace` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_DrawTextAsPath` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_DrawTextAsMaskWithPathFallback` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_DrawTextXY` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_DrawTextAsDFT` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_CacheMissReporting` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_TypefaceWithNoPaths` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_TypefaceWithPaths_MaskThenPath` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRemoteGlyphCache_TypefaceWithPaths_PathThenMask` — `tests/SkRemoteGlyphCacheTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeEffectSimple_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeEffectObeysCapabilities_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeColorFilterReturningInvalidAlpha_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeEffect_Blender_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeStructNameReuse_GPU` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `SkRuntimeShader_TransformedCoords_Ganesh` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `GrSkSLFP_Specialized` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `GrSkSLFP_UniformArray` — `tests/SkRuntimeEffectTest.cpp`; Ganesh macOS OpenGL.
- `SkSLCross` — `tests/SkSLCross.cpp`; Ganesh macOS OpenGL.
- `SkSL_ES2Conformance_Pass_GPU` — `tests/SkSLES2ConformanceTest.cpp`; Ganesh macOS OpenGL.
- `SkSL` — `tests/SkSLTest.cpp`; Ganesh macOS OpenGL.
- `flags` — `tests/SkSLTest.cpp`; Ganesh macOS OpenGL.
- `skbug12214` — `tests/Skbug12214.cpp`; Ganesh macOS OpenGL.
- `skbug5221_GPU` — `tests/Skbug5221.cpp`; Ganesh macOS OpenGL.
- `skbug6653` — `tests/Skbug6653.cpp`; Ganesh macOS OpenGL.
- `Slug_empty` — `tests/SlugTest.cpp`; Ganesh macOS OpenGL.
- `Slug_b520113415` — `tests/SlugTest.cpp`; Ganesh macOS OpenGL.
- `Slug_b520571816` — `tests/SlugTest.cpp`; Ganesh macOS OpenGL.
- `Slug_draw_then_serialize` — `tests/SlugTest.cpp`; Ganesh macOS OpenGL.
- `Slug_draw_then_serialize_PathSubRun` — `tests/SlugTest.cpp`; Ganesh macOS OpenGL.
- `Slug_draw_then_serialize_DrawableSubRun` — `tests/SlugTest.cpp`; Ganesh macOS OpenGL.
- `SmallPathRenderer_crbug505876830` — `tests/SmallPathRendererTest.cpp`; Ganesh macOS OpenGL.
- `SpecialImage_Gpu` — `tests/SpecialImageTest.cpp`; Ganesh macOS OpenGL.
- `SrcSrcOverBatchTest` — `tests/SrcSrcOverBatchTest.cpp`; Ganesh macOS OpenGL.
- `StencilClearTest` — `tests/StencilClearTest.cpp`; Ganesh macOS OpenGL.
- `StencilClearTest_PreservedAcrossSplit` — `tests/StencilClearTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceDrawContextTest` — `tests/SurfaceDrawContextTest.cpp`; Ganesh macOS OpenGL.
- `EmptySurfaceSemaphoreTest` — `tests/SurfaceSemaphoreTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceEmpty_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `GrContext_colorTypeSupportedAsSurface` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `GrContext_maxSurfaceSamplesForColorType` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceCanvasPeek_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceSnapshotAlphaType_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceBackendSurfaceAccessCopyOnWrite_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceBackendHandleAccessIDs_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceAbandonPostFlush_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceBackendAccessAbandoned_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceCopyOnWrite_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceWriteableAfterSnapshotRelease_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceCRBug263329_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfacepeekTexture_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceBudget` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceNoCanvas_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceClear_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfacePartialDraw_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceWrappedWithRelease_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `SurfaceAttachStencil_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `DrawSnapshotBackIntoWappedSurface` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `ReplaceSurfaceBackendTexture` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `OverdrawSurface_Gpu` — `tests/SurfaceTest.cpp`; Ganesh macOS OpenGL.
- `TestGpuAllContexts` — `tests/TestTest.cpp`; Ganesh macOS OpenGL.
- `TestGpuRenderingContexts` — `tests/TestTest.cpp`; Ganesh macOS OpenGL.
- `TestGpuAllContextsWithTrueCondition` — `tests/TestTest.cpp`; Ganesh macOS OpenGL.
- `TestGpuAllContextsWithFalseCondition` — `tests/TestTest.cpp`; Ganesh macOS OpenGL.
- `TestGpuRenderingContextsWithTrueCondition` — `tests/TestTest.cpp`; Ganesh macOS OpenGL.
- `TestGpuRenderingContextsWithFalseCondition` — `tests/TestTest.cpp`; Ganesh macOS OpenGL.
- `TextBlobIntegerOverflowTest` — `tests/TextBlobCacheTest.cpp`; Ganesh macOS OpenGL.
- `TextBlobJaggedGlyph` — `tests/TextBlobCacheTest.cpp`; Ganesh macOS OpenGL.
- `TextBlobSmoothScroll` — `tests/TextBlobCacheTest.cpp`; Ganesh macOS OpenGL.
- `TextBlob_draw_releases_strike_from_cache` — `tests/TextBlobCacheTest.cpp`; Ganesh macOS OpenGL.
- `TextureBindingsResetTest` — `tests/TextureBindingsResetTest.cpp`; Ganesh macOS OpenGL.
- `TextureOpTest` — `tests/TextureOpTest.cpp`; Ganesh macOS OpenGL.
- `TextureProxyTest` — `tests/TextureProxyTest.cpp`; Ganesh macOS OpenGL.
- `ImageSizeTest_Ganesh` — `tests/TextureSizeTest.cpp`; Ganesh macOS OpenGL.
- `TextureStripAtlasManagerGradientTest` — `tests/TextureStripAtlasManagerTest.cpp`; Ganesh macOS OpenGL.
- `TextureStripAtlasManagerColorFilterTest` — `tests/TextureStripAtlasManagerTest.cpp`; Ganesh macOS OpenGL.
- `SkTraceMemoryDump_ownedGLBuffer` — `tests/TraceMemoryDumpTest.cpp`; Ganesh macOS OpenGL.
- `SkTraceMemoryDump_ownedGLTexture` — `tests/TraceMemoryDumpTest.cpp`; Ganesh macOS OpenGL.
- `SkTraceMemoryDump_unownedGLTexture` — `tests/TraceMemoryDumpTest.cpp`; Ganesh macOS OpenGL.
- `SkTraceMemoryDump_ownedGLRenderTarget` — `tests/TraceMemoryDumpTest.cpp`; Ganesh macOS OpenGL.
- `SkTraceMemoryDump_unownedGLRenderTarget` — `tests/TraceMemoryDumpTest.cpp`; Ganesh macOS OpenGL.
- `SkTraceMemoryDump_ownedGLTextureRenderTarget` — `tests/TraceMemoryDumpTest.cpp`; Ganesh macOS OpenGL.
- `TransferPixelsToTextureTest` — `tests/TransferPixelsTest.cpp`; Ganesh macOS OpenGL.
- `TransferPixelsFromTextureTest` — `tests/TransferPixelsTest.cpp`; Ganesh macOS OpenGL.
- `TriangulatingPathRendererTests` — `tests/TriangulatingPathRendererTests.cpp`; Ganesh macOS OpenGL.
- `WrappedSurfaceCopyOnWrite` — `tests/WrappedSurfaceCopyOnWriteTest.cpp`; Ganesh macOS OpenGL.
- `SkipCopyTaskTest` — `tests/WrappedSurfaceCopyOnWriteTest.cpp`; Ganesh macOS OpenGL.
- `SkipOpsTaskTest` — `tests/WrappedSurfaceCopyOnWriteTest.cpp`; Ganesh macOS OpenGL.
- `WritePixels_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh macOS OpenGL.
- `WritePixelsMSAA_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh macOS OpenGL.
- `WritePixelsNonTexture_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh macOS OpenGL.
- `WritePixelsNonTextureMSAA_Gpu` — `tests/WritePixelsTest.cpp`; Ganesh macOS OpenGL.
- `WritePixelsPendingIO` — `tests/WritePixelsTest.cpp`; Ganesh macOS OpenGL.
- `crbug_500172224_aa_double` — `tests/crbug_500172224.cpp`; Ganesh macOS OpenGL.
- `crbug_500172224_non_aa_double` — `tests/crbug_500172224.cpp`; Ganesh macOS OpenGL.
- `crbug_500172224_aa` — `tests/crbug_500172224.cpp`; Ganesh macOS OpenGL.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/DeferredDisplayListTest.cpp` — M38.
- `tests/GrMipMappedTest.cpp` — M38.
- `tests/ProxyTest.cpp` — M38.
- `tests/SurfaceSemaphoreTest.cpp` — M38.

### Preserve these fixtures, metadata and references

- `tests/sksl/runtime/GLSLTypeNames.minified.sksl` — fixture.
- `tests/sksl/runtime/GLSLTypeNames.skrp` — fixture.
- `tests/sksl/runtime/GLSLTypeNames.stage` — fixture.
- `tests/sksl/runtime/GLSLTypeNames.wgsl` — fixture.
- `tests/sksl/runtime/GLSLTypeNamesES3.minified.sksl` — fixture.
- `tests/sksl/runtime/GLSLTypeNamesES3.skrp` — fixture.
- `tests/sksl/runtime/GLSLTypeNamesES3.stage` — fixture.
- `tests/sksl/runtime/GLSLTypeNamesES3.wgsl` — fixture.
- `tools/ganesh/gl/interface/Makefile` — preserve.
- `tools/ganesh/gl/interface/README.md` — preserve.
- `tools/ganesh/gl/interface/interface.json5` — preserve.

## Dependency and integration notes

The initial scan found 1 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=34`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M34 ACCEPTED may the user hand Deepseek M35.
