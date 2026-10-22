# Fix plan: milestones 1–10

## What this plan requires

Work through milestones **1, 2, 3, 4, 5, 6, 7, 8, 9, 10, in that order**.

The goal is to finish missing work and make the Rust implementation match the pinned C++ version on Apple Silicon macOS. Passing the existing tests alone is not enough: the reports explicitly identify differences those tests do not cover.

**DO NOT FIX THE TESTS. LEAVE THEM ALONE. FIX THE IMPLEMENTATION.**

This document is a work plan, not a new acceptance report. No implementation fix or new test run is claimed here. All ten milestones are historically ACCEPTED, but that label does not close the gaps listed below.

## Rules that apply to every milestone

1. Do not edit, delete, rename, replace, reformat or disable existing tests. This includes tests inside production files, test helpers, runners, reference programs, fixtures, expected outputs, snapshots and golden files. Do not change assertions, seeds, loops, tolerances, feature guards or test discovery.
2. Do not change Cargo configuration, lint settings or build flags to hide a failure, exclude a test or select an easier implementation. Production build changes needed to implement actual functionality must preserve existing coverage.
3. Do not regenerate expected results from Rust. The pinned C++ implementation determines the required behavior.
4. Do not change C++ source or patch its behavior to agree with Rust. Build artifacts belong outside the reference source tree.
5. Keep the owned implementation in Rust. Do not link C++ Skia or another Rust Skia port as the implementation. Existing permitted dependencies and Apple framework calls retain the scope in `plan/MACOS-SCOPE.md`.
6. Do not dismiss a difference because the format is deprecated, the difference is small, an existing test passes, or Rust's result seems better. Reproduce and fix defined C++ behavior, including surprising behavior.
7. Do not invent behavior for C++ undefined operations. Identify the exact precondition or undefined operation and record the evidence. A safe Rust API may reject an operation C++ cannot validly perform; that is different from changing a defined result.
8. Rust names and ownership types may differ where required by the language. Results, errors, side effects, lifetime guarantees, serialization and observable memory behavior must still match. Record each adaptation and demonstrate equivalence.
9. No remaining item may be marked complete by moving it to a later milestone. If it needs a later-owned type or subsystem, implement the necessary real dependency before closing this milestone. Keep that dependency's original acceptance obligation as well.
10. Respect the existing platform boundary: Apple Silicon macOS. Intel, other operating systems and excluded runtime backends do not become required through this plan. See `plan/MACOS-SCOPE.md`.
11. An old handoff is not proof a defect still exists. Check later acceptance reports and current production code before changing it. Preserve fixes already made.
12. If an existing test appears wrong, leave it unchanged. Record the exact disagreement and C++ evidence. Mark the milestone BLOCKED if the disagreement prevents completion; do not silently bypass it or claim completion.

## The required FIX → TEST loop

Use this loop for **each** milestone. Do not move to the next milestone until the completion conditions below are met.

```text
Read the handoff, acceptance report and assigned C++ source.
List every missing feature, approximation, mismatch and unverified branch.
Record the starting test-file hashes and test discovery.
    ↓
FIX: change production code to match C++.
    ↓
TEST: run the unchanged milestone tests in debug and release.
      Compare the changed behavior directly against pinned C++.
      Run the full existing suite in debug and release.
      Check formatting, Clippy, plan hashes and test-file integrity.
    ↓
Any failure, mismatch, missing branch or unexplained result?
    YES → record it → FIX production → TEST again → repeat.
    NO  → verify all completion conditions → record COMPLETE.
    ↓
Proceed to the next milestone.
```

If the audit finds no production defect, do not make an unnecessary edit. Run TEST and supply the missing proof instead. If a dependency or environment prevents a required comparison, record BLOCKED rather than COMPLETE.

### Protect the tests before the first fix

- Save a SHA-256 manifest of all existing test inputs and infrastructure, including `tests/`, reference comparison tools, fixtures and expected data elsewhere in the repository, and embedded tests in `src/`.
- Whole production-file hashes will change when production code changes. For embedded tests, compare the actual test sections separately and inspect the diff to prove those sections were untouched.
- Save test-discovery output for every existing test target in both profiles. Match names and enabled cases after every round; explain genuine additions from newly implemented functionality without losing existing cases.
- Hash the pinned C++ files used by the comparison. Keep their source unchanged.
- Check the protected files after every FIX and TEST round. Accidental formatting changes count as test changes and must be restored to the saved baseline.

