# Milestone 22: Pictures, records, serialization and capture

## Outcome and entry gate

Implement picture recording/playback, records/optimization, flattenables, serialization callbacks, picture images, capture, documents and annotation plumbing. Preserve supported format behavior, validation and resource ownership. Reconcile font and shader serialization hooks from earlier milestones.

Begin implementation only after milestone 21 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement picture recording/playback, records/optimization, flattenables, serialization callbacks, picture images, capture, documents and annotation plumbing. Preserve supported format behavior, validation and resource ownership. Reconcile font and shader serialization hooks from earlier milestones.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkAnnotation.h` — port to Rust.
- `include/core/SkCPURecorder.h` — port to Rust.
- `include/core/SkFlattenable.h` — port to Rust.
- `include/core/SkPicture.h` — port to Rust.
- `include/core/SkPictureRecorder.h` — port to Rust.
- `include/core/SkRecorder.h` — port to Rust.
- `include/core/SkSerialProcs.h` — port to Rust.
- `include/cpu/Recorder.h` — port to Rust.
- `include/docs/SkMultiPictureDocument.h` — port to Rust.
- `include/docs/SkPDFDocument.h` — port to Rust.
- `include/docs/SkXPSDocument.h` — port to Rust.
- `include/private/SkThreadAnnotations.h` — port to Rust.
- `include/private/chromium/GrDeferredDisplayListRecorder.h` — port to Rust.
- `src/capture/SkCapture.cpp` — port to Rust.
- `src/capture/SkCapture.h` — port to Rust.
- `src/capture/SkCaptureAsset.h` — port to Rust.
- `src/capture/SkCaptureCanvas.cpp` — port to Rust.
- `src/capture/SkCaptureCanvas.h` — port to Rust.
- `src/capture/SkCaptureManager.cpp` — port to Rust.
- `src/capture/SkCaptureManager.h` — port to Rust.
- `src/core/Recorder.cpp` — port to Rust.
- `src/core/SkAnnotation.cpp` — port to Rust.
- `src/core/SkAnnotationKeys.h` — port to Rust.
- `src/core/SkFlattenable.cpp` — port to Rust.
- `src/core/SkFont_serial.cpp` — port to Rust.
- `src/core/SkPicture.cpp` — port to Rust.
- `src/core/SkPictureData.cpp` — port to Rust.
- `src/core/SkPictureData.h` — port to Rust.
- `src/core/SkPictureFlat.cpp` — port to Rust.
- `src/core/SkPictureFlat.h` — port to Rust.
- `src/core/SkPicturePlayback.cpp` — port to Rust.
- `src/core/SkPicturePlayback.h` — port to Rust.
- `src/core/SkPicturePriv.h` — port to Rust.
- `src/core/SkPictureRecord.cpp` — port to Rust.
- `src/core/SkPictureRecord.h` — port to Rust.
- `src/core/SkPictureRecorder.cpp` — port to Rust.
- `src/core/SkPtrRecorder.cpp` — port to Rust.
- `src/core/SkPtrRecorder.h` — port to Rust.
- `src/core/SkReadBuffer.cpp` — port to Rust.
- `src/core/SkReadBuffer.h` — port to Rust.
- `src/core/SkRecord.cpp` — port to Rust.
- `src/core/SkRecord.h` — port to Rust.
- `src/core/SkRecordCanvas.cpp` — port to Rust.
- `src/core/SkRecordCanvas.h` — port to Rust.
- `src/core/SkRecordDraw.cpp` — port to Rust.
- `src/core/SkRecordDraw.h` — port to Rust.
- `src/core/SkRecordOpts.cpp` — port to Rust.
- `src/core/SkRecordOpts.h` — port to Rust.
- `src/core/SkRecordPattern.h` — port to Rust.
- `src/core/SkRecordedDrawable.cpp` — port to Rust.
- `src/core/SkRecordedDrawable.h` — port to Rust.
- `src/core/SkRecorder.cpp` — port to Rust.
- `src/core/SkRecords.cpp` — port to Rust.
- `src/core/SkRecords.h` — port to Rust.
- `src/core/SkWriteBuffer.cpp` — port to Rust.
- `src/core/SkWriteBuffer.h` — port to Rust.
- `src/image/SkImage_Picture.cpp` — port to Rust.
- `src/image/SkImage_Picture.h` — port to Rust.
- `src/image/SkPictureImageGenerator.cpp` — port to Rust.
- `src/image/SkPictureImageGenerator.h` — port to Rust.
- `src/shaders/SkPictureShader.cpp` — port to Rust.
- `src/shaders/SkPictureShader.h` — port to Rust.
- `src/utils/SkMultiPictureDocument.cpp` — port to Rust.
- `src/utils/SkMultiPictureDocumentPriv.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `src/capture/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate picture, record, buffer, serialization, font-flatten, drawable and annotation tests. Preserve malformed-stream rejection and callback semantics. Run C++-produced fixtures through Rust where the format promises compatibility and rerun all earlier serialization-dependent registrations.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/AnimatedImageTest.cpp`.
- `tests/CPUContextRecorderTest.cpp`.
- `tests/DescriptorTest.cpp`.
- `tests/FlattenDrawableTest.cpp`.
- `tests/FlattenableFactoryToName.cpp`.
- `tests/FlattenableNameToFactory.cpp`.
- `tests/FontScanner.cpp`.
- `tests/FontScanner_FontationsTest.cpp`.
- `tests/FontScanner_FreeTypeTest.cpp`.
- `tests/FontTest.cpp`.
- `tests/ImageGeneratorTest.cpp`.
- `tests/PaintTest.cpp`.
- `tests/PathTest.cpp`.
- `tests/PictureBBHTest.cpp`.
- `tests/PictureShaderTest.cpp`.
- `tests/PictureTest.cpp`.
- `tests/RecordDrawTest.cpp`.
- `tests/RecordOptsTest.cpp`.
- `tests/RecordPatternTest.cpp`.
- `tests/RecordTest.cpp`.
- `tests/RecordTestUtils.h`.
- `tests/RecorderTest.cpp`.
- `tests/RecordingXfermodeTest.cpp`.
- `tests/SerialProcsTest.cpp`.
- `tests/SerializationTest.cpp`.
- `tests/SkFontMetricsPrivTest.cpp`.
- `tests/SkResourceCacheTest.cpp`.
- `tests/SkStrikeCacheTest.cpp`.
- `tests/SkStrikeTest.cpp`.
- `tests/TextBlobTest.cpp`.
- `tests/VerticesTest.cpp`.
- `tests/Writer32Test.cpp`.

