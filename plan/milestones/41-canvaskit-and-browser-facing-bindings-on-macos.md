# Milestone 41: CanvasKit and browser-facing bindings on macOS

## Outcome and entry gate

Implement Rust-backed WebAssembly/JavaScript bindings for the CanvasKit surface in the inventory, preserving TypeScript declarations and JavaScript API behavior. Reuse the owned Rust engine rather than linking C++ Skia. Browser validation is hosted on macOS; this milestone introduces no Windows/Linux coverage.

Begin implementation only after milestone 40 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement Rust-backed WebAssembly/JavaScript bindings for the CanvasKit surface in the inventory, preserving TypeScript declarations and JavaScript API behavior. Reuse the owned Rust engine rather than linking C++ Skia. Browser validation is hosted on macOS; this milestone introduces no Windows/Linux coverage.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `modules/canvaskit/WasmCommon.h` — port to Rust.
- `modules/canvaskit/bidi_bindings.cpp` — port to Rust.
- `modules/canvaskit/bidi_bindings_gen.cpp` — port to Rust.
- `modules/canvaskit/bidi_c_api.cpp` — port to Rust.
- `modules/canvaskit/bidi_c_api.h` — port to Rust.
- `modules/canvaskit/canvaskit_bindings.cpp` — port to Rust.
- `modules/canvaskit/debugger_bindings.cpp` — port to Rust.
- `modules/canvaskit/gm_bindings.cpp` — port to Rust.
- `modules/canvaskit/paragraph_bindings.cpp` — port to Rust.
- `modules/canvaskit/paragraph_bindings_gen.cpp` — port to Rust.
- `modules/canvaskit/skottie_bindings.cpp` — port to Rust.
- `modules/canvaskit/viewer_bindings.cpp` — port to Rust.
- `modules/canvaskit/wasm_tools/SIMD/simd_float_capabilities.cpp` — port to Rust.
- `modules/canvaskit/wasm_tools/SIMD/simd_int_capabilities.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/canvaskit/BUILD.bazel`.
- `modules/canvaskit/BUILD.gn`.
- `modules/canvaskit/bidi.js`.
- `modules/canvaskit/canvaskit.gni`.
- `modules/canvaskit/catchExceptionNop.js`.
- `modules/canvaskit/color.js`.
- `modules/canvaskit/compile.sh`.
- `modules/canvaskit/compile_gm.sh`.
- `modules/canvaskit/compile_skbidi.sh`.
- `modules/canvaskit/cpu.js`.
- `modules/canvaskit/debug.js`.
- `modules/canvaskit/debugger.js`.
- `modules/canvaskit/external_test/typescript_browser/index.html`.
- `modules/canvaskit/external_test/typescript_browser/module_uses_ck.ts`.
- `modules/canvaskit/external_test/typescript_browser/package-lock.json`.
- `modules/canvaskit/external_test/typescript_browser/package.json`.
- `modules/canvaskit/external_test/typescript_browser_es6/index.html`.
- `modules/canvaskit/external_test/typescript_browser_es6/module_uses_ck.ts`.
- `modules/canvaskit/external_test/typescript_browser_es6/package-lock.json`.
- `modules/canvaskit/external_test/typescript_browser_es6/package.json`.
- `modules/canvaskit/externs.js`.
- `modules/canvaskit/font.js`.
- `modules/canvaskit/gm.js`.
- `modules/canvaskit/go/gold_test_env/BUILD.bazel`.
- `modules/canvaskit/go/gold_test_env/gold_test_env.go`.
- `modules/canvaskit/htmlcanvas/_namedcolors.js`.
- `modules/canvaskit/htmlcanvas/canvas2dcontext.js`.
- `modules/canvaskit/htmlcanvas/color.js`.
- `modules/canvaskit/htmlcanvas/font.js`.
- `modules/canvaskit/htmlcanvas/htmlcanvas.js`.
- `modules/canvaskit/htmlcanvas/htmlimage.js`.
- `modules/canvaskit/htmlcanvas/imagedata.js`.
- `modules/canvaskit/htmlcanvas/lineargradient.js`.
- `modules/canvaskit/htmlcanvas/path2d.js`.
- `modules/canvaskit/htmlcanvas/pattern.js`.
- `modules/canvaskit/htmlcanvas/postamble.js`.
- `modules/canvaskit/htmlcanvas/preamble.js`.
- `modules/canvaskit/htmlcanvas/radialgradient.js`.
- `modules/canvaskit/htmlcanvas/util.js`.
- `modules/canvaskit/interface.js`.
- `modules/canvaskit/karma.bazel.js`.
- `modules/canvaskit/karma.conf.js`.
- `modules/canvaskit/make_version.sh`.
- `modules/canvaskit/matrix.js`.
- `modules/canvaskit/memory.js`.
- `modules/canvaskit/npm_build/bidi.html`.
- `modules/canvaskit/npm_build/example.html`.
- `modules/canvaskit/npm_build/extra.html`.
- `modules/canvaskit/npm_build/multicanvas.html`.
- `modules/canvaskit/npm_build/node.example.js`.
- `modules/canvaskit/npm_build/package-lock.json`.
- `modules/canvaskit/npm_build/package.json`.
- `modules/canvaskit/npm_build/paragraphs.html`.
- `modules/canvaskit/npm_build/shaping.html`.
- `modules/canvaskit/npm_build/textapi_utils.js`.
- `modules/canvaskit/npm_build/types/canvaskit-wasm-tests.ts`.
- `modules/canvaskit/npm_build/types/index.d.ts`.
- `modules/canvaskit/package-lock.json`.
- `modules/canvaskit/package.json`.
- `modules/canvaskit/paragraph.js`.
- `modules/canvaskit/pathops.js`.
- `modules/canvaskit/postamble.js`.
- `modules/canvaskit/preamble.js`.
- `modules/canvaskit/release.js`.
- `modules/canvaskit/rt_shader.js`.
- `modules/canvaskit/skbidi_bidi.js`.
- `modules/canvaskit/skottie.js`.
- `modules/canvaskit/skp.js`.
- `modules/canvaskit/tests/assets/BUILD.bazel`.
- `modules/canvaskit/util.js`.
- `modules/canvaskit/wasm_tools/SIMD/build_simd_test.sh`.
- `modules/canvaskit/wasm_tools/SIMD/simd_test.sh`.
- `modules/canvaskit/wasm_tools/gms.html`.
- `modules/canvaskit/wasm_tools/viewer.html`.
- `modules/canvaskit/webgl.js`.
- `modules/canvaskit/webgpu.js`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Port/adapt existing CanvasKit JavaScript/browser tests and assets, retaining original assertions. Validate canvas, paths, fonts, paragraph, animation and runtime shaders in macOS browsers. Record browser/build versions and distinguish wasm-target tests from native macOS tests.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `modules/canvaskit/tests/bazel_test_reporter.js`.
- `modules/canvaskit/tests/bidi_test.js`.
- `modules/canvaskit/tests/canvas2d_test.js`.
- `modules/canvaskit/tests/canvas_test.js`.
- `modules/canvaskit/tests/core_test.js`.
- `modules/canvaskit/tests/font_test.js`.
- `modules/canvaskit/tests/init_with_gold_server.js`.
- `modules/canvaskit/tests/legacy_init.js`.
- `modules/canvaskit/tests/legacy_test_reporter.js`.
- `modules/canvaskit/tests/matrix_test.js`.
- `modules/canvaskit/tests/paragraph_test.js`.
- `modules/canvaskit/tests/path_test.js`.
- `modules/canvaskit/tests/rtshader_test.js`.
- `modules/canvaskit/tests/skbidi_test.js`.
- `modules/canvaskit/tests/skottie_test.js`.
- `modules/canvaskit/tests/util.js`.