### How to check gaps without touching existing tests

Existing tests are immutable. Where they do not cover a documented gap, create **separate, additional verification programs** under `plan/fix-evidence/Mxx/`, outside the existing test tree. Do not replace or modify an existing runner to accommodate them.

Use the same fixed inputs for a small C++ program and a Rust program. Compare outputs, bytes, errors and side effects. Derive expectations from the pinned C++ source and execution, never from the Rust output. Freeze comparison inputs and rules before using them to judge a fix. Additional checks must not reduce or replace existing coverage.

### Commands for every final TEST round

Run from the repository root:

```sh
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
python3 plan/validate_plan.py --source /Volumes/Data/Users/paul/development/src/github/skia --check-hashes
```

Also run the existing milestone-specific and reference commands recorded in its acceptance report. For M02–M10, the existing Rust targets are `m02_test` through `m10_test`; run each target in debug and release. M01 uses the point, scalar, harness and supplemental targets named in its report.

Use `cargo fmt --all -- --check` for verification. Do not run a repository-wide formatting command that rewrites tests. Format only changed production code, preserving any embedded tests exactly.

### What COMPLETE means

- Every assigned source/symbol has been checked, including branches not reached by the existing tests.
- All missing and approximate behavior listed for this milestone is implemented or proven already resolved.
- Every applicable existing test passes unchanged in debug and release, including the full regression suite.
- Direct C++ comparisons cover the former gaps. Integer results, serialized data and pixel bytes match exactly. Floating-point comparisons follow the pinned source's actual contract; do not invent a looser tolerance to pass.
- Required observable allocation, alignment, ownership and error behavior matches. Renamed APIs or different private storage alone do not establish a defect.
- No unexplained numerical difference, unimplemented dependency, unavailable required comparison or postponed item remains.
- Protected tests, fixtures, expected results and C++ source are unchanged.
- A final evidence report records what changed, what ran and why each gap is closed.

## 1. Scalars and points

**Starting position:** No unresolved in-scope numerical defect is recorded. Verify it; do not rewrite working code.

Sources: [handoff](handoffs/M01.md), [acceptance](acceptance/M01.md).

### FIX — only if the audit or comparison finds a difference

- Compare scalar rounding, saturation, interpolation, floating-point bit conversions, point arithmetic, normalization and distance helpers with pinned C++.
- Check zero, negative zero, subnormal values, infinities, NaNs and large finite coordinates where the C++ operation is defined.
- Check the documented special input handling: empty or unequal interpolation slices, explicit counts, empty finite-value checks, integer minimum negation and numeric casts. Separate real defined-result differences from C++ preconditions or undefined behavior.
- Verify that `SetLengthFast` using the ordinary length path is equivalent for the supported inputs. Do not infer a defect merely from different function names.
- Confirm that prerequisites completed by M02–M04 remain covered; do not leave an obsolete M01 handoff note open after a later implementation resolved it.

### TEST → FIX → TEST

Run the unchanged M01 tests and existing C++ reference runner in both profiles. Add separate comparisons for any uncovered boundary above. If a defined result differs, fix production and repeat the common loop.

**Completion condition:** No unresolved scalar/point difference, and all existing tests remain unchanged and pass.

## 2. Ownership, containers, data and streams

**Starting position:** Tests pass, but the acceptance report explicitly retains approximations outside those passing observations. This is a substantial fix milestone.

Sources: [handoff](handoffs/M02.md), [acceptance](acceptance/M02.md).

### FIX