### Original test registrations assigned to this gate

- `AnimatedImage_simple` — `tests/AnimatedImageTest.cpp:35`; CPU; `DEF_TEST`.
- `AnimatedImage_rotation` — `tests/AnimatedImageTest.cpp:54`; CPU; `DEF_TEST`.
- `AnimatedImage_invalidCrop` — `tests/AnimatedImageTest.cpp:89`; CPU; `DEF_TEST`.
- `AnimatedImage_scaled` — `tests/AnimatedImageTest.cpp:149`; CPU; `DEF_TEST`.
- `AnimatedImage_copyOnWrite` — `tests/AnimatedImageTest.cpp:218`; CPU; `DEF_TEST`.
- `AnimatedImage` — `tests/AnimatedImageTest.cpp:289`; CPU; `DEF_TEST`.
- `SkContext_CaptureDisabledByDefault` — `tests/CPUContextRecorderTest.cpp:28`; CPU; `DEF_TEST`.
- `CPUSurface_UsesCPUContextAndRecorderToDraw_DrawsPixels` — `tests/CPUContextRecorderTest.cpp:40`; CPU; `DEF_TEST`.
- `CPUSurface_UsesTODORecorder_DrawsPixels` — `tests/CPUContextRecorderTest.cpp:57`; CPU; `DEF_TEST`.
- `ImageMakeColorSpace_UsesCPURecorderToMakeImage_Success` — `tests/CPUContextRecorderTest.cpp:72`; CPU; `DEF_TEST`.
- `ImageMakeScaled_UsesCPURecorderToMakeImage_Success` — `tests/CPUContextRecorderTest.cpp:87`; CPU; `DEF_TEST`.
- `Descriptor_empty` — `tests/DescriptorTest.cpp:29`; CPU; `DEF_TEST`.
- `Descriptor_valid_simple` — `tests/DescriptorTest.cpp:37`; CPU; `DEF_TEST`.
- `Descriptor_valid_simple_extra_space` — `tests/DescriptorTest.cpp:51`; CPU; `DEF_TEST`.
- `Descriptor_valid_more_tags` — `tests/DescriptorTest.cpp:66`; CPU; `DEF_TEST`.
- `Descriptor_invalid_rec_size` — `tests/DescriptorTest.cpp:84`; CPU; `DEF_TEST`.
- `Descriptor_invalid_length` — `tests/DescriptorTest.cpp:95`; CPU; `DEF_TEST`.
- `Descriptor_entry_too_big` — `tests/DescriptorTest.cpp:109`; CPU; `DEF_TEST`.
- `Descriptor_entry_over_end` — `tests/DescriptorTest.cpp:127`; CPU; `DEF_TEST`.
- `Descriptor_flatten_unflatten` — `tests/DescriptorTest.cpp:140`; CPU; `DEF_TEST`.
- `FlattenDrawable` — `tests/FlattenDrawableTest.cpp:222`; CPU; `DEF_TEST`.
- `FlattenRecordedDrawable` — `tests/FlattenDrawableTest.cpp:263`; CPU; `DEF_TEST`.
- `Flattenable_EmptyDeserialze` — `tests/FlattenDrawableTest.cpp:302`; CPU; `DEF_TEST`.
- `FlattenableFactoryToName` — `tests/FlattenableFactoryToName.cpp:33`; CPU; `DEF_TEST`.
- `FlattenableNameToFactory` — `tests/FlattenableNameToFactory.cpp:11`; CPU; `DEF_TEST`.
- `FontScanner_Fontations_VariableFont` — `tests/FontScanner_FontationsTest.cpp:13`; CPU; `DEF_TEST`.
- `FontScanner_Fontations_NamedInstances1` — `tests/FontScanner_FontationsTest.cpp:17`; CPU; `DEF_TEST`.
- `FontScanner_Fontations_NamedInstances2` — `tests/FontScanner_FontationsTest.cpp:21`; CPU; `DEF_TEST`.
- `FontScanner_Fontations_FontCollection` — `tests/FontScanner_FontationsTest.cpp:25`; CPU; `DEF_TEST`.
- `FontScanner_FreeType_VariableFont` — `tests/FontScanner_FreeTypeTest.cpp:15`; CPU; `DEF_TEST`.
- `FontScanner_FreeType__NamedInstances1` — `tests/FontScanner_FreeTypeTest.cpp:19`; CPU; `DEF_TEST`.
- `FontScanner_FreeType__NamedInstances2` — `tests/FontScanner_FreeTypeTest.cpp:23`; CPU; `DEF_TEST`.
- `FontScanner_FreeType_FontCollection` — `tests/FontScanner_FreeTypeTest.cpp:27`; CPU; `DEF_TEST`.
- `Font_flatten` — `tests/FontTest.cpp:74`; CPU; `DEF_TEST`.
- `ImageGenerator` — `tests/ImageGeneratorTest.cpp:76`; CPU; `DEF_TEST`.
- `PictureImageGenerator` — `tests/ImageGeneratorTest.cpp:104`; CPU; `DEF_TEST`.
- `Paint_copy` — `tests/PaintTest.cpp:44`; CPU; `DEF_TEST`.
- `Paint_regression_cubic` — `tests/PaintTest.cpp:71`; CPU; `DEF_TEST`.
- `Paint_flattening` — `tests/PaintTest.cpp:103`; CPU; `DEF_TEST`.
- `Paint_regression_measureText` — `tests/PaintTest.cpp:149`; CPU; `DEF_TEST`.
- `Paint_MoreFlattening` — `tests/PaintTest.cpp:164`; CPU; `DEF_TEST`.
- `Paint_nothingToDraw` — `tests/PaintTest.cpp:184`; CPU; `DEF_TEST`.
- `Font_getpos` — `tests/PaintTest.cpp:208`; CPU; `DEF_TEST`.
- `Paint_dither` — `tests/PaintTest.cpp:246`; CPU; `DEF_TEST`.
- `PathInterp` — `tests/PathTest.cpp:4756`; CPU; `DEF_TEST`.
- `PathBigCubic` — `tests/PathTest.cpp:4760`; CPU; `DEF_TEST`.
- `PathContains` — `tests/PathTest.cpp:4776`; CPU; `DEF_TEST`.
- `Paths` — `tests/PathTest.cpp:4780`; CPU; `DEF_TEST`.
- `conservatively_contains_rect` — `tests/PathTest.cpp:4875`; CPU; `DEF_TEST`.
- `skbug_6450` — `tests/PathTest.cpp:4891`; CPU; `DEF_TEST`.
- `NonFinitePathIteration` — `tests/PathTest.cpp:4914`; CPU; `DEF_TEST`.
- `AndroidArc` — `tests/PathTest.cpp:4922`; CPU; `DEF_TEST`.
- `HugeGeometry` — `tests/PathTest.cpp:4947`; CPU; `DEF_TEST`.
- `ClipPath_nonfinite` — `tests/PathTest.cpp:4979`; CPU; `DEF_TEST`.
- `Path_isRect` — `tests/PathTest.cpp:5008`; CPU; `DEF_TEST`.
- `triangle_onehalf` — `tests/PathTest.cpp:5174`; CPU; `DEF_TEST`.
- `triangle_big` — `tests/PathTest.cpp:5185`; CPU; `DEF_TEST`.
- `Path_increserve_handle_neg_crbug_883666` — `tests/PathTest.cpp:5201`; CPU; `DEF_TEST`.
- `Path_survive_transform` — `tests/PathTest.cpp:5279`; CPU; `DEF_TEST`.
- `pathedger` — `tests/PathTest.cpp:5387`; CPU; `DEF_TEST`.
- `path_convexity_scale_way_down` — `tests/PathTest.cpp:5406`; CPU; `DEF_TEST`.
- `path_moveto_addrect` — `tests/PathTest.cpp:5419`; CPU; `DEF_TEST`.
- `path_moveto_twopass_convexity` — `tests/PathTest.cpp:5454`; CPU; `DEF_TEST`.
- `path_walk_simple_edges_1154864` — `tests/PathTest.cpp:5474`; CPU; `DEF_TEST`.
- `path_walk_edges_concave_large_dx` — `tests/PathTest.cpp:5493`; CPU; `DEF_TEST`.
- `path_filltype_utils` — `tests/PathTest.cpp:5512`; CPU; `DEF_TEST`.
- `path_computeTightBounds` — `tests/PathTest.cpp:5627`; CPU; `DEF_TEST`.
- `path_trivial_isrect` — `tests/PathTest.cpp:5655`; CPU; `DEF_TEST`.
- `path_infinite_transform` — `tests/PathTest.cpp:5700`; CPU; `DEF_TEST`.
- `path_factory_inverted_bounds` — `tests/PathTest.cpp:5745`; CPU; `DEF_TEST`.
- `path_trailing_moves_bounds` — `tests/PathTest.cpp:5755`; CPU; `DEF_TEST`.
- `path_read_from_memory_corrupt_rrect` — `tests/PathTest.cpp:5828`; CPU; `DEF_TEST`.
- `path_empty_iter` — `tests/PathTest.cpp:5838`; CPU; `DEF_TEST`.
- `path_b511244869` — `tests/PathTest.cpp:5845`; CPU; `DEF_TEST`.
- `Path_snapshot_rrect_rotation` — `tests/PathTest.cpp:5862`; CPU; `DEF_TEST`.
- `Path_snapshot_rrect_perspective` — `tests/PathTest.cpp:5874`; CPU; `DEF_TEST`.
- `Path_addRRect_with_move` — `tests/PathTest.cpp:5887`; CPU; `DEF_TEST`.
- `Path_snapshot_rrect_success` — `tests/PathTest.cpp:5898`; CPU; `DEF_TEST`.
- `Fuzz_b464232697_ExtremeStrokeBounds` — `tests/PathTest.cpp:5930`; CPU; `DEF_TEST`.
- `Fuzz_b42534575_ExtremeStrokeBounds` — `tests/PathTest.cpp:5951`; CPU; `DEF_TEST`.
- `PictureBBH` — `tests/PictureBBHTest.cpp:90`; CPU; `DEF_TEST`.
- `PictureNegativeSpace` — `tests/PictureBBHTest.cpp:99`; CPU; `DEF_TEST`.
- `PictureShader_caching` — `tests/PictureShaderTest.cpp:26`; CPU; `DEF_TEST`.
- `PictureShader_caching2` — `tests/PictureShaderTest.cpp:67`; CPU; `DEF_TEST`.
- `PictureRecorder_replay` — `tests/PictureTest.cpp:219`; CPU; `DEF_TEST`.
- `Picture` — `tests/PictureTest.cpp:517`; CPU; `DEF_TEST`.
- `Picture_EmptyBitmap` — `tests/PictureTest.cpp:549`; CPU; `DEF_TEST`.
- `Canvas_EmptyBitmap` — `tests/PictureTest.cpp:555`; CPU; `DEF_TEST`.
- `DontOptimizeSaveLayerDrawDrawRestore` — `tests/PictureTest.cpp:563`; CPU; `DEF_TEST`.
- `Picture_SkipBBH` — `tests/PictureTest.cpp:634`; CPU; `DEF_TEST`.
- `Picture_BitmapLeak` — `tests/PictureTest.cpp:656`; CPU; `DEF_TEST`.
- `Picture_getRecordingCanvas` — `tests/PictureTest.cpp:690`; CPU; `DEF_TEST`.
- `Picture_preserveCullRect` — `tests/PictureTest.cpp:701`; CPU; `DEF_TEST`.
- `Picture_UpdatedCull_1` — `tests/PictureTest.cpp:727`; CPU; `DEF_TEST`.
- `Picture_UpdatedCull_2` — `tests/PictureTest.cpp:741`; CPU; `DEF_TEST`.
- `Placeholder` — `tests/PictureTest.cpp:758`; CPU; `DEF_TEST`.
- `Picture_empty_serial` — `tests/PictureTest.cpp:782`; CPU; `DEF_TEST`.
- `Picture_drawsNothing` — `tests/PictureTest.cpp:797`; CPU; `DEF_TEST`.
- `Picture_emptyNestedPictureBug` — `tests/PictureTest.cpp:827`; CPU; `DEF_TEST`.
- `Picture_fillsBBH` — `tests/PictureTest.cpp:869`; CPU; `DEF_TEST`.
- `Picture_nested_op_count` — `tests/PictureTest.cpp:895`; CPU; `DEF_TEST`.
- `Picture_nested_draw_drawable` — `tests/PictureTest.cpp:936`; CPU; `DEF_TEST`.
- `Picture_recursion_limit` — `tests/PictureTest.cpp:1084`; CPU; `DEF_TEST`.
- `RecordDraw_LazySaves` — `tests/RecordDrawTest.cpp:44`; CPU; `DEF_TEST`.
- `RecordDraw_Abort` — `tests/RecordDrawTest.cpp:71`; CPU; `DEF_TEST`.
- `RecordDraw_Unbalanced` — `tests/RecordDrawTest.cpp:88`; CPU; `DEF_TEST`.
- `RecordDraw_SetMatrixClobber` — `tests/RecordDrawTest.cpp:103`; CPU; `DEF_TEST`.
- `RecordDraw_BasicBounds` — `tests/RecordDrawTest.cpp:147`; CPU; `DEF_TEST`.
- `RecordDraw_SaveLayerAffectsClipBounds` — `tests/RecordDrawTest.cpp:170`; CPU; `DEF_TEST`.
- `RecordDraw_EmptySaveLayerWithBackdropFilterAffectsCullRect` — `tests/RecordDrawTest.cpp:204`; CPU; `DEF_TEST`.
- `RecordDraw_Metadata` — `tests/RecordDrawTest.cpp:229`; CPU; `DEF_TEST`.
- `RecordDraw_SaveLayerBoundsAffectsClipBounds` — `tests/RecordDrawTest.cpp:263`; CPU; `DEF_TEST`.
- `RecordDraw_drawImage` — `tests/RecordDrawTest.cpp:283`; CPU; `DEF_TEST`.
- `RecordOpts_NoopDraw` — `tests/RecordOptsTest.cpp:34`; CPU; `DEF_TEST`.
- `RecordOpts_SingleNoopSaveRestore` — `tests/RecordOptsTest.cpp:49`; CPU; `DEF_TEST`.
- `RecordOpts_NoopSaveRestores` — `tests/RecordOptsTest.cpp:63`; CPU; `DEF_TEST`.
- `RecordOpts_SaveSaveLayerRestoreRestore` — `tests/RecordOptsTest.cpp:86`; CPU; `DEF_TEST`.
- `RecordOpts_NoopSaveLayerDrawRestore` — `tests/RecordOptsTest.cpp:144`; CPU; `DEF_TEST`.
- `RecordOpts_MergeSvgOpacityAndFilterLayers` — `tests/RecordOptsTest.cpp:225`; CPU; `DEF_TEST`.
- `savelayer_srcmode_opaque` — `tests/RecordOptsTest.cpp:423`; CPU; `DEF_TEST`.
- `savelayer_srcmode_alpha` — `tests/RecordOptsTest.cpp:427`; CPU; `DEF_TEST`.
- `RecordPattern_Simple` — `tests/RecordPatternTest.cpp:23`; CPU; `DEF_TEST`.
- `RecordPattern_StartingIndex` — `tests/RecordPatternTest.cpp:45`; CPU; `DEF_TEST`.
- `RecordPattern_DontMatchSubsequences` — `tests/RecordPatternTest.cpp:68`; CPU; `DEF_TEST`.
- `RecordPattern_Greedy` — `tests/RecordPatternTest.cpp:82`; CPU; `DEF_TEST`.
- `RecordPattern_Complex` — `tests/RecordPatternTest.cpp:102`; CPU; `DEF_TEST`.
- `RecordPattern_SaveLayerIsNotADraw` — `tests/RecordPatternTest.cpp:146`; CPU; `DEF_TEST`.
- `Record` — `tests/RecordTest.cpp:59`; CPU; `DEF_TEST`.
- `Record_defrag` — `tests/RecordTest.cpp:81`; CPU; `DEF_TEST`.
- `Record_Alignment` — `tests/RecordTest.cpp:107`; CPU; `DEF_TEST`.
- `Recorder` — `tests/RecorderTest.cpp:50`; CPU; `DEF_TEST`.
- `Recorder_RefLeaking` — `tests/RecorderTest.cpp:62`; CPU; `DEF_TEST`.
- `Recorder_drawImage_takeReference` — `tests/RecorderTest.cpp:81`; CPU; `DEF_TEST`.
- `Recorder_boundsOverflow` — `tests/RecorderTest.cpp:123`; CPU; `DEF_TEST`.
- `SkRecordingAccuracyXfermode` — `tests/RecordingXfermodeTest.cpp:144`; CPU; `DEF_TEST`.
- `serial_procs_image` — `tests/SerialProcsTest.cpp:64`; CPU; `DEF_TEST`.
- `serial_procs_picture` — `tests/SerialProcsTest.cpp:201`; CPU; `DEF_TEST`.
- `serial_typeface` — `tests/SerialProcsTest.cpp:244`; CPU; `DEF_TEST`.
- `Serialization_PictureTypeface` — `tests/SerializationTest.cpp:577`; CPU; `DEF_TEST`.
- `Serialization_Typeface` — `tests/SerializationTest.cpp:680`; CPU; `DEF_TEST`.
- `Serialization_Typeface_Sanitizer` — `tests/SerializationTest.cpp:685`; CPU; `DEF_TEST`.
- `Serialization_Typeface_Sanitizer_Spy` — `tests/SerializationTest.cpp:750`; CPU; `DEF_TEST`.
- `Serialization` — `tests/SerializationTest.cpp:843`; CPU; `DEF_TEST`.
- `Annotations` — `tests/SerializationTest.cpp:1105`; CPU; `DEF_TEST`.
- `WriteBuffer_storage` — `tests/SerializationTest.cpp:1137`; CPU; `DEF_TEST`.
- `WriteBuffer_external_memory_textblob` — `tests/SerializationTest.cpp:1166`; CPU; `DEF_TEST`.
- `WriteBuffer_external_memory_flattenable` — `tests/SerializationTest.cpp:1190`; CPU; `DEF_TEST`.
- `ReadBuffer_empty` — `tests/SerializationTest.cpp:1206`; CPU; `DEF_TEST`.
- `SkFontMetricsPriv_Basic` — `tests/SkFontMetricsPrivTest.cpp:22`; CPU; `DEF_TEST`.
- `BitmapCache_discarded_bitmap` — `tests/SkResourceCacheTest.cpp:142`; CPU; `DEF_TEST`.
- `BitmapCache_discarded_image` — `tests/SkResourceCacheTest.cpp:190`; CPU; `DEF_TEST`.
- `ResourceCache_purge` — `tests/SkResourceCacheTest.cpp:297`; CPU; `DEF_TEST`.
- `SkStrikeCache_CachePurge` — `tests/SkStrikeCacheTest.cpp:59`; CPU; `DEF_TEST`.
- `SkStrikeCache_ContextOptionsAndLimits` — `tests/SkStrikeCacheTest.cpp:91`; CPU; `DEF_TEST`.
- `SkStrikeCache_ContextPurgeAndEviction` — `tests/SkStrikeCacheTest.cpp:125`; CPU; `DEF_TEST`.
- `SkStrikeCache_ContextPurgePinned` — `tests/SkStrikeCacheTest.cpp:182`; CPU; `DEF_TEST`.
- `SkStrikeCache_IndependentContextsIsolation` — `tests/SkStrikeCacheTest.cpp:219`; CPU; `DEF_TEST`.
- `SkStrikeCache_ClientDrawTextPopulatesContextCache` — `tests/SkStrikeCacheTest.cpp:261`; CPU; `DEF_TEST`.
- `SkStrikeMultiThread` — `tests/SkStrikeTest.cpp:115`; CPU; `DEF_TEST`.
- `SkStrike_FlattenByType` — `tests/SkStrikeTest.cpp:206`; CPU; `DEF_TEST`.
- `SkScalerContext_GetImageNonDeterministicBounds` — `tests/SkStrikeTest.cpp:303`; CPU; `DEF_TEST`.
- `TextBlob_builder` — `tests/TextBlobTest.cpp:335`; CPU; `DEF_TEST`.
- `TextBlob_paint` — `tests/TextBlobTest.cpp:340`; CPU; `DEF_TEST`.
- `TextBlob_extended` — `tests/TextBlobTest.cpp:344`; CPU; `DEF_TEST`.
- `TextBlob_serialize` — `tests/TextBlobTest.cpp:440`; CPU; `DEF_TEST`.
- `TextBlob_MakeAsDrawText` — `tests/TextBlobTest.cpp:473`; CPU; `DEF_TEST`.
- `TextBlob_iter` — `tests/TextBlobTest.cpp:487`; CPU; `DEF_TEST`.
- `TextBlob_getIntercepts` — `tests/TextBlobTest.cpp:519`; CPU; `DEF_TEST`.
- `SkCanvas_drawTextBlob_b513820666` — `tests/TextBlobTest.cpp:621`; CPU; `DEF_TEST`.
- `Vertices` — `tests/VerticesTest.cpp:86`; CPU; `DEF_TEST`.
- `Vertices_clipping` — `tests/VerticesTest.cpp:155`; CPU; `DEF_TEST`.
- `Vertices_invalid` — `tests/VerticesTest.cpp:176`; CPU; `DEF_TEST`.
- `Writer32_dynamic` — `tests/Writer32Test.cpp:173`; CPU; `DEF_TEST`.
- `Writer32_small` — `tests/Writer32Test.cpp:184`; CPU; `DEF_TEST`.
- `Writer32_large` — `tests/Writer32Test.cpp:195`; CPU; `DEF_TEST`.
- `Writer32_misc` — `tests/Writer32Test.cpp:206`; CPU; `DEF_TEST`.
- `Writer32_data` — `tests/Writer32Test.cpp:212`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skottie/tests/PropertyObserver.cpp` — M30.
- `tests/AnnotationTest.cpp` — M27.
- `tests/AtlasOobTest.cpp` — M38.
- `tests/BigImageTest.cpp` — M38.
- `tests/CanvasTest.cpp` — M35, M38.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/DebugLayerManagerTest.cpp` — M40.
- `tests/DeferredDisplayListTest.cpp` — M32, M34, M38.
- `tests/GrDDLImageTest.cpp` — M32.
- `tests/GrGlyphVectorTest.cpp` — M32.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GrUniqueProxyTest.cpp` — M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageIsOpaqueTest.cpp` — M32, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFJpegEmbedTest.cpp` — M27.
- `tests/PDFMetadataAttributeTest.cpp` — M27.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PromiseImageTest.cpp` — M32, M38.
- `tests/ProtectedTest.cpp` — M38.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/StrikeForGPUTest.cpp` — M38.
- `tests/Time.cpp` — M27.
- `tests/WrappedSurfaceCopyOnWriteTest.cpp` — M38.
- `tests/graphite/CacheKeyTest.cpp` — M38.
- `tests/graphite/ImageProviderTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 14 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=22`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M22 ACCEPTED may the user hand Deepseek M23.
