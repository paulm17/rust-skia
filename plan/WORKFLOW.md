# Workflow, ownership and gate rules

## Roles

Deepseek owns Rust production code, build integration, public API mapping and fixes. Codex owns faithful test translations, fixture/test-helper adaptations, test discovery, reference comparison, results and acceptance. The user chooses when to hand the next milestone to Deepseek. Deepseek must never edit acceptance assertions, golden outputs, skip conditions or acceptance status to make its implementation pass.

The source file inventory assigns a primary port milestone, not permission to omit code in a large file. A source spanning optional features needs a symbol/feature-level mapping in the handoff. Any postponed branch must name its integration gate and stay outstanding until verified. No required CPU path may call C++ Skia or report fake success. Supplemental tests are welcome but are labeled separately and never replace original tests.

## State transitions

`NOT_STARTED → IMPLEMENTING → READY_FOR_TESTS → TESTING → ACCEPTED`.

Failures transition to `NEEDS_FIX`; missing dependencies, fixtures or required hardware transition to `BLOCKED`. After Deepseek fixes code, return to TESTING and rerun failed, affected and cumulative accepted tests. Neither NEEDS_FIX nor BLOCKED unlocks a later milestone. Deepseek supplies evidence but Codex writes acceptance. Regressions in previously accepted behavior reopen the current gate and must be fixed before advancement.

For every milestone create `plan/handoffs/MNN.md` and `plan/acceptance/MNN.md` using the templates. Record implementation revision or a reproducible working-tree hash, original source commit, OS, CPU architecture, Rust toolchain, Cargo features, dependency versions, test inventory, commands, exit codes and output-log paths. Update `plan/status.json` only from these records. Existing local M01 results are provisional baseline evidence until this handoff contract is reconciled.

## Faithful translation

Preserve test names, test inputs, seeds and RNG implementation, all nested combinations, ordering where observable, original expected results, numerical precision, tolerances, short-circuit expressions, platform guards, fixture bytes and helper behavior. Keep C++ nonfatal assertions nonfatal. Do not substitute a standard-library function for the implementation under test. Memory-layout tests preserve the same observable address/layout property; unsafe C++ constructs require a documented Rust adaptation with equivalent observations.

Keep a correspondence row per registration/variant with source/hash, original symbol, Rust test path/name, feature/context, status and adaptation notes. Original intentionally disabled tests remain recorded as disabled, not passed. Custom macros, test suites, browser cases, GMs and fuzz entrypoints require runtime/build discovery. Run the applicable extended test modes rather than quietly leaving them disabled.

Some files are intentionally split at registration boundaries. RectTest.cpp is the initial explicit example: arithmetic in M04, matrix containment in M05, path-NaN in M07 and rendering in M14. Test cases stay complete; do not cut out assertions from a single registration to move it earlier. For other mixed files, reconcile reachability at the milestone's test-discovery step and record an amendment if an entire registration needs a different gate. Headers alone do not prove a later runtime dependency.

## Dependencies and independent reference

Build the pinned C++ reference with only macOS-eligible features needed at the current gate. Record GN arguments, compiler/SDK, external DEPS revisions and test runner commands. The reference may run as a separate executable; the shipped Rust library must not link C++ Skia. Initially available files are not proof that all DEPS resources are downloaded. Missing reference build capability is a recorded blocker for comparisons that require it.

Shared resources are usable before their final M43 inventory gate. Shared test helpers are ported by Codex at first use, even if tools packaging is later. A missing production prerequisite is Deepseek's work. Implement its real required behavior and record the early ownership transfer; if it expands the milestone materially, obtain a revised milestone from the user/Codex rather than creating stubs. A syntactic forward include is a review prompt, not an instruction to port a whole later subsystem.

## Test gates

Always run current and cumulative accepted tests in debug and release. Run formatting and Clippy, but never treat those as behavior tests. Use actual backend hardware for GPU milestones and compare test/context counts with the reference. CPU, Metal, OpenGL, Graphite Metal and Graphite Dawn-Metal are separate result variants. Missing supported variants cannot be counted as passing. Original unsupported-feature guards may produce NOT_APPLICABLE with evidence.

Apple Silicon is the native incremental gate host. Intel-specific implementation is tracked from its owning milestone and receives its mandatory native execution gate in M46 before final acceptance. Cross-compilation is useful but cannot replace execution. A missing Intel Mac blocks M46/M47; it does not retroactively claim Intel passage at early gates.

## Complete means complete

M47 requires every applicable owned algorithm ported, every source row reconciled, every applicable original registration/variant translated and passing, all pending integration branches closed, all macOS optional modules/backends exercised, and no C++ Skia implementation dependency. Platform exclusions and external dependency boundaries remain published. This is macOS Skia coverage, not a claim about Windows/Linux/iOS/Android.
