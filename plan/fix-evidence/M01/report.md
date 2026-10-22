# M01 fix evidence — Scalars and points

Final state: **COMPLETE**, 2026-10-22. No production defect remains from this audit. Final production code is unchanged. M02 has not been started.

## Reproduction and provenance

Starting Rust revision: `fbf195f53054825ce1b8c1b47b62a333acd56891`. Initial user change: untracked `plan/fix.md`. This work adds the present evidence directory and corrects relative handoff/acceptance links in that plan. A concurrent `.DS_Store` modification was observed and left alone. Original handoffs, acceptance reports, tests, Cargo settings and C++ sources were preserved.

Pinned C++ revision: `b81c69985fd14bb7bfbd356b7618633ce07dc1a4`, clean working tree at baseline. Native arm64 macOS 15.6 (24G84), Rust/Cargo 1.94.1, Apple clang 17.0.0, SDK 26.1. Exact versions and status commands: [environment](environment.log).

The original M01 reference runner uses its unchanged debug (`-O0 -g -DSK_DEBUG`) and release (`-O3 -DNDEBUG`) configurations. The additional probe uses those profiles, C++20 and **`-ffp-contract=off`**, as required by pinned `gn/skia/BUILD.gn:71–75` for non-Windows clang. See [source policy excerpt](reference-fp-policy.txt). No compiler flag was added to Rust or to any existing test runner. Reference build products stay outside the upstream checkout.

## Assigned-source audit and gap dispositions

- [x] `include/core/SkScalar.h`: constants, floor/ceil/trunc/round, saturation, math wrappers, signs, fractions, inverses, degrees/radians, midpoint, tolerance/snap helpers, interpolation and explicit-count equality. Rust preserves operation order and the shared platform math functions. Existing originals/supplemental checks plus the additional probe cover boundaries. No approximation found.
- [x] `src/core/SkScalar.cpp`: first/last key clamping, exact keys, duplicate keys, table walk and interior interpolation match. Additional fixed duplicate-key table uses keys `[0,1,1,2]`, values `[-3,4,7,9]`, searches `[-1,0,.5,1,1.5,2,3]`. C++ preconditions and Rust slice handling are detailed below.
- [x] `include/private/SkFloatingPoint.h`: finite/NaN classification, all saturation widths, rounding, no-saturate casts, double casts, midpoint, reciprocal sqrt, IEEE division. NaN saturation selects the upper limit exactly as upstream. Mathematical domain errors produce IEEE NaNs. Windows/MSVC branches are excluded by the established native macOS scope.
- [x] `src/core/SkFloatingPoint.cpp`: exponent-mask magnitude, minimum normal magnitude, strict ULP comparison, equality of same-sign infinities, rejection of NaNs/opposite infinities, float-epsilon nearly-zero boundary. Existing supplemental checks cover these branches; source expressions match, including the `maxUlpsDiff + 1` factor and strict `<`.
- [x] `src/core/SkFloatBits.h`: bit copies, sign-bit/complement conversions and scalar aliases. Negative zero becomes positive zero only through the complement path; raw bit copying preserves it. Existing encoding tests check NaN payloads. `Sk2sComplimentToSignBit(INT_MIN)` overflows signed subtraction in C++; Rust wrapping is an extension, not a competing defined C++ result.
- [x] `include/core/SkPoint.h`: integer/float point layout, constructors/accessors, integer promotion, saturating integer arithmetic, float operators, scaling/offset, equality, finite checks, lengths, normalization, products. Rust `repr(C)` layout/offset assertions preserve the scalar-address API. In-place methods preserve C++ aliasing behavior; slices and references express valid storage/lifetimes.
- [x] `src/core/SkPoint.cpp`: float length fast path, double overflow fallback, double-precision normalization, zero/nonfinite-result reset, returned original length, line/segment distances and degenerate fallbacks. Additional probe compares all 529 coordinate pairs, 6 requested lengths, both normal and fast setters, return values and resulting coordinates; it also checks lengths, returned normalization lengths, midpoint, products and distances.
- [x] `src/core/SkPointPriv.h`: finite arrays, normalization predicate, side classification, distances/tolerance, rotations, negation, orthogonal vectors, strided rectangle fan/strip stores. Existing supplemental/development cases exercise these helpers; source audit confirms store order and stride. Unsafe Rust store contracts require valid aligned storage just as C++ does. Side output is an optional mutable reference.
- [x] `SetLengthFast`: the template boolean only constrains an assertion about the optional original-length output. The arithmetic is identical to ordinary setLength. The paired probe demonstrates the same booleans and coordinate bits for supported boundary inputs.
- [x] Later prerequisite closure: M02 supplied annotation/core/ownership infrastructure; M03 supplies Math/Safe32/To conversions; M04 supplies full rectangles. Current `math.rs`, `safe32.rs`, `sk_to.rs`, and rectangle implementations retain the M01 prerequisites. Their existing M02–M04 targets pass cumulatively in both profiles. Old handoff deferrals do not represent missing M01 dependencies. No new dependency was brought forward.

## Preconditions and language adaptations

The separate [Rust adaptation program](adaptations.rs) passes in debug and release. It tests empty/mismatched interpolation slices, explicit equality counts, empty finite checks, minimum integer negation and out-of-range casts.

