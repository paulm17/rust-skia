# Milestone 27: PDF, SVG output and XML utilities

## Outcome and entry gate

Implement PDF document/object/font/image embedding, tagged output, SVG device output and XML/DOM support. Preserve document structure, metadata, destinations, tagging and stream errors. External compression/font engines remain separately accounted dependencies.

Begin implementation only after milestone 26 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement PDF document/object/font/image embedding, tagged output, SVG device output and XML/DOM support. Preserve document structure, metadata, destinations, tagging and stream errors. External compression/font engines remain separately accounted dependencies.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkDocument.h` — port to Rust.
- `include/svg/SkSVGCanvas.h` — port to Rust.
- `src/core/SkDocument.cpp` — port to Rust.
- `src/pdf/SkBitmapKey.h` — port to Rust.
- `src/pdf/SkClusterator.cpp` — port to Rust.
- `src/pdf/SkClusterator.h` — port to Rust.
- `src/pdf/SkDeflate.cpp` — port to Rust.
- `src/pdf/SkDeflate.h` — port to Rust.
- `src/pdf/SkDocument_PDF_None.cpp` — port to Rust.
- `src/pdf/SkKeyedImage.cpp` — port to Rust.
- `src/pdf/SkKeyedImage.h` — port to Rust.
- `src/pdf/SkPDFBitmap.cpp` — port to Rust.
- `src/pdf/SkPDFBitmap.h` — port to Rust.
- `src/pdf/SkPDFDevice.cpp` — port to Rust.
- `src/pdf/SkPDFDevice.h` — port to Rust.
- `src/pdf/SkPDFDocument.cpp` — port to Rust.
- `src/pdf/SkPDFDocumentPriv.h` — port to Rust.
- `src/pdf/SkPDFFont.cpp` — port to Rust.
- `src/pdf/SkPDFFont.h` — port to Rust.
- `src/pdf/SkPDFFormXObject.cpp` — port to Rust.
- `src/pdf/SkPDFFormXObject.h` — port to Rust.
- `src/pdf/SkPDFGlyphUse.h` — port to Rust.
- `src/pdf/SkPDFGradientShader.cpp` — port to Rust.
- `src/pdf/SkPDFGradientShader.h` — port to Rust.
- `src/pdf/SkPDFGraphicStackState.cpp` — port to Rust.
- `src/pdf/SkPDFGraphicStackState.h` — port to Rust.
- `src/pdf/SkPDFGraphicState.cpp` — port to Rust.
- `src/pdf/SkPDFGraphicState.h` — port to Rust.
- `src/pdf/SkPDFMakeCIDGlyphWidthsArray.cpp` — port to Rust.
- `src/pdf/SkPDFMakeCIDGlyphWidthsArray.h` — port to Rust.
- `src/pdf/SkPDFMakeToUnicodeCmap.cpp` — port to Rust.
- `src/pdf/SkPDFMakeToUnicodeCmap.h` — port to Rust.
- `src/pdf/SkPDFMetadata.cpp` — port to Rust.
- `src/pdf/SkPDFMetadata.h` — port to Rust.
- `src/pdf/SkPDFResourceDict.cpp` — port to Rust.
- `src/pdf/SkPDFResourceDict.h` — port to Rust.
- `src/pdf/SkPDFShader.cpp` — port to Rust.
- `src/pdf/SkPDFShader.h` — port to Rust.
- `src/pdf/SkPDFSubsetFont.cpp` — port to Rust.
- `src/pdf/SkPDFSubsetFont.h` — port to Rust.
- `src/pdf/SkPDFTag.cpp` — port to Rust.
- `src/pdf/SkPDFTag.h` — port to Rust.
- `src/pdf/SkPDFType1Font.cpp` — port to Rust.
- `src/pdf/SkPDFType1Font.h` — port to Rust.
- `src/pdf/SkPDFTypes.cpp` — port to Rust.
- `src/pdf/SkPDFTypes.h` — port to Rust.
- `src/pdf/SkPDFUnion.h` — port to Rust.
- `src/pdf/SkPDFUtils.cpp` — port to Rust.
- `src/pdf/SkPDFUtils.h` — port to Rust.
- `src/pdf/SkUUID.h` — port to Rust.
- `src/svg/SkSVGCanvas.cpp` — port to Rust.
- `src/svg/SkSVGDevice.cpp` — port to Rust.
- `src/svg/SkSVGDevice.h` — port to Rust.
- `src/xml/SkDOM.cpp` — port to Rust.
- `src/xml/SkDOM.h` — port to Rust.
- `src/xml/SkXMLParser.cpp` — port to Rust.
- `src/xml/SkXMLParser.h` — port to Rust.
- `src/xml/SkXMLWriter.cpp` — port to Rust.
- `src/xml/SkXMLWriter.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/svg/BUILD.bazel`.
- `src/pdf/BUILD.bazel`.
- `src/svg/BUILD.bazel`.
- `src/xml/BUILD.bazel`.

## Codex: acceptance-test contract

Translate PDF and SVG-device tests, including tagged links/tables, metadata and malformed data. Verify structural assertions and expected byte/text properties; use rendered comparisons only where the original suite does. Keep deterministic metadata settings for reproducible output.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/AnnotationTest.cpp`.
- `tests/PDFDeflateWStreamTest.cpp`.
- `tests/PDFDocumentTest.cpp`.
- `tests/PDFGlyphsToUnicodeTest.cpp`.
- `tests/PDFJpegEmbedTest.cpp`.
- `tests/PDFMetadataAttributeTest.cpp`.
- `tests/PDFOpaqueSrcModeToSrcOverTest.cpp`.
- `tests/PDFPrimitivesTest.cpp`.
- `tests/PDFTaggedLinkTest.cpp`.
- `tests/PDFTaggedTableTest.cpp`.
- `tests/PDFTaggedTest.cpp`.
- `tests/SVGDeviceTest.cpp`.
- `tests/SkDOMTest.cpp`.
- `tests/Time.cpp`.

