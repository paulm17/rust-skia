# M07 test-writing handoff

Reference: Skia commit `b81c69985fd14bb7bfbd356b7618633ce07dc1a4`, source checkout `/Volumes/Data/Users/paul/development/src/github/skia`.
License: `tests/LICENSE.skia`. Copyright notices accompany translated modules.

Only tests and their local support were written. No execution, compilation, formatting, reference run, discovery, hashing, validation, audit or acceptance update was performed. These are unexecuted translations, not evidence of acceptance. Deepseek owns subsequent execution and acceptance work.

## Source correspondence

Each name below is the Rust test function of the same name, within the indicated module in the `m07_test` integration target. Locations are the original assigned registration lines, not newly measured source locations.

| Original source and line | Rust module | Original/Rust registration |
| --- | --- | --- |
| IsClosedSingleContourTest.cpp:11 | misc.rs | IsClosedSingleContourTest |
| M44Test.cpp:262 | misc.rs | M44_mapRect |
| OSPathTest.cpp:69 | parse_os.rs | OSPath |
| ParsePathTest.cpp:49 | parse_os.rs | ParsePath |
| ParsePathTest.cpp:78 | parse_os.rs | ParsePath_InvalidDoesNotCrash |
| ParsePathTest.cpp:91 | parse_os.rs | ParsePathOptionalCommand |
| ParsePathTest.cpp:132 | parse_os.rs | ParsePathArcFlags |
| PathBuilderTest.cpp:34 | builder.rs | pathbuilder |
| PathBuilderTest.cpp:59 | builder.rs | pathbuilder_filltype |
| PathBuilderTest.cpp:112 | builder.rs | pathbuilder_missing_move |
| PathBuilderTest.cpp:129 | builder.rs | pathbuilder_addRect |
| PathBuilderTest.cpp:203 | builder.rs | pathbuilder_addOval |
| PathBuilderTest.cpp:231 | builder.rs | pathbuilder_addRRect |
| PathBuilderTest.cpp:256 | builder.rs | pathbuilder_make |
| PathBuilderTest.cpp:276 | builder.rs | pathbuilder_genid |
| PathBuilderTest.cpp:288 | builder.rs | pathbuilder_addPolygon |
| PathBuilderTest.cpp:515 | builder.rs | pathbuilder_addPath |
| PathBuilderTest.cpp:541 | builder.rs | pathbuilder_addpath_crbug_1153516 |
| PathBuilderTest.cpp:592 | builder.rs | SkPathBuilder_multipleMoveTos |
| PathBuilderTest.cpp:615 | builder.rs | SkPathBuilder_lineToMoveTo |
| PathBuilderTest.cpp:634 | builder.rs | SkPathBuilder_arcToPtPtRad_invalidInputsResultInALine |
| PathBuilderTest.cpp:659 | builder.rs | SkPathBuilder_assign |
| PathBuilderTest.cpp:679 | builder.rs | SkPathBuilder_getLastPt |
| PathBuilderTest.cpp:691 | builder.rs | SkPathBuilder_transform |
| PathBuilderTest.cpp:793 | builder.rs | SkPathBuilder_cleaning |
| PathBuilderTest.cpp:810 | builder.rs | SkPathBuilder_path_roundtrip |
| PathBuilderTest.cpp:908 | builder.rs | SkPathBuilder_rMoveTo |
| PathBuilderTest.cpp:947 | builder.rs | SkPathBuilder_equality |
| PathBuilderTest.cpp:998 | builder.rs | SkPathBuilder_dump |
| PathBuilderTest.cpp:1025 | builder.rs | SkPathBuilder_trailingmove_addpath |
| PathBuilderTest.cpp:1063 | builder.rs | SkPathBuilder_b_463584612 |
| PathBuilderTest.cpp:1081 | builder.rs | SkPathBuilder_conic_semantics |
| PathBuilderTest.cpp:1127 | builder.rs | SkPathBuilder_b520944501 |
| PathCoverageTest.cpp:165 | misc.rs | PathCoverage |
| PathDataTest.cpp:32 | data.rs | pathdata_empty |
| PathDataTest.cpp:121 | data.rs | pathdata_rect |
| PathDataTest.cpp:176 | data.rs | pathdata_polygon |
| PathDataTest.cpp:233 | data.rs | pathdata_oval |
| PathDataTest.cpp:267 | data.rs | pathdata_rrect |
| PathDataTest.cpp:293 | data.rs | pathdata_make_edgecases |
| PathDataTest.cpp:375 | data.rs | pathdata_make_nonfinite |
| PathDataTest.cpp:417 | data.rs | pathdata_transform |
| PathDataTest.cpp:458 | data.rs | pathdata_transform_convexity |
| PathDataTest.cpp:509 | data.rs | pathdata_inverted_bounds |
| PathMeasureTest.cpp:107 | measure.rs | PathMeasure |
| PathMeasureTest.cpp:236 | measure.rs | PathMeasureConic |
| PathMeasureTest.cpp:255 | measure.rs | PathMeasure_nextctr |
| PathMeasureTest.cpp:310 | measure.rs | contour_measure |
| PathMeasureTest.cpp:338 | measure.rs | contour_measure_verbs |
| PathRawShapesTest.cpp:47 | raw.rs | pathrawshapes_rect |
| PathRawShapesTest.cpp:63 | raw.rs | pathrawshapes_oval |
| PathRawShapesTest.cpp:79 | raw.rs | pathrawshapes_rrect |
| PathRawTest.cpp:171 | raw.rs | pathraw_iter |
| PathRawTest.cpp:231 | raw.rs | pathraw_segmentmask |
| RRectInPathTest.cpp:380 | rrect.rs | RoundRectInPath |
| RRectInPathTest.cpp:393 | rrect.rs | RRect_fragile |
| RectTest.cpp:113 | misc.rs | Rect_path_nan |
| RoundRectTest.cpp:1786 | rrect.rs | RRect_b561770646_part1 |
| RoundRectTest.cpp:1805 | rrect.rs | RRect_b561770646_part2 |
| SkPathRangeIterTest.cpp:17 | misc.rs | SkPath_RangeIter |
| SubsetPath.cpp/.h | subset.rs | SubsetPath, SubsetContours, SubsetVerbs (utility; no registration) |

