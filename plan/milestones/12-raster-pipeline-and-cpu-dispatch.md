# Milestone 12: Raster pipeline and CPU dispatch

## Outcome and entry gate

Implement raster-pipeline stages, contexts, stage ordering and portable CPU dispatch. Port architecture intrinsics for Apple Silicon only, retaining a scalar fallback and exact dispatch selection. Preserve lane masks, precision and blend behavior.

Begin implementation only after milestone 11 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement raster-pipeline stages, contexts, stage ordering and portable CPU dispatch. Port architecture intrinsics for Apple Silicon only, retaining a scalar fallback and exact dispatch selection. Preserve lane masks, precision and blend behavior.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `src/core/SkCpu.cpp` — port to Rust.
- `src/core/SkCpu.h` — port to Rust.
- `src/core/SkMemset.h` — port to Rust.
- `src/core/SkMemset_opts.cpp` — port to Rust.
- `src/core/SkMemset_opts_avx.cpp` — port to Rust.
- `src/core/SkMemset_opts_erms.cpp` — port to Rust.
- `src/core/SkOpts.cpp` — port to Rust.
- `src/core/SkOpts.h` — port to Rust.
- `src/core/SkOptsTargets.h` — port to Rust.
- `src/core/SkRasterPipeline.cpp` — port to Rust.
- `src/core/SkRasterPipeline.h` — port to Rust.
- `src/core/SkRasterPipelineBlitter.cpp` — port to Rust.
- `src/core/SkRasterPipelineContextUtils.h` — port to Rust.
- `src/core/SkRasterPipelineOpContexts.h` — port to Rust.
- `src/core/SkRasterPipelineOpList.h` — port to Rust.
- `src/core/SkRasterPipelineVizualizer.h` — port to Rust.
- `src/opts/SkBitmapProcState_opts.h` — port to Rust.
- `src/opts/SkBlitMask_opts.h` — port to Rust.
- `src/opts/SkBlitRow_opts.h` — port to Rust.
- `src/opts/SkMemset_opts.h` — port to Rust.
- `src/opts/SkOpts_RestoreTarget.h` — port to Rust.
- `src/opts/SkOpts_SetTarget.h` — port to Rust.
- `src/opts/SkOpts_ml3.cpp` — port to Rust.
- `src/opts/SkOpts_ml4.cpp` — port to Rust.
- `src/opts/SkRasterPipeline_opts.h` — port to Rust.
- `src/opts/SkSwizzler_opts.inc` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `src/opts/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate raster-pipeline, F16-stage, parametric-stage, CPU-optimization and SIMD tests. Compare reference stage outputs for aligned and unaligned buffers, tails, NaNs and saturation. Release results must match the original precision contract; fast-math is not an acceptable workaround.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/BitmapCopyTest.cpp`.
- `tests/F16StagesTest.cpp`.
- `tests/MatrixProcsTest.cpp`.
- `tests/MemsetTest.cpp`.
- `tests/ParametricStageTest.cpp`.
- `tests/SRGBTest.cpp`.
- `tests/SkRasterPipelineOptsTest.cpp`.
- `tests/SkRasterPipelineTest.cpp`.

### Original test registrations assigned to this gate