### Original test registrations assigned to this gate

- `Annotation_NoDraw` — `tests/AnnotationTest.cpp:39`; CPU; `DEF_TEST`.
- `Annotation_PdfLink` — `tests/AnnotationTest.cpp:55`; CPU; `DEF_TEST`.
- `Annotation_PdfDefineNamedDestination` — `tests/AnnotationTest.cpp:74`; CPU; `DEF_TEST`.
- `Annotation_SvgLink` — `tests/AnnotationTest.cpp:97`; CPU; `DEF_TEST`.
- `Annotation_SvgLinkNamedDestination` — `tests/AnnotationTest.cpp:114`; CPU; `DEF_TEST`.
- `SkPDF_DeflateWStream` — `tests/PDFDeflateWStreamTest.cpp:120`; CPU; `DEF_TEST`.
- `SkPDF_document_tests` — `tests/PDFDocumentTest.cpp:136`; CPU; `DEF_TEST`.
- `SkPDF_document_skbug_4734` — `tests/PDFDocumentTest.cpp:145`; CPU; `DEF_TEST`.
- `SkPDF_pdfa_document` — `tests/PDFDocumentTest.cpp:169`; CPU; `DEF_TEST`.
- `SkPDF_unicode_metadata` — `tests/PDFDocumentTest.cpp:216`; CPU; `DEF_TEST`.
- `SkPDF_multiple_pages` — `tests/PDFDocumentTest.cpp:250`; CPU; `DEF_TEST`.
- `SkPDF_abort_jobs` — `tests/PDFDocumentTest.cpp:267`; CPU; `DEF_TEST`.
- `SkPDF_ToUnicode` — `tests/PDFGlyphsToUnicodeTest.cpp:43`; CPU; `DEF_TEST`.
- `SkPDF_JpegEmbedTest` — `tests/PDFJpegEmbedTest.cpp:65`; CPU; `DEF_TEST`.
- `SkPDF_JpegIdentification` — `tests/PDFJpegEmbedTest.cpp:160`; CPU; `DEF_TEST`.
- `SkPDF_Metadata` — `tests/PDFMetadataAttributeTest.cpp:23`; CPU; `DEF_TEST`.
- `SkPDF_OpaqueSrcModeToSrcOver` — `tests/PDFOpaqueSrcModeToSrcOverTest.cpp:41`; CPU; `DEF_TEST`.
- `SkPDF_Primitives` — `tests/PDFPrimitivesTest.cpp:274`; CPU; `DEF_TEST`.
- `SkPDF_ImageFilter` — `tests/PDFPrimitivesTest.cpp:321`; CPU; `DEF_TEST`.
- `SkPDF_FontCanEmbedTypeface` — `tests/PDFPrimitivesTest.cpp:342`; CPU; `DEF_TEST`.
- `SkPDF_Primitives_Scalar` — `tests/PDFPrimitivesTest.cpp:386`; CPU; `DEF_TEST`.
- `SkPDF_Primitives_Color` — `tests/PDFPrimitivesTest.cpp:407`; CPU; `DEF_TEST`.
- `SkPDF_Clusterator` — `tests/PDFPrimitivesTest.cpp:430`; CPU; `DEF_TEST`.
- `fuzz875632f0` — `tests/PDFPrimitivesTest.cpp:478`; CPU; `DEF_TEST`.
- `SkPDF_EmitPath` — `tests/PDFPrimitivesTest.cpp:501`; CPU; `DEF_TEST`.
- `SkPDF_RasterizeAlphaGradientForPrinting` — `tests/PDFPrimitivesTest.cpp:620`; CPU; `DEF_TEST`.
- `SkPDF_RasterizeAlphaGradientForPrinting_OpaqueUnchanged` — `tests/PDFPrimitivesTest.cpp:642`; CPU; `DEF_TEST`.
- `SkPDF_GradientDegenerateStopsDoesNotCrash` — `tests/PDFPrimitivesTest.cpp:661`; CPU; `DEF_TEST`.
- `SkPDF_tagged_links` — `tests/PDFTaggedLinkTest.cpp:39`; CPU; `DEF_TEST`.
- `SkPDF_tagged_table` — `tests/PDFTaggedTableTest.cpp:36`; CPU; `DEF_TEST`.
- `SkPDF_structelem_header_outline_doc` — `tests/PDFTaggedTest.cpp:228`; CPU; `DEF_TEST`.
- `SkPDF_structelem_outline_doc` — `tests/PDFTaggedTest.cpp:236`; CPU; `DEF_TEST`.
- `SkPDF_structelem_header_outline_doc_noheader` — `tests/PDFTaggedTest.cpp:244`; CPU; `DEF_TEST`.
- `SkPDF_tagged_saveLayer` — `tests/PDFTaggedTest.cpp:253`; CPU; `DEF_TEST`.
- `SVGDevice_whitespace_pos` — `tests/SVGDeviceTest.cpp:185`; CPU; `DEF_TEST`.
- `SVGDevice_image_shader_norepeat` — `tests/SVGDeviceTest.cpp:267`; CPU; `DEF_TEST`.
- `SVGDevice_image_shader_tilex` — `tests/SVGDeviceTest.cpp:292`; CPU; `DEF_TEST`.
- `SVGDevice_image_shader_tiley` — `tests/SVGDeviceTest.cpp:322`; CPU; `DEF_TEST`.
- `SVGDevice_image_shader_tileboth` — `tests/SVGDeviceTest.cpp:352`; CPU; `DEF_TEST`.
- `SVGDevice_ColorFilters` — `tests/SVGDeviceTest.cpp:380`; CPU; `DEF_TEST`.
- `SVGDevice_textpath` — `tests/SVGDeviceTest.cpp:461`; CPU; `DEF_TEST`.
- `SVGDevice_fill_stroke` — `tests/SVGDeviceTest.cpp:492`; CPU; `DEF_TEST`.
- `SVGDevice_fill_opacity_black_fill` — `tests/SVGDeviceTest.cpp:532`; CPU; `DEF_TEST`.
- `SVGDevice_fill_rect_hex` — `tests/SVGDeviceTest.cpp:568`; CPU; `DEF_TEST`.
- `SVGDevice_fill_rect_custom_hex` — `tests/SVGDeviceTest.cpp:585`; CPU; `DEF_TEST`.
- `SVGDevice_fill_stroke_rect_hex` — `tests/SVGDeviceTest.cpp:617`; CPU; `DEF_TEST`.
- `SVGDevice_rect_with_path_effect` — `tests/SVGDeviceTest.cpp:644`; CPU; `DEF_TEST`.
- `SVGDevice_rrect_with_path_effect` — `tests/SVGDeviceTest.cpp:664`; CPU; `DEF_TEST`.
- `SVGDevice_oval_with_path_effect` — `tests/SVGDeviceTest.cpp:684`; CPU; `DEF_TEST`.
- `SVGDevice_path_effect` — `tests/SVGDeviceTest.cpp:705`; CPU; `DEF_TEST`.
- `SVGDevice_relative_path_encoding` — `tests/SVGDeviceTest.cpp:743`; CPU; `DEF_TEST`.
- `SVGDevice_color_shader` — `tests/SVGDeviceTest.cpp:765`; CPU; `DEF_TEST`.
- `SVGDevice_parse_minmax` — `tests/SVGDeviceTest.cpp:785`; CPU; `DEF_TEST`.
- `SkDOM_test` — `tests/SkDOMTest.cpp:26`; CPU; `DEF_TEST`.
- `SkPDFUtils_GetDateTime` — `tests/Time.cpp:19`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/CanvasTest.cpp` — M35, M38.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=27`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M27 ACCEPTED may the user hand Deepseek M28.
