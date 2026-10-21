# Milestone 17: Image encoding and round trips

## Outcome and entry gate

Implement PNG/JPEG/WebP and other encoder APIs present in the manifest, including pixmap paths, options, metadata and gainmap encoding. Keep external compression engines explicitly versioned and separate from Skia-owned adapters.

Begin implementation only after milestone 16 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement PNG/JPEG/WebP and other encoder APIs present in the manifest, including pixmap paths, options, metadata and gainmap encoding. Keep external compression engines explicitly versioned and separate from Skia-owned adapters.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/encode/SkEncoder.h` — port to Rust.
- `include/encode/SkICC.h` — port to Rust.
- `include/encode/SkJpegEncoder.h` — port to Rust.
- `include/encode/SkPngEncoder.h` — port to Rust.
- `include/encode/SkPngRustEncoder.h` — port to Rust.
- `include/encode/SkWebpEncoder.h` — port to Rust.
- `include/private/SkEncodedInfo.h` — port to Rust.
- `include/private/SkJpegGainmapEncoder.h` — port to Rust.
- `src/encode/SkEncoder.cpp` — port to Rust.
- `src/encode/SkICC.cpp` — port to Rust.
- `src/encode/SkICCPriv.h` — port to Rust.
- `src/encode/SkImageEncoderFns.h` — port to Rust.
- `src/encode/SkImageEncoderPriv.h` — port to Rust.
- `src/encode/SkJPEGWriteUtility.cpp` — port to Rust.
- `src/encode/SkJPEGWriteUtility.h` — port to Rust.
- `src/encode/SkJpegEncoderImpl.cpp` — port to Rust.
- `src/encode/SkJpegEncoderImpl.h` — port to Rust.
- `src/encode/SkJpegEncoder_none.cpp` — port to Rust.
- `src/encode/SkJpegGainmapEncoder.cpp` — port to Rust.
- `src/encode/SkPngEncoderBase.cpp` — port to Rust.
- `src/encode/SkPngEncoderBase.h` — port to Rust.
- `src/encode/SkPngEncoderImpl.cpp` — port to Rust.
- `src/encode/SkPngEncoderImpl.h` — port to Rust.
- `src/encode/SkPngEncoder_none.cpp` — port to Rust.
- `src/encode/SkPngRustEncoder.cpp` — port to Rust.
- `src/encode/SkPngRustEncoderImpl.cpp` — port to Rust.
- `src/encode/SkPngRustEncoderImpl.h` — port to Rust.
- `src/encode/SkWebpEncoderImpl.cpp` — port to Rust.
- `src/encode/SkWebpEncoder_none.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/encode/BUILD.bazel`.
- `src/encode/BUILD.bazel`.

## Codex: acceptance-test contract

Translate encoder tests and original decode-after-encode checks. Preserve expected metadata, alpha handling, color profiles and failure cases. Use byte equality only where the original tests demand it; do not invent lossy comparisons for exact assertions.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/CanvasStateTest.cpp`.
- `tests/CodecPriv.h`.
- `tests/CodecRecommendedTypeTest.cpp`.
- `tests/CodecTest.cpp`.
- `tests/EncodeTest.cpp`.
- `tests/EncodedInfoTest.cpp`.
- `tests/ExifTest.cpp`.
- `tests/ICCTest.cpp`.
- `tests/JpegGainmapTest.cpp`.
- `tests/PngGainmapTest.cpp`.
- `tests/RustIccTest.cpp`.
- `tests/SkJpegRustCodecTest.cpp`.
- `tests/SkPngRustEncoderTest.cpp`.
- `tests/YUVTest.cpp`.

### Original test registrations assigned to this gate

