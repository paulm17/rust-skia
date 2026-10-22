# Milestone 26: Runtime effects, meshes and deferred shader integration

## Outcome and entry gate

Implement RuntimeEffect, runtime blenders/shaders/filters, known effects, mesh specifications and their CPU execution. Complete the runtime-backed branches intentionally staged in M13/M15 and integrate SkSL with rendering and serialization.

Begin implementation only after milestone 25 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement RuntimeEffect, runtime blenders/shaders/filters, known effects, mesh specifications and their CPU execution. Complete the runtime-backed branches intentionally staged in M13/M15 and integrate SkSL with rendering and serialization.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/core/SkMesh.h` — port to Rust.
- `include/effects/SkRuntimeEffect.h` — port to Rust.
- `src/core/SkKnownRuntimeEffects.cpp` — port to Rust.
- `src/core/SkKnownRuntimeEffects.h` — port to Rust.
- `src/core/SkMesh.cpp` — port to Rust.
- `src/core/SkMeshPriv.h` — port to Rust.
- `src/core/SkRuntimeBlender.cpp` — port to Rust.
- `src/core/SkRuntimeBlender.h` — port to Rust.
- `src/core/SkRuntimeEffect.cpp` — port to Rust.
- `src/core/SkRuntimeEffectPriv.h` — port to Rust.
- `src/effects/colorfilters/SkRuntimeColorFilter.cpp` — port to Rust.
- `src/effects/colorfilters/SkRuntimeColorFilter.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate runtime-effect, runtime-blend, mesh and shader-debug tests. Run all deferred CPU filter/shader registrations and relevant GMs, then rerun M13–M15 and M22. Verify uniforms, children, capability limits, invalid programs and serialization failures.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/MeshTest.cpp`.
- `tests/SkGlyphTest.cpp`.
- `tests/SkSLDebugTracePlayerTest.cpp`.
- `tests/SkSLDebugTraceTest.cpp`.

### Original test registrations assigned to this gate

- `MeshSpec_Valid` — `tests/MeshTest.cpp:167`; CPU; `DEF_TEST`.
- `MeshSpec_InvalidSignature` — `tests/MeshTest.cpp:180`; CPU; `DEF_TEST`.
- `MeshSpec_Float4Color` — `tests/MeshTest.cpp:256`; CPU; `DEF_TEST`.
- `MeshSpec_DisallowsChildEffectInVertex` — `tests/MeshTest.cpp:272`; CPU; `DEF_TEST`.
- `MeshSpec_AllowsChildEffectInFragment` — `tests/MeshTest.cpp:308`; CPU; `DEF_TEST`.
- `MeshSpec_FindChild` — `tests/MeshTest.cpp:330`; CPU; `DEF_TEST`.
- `Mesh_ChildEffectsMatchSpec` — `tests/MeshTest.cpp:353`; CPU; `DEF_TEST`.
- `MeshSpec_ValidUniforms` — `tests/MeshTest.cpp:435`; CPU; `DEF_TEST`.
- `MeshSpec_InvalidUniforms` — `tests/MeshTest.cpp:665`; CPU; `DEF_TEST`.
- `MeshSpec_MissingMain` — `tests/MeshTest.cpp:704`; CPU; `DEF_TEST`.
- `MeshSpec_ZeroAttributes` — `tests/MeshTest.cpp:748`; CPU; `DEF_TEST`.
- `MeshSpec_ZeroVaryings` — `tests/MeshTest.cpp:758`; CPU; `DEF_TEST`.
- `MeshSpec_InvalidStride` — `tests/MeshTest.cpp:768`; CPU; `DEF_TEST`.
- `MeshSpec_InvalidOffset` — `tests/MeshTest.cpp:800`; CPU; `DEF_TEST`.
- `MeshSpec_TooManyAttributes` — `tests/MeshTest.cpp:843`; CPU; `DEF_TEST`.
- `MeshSpec_TooManyVaryings` — `tests/MeshTest.cpp:858`; CPU; `DEF_TEST`.
- `MeshSpec_DuplicateAttributeNames` — `tests/MeshTest.cpp:873`; CPU; `DEF_TEST`.
- `MeshSpec_DuplicateVaryingNames` — `tests/MeshTest.cpp:886`; CPU; `DEF_TEST`.
- `MeshSpec_SneakyExtraAttribute` — `tests/MeshTest.cpp:901`; CPU; `DEF_TEST`.
- `MeshSpec_SneakyExtraVarying` — `tests/MeshTest.cpp:913`; CPU; `DEF_TEST`.
- `MeshSpec_AllowsFloat2PositionVarying` — `tests/MeshTest.cpp:925`; CPU; `DEF_TEST`.
- `MeshSpec_InvalidPositionType` — `tests/MeshTest.cpp:938`; CPU; `DEF_TEST`.
- `MeshSpec_EmptyAttributeName` — `tests/MeshTest.cpp:951`; CPU; `DEF_TEST`.
- `MeshSpec_EmptyVaryingName` — `tests/MeshTest.cpp:963`; CPU; `DEF_TEST`.
- `MeshSpecVaryingPassthrough` — `tests/MeshTest.cpp:975`; CPU; `DEF_TEST`.
- `MeshSpecUnusedVaryings` — `tests/MeshTest.cpp:1091`; CPU; `DEF_TEST`.
- `SkGlyphRectBasic` — `tests/SkGlyphTest.cpp:36`; CPU; `DEF_TEST`.
- `SkGlyph_SendMetrics` — `tests/SkGlyphTest.cpp:88`; CPU; `DEF_TEST`.
- `SkGlyph_SendWithImage` — `tests/SkGlyphTest.cpp:131`; CPU; `DEF_TEST`.
- `SkGlyph_SendWithPath` — `tests/SkGlyphTest.cpp:196`; CPU; `DEF_TEST`.
- `SkGlyph_SendWithDrawable` — `tests/SkGlyphTest.cpp:267`; CPU; `DEF_TEST`.
- `SkPictureBackedGlyphDrawable_Basic` — `tests/SkGlyphTest.cpp:332`; CPU; `DEF_TEST`.
- `SkPictureBackedGlyphDrawable_RejectsAnyShadersThatNeedSkSL` — `tests/SkGlyphTest.cpp:465`; CPU; `DEF_TEST`.
- `SkSLTracePlayerCanResetToNull` — `tests/SkSLDebugTracePlayerTest.cpp:128`; CPU; `DEF_TEST`.
- `SkSLTracePlayerHelloWorld` — `tests/SkSLDebugTracePlayerTest.cpp:141`; CPU; `DEF_TEST`.
- `SkSLTracePlayerReset` — `tests/SkSLDebugTracePlayerTest.cpp:182`; CPU; `DEF_TEST`.
- `SkSLTracePlayerFunctions` — `tests/SkSLDebugTracePlayerTest.cpp:218`; CPU; `DEF_TEST`.
- `SkSLTracePlayerVariables` — `tests/SkSLDebugTracePlayerTest.cpp:308`; CPU; `DEF_TEST`.
- `SkSLTracePlayerVariableGroups` — `tests/SkSLDebugTracePlayerTest.cpp:405`; CPU; `DEF_TEST`.
- `SkSLTracePlayerIfStatement` — `tests/SkSLDebugTracePlayerTest.cpp:469`; CPU; `DEF_TEST`.
- `SkSLTracePlayerForLoop` — `tests/SkSLDebugTracePlayerTest.cpp:540`; CPU; `DEF_TEST`.
- `SkSLTracePlayerStepOut` — `tests/SkSLDebugTracePlayerTest.cpp:616`; CPU; `DEF_TEST`.
- `SkSLTracePlayerVariableScope` — `tests/SkSLDebugTracePlayerTest.cpp:673`; CPU; `DEF_TEST`.
- `SkSLTracePlayerNestedBlocks` — `tests/SkSLDebugTracePlayerTest.cpp:766`; CPU; `DEF_TEST`.
- `SkSLTracePlayerSwitchStatement` — `tests/SkSLDebugTracePlayerTest.cpp:789`; CPU; `DEF_TEST`.
- `SkSLTracePlayerBreakpoint` — `tests/SkSLDebugTracePlayerTest.cpp:828`; CPU; `DEF_TEST`.
- `SkSLTracePlayerStepOverWithBreakpoint` — `tests/SkSLDebugTracePlayerTest.cpp:882`; CPU; `DEF_TEST`.
- `SkSLTracePlayerStepOutWithBreakpoint` — `tests/SkSLDebugTracePlayerTest.cpp:913`; CPU; `DEF_TEST`.
- `DebugTracePrivSetSource` — `tests/SkSLDebugTraceTest.cpp:24`; CPU; `DEF_TEST`.
- `DebugTracePrivSetSourceReplacesExistingText` — `tests/SkSLDebugTraceTest.cpp:38`; CPU; `DEF_TEST`.
- `DebugTracePrivWrite` — `tests/SkSLDebugTraceTest.cpp:48`; CPU; `DEF_TEST`.
- `DebugTracePrivRead` — `tests/SkSLDebugTraceTest.cpp:86`; CPU; `DEF_TEST`.
- `DebugTracePrivGetSlotComponentSuffix` — `tests/SkSLDebugTraceTest.cpp:144`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/F16DrawTest.cpp` — M38.
- `tests/ShaderImageFilterTest.cpp` — M32, M38.
- `tests/ShaderTest.cpp` — M35, M38.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLES2ConformanceTest.cpp` — M32, M38.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/graphite/RTEffectTest.cpp` — M38.
- `tests/graphite/precompile/CombinationBuilderTest.cpp` — M38.
- `tests/graphite/precompile/PaintParamsKeyTest.cpp` — M38.
- `tests/graphite/precompile/UserdefinedStableKeyTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 1 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=26`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M26 ACCEPTED may the user hand Deepseek M27.
