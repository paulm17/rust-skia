# Milestone 20: Unicode and shaping

## Outcome and entry gate

Implement skunicode and skshaper APIs, bidi, line/word/grapheme boundaries, script segmentation and shaping adapters. Preserve UTF conversion and byte/codepoint/cluster index conventions. ICU, HarfBuzz and other engines remain explicit external dependencies, not a reason to omit adapters.

Begin implementation only after milestone 19 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement skunicode and skshaper APIs, bidi, line/word/grapheme boundaries, script segmentation and shaping adapters. Preserve UTF conversion and byte/codepoint/cluster index conventions. ICU, HarfBuzz and other engines remain explicit external dependencies, not a reason to omit adapters.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `modules/skshaper/include/SkShaper.h` — port to Rust.
- `modules/skshaper/include/SkShaper_coretext.h` — port to Rust.
- `modules/skshaper/include/SkShaper_factory.h` — port to Rust.
- `modules/skshaper/include/SkShaper_harfbuzz.h` — port to Rust.
- `modules/skshaper/include/SkShaper_skunicode.h` — port to Rust.
- `modules/skshaper/src/SkShaper.cpp` — port to Rust.
- `modules/skshaper/src/SkShaper_coretext.cpp` — port to Rust.
- `modules/skshaper/src/SkShaper_factory.cpp` — port to Rust.
- `modules/skshaper/src/SkShaper_harfbuzz.cpp` — port to Rust.
- `modules/skshaper/src/SkShaper_primitive.cpp` — port to Rust.
- `modules/skshaper/src/SkShaper_skunicode.cpp` — port to Rust.
- `modules/skshaper/utils/FactoryHelpers.h` — port to Rust.
- `modules/skunicode/include/SkUnicode.h` — port to Rust.
- `modules/skunicode/include/SkUnicode_apple.h` — port to Rust.
- `modules/skunicode/include/SkUnicode_bidi.h` — port to Rust.
- `modules/skunicode/include/SkUnicode_client.h` — port to Rust.
- `modules/skunicode/include/SkUnicode_icu.h` — port to Rust.
- `modules/skunicode/include/SkUnicode_icu4x.h` — port to Rust.
- `modules/skunicode/include/SkUnicode_libgrapheme.h` — port to Rust.
- `modules/skunicode/src/SkBidiFactory_icu_full.cpp` — port to Rust.
- `modules/skunicode/src/SkBidiFactory_icu_full.h` — port to Rust.
- `modules/skunicode/src/SkBidiFactory_icu_subset.cpp` — port to Rust.
- `modules/skunicode/src/SkBidiFactory_icu_subset.h` — port to Rust.
- `modules/skunicode/src/SkUnicode.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_apple.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_bidi.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_client.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_hardcoded.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_hardcoded.h` — port to Rust.
- `modules/skunicode/src/SkUnicode_icu.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_icu4x.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_icu_bidi.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_icu_bidi.h` — port to Rust.
- `modules/skunicode/src/SkUnicode_icu_builtin.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_icu_runtime.cpp` — port to Rust.
- `modules/skunicode/src/SkUnicode_icupriv.h` — port to Rust.
- `modules/skunicode/src/SkUnicode_libgrapheme.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/skshaper/BUILD.bazel`.
- `modules/skshaper/BUILD.gn`.
- `modules/skshaper/include/BUILD.bazel`.
- `modules/skshaper/skshaper.gni`.
- `modules/skshaper/src/BUILD.bazel`.
- `modules/skshaper/tests/BUILD.bazel`.
- `modules/skshaper/utils/BUILD.bazel`.
- `modules/skunicode/BUILD.bazel`.
- `modules/skunicode/BUILD.gn`.
- `modules/skunicode/include/BUILD.bazel`.
- `modules/skunicode/skunicode.gni`.
- `modules/skunicode/src/BUILD.bazel`.
- `modules/skunicode/tests/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate Unicode and shaping suites with the original engine variants available on macOS. Cover invalid encodings, RTL, combining marks, emoji clusters, fallback and empty runs. Preserve engine-specific conditional expectations and test-data versions.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `modules/skshaper/tests/ShaperTest.cpp`.
- `modules/skunicode/tests/SkUnicodeTest.cpp`.
- `tests/FontationsFtCompTest.cpp`.

### Original test registrations assigned to this gate

- `Shaper_cluster_empty` — `modules/skshaper/tests/ShaperTest.cpp:239`; CPU; `DEF_TEST`.
- `Shaper_LineBreaks` — `modules/skshaper/tests/ShaperTest.cpp:278`; CPU; `DEF_TEST`.
- `Shaper_ZeroGlyphRuns` — `modules/skshaper/tests/ShaperTest.cpp:480`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_Native` — `modules/skunicode/tests/SkUnicodeTest.cpp:117`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_GetUtf8Words` — `modules/skunicode/tests/SkUnicodeTest.cpp:168`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_GetSentences` — `modules/skunicode/tests/SkUnicodeTest.cpp:205`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_BreakIterator_Utf8_Hostile` — `modules/skunicode/tests/SkUnicodeTest.cpp:242`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_BreakIterator_Utf16_Hostile` — `modules/skunicode/tests/SkUnicodeTest.cpp:345`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_CharProperties` — `modules/skunicode/tests/SkUnicodeTest.cpp:560`; CPU; `DEF_TEST`.
- `SkUnicode_GetBidiRegionsLTR` — `modules/skunicode/tests/SkUnicodeTest.cpp:583`; CPU; `DEF_TEST_ICU_UNICODES`.
- `SkUnicode_GetBidiRegionsRTL` — `modules/skunicode/tests/SkUnicodeTest.cpp:600`; CPU; `DEF_TEST_ICU_UNICODES`.
- `SkUnicode_GetBidiRegionsMix1` — `modules/skunicode/tests/SkUnicodeTest.cpp:617`; CPU; `DEF_TEST_ICU_UNICODES`.
- `SkUnicode_GetBidiRegionsMix2` — `modules/skunicode/tests/SkUnicodeTest.cpp:652`; CPU; `DEF_TEST_ICU_UNICODES`.
- `SkUnicode_ToUpper` — `modules/skunicode/tests/SkUnicodeTest.cpp:679`; CPU; `DEF_TEST_ICU_UNICODES`.
- `SkUnicode_ComputeCodeUnitFlags` — `modules/skunicode/tests/SkUnicodeTest.cpp:691`; CPU; `DEF_TEST_ICU_UNICODES`.
- `SkUnicode_ReorderVisual` — `modules/skunicode/tests/SkUnicodeTest.cpp:726`; CPU; `DEF_TEST_UNICODES`.
- `SkUnicode_Apple_Emoji` — `modules/skunicode/tests/SkUnicodeTest.cpp:778`; CPU; `DEF_TEST`.
- `SkUnicode_Apple_Ideographic` — `modules/skunicode/tests/SkUnicodeTest.cpp:822`; CPU; `DEF_TEST`.
- `Fontations_NotoSans` — `tests/FontationsFtCompTest.cpp:188`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Deva` — `tests/FontationsFtCompTest.cpp:199`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_ar_Arab` — `tests/FontationsFtCompTest.cpp:203`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Beng` — `tests/FontationsFtCompTest.cpp:208`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Jpan` — `tests/FontationsFtCompTest.cpp:212`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Thai` — `tests/FontationsFtCompTest.cpp:216`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Hans` — `tests/FontationsFtCompTest.cpp:220`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Hant` — `tests/FontationsFtCompTest.cpp:224`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Kore` — `tests/FontationsFtCompTest.cpp:228`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Taml` — `tests/FontationsFtCompTest.cpp:232`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Newa` — `tests/FontationsFtCompTest.cpp:236`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Knda` — `tests/FontationsFtCompTest.cpp:240`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Tglg` — `tests/FontationsFtCompTest.cpp:244`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Telu` — `tests/FontationsFtCompTest.cpp:248`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Gujr` — `tests/FontationsFtCompTest.cpp:252`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Geor` — `tests/FontationsFtCompTest.cpp:256`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Mlym` — `tests/FontationsFtCompTest.cpp:260`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Khmr` — `tests/FontationsFtCompTest.cpp:264`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Sinh` — `tests/FontationsFtCompTest.cpp:268`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Mymr` — `tests/FontationsFtCompTest.cpp:272`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Java` — `tests/FontationsFtCompTest.cpp:276`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Mong` — `tests/FontationsFtCompTest.cpp:280`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Armn` — `tests/FontationsFtCompTest.cpp:284`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Elba` — `tests/FontationsFtCompTest.cpp:288`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Vith` — `tests/FontationsFtCompTest.cpp:292`; CPU; `DEF_TEST`.
- `Fontations_NotoSans_Guru` — `tests/FontationsFtCompTest.cpp:296`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/skottie/tests/Shaper.cpp` — M30.
- `modules/skottie/tests/Text.cpp` — M30.
- `modules/skparagraph/tests/SkParagraphTest.cpp` — M21.
- `modules/svg/tests/Text.cpp` — M29.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 3 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=20`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M20 ACCEPTED may the user hand Deepseek M21.