All other registrations in the split M44Test, RectTest and RoundRectTest sources remain at their assigned other gates. This translation adds no runner for those registrations.

## Helper and language correspondence

- `m07_test.rs` imports production APIs and supplies constructors for points, rectangles and matrices, the default scalar-nearly-equal tolerance, and construction of the production `RangeIter`. No geometric algorithm under test is replaced by a helper.
- The existing shared `support::Reporter` accumulates nonfatal reporter failures and evaluates each condition once. Each registration finishes its reporter. Original debug-only `SkASSERT`s use `debug_assert!`. Reporter contexts and diagnostics use `eprintln!`; optional/pointer dereferences use `unwrap`, corresponding to original unchecked dereferences after reporter assertions.
- C++ overloads/default arguments map to explicit Rust spellings, references and options: builder `snapshot(None)`/`detach(None)`, default Winding/CW/shape start index, Append mode, measurement resolution 1. Rect starts at 0, oval at 1, and rounded rectangle at 6 CW or 7 CCW. Constructors taking path use `from_path`. Builder assignment from a path uses that same production conversion. Move construction and assignment of `SkPathMeasure` are ownership moves.
- `builder.rs` includes `is_empty`, `check_points`, `is_eq`, every reached `test_addPath*` helper, closed/empty-path extension, injected-move helpers, range and raw iterator assertion helpers, round-trip and fill-type helpers. Array-of-expected records preserves each original contour-verb block's assertion and point order. Unrolled equivalent cases may use ordered loops, retaining each case's assertions and tolerances.
- `data.rs` retains all six rect makers, both polygon/oval/rrect makers, transform predicates, span equality, bad-rrect conditional construction and all count/nonfinite combinations. Rust `SkPathData::polygon` returns an Arc rather than nullable storage; null failure is expressed as the production error singleton. Borrowed spans become slices. The rect-point fixtures are the original four corners in direction order, optionally repeating the first point.
- `raw.rs` retains the fixed-capacity raw fixture builder, extension debug assertions and production segment-mask/bounds calls. Its owned Rust raw view is the production API's lifetime adaptation. `check_iter` retains point/verb/weight indices and the repeated terminal call.
- `measure.rs` retains all three small-segment helpers, the recursion-count assertions at default and 1000 resolution, all position/tangent assertions, empty-contour and MLM helpers, quarter-circle helpers, and both forms of verb iteration. Iterator presence/end observations map to Option/at_end. Strict original inequalities and numerical tolerances remain unchanged.
- `rrect.rs` retains the full reached helper closure, including nested nine-patch loops, no-check variants, original apparently redundant checks of rr rather than rr2, ignored inset outputs, and ignored transform return. Original raw float fixtures use `f32::from_bits`.
- `misc.rs` retains the complete PathCoverage helper formulas and five arrays, including the false EC/CE branch; RangeIter uses two independent production SkRandom generators and identical draw order, short circuit and early error return. The unused standalone `next_point` helper maps to the same helper. M44 retains corner containment, clipped edge tolerance adjustment (including signs), path comparison and all five matrix cases.
- Disabled behavior stays disabled: ParsePath's `#if 0` equality assertions; RRect_fragile's `if(false)` body; PathCoverage's false alternative estimators; `CONIC_PERSPECTIVE_BUG_FIXED=0` conic points/call. The default configuration leaves `SK_SUPPORT_LEGACY_PATHBUILDER_SETLASTPT` undefined, retaining its moveTo branch. The executable-path supplemental test retains a macOS guard.
- SubsetPath has borrowed source lifetime and owned selection storage. Its contour/verb variants share the common range and subset machinery, preserving restoration, retry counts, selection diagnostics and path reconstruction; it is not an implementation of a production path API.

## Production API gap visible while writing

The current `RRect` has no `transform(&Matrix) -> Option<RRect>` API. The original ignored call in `test_empty_crbug_458524` and asserted failure in `RRect_b561770646_part2` remain in the tests. Deepseek must implement this reached production prerequisite. No shim, ignore, altered assertion or production edit was added here. This has not been confirmed through compilation, which was prohibited for this writing pass.

## Supplemental contracts

`supplemental.rs` separately covers distance endpoints, infinities/NaNs and output preservation, matrix tangent/position, extraction and reset; 30-bit t endpoints, segment distance growth and finite-coordinate overflow; path clone/snapshot independence, generation identity, fill/volatile independence, owned raw and contour lifetimes, Arc ownership/drop; explicit v5 little-endian serialization bytes on Apple Silicon, v4 backward verbs, trailing input, decoded storage lifetime, all truncation points, signed/unsigned count overflow boundaries, bad version/type/verb/nonfinite/weight payloads; point-output capacity/sentinels and raw count mismatches; macOS executable-path basic contract.

## Commands for Deepseek

Run from `/Volumes/Data/Users/paul/development/src/github/rust-skia` after supplying any missing production APIs:

```sh
cargo test --locked --test m07_test
cargo test --release --locked --test m07_test
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

Reference execution, expanded registration discovery, fixture/source hash recording, auditing and acceptance/status updates remain Deepseek's work. No new C++ reference runner was needed for the test-writing scope; no speculative reference command is supplied.
