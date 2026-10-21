# Milestone 18: Font objects, glyph data and SFNT

## Outcome and entry gate

Implement font/typeface abstractions, font metrics, glyph structures, scaler contracts, SFNT tables, descriptors, strike/cache foundations and text blobs. Preserve glyph IDs, table byte access, font arguments and lifetime rules. CoreText integration follows next.

Begin implementation only after milestone 17 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement font/typeface abstractions, font metrics, glyph structures, scaler contracts, SFNT tables, descriptors, strike/cache foundations and text blobs. Preserve glyph IDs, table byte access, font arguments and lifetime rules. CoreText integration follows next.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkFont.h` — port to Rust.
- `include/core/SkFontArguments.h` — port to Rust.
- `include/core/SkFontMetrics.h` — port to Rust.
- `include/core/SkFontMgr.h` — port to Rust.
- `include/core/SkFontParameters.h` — port to Rust.
- `include/core/SkFontScanner.h` — port to Rust.
- `include/core/SkFontStyle.h` — port to Rust.
- `include/core/SkFontTypes.h` — port to Rust.
- `include/core/SkStrikeRef.h` — port to Rust.
- `include/core/SkTextBlob.h` — port to Rust.
- `include/core/SkTypeface.h` — port to Rust.
- `include/private/chromium/SkChromeRemoteGlyphCache.h` — port to Rust.
- `include/utils/SkCustomTypeface.h` — port to Rust.
- `include/utils/SkOrderedFontMgr.h` — port to Rust.
- `src/core/SkAdvancedTypefaceMetrics.h` — port to Rust.
- `src/core/SkDescriptor.cpp` — port to Rust.
- `src/core/SkDescriptor.h` — port to Rust.
- `src/core/SkFont.cpp` — port to Rust.
- `src/core/SkFontDescriptor.cpp` — port to Rust.
- `src/core/SkFontDescriptor.h` — port to Rust.
- `src/core/SkFontMetricsPriv.cpp` — port to Rust.
- `src/core/SkFontMetricsPriv.h` — port to Rust.
- `src/core/SkFontMgr.cpp` — port to Rust.
- `src/core/SkFontPriv.h` — port to Rust.
- `src/core/SkFontStream.cpp` — port to Rust.
- `src/core/SkFontStream.h` — port to Rust.
- `src/core/SkGlyph.cpp` — port to Rust.
- `src/core/SkGlyph.h` — port to Rust.
- `src/core/SkScalerContext.cpp` — port to Rust.
- `src/core/SkScalerContext.h` — port to Rust.
- `src/core/SkStrike.cpp` — port to Rust.
- `src/core/SkStrike.h` — port to Rust.
- `src/core/SkStrikeCache.cpp` — port to Rust.
- `src/core/SkStrikeCache.h` — port to Rust.
- `src/core/SkStrikeRef.cpp` — port to Rust.
- `src/core/SkStrikeSpec.cpp` — port to Rust.
- `src/core/SkStrikeSpec.h` — port to Rust.
- `src/core/SkTextBlob.cpp` — port to Rust.
- `src/core/SkTextBlobPriv.h` — port to Rust.
- `src/core/SkTextFormatParams.h` — port to Rust.
- `src/core/SkTypeface.cpp` — port to Rust.
- `src/core/SkTypefaceCache.cpp` — port to Rust.
- `src/core/SkTypefaceCache.h` — port to Rust.
- `src/core/SkTypeface_remote.cpp` — port to Rust.
- `src/core/SkTypeface_remote.h` — port to Rust.
- `src/sfnt/SkIBMFamilyClass.h` — port to Rust.
- `src/sfnt/SkOTTableTypes.h` — port to Rust.
- `src/sfnt/SkOTTable_EBDT.h` — port to Rust.
- `src/sfnt/SkOTTable_EBLC.h` — port to Rust.
- `src/sfnt/SkOTTable_EBSC.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2_V0.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2_V1.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2_V2.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2_V3.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2_V4.h` — port to Rust.
- `src/sfnt/SkOTTable_OS_2_VA.h` — port to Rust.
- `src/sfnt/SkOTTable_fvar.h` — port to Rust.
- `src/sfnt/SkOTTable_gasp.h` — port to Rust.
- `src/sfnt/SkOTTable_glyf.h` — port to Rust.
- `src/sfnt/SkOTTable_head.h` — port to Rust.
- `src/sfnt/SkOTTable_hhea.h` — port to Rust.
- `src/sfnt/SkOTTable_hmtx.h` — port to Rust.
- `src/sfnt/SkOTTable_loca.h` — port to Rust.
- `src/sfnt/SkOTTable_maxp.h` — port to Rust.
- `src/sfnt/SkOTTable_maxp_CFF.h` — port to Rust.
- `src/sfnt/SkOTTable_maxp_TT.h` — port to Rust.
- `src/sfnt/SkOTTable_name.cpp` — port to Rust.
- `src/sfnt/SkOTTable_name.h` — port to Rust.
- `src/sfnt/SkOTTable_post.h` — port to Rust.
- `src/sfnt/SkOTTable_sbix.h` — port to Rust.
- `src/sfnt/SkOTUtils.cpp` — port to Rust.
- `src/sfnt/SkOTUtils.h` — port to Rust.
- `src/sfnt/SkPanose.h` — port to Rust.
- `src/sfnt/SkSFNTHeader.h` — port to Rust.
- `src/sfnt/SkTTCFHeader.h` — port to Rust.
- `src/utils/SkCharToGlyphCache.cpp` — port to Rust.
- `src/utils/SkCharToGlyphCache.h` — port to Rust.
- `src/utils/SkCustomTypeface.cpp` — port to Rust.
- `src/utils/SkOrderedFontMgr.cpp` — port to Rust.
- `src/utils/mac/SkCTFont.cpp` — port to Rust.
- `src/utils/mac/SkCTFont.h` — port to Rust.
- `src/utils/mac/SkCTFontCreateExactCopy.cpp` — port to Rust.
- `src/utils/mac/SkCTFontCreateExactCopy.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `src/sfnt/BUILD.bazel`.

