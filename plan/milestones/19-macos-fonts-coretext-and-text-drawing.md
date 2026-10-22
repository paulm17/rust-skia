# Milestone 19: macOS fonts, CoreText and text drawing

## Outcome and entry gate

Implement CoreText/CoreGraphics font and image bridges plus macOS font discovery, scaler behavior, glyph runs, text drawing and strike integration. Support macOS-compatible FreeType/Fontations optional paths where present. Use narrowly scoped Apple framework FFI with Rust-owned Skia logic.

Begin implementation only after milestone 18 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement CoreText/CoreGraphics font and image bridges plus macOS font discovery, scaler behavior, glyph runs, text drawing and strike integration. Support macOS-compatible FreeType/Fontations optional paths where present. Use narrowly scoped Apple framework FFI with Rust-owned Skia logic.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/ports/SkCFObject.h` — port to Rust.
- `include/ports/SkFontMgr_Fontations.h` — port to Rust.
- `include/ports/SkFontMgr_data.h` — port to Rust.
- `include/ports/SkFontMgr_directory.h` — port to Rust.
- `include/ports/SkFontMgr_empty.h` — port to Rust.
- `include/ports/SkFontMgr_mac_ct.h` — port to Rust.
- `include/ports/SkFontScanner_Fontations.h` — port to Rust.
- `include/ports/SkFontScanner_FreeType.h` — port to Rust.
- `include/ports/SkTypeface_fontations.h` — port to Rust.
- `include/ports/SkTypeface_mac.h` — port to Rust.
- `include/utils/SkTextUtils.h` — port to Rust.
- `src/core/SkDistanceFieldGen.cpp` — port to Rust.
- `src/core/SkDistanceFieldGen.h` — port to Rust.
- `src/core/SkDraw_text.cpp` — port to Rust.
- `src/ports/SkFontHost_FreeType.cpp` — port to Rust.
- `src/ports/SkFontHost_FreeType_common.cpp` — port to Rust.
- `src/ports/SkFontHost_FreeType_common.h` — port to Rust.
- `src/ports/SkFontMgr_custom.cpp` — port to Rust.
- `src/ports/SkFontMgr_custom.h` — port to Rust.
- `src/ports/SkFontMgr_custom_directory.cpp` — port to Rust.
- `src/ports/SkFontMgr_custom_embedded.cpp` — port to Rust.
- `src/ports/SkFontMgr_custom_empty.cpp` — port to Rust.
- `src/ports/SkFontMgr_fontations_empty.cpp` — port to Rust.
- `src/ports/SkFontMgr_mac_ct.cpp` — port to Rust.
- `src/ports/SkFontScanner_FreeType_priv.h` — port to Rust.
- `src/ports/SkFontScanner_fontations.cpp` — port to Rust.
- `src/ports/SkFontScanner_fontations_priv.h` — port to Rust.
- `src/ports/SkScalerContext_mac_ct.cpp` — port to Rust.
- `src/ports/SkScalerContext_mac_ct.h` — port to Rust.
- `src/ports/SkTypeface_FreeType.h` — port to Rust.
- `src/ports/SkTypeface_fontations.cpp` — port to Rust.
- `src/ports/SkTypeface_fontations_factory.h` — port to Rust.
- `src/ports/SkTypeface_fontations_priv.h` — port to Rust.
- `src/ports/SkTypeface_mac_ct.cpp` — port to Rust.
- `src/ports/SkTypeface_mac_ct.h` — port to Rust.
- `src/ports/SkTypeface_proxy.cpp` — port to Rust.
- `src/ports/SkTypeface_proxy.h` — port to Rust.
- `src/ports/fontations/src/base.rs` — integrate existing Rust.
- `src/ports/fontations/src/bitmap.rs` — integrate existing Rust.
- `src/ports/fontations/src/colr.rs` — integrate existing Rust.
- `src/ports/fontations/src/ffi.rs` — integrate existing Rust.
- `src/ports/fontations/src/hinting.rs` — integrate existing Rust.
- `src/ports/fontations/src/names.rs` — integrate existing Rust.
- `src/ports/fontations/src/skpath_bridge.h` — port to Rust.
- `src/ports/fontations/src/verbs_points_pen.rs` — integrate existing Rust.
- `src/text/GlyphRun.cpp` — port to Rust.
- `src/text/GlyphRun.h` — port to Rust.
- `src/text/SlugFromBuffer.cpp` — port to Rust.
- `src/text/StrikeForGPU.cpp` — port to Rust.
- `src/text/StrikeForGPU.h` — port to Rust.
- `src/text/gpu/DistanceFieldAdjustTable.cpp` — port to Rust.
- `src/text/gpu/DistanceFieldAdjustTable.h` — port to Rust.
- `src/text/gpu/GlyphUtils.h` — port to Rust.
- `src/text/gpu/GlyphVector.cpp` — port to Rust.
- `src/text/gpu/GlyphVector.h` — port to Rust.
- `src/text/gpu/PackedGPUGlyphID.h` — port to Rust.
- `src/text/gpu/SDFMaskFilter.cpp` — port to Rust.
- `src/text/gpu/SDFMaskFilter.h` — port to Rust.
- `src/text/gpu/SkChromeRemoteGlyphCache.cpp` — port to Rust.
- `src/text/gpu/SlugImpl.cpp` — port to Rust.
- `src/text/gpu/SlugImpl.h` — port to Rust.
- `src/text/gpu/StrikeCache.cpp` — port to Rust.
- `src/text/gpu/StrikeCache.h` — port to Rust.
- `src/text/gpu/SubRunAllocator.cpp` — port to Rust.
- `src/text/gpu/SubRunAllocator.h` — port to Rust.
- `src/text/gpu/SubRunContainer.cpp` — port to Rust.
- `src/text/gpu/SubRunContainer.h` — port to Rust.
- `src/text/gpu/SubRunControl.cpp` — port to Rust.
- `src/text/gpu/SubRunControl.h` — port to Rust.
- `src/text/gpu/TextBlob.cpp` — port to Rust.
- `src/text/gpu/TextBlob.h` — port to Rust.
- `src/text/gpu/TextBlobRedrawCoordinator.cpp` — port to Rust.
- `src/text/gpu/TextBlobRedrawCoordinator.h` — port to Rust.
- `src/text/gpu/VertexFiller.cpp` — port to Rust.
- `src/text/gpu/VertexFiller.h` — port to Rust.
- `src/utils/SkTextUtils.cpp` — port to Rust.
- `tools/fonts/RandomScalerContext.cpp` — port to Rust.
- `tools/fonts/RandomScalerContext.h` — port to Rust.
- `tools/fonts/create_test_font.cpp` — port to Rust.
- `tools/fonts/create_test_font_color.cpp` — port to Rust.
- `tools/fonts/test_font_index.inc` — port to Rust.
- `tools/fonts/test_font_monospace.inc` — port to Rust.
- `tools/fonts/test_font_sans_serif.inc` — port to Rust.
- `tools/fonts/test_font_serif.inc` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `src/ports/fontations/BUILD.bazel`.
- `src/ports/fontations/Cargo.toml`.
- `src/text/BUILD.bazel`.
- `src/text/gpu/BUILD.bazel`.
- `tools/fonts/BUILD.bazel`.
- `tools/fonts/generate_fir_coeff.py`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate macOS typeface, font scanner, glyph-cache and text-drawing tests. Run with pinned fixture fonts, not whichever system font happens to be installed. Compare rasterized glyphs under the same OS/font/backend configuration and preserve native resource lifetime checks.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/FontHostStreamTest.cpp`.
- `tests/FontHostTest.cpp`.
- `tests/FontMgrFlags.cpp`.
- `tests/FontMgrTest.cpp`.
- `tests/FontNamesTest.cpp`.
- `tests/FontScanner.h`.
- `tests/FontationsTest.cpp`.
- `tests/TypefaceMacTest.cpp`.
- `tests/TypefaceTest.cpp`.
- `tools/TestFontDataProvider.cpp`.
- `tools/TestFontDataProvider.h`.
- `tools/fonts/FontToolUtils.cpp`.
- `tools/fonts/FontToolUtils.h`.
- `tools/fonts/TestEmptyTypeface.h`.
- `tools/fonts/TestFontMgr.cpp`.
- `tools/fonts/TestFontMgr.h`.
- `tools/fonts/TestSVGTypeface.cpp`.
- `tools/fonts/TestSVGTypeface.h`.
- `tools/fonts/TestTypeface.cpp`.
- `tools/fonts/TestTypeface.h`.