- Finish exact block-allocator and block-list behavior: scratch-block reuse, fit decisions, reserve, growth, shrink, ownership of blocks, release and reported memory usage. Matching selected accounting formulas is not proof the complete allocator behavior matches.
- Finish ResourceCache, synchronized-cache and CachedData behavior. Check lookup, replacement, eviction order, purge limits, locking, reference transitions and destruction. Replace simplified behavior wherever it changes an observable result.
- Match message-bus sharing between callers. An explicit bus instance must not prevent communication that the C++ per-type bus permits. Preserve message filtering, delivery and inbox lifetime behavior.
- Restore no-copy/ownership behavior where DataTable currently copies caller arrays. Verify identity, release callback timing, callback count and source lifetime requirements.
- Check file and descriptor ownership and implement the C++ mapping behavior where reading into a new buffer changes the contract. Cover mapping failure, empty files, file changes, cleanup and descriptor lifetime according to the actual upstream APIs.
- Check numeric intrusive-list handles against the original lifetime and identity guarantees. Fix observable differences; a different private representation alone does not require replacement.
- Audit the documented formatting subset and `prependScalar` behavior. Match every applicable defined formatting case and the pinned append/prepend behavior, including an upstream quirk if it is defined.
- Complete typed Writer32 serializers named in the handoff: points, rectangles, rounded rectangles, matrices, paths, regions and sampling information. Check whether later milestones already supplied any part before implementing it again.
- Confirm the previously postponed color parsing work is now supplied by M10.
- Finish tile-mode/blend-mode integration, discardable-memory creation/pooling, global registration and JSON serialization callbacks where they belong to these assigned source contracts. Implement necessary later-owned dependencies rather than leaving partial interfaces behind.
- Preserve the existing smart-pointer, projected-base ownership, thread-safety, executor, MD5, UTF and stream fixes. Do not regress them while changing ownership or allocation.

### TEST → FIX → TEST

Run all unchanged M02 originals, supplemental checks, positive/negative API compilation checks and the existing C++ reference commands in both profiles. Separately compare allocator operation sequences, eviction order, cross-caller messages, pointer identity, callback timing, mapped-file behavior and serializer bytes. Fix production after every discrepancy and repeat.

**Completion condition:** The documented approximations are closed with evidence, every applicable assigned API works, and all ownership/API checks pass unchanged.

## 3. Numeric primitives, vector operations and random numbers

**Starting position:** Scalar results pass. Vector alignment and architecture-specific implementation were deferred. Some handoff defects were already fixed during acceptance.

Sources: [handoff](handoffs/M03.md), [acceptance](acceptance/M03.md).

### FIX

- Implement the vector alignment guarantees required by the pinned Apple Silicon configuration, including values stored in arrays, containers and allocations. Matching size alone is insufficient.
- Finish the applicable Apple Silicon vector operations and dispatch promised by the original scope. Do not add excluded Intel kernels. Verify the selected path actually runs and preserves numerical behavior.
- Audit rounding, overflow, casts, masks, lane ordering, half-float conversion and NaN handling across scalar and vector paths.
- Verify the Apple Silicon build's actual YUV table branch. The portable low-precision table is sufficient only if it produces the reference behavior required by that configuration.
- Preserve acceptance fixes: `SkTo` debug/release behavior, truncated SafeMath cast results, scalar NaN clamping, stream duplicate/fork/peek semantics and the STArray fix. Do not reintroduce stale handoff behavior.
- Check whether omitted diagnostic functions are needed for full assigned API coverage. Implement applicable diagnostic behavior rather than silently treating it as completed production functionality.

### TEST → FIX → TEST

Run the unchanged 87 original registrations and supplemental suite in both profiles. Separately compare size/alignment, actual allocation alignment, numerical bit patterns and deterministic random sequences with the same C++ configuration. Exercise each implemented dispatch path. Fix production and repeat.

**Completion condition:** Numerical results and required memory layout match, and the applicable vector implementation is present and verified.

## 4. Rectangles, rounded rectangles, sizes and 3D points

**Starting position:** Assigned tests pass. M07 subsequently implemented the non-legacy RRect transform; do not treat it as still wholly missing.

Sources: [handoff](handoffs/M04.md), [acceptance](acceptance/M04.md), [M07 fixes](acceptance/M07.md).

### FIX

