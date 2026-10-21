# Milestone 21: Paragraph layout and editing

## Outcome and entry gate

Implement skparagraph and skplaintexteditor surfaces, text styles, runs, line breaking, ellipsis, hit testing, selections and editing. Preserve UTF indexing, rounding, font fallback and cache invalidation across layout calls.

Begin implementation only after milestone 20 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement skparagraph and skplaintexteditor surfaces, text styles, runs, line breaking, ellipsis, hit testing, selections and editing. Preserve UTF indexing, rounding, font fallback and cache invalidation across layout calls.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `modules/skparagraph/include/DartTypes.h` — port to Rust.
- `modules/skparagraph/include/FontArguments.h` — port to Rust.
- `modules/skparagraph/include/FontCollection.h` — port to Rust.
- `modules/skparagraph/include/Metrics.h` — port to Rust.
- `modules/skparagraph/include/Paragraph.h` — port to Rust.
- `modules/skparagraph/include/ParagraphBuilder.h` — port to Rust.
- `modules/skparagraph/include/ParagraphCache.h` — port to Rust.
- `modules/skparagraph/include/ParagraphPainter.h` — port to Rust.
- `modules/skparagraph/include/ParagraphStyle.h` — port to Rust.
- `modules/skparagraph/include/TextShadow.h` — port to Rust.
- `modules/skparagraph/include/TextStyle.h` — port to Rust.
- `modules/skparagraph/include/TypefaceFontProvider.h` — port to Rust.
- `modules/skparagraph/slides/ParagraphSlide.cpp` — port to Rust.
- `modules/skparagraph/src/Decorations.cpp` — port to Rust.
- `modules/skparagraph/src/Decorations.h` — port to Rust.
- `modules/skparagraph/src/FontArguments.cpp` — port to Rust.
- `modules/skparagraph/src/FontCollection.cpp` — port to Rust.
- `modules/skparagraph/src/Iterators.h` — port to Rust.
- `modules/skparagraph/src/OneLineShaper.cpp` — port to Rust.
- `modules/skparagraph/src/OneLineShaper.h` — port to Rust.
- `modules/skparagraph/src/ParagraphBuilderImpl.cpp` — port to Rust.
- `modules/skparagraph/src/ParagraphBuilderImpl.h` — port to Rust.
- `modules/skparagraph/src/ParagraphCache.cpp` — port to Rust.
- `modules/skparagraph/src/ParagraphImpl.cpp` — port to Rust.
- `modules/skparagraph/src/ParagraphImpl.h` — port to Rust.
- `modules/skparagraph/src/ParagraphPainterImpl.cpp` — port to Rust.
- `modules/skparagraph/src/ParagraphPainterImpl.h` — port to Rust.
- `modules/skparagraph/src/ParagraphStyle.cpp` — port to Rust.
- `modules/skparagraph/src/Run.cpp` — port to Rust.
- `modules/skparagraph/src/Run.h` — port to Rust.
- `modules/skparagraph/src/TextLine.cpp` — port to Rust.
- `modules/skparagraph/src/TextLine.h` — port to Rust.
- `modules/skparagraph/src/TextShadow.cpp` — port to Rust.
- `modules/skparagraph/src/TextStyle.cpp` — port to Rust.
- `modules/skparagraph/src/TextWrapper.cpp` — port to Rust.
- `modules/skparagraph/src/TextWrapper.h` — port to Rust.
- `modules/skparagraph/src/TypefaceFontProvider.cpp` — port to Rust.
- `modules/skparagraph/utils/TestFontCollection.cpp` — port to Rust.
- `modules/skparagraph/utils/TestFontCollection.h` — port to Rust.
- `modules/skplaintexteditor/app/editor_application.cpp` — port to Rust.
- `modules/skplaintexteditor/include/editor.h` — port to Rust.
- `modules/skplaintexteditor/include/stringslice.h` — port to Rust.
- `modules/skplaintexteditor/include/stringview.h` — port to Rust.
- `modules/skplaintexteditor/src/editor.cpp` — port to Rust.
- `modules/skplaintexteditor/src/shape.cpp` — port to Rust.
- `modules/skplaintexteditor/src/shape.h` — port to Rust.
- `modules/skplaintexteditor/src/stringslice.cpp` — port to Rust.
- `modules/skplaintexteditor/src/word_boundaries.cpp` — port to Rust.
- `modules/skplaintexteditor/src/word_boundaries.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/skparagraph/BUILD.bazel`.
- `modules/skparagraph/BUILD.gn`.
- `modules/skparagraph/bench/BUILD.bazel`.
- `modules/skparagraph/gm/BUILD.bazel`.
- `modules/skparagraph/include/BUILD.bazel`.
- `modules/skparagraph/skparagraph.gni`.
- `modules/skparagraph/slides/BUILD.bazel`.
- `modules/skparagraph/src/BUILD.bazel`.
- `modules/skparagraph/test.html`.
- `modules/skparagraph/tests/BUILD.bazel`.
- `modules/skparagraph/utils/BUILD.bazel`.
- `modules/skplaintexteditor/BUILD.gn`.

## Codex: acceptance-test contract

Translate paragraph tests and test JSON writers, then their module GMs. Validate bidi/ellipsis/soft-hyphen fixtures and exact original layout assertions. Do not substitute screenshots for numeric glyph/line/selection checks.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `modules/skparagraph/bench/ParagraphBench.cpp`.
- `modules/skparagraph/gm/ellipsis_bidi_gm.cpp`.
- `modules/skparagraph/gm/simple_gm.cpp`.
- `modules/skparagraph/gm/softhyphen_gm.cpp`.
- `modules/skparagraph/tests/SkParagraphTest.cpp`.
- `modules/skparagraph/tests/SkShaperJSONWriter.cpp`.
- `modules/skparagraph/tests/SkShaperJSONWriter.h`.
- `modules/skparagraph/tests/SkShaperJSONWriterTest.cpp`.