### Original test registrations assigned to this gate

This milestone uses runner-discovered cases, GMs, fuzz/benchmark/browser suites or supplemental tests. Capture each discovered case in the acceptance manifest; absence from the static registration scan is not an exclusion.

### Preserve these fixtures, metadata and references

- `modules/canvaskit/.gitignore` — preserve.
- `modules/canvaskit/CHANGELOG.md` — preserve.
- `modules/canvaskit/Makefile` — preserve.
- `modules/canvaskit/README.md` — preserve.
- `modules/canvaskit/external_test/.gitignore` — preserve.
- `modules/canvaskit/external_test/Makefile` — preserve.
- `modules/canvaskit/external_test/typescript_browser/tsconfig.json` — preserve.
- `modules/canvaskit/external_test/typescript_browser_es6/tsconfig.json` — preserve.
- `modules/canvaskit/fonts/NotoMono-Regular.ttf` — preserve.
- `modules/canvaskit/fonts/README.md` — preserve.
- `modules/canvaskit/future_apis/ImageDecoder.md` — preserve.
- `modules/canvaskit/future_apis/WebGPU.md` — preserve.
- `modules/canvaskit/npm_build/.gitignore` — preserve.
- `modules/canvaskit/npm_build/CODE_OF_CONDUCT.md` — preserve.
- `modules/canvaskit/npm_build/CONTRIBUTING.md` — preserve.
- `modules/canvaskit/npm_build/LICENSE` — preserve.
- `modules/canvaskit/npm_build/README.md` — preserve.
- `modules/canvaskit/npm_build/types/README.md` — preserve.
- `modules/canvaskit/npm_build/types/tsconfig.json` — preserve.
- `modules/canvaskit/npm_build/types/tslint.json` — preserve.
- `modules/canvaskit/tests/assets/Bungee-Regular.ttf` — fixture.
- `modules/canvaskit/tests/assets/NotoColorEmoji.ttf` — fixture.
- `modules/canvaskit/tests/assets/NotoSerif-BoldItalic.ttf` — fixture.
- `modules/canvaskit/tests/assets/NotoSerif-Regular.ttf` — fixture.
- `modules/canvaskit/tests/assets/Roboto-Regular.otf` — fixture.
- `modules/canvaskit/tests/assets/Roboto-Regular.woff` — fixture.
- `modules/canvaskit/tests/assets/Roboto-Regular.woff2` — fixture.
- `modules/canvaskit/tests/assets/RobotoSlab-VariableFont_wght.ttf` — fixture.
- `modules/canvaskit/tests/assets/animated_gif.json` — fixture.
- `modules/canvaskit/tests/assets/audio_external.json` — fixture.
- `modules/canvaskit/tests/assets/brickwork-texture.jpg` — fixture.
- `modules/canvaskit/tests/assets/color_wheel.gif` — fixture.
- `modules/canvaskit/tests/assets/color_wheel.webp` — fixture.
- `modules/canvaskit/tests/assets/exif_rotated_heart.jpg` — fixture.
- `modules/canvaskit/tests/assets/flightAnim.gif` — fixture.
- `modules/canvaskit/tests/assets/flutter_106433.png` — fixture.
- `modules/canvaskit/tests/assets/mandrill_16.png` — fixture.
- `modules/canvaskit/tests/assets/mandrill_512.png` — fixture.
- `modules/canvaskit/tests/assets/mandrill_h1v1.jpg` — fixture.
- `modules/canvaskit/tests/assets/map-shield.json` — fixture.
- `modules/canvaskit/tests/assets/red_line.skp` — fixture.
- `modules/canvaskit/tests/assets/skottie_basic_slots.json` — fixture.
- `modules/canvaskit/tests/assets/skottie_inline_font.json` — fixture.
- `modules/canvaskit/tests/assets/test.ttc` — fixture.
- `modules/canvaskit/tests/assets/test_glyphs-glyf_colr_1.ttf` — fixture.
- `modules/canvaskit/tests/assets/text_edit.json` — fixture.
- `modules/canvaskit/wasm_tools/SIMD/.gitignore` — preserve.
- `modules/canvaskit/wasm_tools/SIMD/wasm_simd_types.txt` — preserve.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=41`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M41 ACCEPTED may the user hand Deepseek M42.