- Audit the current RRect transform against all applicable pinned overloads and branches. Preserve the transform implemented during M07 acceptance and finish any remaining transform contract.
- Implement the missing conservative rounded-rectangle intersection behavior.
- Audit all remaining assigned Rect/RRect methods, including operations whose original tests were scheduled for M14/M23. Port their real dependencies when necessary.
- Check overflow, empty and inverted rectangles, invalid radii, non-finite coordinates, transformations and serialized memory formats.
- Implement applicable omitted dump/string helpers if required to close the assigned source API. Keep platform-excluded branches explicitly out of scope.

### TEST → FIX → TEST

Run the unchanged M04 and M07 suites, plus any existing later-gate tests already available. Use separate C++ comparisons for transforms and conservative intersections, including touching, empty, disjoint and difficult curved-corner cases. Keep the existing fuzzer fixture bytes and radius tolerances untouched. Fix production and repeat.

**Completion condition:** No applicable Rect/RRect operation remains missing, and existing fixtures and assertions pass without edits.

## 5. Matrices, camera transforms and coordinate mapping

**Starting position:** Core mathematics passes. Several assigned functions or complete source files remain postponed.

Sources: [handoff](handoffs/M05.md), [acceptance](acceptance/M05.md).

### FIX

- Implement the local-matrix image-filter wrapper, including mapping, bounds, creation and serialization behavior. Supply the real filter and buffer dependencies needed to exercise it.
- Complete bitmap matrix-procedure selection and generator closures, using real bitmap, pixel and tile-mode state.
- Implement the missing map-points procedure selection and dispatch.
- Implement sprite classification with the actual sampling options.
- Implement applying camera/view transforms to Canvas using the real Canvas contract.
- Compare perspective `mapRect` with the pinned path-based implementation across clipping, near-zero perspective denominators, negative signs, non-finite values and degenerate rectangles. Keep the alternate algorithm only if equivalence is demonstrated; otherwise port the reference behavior.
- Audit strided mapping: the Rust element-stride interface must cover the defined byte-stride layouts supported by C++ or provide an appropriate production API for them.
- Preserve the acceptance fix for anisotropic-scale decomposition and all previously established matrix edge behavior.

### TEST → FIX → TEST

Run unchanged M05 tests in both profiles and the recorded reference commands. Add separate comparisons of mapping output, procedure selection, sprite decisions, filter bounds/serialization and Canvas transform effects. Fix production and repeat.

**Completion condition:** The missing functions and filter files are implemented, with no remaining integration deferral or unexplained mapping difference.

## 6. Bézier geometry, roots and clipping primitives

**Starting position:** Assigned tests pass. M07 supplied the deferred `ClipPath` dispatcher. Some direct C++ verification was limited by the old reference build.

Sources: [handoff](handoffs/M06.md), [acceptance](acceptance/M06.md), [M07 handoff](handoffs/M07.md).

### FIX

- Verify the current `ClipPath` dispatcher is complete for lines, quadratics, conics and cubics, including perspective-clipping consumers.
- Close the direct-reference coverage gap for `SkConic::BuildUnitArc` and perspective `TransformW`. Build a usable unchanged reference with the required initialization instead of retaining the old partial-link limitation.
- Investigate the recorded small floating-point differences. Establish whether they satisfy the pinned function's actual contract; fix evaluation order, intermediate precision or algorithm differences wherever exact behavior is required. Do not simply copy the previous comparison tolerance.
- Audit roots, splitting, extrema, tangents, clipping, polygon operations and degenerate inputs beyond the selected differential fixtures.
- Do not invent implementations for declarations with no upstream definition or upstream code explicitly disabled as unimplemented. Record the exact source disposition.

### TEST → FIX → TEST

Run unchanged M06 tests and the existing differential tools. Separately compare unit arcs, perspective weights and `ClipPath`, using a correctly initialized C++ reference. Fix any production mismatch and repeat.

**Completion condition:** Both former coverage gaps are directly verified and no applicable geometry mismatch remains unresolved.

## 7. Paths, builders, iteration and measurement

**Starting position:** Core path tests pass. Region-path conversion and non-AA filling were subsequently added in M09. Paint-aware path conversion and full scan conversion remain deferred.

Sources: [handoff](handoffs/M07.md), [acceptance](acceptance/M07.md), [M09 handoff](handoffs/M09.md).

### FIX

