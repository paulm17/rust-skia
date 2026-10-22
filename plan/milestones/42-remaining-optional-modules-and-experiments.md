# Milestone 42: Remaining optional modules and experiments

## Outcome and entry gate

Port Bentley–Ottmann, JSON reader, remaining optional modules and macOS-eligible experimental prototypes listed here. Preserve experimental status and each existing entrypoint; existing Rust logic is audited/adapted rather than mechanically retranslated.

Begin implementation only after milestone 41 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Port Bentley–Ottmann, JSON reader, remaining optional modules and macOS-eligible experimental prototypes listed here. Preserve experimental status and each existing entrypoint; existing Rust logic is audited/adapted rather than mechanically retranslated.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `experimental/ffmpeg/SkVideoDecoder.cpp` — port to Rust.
- `experimental/ffmpeg/SkVideoDecoder.h` — port to Rust.
- `experimental/ffmpeg/SkVideoEncoder.cpp` — port to Rust.
- `experimental/ffmpeg/SkVideoEncoder.h` — port to Rust.
- `experimental/filterfuzz/filterfuzz.cpp` — port to Rust.
- `experimental/lowp-basic/QMath.h` — port to Rust.
- `experimental/lowp-basic/bilerp-study.cpp` — port to Rust.
- `experimental/lowp-basic/lerp-study.cpp` — port to Rust.
- `experimental/lowp-basic/lowp_experiments.cpp` — port to Rust.
- `experimental/rust_cxx/hello-world.rs` — integrate existing Rust.
- `experimental/rust_cxx/hype-bridge.rs` — integrate existing Rust.
- `experimental/rust_cxx/main.cpp` — port to Rust.
- `experimental/rust_ico/decoder/SkIcoRustDecoder.cpp` — port to Rust.
- `experimental/rust_ico/decoder/SkIcoRustDecoder.h` — port to Rust.
- `experimental/rust_ico/decoder/impl/SkIcoRustCodec.cpp` — port to Rust.
- `experimental/rust_ico/decoder/impl/SkIcoRustCodec.h` — port to Rust.
- `experimental/rust_ico/ffi/FFI.rs` — integrate existing Rust.
- `experimental/rust_jpeg/decoder/SkJpegRustDecoder.cpp` — port to Rust.
- `experimental/rust_jpeg/decoder/SkJpegRustDecoder.h` — port to Rust.
- `experimental/rust_jpeg/decoder/impl/SkJpegRustCodec.cpp` — port to Rust.
- `experimental/rust_jpeg/decoder/impl/SkJpegRustCodec.h` — port to Rust.
- `experimental/rust_jpeg/encoder/SkJpegRustEncoder.cpp` — port to Rust.
- `experimental/rust_jpeg/encoder/SkJpegRustEncoder.h` — port to Rust.
- `experimental/rust_jpeg/ffi/FFI.rs` — integrate existing Rust.
- `experimental/rust_jpeg/ffi/jpeg_metadata.rs` — integrate existing Rust.
- `experimental/rust_jpeg/ffi/jpeg_multipicture.rs` — integrate existing Rust.
- `experimental/rust_jpeg/ffi/jpeg_segment_scan.rs` — integrate existing Rust.
- `experimental/rust_jpeg/ffi/tiff_ifd.rs` — integrate existing Rust.
- `experimental/tools/coreGraphicsPdf2png.cpp` — port to Rust.
- `experimental/tskit/bindings/bindings.h` — port to Rust.
- `experimental/tskit/bindings/core.cpp` — port to Rust.
- `experimental/tskit/bindings/extension.cpp` — port to Rust.
- `experimental/tskit/go/gen_types/testdata/bindings1.cpp` — port to Rust.
- `experimental/webgpu-bazel/src/bindings.cpp` — port to Rust.
- `modules/bentleyottmann/include/BentleyOttmann1.h` — port to Rust.
- `modules/bentleyottmann/include/BruteForceCrossings.h` — port to Rust.
- `modules/bentleyottmann/include/Contour.h` — port to Rust.
- `modules/bentleyottmann/include/EventQueue.h` — port to Rust.
- `modules/bentleyottmann/include/EventQueueInterface.h` — port to Rust.
- `modules/bentleyottmann/include/Int96.h` — port to Rust.
- `modules/bentleyottmann/include/Myers.h` — port to Rust.
- `modules/bentleyottmann/include/Point.h` — port to Rust.
- `modules/bentleyottmann/include/Segment.h` — port to Rust.
- `modules/bentleyottmann/include/SweepLine.h` — port to Rust.
- `modules/bentleyottmann/src/BentleyOttmann1.cpp` — port to Rust.
- `modules/bentleyottmann/src/BruteForceCrossings.cpp` — port to Rust.
- `modules/bentleyottmann/src/Contour.cpp` — port to Rust.
- `modules/bentleyottmann/src/EventQueue.cpp` — port to Rust.
- `modules/bentleyottmann/src/Int96.cpp` — port to Rust.
- `modules/bentleyottmann/src/Myers.cpp` — port to Rust.
- `modules/bentleyottmann/src/Point.cpp` — port to Rust.
- `modules/bentleyottmann/src/Segment.cpp` — port to Rust.
- `modules/bentleyottmann/src/SweepLine.cpp` — port to Rust.
- `modules/jsonreader/SkJSONReader.cpp` — port to Rust.
- `modules/jsonreader/SkJSONReader.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `experimental/BUILD.bazel`.
- `experimental/ffmpeg/BUILD.gn`.
- `experimental/rust_cxx/BUILD.bazel`.
- `experimental/rust_ico/decoder/BUILD.bazel`.
- `experimental/rust_ico/decoder/impl/BUILD.bazel`.
- `experimental/rust_ico/ffi/BUILD.bazel`.
- `experimental/rust_jpeg/decoder/BUILD.bazel`.
- `experimental/rust_jpeg/decoder/impl/BUILD.bazel`.
- `experimental/rust_jpeg/encoder/BUILD.bazel`.
- `experimental/rust_jpeg/ffi/BUILD.bazel`.
- `experimental/sktext/BUILD.gn`.
- `experimental/tools/mskp_parser.py`.
- `experimental/tools/pdf-comparison.py`.
- `experimental/tskit/.eslintrc.js`.
- `experimental/tskit/bindings/core.d.ts`.
- `experimental/tskit/bindings/embind.d.ts`.
- `experimental/tskit/bindings/extension.d.ts`.
- `experimental/tskit/build/externs.js`.
- `experimental/tskit/compile.sh`.
- `experimental/tskit/go/gen_types/gen_types.go`.
- `experimental/tskit/go/gen_types/testdata/expectedambientnamespace1.d.ts`.
- `experimental/tskit/interface/core.ts`.
- `experimental/tskit/interface/extension.ts`.
- `experimental/tskit/interface/load.ts`.
- `experimental/tskit/interface/memory.ts`.
- `experimental/tskit/interface/public_api.d.ts`.
- `experimental/tskit/npm_build/example.html`.
- `experimental/tskit/npm_build/types/index.d.ts`.
- `experimental/tskit/package-lock.json`.
- `experimental/tskit/package.json`.
- `experimental/webgpu-bazel/example/index.html`.
- `experimental/webgpu-bazel/src/BUILD.bazel`.
- `modules/bentleyottmann/BUILD.bazel`.
- `modules/bentleyottmann/BUILD.gn`.
- `modules/bentleyottmann/bentleyottmann.gni`.
- `modules/bentleyottmann/include/BUILD.bazel`.
- `modules/bentleyottmann/src/BUILD.bazel`.
- `modules/bentleyottmann/tests/BUILD.bazel`.
- `modules/jsonreader/BUILD.bazel`.
- `modules/jsonreader/BUILD.gn`.
- `modules/jsonreader/jsonreader.gni`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate module tests such as integer/segment/sweep-line algorithms and experiment-specific fixtures. Compare original outputs and error paths. Experiments without tests require explicitly labeled additional tests; lack of upstream tests does not make a port automatically accepted.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `experimental/tskit/go/gen_types/gen_types_test.go`.
- `modules/bentleyottmann/tests/BentleyOttmann1Test.cpp`.
- `modules/bentleyottmann/tests/BruteForceCrossingsTest.cpp`.
- `modules/bentleyottmann/tests/ContourTest.cpp`.
- `modules/bentleyottmann/tests/EventQueueTest.cpp`.
- `modules/bentleyottmann/tests/Int96Test.cpp`.
- `modules/bentleyottmann/tests/MyersTest.cpp`.
- `modules/bentleyottmann/tests/PointTest.cpp`.
- `modules/bentleyottmann/tests/SegmentTest.cpp`.
- `modules/bentleyottmann/tests/SweepLineTest.cpp`.
- `tests/JSONTest.cpp`.

### Original test registrations assigned to this gate

- `BO_bentley_ottmann_1_Basic` — `modules/bentleyottmann/tests/BentleyOttmann1Test.cpp:14`; CPU; `DEF_TEST`.
- `BO_bruteForceCrossingsBasic` — `modules/bentleyottmann/tests/BruteForceCrossingsTest.cpp:14`; CPU; `DEF_TEST`.
- `CFC_Contours_Basic` — `modules/bentleyottmann/tests/ContourTest.cpp:12`; CPU; `DEF_TEST`.
- `BO_EventQueueOrdering` — `modules/bentleyottmann/tests/EventQueueTest.cpp:28`; CPU; `DEF_TEST`.
- `BO_EventQueueBasic` — `modules/bentleyottmann/tests/EventQueueTest.cpp:51`; CPU; `DEF_TEST`.
- `BO_EventQueueHandlerInterface` — `modules/bentleyottmann/tests/EventQueueTest.cpp:191`; CPU; `DEF_TEST`.
- `BO_Int96Basic` — `modules/bentleyottmann/tests/Int96Test.cpp:12`; CPU; `DEF_TEST`.
- `BO_Int96Less` — `modules/bentleyottmann/tests/Int96Test.cpp:84`; CPU; `DEF_TEST`.
- `BO_Int96Add` — `modules/bentleyottmann/tests/Int96Test.cpp:106`; CPU; `DEF_TEST`.
- `BO_Int96Mult` — `modules/bentleyottmann/tests/Int96Test.cpp:131`; CPU; `DEF_TEST`.
- `MFC_order_segment_points` — `modules/bentleyottmann/tests/MyersTest.cpp:26`; CPU; `DEF_TEST`.
- `MFC_segment_ctor` — `modules/bentleyottmann/tests/MyersTest.cpp:47`; CPU; `DEF_TEST`.
- `MFC_slope_less_than` — `modules/bentleyottmann/tests/MyersTest.cpp:67`; CPU; `DEF_TEST`.
- `MFC_segment_less_than_upper_to_insert` — `modules/bentleyottmann/tests/MyersTest.cpp:93`; CPU; `DEF_TEST`.
- `MFC_less_than_at_y` — `modules/bentleyottmann/tests/MyersTest.cpp:109`; CPU; `DEF_TEST`.
- `MFC_has_inner_intersection` — `modules/bentleyottmann/tests/MyersTest.cpp:138`; CPU; `DEF_TEST`.
- `MFC_myers_brute_force_comparison` — `modules/bentleyottmann/tests/MyersTest.cpp:213`; CPU; `DEF_TEST`.
- `MFC_myers_brute_force_random_comparison` — `modules/bentleyottmann/tests/MyersTest.cpp:305`; CPU; `DEF_TEST`.
- `BO_PointBasic` — `modules/bentleyottmann/tests/PointTest.cpp:9`; CPU; `DEF_TEST`.
- `BO_SegmentBasic` — `modules/bentleyottmann/tests/SegmentTest.cpp:9`; CPU; `DEF_TEST`.
- `BO_no_intersection_bounding_box` — `modules/bentleyottmann/tests/SegmentTest.cpp:39`; CPU; `DEF_TEST`.
- `BO_intersectBasic` — `modules/bentleyottmann/tests/SegmentTest.cpp:90`; CPU; `DEF_TEST`.
- `BO_lessAtBasic` — `modules/bentleyottmann/tests/SegmentTest.cpp:181`; CPU; `DEF_TEST`.
- `BO_compareSlopesBasic` — `modules/bentleyottmann/tests/SegmentTest.cpp:222`; CPU; `DEF_TEST`.
- `BO_rounded_point_less_than_segment_in_x_lower` — `modules/bentleyottmann/tests/SegmentTest.cpp:274`; CPU; `DEF_TEST`.
- `BO_rounded_point_less_than_segment_in_x_upper` — `modules/bentleyottmann/tests/SegmentTest.cpp:309`; CPU; `DEF_TEST`.
- `BO_SweepLineSearch` — `modules/bentleyottmann/tests/SweepLineTest.cpp:38`; CPU; `DEF_TEST`.
- `BO_SweepLineInsert` — `modules/bentleyottmann/tests/SweepLineTest.cpp:96`; CPU; `DEF_TEST`.
- `JSON_Parse` — `tests/JSONTest.cpp:23`; CPU; `DEF_TEST`.
- `JSON_DOM_visit` — `tests/JSONTest.cpp:193`; CPU; `DEF_TEST`.
- `JSON_DOM_build` — `tests/JSONTest.cpp:353`; CPU; `DEF_TEST`.
- `JSON_ParseNumber` — `tests/JSONTest.cpp:438`; CPU; `DEF_SERIAL_TEST`.
- `JSON_Lookup` — `tests/JSONTest.cpp:500`; CPU; `DEF_TEST`.
- `JSON_Writable` — `tests/JSONTest.cpp:515`; CPU; `DEF_TEST`.

### Preserve these fixtures, metadata and references

- `experimental/documentation/gerrit.md` — preserve.
- `experimental/lowp-basic/.gitignore` — preserve.
- `experimental/lowp-basic/CMakeLists.txt` — preserve.
- `experimental/tools/generate-unicode-test-txt` — preserve.
- `experimental/tools/gerrit_percent_encode` — preserve.
- `experimental/tools/set-change-id-hook` — preserve.
- `experimental/tools/web_to_mskp` — preserve.
- `experimental/tools/web_to_skp` — preserve.
- `experimental/tskit/.eslintignore` — preserve.
- `experimental/tskit/.gitignore` — preserve.
- `experimental/tskit/Makefile` — preserve.
- `experimental/tskit/tsconfig.json` — preserve.
- `experimental/webgpu-bazel/.gitignore` — preserve.
- `experimental/webgpu-bazel/Makefile` — preserve.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=42`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M42 ACCEPTED may the user hand Deepseek M43.
