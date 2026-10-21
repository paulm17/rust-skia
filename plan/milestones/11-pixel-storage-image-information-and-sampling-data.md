# Milestone 11: Pixel storage, image information and sampling data

## Outcome and entry gate

Implement ImageInfo, PixelRef, Bitmap, Pixmap, pixel conversion/swizzle, mipmaps, YUVA structures and read/write-pixel records. Preserve row bytes, ownership, mutability, color/alpha type validation and allocation failure paths.

Begin implementation only after milestone 10 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement ImageInfo, PixelRef, Bitmap, Pixmap, pixel conversion/swizzle, mipmaps, YUVA structures and read/write-pixel records. Preserve row bytes, ownership, mutability, color/alpha type validation and allocation failure paths.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkAlphaType.h` — port to Rust.
- `include/core/SkBitmap.h` — port to Rust.
- `include/core/SkColorType.h` — port to Rust.
- `include/core/SkImageInfo.h` — port to Rust.
- `include/core/SkMallocPixelRef.h` — port to Rust.
- `include/core/SkPixelRef.h` — port to Rust.
- `include/core/SkPixmap.h` — port to Rust.
- `include/core/SkSamplingOptions.h` — port to Rust.
- `include/core/SkSwizzle.h` — port to Rust.
- `include/core/SkTextureCompressionType.h` — port to Rust.
- `include/core/SkYUVAInfo.h` — port to Rust.
- `include/core/SkYUVAPixmaps.h` — port to Rust.
- `include/private/SkPixelStorage.h` — port to Rust.
- `src/core/Sk4px.h` — port to Rust.
- `src/core/SkAutoPixmapStorage.cpp` — port to Rust.
- `src/core/SkAutoPixmapStorage.h` — port to Rust.
- `src/core/SkBitmap.cpp` — port to Rust.
- `src/core/SkBitmapCache.cpp` — port to Rust.
- `src/core/SkBitmapCache.h` — port to Rust.
- `src/core/SkBitmapDevice.cpp` — port to Rust.
- `src/core/SkBitmapDevice.h` — port to Rust.
- `src/core/SkBitmapProcState.cpp` — port to Rust.
- `src/core/SkBitmapProcState.h` — port to Rust.
- `src/core/SkBitmapProcState_opts.cpp` — port to Rust.
- `src/core/SkBitmapProcState_opts_ssse3.cpp` — port to Rust.
- `src/core/SkCompressedDataUtils.cpp` — port to Rust.
- `src/core/SkCompressedDataUtils.h` — port to Rust.
- `src/core/SkConvertPixels.cpp` — port to Rust.
- `src/core/SkConvertPixels.h` — port to Rust.
- `src/core/SkImageInfo.cpp` — port to Rust.
- `src/core/SkImageInfoPriv.h` — port to Rust.
- `src/core/SkMallocPixelRef.cpp` — port to Rust.
- `src/core/SkMipmap.cpp` — port to Rust.
- `src/core/SkMipmap.h` — port to Rust.
- `src/core/SkMipmapAccessor.cpp` — port to Rust.
- `src/core/SkMipmapAccessor.h` — port to Rust.
- `src/core/SkMipmapBuilder.cpp` — port to Rust.
- `src/core/SkMipmapBuilder.h` — port to Rust.
- `src/core/SkMipmapDrawDownSampler.cpp` — port to Rust.
- `src/core/SkMipmapHQDownSampler.cpp` — port to Rust.
- `src/core/SkPixelRef.cpp` — port to Rust.
- `src/core/SkPixelRefPriv.h` — port to Rust.
- `src/core/SkPixelStorage.cpp` — port to Rust.
- `src/core/SkPixmap.cpp` — port to Rust.
- `src/core/SkPixmapDraw.cpp` — port to Rust.
- `src/core/SkReadPixelsRec.cpp` — port to Rust.
- `src/core/SkReadPixelsRec.h` — port to Rust.
- `src/core/SkSamplingPriv.h` — port to Rust.
- `src/core/SkSwizzle.cpp` — port to Rust.
- `src/core/SkSwizzlePriv.h` — port to Rust.
- `src/core/SkSwizzler_opts.cpp` — port to Rust.
- `src/core/SkSwizzler_opts_ml3.cpp` — port to Rust.
- `src/core/SkSwizzler_opts_ssse3.cpp` — port to Rust.
- `src/core/SkWritePixelsRec.cpp` — port to Rust.
- `src/core/SkWritePixelsRec.h` — port to Rust.
- `src/core/SkYUVAInfo.cpp` — port to Rust.
- `src/core/SkYUVAInfoLocation.h` — port to Rust.
- `src/core/SkYUVAPixmaps.cpp` — port to Rust.
- `src/core/SkYUVPlanesCache.cpp` — port to Rust.
- `src/core/SkYUVPlanesCache.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: acceptance-test contract