- Implement `SkPathUtils::FillPathWithPaint` and its real paint, stroke and path-effect dependencies.
- Audit the M09 implementation of region-path conversion and non-AA scan conversion against all applicable assigned branches; reuse the real implementation rather than creating a parallel substitute.
- Finish assigned anti-aliased and analytic scan-conversion work. Coordinate shared code with M09's exact-AA requirement; record the implementation here and verify its clip use again at milestone 9.
- Audit path ownership adaptations: raw views converted to owned data, builder optional results converted to total results, iterator copies and error-singleton handling. Restore any observable result, lifetime or failure distinction lost by the adaptation.
- Verify complete path serialization, malformed-input rejection, generation IDs, copy-on-write, trailing moves, perspective clipping and contour measurement.
- Preserve the seven production corrections recorded in M07 acceptance, including the RRect transform used by milestone 4.

### TEST → FIX → TEST

Run unchanged M07 tests and all regression suites. Separately compare paint-to-path output, scan-converted pixel coverage, serialization bytes and defined failure cases against C++. Fix production and repeat.

**Completion condition:** Every applicable assigned path and scan-conversion function is present; drawing dependencies are implemented and exercised rather than postponed.

## 8. Stroking and path effects

**Starting position:** The three original registrations establish dash-description behavior. They do not establish full stroke or effect geometry parity. Recorded coordinate differences remain.

Sources: [handoff](handoffs/M08.md), [acceptance](acceptance/M08.md).

### FIX

- Reproduce the recorded 1–4 ULP differences in curved strokes and dash/discrete coordinates. A ULP is one step between neighboring floating-point values; these are small but real differences.
- Match reference calculation order, intermediate precision, recursive subdivision, joins, caps and curve offsetting. Close the differences instead of raising tolerances.
- Audit the complete outputs of corner, dash, discrete, trim, 1D and 2D effects, including composed and summed effects. Compare verb sequences, point coordinates, weights, bounds and stroke-state changes.
- Implement Paint-based stroke constructors, `applyToPaint` and Paint-based inflation-radius calculation.
- Implement effect flattening, creation from buffers, type identification, registration and deserialization using real serialization infrastructure.
- Verify the paint-aware drawing entry points using path effects, reusing work completed at milestone 7.

### TEST → FIX → TEST

Run unchanged M08 tests and the existing dash reference comparison. Separately reproduce the broader geometry comparison with fixed seeds and inputs, including the previously differing cases. Compare effect serialization and reconstruction with C++. Fix production and repeat until the differences are closed.

**Completion condition:** Full effect and stroke behavior is verified, recorded coordinate differences are resolved, and Paint/serialization work is complete.

## 9. Regions, clip storage and spatial indexes

**Starting position:** ClipStack and RTree pass. AA path coverage is explicitly approximate and produces different mask bytes.

Sources: [handoff](handoffs/M09.md), [acceptance](acceptance/M09.md).

### FIX

- Replace 4×4 supersampling with the pinned analytic AA coverage algorithm. Reuse exact scan-conversion work from milestone 7 and finish analytic edges and builder branches as needed.
- Reproduce the recorded circle/combined/difference mismatches: C++ sums `1999853`, `1999853`, `1591853`; old Rust sums `2002004`, `2002004`, `1594004`. These sums identify regressions; equal sums alone do not prove equal pixels.
- Implement AA clip blitters, wrappers and `copyToMask` with real mask and blitter types.
- Match large-coordinate and overflow behavior, including the reference's AA-disable thresholds and fallback decisions wherever they affect output. A wider Rust run representation is not proof the policy is unobservable.
- Implement clip-shader materialization and composition using real shaders.
- Finish clip-stack-to-path conversion beyond the first/replace element, using real boolean path operations.
- Implement Device integration and verify save/restore, deferred copies, generation IDs, inverse fills and mixed clip types.
- Preserve the existing saturation and inclusive-row behavior confirmed by the pinned C++ probes. Do not alter the supplemental expectations.

### TEST → FIX → TEST

Run unchanged M09 tests and reference commands. Separately compare every mask byte, bounds, run data where public, and clipping results for circles, curves, inverse fills, combinations, large coordinates and empty clips. Verify shader, Device and path-conversion behavior. Fix production and repeat.