- `BitmapCopy_extractSubset` — `tests/BitmapCopyTest.cpp:89`; CPU; `DEF_TEST`.
- `BitmapReadPixels` — `tests/BitmapCopyTest.cpp:164`; CPU; `DEF_TEST`.
- `F16Stages` — `tests/F16StagesTest.cpp:15`; CPU; `DEF_TEST`.
- `MatrixProcs_pack_clamp` — `tests/MatrixProcsTest.cpp:31`; CPU; `DEF_TEST`.
- `MatrixProcs_pack_clamp_out_of_range` — `tests/MatrixProcsTest.cpp:83`; CPU; `DEF_TEST`.
- `MatrixProcs_pack_repeat` — `tests/MatrixProcsTest.cpp:125`; CPU; `DEF_TEST`.
- `MatrixProcs_pack_mirror` — `tests/MatrixProcsTest.cpp:185`; CPU; `DEF_TEST`.
- `MatrixProcs_unpack_int` — `tests/MatrixProcsTest.cpp:246`; CPU; `DEF_TEST`.
- `Memset` — `tests/MemsetTest.cpp:88`; CPU; `DEF_TEST`.
- `Parametric_sRGB` — `tests/ParametricStageTest.cpp:54`; CPU; `DEF_TEST`.
- `Parametric_1dot0` — `tests/ParametricStageTest.cpp:68`; CPU; `DEF_TEST`.
- `Parametric_1dot2` — `tests/ParametricStageTest.cpp:70`; CPU; `DEF_TEST`.
- `Parametric_1dot4` — `tests/ParametricStageTest.cpp:71`; CPU; `DEF_TEST`.
- `Parametric_1dot8` — `tests/ParametricStageTest.cpp:72`; CPU; `DEF_TEST`.
- `Parametric_2dot0` — `tests/ParametricStageTest.cpp:73`; CPU; `DEF_TEST`.
- `Parametric_2dot2` — `tests/ParametricStageTest.cpp:74`; CPU; `DEF_TEST`.
- `Parametric_2dot4` — `tests/ParametricStageTest.cpp:75`; CPU; `DEF_TEST`.
- `Parametric_inv_1dot2` — `tests/ParametricStageTest.cpp:77`; CPU; `DEF_TEST`.
- `Parametric_inv_1dot4` — `tests/ParametricStageTest.cpp:78`; CPU; `DEF_TEST`.
- `Parametric_inv_1dot8` — `tests/ParametricStageTest.cpp:79`; CPU; `DEF_TEST`.
- `Parametric_inv_2dot0` — `tests/ParametricStageTest.cpp:80`; CPU; `DEF_TEST`.
- `Parametric_inv_2dot2` — `tests/ParametricStageTest.cpp:81`; CPU; `DEF_TEST`.
- `Parametric_inv_2dot4` — `tests/ParametricStageTest.cpp:82`; CPU; `DEF_TEST`.
- `srgb_roundtrip` — `tests/SRGBTest.cpp:23`; CPU; `DEF_TEST`.
- `srgb_edge_cases` — `tests/SRGBTest.cpp:53`; CPU; `DEF_TEST`.
- `srgb_roundtrip_extended` — `tests/SRGBTest.cpp:85`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Any` — `tests/SkRasterPipelineOptsTest.cpp:50`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_All` — `tests/SkRasterPipelineOptsTest.cpp:64`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Sin` — `tests/SkRasterPipelineOptsTest.cpp:78`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Cos` — `tests/SkRasterPipelineOptsTest.cpp:90`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Tan` — `tests/SkRasterPipelineOptsTest.cpp:102`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Asin` — `tests/SkRasterPipelineOptsTest.cpp:121`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Acos` — `tests/SkRasterPipelineOptsTest.cpp:132`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Atan` — `tests/SkRasterPipelineOptsTest.cpp:143`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Atan2` — `tests/SkRasterPipelineOptsTest.cpp:154`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Log2` — `tests/SkRasterPipelineOptsTest.cpp:167`; CPU; `DEF_TEST`.
- `SkRasterPipelineOpts_Pow2` — `tests/SkRasterPipelineOptsTest.cpp:178`; CPU; `DEF_TEST`.
- `SkRasterPipeline` — `tests/SkRasterPipelineTest.cpp:25`; CPU; `DEF_TEST`.
- `SkRasterPipeline_PackSmallContext` — `tests/SkRasterPipelineTest.cpp:50`; CPU; `DEF_TEST`.
- `SkRasterPipeline_PackBigContext` — `tests/SkRasterPipelineTest.cpp:83`; CPU; `DEF_TEST`.
- `SkRasterPipeline_LoadStoreConditionMask` — `tests/SkRasterPipelineTest.cpp:109`; CPU; `DEF_TEST`.
- `SkRasterPipeline_LoadStoreLoopMask` — `tests/SkRasterPipelineTest.cpp:153`; CPU; `DEF_TEST`.
- `SkRasterPipeline_LoadStoreReturnMask` — `tests/SkRasterPipelineTest.cpp:197`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MergeConditionMask` — `tests/SkRasterPipelineTest.cpp:241`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MergeLoopMask` — `tests/SkRasterPipelineTest.cpp:271`; CPU; `DEF_TEST`.
- `SkRasterPipeline_ReenableLoopMask` — `tests/SkRasterPipelineTest.cpp:307`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CaseOp` — `tests/SkRasterPipelineTest.cpp:343`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MaskOffLoopMask` — `tests/SkRasterPipelineTest.cpp:404`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MaskOffReturnMask` — `tests/SkRasterPipelineTest.cpp:437`; CPU; `DEF_TEST`.
- `SkRasterPipeline_InitLaneMasks` — `tests/SkRasterPipelineTest.cpp:470`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopyFromIndirectUnmasked` — `tests/SkRasterPipelineTest.cpp:526`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopyFromIndirectUniformUnmasked` — `tests/SkRasterPipelineTest.cpp:598`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopyToIndirectMasked` — `tests/SkRasterPipelineTest.cpp:670`; CPU; `DEF_TEST`.
- `SkRasterPipeline_SwizzleCopyToIndirectMasked` — `tests/SkRasterPipelineTest.cpp:761`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopySlotsMasked` — `tests/SkRasterPipelineTest.cpp:1163`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopySlotsUnmasked` — `tests/SkRasterPipelineTest.cpp:1235`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopyUniforms` — `tests/SkRasterPipelineTest.cpp:1289`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CopyConstant` — `tests/SkRasterPipelineTest.cpp:1342`; CPU; `DEF_TEST`.
- `SkRasterPipeline_Swizzle` — `tests/SkRasterPipelineTest.cpp:1379`; CPU; `DEF_TEST`.
- `SkRasterPipeline_SwizzleCopy` — `tests/SkRasterPipelineTest.cpp:1428`; CPU; `DEF_TEST`.
- `SkRasterPipeline_Shuffle` — `tests/SkRasterPipelineTest.cpp:1487`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MatrixMultiply2x2` — `tests/SkRasterPipelineTest.cpp:1545`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MatrixMultiply3x3` — `tests/SkRasterPipelineTest.cpp:1590`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MatrixMultiply4x4` — `tests/SkRasterPipelineTest.cpp:1643`; CPU; `DEF_TEST`.
- `SkRasterPipeline_FloatArithmeticWithNSlots` — `tests/SkRasterPipelineTest.cpp:1699`; CPU; `DEF_TEST`.
- `SkRasterPipeline_FloatArithmeticWithHardcodedSlots` — `tests/SkRasterPipelineTest.cpp:1752`; CPU; `DEF_TEST`.
- `SkRasterPipeline_IntArithmeticWithNSlots` — `tests/SkRasterPipelineTest.cpp:1819`; CPU; `DEF_TEST`.
- `SkRasterPipeline_IntArithmeticWithHardcodedSlots` — `tests/SkRasterPipelineTest.cpp:1880`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CompareFloatsWithNSlots` — `tests/SkRasterPipelineTest.cpp:1975`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CompareFloatsWithHardcodedSlots` — `tests/SkRasterPipelineTest.cpp:2032`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CompareIntsWithNSlots` — `tests/SkRasterPipelineTest.cpp:2102`; CPU; `DEF_TEST`.
- `SkRasterPipeline_CompareIntsWithHardcodedSlots` — `tests/SkRasterPipelineTest.cpp:2165`; CPU; `DEF_TEST`.
- `SkRasterPipeline_UnaryIntOps` — `tests/SkRasterPipelineTest.cpp:2246`; CPU; `DEF_TEST`.
- `SkRasterPipeline_UnaryFloatOps` — `tests/SkRasterPipelineTest.cpp:2301`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MixTest` — `tests/SkRasterPipelineTest.cpp:2382`; CPU; `DEF_TEST`.
- `SkRasterPipeline_MixIntTest` — `tests/SkRasterPipelineTest.cpp:2449`; CPU; `DEF_TEST`.
- `SkRasterPipeline_Jump` — `tests/SkRasterPipelineTest.cpp:2518`; CPU; `DEF_TEST`.
- `SkRasterPipeline_ExchangeSrc` — `tests/SkRasterPipelineTest.cpp:2546`; CPU; `DEF_TEST`.
- `SkRasterPipeline_BranchIfAllLanesActive` — `tests/SkRasterPipelineTest.cpp:2576`; CPU; `DEF_TEST`.
- `SkRasterPipeline_BranchIfAnyLanesActive` — `tests/SkRasterPipelineTest.cpp:2658`; CPU; `DEF_TEST`.
- `SkRasterPipeline_BranchIfNoLanesActive` — `tests/SkRasterPipelineTest.cpp:2739`; CPU; `DEF_TEST`.
- `SkRasterPipeline_BranchIfActiveLanesEqual` — `tests/SkRasterPipelineTest.cpp:2820`; CPU; `DEF_TEST`.
- `SkRasterPipeline_empty` — `tests/SkRasterPipelineTest.cpp:2919`; CPU; `DEF_TEST`.
- `SkRasterPipeline_nonsense` — `tests/SkRasterPipelineTest.cpp:2925`; CPU; `DEF_TEST`.
- `SkRasterPipeline_JIT` — `tests/SkRasterPipelineTest.cpp:2933`; CPU; `DEF_TEST`.
- `SkRasterPipeline_tail` — `tests/SkRasterPipelineTest.cpp:2977`; CPU; `DEF_TEST`.
- `SkRasterPipeline_u16` — `tests/SkRasterPipelineTest.cpp:3166`; CPU; `DEF_TEST`.
- `SkRasterPipeline_lowp` — `tests/SkRasterPipelineTest.cpp:3294`; CPU; `DEF_TEST`.
- `SkRasterPipeline_lowp_clamp01` — `tests/SkRasterPipelineTest.cpp:3381`; CPU; `DEF_TEST`.
- `SkRasterPipeline_stack_rewind` — `tests/SkRasterPipelineTest.cpp:3458`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/ApplyGammaTest.cpp` — M38.
- `tests/CachedDecodingPixelRefTest.cpp` — M16.
- `tests/CopySurfaceTest.cpp` — M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/RasterPipelineBuilderTest.cpp` — M25.
- `tests/RasterPipelineCodeGeneratorTest.cpp` — M25.
- `tests/SkRasterPipelineTest.cpp` — M24, M31.
- `tests/SkSLDebugTracePlayerTest.cpp` — M26.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/SwizzlerTest.cpp` — M16.
- `tests/graphite/TextureFormatTest.cpp` — M38.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 14 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=12`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M12 ACCEPTED may the user hand Deepseek M13.
