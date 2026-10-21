# Milestone 10: Colors, profiles and skcms

## Outcome and entry gate

Implement packed/unpacked colors, alpha conversions, color matrices, color spaces, color-space transforms and the bundled first-party skcms algorithms in Rust. Preserve ICC parsing behavior and transfer functions; record external ICC helpers separately. Do not make bundled skcms a permanent C++ fallback.

Begin implementation only after milestone 09 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement packed/unpacked colors, alpha conversions, color matrices, color spaces, color-space transforms and the bundled first-party skcms algorithms in Rust. Preserve ICC parsing behavior and transfer functions; record external ICC helpers separately. Do not make bundled skcms a permanent C++ fallback.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkColor.h` — port to Rust.
- `include/core/SkColorSpace.h` — port to Rust.
- `include/core/SkColorTable.h` — port to Rust.
- `include/core/SkUnPreMultiply.h` — port to Rust.
- `include/private/chromium/SkCodecsICCProfileChromium.h` — port to Rust.
- `include/private/chromium/SkPMColor.h` — port to Rust.
- `modules/skcms/skcms.cc` — port to Rust.
- `modules/skcms/skcms.h` — port to Rust.
- `modules/skcms/src/Transform_inl.h` — port to Rust.
- `modules/skcms/src/skcms_Transform.h` — port to Rust.
- `modules/skcms/src/skcms_TransformBaseline.cc` — port to Rust.
- `modules/skcms/src/skcms_TransformHsw.cc` — port to Rust.
- `modules/skcms/src/skcms_TransformSkx.cc` — port to Rust.
- `modules/skcms/src/skcms_internals.h` — port to Rust.
- `modules/skcms/src/skcms_public.h` — port to Rust.
- `src/core/SkColor.cpp` — port to Rust.
- `src/core/SkColorData.h` — port to Rust.
- `src/core/SkColorPriv.h` — port to Rust.
- `src/core/SkColorSpace.cpp` — port to Rust.
- `src/core/SkColorSpacePriv.h` — port to Rust.
- `src/core/SkColorSpaceXformSteps.cpp` — port to Rust.
- `src/core/SkColorSpaceXformSteps.h` — port to Rust.
- `src/core/SkColorTable.cpp` — port to Rust.
- `src/core/SkUnPreMultiply.cpp` — port to Rust.
- `src/utils/SkParseColor.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/skcms/BUILD.bazel`.
- `modules/skcms/BUILD.gn`.
- `modules/skcms/skcms.gni`.

## Codex: acceptance-test contract

Translate color, premultiplication, ICC, color-space and skcms-facing tests. Preserve rounding, gamut, malformed-profile and profile-fixture expectations. Some tests compare rendered pixels and receive a later rendering registration; numeric conversions are gated now.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/ColorPrivTest.cpp`.
- `tests/ColorTest.cpp`.
- `tests/HSVRoundTripTest.cpp`.
- `tests/ParseColorTest.cpp`.
- `tests/SkColor4fTest.cpp`.

### Original test registrations assigned to this gate