**Completion condition:** Exact AA output replaces the approximation, pixel comparisons match, and no clip integration remains postponed.

## 10. Colors, profiles and skcms

**Starting position:** The ten originals and selected numeric differential pass. Other pixel formats and integration paths have known differences.

Sources: [handoff](handoffs/M10.md), [acceptance](acceptance/M10.md).

### FIX

- Match pinned `ABGR_4444`/`ARGB_4444` store bytes. Reproduce the reported `0x400b → 0x320b` behavior with the exact reference configuration and call. If it is defined behavior, match it even though it does not round-trip. Do not substitute the declared nibble packing merely because it seems correct.
- Match half-float premultiplied conversion, including the reference's intermediate half-precision rounding. Close the recorded low-bit differences instead of accepting an arithmetic-width adaptation.
- Determine whether the reported out-of-domain 10-bit XR cases are valid calls under the pinned API. For defined cases, match sign extension, saturation and packing exactly. For excluded cases, record the actual source precondition; do not invent a valid-domain restriction.
- Implement 8888 row transforms and raster-pipeline application for ColorSpaceXformSteps using real pipeline code.
- Complete ColorTable's bitmap integration and verify storage/ownership behavior, reusing milestone 2's ownership work and real pixel-storage dependencies.
- Implement ReadBuffer/WriteBuffer and flattenable bridges for color tables and color spaces.
- Complete the color-profile consumers and integration needed to exercise the assigned color-management APIs, including high-bit-depth ICC transforms, color filters and applicable GPU consumers. Implement necessary dependencies before claiming full integration; keep their later original acceptance gates mandatory too.
- Audit scalar versus Apple Silicon vector execution for transform order, rounding, alpha, CMYK, gray channels, curve tables, CLUTs and byte order.
- Preserve the `classify_into` and double-byte-swap fixes recorded in acceptance. Do not add excluded x86 kernels.

### TEST → FIX → TEST

Run unchanged M10 tests, fixture checks, reference builds and the existing numeric differential. Separately compare the formerly different 4444, half-float and XR cases, then cover supported pixel/alpha formats, valid profile paths, malformed input and serialization. Compare destination bytes exactly. Fix production and repeat.

**Completion condition:** All recorded format differences are resolved or proven outside the defined API using source evidence; required integration works; existing tests are unchanged and pass.

## Evidence and progress record

For each milestone, save a new report at `plan/fix-evidence/Mxx/report.md`. Keep the original handoff and acceptance history intact.

Each report must include:

- Starting Rust revision and working-tree changes, reference revision/configuration, host and tool versions.
- A checklist of every gap above and every additional gap found in the full assigned-source audit.
- For each FIX → TEST round: production change, exact commands, exit codes, results and remaining failures.
- C++ and Rust comparison inputs, outputs and exact differences, including failures from earlier rounds.
- Protected test/source manifests and proof existing tests, fixtures, expected outputs and discovery were preserved.
- A disposition for every language adaptation, platform exclusion and dependency brought forward from a later milestone.
- Final state: `COMPLETE` only when all completion conditions pass; otherwise `IN_PROGRESS` or `BLOCKED`, with the concrete reason.

| Order | Milestone | Fix-plan state |
| --- | --- | --- |
| 1 | Scalars and points | COMPLETE — see [evidence](fix-evidence/M01/report.md) |
| 2 | Ownership, containers, data and streams | NOT STARTED |
| 3 | Numeric primitives, vectors and random | NOT STARTED |
| 4 | Rectangles, rounded rectangles, sizes and 3D points | NOT STARTED |
| 5 | Matrices and camera transforms | NOT STARTED |
| 6 | Bézier geometry, roots and clipping | NOT STARTED — verification first |
| 7 | Paths, builders, iteration and measurement | NOT STARTED |
| 8 | Stroking and path effects | NOT STARTED |
| 9 | Regions, clipping and spatial indexes | NOT STARTED |
| 10 | Colors, profiles and skcms | NOT STARTED |

**Do not turn a red result green by changing a test. FIX production, TEST again, and repeat until the milestone is complete.**