### Original test registrations assigned to this gate

- `FontHostStream` — `tests/FontHostStreamTest.cpp:72`; CPU; `DEF_TEST`.
- `FontHost` — `tests/FontHostTest.cpp:288`; CPU; `DEF_TEST`.
- `FontMgr_Font` — `tests/FontMgrTest.cpp:41`; CPU; `DEF_TEST`.
- `FontMgr_AliasNames` — `tests/FontMgrTest.cpp:75`; CPU; `DEF_TEST`.
- `FontMgr_Iter` — `tests/FontMgrTest.cpp:104`; CPU; `DEF_TEST`.
- `FontMgr_Match` — `tests/FontMgrTest.cpp:214`; CPU; `DEF_TEST`.
- `FontMgr_MatchFamilyStyle` — `tests/FontMgrTest.cpp:220`; CPU; `DEF_TEST`.
- `FontMgr_MatchStyleCSS3` — `tests/FontMgrTest.cpp:244`; CPU; `DEF_TEST`.
- `FontMgr_MatchCharacter` — `tests/FontMgrTest.cpp:858`; CPU; `DEF_TEST`.
- `FontNames` — `tests/FontNamesTest.cpp:243`; CPU; `DEF_TEST`.
- `Fontations_DoNotMakeFromNull` — `tests/FontationsTest.cpp:82`; CPU; `DEF_TEST`.
- `Fontations_DoNotMakeFromNonSfnt` — `tests/FontationsTest.cpp:89`; CPU; `DEF_TEST`.
- `Fontations_MakeFromFont` — `tests/FontationsTest.cpp:98`; CPU; `DEF_TEST`.
- `Fontations_MakeFromCollection` — `tests/FontationsTest.cpp:104`; CPU; `DEF_TEST`.
- `Fontations_MakeFromCollectionNonNullIndex` — `tests/FontationsTest.cpp:110`; CPU; `DEF_TEST`.
- `Fontations_CloneVariableCollectionNonNullIndex` — `tests/FontationsTest.cpp:118`; CPU; `DEF_TEST`.
- `Fontations_DoNotMakeFromCollection_Invalid_Index` — `tests/FontationsTest.cpp:141`; CPU; `DEF_TEST`.
- `Fontations_TableData` — `tests/FontationsTest.cpp:149`; CPU; `DEF_TEST`.
- `Fontations_TableTags` — `tests/FontationsTest.cpp:210`; CPU; `DEF_TEST`.
- `Fontations_VariationPosition` — `tests/FontationsTest.cpp:227`; CPU; `DEF_TEST`.
- `Fontations_VariationParameters` — `tests/FontationsTest.cpp:270`; CPU; `DEF_TEST`.
- `Fontations_VariationParameters_BufferTooSmall` — `tests/FontationsTest.cpp:288`; CPU; `DEF_TEST`.
- `Fontations_SyntheticCapHeight` — `tests/FontationsTest.cpp:300`; CPU; `DEF_TEST`.
- `Fontations_SyntheticXHeight` — `tests/FontationsTest.cpp:328`; CPU; `DEF_TEST`.
- `Fontations_CffLinearAdvanceWidth` — `tests/FontationsTest.cpp:361`; CPU; `DEF_TEST`.
- `TypefaceMacVariation` — `tests/TypefaceMacTest.cpp:31`; CPU; `DEF_TEST`.
- `TypefaceStyle` — `tests/TypefaceTest.cpp:110`; CPU; `DEF_TEST`.
- `TypefaceGlyphToUnicode` — `tests/TypefaceTest.cpp:131`; CPU; `DEF_TEST`.
- `TypefaceStyleVariable` — `tests/TypefaceTest.cpp:162`; CPU; `DEF_TEST`.
- `TypefacePostScriptName` — `tests/TypefaceTest.cpp:251`; CPU; `DEF_TEST`.
- `TypefaceNameIter` — `tests/TypefaceTest.cpp:267`; CPU; `DEF_TEST`.
- `TypefaceRoundTrip` — `tests/TypefaceTest.cpp:296`; CPU; `DEF_TEST`.
- `FontDescriptorNegativeVariationSerialize` — `tests/TypefaceTest.cpp:311`; CPU; `DEF_TEST`.
- `TypefaceAxes` — `tests/TypefaceTest.cpp:335`; CPU; `DEF_TEST`.
- `TypefaceVariationIndex` — `tests/TypefaceTest.cpp:466`; CPU; `DEF_TEST`.
- `Typeface` — `tests/TypefaceTest.cpp:504`; CPU; `DEF_TEST`.
- `TypefaceAxesParameters` — `tests/TypefaceTest.cpp:518`; CPU; `DEF_TEST`.
- `TypefaceCache` — `tests/TypefaceTest.cpp:669`; CPU; `DEF_TEST`.
- `Typeface_serialize` — `tests/TypefaceTest.cpp:712`; CPU; `DEF_TEST`.
- `Typeface_glyph_to_char` — `tests/TypefaceTest.cpp:719`; CPU; `DEF_TEST`.
- `LegacyMakeTypeface` — `tests/TypefaceTest.cpp:762`; CPU; `DEF_TEST`.
- `CustomTypeface_invalid_glyphid` — `tests/TypefaceTest.cpp:786`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/FontScanner_FontationsTest.cpp` — M22.
- `tests/FontScanner_FreeTypeTest.cpp` — M22.
- `tests/FontationsFtCompTest.cpp` — M20.
- `tests/GrGlyphVectorTest.cpp` — M32.
- `tests/GrTextBlobTest.cpp` — M32, M38.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/SkRemoteGlyphCacheTest.cpp` — M38.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/StrikeForGPUTest.cpp` — M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 47 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=19`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M19 ACCEPTED may the user hand Deepseek M20.