- `Splay` — `tests/ColorPrivTest.cpp:18`; CPU; `DEF_TEST`.
- `FourByteInterp` — `tests/ColorPrivTest.cpp:32`; CPU; `DEF_TEST`.
- `SkBlendARGB32_SameOpaqueSrcAndDstWithAnyAlpha_ProducesSameColor` — `tests/ColorPrivTest.cpp:56`; CPU; `DEF_TEST`.
- `ColorPremul` — `tests/ColorTest.cpp:19`; CPU; `DEF_TEST`.
- `SkPMColor_SetAndRetrieveChannels` — `tests/ColorTest.cpp:40`; CPU; `DEF_TEST`.
- `ColorInterp` — `tests/ColorTest.cpp:60`; CPU; `DEF_TEST`.
- `ColorFastIterp` — `tests/ColorTest.cpp:78`; CPU; `DEF_TEST`.
- `ColorToHSVRoundTrip` — `tests/HSVRoundTripTest.cpp:14`; CPU; `DEF_TEST`.
- `ParseNamedColor` — `tests/ParseColorTest.cpp:168`; CPU; `DEF_TEST`.
- `SkColor4f_FromColor` — `tests/SkColor4fTest.cpp:11`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skottie/tests/PropertyObserver.cpp` — M30.
- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/svg/tests/PaintServer.cpp` — M28.
- `tests/AAClipTest.cpp` — M14.
- `tests/AdvancedBlendTest.cpp` — M38.
- `tests/AnimatedImageTest.cpp` — M22.
- `tests/AnnotationTest.cpp` — M27.
- `tests/ApplyGammaTest.cpp` — M38.
- `tests/BackendAllocationTest.cpp` — M32, M34.
- `tests/BigImageTest.cpp` — M38.
- `tests/BitmapCopyTest.cpp` — M12.
- `tests/BitmapGetColorTest.cpp` — M11.
- `tests/BitmapTest.cpp` — M11.
- `tests/BlendTest.cpp` — M32, M34, M38.
- `tests/BlitMaskClip.cpp` — M14.
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
- `tests/CodecRecommendedTypeTest.cpp` — M17.
- `tests/CodecTest.cpp` — M17.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/ColorMatrixTest.cpp` — M14.
- `tests/ColorSpaceTest.cpp` — M16.
- `tests/CompressedBackendAllocationTest.cpp` — M38.
- `tests/ConvertPixelsTest.cpp` — M14.
- `tests/CoreBlittersTest.cpp` — M14.
- `tests/DMSAATest.cpp` — M38.
- `tests/DebugLayerManagerTest.cpp` — M40.
- `tests/DefaultPathRendererTest.cpp` — M38.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/DrawPathTest.cpp` — M14.
- `tests/DrawTextTest.cpp` — M18.
- `tests/EmptyPathTest.cpp` — M14.
- `tests/EncodeTest.cpp` — M17.
- `tests/EncodedInfoTest.cpp` — M17.
- `tests/ExtendedSkColorTypeTests.cpp` — M32, M38.
- `tests/FillPathTest.cpp` — M14.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FlattenDrawableTest.cpp` — M22.
- `tests/FlattenableFactoryToName.cpp` — M22.
- `tests/FloatingPointTextureTest.cpp` — M38.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/GLBackendSurfaceTest.cpp` — M34.
- `tests/GLFramebufferDeletionTest.cpp` — M34.
- `tests/GainmapShaderTest.cpp` — M16.
- `tests/GaneshContextRecorderTest.cpp` — M38.
- `tests/GifTest.cpp` — M16.
- `tests/GpuDrawPathTest.cpp` — M34, M38.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrDDLImageTest.cpp` — M32.
- `tests/GrFinishedFlushTest.cpp` — M38.
- `tests/GrGpuBufferTest.cpp` — M38.
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
- `tests/HighContrastFilterTest.cpp` — M13.
- `tests/ICCTest.cpp` — M17.
- `tests/ImageBitmapTest.cpp` — M14.
- `tests/ImageFilterCacheTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageFrom565Bitmap.cpp` — M14.
- `tests/ImageGeneratorOrientationTest.cpp` — M16.
- `tests/ImageGeneratorTest.cpp` — M22.
- `tests/ImageIsOpaqueTest.cpp` — M32, M38.
- `tests/ImageNewShaderTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/JpegGainmapTest.cpp` — M17.
- `tests/JpegxlTest.cpp` — M16.
- `tests/LazyProxyTest.cpp` — M32.
- `tests/LazyStencilAttachmentTest.cpp` — M38.
- `tests/MatrixColorFilterTest.cpp` — M38.
- `tests/MeshTest.cpp` — M26.
- `tests/MipMapTest.cpp` — M14.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/NonlinearBlendingTest.cpp` — M13.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/ParametricStageTest.cpp` — M12.
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
- `tests/RasterPipelineCodeGeneratorTest.cpp` — M25.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/ReadWritePixelsGpuTest.cpp` — M32, M34, M38.
- `tests/RecordDrawTest.cpp` — M22.
- `tests/RecordOptsTest.cpp` — M22.
- `tests/RecorderTest.cpp` — M22.
- `tests/RectTest.cpp` — M14.
- `tests/RectangleTextureTest.cpp` — M34.
- `tests/RepeatedClippedBlurTest.cpp` — M38.
- `tests/ResourceAllocatorTest.cpp` — M38.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RuntimeBlendTest.cpp` — M35, M38.
- `tests/RustIccTest.cpp` — M17.
- `tests/SRGBReadWritePixelsTest.cpp` — M38.
- `tests/SRGBTest.cpp` — M12.
- `tests/SVGDeviceTest.cpp` — M27.
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
- `tests/SkXmpTest.cpp` — M16.
- `tests/SkYUVAInfoTest.cpp` — M16.
- `tests/Skbug6653.cpp` — M38.
- `tests/SlugTest.cpp` — M38.
- `tests/SpecialImageTest.cpp` — M32, M38.
- `tests/SrcOverTest.cpp` — M14.
- `tests/SrcSrcOverBatchTest.cpp` — M38.
- `tests/SurfaceDrawContextTest.cpp` — M38.
- `tests/SurfaceSemaphoreTest.cpp` — M32, M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TextureBindingsResetTest.cpp` — M34.
- `tests/TextureSizeTest.cpp` — M38.
- `tests/TextureStripAtlasManagerTest.cpp` — M38.
- `tests/TransferPixelsTest.cpp` — M38.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/VerticesTest.cpp` — M22.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp` — M38.
- `tests/WritePixelsTest.cpp` — M35, M38.
- `tests/crbug_500172224.cpp` — M38.
- `tests/graphite/BackendTextureTest.cpp` — M38.
- `tests/graphite/CacheKeyTest.cpp` — M38.
- `tests/graphite/GraphiteContextRecorderTest.cpp` — M38.
- `tests/graphite/GraphitePromiseImageTest.cpp` — M38.
- `tests/graphite/GraphiteYUVAPromiseImageTest.cpp` — M38.
- `tests/graphite/ImageOriginTest.cpp` — M38.
- `tests/graphite/ImageProviderTest.cpp` — M38.
- `tests/graphite/MutableImagesTest.cpp` — M38.
- `tests/graphite/NotifyInUseTest.cpp` — M38.
- `tests/graphite/ReadWritePixelsGraphiteTest.cpp` — M38.
- `tests/graphite/StorageContextTest.cpp` — M38.
- `tests/graphite/SubmitWithFinishProcTest.cpp` — M38.
- `tests/graphite/TextureFallbackTest.cpp` — M38.
- `tests/graphite/TextureFormatTest.cpp` — M38.
- `tests/graphite/TextureProxyTest.cpp` — M38.
- `tests/graphite/UpdateBackendTextureTest.cpp` — M38.
- `tests/graphite/crbug_513836996.cpp` — M38.
- `tests/graphite/precompile/CombinationBuilderTest.cpp` — M38.
- `tests/graphite/precompile/UserdefinedStableKeyTest.cpp` — M38.

### Preserve these fixtures, metadata and references

- `modules/skcms/LICENSE` — preserve.
- `modules/skcms/OWNERS` — preserve.
- `modules/skcms/README.chromium` — preserve.
- `modules/skcms/version.sha1` — preserve.

## Dependency and integration notes

The initial scan found 10 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=10`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M10 ACCEPTED may the user hand Deepseek M11.
