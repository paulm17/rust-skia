# Giving a milestone to Deepseek

Use one milestone at a time. The current starting point is M01: existing scalar/point code and tests have already passed locally, so Deepseek should reconcile the baseline handoff rather than replace it. M01 also establishes the expanded test-discovery and ownership contract; new acceptance work is still Codex-owned.

## Implementation prompt

Replace `NN` and the milestone filename before sending:

> Implement milestone MNN from `/Volumes/Data/Users/paul/development/src/github/rust-skia/plan/milestones/NN-title.md`. Read `plan/README.md`, `plan/WORKFLOW.md` and `plan/MACOS-SCOPE.md` first. Your scope is the Deepseek-owned implementation/build files and real prerequisites documented by the milestone. Use the pinned C++ checkout as the reference. This is an owned Rust Skia implementation for macOS only. Do not modify Codex's acceptance tests, original expected values or source reference. Preserve the source-to-Rust API mapping and record incomplete optional branches explicitly. When ready, write `plan/handoffs/MNN.md` using the handoff template and stop. I will ask Codex to generate and run the acceptance tests. Do not begin the next milestone until Codex accepts this one and I hand over the next milestone.

## Testing prompt

After the implementation handoff:

> Validate milestone MNN in `/Volumes/Data/Users/paul/development/src/github/rust-skia/plan`. Read Deepseek's handoff and the milestone's Codex test section. Translate the assigned original Skia tests faithfully, preserve their helpers/fixtures and reconcile the reference test discovery. Run them in debug and release, plus all previously accepted tests and the required backend variants. Your scope is tests and acceptance evidence; leave implementation fixes to Deepseek. Write `plan/acceptance/MNN.md`, update the milestone status from actual results, and report exact failures for Deepseek if any. Do not accept the milestone merely because it compiles or because a subset passes.

## Fix prompt

> Fix the production failures in Codex's `plan/acceptance/MNN.md`. Do not change the assertions, tolerances, fixtures, exclusions or gate status. Update the implementation handoff with the fix revision and reproduction commands, then stop for Codex to rerun acceptance.

## Plan consistency check

Run from the Rust repository:

```sh
python3 plan/validate_plan.py \
  --source /Volumes/Data/Users/paul/development/src/github/skia \
  --check-hashes
```

This validates inventory coverage, source hashes, source locations, milestone numbering and document links. It does not run the Rust library or establish that a milestone passed.

## Changes discovered during implementation

The source inventory is comprehensive, but conditional compilation and the true API call graph require milestone-time verification. Do not silently reinterpret a file list. If a prerequisite or an original test needs a different gate, document the exact dependency and amend the affected milestone and inventory rows together before accepting it. Preserve a record of the change and all test obligations.
