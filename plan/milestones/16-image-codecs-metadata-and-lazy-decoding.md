# Milestone 16: Image codecs, metadata and lazy decoding

## Outcome and entry gate

Implement the Skia codec abstraction, incremental/animated decoding, encoded metadata, orientation, gainmaps and lazy-image decode integration. Cover each codec eligible on macOS, including the checkout's existing Rust codec modules. External codec libraries may be dependencies, but Skia adapters and policy logic must be owned Rust.

Begin implementation only after milestone 15 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement the Skia codec abstraction, incremental/animated decoding, encoded metadata, orientation, gainmaps and lazy-image decode integration. Cover each codec eligible on macOS, including the checkout's existing Rust codec modules. External codec libraries may be dependencies, but Skia adapters and policy logic must be owned Rust.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/codec/SkAvifDecoder.h` — port to Rust.
- `include/codec/SkBmpDecoder.h` — port to Rust.
- `include/codec/SkBmpRustDecoder.h` — port to Rust.
- `include/codec/SkCodec.h` — port to Rust.
- `include/codec/SkCodecAnimation.h` — port to Rust.
- `include/codec/SkEncodedImageFormat.h` — port to Rust.
- `include/codec/SkEncodedOrigin.h` — port to Rust.
- `include/codec/SkGifDecoder.h` — port to Rust.
- `include/codec/SkIcoDecoder.h` — port to Rust.
- `include/codec/SkJpegDecoder.h` — port to Rust.
- `include/codec/SkJpegxlDecoder.h` — port to Rust.
- `include/codec/SkPixmapUtils.h` — port to Rust.
- `include/codec/SkPngChunkReader.h` — port to Rust.
- `include/codec/SkPngDecoder.h` — port to Rust.
- `include/codec/SkPngRustDecoder.h` — port to Rust.
- `include/codec/SkRawDecoder.h` — port to Rust.
- `include/codec/SkWbmpDecoder.h` — port to Rust.
- `include/codec/SkWebpDecoder.h` — port to Rust.
- `include/core/SkImageGenerator.h` — port to Rust.
- `include/core/SkOpenTypeSVGDecoder.h` — port to Rust.
- `include/docs/SkPDFJpegHelpers.h` — port to Rust.
- `include/ports/SkImageGeneratorCG.h` — port to Rust.
- `include/private/SkExif.h` — port to Rust.
- `include/private/SkGainmapInfo.h` — port to Rust.
- `include/private/SkHdrMetadata.h` — port to Rust.
- `include/private/SkJpegMetadataDecoder.h` — port to Rust.
- `include/private/SkXmp.h` — port to Rust.
- `rust/bmp/FFI.rs` — integrate existing Rust.
- `rust/common/SkStreamAdapter.cpp` — port to Rust.
- `rust/common/SkStreamAdapter.h` — port to Rust.
- `rust/common/SpanUtils.h` — port to Rust.
- `rust/common/io_traits_ffi.rs` — integrate existing Rust.
- `rust/exif/FFI.cpp` — port to Rust.
- `rust/exif/FFI.h` — port to Rust.
- `rust/exif/FFI.rs` — integrate existing Rust.
- `rust/exif/exif_parse.rs` — integrate existing Rust.
- `rust/icc/FFI.cpp` — port to Rust.
- `rust/icc/FFI.h` — port to Rust.
- `rust/icc/FFI.rs` — integrate existing Rust.
- `rust/icc/icc_parse.rs` — integrate existing Rust.
- `rust/png/FFI.h` — port to Rust.
- `rust/png/FFI.rs` — integrate existing Rust.
- `src/codec/SkAvifCodec.cpp` — port to Rust.
- `src/codec/SkAvifCodec.h` — port to Rust.
- `src/codec/SkBmpBaseCodec.cpp` — port to Rust.
- `src/codec/SkBmpBaseCodec.h` — port to Rust.
- `src/codec/SkBmpCodec.cpp` — port to Rust.
- `src/codec/SkBmpCodec.h` — port to Rust.
- `src/codec/SkBmpMaskCodec.cpp` — port to Rust.
- `src/codec/SkBmpMaskCodec.h` — port to Rust.
- `src/codec/SkBmpRLECodec.cpp` — port to Rust.
- `src/codec/SkBmpRLECodec.h` — port to Rust.
- `src/codec/SkBmpRustCodec.cpp` — port to Rust.
- `src/codec/SkBmpRustCodec.h` — port to Rust.
- `src/codec/SkBmpRustDecoder.cpp` — port to Rust.
- `src/codec/SkBmpStandardCodec.cpp` — port to Rust.
- `src/codec/SkBmpStandardCodec.h` — port to Rust.
- `src/codec/SkCodec.cpp` — port to Rust.
- `src/codec/SkCodecColorProfile.cpp` — port to Rust.
- `src/codec/SkCodecColorProfileRust.cpp` — port to Rust.
- `src/codec/SkCodecColorProfileRust.h` — port to Rust.
- `src/codec/SkCodecImageGenerator.cpp` — port to Rust.
- `src/codec/SkCodecImageGenerator.h` — port to Rust.
- `src/codec/SkCodecPriv.h` — port to Rust.
- `src/codec/SkColorPalette.cpp` — port to Rust.
- `src/codec/SkColorPalette.h` — port to Rust.
- `src/codec/SkCrabbyAvifCodec.cpp` — port to Rust.
- `src/codec/SkCrabbyAvifCodec.h` — port to Rust.
- `src/codec/SkEncodedInfo.cpp` — port to Rust.
- `src/codec/SkExif.cpp` — port to Rust.
- `src/codec/SkFrameHolder.h` — port to Rust.
- `src/codec/SkGainmapInfo.cpp` — port to Rust.
- `src/codec/SkHdrAgtm.cpp` — port to Rust.
- `src/codec/SkHdrAgtmParse.cpp` — port to Rust.
- `src/codec/SkHdrAgtmPriv.h` — port to Rust.
- `src/codec/SkHdrMetadata.cpp` — port to Rust.
- `src/codec/SkIcoCodec.cpp` — port to Rust.
- `src/codec/SkIcoCodec.h` — port to Rust.
- `src/codec/SkImageGenerator_FromEncoded.cpp` — port to Rust.
- `src/codec/SkJpegCodec.cpp` — port to Rust.
- `src/codec/SkJpegCodec.h` — port to Rust.
- `src/codec/SkJpegConstants.h` — port to Rust.
- `src/codec/SkJpegDecoderMgr.cpp` — port to Rust.
- `src/codec/SkJpegDecoderMgr.h` — port to Rust.
- `src/codec/SkJpegMetadataDecoderImpl.cpp` — port to Rust.
- `src/codec/SkJpegMetadataDecoderImpl.h` — port to Rust.
- `src/codec/SkJpegMultiPicture.cpp` — port to Rust.
- `src/codec/SkJpegMultiPicture.h` — port to Rust.
- `src/codec/SkJpegPriv.h` — port to Rust.
- `src/codec/SkJpegSegmentScan.cpp` — port to Rust.
- `src/codec/SkJpegSegmentScan.h` — port to Rust.
- `src/codec/SkJpegSourceMgr.cpp` — port to Rust.
- `src/codec/SkJpegSourceMgr.h` — port to Rust.
- `src/codec/SkJpegUtility.cpp` — port to Rust.
- `src/codec/SkJpegUtility.h` — port to Rust.
- `src/codec/SkJpegXmp.cpp` — port to Rust.
- `src/codec/SkJpegXmp.h` — port to Rust.
- `src/codec/SkJpegxlCodec.cpp` — port to Rust.
- `src/codec/SkJpegxlCodec.h` — port to Rust.
- `src/codec/SkMaskSwizzler.cpp` — port to Rust.
- `src/codec/SkMaskSwizzler.h` — port to Rust.
- `src/codec/SkParseEncodedOrigin.cpp` — port to Rust.
- `src/codec/SkParseEncodedOrigin.h` — port to Rust.
- `src/codec/SkPixmapUtils.cpp` — port to Rust.
- `src/codec/SkPixmapUtilsPriv.h` — port to Rust.
- `src/codec/SkPngCodec.cpp` — port to Rust.
- `src/codec/SkPngCodec.h` — port to Rust.
- `src/codec/SkPngCodecBase.cpp` — port to Rust.
- `src/codec/SkPngCodecBase.h` — port to Rust.
- `src/codec/SkPngCompositeChunkReader.cpp` — port to Rust.
- `src/codec/SkPngCompositeChunkReader.h` — port to Rust.
- `src/codec/SkPngPriv.h` — port to Rust.
- `src/codec/SkPngRustCodec.cpp` — port to Rust.
- `src/codec/SkPngRustCodec.h` — port to Rust.
- `src/codec/SkPngRustDecoder.cpp` — port to Rust.
- `src/codec/SkRawCodec.cpp` — port to Rust.
- `src/codec/SkRawCodec.h` — port to Rust.
- `src/codec/SkSampledCodec.cpp` — port to Rust.
- `src/codec/SkSampledCodec.h` — port to Rust.
- `src/codec/SkSampler.cpp` — port to Rust.
- `src/codec/SkSampler.h` — port to Rust.
- `src/codec/SkScalingCodec.h` — port to Rust.
- `src/codec/SkSwizzler.cpp` — port to Rust.
- `src/codec/SkSwizzler.h` — port to Rust.
- `src/codec/SkTiffUtility.cpp` — port to Rust.
- `src/codec/SkTiffUtility.h` — port to Rust.
- `src/codec/SkWbmpCodec.cpp` — port to Rust.
- `src/codec/SkWbmpCodec.h` — port to Rust.
- `src/codec/SkWebpCodec.cpp` — port to Rust.
- `src/codec/SkWebpCodec.h` — port to Rust.
- `src/codec/SkWuffsCodec.cpp` — port to Rust.
- `src/codec/SkXmp.cpp` — port to Rust.
- `src/core/SkImageGenerator.cpp` — port to Rust.
- `src/image/SkImageGeneratorPriv.h` — port to Rust.
- `src/image/SkImage_Lazy.cpp` — port to Rust.
- `src/image/SkImage_Lazy.h` — port to Rust.
- `src/image/SkImage_LazyFactories.cpp` — port to Rust.
- `src/lazy/SkDiscardableMemoryPool.cpp` — port to Rust.
- `src/lazy/SkDiscardableMemoryPool.h` — port to Rust.
- `src/ports/SkImageGeneratorCG.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/codec/BUILD.bazel`.
- `rust/bmp/BUILD.bazel`.
- `rust/common/BUILD.bazel`.
- `rust/exif/BUILD.bazel`.
- `rust/icc/BUILD.bazel`.
- `rust/png/BUILD.bazel`.
- `src/codec/BUILD.bazel`.
- `src/lazy/BUILD.bazel`.

