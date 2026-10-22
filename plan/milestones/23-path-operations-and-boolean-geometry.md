# Milestone 23: Path operations and boolean geometry

## Outcome and entry gate

Implement the complete pathops library, winding conversion, intersection solvers, simplification, builders and threaded operation support. Preserve numerical thresholds and topology handling. Do not replace difficult operations with approximate polygonization.

Begin implementation only after milestone 22 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement the complete pathops library, winding conversion, intersection solvers, simplification, builders and threaded operation support. Preserve numerical thresholds and topology handling. Do not replace difficult operations with approximate polygonization.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/pathops/SkPathOps.h` — port to Rust.
- `src/pathops/SkAddIntersections.cpp` — port to Rust.
- `src/pathops/SkAddIntersections.h` — port to Rust.
- `src/pathops/SkDConicLineIntersection.cpp` — port to Rust.
- `src/pathops/SkDCubicLineIntersection.cpp` — port to Rust.
- `src/pathops/SkDCubicToQuads.cpp` — port to Rust.
- `src/pathops/SkDLineIntersection.cpp` — port to Rust.
- `src/pathops/SkDQuadLineIntersection.cpp` — port to Rust.
- `src/pathops/SkIntersectionHelper.h` — port to Rust.
- `src/pathops/SkIntersections.cpp` — port to Rust.
- `src/pathops/SkIntersections.h` — port to Rust.
- `src/pathops/SkLineParameters.h` — port to Rust.
- `src/pathops/SkOpAngle.cpp` — port to Rust.
- `src/pathops/SkOpAngle.h` — port to Rust.
- `src/pathops/SkOpBuilder.cpp` — port to Rust.
- `src/pathops/SkOpCoincidence.cpp` — port to Rust.
- `src/pathops/SkOpCoincidence.h` — port to Rust.
- `src/pathops/SkOpContour.cpp` — port to Rust.
- `src/pathops/SkOpContour.h` — port to Rust.
- `src/pathops/SkOpCubicHull.cpp` — port to Rust.
- `src/pathops/SkOpEdgeBuilder.cpp` — port to Rust.
- `src/pathops/SkOpEdgeBuilder.h` — port to Rust.
- `src/pathops/SkOpSegment.cpp` — port to Rust.
- `src/pathops/SkOpSegment.h` — port to Rust.
- `src/pathops/SkOpSpan.cpp` — port to Rust.
- `src/pathops/SkOpSpan.h` — port to Rust.
- `src/pathops/SkPathOpsAsWinding.cpp` — port to Rust.
- `src/pathops/SkPathOpsBounds.h` — port to Rust.
- `src/pathops/SkPathOpsCommon.cpp` — port to Rust.
- `src/pathops/SkPathOpsCommon.h` — port to Rust.
- `src/pathops/SkPathOpsConic.cpp` — port to Rust.
- `src/pathops/SkPathOpsConic.h` — port to Rust.
- `src/pathops/SkPathOpsCubic.cpp` — port to Rust.
- `src/pathops/SkPathOpsCubic.h` — port to Rust.
- `src/pathops/SkPathOpsCurve.cpp` — port to Rust.
- `src/pathops/SkPathOpsCurve.h` — port to Rust.
- `src/pathops/SkPathOpsDebug.cpp` — port to Rust.
- `src/pathops/SkPathOpsDebug.h` — port to Rust.
- `src/pathops/SkPathOpsLine.cpp` — port to Rust.
- `src/pathops/SkPathOpsLine.h` — port to Rust.
- `src/pathops/SkPathOpsOp.cpp` — port to Rust.
- `src/pathops/SkPathOpsPoint.h` — port to Rust.
- `src/pathops/SkPathOpsQuad.cpp` — port to Rust.
- `src/pathops/SkPathOpsQuad.h` — port to Rust.
- `src/pathops/SkPathOpsRect.cpp` — port to Rust.
- `src/pathops/SkPathOpsRect.h` — port to Rust.
- `src/pathops/SkPathOpsSimplify.cpp` — port to Rust.
- `src/pathops/SkPathOpsTCurve.h` — port to Rust.
- `src/pathops/SkPathOpsTSect.cpp` — port to Rust.
- `src/pathops/SkPathOpsTSect.h` — port to Rust.
- `src/pathops/SkPathOpsTightBounds.cpp` — port to Rust.
- `src/pathops/SkPathOpsTypes.cpp` — port to Rust.
- `src/pathops/SkPathOpsTypes.h` — port to Rust.
- `src/pathops/SkPathOpsWinding.cpp` — port to Rust.
- `src/pathops/SkPathWriter.cpp` — port to Rust.
- `src/pathops/SkPathWriter.h` — port to Rust.
- `src/pathops/SkReduceOrder.cpp` — port to Rust.
- `src/pathops/SkReduceOrder.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/pathops/BUILD.bazel`.
- `modules/pathops/pathops.gni`.
- `src/pathops/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate every pathops test and shared helper, including fuzz regressions, threaded cases and resource-backed SKP cases. Keep extended-mode tests visible and run them in the milestone gate. Compare path semantics and the original rendering-based validation unchanged.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/CubicChopTest.cpp`.
- `tests/CubicRootsTest.cpp`.
- `tests/PathOpsAngleIdeas.cpp`.
- `tests/PathOpsAngleTest.cpp`.
- `tests/PathOpsAsWindingTest.cpp`.
- `tests/PathOpsBattles.cpp`.
- `tests/PathOpsBoundsTest.cpp`.
- `tests/PathOpsBuildUseTest.cpp`.
- `tests/PathOpsBuilderConicTest.cpp`.
- `tests/PathOpsBuilderTest.cpp`.
- `tests/PathOpsChalkboardTest.cpp`.
- `tests/PathOpsConicIntersectionTest.cpp`.
- `tests/PathOpsConicLineIntersectionTest.cpp`.
- `tests/PathOpsConicQuadIntersectionTest.cpp`.
- `tests/PathOpsCubicConicIntersectionTest.cpp`.
- `tests/PathOpsCubicIntersectionTest.cpp`.
- `tests/PathOpsCubicIntersectionTestData.cpp`.
- `tests/PathOpsCubicIntersectionTestData.h`.
- `tests/PathOpsCubicLineIntersectionIdeas.cpp`.
- `tests/PathOpsCubicLineIntersectionTest.cpp`.
- `tests/PathOpsCubicQuadIntersectionTest.cpp`.
- `tests/PathOpsCubicReduceOrderTest.cpp`.
- `tests/PathOpsDCubicTest.cpp`.
- `tests/PathOpsDLineTest.cpp`.
- `tests/PathOpsDPointTest.cpp`.
- `tests/PathOpsDRectTest.cpp`.
- `tests/PathOpsDVectorTest.cpp`.
- `tests/PathOpsDebug.cpp`.
- `tests/PathOpsDebug.h`.
- `tests/PathOpsExtendedTest.cpp`.
- `tests/PathOpsExtendedTest.h`.
- `tests/PathOpsFuzz763Test.cpp`.
- `tests/PathOpsInverseTest.cpp`.
- `tests/PathOpsIssue3651.cpp`.
- `tests/PathOpsLineIntersectionTest.cpp`.
- `tests/PathOpsLineParametetersTest.cpp`.
- `tests/PathOpsOpCircleThreadedTest.cpp`.
- `tests/PathOpsOpCubicThreadedTest.cpp`.
- `tests/PathOpsOpLoopThreadedTest.cpp`.
- `tests/PathOpsOpRectThreadedTest.cpp`.
- `tests/PathOpsOpTest.cpp`.
- `tests/PathOpsQuadIntersectionTest.cpp`.
- `tests/PathOpsQuadIntersectionTestData.cpp`.
- `tests/PathOpsQuadIntersectionTestData.h`.
- `tests/PathOpsQuadLineIntersectionTest.cpp`.
- `tests/PathOpsQuadLineIntersectionThreadedTest.cpp`.
- `tests/PathOpsQuadReduceOrderTest.cpp`.
- `tests/PathOpsSimplifyDegenerateThreadedTest.cpp`.
- `tests/PathOpsSimplifyFailTest.cpp`.
- `tests/PathOpsSimplifyQuadThreadedTest.cpp`.
- `tests/PathOpsSimplifyQuadralateralsThreadedTest.cpp`.
- `tests/PathOpsSimplifyRectThreadedTest.cpp`.
- `tests/PathOpsSimplifyTest.cpp`.
- `tests/PathOpsSimplifyTrianglesThreadedTest.cpp`.
- `tests/PathOpsSkpTest.cpp`.
- `tests/PathOpsTSectDebug.h`.
- `tests/PathOpsTestCommon.cpp`.
- `tests/PathOpsTestCommon.h`.
- `tests/PathOpsThreadedCommon.cpp`.
- `tests/PathOpsThreadedCommon.h`.
- `tests/PathOpsThreeWayTest.cpp`.
- `tests/PathOpsTigerTest.cpp`.
- `tests/PathOpsTightBoundsTest.cpp`.
- `tests/PathOpsTypesTest.cpp`.
- `tests/QuadRootsTest.cpp`.
- `tests/RoundRectTest.cpp`.
- `tests/StrokerTest.cpp`.

### Original test registrations assigned to this gate

- `ChopCubicAtT_NormalizedCubic` — `tests/CubicChopTest.cpp:83`; CPU; `DEF_TEST`.
- `ChopCubicAtT_ArbitraryCubic` — `tests/CubicChopTest.cpp:173`; CPU; `DEF_TEST`.
- `CubicExtrema_FindsLocalMinAndMaxInRangeZeroToOneInclusive` — `tests/CubicChopTest.cpp:268`; CPU; `DEF_TEST`.
- `CubicInflectionPoints_FindsConvexityChangesInRangeZeroToOneInclusive` — `tests/CubicChopTest.cpp:390`; CPU; `DEF_TEST`.
- `BezierCurveHorizontalIntersectBinarySearch` — `tests/CubicChopTest.cpp:479`; CPU; `DEF_TEST`.
- `CubicRootsReal_ActualCubics` — `tests/CubicRootsTest.cpp:96`; CPU; `DEF_TEST`.
- `CubicRootsReal_Quadratics` — `tests/CubicRootsTest.cpp:202`; CPU; `DEF_TEST`.
- `CubicRootsReal_Linear` — `tests/CubicRootsTest.cpp:230`; CPU; `DEF_TEST`.
- `CubicRootsReal_Constant` — `tests/CubicRootsTest.cpp:240`; CPU; `DEF_TEST`.
- `CubicRootsReal_NonFiniteNumbers` — `tests/CubicRootsTest.cpp:250`; CPU; `DEF_TEST`.
- `CubicRootsValidT` — `tests/CubicRootsTest.cpp:381`; CPU; `DEF_TEST`.
- `CubicRootsValidT_ClampToZeroAndOne` — `tests/CubicRootsTest.cpp:422`; CPU; `DEF_TEST`.
- `PathOpsAngleOverlapHullsOne` — `tests/PathOpsAngleIdeas.cpp:580`; CPU; `DEF_TEST`.
- `PathOpsAngleOverlapHulls` — `tests/PathOpsAngleIdeas.cpp:599`; CPU; `DEF_TEST`.
- `PathOpsAngleBruteT` — `tests/PathOpsAngleIdeas.cpp:631`; CPU; `DEF_TEST`.
- `PathOpsAngleBruteTOne` — `tests/PathOpsAngleIdeas.cpp:676`; CPU; `DEF_TEST`.
- `PathOpsAngleExtreme` — `tests/PathOpsAngleIdeas.cpp:844`; CPU; `DEF_TEST`.
- `PathOpsAngleFindCrossEpsilon` — `tests/PathOpsAngleTest.cpp:48`; CPU; `DEF_TEST`.
- `PathOpsAngleFindQuadEpsilon` — `tests/PathOpsAngleTest.cpp:82`; CPU; `DEF_TEST`.
- `PathOpsAngleFindSlop` — `tests/PathOpsAngleTest.cpp:188`; CPU; `DEF_TEST`.
- `PathOpsAngleCircle` — `tests/PathOpsAngleTest.cpp:261`; CPU; `DEF_TEST`.
- `PathOpsAngleAfter` — `tests/PathOpsAngleTest.cpp:453`; CPU; `DEF_TEST`.
- `PathOpsAngleAllOnOneSide` — `tests/PathOpsAngleTest.cpp:499`; CPU; `DEF_TEST`.
- `PathOpsAsWinding` — `tests/PathOpsAsWindingTest.cpp:421`; CPU; `DEF_TEST`.
- `PathOpsBattle` — `tests/PathOpsBattles.cpp:11130`; CPU; `DEF_TEST`.
- `PathOpsBounds` — `tests/PathOpsBoundsTest.cpp:41`; CPU; `DEF_TEST`.
- `PathOpsBuildUse` — `tests/PathOpsBuildUseTest.cpp:2113`; CPU; `DEF_TEST`.
- `SixtyOvals` — `tests/PathOpsBuilderConicTest.cpp:110`; CPU; `DEF_TEST`.
- `SixtyOvalsOneOff` — `tests/PathOpsBuilderConicTest.cpp:134`; CPU; `DEF_TEST`.
- `SixtyOvals_2_2_9_73` — `tests/PathOpsBuilderConicTest.cpp:140`; CPU; `DEF_TEST`.
- `SixtyOvals_2_2_9_73_reduced` — `tests/PathOpsBuilderConicTest.cpp:363`; CPU; `DEF_TEST`.
- `SixtyOvalsA` — `tests/PathOpsBuilderConicTest.cpp:377`; CPU; `DEF_TEST`.
- `SixtyOvalsAX` — `tests/PathOpsBuilderConicTest.cpp:418`; CPU; `DEF_TEST`.
- `PathOpsBuilder` — `tests/PathOpsBuilderTest.cpp:17`; CPU; `DEF_TEST`.
- `BuilderIssue3838` — `tests/PathOpsBuilderTest.cpp:90`; CPU; `DEF_TEST`.
- `BuilderIssue3838_2` — `tests/PathOpsBuilderTest.cpp:112`; CPU; `DEF_TEST`.
- `BuilderIssue3838_3` — `tests/PathOpsBuilderTest.cpp:124`; CPU; `DEF_TEST`.
- `BuilderIssue502792_2` — `tests/PathOpsBuilderTest.cpp:145`; CPU; `DEF_TEST`.
- `Fuzz846` — `tests/PathOpsBuilderTest.cpp:160`; CPU; `DEF_TEST`.
- `Issue569540` — `tests/PathOpsBuilderTest.cpp:279`; CPU; `DEF_TEST`.
- `SkOpBuilderFuzz665` — `tests/PathOpsBuilderTest.cpp:311`; CPU; `DEF_TEST`.
- `SkOpBuilder618991` — `tests/PathOpsBuilderTest.cpp:332`; CPU; `DEF_TEST`.
- `SkOpBuilderKFuzz1` — `tests/PathOpsBuilderTest.cpp:355`; CPU; `DEF_TEST`.
- `PathOpsBuilder_UnionAbuttingRotatedRects` — `tests/PathOpsBuilderTest.cpp:374`; CPU; `DEF_TEST`.
- `PathOpsChalkboard` — `tests/PathOpsChalkboardTest.cpp:195`; CPU; `DEF_TEST`.
- `PathOpsConicIntersectionOneOff` — `tests/PathOpsConicIntersectionTest.cpp:356`; CPU; `DEF_TEST`.
- `PathOpsConicIntersection` — `tests/PathOpsConicIntersectionTest.cpp:360`; CPU; `DEF_TEST`.
- `PathOpsConicLineIntersectionOneOff` — `tests/PathOpsConicLineIntersectionTest.cpp:109`; CPU; `DEF_TEST`.
- `PathOpsConicLineIntersection` — `tests/PathOpsConicLineIntersectionTest.cpp:113`; CPU; `DEF_TEST`.
- `PathOpsConicQuadIntersection` — `tests/PathOpsConicQuadIntersectionTest.cpp:82`; CPU; `DEF_TEST`.
- `PathOpsConicQuadIntersectionOneOff` — `tests/PathOpsConicQuadIntersectionTest.cpp:89`; CPU; `DEF_TEST`.
- `PathOpsCubicConicIntersection` — `tests/PathOpsCubicConicIntersectionTest.cpp:98`; CPU; `DEF_TEST`.
- `PathOpsCubicConicIntersectionOneOff` — `tests/PathOpsCubicConicIntersectionTest.cpp:105`; CPU; `DEF_TEST`.
- `PathOpsCubicCoinOneOff` — `tests/PathOpsCubicIntersectionTest.cpp:886`; CPU; `DEF_TEST`.
- `PathOpsCubicIntersectionOneOff` — `tests/PathOpsCubicIntersectionTest.cpp:890`; CPU; `DEF_TEST`.
- `PathOpsCubicIntersectionTestsOneOff` — `tests/PathOpsCubicIntersectionTest.cpp:894`; CPU; `DEF_TEST`.
- `PathOpsCubicSelfOneOff` — `tests/PathOpsCubicIntersectionTest.cpp:898`; CPU; `DEF_TEST`.
- `PathOpsCubicIntersection` — `tests/PathOpsCubicIntersectionTest.cpp:902`; CPU; `DEF_TEST`.
- `PathOpsCubicLineRoots` — `tests/PathOpsCubicLineIntersectionIdeas.cpp:150`; CPU; `DEF_TEST`.
- `PathOpsCubicLineFailures` — `tests/PathOpsCubicLineIntersectionIdeas.cpp:287`; CPU; `DEF_TEST`.
- `PathOpsCubicLineOneFailure` — `tests/PathOpsCubicLineIntersectionIdeas.cpp:297`; CPU; `DEF_TEST`.
- `PathOpsFailCubicLineIntersection` — `tests/PathOpsCubicLineIntersectionTest.cpp:208`; CPU; `DEF_TEST`.
- `PathOpsCubicLineIntersection` — `tests/PathOpsCubicLineIntersectionTest.cpp:216`; CPU; `DEF_TEST`.
- `PathOpsCubicLineIntersectionOneOff` — `tests/PathOpsCubicLineIntersectionTest.cpp:224`; CPU; `DEF_TEST`.
- `PathOpsCubicQuadIntersection` — `tests/PathOpsCubicQuadIntersectionTest.cpp:127`; CPU; `DEF_TEST`.
- `PathOpsCubicQuadIntersectionOneOff` — `tests/PathOpsCubicQuadIntersectionTest.cpp:134`; CPU; `DEF_TEST`.
- `PathOpsReduceOrderCubic` — `tests/PathOpsCubicReduceOrderTest.cpp:68`; CPU; `DEF_TEST`.
- `PathOpsCubicHull` — `tests/PathOpsDCubicTest.cpp:27`; CPU; `DEF_TEST`.
- `SkDCubicPolynomialCoefficients` — `tests/PathOpsDCubicTest.cpp:65`; CPU; `DEF_TEST`.
- `PathOpsLineUtilities` — `tests/PathOpsDLineTest.cpp:29`; CPU; `DEF_TEST`.
- `PathOpsDPoint` — `tests/PathOpsDPointTest.cpp:30`; CPU; `DEF_TEST`.
- `PathOpsDRect` — `tests/PathOpsDRectTest.cpp:48`; CPU; `DEF_TEST`.
- `PathOpsDVector` — `tests/PathOpsDVectorTest.cpp:32`; CPU; `DEF_TEST`.
- `SkDVector_normalize` — `tests/PathOpsDVectorTest.cpp:59`; CPU; `DEF_TEST`.
- `SkDVector_normalize_infinity_and_nan` — `tests/PathOpsDVectorTest.cpp:77`; CPU; `DEF_TEST`.
- `PathOpsFuzz763` — `tests/PathOpsFuzz763Test.cpp:2448`; CPU; `DEF_TEST`.
- `PathOpsInverse` — `tests/PathOpsInverseTest.cpp:14`; CPU; `DEF_TEST`.
- `PathOpsIssue3651` — `tests/PathOpsIssue3651.cpp:1688`; CPU; `DEF_TEST`.
- `PathOpsLineIntersection` — `tests/PathOpsLineIntersectionTest.cpp:209`; CPU; `DEF_TEST`.
- `PathOpsLineIntersectionOneOff` — `tests/PathOpsLineIntersectionTest.cpp:232`; CPU; `DEF_TEST`.
- `PathOpsLineIntersectionExactOneOff` — `tests/PathOpsLineIntersectionTest.cpp:238`; CPU; `DEF_TEST`.
- `PathOpsLineIntersectionOneCoincident` — `tests/PathOpsLineIntersectionTest.cpp:244`; CPU; `DEF_TEST`.
- `PathOpsLineParameters` — `tests/PathOpsLineParametetersTest.cpp:49`; CPU; `DEF_TEST`.
- `PathOpsOpCircleThreaded` — `tests/PathOpsOpCircleThreadedTest.cpp:80`; CPU; `DEF_TEST`.
- `PathOpsOpCubicsThreaded` — `tests/PathOpsOpCubicThreadedTest.cpp:92`; CPU; `DEF_TEST`.
- `PathOpsOpLoopsThreaded` — `tests/PathOpsOpLoopThreadedTest.cpp:105`; CPU; `DEF_TEST`.
- `PathOpsRectsThreaded` — `tests/PathOpsOpRectThreadedTest.cpp:105`; CPU; `DEF_TEST`.
- `PathOpsFastThreaded` — `tests/PathOpsOpRectThreadedTest.cpp:202`; CPU; `DEF_TEST`.
- `bug_513820666` — `tests/PathOpsOpTest.cpp:9108`; CPU; `DEF_TEST`.
- `PathOpsOp` — `tests/PathOpsOpTest.cpp:9518`; CPU; `DEF_TEST`.
- `PathOpsFailOp` — `tests/PathOpsOpTest.cpp:12548`; CPU; `DEF_TEST`.
- `PathOpsRepOp` — `tests/PathOpsOpTest.cpp:12556`; CPU; `DEF_TEST`.
- `PathOpsQuadIntersectionOneOff` — `tests/PathOpsQuadIntersectionTest.cpp:611`; CPU; `DEF_TEST`.
- `PathOpsQuadIntersectionCoincidenceOneOff` — `tests/PathOpsQuadIntersectionTest.cpp:615`; CPU; `DEF_TEST`.
- `PathOpsQuadIntersection` — `tests/PathOpsQuadIntersectionTest.cpp:619`; CPU; `DEF_TEST`.
- `PathOpsQuadBinaryProfile` — `tests/PathOpsQuadIntersectionTest.cpp:626`; CPU; `DEF_TEST`.
- `PathOpsQuadLineIntersectionOneOff` — `tests/PathOpsQuadLineIntersectionTest.cpp:119`; CPU; `DEF_TEST`.
- `PathOpsQuadLineIntersection` — `tests/PathOpsQuadLineIntersectionTest.cpp:123`; CPU; `DEF_TEST`.
- `PathOpsQuadLineIntersectionThreaded` — `tests/PathOpsQuadLineIntersectionThreadedTest.cpp:124`; CPU; `DEF_TEST`.
- `PathOpsReduceOrderQuad` — `tests/PathOpsQuadReduceOrderTest.cpp:77`; CPU; `DEF_TEST`.
- `PathOpsSimplifyDegeneratesThreaded` — `tests/PathOpsSimplifyDegenerateThreadedTest.cpp:74`; CPU; `DEF_TEST`.
- `PathOpsSimplifyFail` — `tests/PathOpsSimplifyFailTest.cpp:224`; CPU; `DEF_TEST`.
- `PathOpsSimplifyDontFailOne` — `tests/PathOpsSimplifyFailTest.cpp:239`; CPU; `DEF_TEST`.
- `PathOpsSimplifyQuadsThreaded` — `tests/PathOpsSimplifyQuadThreadedTest.cpp:86`; CPU; `DEF_TEST`.
- `PathOpsSimplifyQuadralateralsThreaded` — `tests/PathOpsSimplifyQuadralateralsThreadedTest.cpp:88`; CPU; `DEF_TEST`.
- `PathOpsSimplifyRectsThreaded` — `tests/PathOpsSimplifyRectThreadedTest.cpp:198`; CPU; `DEF_TEST`.
- `PathOpsSimplify` — `tests/PathOpsSimplifyTest.cpp:9979`; CPU; `DEF_TEST`.
- `bug_513001309` — `tests/PathOpsSimplifyTest.cpp:9989`; CPU; `DEF_TEST`.
- `PathOpsSimplifyTrianglesThreaded` — `tests/PathOpsSimplifyTrianglesThreadedTest.cpp:77`; CPU; `DEF_TEST`.
- `PathOpsSkp` — `tests/PathOpsSkpTest.cpp:4766`; CPU; `DEF_TEST`.
- `PathOpsThreeWay` — `tests/PathOpsThreeWayTest.cpp:82`; CPU; `DEF_TEST`.
- `PathOpsThreeWayOneOff` — `tests/PathOpsThreeWayTest.cpp:89`; CPU; `DEF_TEST`.
- `PathOpsTiger` — `tests/PathOpsTigerTest.cpp:349`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsLines` — `tests/PathOpsTightBoundsTest.cpp:48`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsQuads` — `tests/PathOpsTightBoundsTest.cpp:124`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsMove` — `tests/PathOpsTightBoundsTest.cpp:137`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsMoveOne` — `tests/PathOpsTightBoundsTest.cpp:154`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsMoveTwo` — `tests/PathOpsTightBoundsTest.cpp:164`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsTiny` — `tests/PathOpsTightBoundsTest.cpp:175`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsWellBehaved` — `tests/PathOpsTightBoundsTest.cpp:188`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsIllBehaved` — `tests/PathOpsTightBoundsTest.cpp:199`; CPU; `DEF_TEST`.
- `PathOpsTightBoundsIllBehavedScaled` — `tests/PathOpsTightBoundsTest.cpp:210`; CPU; `DEF_TEST`.
- `PathOpsRoughly` — `tests/PathOpsTypesTest.cpp:19`; CPU; `DEF_TEST`.
- `QuadRootsReal_ActualQuadratics` — `tests/QuadRootsTest.cpp:92`; CPU; `DEF_TEST`.
- `QuadRootsReal_Linear` — `tests/QuadRootsTest.cpp:160`; CPU; `DEF_TEST`.
- `QuadRootsReal_Constant` — `tests/QuadRootsTest.cpp:170`; CPU; `DEF_TEST`.
- `QuadRootsReal_NonFiniteNumbers` — `tests/QuadRootsTest.cpp:180`; CPU; `DEF_TEST`.
- `QuadDiscriminant_Fibonacci` — `tests/QuadRootsTest.cpp:209`; CPU; `DEF_TEST`.
- `QuadRoots_Basic` — `tests/QuadRootsTest.cpp:224`; CPU; `DEF_TEST`.
- `QuadRoots_Fibonacci` — `tests/QuadRootsTest.cpp:243`; CPU; `DEF_TEST`.
- `QuadRoots_Hard` — `tests/QuadRootsTest.cpp:287`; CPU; `DEF_TEST`.
- `RoundRect` — `tests/RoundRectTest.cpp:1633`; CPU; `DEF_TEST`.
- `QuadStrokerSet` — `tests/StrokerTest.cpp:135`; CPU; `DEF_TEST`.
- `CubicStrokerSet` — `tests/StrokerTest.cpp:142`; CPU; `DEF_TEST`.
- `QuadStrokerUnbounded` — `tests/StrokerTest.cpp:163`; CPU; `DEF_TEST`.
- `CubicStrokerUnbounded` — `tests/StrokerTest.cpp:202`; CPU; `DEF_TEST`.
- `QuadStrokerConstrained` — `tests/StrokerTest.cpp:244`; CPU; `DEF_TEST`.
- `CubicStrokerConstrained` — `tests/StrokerTest.cpp:296`; CPU; `DEF_TEST`.
- `QuadStrokerRange` — `tests/StrokerTest.cpp:356`; CPU; `DEF_TEST`.
- `CubicStrokerRange` — `tests/StrokerTest.cpp:402`; CPU; `DEF_TEST`.
- `QuadStrokerOneOff` — `tests/StrokerTest.cpp:444`; CPU; `DEF_TEST`.
- `CubicStrokerOneOff` — `tests/StrokerTest.cpp:470`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/sksg/tests/SGTest.cpp` — M29.
- `tests/GrStyledShapeTest.cpp` — M32.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 2 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=23`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Deepseek runs the tests, completes the audit and marks M23 ACCEPTED may the user hand Deepseek M24.
