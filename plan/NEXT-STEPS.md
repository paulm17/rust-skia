# Milestone next steps

Target Apple Silicon macOS only. Skip Intel/x86_64 implementation, builds and validation completely; do not report them as pending or blocked.

Use these three prompts in order. Replace `MNN` and `NN-title.md` with the milestone being worked on.

## 1. Deepseek — implement the code

> Implement milestone MNN from `/Volumes/Data/Users/paul/development/src/github/rust-skia/plan/milestones/NN-title.md`. Read `plan/README.md`, `plan/WORKFLOW.md` and `plan/MACOS-SCOPE.md`. Use the pinned Skia checkout as the reference. Implement the production code and build prerequisites, preserve the source-to-Rust API mapping, and document incomplete branches and adaptations in `plan/handoffs/MNN.md`. Do not change existing Codex tests, assertions, expected values, tolerances or fixtures. Stop after the handoff. Codex will write the tests next; do not start the next milestone.

## 2. Codex — build the tests ONLY

**Send this prompt to Codex after Deepseek's implementation handoff:**

> Build the acceptance tests for milestone MNN in `/Volumes/Data/Users/paul/development/src/github/rust-skia/plan`. Read Deepseek's `plan/handoffs/MNN.md` and the milestone's test requirements. Translate every assigned original Skia test faithfully, preserving its complete body, reached helpers, fixtures, inputs, assertions, tolerances and conditional behavior. Include the required supplemental boundary, overflow, byte-order, ownership and lifetime checks in this initial test-writing pass. Write only tests and the minimum test support needed. Do the absolute minimum work necessary for that scope. Do not run tests, compile checks, C++ references, discovery, formatting, Clippy, validators, audits or acceptance workflows. Do not change production code, acceptance reports, inventories or milestone status. Finish by listing the test files and exact commands Deepseek should run. Deepseek owns all execution, auditing and acceptance updates. Stop when the tests are written.

## 3. Deepseek — run tests, audit and update acceptance

**Send this prompt to Deepseek after Codex has written the tests:**

> Run the supplied tests for milestone MNN in debug and release, including all previously accepted tests and required backend variants. Audit coverage against every assigned original registration, reached helper and fixture. Reconcile conditional exclusions and actual reference discovery against the pinned Skia checkout. Build and run the required C++ references; reuse valid unchanged evidence where appropriate. Run the required formatting, Clippy and source-hash plan checks. Save exact commands, full logs, counts and failures under `plan/acceptance/MNN-evidence/`. Fix production defects without changing or weakening Codex's tests, assertions, expected values, tolerances, fixtures or exclusions. Rerun affected checks after fixes. If a test-source defect prevents progress, document its exact location and diagnostics for correction. Write `plan/acceptance/MNN.md` and update inventory results and `plan/status.json` from actual evidence. Accept only when every applicable requirement passes; otherwise record NEEDS_FIX or BLOCKED and explain the exact remaining requirements. Preserve previously accepted milestones and leave the next milestone unopened. You own test execution, the audit and acceptance updates; do not send those steps back to Codex.