## Codex: acceptance-test contract

Translate all applicable decoder, malformed/truncated-input, animation, EXIF, ICC, gainmap and codec-selection tests. Preserve resource bytes and expected status codes, frame dependencies and read counts. Exercise the same external dependency versions as the C++ reference or document and resolve observable differences.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/AvifTest.cpp`.
- `tests/BRDTest.cpp`.
- `tests/BadIcoTest.cpp`.
- `tests/CachedDataTest.cpp`.
- `tests/CachedDecodingPixelRefTest.cpp`.
- `tests/CodecExactReadTest.cpp`.
- `tests/CodecPartialTest.cpp`.
- `tests/ColorSpaceTest.cpp`.
- `tests/DiscardableMemoryPoolTest.cpp`.
- `tests/DiscardableMemoryTest.cpp`.
- `tests/FrontBufferedStreamTest.cpp`.
- `tests/GainmapShaderTest.cpp`.
- `tests/GainmapTestCommon.h`.
- `tests/GifTest.cpp`.
- `tests/HdrMetadataTest.cpp`.
- `tests/ImageCacheTest.cpp`.
- `tests/ImageGeneratorOrientationTest.cpp`.
- `tests/IndexedPngOverflowTest.cpp`.
- `tests/InvalidIndexedPngTest.cpp`.
- `tests/JpegxlTest.cpp`.
- `tests/RustExifTest.cpp`.
- `tests/SkBmpRustDecoderTest.cpp`.
- `tests/SkIcoRustDecoderTest.cpp`.
- `tests/SkJpegXmpTest.cpp`.
- `tests/SkPngRustDecoderTest.cpp`.
- `tests/SkXmpTest.cpp`.
- `tests/SkYUVAInfoTest.cpp`.
- `tests/SwizzlerTest.cpp`.
- `tests/WebpTest.cpp`.

### Original test registrations assigned to this gate

- `AvifDecodeBasic` — `tests/AvifTest.cpp:77`; CPU; `DEF_TEST`.
- `AvifDecodeOddDimensions` — `tests/AvifTest.cpp:90`; CPU; `DEF_TEST`.
- `AvifDecodeScaleDown` — `tests/AvifTest.cpp:103`; CPU; `DEF_TEST`.
- `AvifDecode10BitToRGBA8888Bitmap` — `tests/AvifTest.cpp:116`; CPU; `DEF_TEST`.
- `AvifDecode10BitToRGBAF16Bitmap` — `tests/AvifTest.cpp:129`; CPU; `DEF_TEST`.
- `AvifDecode10BitToRGBAF16BitmapDownscale` — `tests/AvifTest.cpp:142`; CPU; `DEF_TEST`.
- `AvifDecode12BitToRGBA8888Bitmap` — `tests/AvifTest.cpp:155`; CPU; `DEF_TEST`.
- `AvifDecode12BitToRGBAF16Bitmap` — `tests/AvifTest.cpp:168`; CPU; `DEF_TEST`.
- `AvifDecode12BitToRGBAF16BitmapDownscale` — `tests/AvifTest.cpp:181`; CPU; `DEF_TEST`.
- `AvifDecodeImageWithAlpha` — `tests/AvifTest.cpp:194`; CPU; `DEF_TEST`.
- `AvifDecodeAnimation` — `tests/AvifTest.cpp:207`; CPU; `DEF_TEST`.
- `AvifDecodeAnimationWithAlpha` — `tests/AvifTest.cpp:220`; CPU; `DEF_TEST`.
- `AvifDecodeMonochrome` — `tests/AvifTest.cpp:233`; CPU; `DEF_TEST`.
- `BRD_types` — `tests/BRDTest.cpp:18`; CPU; `DEF_TEST`.
- `BadImage` — `tests/BadIcoTest.cpp:33`; CPU; `DEF_TEST`.
- `Ico_malformedPngWithRustDecoderDoesNotFallback` — `tests/BadIcoTest.cpp:115`; CPU; `DEF_SERIAL_TEST`.
- `Ico_malformedPngWithLibpngNoRedundantFallback` — `tests/BadIcoTest.cpp:121`; CPU; `DEF_SERIAL_TEST`.
- `CachedData` — `tests/CachedDataTest.cpp:81`; CPU; `DEF_TEST`.
- `Image_NewFromGenerator` — `tests/CachedDecodingPixelRefTest.cpp:94`; CPU; `DEF_TEST`.
- `Codec_end` — `tests/CodecExactReadTest.cpp:53`; CPU; `DEF_TEST`.
- `Codec_partial` — `tests/CodecPartialTest.cpp:139`; CPU; `DEF_TEST`.
- `Codec_partialWuffs` — `tests/CodecPartialTest.cpp:159`; CPU; `DEF_TEST`.
- `Codec_requiredFrame` — `tests/CodecPartialTest.cpp:176`; CPU; `DEF_TEST`.
- `Codec_partialAnim` — `tests/CodecPartialTest.cpp:227`; CPU; `DEF_TEST`.
- `Codec_rewind` — `tests/CodecPartialTest.cpp:368`; CPU; `DEF_TEST`.
- `Codec_GifPreMap` — `tests/CodecPartialTest.cpp:399`; CPU; `DEF_TEST`.
- `Codec_emptyIDAT` — `tests/CodecPartialTest.cpp:452`; CPU; `DEF_TEST`.
- `Codec_incomplete` — `tests/CodecPartialTest.cpp:476`; CPU; `DEF_TEST`.
- `ColorSpace_sRGB` — `tests/ColorSpaceTest.cpp:74`; CPU; `DEF_TEST`.
- `ColorSpaceParseICCProfiles` — `tests/ColorSpaceTest.cpp:79`; CPU; `DEF_TEST`.
- `ColorSpace_Serialize` — `tests/ColorSpaceTest.cpp:129`; CPU; `DEF_TEST`.
- `ColorSpace_Equals` — `tests/ColorSpaceTest.cpp:163`; CPU; `DEF_TEST`.
- `ColorSpace_Primaries` — `tests/ColorSpaceTest.cpp:228`; CPU; `DEF_TEST`.
- `ColorSpace_MatrixHash` — `tests/ColorSpaceTest.cpp:306`; CPU; `DEF_TEST`.
- `ColorSpace_IsSRGB` — `tests/ColorSpaceTest.cpp:323`; CPU; `DEF_TEST`.
- `ColorSpace_skcms_IsSRGB` — `tests/ColorSpaceTest.cpp:340`; CPU; `DEF_TEST`.
- `ColorSpace_skcms_sRGB_exact` — `tests/ColorSpaceTest.cpp:345`; CPU; `DEF_TEST`.
- `ColorSpace_classifyUnderflow` — `tests/ColorSpaceTest.cpp:352`; CPU; `DEF_TEST`.
- `ColorSpace_equiv` — `tests/ColorSpaceTest.cpp:366`; CPU; `DEF_TEST`.
- `DiscardableMemoryPool` — `tests/DiscardableMemoryPoolTest.cpp:15`; CPU; `DEF_TEST`.
- `DiscardableMemory_global` — `tests/DiscardableMemoryTest.cpp:51`; CPU; `DEF_TEST`.
- `DiscardableMemory_nonglobal` — `tests/DiscardableMemoryTest.cpp:58`; CPU; `DEF_TEST`.
- `FrontBufferedStream` — `tests/FrontBufferedStreamTest.cpp:263`; CPU; `DEF_TEST`.
- `ShortFrontBufferedStream` — `tests/FrontBufferedStreamTest.cpp:293`; CPU; `DEF_TEST`.
- `GainmapShader_rects` — `tests/GainmapShaderTest.cpp:99`; CPU; `DEF_TEST`.
- `GainmapShader_baseImageIsHdr` — `tests/GainmapShaderTest.cpp:188`; CPU; `DEF_TEST`.
- `GainmapShader_colorSpace` — `tests/GainmapShaderTest.cpp:279`; CPU; `DEF_TEST`.
- `GainmapShader_apple` — `tests/GainmapShaderTest.cpp:317`; CPU; `DEF_TEST`.
- `Gif` — `tests/GifTest.cpp:165`; CPU; `DEF_TEST`.
- `Codec_GifInterlacedTruncated` — `tests/GifTest.cpp:213`; CPU; `DEF_TEST`.
- `Gif_Sampled` — `tests/GifTest.cpp:261`; CPU; `DEF_TEST`.
- `Codec_GifTruncated` — `tests/GifTest.cpp:286`; CPU; `DEF_TEST`.
- `Codec_GifTruncated2` — `tests/GifTest.cpp:323`; CPU; `DEF_TEST`.
- `Codec_GifTruncated3` — `tests/GifTest.cpp:379`; CPU; `DEF_TEST`.
- `Codec_gif_out_of_palette` — `tests/GifTest.cpp:418`; CPU; `DEF_TEST`.
- `Codec_AnimatedTransparentGif` — `tests/GifTest.cpp:466`; CPU; `DEF_TEST`.
- `Codec_xOffsetTooBig` — `tests/GifTest.cpp:545`; CPU; `DEF_TEST`.
- `HdrMetadata_ParseSerialize_ContentLightLevelInformation` — `tests/HdrMetadataTest.cpp:20`; CPU; `DEF_TEST`.
- `HdrMetadata_ParseSerialize_MasteringDisplayColorVolume` — `tests/HdrMetadataTest.cpp:49`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_Cubic` — `tests/HdrMetadataTest.cpp:75`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_PCHIP_EdgeCases` — `tests/HdrMetadataTest.cpp:112`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_Mix` — `tests/HdrMetadataTest.cpp:228`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_RWTMO` — `tests/HdrMetadataTest.cpp:283`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_Weighting` — `tests/HdrMetadataTest.cpp:323`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_RoundTripSerialize` — `tests/HdrMetadataTest.cpp:501`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_Apply_and_Shader` — `tests/HdrMetadataTest.cpp:951`; CPU; `DEF_TEST`.
- `HdrMetadata_ShaderParams` — `tests/HdrMetadataTest.cpp:1136`; CPU; `DEF_TEST`.
- `HdrMetadata_Agtm_Invalid` — `tests/HdrMetadataTest.cpp:1243`; CPU; `DEF_TEST`.
- `ImageCache` — `tests/ImageCacheTest.cpp:120`; CPU; `DEF_TEST`.
- `ImageCache_doubleAdd` — `tests/ImageCacheTest.cpp:143`; CPU; `DEF_TEST`.
- `ImageGeneratorOrientationTest` — `tests/ImageGeneratorOrientationTest.cpp:40`; CPU; `DEF_TEST`.
- `IndexedPngOverflow` — `tests/IndexedPngOverflowTest.cpp:32`; CPU; `DEF_TEST`.
- `InvalidIndexedPng` — `tests/InvalidIndexedPngTest.cpp:30`; CPU; `DEF_TEST`.
- `Jpegxl_DecodeP3` — `tests/JpegxlTest.cpp:27`; CPU; `DEF_TEST`.
- `Jpegxl_Decode1010102` — `tests/JpegxlTest.cpp:83`; CPU; `DEF_TEST`.
- `Jpegxl_AlphaPremulAndUnpremul` — `tests/JpegxlTest.cpp:111`; CPU; `DEF_TEST`.
- `RustExif_empty_data_returns_false` — `tests/RustExifTest.cpp:145`; CPU; `DEF_TEST`.
- `RustExif_invalid_data_returns_false` — `tests/RustExifTest.cpp:151`; CPU; `DEF_TEST`.
- `RustExif_orientation_parsing` — `tests/RustExifTest.cpp:159`; CPU; `DEF_TEST`.
- `RustExif_orientation_out_of_range_ignored` — `tests/RustExifTest.cpp:177`; CPU; `DEF_TEST`.
- `RustExif_little_endian_parsing` — `tests/RustExifTest.cpp:190`; CPU; `DEF_TEST`.
- `RustExif_resolution_parsing` — `tests/RustExifTest.cpp:204`; CPU; `DEF_TEST`.
- `RustExif_pixel_dimensions_parsing` — `tests/RustExifTest.cpp:254`; CPU; `DEF_TEST`.
- `RustExif_equivalence_with_resource_files` — `tests/RustExifTest.cpp:407`; CPU; `DEF_TEST`.
- `RustBmpCodec_decode_valid_bmp` — `tests/SkBmpRustDecoderTest.cpp:123`; CPU; `DEF_TEST`.
- `RustBmpCodec_nullptr_result` — `tests/SkBmpRustDecoderTest.cpp:161`; CPU; `DEF_TEST`.
- `RustBmpCodec_reject_non_bmp` — `tests/SkBmpRustDecoderTest.cpp:173`; CPU; `DEF_TEST`.
- `RustBmpCodec_rewind` — `tests/SkBmpRustDecoderTest.cpp:189`; CPU; `DEF_TEST`.
- `RustBmpCodec_getPixels` — `tests/SkBmpRustDecoderTest.cpp:228`; CPU; `DEF_TEST`.
- `RustBmpCodec_IsBmp_positive` — `tests/SkBmpRustDecoderTest.cpp:254`; CPU; `DEF_TEST`.
- `RustBmpCodec_IsBmp_negative` — `tests/SkBmpRustDecoderTest.cpp:266`; CPU; `DEF_TEST`.
- `RustBmpCodec_IsBmp_insufficient_data` — `tests/SkBmpRustDecoderTest.cpp:278`; CPU; `DEF_TEST`.
- `RustBmpCodec_invalid_bmp_handling` — `tests/SkBmpRustDecoderTest.cpp:286`; CPU; `DEF_TEST`.
- `RustBmpCodec_explicit_rewind` — `tests/SkBmpRustDecoderTest.cpp:317`; CPU; `DEF_TEST`.
- `RustBmpCodec_IncrementalDecode` — `tests/SkBmpRustDecoderTest.cpp:350`; CPU; `DEF_TEST`.
- `RustBmpCodec_IncrementalDecode_PartialStreaming` — `tests/SkBmpRustDecoderTest.cpp:417`; CPU; `DEF_TEST`.
- `RustBmpCodec_IncompleteMetadata_PartialHeader` — `tests/SkBmpRustDecoderTest.cpp:622`; CPU; `DEF_TEST`.
- `RustBmpCodec_IncompleteRowData` — `tests/SkBmpRustDecoderTest.cpp:647`; CPU; `DEF_TEST`.
- `RustBmpCodec_ICCProfile` — `tests/SkBmpRustDecoderTest.cpp:706`; CPU; `DEF_TEST`.
- `RustBmpCodec_ICCProfileTransform` — `tests/SkBmpRustDecoderTest.cpp:750`; CPU; `DEF_TEST`.
- `RustIcoCodec_decode_multi_bmp` — `tests/SkIcoRustDecoderTest.cpp:62`; CPU; `DEF_TEST`.
- `RustIcoCodec_alternative_dimensions` — `tests/SkIcoRustDecoderTest.cpp:81`; CPU; `DEF_TEST`.
- `RustIcoCodec_indexed_full_canvas` — `tests/SkIcoRustDecoderTest.cpp:135`; CPU; `DEF_TEST`.
- `RustIcoCodec_matches_legacy_size_selection` — `tests/SkIcoRustDecoderTest.cpp:200`; CPU; `DEF_TEST`.
- `RustIcoCodec_decode_with_png` — `tests/SkIcoRustDecoderTest.cpp:247`; CPU; `DEF_TEST`.
- `RustIcoCodec_nullptr_result` — `tests/SkIcoRustDecoderTest.cpp:268`; CPU; `DEF_TEST`.
- `RustIcoCodec_reject_non_ico` — `tests/SkIcoRustDecoderTest.cpp:279`; CPU; `DEF_TEST`.
- `RustIcoCodec_rewind` — `tests/SkIcoRustDecoderTest.cpp:295`; CPU; `DEF_TEST`.
- `RustIcoCodec_IsIco_positive` — `tests/SkIcoRustDecoderTest.cpp:334`; CPU; `DEF_TEST`.
- `RustIcoCodec_IsIco_negative` — `tests/SkIcoRustDecoderTest.cpp:346`; CPU; `DEF_TEST`.
- `RustIcoCodec_IsIco_insufficient_data` — `tests/SkIcoRustDecoderTest.cpp:358`; CPU; `DEF_TEST`.
- `RustIcoCodec_IsIco_cursor` — `tests/SkIcoRustDecoderTest.cpp:366`; CPU; `DEF_TEST`.
- `RustIcoCodec_cursor_hotspots` — `tests/SkIcoRustDecoderTest.cpp:373`; CPU; `DEF_TEST`.
- `RustIcoCodec_invalid_ico_handling` — `tests/SkIcoRustDecoderTest.cpp:554`; CPU; `DEF_TEST`.
- `RustIcoCodec_accept_directory_payload_dimension_mismatch` — `tests/SkIcoRustDecoderTest.cpp:589`; CPU; `DEF_TEST`.
- `RustIcoCodec_partial_directory_payload_dimension_mismatch` — `tests/SkIcoRustDecoderTest.cpp:633`; CPU; `DEF_TEST`.
- `RustIcoCodec_incomplete_before_first_payload` — `tests/SkIcoRustDecoderTest.cpp:673`; CPU; `DEF_TEST`.
- `RustIcoCodec_metadata_only_entry_preserves_size` — `tests/SkIcoRustDecoderTest.cpp:692`; CPU; `DEF_TEST`.
- `RustIcoCodec_mixed_metadata_only_entry` — `tests/SkIcoRustDecoderTest.cpp:739`; CPU; `DEF_TEST`.
- `RustIcoCodec_getPixels` — `tests/SkIcoRustDecoderTest.cpp:795`; CPU; `DEF_TEST`.
- `RustIcoCodec_explicit_rewind` — `tests/SkIcoRustDecoderTest.cpp:814`; CPU; `DEF_TEST`.
- `RustIcoCodec_nonstandard_dimensions` — `tests/SkIcoRustDecoderTest.cpp:847`; CPU; `DEF_TEST`.
- `RustIcoCodec_oversized_payload_is_metadata_only` — `tests/SkIcoRustDecoderTest.cpp:884`; CPU; `DEF_TEST`.
- `RustIcoCodec_IncrementalDecode_PartialStreaming` — `tests/SkIcoRustDecoderTest.cpp:912`; CPU; `DEF_TEST`.
- `RustIcoCodec_IncrementalDecode_API` — `tests/SkIcoRustDecoderTest.cpp:977`; CPU; `DEF_TEST`.
- `RustIcoCodec_truncated_all_entries_missing` — `tests/SkIcoRustDecoderTest.cpp:1025`; CPU; `DEF_TEST`.
- `RustIcoCodec_truncated_some_entries_missing` — `tests/SkIcoRustDecoderTest.cpp:1059`; CPU; `DEF_TEST`.
- `SkJpegXmp_standardXmp` — `tests/SkJpegXmpTest.cpp:19`; CPU; `DEF_TEST`.
- `SkJpegXmp_defaultValues` — `tests/SkJpegXmpTest.cpp:46`; CPU; `DEF_TEST`.
- `SkJpegXmp_readExtendedXmp` — `tests/SkJpegXmpTest.cpp:107`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_basic_trivial_static_image` — `tests/SkPngRustDecoderTest.cpp:750`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_basic_using_default_image` — `tests/SkPngRustDecoderTest.cpp:756`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_basic_ignoring_default_image` — `tests/SkPngRustDecoderTest.cpp:776`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_ignoring_default_image_without_getFrameCount` — `tests/SkPngRustDecoderTest.cpp:792`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_dispose_op_none_basic` — `tests/SkPngRustDecoderTest.cpp:864`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_dispose_op_none_basic_incomplete_input1` — `tests/SkPngRustDecoderTest.cpp:935`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_dispose_op_none_basic_incomplete_input2` — `tests/SkPngRustDecoderTest.cpp:992`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_blend_ops_source_on_solid` — `tests/SkPngRustDecoderTest.cpp:1075`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_blend_ops_source_on_nearly_transparent` — `tests/SkPngRustDecoderTest.cpp:1102`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_blend_ops_over_on_solid_and_transparent` — `tests/SkPngRustDecoderTest.cpp:1125`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_blend_ops_over_repeatedly` — `tests/SkPngRustDecoderTest.cpp:1143`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_regions_dispose_op_none` — `tests/SkPngRustDecoderTest.cpp:1161`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_num_plays_0` — `tests/SkPngRustDecoderTest.cpp:1188`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_num_plays_1` — `tests/SkPngRustDecoderTest.cpp:1199`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_num_plays_2` — `tests/SkPngRustDecoderTest.cpp:1210`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_invalid_num_frames_outside_valid_range` — `tests/SkPngRustDecoderTest.cpp:1227`; CPU; `DEF_TEST`.
- `RustPngCodec_png_swizzling_target_unimplemented` — `tests/SkPngRustDecoderTest.cpp:1242`; CPU; `DEF_TEST`.
- `RustPngCodec_png_was_encoded_with_16_bits_or_more_per_component` — `tests/SkPngRustDecoderTest.cpp:1262`; CPU; `DEF_TEST`.
- `RustPngCodec_png_cicp` — `tests/SkPngRustDecoderTest.cpp:1281`; CPU; `DEF_TEST`.
- `RustPngCodec_green15x15` — `tests/SkPngRustDecoderTest.cpp:1300`; CPU; `DEF_TEST`.
- `RustPngCodec_exif_orientation` — `tests/SkPngRustDecoderTest.cpp:1320`; CPU; `DEF_TEST`.
- `RustPngCodec_f16_trc_tables` — `tests/SkPngRustDecoderTest.cpp:1329`; CPU; `DEF_TEST`.
- `RustPngCodec_crbug445556737` — `tests/SkPngRustDecoderTest.cpp:1344`; CPU; `DEF_TEST`.
- `RustPngCodec_invalid_profile` — `tests/SkPngRustDecoderTest.cpp:1361`; CPU; `DEF_TEST`.
- `RustPngCodec_subset` — `tests/SkPngRustDecoderTest.cpp:1448`; CPU; `DEF_TEST`.
- `RustPngCodec_interlaced_animated_blending` — `tests/SkPngRustDecoderTest.cpp:1456`; CPU; `DEF_TEST`.
- `RustPngCodec_sbit565_ihdr16bits` — `tests/SkPngRustDecoderTest.cpp:1481`; CPU; `DEF_TEST`.
- `RustPngCodec_gainmapDecode` — `tests/SkPngRustDecoderTest.cpp:1507`; CPU; `DEF_TEST`.
- `RustPngCodec_gainmapRewind` — `tests/SkPngRustDecoderTest.cpp:1546`; CPU; `DEF_TEST`.
- `RustPngCodec_ninepatchPngChunkReader` — `tests/SkPngRustDecoderTest.cpp:1573`; CPU; `DEF_TEST`.
- `RustPngCodec_exactRead` — `tests/SkPngRustDecoderTest.cpp:1606`; CPU; `DEF_TEST`.
- `RustPngCodec_gainmapNoOp` — `tests/SkPngRustDecoderTest.cpp:1663`; CPU; `DEF_TEST`.
- `RustPngCodec_exactRead_overshoot` — `tests/SkPngRustDecoderTest.cpp:1684`; CPU; `DEF_TEST`.
- `RustPngCodec_subset_halting` — `tests/SkPngRustDecoderTest.cpp:1730`; CPU; `DEF_TEST`.
- `RustPngCodec_subsampling` — `tests/SkPngRustDecoderTest.cpp:1767`; CPU; `DEF_TEST`.
- `RustPngCodec_subsampling_interlaced` — `tests/SkPngRustDecoderTest.cpp:1773`; CPU; `DEF_TEST`.
- `RustPngCodec_subsampling_subset` — `tests/SkPngRustDecoderTest.cpp:1779`; CPU; `DEF_TEST`.
- `RustPngCodec_subsampling_subset_interlaced` — `tests/SkPngRustDecoderTest.cpp:1787`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_offset_frame_does_not_overflow_dst` — `tests/SkPngRustDecoderTest.cpp:1809`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_offset_frame_small_row_bytes` — `tests/SkPngRustDecoderTest.cpp:1855`; CPU; `DEF_TEST`.
- `RustPngCodec_apng_offset_frame_full_canvas_placement` — `tests/SkPngRustDecoderTest.cpp:1886`; CPU; `DEF_TEST`.
- `RustPngCodec_subsetDoesNotReadToIend` — `tests/SkPngRustDecoderTest.cpp:1968`; CPU; `DEF_TEST`.
- `RustPngCodec_subsetDoesNotReadToIend_interlaced` — `tests/SkPngRustDecoderTest.cpp:1977`; CPU; `DEF_TEST`.
- `RustPngCodec_subsamplingDoesNotReadToIend` — `tests/SkPngRustDecoderTest.cpp:1986`; CPU; `DEF_TEST`.
- `RustPngCodec_subsamplingDoesNotReadToIend_interlaced` — `tests/SkPngRustDecoderTest.cpp:1991`; CPU; `DEF_TEST`.
- `RustPngCodec_subsetOfTruncatedFile` — `tests/SkPngRustDecoderTest.cpp:1998`; CPU; `DEF_TEST`.
- `RustPngCodec_missingOrCorruptIendSucceeds` — `tests/SkPngRustDecoderTest.cpp:2030`; CPU; `DEF_TEST`.
- `RustPngCodec_missingOrCorruptIendSucceeds_interlaced` — `tests/SkPngRustDecoderTest.cpp:2034`; CPU; `DEF_TEST`.
- `RustPngCodec_missingOrCorruptIendSucceeds_readRow` — `tests/SkPngRustDecoderTest.cpp:2040`; CPU; `DEF_TEST`.
- `RustPngCodec_missingOrCorruptIendSucceeds_sampled` — `tests/SkPngRustDecoderTest.cpp:2044`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedPreblendBufferBudget_sampled` — `tests/SkPngRustDecoderTest.cpp:2054`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedPreblendBufferBudget_subset` — `tests/SkPngRustDecoderTest.cpp:2073`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedRowBufferBudget_readRow` — `tests/SkPngRustDecoderTest.cpp:2104`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedRowBufferBudget_xform` — `tests/SkPngRustDecoderTest.cpp:2129`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedXformZeroInitialized` — `tests/SkPngRustDecoderTest.cpp:2159`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedRowBufferBudget_colorXform` — `tests/SkPngRustDecoderTest.cpp:2181`; CPU; `DEF_TEST`.
- `RustPngCodec_rowsDecodedReportedOnErrorInInput_readRow` — `tests/SkPngRustDecoderTest.cpp:2296`; CPU; `DEF_TEST`.
- `RustPngCodec_rowsDecodedReportedOnErrorInInput_xform` — `tests/SkPngRustDecoderTest.cpp:2301`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedHalting` — `tests/SkPngRustDecoderTest.cpp:2461`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedSubsetHalting` — `tests/SkPngRustDecoderTest.cpp:2477`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedSamplingHalting` — `tests/SkPngRustDecoderTest.cpp:2521`; CPU; `DEF_TEST`.
- `RustPngCodec_interlacedSamplingSubsetHalting` — `tests/SkPngRustDecoderTest.cpp:2527`; CPU; `DEF_TEST`.
- `SkXmp_invalidXml` — `tests/SkXmpTest.cpp:21`; CPU; `DEF_TEST`.
- `SkXmp_xmpHdrgmAsFieldValue` — `tests/SkXmpTest.cpp:34`; CPU; `DEF_TEST`.
- `SkXmp_xmpHdrgmRequiresVersion` — `tests/SkXmpTest.cpp:59`; CPU; `DEF_TEST`.
- `SkXmp_xmpHdrgmAsDescriptionPropertyAttributes` — `tests/SkXmpTest.cpp:81`; CPU; `DEF_TEST`.
- `SkXmp_xmpHdrgmList` — `tests/SkXmpTest.cpp:106`; CPU; `DEF_TEST`.
- `SkXmp_xmpContainerTypedNode` — `tests/SkXmpTest.cpp:169`; CPU; `DEF_TEST`.
- `SkXmp_xmpContainerTypedNodeRdfEquivalent` — `tests/SkXmpTest.cpp:207`; CPU; `DEF_TEST`.
- `SkYUVAInfoPackedYUYV` — `tests/SkYUVAInfoTest.cpp:21`; CPU; `DEF_TEST`.
- `SkYUVAInfoSinglePlaneConfigs` — `tests/SkYUVAInfoTest.cpp:85`; CPU; `DEF_TEST`.
- `SwizzlerFill` — `tests/SwizzlerTest.cpp:73`; CPU; `DEF_TEST`.
- `SwizzleOpts` — `tests/SwizzlerTest.cpp:122`; CPU; `DEF_TEST`.
- `PublicSwizzleOpts` — `tests/SwizzlerTest.cpp:154`; CPU; `DEF_TEST`.
- `ReciprocalAlphaOptimized` — `tests/SwizzlerTest.cpp:187`; CPU; `DEF_TEST`.
- `ReciprocalAlphaPortable` — `tests/SwizzlerTest.cpp:193`; CPU; `DEF_TEST`.
- `UnpremulSimulatingRP` — `tests/SwizzlerTest.cpp:234`; CPU; `DEF_TEST`.
- `WebpCodecBlend` — `tests/WebpTest.cpp:18`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/AnimatedImageTest.cpp` — M22.
- `tests/AnnotationTest.cpp` — M27.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/CodecAnimTest.cpp` — M29.
- `tests/CodecTest.cpp` — M17.
- `tests/EncodeTest.cpp` — M17.
- `tests/EncodedInfoTest.cpp` — M17.
- `tests/ExifTest.cpp` — M17.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageGeneratorTest.cpp` — M22.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/JpegGainmapTest.cpp` — M17.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/PDFMetadataAttributeTest.cpp` — M27.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PngGainmapTest.cpp` — M17.
- `tests/RustIccTest.cpp` — M17.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/SkJpegRustCodecTest.cpp` — M17.
- `tests/SkPngRustEncoderTest.cpp` — M17.
- `tests/SkResourceCacheTest.cpp` — M22.
- `tests/YUVTest.cpp` — M17.
- `tests/graphite/ImageProviderTest.cpp` — M38.

### Preserve these fixtures, metadata and references

- `rust/README.md` — preserve.
- `rust/bmp/README.md` — preserve.
- `rust/icc/README.md` — preserve.
- `rust/png/README.md` — preserve.

## Dependency and integration notes

The initial scan found 40 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=16`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M16 ACCEPTED may the user hand Deepseek M17.
