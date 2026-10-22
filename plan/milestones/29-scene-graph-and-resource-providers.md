# Milestone 29: Scene graph and resource providers

## Outcome and entry gate

Implement sksg retained nodes, invalidation, transforms, effects, rendering and skresources providers/cache lifetimes. Preserve observer behavior and damage propagation needed by animation.

Begin implementation only after milestone 28 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement sksg retained nodes, invalidation, transforms, effects, rendering and skresources providers/cache lifetimes. Preserve observer behavior and damage propagation needed by animation.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `modules/skresources/include/SkResources.h` — port to Rust.
- `modules/skresources/src/SkAnimCodecPlayer.cpp` — port to Rust.
- `modules/skresources/src/SkAnimCodecPlayer.h` — port to Rust.
- `modules/skresources/src/SkResources.cpp` — port to Rust.
- `modules/sksg/include/SkSGClipEffect.h` — port to Rust.
- `modules/sksg/include/SkSGColorFilter.h` — port to Rust.
- `modules/sksg/include/SkSGDraw.h` — port to Rust.
- `modules/sksg/include/SkSGEffectNode.h` — port to Rust.
- `modules/sksg/include/SkSGGeometryEffect.h` — port to Rust.
- `modules/sksg/include/SkSGGeometryNode.h` — port to Rust.
- `modules/sksg/include/SkSGGradient.h` — port to Rust.
- `modules/sksg/include/SkSGGroup.h` — port to Rust.
- `modules/sksg/include/SkSGImage.h` — port to Rust.
- `modules/sksg/include/SkSGInvalidationController.h` — port to Rust.
- `modules/sksg/include/SkSGMaskEffect.h` — port to Rust.
- `modules/sksg/include/SkSGMerge.h` — port to Rust.
- `modules/sksg/include/SkSGNode.h` — port to Rust.
- `modules/sksg/include/SkSGOpacityEffect.h` — port to Rust.
- `modules/sksg/include/SkSGPaint.h` — port to Rust.
- `modules/sksg/include/SkSGPath.h` — port to Rust.
- `modules/sksg/include/SkSGPlane.h` — port to Rust.
- `modules/sksg/include/SkSGRect.h` — port to Rust.
- `modules/sksg/include/SkSGRenderEffect.h` — port to Rust.
- `modules/sksg/include/SkSGRenderNode.h` — port to Rust.
- `modules/sksg/include/SkSGScene.h` — port to Rust.
- `modules/sksg/include/SkSGText.h` — port to Rust.
- `modules/sksg/include/SkSGTransform.h` — port to Rust.
- `modules/sksg/slides/SVGPongSlide.cpp` — port to Rust.
- `modules/sksg/src/SkSGClipEffect.cpp` — port to Rust.
- `modules/sksg/src/SkSGColorFilter.cpp` — port to Rust.
- `modules/sksg/src/SkSGDraw.cpp` — port to Rust.
- `modules/sksg/src/SkSGEffectNode.cpp` — port to Rust.
- `modules/sksg/src/SkSGGeometryEffect.cpp` — port to Rust.
- `modules/sksg/src/SkSGGeometryNode.cpp` — port to Rust.
- `modules/sksg/src/SkSGGradient.cpp` — port to Rust.
- `modules/sksg/src/SkSGGroup.cpp` — port to Rust.
- `modules/sksg/src/SkSGImage.cpp` — port to Rust.
- `modules/sksg/src/SkSGInvalidationController.cpp` — port to Rust.
- `modules/sksg/src/SkSGMaskEffect.cpp` — port to Rust.
- `modules/sksg/src/SkSGMerge.cpp` — port to Rust.
- `modules/sksg/src/SkSGNode.cpp` — port to Rust.
- `modules/sksg/src/SkSGNodePriv.h` — port to Rust.
- `modules/sksg/src/SkSGOpacityEffect.cpp` — port to Rust.
- `modules/sksg/src/SkSGPaint.cpp` — port to Rust.
- `modules/sksg/src/SkSGPath.cpp` — port to Rust.
- `modules/sksg/src/SkSGPlane.cpp` — port to Rust.
- `modules/sksg/src/SkSGRect.cpp` — port to Rust.
- `modules/sksg/src/SkSGRenderEffect.cpp` — port to Rust.
- `modules/sksg/src/SkSGRenderNode.cpp` — port to Rust.
- `modules/sksg/src/SkSGScene.cpp` — port to Rust.
- `modules/sksg/src/SkSGText.cpp` — port to Rust.
- `modules/sksg/src/SkSGTransform.cpp` — port to Rust.
- `modules/sksg/src/SkSGTransformPriv.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/skresources/BUILD.bazel`.
- `modules/skresources/BUILD.gn`.
- `modules/skresources/include/BUILD.bazel`.
- `modules/skresources/skresources.gni`.
- `modules/skresources/src/BUILD.bazel`.
- `modules/sksg/BUILD.bazel`.
- `modules/sksg/BUILD.gn`.
- `modules/sksg/include/BUILD.bazel`.
- `modules/sksg/sksg.gni`.
- `modules/sksg/slides/BUILD.bazel`.
- `modules/sksg/src/BUILD.bazel`.
- `modules/sksg/tests/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate SGTest and scene/resource regressions. Confirm original node updates, invalidation and bounds assertions, then render representative graph outputs. Failed resource loads must preserve upstream behavior rather than silently producing success.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `modules/sksg/tests/SGTest.cpp`.
- `modules/svg/tests/Text.cpp`.
- `tests/CodecAnimTest.cpp`.

### Original test registrations assigned to this gate

- `SGInvalidation` — `modules/sksg/tests/SGTest.cpp:357`; CPU; `DEF_TEST`.
- `SGMerge` — `modules/sksg/tests/SGTest.cpp:478`; CPU; `DEF_TEST`.
- `Svg_Text_PosProvider` — `modules/svg/tests/Text.cpp:21`; CPU; `DEF_TEST`.
- `Codec_trunc` — `tests/CodecAnimTest.cpp:36`; CPU; `DEF_TEST`.
- `Codec_565` — `tests/CodecAnimTest.cpp:50`; CPU; `DEF_TEST`.
- `Codec_frames` — `tests/CodecAnimTest.cpp:99`; CPU; `DEF_TEST`.
- `AndroidCodec_animated` — `tests/CodecAnimTest.cpp:583`; CPU; `DEF_TEST`.
- `AndroidCodec_animated_gif` — `tests/CodecAnimTest.cpp:587`; CPU; `DEF_TEST`.
- `EncodedOriginToMatrixTest` — `tests/CodecAnimTest.cpp:591`; CPU; `DEF_TEST`.
- `AnimCodecPlayer` — `tests/CodecAnimTest.cpp:613`; CPU; `DEF_TEST`.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 3 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=29`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M29 ACCEPTED may the user hand Deepseek M30.