- `CanvasState_test_complex_layers` — `tests/CanvasStateTest.cpp:125`; CPU; `DEF_TEST`.
- `CanvasState_test_complex_clips` — `tests/CanvasStateTest.cpp:204`; CPU; `DEF_TEST`.
- `CanvasState_test_soft_clips` — `tests/CanvasStateTest.cpp:302`; CPU; `DEF_TEST`.
- `CanvasState_test_saveLayer_clip` — `tests/CanvasStateTest.cpp:316`; CPU; `DEF_TEST`.
- `Codec_recommendedF16` — `tests/CodecRecommendedTypeTest.cpp:23`; CPU; `DEF_TEST`.
- `Codec_wbmp` — `tests/CodecTest.cpp:596`; CPU; `DEF_TEST`.
- `Codec_webp` — `tests/CodecTest.cpp:600`; CPU; `DEF_TEST`.
- `Codec_bmp` — `tests/CodecTest.cpp:606`; CPU; `DEF_TEST`.
- `Codec_ico` — `tests/CodecTest.cpp:612`; CPU; `DEF_TEST`.
- `Codec_gif` — `tests/CodecTest.cpp:622`; CPU; `DEF_TEST`.
- `Codec_jpg` — `tests/CodecTest.cpp:629`; CPU; `DEF_TEST`.
- `Codec_png` — `tests/CodecTest.cpp:639`; CPU; `DEF_TEST`.
- `Codec_png_plte_trns` — `tests/CodecTest.cpp:707`; CPU; `DEF_TEST`.
- `Codec_png_plte_trns_gama` — `tests/CodecTest.cpp:720`; CPU; `DEF_TEST`.
- `Codec_raw` — `tests/CodecTest.cpp:737`; CPU; `DEF_TEST`.
- `Codec_leaks` — `tests/CodecTest.cpp:754`; CPU; `DEF_TEST`.
- `Codec_null` — `tests/CodecTest.cpp:775`; CPU; `DEF_TEST`.
- `Codec_Dimensions` — `tests/CodecTest.cpp:819`; CPU; `DEF_TEST`.
- `Codec_Empty` — `tests/CodecTest.cpp:860`; CPU; `DEF_TEST`.
- `Codec_pngChunkReader` — `tests/CodecTest.cpp:901`; CPU; `DEF_TEST`.
- `Codec_raw_notseekable` — `tests/CodecTest.cpp:1079`; CPU; `DEF_TEST`.
- `Codec_webp_peek` — `tests/CodecTest.cpp:1097`; CPU; `DEF_TEST`.
- `Codec_wbmp_restrictive` — `tests/CodecTest.cpp:1123`; CPU; `DEF_TEST`.
- `Codec_wbmp_max_size` — `tests/CodecTest.cpp:1149`; CPU; `DEF_TEST`.
- `Codec_jpeg_rewind` — `tests/CodecTest.cpp:1173`; CPU; `DEF_TEST`.
- `Codec_ColorXform` — `tests/CodecTest.cpp:1246`; CPU; `DEF_TEST`.
- `Codec_pngRoundTrip` — `tests/CodecTest.cpp:1295`; CPU; `DEF_TEST`.
- `Codec_F16ConversionPossible` — `tests/CodecTest.cpp:1391`; CPU; `DEF_TEST`.
- `Codec_skipFullParse` — `tests/CodecTest.cpp:1410`; CPU; `DEF_TEST`.
- `Codec_fallBack` — `tests/CodecTest.cpp:1487`; CPU; `DEF_TEST`.
- `Wuffs_seek_and_decode` — `tests/CodecTest.cpp:1558`; CPU; `DEF_TEST`.
- `Codec_reusePng` — `tests/CodecTest.cpp:1573`; CPU; `DEF_TEST`.
- `Codec_rowsDecoded` — `tests/CodecTest.cpp:1601`; CPU; `DEF_TEST`.
- `Codec_InvalidImages` — `tests/CodecTest.cpp:1643`; CPU; `DEF_TEST`.
- `Codec_InvalidHeader` — `tests/CodecTest.cpp:1677`; CPU; `DEF_TEST`.
- `Codec_InvalidAnimated` — `tests/CodecTest.cpp:1754`; CPU; `DEF_TEST`.
- `Codec_EncodeICC` — `tests/CodecTest.cpp:1835`; CPU; `DEF_TEST`.
- `Codec_webp_rowsDecoded` — `tests/CodecTest.cpp:1841`; CPU; `DEF_TEST`.
- `Codec_ossfuzz6274` — `tests/CodecTest.cpp:1886`; CPU; `DEF_TEST`.
- `Codec_78329453` — `tests/CodecTest.cpp:1899`; CPU; `DEF_TEST`.
- `Codec_A8` — `tests/CodecTest.cpp:1935`; CPU; `DEF_TEST`.
- `Codec_crbug807324` — `tests/CodecTest.cpp:1954`; CPU; `DEF_TEST`.
- `Codec_F16_noColorSpace` — `tests/CodecTest.cpp:1989`; CPU; `DEF_TEST`.
- `Codec_noConversion` — `tests/CodecTest.cpp:2005`; CPU; `DEF_TEST`.
- `Codec_kBGR_101010x_XR_SkColorType_supported` — `tests/CodecTest.cpp:2046`; CPU; `DEF_TEST`.
- `Codec_gif_notseekable` — `tests/CodecTest.cpp:2064`; CPU; `DEF_TEST`.
- `Codec_gif_notseekable2` — `tests/CodecTest.cpp:2097`; CPU; `DEF_TEST`.
- `Codec_gif_null_param` — `tests/CodecTest.cpp:2140`; CPU; `DEF_TEST`.
- `Codec_gif_can_preserve_original_data` — `tests/CodecTest.cpp:2164`; CPU; `DEF_TEST`.
- `Codec_jpeg_can_return_data_from_original_stream` — `tests/CodecTest.cpp:2195`; CPU; `DEF_TEST`.
- `Codec_jpeg_decode_progressive_truncated_stream` — `tests/CodecTest.cpp:2228`; CPU; `DEF_TEST`.
- `Codec_jpeg_decode_progressive_stream_incomplete` — `tests/CodecTest.cpp:2245`; CPU; `DEF_TEST`.
- `Codec_bmp_indexed_colorxform` — `tests/CodecTest.cpp:2263`; CPU; `DEF_TEST`.
- `Codec_GenerateColorTransformFiles` — `tests/CodecTest.cpp:2399`; CPU; `DEF_TEST`.
- `Codec_RoundTripColorXform` — `tests/CodecTest.cpp:2445`; CPU; `DEF_TEST`.
- `Codec_webp_animated_image_rewind` — `tests/CodecTest.cpp:2522`; CPU; `DEF_TEST`.
- `LibpngCodec_f16_trc_tables` — `tests/CodecTest.cpp:2548`; CPU; `DEF_TEST`.
- `PngRustHdrMetadataRoundTrip` — `tests/CodecTest.cpp:2589`; CPU; `DEF_TEST`.
- `jpeg_invalid_stream_state` — `tests/CodecTest.cpp:2613`; CPU; `DEF_TEST`.
- `Codec_Bmp_b511820841` — `tests/CodecTest.cpp:2645`; CPU; `DEF_TEST`.
- `Ico_usesRegisteredPngDecoder` — `tests/CodecTest.cpp:2690`; CPU; `DEF_SERIAL_TEST`.
- `Ico_fallbackWhenPngNotRegistered` — `tests/CodecTest.cpp:2729`; CPU; `DEF_SERIAL_TEST`.
- `Codec_budgeted_buffer` — `tests/CodecTest.cpp:2746`; CPU; `DEF_TEST`.
- `Encode` — `tests/EncodeTest.cpp:127`; CPU; `DEF_TEST`.
- `Encode_png_roundtrip_for_different_source_formats` — `tests/EncodeTest.cpp:301`; CPU; `DEF_TEST`.
- `Encode_JPG` — `tests/EncodeTest.cpp:324`; CPU; `DEF_TEST`.
- `Encode_JpegDownsample` — `tests/EncodeTest.cpp:361`; CPU; `DEF_TEST`.
- `Encode_PngOptions` — `tests/EncodeTest.cpp:466`; CPU; `DEF_TEST`.
- `Encode_WebpQuality` — `tests/EncodeTest.cpp:509`; CPU; `DEF_TEST`.
- `Encode_WebpOptions` — `tests/EncodeTest.cpp:562`; CPU; `DEF_TEST`.
- `Encode_WebpMethod` — `tests/EncodeTest.cpp:610`; CPU; `DEF_TEST`.
- `Encode_WebpMethod_Alpha` — `tests/EncodeTest.cpp:658`; CPU; `DEF_TEST`.
- `Encode_WebpAnimated` — `tests/EncodeTest.cpp:712`; CPU; `DEF_TEST`.
- `Encode_WebpAnimated_FrameUnmatched` — `tests/EncodeTest.cpp:760`; CPU; `DEF_TEST`.
- `Encode_Alpha` — `tests/EncodeTest.cpp:785`; CPU; `DEF_TEST`.
- `Encode_jpeg_blend_to_black` — `tests/EncodeTest.cpp:818`; CPU; `DEF_TEST`.
- `Encode_HdrMetadata` — `tests/EncodeTest.cpp:958`; CPU; `DEF_TEST`.
- `AlphaEncodedInfo` — `tests/EncodedInfoTest.cpp:22`; CPU; `DEF_TEST`.
- `ExifOrientation` — `tests/ExifTest.cpp:24`; CPU; `DEF_TEST`.
- `GetImageRespectsExif` — `tests/ExifTest.cpp:42`; CPU; `DEF_TEST`.
- `ExifOrientationInExif` — `tests/ExifTest.cpp:65`; CPU; `DEF_TEST`.
- `ExifOrientationInSubIFD` — `tests/ExifTest.cpp:74`; CPU; `DEF_TEST`.
- `ExifParse` — `tests/ExifTest.cpp:85`; CPU; `DEF_TEST`.
- `ExifTruncate` — `tests/ExifTest.cpp:193`; CPU; `DEF_TEST`.
- `ExifWriteOrientation` — `tests/ExifTest.cpp:231`; CPU; `DEF_TEST`.
- `ExifWriteTest` — `tests/ExifTest.cpp:265`; CPU; `DEF_TEST`.
- `ExifWriteFailsHDRheadroom` — `tests/ExifTest.cpp:287`; CPU; `DEF_TEST`.
- `AdobeRGB` — `tests/ICCTest.cpp:22`; CPU; `DEF_TEST`.
- `HDR_ICC` — `tests/ICCTest.cpp:34`; CPU; `DEF_TEST`.
- `Codec_jpegSegmentScan` — `tests/JpegGainmapTest.cpp:116`; CPU; `DEF_TEST`.
- `Codec_multiPictureParams` — `tests/JpegGainmapTest.cpp:198`; CPU; `DEF_TEST`.
- `Codec_jpegMultiPicture` — `tests/JpegGainmapTest.cpp:336`; CPU; `DEF_TEST`.
- `AndroidCodec_jpegGainmapDecode` — `tests/JpegGainmapTest.cpp:463`; CPU; `DEF_TEST`.
- `AndroidCodec_jpegNoGainmap` — `tests/JpegGainmapTest.cpp:574`; CPU; `DEF_TEST`.
- `AndroidCodec_gainmapInfoEncode` — `tests/JpegGainmapTest.cpp:615`; CPU; `DEF_TEST`.
- `AndroidCodec_jpegGainmapTranscode` — `tests/JpegGainmapTest.cpp:765`; CPU; `DEF_TEST`.
- `AndroidCodec_mpfParse` — `tests/JpegGainmapTest.cpp:893`; CPU; `DEF_TEST`.
- `AndroidCodec_gainmapInfoParse` — `tests/JpegGainmapTest.cpp:991`; CPU; `DEF_TEST`.
- `AndroidCodec_gainmapInfoParseInvalidRational` — `tests/JpegGainmapTest.cpp:1072`; CPU; `DEF_TEST`.
- `AndroidCodec_pngGainmapDecode` — `tests/PngGainmapTest.cpp:71`; CPU; `DEF_TEST`.
- `AndroidCodec_pngGainmapInvalidDecode` — `tests/PngGainmapTest.cpp:119`; CPU; `DEF_TEST`.
- `AndroidCodec_pngGainmapEncodeAndDecode` — `tests/PngGainmapTest.cpp:140`; CPU; `DEF_TEST`.
- `RustIcc_matrix_conversion` — `tests/RustIccTest.cpp:88`; CPU; `DEF_TEST`.
- `RustIcc_transfer_function_conversion` — `tests/RustIccTest.cpp:109`; CPU; `DEF_TEST`.
- `RustIcc_profile_conversion` — `tests/RustIccTest.cpp:128`; CPU; `DEF_TEST`.
- `RustIcc_profile_conversion_fails_without_data` — `tests/RustIccTest.cpp:191`; CPU; `DEF_TEST`.
- `RustIcc_profile_conversion_fails_trc_without_matrix` — `tests/RustIccTest.cpp:205`; CPU; `DEF_TEST`.
- `RustIcc_profile_conversion_fails_matrix_without_trc` — `tests/RustIccTest.cpp:226`; CPU; `DEF_TEST`.
- `RustIcc_cicp_conversion` — `tests/RustIccTest.cpp:244`; CPU; `DEF_TEST`.
- `RustIcc_a2b_b2a_flags` — `tests/RustIccTest.cpp:283`; CPU; `DEF_TEST`.
- `RustIcc_profile_with_a2b_curves` — `tests/RustIccTest.cpp:317`; CPU; `DEF_TEST`.
- `RustIcc_profile_with_a2b_matrix` — `tests/RustIccTest.cpp:411`; CPU; `DEF_TEST`.
- `RustIcc_profile_with_table_curves` — `tests/RustIccTest.cpp:498`; CPU; `DEF_TEST`.
- `RustIcc_reject_malformed_a2b` — `tests/RustIccTest.cpp:605`; CPU; `DEF_TEST`.
- `RustIcc_profile_with_b2a` — `tests/RustIccTest.cpp:668`; CPU; `DEF_TEST`.
- `RustIcc_clut_transform_exercises_gather_overread` — `tests/RustIccTest.cpp:765`; CPU; `DEF_TEST`.
- `RustIcc_equivalence_with_skcms_resource_files` — `tests/RustIccTest.cpp:946`; CPU; `DEF_TEST`.
- `RustIcc_trc_table_passthrough` — `tests/RustIccTest.cpp:1318`; CPU; `DEF_TEST`.
- `RustIcc_pq_hdr_b2a_present` — `tests/RustIccTest.cpp:1363`; CPU; `DEF_TEST`.
- `RustIcc_writer_hagc_matches_parsed` — `tests/RustIccTest.cpp:1398`; CPU; `DEF_TEST`.
- `RustJpegCodec_IsJpeg` — `tests/SkJpegRustCodecTest.cpp:120`; CPU; `DEF_TEST`.
- `RustJpegCodec_decode_valid_jpgs` — `tests/SkJpegRustCodecTest.cpp:138`; CPU; `DEF_TEST`.
- `RustJpeg_IncrementalDecode_PartialStreaming` — `tests/SkJpegRustCodecTest.cpp:187`; CPU; `DEF_TEST`.
- `RustJpeg_IncrementalDecode_RejectsSubset` — `tests/SkJpegRustCodecTest.cpp:450`; CPU; `DEF_TEST`.
- `RustJpeg_IncrementalDecode_RejectsShortRowBytes` — `tests/SkJpegRustCodecTest.cpp:471`; CPU; `DEF_TEST`.
- `RustJpeg_IncrementalDecode_AfterRewind` — `tests/SkJpegRustCodecTest.cpp:488`; CPU; `DEF_TEST`.
- `RustJpeg_IncrementalDecode_ColorTransformPaddedStride` — `tests/SkJpegRustCodecTest.cpp:538`; CPU; `DEF_TEST`.
- `RustJpegCodec_rewind` — `tests/SkJpegRustCodecTest.cpp:629`; CPU; `DEF_TEST`.
- `RustJpegCodec_incomplete_input` — `tests/SkJpegRustCodecTest.cpp:646`; CPU; `DEF_TEST`.
- `RustJpegCodec_short_input_classification` — `tests/SkJpegRustCodecTest.cpp:671`; CPU; `DEF_TEST`.
- `RustJpegCodec_reject_non_jpeg` — `tests/SkJpegRustCodecTest.cpp:707`; CPU; `DEF_TEST`.
- `RustJpegCodec_decode_with_color_transform` — `tests/SkJpegRustCodecTest.cpp:722`; CPU; `DEF_TEST`.
- `RustJpegCodec_RawYUVAParity` — `tests/SkJpegRustCodecTest.cpp:750`; CPU; `DEF_TEST`.
- `RustJpegCodec_ProgressiveRawYUVRetry` — `tests/SkJpegRustCodecTest.cpp:912`; CPU; `DEF_TEST`.
- `RustJpegCodec_RawDirectPlaneContracts` — `tests/SkJpegRustCodecTest.cpp:964`; CPU; `DEF_TEST`.
- `RustJpegCodec_ScanlineWorkflows` — `tests/SkJpegRustCodecTest.cpp:1110`; CPU; `DEF_TEST`.
- `RustJpegCodec_GetPixelsUsesScanlines` — `tests/SkJpegRustCodecTest.cpp:1249`; CPU; `DEF_TEST`.
- `RustJpegCodec_GetPixelsBaselineGrayscale` — `tests/SkJpegRustCodecTest.cpp:1353`; CPU; `DEF_TEST`.
- `RustJpegCodec_GetPixelsProgressiveFallback` — `tests/SkJpegRustCodecTest.cpp:1416`; CPU; `DEF_TEST`.
- `RustJpegCodec_GetPixelsExtendedSequential` — `tests/SkJpegRustCodecTest.cpp:1442`; CPU; `DEF_TEST`.
- `RustJpegCodec_GetPixelsTruncatedRows` — `tests/SkJpegRustCodecTest.cpp:1472`; CPU; `DEF_TEST`.
- `RustJpegCodec_GetPixelsMissingEoiAfterAllRows` — `tests/SkJpegRustCodecTest.cpp:1524`; CPU; `DEF_TEST`.
- `RustJpegCodec_StreamingOutputRetry` — `tests/SkJpegRustCodecTest.cpp:1566`; CPU; `DEF_TEST`.
- `RustJpegEncode_smoke_test` — `tests/SkJpegRustCodecTest.cpp:1687`; CPU; `DEF_TEST`.
- `RustJpegEncode_quality_affects_size` — `tests/SkJpegRustCodecTest.cpp:1748`; CPU; `DEF_TEST`.
- `RustJpegEncode_color_formats` — `tests/SkJpegRustCodecTest.cpp:1776`; CPU; `DEF_TEST`.
- `RustJpeg_gainmap_extraction` — `tests/SkJpegRustCodecTest.cpp:1842`; CPU; `DEF_TEST`.
- `RustJpeg_vs_CppJpeg` — `tests/SkJpegRustCodecTest.cpp:1979`; CPU; `DEF_TEST`.
- `RustJpeg_vs_CppJpeg_Orientation` — `tests/SkJpegRustCodecTest.cpp:2017`; CPU; `DEF_TEST`.
- `RustJpeg_vs_CppJpeg_ICCProfile` — `tests/SkJpegRustCodecTest.cpp:2061`; CPU; `DEF_TEST`.
- `RustEncodePng_smoke_test` — `tests/SkPngRustEncoderTest.cpp:23`; CPU; `DEF_TEST`.
- `RustEncodePng_gainmap` — `tests/SkPngRustEncoderTest.cpp:57`; CPU; `DEF_TEST`.
- `RustEncodePng_sBIT` — `tests/SkPngRustEncoderTest.cpp:146`; CPU; `DEF_TEST`.
- `Jpeg_YUV_Codec` — `tests/YUVTest.cpp:99`; CPU; `DEF_TEST`.
- `Jpeg_YUV_Encode` — `tests/YUVTest.cpp:182`; CPU; `DEF_TEST`.
- `YUVMath` — `tests/YUVTest.cpp:213`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `tests/BigImageTest.cpp` — M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/TextBlobCacheTest.cpp` — M32, M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 9 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=17`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M17 ACCEPTED may the user hand Deepseek M18.
