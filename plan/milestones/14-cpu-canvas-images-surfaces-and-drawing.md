# Milestone 14: CPU canvas, images, surfaces and drawing

## Outcome and entry gate

Complete the CPU rendering pipeline: devices, Canvas state, images/surfaces, raster scan conversion, blitters, masks, atlas/vertices drawing and readback. Integrate prior geometry, pixels and shaders. Provide the CPU recorder/context surface present in this checkout. Later text/runtime hooks must have explicit feature boundaries, not fake successful results.

Begin implementation only after milestone 13 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Complete the CPU rendering pipeline: devices, Canvas state, images/surfaces, raster scan conversion, blitters, masks, atlas/vertices drawing and readback. Integrate prior geometry, pixels and shaders. Provide the CPU recorder/context surface present in this checkout. Later text/runtime hooks must have explicit feature boundaries, not fake successful results.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/RasterContext.h` — port to Rust.
- `include/core/SkCanvas.h` — port to Rust.
- `include/core/SkCanvasVirtualEnforcer.h` — port to Rust.
- `include/core/SkCapabilities.h` — port to Rust.
- `include/core/SkContext.h` — port to Rust.
- `include/core/SkContextOptions.h` — port to Rust.
- `include/core/SkDrawable.h` — port to Rust.
- `include/core/SkGraphics.h` — port to Rust.
- `include/core/SkImage.h` — port to Rust.
- `include/core/SkOverdrawCanvas.h` — port to Rust.
- `include/core/SkRasterHandleAllocator.h` — port to Rust.
- `include/core/SkSurface.h` — port to Rust.
- `include/core/SkSurfaceProps.h` — port to Rust.
- `include/core/SkTiledImageUtils.h` — port to Rust.
- `include/core/SkVertices.h` — port to Rust.
- `include/private/SkEnumBitMask.h` — port to Rust.
- `include/private/SkGPUContext.h` — port to Rust.
- `include/private/chromium/GrPromiseImageTexture.h` — port to Rust.
- `include/private/chromium/GrSurfaceCharacterization.h` — port to Rust.
- `include/private/chromium/SkImageChromium.h` — port to Rust.
- `include/utils/SkCanvasStateUtils.h` — port to Rust.
- `include/utils/SkNWayCanvas.h` — port to Rust.
- `include/utils/SkNoDrawCanvas.h` — port to Rust.
- `include/utils/SkNullCanvas.h` — port to Rust.
- `src/core/RasterContext.cpp` — port to Rust.
- `src/core/SkAlphaRuns.cpp` — port to Rust.
- `src/core/SkAlphaRuns.h` — port to Rust.
- `src/core/SkAnalyticEdge.cpp` — port to Rust.
- `src/core/SkAnalyticEdge.h` — port to Rust.
- `src/core/SkAutoBlitterChoose.h` — port to Rust.
- `src/core/SkBlitBWMaskTemplate.h` — port to Rust.
- `src/core/SkBlitMask.h` — port to Rust.
- `src/core/SkBlitMask_opts.cpp` — port to Rust.
- `src/core/SkBlitMask_opts_ssse3.cpp` — port to Rust.
- `src/core/SkBlitRow.h` — port to Rust.
- `src/core/SkBlitRow_D32.cpp` — port to Rust.
- `src/core/SkBlitRow_opts.cpp` — port to Rust.
- `src/core/SkBlitRow_opts_ml3.cpp` — port to Rust.
- `src/core/SkBlitter.cpp` — port to Rust.
- `src/core/SkBlitter.h` — port to Rust.
- `src/core/SkBlitter_A8.cpp` — port to Rust.
- `src/core/SkBlitter_A8.h` — port to Rust.
- `src/core/SkBlitter_ARGB32.cpp` — port to Rust.
- `src/core/SkBlitter_Sprite.cpp` — port to Rust.
- `src/core/SkCanvas.cpp` — port to Rust.
- `src/core/SkCanvasPriv.cpp` — port to Rust.
- `src/core/SkCanvasPriv.h` — port to Rust.
- `src/core/SkCanvas_Raster.cpp` — port to Rust.
- `src/core/SkCapabilities.cpp` — port to Rust.
- `src/core/SkContext.cpp` — port to Rust.
- `src/core/SkContextPriv.h` — port to Rust.
- `src/core/SkCoreBlitters.h` — port to Rust.
- `src/core/SkDevice.cpp` — port to Rust.
- `src/core/SkDevice.h` — port to Rust.
- `src/core/SkDraw.cpp` — port to Rust.
- `src/core/SkDraw.h` — port to Rust.
- `src/core/SkDrawProcs.h` — port to Rust.
- `src/core/SkDrawTypes.h` — port to Rust.
- `src/core/SkDraw_atlas.cpp` — port to Rust.
- `src/core/SkDraw_vertices.cpp` — port to Rust.
- `src/core/SkDrawable.cpp` — port to Rust.
- `src/core/SkEdge.cpp` — port to Rust.
- `src/core/SkEdge.h` — port to Rust.
- `src/core/SkEdgeBuilder.cpp` — port to Rust.
- `src/core/SkEdgeBuilder.h` — port to Rust.
- `src/core/SkGraphics.cpp` — port to Rust.
- `src/core/SkLatticeIter.cpp` — port to Rust.
- `src/core/SkLatticeIter.h` — port to Rust.
- `src/core/SkMask.cpp` — port to Rust.
- `src/core/SkMask.h` — port to Rust.
- `src/core/SkMaskGamma.cpp` — port to Rust.
- `src/core/SkMaskGamma.h` — port to Rust.
- `src/core/SkMasks.cpp` — port to Rust.
- `src/core/SkMasks.h` — port to Rust.
- `src/core/SkOverdrawCanvas.cpp` — port to Rust.
- `src/core/SkScan.cpp` — port to Rust.
- `src/core/SkScan.h` — port to Rust.
- `src/core/SkScanPriv.h` — port to Rust.
- `src/core/SkScan_Antihair.cpp` — port to Rust.
- `src/core/SkScan_Hairline.cpp` — port to Rust.
- `src/core/SkSharedContext.cpp` — port to Rust.
- `src/core/SkSharedContext.h` — port to Rust.
- `src/core/SkSpriteBlitter.h` — port to Rust.
- `src/core/SkSpriteBlitter_ARGB32.cpp` — port to Rust.
- `src/core/SkSurfacePriv.h` — port to Rust.
- `src/core/SkVertState.cpp` — port to Rust.
- `src/core/SkVertState.h` — port to Rust.
- `src/core/SkVertices.cpp` — port to Rust.
- `src/core/SkVerticesPriv.h` — port to Rust.
- `src/image/SkImage.cpp` — port to Rust.
- `src/image/SkImage_Base.cpp` — port to Rust.
- `src/image/SkImage_Base.h` — port to Rust.
- `src/image/SkImage_Raster.cpp` — port to Rust.
- `src/image/SkImage_Raster.h` — port to Rust.
- `src/image/SkImage_RasterFactories.cpp` — port to Rust.
- `src/image/SkRescaleAndReadPixels.cpp` — port to Rust.
- `src/image/SkRescaleAndReadPixels.h` — port to Rust.
- `src/image/SkSurface.cpp` — port to Rust.
- `src/image/SkSurface_Base.cpp` — port to Rust.
- `src/image/SkSurface_Base.h` — port to Rust.
- `src/image/SkSurface_Null.cpp` — port to Rust.
- `src/image/SkSurface_Raster.cpp` — port to Rust.
- `src/image/SkSurface_Raster.h` — port to Rust.
- `src/image/SkTiledImageUtils.cpp` — port to Rust.
- `src/utils/SkCanvasStack.cpp` — port to Rust.
- `src/utils/SkCanvasStack.h` — port to Rust.
- `src/utils/SkCanvasStateUtils.cpp` — port to Rust.
- `src/utils/SkNWayCanvas.cpp` — port to Rust.
- `src/utils/SkNullCanvas.cpp` — port to Rust.
- `src/utils/SkPatchUtils.cpp` — port to Rust.
- `src/utils/SkPatchUtils.h` — port to Rust.
- `src/utils/mac/SkCreateCGImageRef.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `src/image/BUILD.bazel`.

