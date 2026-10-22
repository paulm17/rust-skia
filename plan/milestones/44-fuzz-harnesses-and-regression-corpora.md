# Milestone 44: Fuzz harnesses and regression corpora

## Outcome and entry gate

Finish all macOS-eligible fuzz entrypoints against Rust APIs and preserve input consumption order. Connect sanitizer/fuzz tooling appropriate for Rust and macOS; import accessible corpora with provenance. Do not alter production behavior to suppress findings.

Begin implementation only after milestone 43 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Finish all macOS-eligible fuzz entrypoints against Rust APIs and preserve input consumption order. Connect sanitizer/fuzz tooling appropriate for Rust and macOS; import accessible corpora with provenance. Do not alter production behavior to suppress findings.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

No new production-source ownership is assigned here. Deepseek fixes integration defects in already owned Rust implementations and completes the outcome above; Codex owns the suite below.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate each original fuzz harness and resource regression, replay known seeds, run bounded campaigns with recorded duration/seed/tool versions, and save minimized failures. Missing external corpora remain explicit missing coverage. Fixes must rerun affected earlier milestones.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `fuzz/Fuzz.cpp`.
- `fuzz/Fuzz.h`.
- `fuzz/FuzzCanvas.cpp`.
- `fuzz/FuzzCanvasHelpers.cpp`.
- `fuzz/FuzzCanvasHelpers.h`.
- `fuzz/FuzzCommon.cpp`.
- `fuzz/FuzzCommon.h`.
- `fuzz/FuzzCreateDDL.cpp`.
- `fuzz/FuzzCubicRoots.cpp`.
- `fuzz/FuzzDDLThreading.cpp`.
- `fuzz/FuzzDrawFunctions.cpp`.
- `fuzz/FuzzEncoders.cpp`.
- `fuzz/FuzzGrStyledShape.cpp`.
- `fuzz/FuzzGradients.cpp`.
- `fuzz/FuzzMain.cpp`.
- `fuzz/FuzzParsePath.cpp`.
- `fuzz/FuzzPath.cpp`.
- `fuzz/FuzzPathMeasure.cpp`.
- `fuzz/FuzzPathop.cpp`.
- `fuzz/FuzzPolyUtils.cpp`.
- `fuzz/FuzzPrecompile.cpp`.
- `fuzz/FuzzQuadRoots.cpp`.
- `fuzz/FuzzRRect.cpp`.
- `fuzz/FuzzRegionOp.cpp`.
- `fuzz/FuzzSkParagraph.cpp`.
- `fuzz/FuzzTriangulation.cpp`.
- `fuzz/oss_fuzz/FuzzAPICreateDDL.cpp`.
- `fuzz/oss_fuzz/FuzzAPIImageFilter.cpp`.
- `fuzz/oss_fuzz/FuzzAPISVGCanvas.cpp`.
- `fuzz/oss_fuzz/FuzzAnimatedImage.cpp`.
- `fuzz/oss_fuzz/FuzzBMPRustDecoder.cpp`.
- `fuzz/oss_fuzz/FuzzCOLRv1.cpp`.
- `fuzz/oss_fuzz/FuzzColorspace.cpp`.
- `fuzz/oss_fuzz/FuzzCubicRoots.cpp`.
- `fuzz/oss_fuzz/FuzzDDLThreading.cpp`.
- `fuzz/oss_fuzz/FuzzDrawFunctions.cpp`.
- `fuzz/oss_fuzz/FuzzGrStyledShape.cpp`.
- `fuzz/oss_fuzz/FuzzGradients.cpp`.
- `fuzz/oss_fuzz/FuzzHdrAgtm.cpp`.
- `fuzz/oss_fuzz/FuzzImage.cpp`.
- `fuzz/oss_fuzz/FuzzImageFilterDeserialize.cpp`.
- `fuzz/oss_fuzz/FuzzIncrementalImage.cpp`.
- `fuzz/oss_fuzz/FuzzJPEGEncoder.cpp`.
- `fuzz/oss_fuzz/FuzzJPEGRustDecoder.cpp`.
- `fuzz/oss_fuzz/FuzzJSON.cpp`.
- `fuzz/oss_fuzz/FuzzMockGPUCanvas.cpp`.
- `fuzz/oss_fuzz/FuzzNullCanvas.cpp`.
- `fuzz/oss_fuzz/FuzzPNGEncoder.cpp`.
- `fuzz/oss_fuzz/FuzzPNGRustDecoder.cpp`.
- `fuzz/oss_fuzz/FuzzPNGRustEncoder.cpp`.
- `fuzz/oss_fuzz/FuzzParsePath.cpp`.
- `fuzz/oss_fuzz/FuzzPathDeserialize.cpp`.
- `fuzz/oss_fuzz/FuzzPathMeasure.cpp`.
- `fuzz/oss_fuzz/FuzzPathop.cpp`.
- `fuzz/oss_fuzz/FuzzPolyUtils.cpp`.
- `fuzz/oss_fuzz/FuzzPrecompile.cpp`.
- `fuzz/oss_fuzz/FuzzQuadRoots.cpp`.
- `fuzz/oss_fuzz/FuzzRasterN32Canvas.cpp`.
- `fuzz/oss_fuzz/FuzzRegionDeserialize.cpp`.
- `fuzz/oss_fuzz/FuzzRegionOp.cpp`.
- `fuzz/oss_fuzz/FuzzRegionSetPath.cpp`.
- `fuzz/oss_fuzz/FuzzSKP.cpp`.
- `fuzz/oss_fuzz/FuzzSKSL2GLSL.cpp`.
- `fuzz/oss_fuzz/FuzzSKSL2Metal.cpp`.
- `fuzz/oss_fuzz/FuzzSKSL2Pipeline.cpp`.
- `fuzz/oss_fuzz/FuzzSKSL2SPIRV.cpp`.
- `fuzz/oss_fuzz/FuzzSKSL2WGSL.cpp`.
- `fuzz/oss_fuzz/FuzzSVG.cpp`.
- `fuzz/oss_fuzz/FuzzSkDescriptorDeserialize.cpp`.
- `fuzz/oss_fuzz/FuzzSkMeshSpecification.cpp`.
- `fuzz/oss_fuzz/FuzzSkParagraph.cpp`.
- `fuzz/oss_fuzz/FuzzSkRuntimeBlender.cpp`.
- `fuzz/oss_fuzz/FuzzSkRuntimeColorFilter.cpp`.
- `fuzz/oss_fuzz/FuzzSkRuntimeEffect.cpp`.
- `fuzz/oss_fuzz/FuzzTextBlobDeserialize.cpp`.
- `fuzz/oss_fuzz/FuzzTriangulation.cpp`.
- `fuzz/oss_fuzz/FuzzWEBPEncoder.cpp`.

### Original test registrations assigned to this gate

This milestone uses runner-discovered cases, GMs, fuzz/benchmark/browser suites or supplemental tests. Capture each discovered case in the acceptance manifest; absence from the static registration scan is not an exclusion.

### Preserve these fixtures, metadata and references

- `fuzz/README.md` — fixture.
- `fuzz/coverage` — fixture.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=44`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M44 ACCEPTED may the user hand Deepseek M45.