## Codex: acceptance-test contract

Translate table, descriptor, glyph and pure font-data tests using bundled fonts and mock/test typefaces. Retain exact font fixtures and metrics assertions. Font_flatten and other high-level serialization tests are explicitly scheduled at M22.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/CharToGlyphCache.cpp`.
- `tests/DrawTextTest.cpp`.
- `tests/FCITest.cpp`.
- `tests/UnicodeTest.cpp`.

### Original test registrations assigned to this gate

- `chartoglyph_cache` — `tests/CharToGlyphCache.cpp:30`; CPU; `DEF_TEST`.
- `DrawText_dashout` — `tests/DrawTextTest.cpp:71`; CPU; `DEF_TEST`.
- `DrawText_weirdCoordinates` — `tests/DrawTextTest.cpp:117`; CPU; `DEF_TEST`.
- `DrawText_weirdMatricies` — `tests/DrawTextTest.cpp:136`; CPU; `DEF_TEST`.
- `DrawText_noglyphs` — `tests/DrawTextTest.cpp:173`; CPU; `DEF_TEST`.
- `FontConfigInterface_MatchStyleNamedInstance` — `tests/FCITest.cpp:61`; CPU; `DEF_TEST`.
- `FontConfigInterface_MissingFontRejection` — `tests/FCITest.cpp:155`; CPU; `DEF_TEST`.
- `FontConfigInterface_GenericFallbackAllowed` — `tests/FCITest.cpp:175`; CPU; `DEF_TEST`.
- `FontConfigInterface_OnlyCheckFirstValidCandidate` — `tests/FCITest.cpp:191`; CPU; `DEF_TEST`.
- `Unicode_textencodings` — `tests/UnicodeTest.cpp:22`; CPU; `DEF_TEST`.
- `glyphs_to_unichars` — `tests/UnicodeTest.cpp:53`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skottie/tests/Expression.cpp` — M30.
- `modules/skottie/tests/PropertyObserver.cpp` — M30.
- `modules/skottie/tests/Shaper.cpp` — M30.
- `modules/skottie/tests/Text.cpp` — M30.
- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/skshaper/tests/ShaperTest.cpp` — M20.
- `modules/skunicode/tests/SkUnicodeTest.cpp` — M20.
- `tests/AtlasOobTest.cpp` — M38.
- `tests/DescriptorTest.cpp` — M22.
- `tests/DirectMaskLimitTest.cpp` — M38.
- `tests/DrawOpAtlasTest.cpp` — M32, M38.
- `tests/FlattenDrawableTest.cpp` — M22.
- `tests/FontHostStreamTest.cpp` — M19.
- `tests/FontHostTest.cpp` — M19.
- `tests/FontMgrTest.cpp` — M19.
- `tests/FontNamesTest.cpp` — M19.
- `tests/FontTest.cpp` — M22.
- `tests/FontationsFtCompTest.cpp` — M20.
- `tests/FontationsTest.cpp` — M19.
- `tests/GrGlyphVectorTest.cpp` — M32.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/MultiPictureDocumentTest.cpp` — M32, M38.
- `tests/PDFDocumentTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PDFTaggedLinkTest.cpp` — M27.
- `tests/PDFTaggedTableTest.cpp` — M27.
- `tests/PDFTaggedTest.cpp` — M27.
- `tests/PaintTest.cpp` — M22.
- `tests/PathTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SerialProcsTest.cpp` — M22.
- `tests/SerializationTest.cpp` — M22.
- `tests/SkFontMetricsPrivTest.cpp` — M22.
- `tests/SkGlyphTest.cpp` — M26.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkStrikeCacheTest.cpp` — M22.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/Skbug5221.cpp` — M32, M38.
- `tests/SlugTest.cpp` — M38.
- `tests/StrikeForGPUTest.cpp` — M38.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/TypefaceMacTest.cpp` — M19.
- `tests/TypefaceTest.cpp` — M19.
- `tests/graphite/RecordingOrderTest.cpp` — M38.
- `tests/graphite/precompile/ThreadedPrecompileTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 34 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=18`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M18 ACCEPTED may the user hand Deepseek M19.