### Original test registrations assigned to this gate

- `SkParagraph_BaselineShiftWithHeightMultiplier` — `modules/skparagraph/tests/SkParagraphTest.cpp:1677`; CPU; `DEF_TEST`.
- `SkParagraph_BaselineShiftLineHeightWithHeightOverride` — `modules/skparagraph/tests/SkParagraphTest.cpp:1724`; CPU; `DEF_TEST`.
- `SkParagraph_EmptyMetricsHalfLeadingFromTextStyle` — `modules/skparagraph/tests/SkParagraphTest.cpp:2111`; CPU; `DEF_TEST`.
- `SkParagraph_TextStyleFontMetricsMatchRun` — `modules/skparagraph/tests/SkParagraphTest.cpp:2170`; CPU; `DEF_TEST`.
- `SkParagraph_UnderlineGapsFollowDrawnGlyphs` — `modules/skparagraph/tests/SkParagraphTest.cpp:2951`; CPU; `DEF_TEST`.
- `SkParagraph_UnderlineGapsWithoutIntersections` — `modules/skparagraph/tests/SkParagraphTest.cpp:2999`; CPU; `DEF_TEST`.
- `SkParagraph_DecorationsFollowBaselineShift` — `modules/skparagraph/tests/SkParagraphTest.cpp:3017`; CPU; `DEF_TEST`.
- `SkParagraph_GetRectsForRangeIncludeCombiningCharacter` — `modules/skparagraph/tests/SkParagraphTest.cpp:4028`; CPU; `DEF_TEST_DISABLED`.
- `SkParagraph_JSON1` — `modules/skparagraph/tests/SkParagraphTest.cpp:5971`; CPU; `DEF_TEST_DISABLED`.
- `SkParagraph_JSON2` — `modules/skparagraph/tests/SkParagraphTest.cpp:6010`; CPU; `DEF_TEST_DISABLED`.
- `SkParagraph_ParagraphStyleHeightIgnored` — `modules/skparagraph/tests/SkParagraphTest.cpp:6239`; CPU; `DEF_TEST`.
- `SkParagraph_EqualsByFontsUsesEffectiveHeight` — `modules/skparagraph/tests/SkParagraphTest.cpp:6271`; CPU; `DEF_TEST`.
- `SkParagraph_CacheKeyCoversShapingInputs` — `modules/skparagraph/tests/SkParagraphTest.cpp:6331`; CPU; `DEF_TEST`.
- `SkParagraph_CachedLayoutFollowsHalfLeading` — `modules/skparagraph/tests/SkParagraphTest.cpp:6389`; CPU; `DEF_TEST`.
- `SkParagraph_CacheRestoresFontResolution` — `modules/skparagraph/tests/SkParagraphTest.cpp:6423`; CPU; `DEF_TEST`.
- `SkParagraph_ShapingKeepsFontSettingsPerStyle` — `modules/skparagraph/tests/SkParagraphTest.cpp:6459`; CPU; `DEF_TEST`.
- `SkParagraph_PlaceholderHeightInf` — `modules/skparagraph/tests/SkParagraphTest.cpp:7091`; CPU; `DEF_TEST_DISABLED`.
- `SkParagraph_DiacriticWithMultipleStyles` — `modules/skparagraph/tests/SkParagraphTest.cpp:10450`; CPU; `DEF_TEST`.
- `SkParagraph_LigatureWithMultipleStyles` — `modules/skparagraph/tests/SkParagraphTest.cpp:10528`; CPU; `DEF_TEST`.
- `SkParagraph_InlineCapacityGrowthRelocation` — `modules/skparagraph/tests/SkParagraphTest.cpp:10885`; CPU; `DEF_TEST`.
- `SkParagraph_JustificationHitTest` — `modules/skparagraph/tests/SkParagraphTest.cpp:11041`; CPU; `DEF_TEST`.
- `SkParagraph_BidiControlGaps` — `modules/skparagraph/tests/SkParagraphTest.cpp:11070`; CPU; `DEF_TEST`.
- `SkParagraph_ZeroWidthClusterSplit` — `modules/skparagraph/tests/SkParagraphTest.cpp:11108`; CPU; `DEF_TEST`.
- `SkParagraph_BidiIsolateStyledBlockNoSpaceFont` — `modules/skparagraph/tests/SkParagraphTest.cpp:11208`; CPU; `DEF_TEST`.
- `SkParagraph_CrLfDoesNotRequestFallback` — `modules/skparagraph/tests/SkParagraphTest.cpp:11296`; CPU; `DEF_TEST`.
- `SkShaperTest_cluster` — `modules/skparagraph/tests/SkShaperJSONWriterTest.cpp:16`; CPU; `DEF_TEST`.
- `SkShaperTest_VisualizeCluster` — `modules/skparagraph/tests/SkShaperJSONWriterTest.cpp:73`; CPU; `DEF_TEST`.
- `SkShaperTest_basic` — `modules/skparagraph/tests/SkShaperJSONWriterTest.cpp:126`; CPU; `DEF_TEST`.

### Preserve these fixtures, metadata and references

- `modules/skplaintexteditor/README.md` — preserve.

## Dependency and integration notes

The initial scan found 9 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=21`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M21 ACCEPTED may the user hand Deepseek M22.