Translate storage and pixel conversion tests with exact byte arrays and original formats. Verify odd widths, row padding, subsets, aliasing, null/empty buffers, color conversion and YUV plane layout. Canvas/surface readback registrations run at M14 or GPU gates.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/BitmapGetColorTest.cpp`.
- `tests/BitmapTest.cpp`.
- `tests/ComparePixels.cpp`.
- `tests/ComparePixels.h`.
- `tests/MallocPixelRefTest.cpp`.
- `tests/PixelRefTest.cpp`.
- `tests/PixelsRecTest.cpp`.
- `tests/TracingTest.cpp`.
- `tests/YUVCacheTest.cpp`.

### Original test registrations assigned to this gate

- `GetColor` — `tests/BitmapGetColorTest.cpp:21`; CPU; `DEF_TEST`.
- `Bitmap` — `tests/BitmapTest.cpp:159`; CPU; `DEF_TEST`.
- `Bitmap_setColorSpace` — `tests/BitmapTest.cpp:179`; CPU; `DEF_TEST`.
- `Bitmap_getColor_Swizzle` — `tests/BitmapTest.cpp:227`; CPU; `DEF_TEST`.
- `Bitmap_eraseColor_Premul` — `tests/BitmapTest.cpp:257`; CPU; `DEF_TEST`.
- `Bitmap_compute_is_opaque` — `tests/BitmapTest.cpp:267`; CPU; `DEF_TEST`.
- `Bitmap_erase_f16_erase_getColor` — `tests/BitmapTest.cpp:285`; CPU; `DEF_TEST`.
- `Bitmap_erase_srgb` — `tests/BitmapTest.cpp:309`; CPU; `DEF_TEST`.
- `Bitmap_clear_pixelref_keep_info` — `tests/BitmapTest.cpp:322`; CPU; `DEF_TEST`.
- `Bitmap_erase` — `tests/BitmapTest.cpp:331`; CPU; `DEF_TEST`.
- `getalphaf` — `tests/BitmapTest.cpp:399`; CPU; `DEF_TEST`.
- `bitmap_zerowidth_crbug_1103827` — `tests/BitmapTest.cpp:477`; CPU; `DEF_TEST`.
- `MallocPixelRef` — `tests/MallocPixelRefTest.cpp:28`; CPU; `DEF_TEST`.
- `PixelRef_GenIDChange` — `tests/PixelRefTest.cpp:72`; CPU; `DEF_TEST`.
- `ReadPixelsRec_trim` — `tests/PixelsRecTest.cpp:18`; CPU; `DEF_TEST`.
- `WritePixelsRec_trim` — `tests/PixelsRecTest.cpp:115`; CPU; `DEF_TEST`.
- `Tracing` — `tests/TracingTest.cpp:180`; CPU; `DEF_TEST`.
- `YUVPlanesCache` — `tests/YUVCacheTest.cpp:38`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/svg/tests/PaintServer.cpp` — M28.
- `tests/AAClipTest.cpp` — M14.
- `tests/AnimatedImageTest.cpp` — M22.
- `tests/AnnotationTest.cpp` — M27.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/AvifTest.cpp` — M16.
- `tests/BackendAllocationTest.cpp` — M32, M34.
- `tests/BadIcoTest.cpp` — M16.
- `tests/BigImageTest.cpp` — M38.
- `tests/BitmapCopyTest.cpp` — M12.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/BulkRectTest.cpp` — M38.
- `tests/CPUContextRecorderTest.cpp` — M22.
- `tests/CachedDecodingPixelRefTest.cpp` — M16.
- `tests/CanvasStateTest.cpp` — M17.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/CappedHairlinesTest.cpp` — M14.
- `tests/ClearTest.cpp` — M38.
- `tests/ClipCubicTest.cpp` — M14.
- `tests/ClipperTest.cpp` — M14.
- `tests/CodecAnimTest.cpp` — M29.
- `tests/CodecExactReadTest.cpp` — M16.
- `tests/CodecPartialTest.cpp` — M16.
- `tests/CodecRecommendedTypeTest.cpp` — M17.
- `tests/CodecTest.cpp` — M17.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/ColorMatrixTest.cpp` — M14.
- `tests/ColorSpaceTest.cpp` — M16.
- `tests/CompressedBackendAllocationTest.cpp` — M38.
- `tests/ConvertPixelsTest.cpp` — M14.
- `tests/CopySurfaceTest.cpp` — M38.
- `tests/CoreBlittersTest.cpp` — M14.
- `tests/DMSAATest.cpp` — M38.
- `tests/DashPathEffectTest.cpp` — M14.
- `tests/DashPathEffectTestGanesh.cpp` — M38.
- `tests/DebugLayerManagerTest.cpp` — M40.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/DeviceTest.cpp` — M32, M38.
- `tests/DirectMaskLimitTest.cpp` — M38.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/DrawPathTest.cpp` — M14.
- `tests/DrawTextTest.cpp` — M18.
- `tests/EmptyPathTest.cpp` — M14.
- `tests/EncodeTest.cpp` — M17.
- `tests/EncodedInfoTest.cpp` — M17.
- `tests/ExifTest.cpp` — M17.
- `tests/ExtendedSkColorTypeTests.cpp` — M32, M38.
- `tests/F16DrawTest.cpp` — M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FlattenableFactoryToName.cpp` — M22.
- `tests/FloatingPointTextureTest.cpp` — M38.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/FrontBufferedStreamTest.cpp` — M16.
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
- `tests/GrMeshTest.cpp` — M38.
- `tests/GrMipMappedTest.cpp` — M32, M38.
- `tests/GrOpListFlushTest.cpp` — M38.
- `tests/GrPipelineDynamicStateTest.cpp` — M38.
- `tests/GrPorterDuffTest.cpp` — M32.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/GrSubmittedFlushTest.cpp` — M38.
- `tests/GrSurfaceResolveTest.cpp` — M32, M38.
- `tests/GrSurfaceTest.cpp` — M32, M38.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/GrTextureMipMapInvalidationTest.cpp` — M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/HdrMetadataTest.cpp` — M16.
- `tests/ImageBitmapTest.cpp` — M14.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageFrom565Bitmap.cpp` — M14.
- `tests/ImageGeneratorOrientationTest.cpp` — M16.
- `tests/ImageGeneratorTest.cpp` — M22.
- `tests/ImageIsOpaqueTest.cpp` — M32, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/IndexedPngOverflowTest.cpp` — M16.
- `tests/InvalidIndexedPngTest.cpp` — M16.
- `tests/JpegGainmapTest.cpp` — M17.
- `tests/JpegxlTest.cpp` — M16.
- `tests/LazyProxyTest.cpp` — M32.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/MatrixColorFilterTest.cpp` — M38.
- `tests/MatrixProcsTest.cpp` — M12.
- `tests/MipMapTest.cpp` — M14.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/NonlinearBlendingTest.cpp` — M13.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathOpsConicIntersectionTest.cpp` — M23.
- `tests/PathOpsTightBoundsTest.cpp` — M23.
- `tests/PathTest.cpp` — M22.
- `tests/PictureBBHTest.cpp` — M22.
- `tests/PictureShaderTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/PinnedImageTest.cpp` — M38.
- `tests/PngGainmapTest.cpp` — M17.
- `tests/PremulAlphaRoundTripTest.cpp` — M32, M38.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/ProgramsTest.cpp` — M32.
- `tests/PromiseImageTest.cpp` — M32, M38.
- `tests/ProtectedTest.cpp` — M38.
- `tests/ProxyTest.cpp` — M38.
- `tests/QuickRejectTest.cpp` — M15.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RecorderTest.cpp` — M22.
- `tests/RecordingXfermodeTest.cpp` — M22.
- `tests/RectTest.cpp` — M14.
- `tests/RectangleTextureTest.cpp` — M34.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/ResourceAllocatorTest.cpp` — M38.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RuntimeBlendTest.cpp` — M35, M38.
- `tests/SRGBReadWritePixelsTest.cpp` — M38.
- `tests/SRGBTest.cpp` — M12.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SamplingTest.cpp` — M14.
- `tests/SaveLayerOriginTest.cpp` — M38.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderOpacityTest.cpp` — M13.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/SimplifyPaintTest.cpp` — M35, M38.
- `tests/SkBmpRustDecoderTest.cpp` — M16.
- `tests/SkColorSpaceXformStepsTest.cpp` — M35, M38.
- `tests/SkIcoRustDecoderTest.cpp` — M16.
- `tests/SkImageTest.cpp` — M14.
- `tests/SkJpegRustCodecTest.cpp` — M17.
- `tests/SkPngRustDecoderTest.cpp` — M16.
- `tests/SkPngRustEncoderTest.cpp` — M17.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkResourceCacheTest.cpp` — M22.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkSLES2ConformanceTest.cpp` — M32, M38.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/SkStrikeCacheTest.cpp` — M22.
- `tests/SkYUVAInfoTest.cpp` — M16.
- `tests/Skbug12214.cpp` — M38.
- `tests/Skbug5221.cpp` — M32, M38.
- `tests/Skbug6389.cpp` — M15.
- `tests/Skbug6653.cpp` — M38.
- `tests/SlugTest.cpp` — M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/SrcSrcOverBatchTest.cpp` — M38.
- `tests/SurfaceDrawContextTest.cpp` — M38.
- `tests/SurfaceSemaphoreTest.cpp` — M32, M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/SwizzlerTest.cpp` — M16.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureBindingsResetTest.cpp` — M34.
- `tests/TextureOpTest.cpp` — M38.
- `tests/TextureProxyTest.cpp` — M38.
- `tests/TextureSizeTest.cpp` — M38.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TransferPixelsTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/VerticesTest.cpp` — M22.
- `tests/WebpTest.cpp` — M16.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp` — M38.
- `tests/WritePixelsTest.cpp` — M35, M38.
- `tests/YUVTest.cpp` — M17.
- `tests/crbug_500172224.cpp` — M38.
- `tests/graphite/CacheKeyTest.cpp` — M38.
- `tests/graphite/ComputeTest.cpp` — M36, M37, M38.
- `tests/graphite/DeviceTest.cpp` — M38.
- `tests/graphite/DrawAtlasTest.cpp` — M35, M38.
- `tests/graphite/GraphiteContextRecorderTest.cpp` — M38.
- `tests/graphite/GraphiteResourceCacheTest.cpp` — M38.
- `tests/graphite/ImageOriginTest.cpp` — M38.
- `tests/graphite/ImageProviderTest.cpp` — M38.
- `tests/graphite/ImageShaderTest.cpp` — M38.
- `tests/graphite/ImageWrapTextureMipmapsTest.cpp` — M38.
- `tests/graphite/MultisampleTest.cpp` — M38.
- `tests/graphite/MutableImagesTest.cpp` — M38.
- `tests/graphite/NotifyInUseTest.cpp` — M38.
- `tests/graphite/ProxyCacheTest.cpp` — M38.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp` — M38.
- `tests/graphite/RecordingOrderTest.cpp` — M38.
- `tests/graphite/RecordingSurfacesTest.cpp` — M38.
- `tests/graphite/SubmitWithFinishProcTest.cpp` — M38.
- `tests/graphite/TextureFormatTest.cpp` — M38.
- `tests/graphite/TextureProxyTest.cpp` — M38.
- `tests/graphite/UpdateBackendTextureTest.cpp` — M38.
- `tests/graphite/VerticesPaddingTest.cpp` — M37.
- `tests/graphite/crbug_513836996.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 72 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=11`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M11 ACCEPTED may the user hand Deepseek M12.