- C++ `SkFloatInterpFunc` asserts `length > 0`, non-null arrays and sorted keys; its release empty call would access `values[0]` or `values[-1]`. Rust returns zero for empty equal slices. Unequal slices panic in debug and use their shorter length in release. The earlier handoff's shorter-length description omitted the debug assertion; no production change is needed. Valid explicit-count C++ calls are represented by passing equal-length prefixes of the arrays. Duplicate sorted keys are supported and directly compared.
- C++ `SkScalarsEqual` asserts a nonnegative count and reads exactly that prefix; zero reads nothing and returns true. Rust `scalars_equal_n` matches for valid counts, including zero, and rejects an oversized count by bounds checks in both profiles. `scalars_equal` additionally checks slice lengths, expressing the whole-slice operation.
- C++ `SkIsFinite(array,count)` reads `array[0]` before checking count. `SkPointPriv::AreFinite` similarly forms the first scalar address. An empty array without a readable first element cannot validly call these implementations. A zero-count call with readable backing storage actually classifies its first scalar, rather than being an empty-array identity. Rust's empty slices return true and never read an absent element; the slice interface cannot express that hidden backing-element operation. This is an explicit safe empty-input extension; nonempty behavior follows the reference product walk.
- C++ integer point negation at `INT_MIN` is signed overflow. Rust wrapping defines an extra case; ordinary negation matches. C++ no-saturate float-to-integer casts are undefined when the truncated result is unrepresentable, including infinities and NaNs. Rust defines these extra cases through saturating casts. Valid conversions are unchanged. `sk_double_to_float` explicitly permits a bracketing largest finite value or infinity on overflow; this host and Rust select infinity.
- Renamed overloads and reference/slice ownership are the same adaptations listed in M01 acceptance. No required allocation or serialization behavior exists in the scalar/point closure. Windows and 32-bit rectangle branches remain excluded; Intel is excluded under the current scope document, superseding the historical pending-Intel note.

## FIX → TEST rounds

1. Baseline: full debug/release suites, formatting, Clippy, original C++ M01 runner and integrity passed. Plan validation failed because the new `plan/fix.md` linked to `plan/handoffs/...` relative to its own `plan/` directory. [Initial error](initial-plan-link-failure.log). Corrected only those documentation links.
2. Initial standalone probe: a missing upstream logging translation unit caused a link error, retained in [initial link failure](initial-probe-link-failure.log). Adding the actual logging sources resolved it. The first numerical comparison then reported 14 differences per profile, retained in [round-one results](round1-probe-results.json) and original C++/Rust output files. A tentative fused-arithmetic production change eliminated the debug probe differences but caused eight OffsetSimplePoly assertions to fail in the full regression suite; [debug](round2/test-debug.log) and [release](round2/test-release.log) retain those failures. Tests were never edited.
3. Investigation located the pinned build's explicit contraction policy. The initial standalone probe used clang defaults and was therefore not configured like Skia. Restored all tentative production changes exactly to HEAD and added the source-required flag to the new probe. This corrects the comparison configuration; it does not relax expected values or choose a different Rust implementation. The unchanged original reference runner still passes separately. Expanded the frozen probe with additional scalar wrappers, products and interpolation. Final debug/release comparisons each contain **26,457 matching output tokens, zero differences**; all existing suites pass again.

All generated comparison sources and fixed inputs are separate from the existing test tree: [generator](make_probe.py), [inputs](inputs.txt), [C++ program](boundary.cpp), [Rust program](boundary.rs), [driver](run_probe.py). Non-NaN floats compare exact IEEE bits; NaNs compare classification because these arithmetic APIs do not specify payload/sign propagation. Raw float-bit identity remains checked by the existing encoding tests. Integer and boolean outputs compare exactly; no numeric tolerance is used. The additional probe exits unsuccessfully for any difference or unequal output length.

## Final commands and outcomes

Exact command lines and exit codes are in the linked logs and driver files. All final commands exited **0**.

| Command | Result / evidence |
| --- | --- |
| `cargo test --locked` | 514 passed, none failed/ignored/filtered; [log](test-debug.log) |
| `cargo test --release --locked` | Same 514 passed; [log](test-release.log) |
| M01 targets: `cargo test [--release] --locked --test point_test --test scalar_test --test harness_test --test supplemental_m01` | All 15 integration cases pass per profile; [debug](m01-debug.log), [release](m01-release.log) |
| `cargo fmt --all -- --check` | [PASS](fmt.log) |
| `cargo clippy --all-targets --locked -- -D warnings` | [PASS](clippy.log) |
| `python3 plan/validate_plan.py --source /Volumes/Data/Users/paul/development/src/github/skia --check-hashes` | All 12,474 source paths and plan accounting pass; [log](validate-plan.log) |
| `python3 tests/reference/run_m01.py --source /Volumes/Data/Users/paul/development/src/github/skia --output plan/fix-evidence/M01/reference` | Original registry and supplemental contracts pass in both profiles; [driver log](reference.log) |
| `python3 plan/fix-evidence/M01/run_probe.py` | Exact boundary comparisons and adaptation checks pass in both profiles; [results](probe-results.json) |
| `python3 plan/fix-evidence/M01/verify.py check` | 7,949 protected entries unchanged; [log](integrity.log) |

## Integrity and completion

[Baseline SHA-256 manifest](protected.json) covers all existing tests/reference tools, historical acceptance evidence, handoffs/inventory, every embedded test section from its first `#[cfg(test)]` onward, and reference `.h`/`.cpp` sources. The independent plan validator additionally verifies every pinned reference file, including GN build files. Final production files are byte-identical to HEAD, so embedded tests also retain their entire original containing files. No test/fixture/expected data was regenerated, and no existing runner was changed.

Discovery is identical before/after in both profiles: 514 names per profile, zero removed or added. See [discovery comparison](discovery-integrity.json) and the four discovery logs. Required numerical branches, adaptations and prerequisites above are resolved; there is no postponed M01 item or unavailable required comparison. Completion applies only to M01; later fix milestones retain their own obligations.