## Codex: acceptance-test contract

Run the CPU rendering registrations, including Rect, Rect_grow and big_tiled_rect_crbug_927075. Translate canvas, surface, bitmap drawing, clips, AA, read/write-pixel and image tests. Run all earlier geometry tests with rendering portions enabled; compare output with the pinned C++ CPU build.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/AAClipTest.cpp`.
- `tests/BlitMaskClip.cpp`.
- `tests/CanvasStateHelpers.cpp`.
- `tests/CanvasStateHelpers.h`.
- `tests/CappedHairlinesTest.cpp`.
- `tests/ClipCubicTest.cpp`.
- `tests/ClipperTest.cpp`.
- `tests/ColorMatrixTest.cpp`.
- `tests/ConvertPixelsTest.cpp`.
- `tests/CoreBlittersTest.cpp`.
- `tests/DashPathEffectTest.cpp`.
- `tests/DrawBitmapRectTest.cpp`.
- `tests/DrawPathTest.cpp`.
- `tests/EdgeTest.cpp`.
- `tests/EmptyPathTest.cpp`.
- `tests/FillPathTest.cpp`.
- `tests/ImageBitmapTest.cpp`.
- `tests/ImageFrom565Bitmap.cpp`.
- `tests/InfRectTest.cpp`.
- `tests/MipMapTest.cpp`.
- `tests/ReadPixelsTest.cpp`.
- `tests/RectTest.cpp`.
- `tests/RegionTest.cpp`.
- `tests/SamplingTest.cpp`.
- `tests/SkEnumBitMaskTest.cpp`.
- `tests/SkImageTest.cpp`.
- `tests/SrcOverTest.cpp`.
- `tools/ToolUtils.cpp`.
- `tools/ToolUtils.h`.

### Original test registrations assigned to this gate

- `AAClip_setPath_RandomRegion_MatchesSkRegion` — `tests/AAClipTest.cpp:173`; CPU; `DEF_TEST`.
- `AAClip_setPath_ClipBoundsMatchExpectations` — `tests/AAClipTest.cpp:197`; CPU; `DEF_TEST`.
- `AAClip_BasicFunctionality` — `tests/AAClipTest.cpp:223`; CPU; `DEF_TEST`.
- `AAClip_setRect_RandomRects_MatchesSkRegion` — `tests/AAClipTest.cpp:257`; CPU; `DEF_TEST`.
- `AAClip_setPath_PathHasHole_MaskIsCorrect` — `tests/AAClipTest.cpp:300`; CPU; `DEF_TEST`.
- `AAClip_RRectIsReallyARect_ClipIsRect` — `tests/AAClipTest.cpp:340`; CPU; `DEF_TEST`.
- `AAClip_op_NearlyIntegral_GenerateSameRasterClips` — `tests/AAClipTest.cpp:386`; CPU; `DEF_TEST`.
- `AAClip_setPath_AvoidAssertion` — `tests/AAClipTest.cpp:400`; CPU; `DEF_TEST`.
- `AAClip_crbug_422693_AvoidOverflow` — `tests/AAClipTest.cpp:419`; CPU; `DEF_TEST`.
- `AAClip_setRect_HugeRect_ReturnsFalse` — `tests/AAClipTest.cpp:425`; CPU; `DEF_TEST`.
- `AAClip_setPath_LargePathSmallClip_StillBlits` — `tests/AAClipTest.cpp:434`; CPU; `DEF_TEST`.
- `BlitAndClip` — `tests/BlitMaskClip.cpp:43`; CPU; `DEF_TEST`.
- `DrawAntialiasHairline_SquareCappedNoPixelOffsetClosedContour_DoublePixelWideOutput` — `tests/CappedHairlinesTest.cpp:121`; CPU; `DEF_TEST`.
- `DrawAntialiasHairline_RoundCappedNoPixelOffsetClosedContour_DoublePixelWideOutput` — `tests/CappedHairlinesTest.cpp:126`; CPU; `DEF_TEST`.
- `DrawAntialiasHairline_ButtCappedNoPixelOffsetClosedContour_DoublePixelWideOutput` — `tests/CappedHairlinesTest.cpp:131`; CPU; `DEF_TEST`.
- `DrawAntialiasHairline_SquareCappedHalfPixelOffsetClosedContour_SinglePixelWideOutput` — `tests/CappedHairlinesTest.cpp:185`; CPU; `DEF_TEST`.
- `DrawAntialiasHairline_RoundCappedHalfPixelOffsetClosedContour_SinglePixelWideOutput` — `tests/CappedHairlinesTest.cpp:190`; CPU; `DEF_TEST`.
- `DrawAntialiasHairline_ButtCappedHalfPixelOffsetClosedContour_SinglePixelWideOutput` — `tests/CappedHairlinesTest.cpp:195`; CPU; `DEF_TEST`.
- `ClipCubic` — `tests/ClipCubicTest.cpp:80`; CPU; `DEF_TEST`.
- `test_fuzz_crbug_698714` — `tests/ClipCubicTest.cpp:175`; CPU; `DEF_TEST`.
- `cubic_scan_error_crbug_844457_and_845489` — `tests/ClipCubicTest.cpp:213`; CPU; `DEF_TEST`.
- `Clipper` — `tests/ClipperTest.cpp:159`; CPU; `DEF_TEST`.
- `LineClipper_skbug_7981` — `tests/ClipperTest.cpp:165`; CPU; `DEF_TEST`.
- `ColorMatrix` — `tests/ColorMatrixTest.cpp:99`; CPU; `DEF_TEST`.
- `ColorMatrix_clamp_while_unpremul` — `tests/ColorMatrixTest.cpp:104`; CPU; `DEF_TEST`.
- `ConvertPixels_in_place` — `tests/ConvertPixelsTest.cpp:19`; CPU; `DEF_TEST`.
- `SkARGB32OpaqueBlitter_MaskAndAntiHDrawTheSame` — `tests/CoreBlittersTest.cpp:122`; CPU; `DEF_TEST`.
- `SkARGB32BlackBlitter_MaskAndAntiHDrawTheSame` — `tests/CoreBlittersTest.cpp:149`; CPU; `DEF_TEST`.
- `SkARGB32Blitter_MaskAndAntiHDrawTheSame` — `tests/CoreBlittersTest.cpp:168`; CPU; `DEF_TEST`.
- `DashPathEffectTest_crbug_348821` — `tests/DashPathEffectTest.cpp:32`; CPU; `DEF_TEST`.
- `DashPathEffectTest_asPoints` — `tests/DashPathEffectTest.cpp:41`; CPU; `DEF_TEST`.
- `DashPath_bug4871` — `tests/DashPathEffectTest.cpp:100`; CPU; `DEF_TEST`.
- `DashPathEffectTest_asPoints_limit` — `tests/DashPathEffectTest.cpp:118`; CPU; `DEF_TEST`.
- `DashCrazy_crbug_875494` — `tests/DashPathEffectTest.cpp:133`; CPU; `DEF_TEST`.
- `DrawBitmapRect` — `tests/DrawBitmapRectTest.cpp:250`; CPU; `DEF_TEST`.
- `DrawPath` — `tests/DrawPathTest.cpp:440`; CPU; `DEF_TEST`.
- `SkEdge_CWLineWithoutClip_PublicMembersMatchMathematics` — `tests/EdgeTest.cpp:19`; CPU; `DEF_TEST`.
- `SkEdge_CWLineNullClip_PublicMembersMatchMathematics` — `tests/EdgeTest.cpp:37`; CPU; `DEF_TEST`.
- `SkEdge_CCWLineWithoutClip_PublicMembersMatchMathematics` — `tests/EdgeTest.cpp:55`; CPU; `DEF_TEST`.
- `SkEdge_Quad_ApproximatedWithMultipleLineSegments` — `tests/EdgeTest.cpp:128`; CPU; `DEF_TEST`.
- `SkEdge_MostlyFlatQuad_SkipsZeroHeightSegments` — `tests/EdgeTest.cpp:186`; CPU; `DEF_TEST`.
- `SkEdge_HorizontalishQuad_ReturnsFalse` — `tests/EdgeTest.cpp:213`; CPU; `DEF_TEST`.
- `SkEdge_Cubic_ApproximatedWithMultipleLineSegments` — `tests/EdgeTest.cpp:220`; CPU; `DEF_TEST`.
- `SkBasicEdgeBuilder_TrapezoidNoClip_HasTwoEdges` — `tests/EdgeTest.cpp:277`; CPU; `DEF_TEST`.
- `EmptyPath` — `tests/EmptyPathTest.cpp:172`; CPU; `DEF_TEST`.
- `FillPathInverse` — `tests/FillPathTest.cpp:39`; CPU; `DEF_TEST`.
- `ImageBitmapIdentity` — `tests/ImageBitmapTest.cpp:19`; CPU; `DEF_TEST`.
- `ImageFrom565Bitmap` — `tests/ImageFrom565Bitmap.cpp:17`; CPU; `DEF_TEST`.
- `InfRect` — `tests/InfRectTest.cpp:26`; CPU; `DEF_TEST`.
- `MipMap` — `tests/MipMapTest.cpp:35`; CPU; `DEF_TEST`.
- `MipMap_DirectLevelAccess` — `tests/MipMapTest.cpp:111`; CPU; `DEF_TEST`.
- `MipMap_ComputeLevelCount` — `tests/MipMapTest.cpp:140`; CPU; `DEF_TEST`.
- `MipMap_ComputeLevelSize` — `tests/MipMapTest.cpp:190`; CPU; `DEF_TEST`.
- `MipMap_F16` — `tests/MipMapTest.cpp:231`; CPU; `DEF_TEST`.
- `image_mip_factory` — `tests/MipMapTest.cpp:248`; CPU; `DEF_TEST`.
- `image_mip_mismatch` — `tests/MipMapTest.cpp:268`; CPU; `DEF_TEST`.
- `ReadPixels` — `tests/ReadPixelsTest.cpp:397`; CPU; `DEF_TEST`.
- `ReadPixels_ValidConversion` — `tests/ReadPixelsTest.cpp:500`; CPU; `DEF_TEST`.
- `ReadPixels_InvalidRowBytes` — `tests/ReadPixelsTest.cpp:541`; CPU; `DEF_TEST`.
- `Rect` — `tests/RectTest.cpp:103`; CPU; `DEF_TEST`.
- `Rect_grow` — `tests/RectTest.cpp:108`; CPU; `DEF_TEST`.
- `big_tiled_rect_crbug_927075` — `tests/RectTest.cpp:440`; CPU; `DEF_TEST`.
- `Region` — `tests/RegionTest.cpp:220`; CPU; `DEF_TEST`.
- `Region_writeToMemory` — `tests/RegionTest.cpp:241`; CPU; `DEF_TEST`.
- `Region_readFromMemory_bad` — `tests/RegionTest.cpp:271`; CPU; `DEF_TEST`.
- `region_toobig` — `tests/RegionTest.cpp:360`; CPU; `DEF_TEST`.
- `region_inverse_union_skbug_7491` — `tests/RegionTest.cpp:382`; CPU; `DEF_TEST`.
- `giant_path_region` — `tests/RegionTest.cpp:400`; CPU; `DEF_TEST`.
- `rrect_region_crbug_850350` — `tests/RegionTest.cpp:411`; CPU; `DEF_TEST`.
- `region_bug_chromium_873051` — `tests/RegionTest.cpp:432`; CPU; `DEF_TEST`.
- `region_empty_iter` — `tests/RegionTest.cpp:440`; CPU; `DEF_TEST`.
- `region_very_large` — `tests/RegionTest.cpp:481`; CPU; `DEF_TEST`.
- `region_b510359475` — `tests/RegionTest.cpp:537`; CPU; `DEF_TEST`.
- `SkRegion_Iterator_StepsThroughAllScanlines` — `tests/RegionTest.cpp:558`; CPU; `DEF_TEST`.
- `SkRegion_ReadFromMemory_ConsecutiveEmptySlices_Invalid` — `tests/RegionTest.cpp:604`; CPU; `DEF_TEST`.
- `SkRegion_ReadFromMemory_SingleEmptySlice_Valid` — `tests/RegionTest.cpp:640`; CPU; `DEF_TEST`.
- `Region_setRects` — `tests/RegionTest.cpp:688`; CPU; `DEF_TEST`.
- `sampling_with_identity_matrix` — `tests/SamplingTest.cpp:26`; CPU; `DEF_TEST`.
- `skgpu_Mask` — `tests/SkEnumBitMaskTest.cpp:19`; CPU; `DEF_TEST`.
- `SkImageFromBitmap_extractSubset` — `tests/SkImageTest.cpp:27`; CPU; `DEF_TEST`.
- `SkImageRequiredPropertiesCanBeMapKey` — `tests/SkImageTest.cpp:62`; CPU; `DEF_TEST`.
- `SrcOver` — `tests/SrcOverTest.cpp:29`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skottie/tests/Image.cpp` — M30.
- `modules/skottie/tests/SkottieTest.cpp` — M30.
- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/svg/tests/Filters.cpp` — M28.
- `modules/svg/tests/PaintServer.cpp` — M28.
- `modules/svg/tests/Text.cpp` — M29.
- `tests/AnimatedImageTest.cpp` — M22.
- `tests/AnnotationTest.cpp` — M27.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/AtlasOobTest.cpp` — M38.
- `tests/BigImageTest.cpp` — M38.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/BulkRectTest.cpp` — M38.
- `tests/CPUBackendTest.cpp` — M15.
- `tests/CPUContextRecorderTest.cpp` — M22.
- `tests/CachedDecodingPixelRefTest.cpp` — M16.
- `tests/CanvasStateTest.cpp` — M17.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/ClearTest.cpp` — M38.
- `tests/CodecAnimTest.cpp` — M29.
- `tests/CodecTest.cpp` — M17.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/CompressedBackendAllocationTest.cpp` — M38.
- `tests/DMSAATest.cpp` — M38.
- `tests/DashPathEffectTestGanesh.cpp` — M38.
- `tests/DebugLayerManagerTest.cpp` — M40.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/DeviceTest.cpp` — M32, M38.
- `tests/DirectMaskLimitTest.cpp` — M38.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/DrawTextTest.cpp` — M18.
- `tests/EncodeTest.cpp` — M17.
- `tests/ExifTest.cpp` — M17.
- `tests/ExtendedSkColorTypeTests.cpp` — M32, M38.
- `tests/F16DrawTest.cpp` — M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FlattenDrawableTest.cpp` — M22.
- `tests/FlattenableFactoryToName.cpp` — M22.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/GLBackendSurfaceTest.cpp` — M34.
- `tests/GLFramebufferDeletionTest.cpp` — M34.
- `tests/GainmapShaderTest.cpp` — M16.
- `tests/GaneshContextRecorderTest.cpp` — M38.
- `tests/GifTest.cpp` — M16.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
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
- `tests/GrSubmittedFlushTest.cpp` — M38.
- `tests/GrSurfaceResolveTest.cpp` — M32, M38.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/GrTextureMipMapInvalidationTest.cpp` — M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GrUniqueProxyTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/HdrMetadataTest.cpp` — M16.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageGeneratorOrientationTest.cpp` — M16.
- `tests/ImageGeneratorTest.cpp` — M22.
- `tests/ImageIsOpaqueTest.cpp` — M32, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/IndexedPngOverflowTest.cpp` — M16.
- `tests/JpegGainmapTest.cpp` — M17.
- `tests/LazyProxyTest.cpp` — M32.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/MaskCacheTest.cpp` — M15.
- `tests/MatrixColorFilterTest.cpp` — M38.
- `tests/MtlCopySurfaceTest.cc` — M33.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PathOpsConicIntersectionTest.cpp` — M23.
- `tests/PathOpsTightBoundsTest.cpp` — M23.
- `tests/PathRendererCacheTests.cpp` — M32.
- `tests/PathTest.cpp` — M22.
- `tests/PictureBBHTest.cpp` — M22.
- `tests/PictureShaderTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/PinnedImageTest.cpp` — M38.
- `tests/PngGainmapTest.cpp` — M17.
- `tests/PremulAlphaRoundTripTest.cpp` — M32, M38.
- `tests/PrimitiveProcessorTest.cpp` — M38.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/ProgramsTest.cpp` — M32.
- `tests/PromiseImageTest.cpp` — M32, M38.
- `tests/ProtectedTest.cpp` — M38.
- `tests/QuickRejectTest.cpp` — M15.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RecordPatternTest.cpp` — M22.
- `tests/RecorderTest.cpp` — M22.
- `tests/RecordingXfermodeTest.cpp` — M22.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/ResourceAllocatorTest.cpp` — M38.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RuntimeBlendTest.cpp` — M35, M38.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SaveLayerOriginTest.cpp` — M38.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/ShadowTest.cpp` — M15.
- `tests/SimplifyPaintTest.cpp` — M35, M38.
- `tests/SkBmpRustDecoderTest.cpp` — M16.
- `tests/SkColorSpaceXformStepsTest.cpp` — M35, M38.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkIcoRustDecoderTest.cpp` — M16.
- `tests/SkJpegRustCodecTest.cpp` — M17.
- `tests/SkPngRustDecoderTest.cpp` — M16.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkResourceCacheTest.cpp` — M22.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkSLES2ConformanceTest.cpp` — M32, M38.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/SkStrikeCacheTest.cpp` — M22.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/Skbug12214.cpp` — M38.
- `tests/Skbug5221.cpp` — M32, M38.
- `tests/Skbug6389.cpp` — M15.
- `tests/Skbug6653.cpp` — M38.
- `tests/SlugTest.cpp` — M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/SrcSrcOverBatchTest.cpp` — M38.
- `tests/StencilClearTest.cpp` — M38.
- `tests/SurfaceDrawContextTest.cpp` — M38.
- `tests/SurfaceSemaphoreTest.cpp` — M32, M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureBindingsResetTest.cpp` — M34.
- `tests/TextureProxyTest.cpp` — M38.
- `tests/TextureSizeTest.cpp` — M38.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/VerticesTest.cpp` — M22.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp` — M38.
- `tests/WritePixelsTest.cpp` — M35, M38.
- `tests/crbug_500172224.cpp` — M38.
- `tests/graphite/BackendTextureTest.cpp` — M38.
- `tests/graphite/CacheKeyTest.cpp` — M38.
- `tests/graphite/DeviceTest.cpp` — M38.
- `tests/graphite/GraphiteContextRecorderTest.cpp` — M38.
- `tests/graphite/GraphitePromiseImageTest.cpp` — M38.
- `tests/graphite/GraphiteResourceCacheTest.cpp` — M38.
- `tests/graphite/GraphiteYUVAPromiseImageTest.cpp` — M38.
- `tests/graphite/ImageProviderTest.cpp` — M38.
- `tests/graphite/ImageWrapTextureMipmapsTest.cpp` — M38.
- `tests/graphite/MultisampleTest.cpp` — M38.
- `tests/graphite/MutableImagesTest.cpp` — M38.
- `tests/graphite/NotifyInUseTest.cpp` — M38.
- `tests/graphite/PersistentPipelineStorageTest.cpp` — M38.
- `tests/graphite/PipelineCallbackTest.cpp` — M38.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp` — M38.
- `tests/graphite/RecorderTest.cpp` — M38.
- `tests/graphite/RecordingSurfacesTest.cpp` — M38.
- `tests/graphite/SubmitWithFinishProcTest.cpp` — M38.
- `tests/graphite/TextureFallbackTest.cpp` — M38.
- `tests/graphite/TextureProxyTest.cpp` — M38.
- `tests/graphite/crbug_513836996.cpp` — M38.
- `tests/graphite/precompile/PaintParamsKeyTest.cpp` — M38.
- `tests/graphite/precompile/PipelineManagerThreaded.cpp` — M38.
- `tests/graphite/precompile/ThreadedPrecompileTest.cpp` — M38.
- `tests/graphite/precompile/UserdefinedStableKeyTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 82 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=14`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M14 ACCEPTED may the user hand Deepseek M15.
