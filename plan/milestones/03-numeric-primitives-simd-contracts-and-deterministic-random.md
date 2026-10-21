# Milestone 03: Numeric primitives, SIMD contracts and deterministic random

## Outcome and entry gate

Complete numeric helpers beyond the initial scalar subset: fixed point, half floats, safe conversions, vector operations, rounding and SkRandom. Preserve arithmetic widths, wrapping semantics, signed zeros and seed advancement. Provide scalar reference behavior before architecture-specific acceleration.

Begin implementation only after milestone 02 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Complete numeric helpers beyond the initial scalar subset: fixed point, half floats, safe conversions, vector operations, rounding and SkRandom. Preserve arithmetic widths, wrapping semantics, signed zeros and seed advancement. Provide scalar reference behavior before architecture-specific acceleration.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/private/SkFixed.h` — port to Rust.
- `include/private/SkMath.h` — port to Rust.
- `include/private/SkSafe32.h` — port to Rust.
- `include/private/SkTFitsIn.h` — port to Rust.
- `include/private/SkTo.h` — port to Rust.
- `src/core/SkBitmaskEnum.h` — port to Rust.
- `src/core/SkFDot6.h` — port to Rust.
- `src/core/SkFixedArray.h` — port to Rust.
- `src/core/SkHalf.cpp` — port to Rust.
- `src/core/SkHalf.h` — port to Rust.
- `src/core/SkMathPriv.cpp` — port to Rust.
- `src/core/SkMathPriv.h` — port to Rust.
- `src/core/SkRandom.h` — port to Rust.
- `src/core/SkSafeMath.cpp` — port to Rust.
- `src/core/SkSafeMath.h` — port to Rust.
- `src/core/SkVx.h` — port to Rust.
- `src/core/SkYUVMath.cpp` — port to Rust.
- `src/core/SkYUVMath.h` — port to Rust.
- `src/utils/SkFloatToDecimal.cpp` — port to Rust.
- `src/utils/SkFloatToDecimal.h` — port to Rust.
- `src/utils/SkFloatUtils.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

No additional build/tooling files are assigned in this milestone. Update the Rust build only as needed for this milestone.

## Codex: acceptance-test contract

Translate the math, floating-point, float16, safe-math, vector, random and conversion tests. Run boundary constants and deterministic sequences unchanged. Check both debug overflow behavior and optimized arithmetic; do not replace SkRandom with a different generator.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/BitSetTest.cpp`.
- `tests/ChecksumTest.cpp`.
- `tests/FitsInTest.cpp`.
- `tests/Float16Test.cpp`.
- `tests/FloatingPointTest.cpp`.
- `tests/MathTest.cpp`.
- `tests/OverAlignedTest.cpp`.
- `tests/RandomTest.cpp`.
- `tests/SafeMathTest.cpp`.
- `tests/SkBase64Test.cpp`.
- `tests/SkVxTest.cpp`.
- `tests/SortTest.cpp`.
- `tests/StreamTest.cpp`.
- `tests/TArrayTest.cpp`.
- `tests/TDPQueueTest.cpp`.
- `tests/UtilsTest.cpp`.

### Original test registrations assigned to this gate

- `BitSet` — `tests/BitSetTest.cpp:14`; CPU; `DEF_TEST`.
- `BitSetComparisonSizeMatching` — `tests/BitSetTest.cpp:65`; CPU; `DEF_TEST`.
- `BitSetComparisonBitMatching` — `tests/BitSetTest.cpp:81`; CPU; `DEF_TEST`.
- `Checksum` — `tests/ChecksumTest.cpp:21`; CPU; `DEF_TEST`.
- `GoodHash` — `tests/ChecksumTest.cpp:45`; CPU; `DEF_TEST`.
- `ChecksumCollisions` — `tests/ChecksumTest.cpp:51`; CPU; `DEF_TEST`.
- `ChecksumConsistent` — `tests/ChecksumTest.cpp:79`; CPU; `DEF_TEST`.
- `ChecksumStrings` — `tests/ChecksumTest.cpp:98`; CPU; `DEF_TEST`.
- `FitsIn` — `tests/FitsInTest.cpp:23`; CPU; `DEF_TEST`.
- `FloatToHalf` — `tests/Float16Test.cpp:29`; CPU; `DEF_TEST`.
- `FloatToHalf_Constants` — `tests/Float16Test.cpp:119`; CPU; `DEF_TEST`.
- `HalfToFloat` — `tests/Float16Test.cpp:129`; CPU; `DEF_TEST`.
- `DoubleNearlyZero` — `tests/FloatingPointTest.cpp:20`; CPU; `DEF_TEST`.
- `DoubleNearlyEqualUlps` — `tests/FloatingPointTest.cpp:47`; CPU; `DEF_TEST`.
- `BitCastDoubleRoundTrip` — `tests/FloatingPointTest.cpp:111`; CPU; `DEF_TEST`.
- `FMA` — `tests/FloatingPointTest.cpp:133`; CPU; `DEF_TEST`.
- `Midpoint` — `tests/FloatingPointTest.cpp:152`; CPU; `DEF_TEST`.
- `PopCount` — `tests/MathTest.cpp:369`; CPU; `DEF_TEST`.
- `Math` — `tests/MathTest.cpp:421`; CPU; `DEF_TEST`.
- `TestEndian` — `tests/MathTest.cpp:525`; CPU; `DEF_TEST`.
- `divmod_u8` — `tests/MathTest.cpp:598`; CPU; `DEF_TEST`.
- `divmod_u16` — `tests/MathTest.cpp:602`; CPU; `DEF_TEST`.
- `divmod_u32` — `tests/MathTest.cpp:606`; CPU; `DEF_TEST`.
- `divmod_u64` — `tests/MathTest.cpp:610`; CPU; `DEF_TEST`.
- `divmod_s8` — `tests/MathTest.cpp:614`; CPU; `DEF_TEST`.
- `divmod_s16` — `tests/MathTest.cpp:618`; CPU; `DEF_TEST`.
- `divmod_s32` — `tests/MathTest.cpp:622`; CPU; `DEF_TEST`.
- `divmod_s64` — `tests/MathTest.cpp:626`; CPU; `DEF_TEST`.
- `SkNextSizePow2` — `tests/MathTest.cpp:637`; CPU; `DEF_TEST`.
- `FloatSaturate32` — `tests/MathTest.cpp:668`; CPU; `DEF_TEST`.
- `FloatSaturate64` — `tests/MathTest.cpp:694`; CPU; `DEF_TEST`.
- `SkNextPow2` — `tests/MathTest.cpp:716`; CPU; `DEF_TEST`.
- `DoubleSaturate32` — `tests/MathTest.cpp:750`; CPU; `DEF_TEST`.
- `NeonU16Div255` — `tests/MathTest.cpp:777`; CPU; `DEF_TEST`.
- `OverAligned` — `tests/OverAlignedTest.cpp:18`; CPU; `DEF_TEST`.
- `Random` — `tests/RandomTest.cpp:186`; CPU; `DEF_TEST`.
- `SafeMath_SizeT_Success` — `tests/SafeMathTest.cpp:14`; CPU; `DEF_TEST`.
- `SafeMath_SizeT_Failure` — `tests/SafeMathTest.cpp:34`; CPU; `DEF_TEST`.
- `SafeMath_Int_Success` — `tests/SafeMathTest.cpp:58`; CPU; `DEF_TEST`.
- `SafeMath_Int_Failure` — `tests/SafeMathTest.cpp:75`; CPU; `DEF_TEST`.
- `SkBase64` — `tests/SkBase64Test.cpp:18`; CPU; `DEF_TEST`.
- `SkVx` — `tests/SkVxTest.cpp:18`; CPU; `DEF_TEST`.
- `SkVx_xy` — `tests/SkVxTest.cpp:159`; CPU; `DEF_TEST`.
- `SkVx_xyzw` — `tests/SkVxTest.cpp:190`; CPU; `DEF_TEST`.
- `SkVx_cross_dot` — `tests/SkVxTest.cpp:246`; CPU; `DEF_TEST`.
- `SkVx_strided_loads` — `tests/SkVxTest.cpp:306`; CPU; `DEF_TEST`.
- `SkVx_ScaledDividerU32` — `tests/SkVxTest.cpp:316`; CPU; `DEF_TEST`.
- `SkVx_saturated_add` — `tests/SkVxTest.cpp:354`; CPU; `DEF_TEST`.
- `SkVx_length` — `tests/SkVxTest.cpp:366`; CPU; `DEF_TEST`.
- `SkVx_normalize` — `tests/SkVxTest.cpp:385`; CPU; `DEF_TEST`.
- `SkVx_normalize_infinity_and_nan` — `tests/SkVxTest.cpp:408`; CPU; `DEF_TEST`.
- `SkVx_isfinite` — `tests/SkVxTest.cpp:425`; CPU; `DEF_TEST`.
- `Sort` — `tests/SortTest.cpp:38`; CPU; `DEF_TEST`.
- `Stream` — `tests/StreamTest.cpp:207`; CPU; `DEF_TEST`.
- `StreamPeek` — `tests/StreamTest.cpp:310`; CPU; `DEF_TEST`.
- `StreamPeek_BlockMemoryStream` — `tests/StreamTest.cpp:390`; CPU; `DEF_TEST`.
- `RemainingLengthIsBelow_MemoryStream` — `tests/StreamTest.cpp:420`; CPU; `DEF_TEST`.
- `DynamicMemoryWStream_detachAsData` — `tests/StreamTest.cpp:482`; CPU; `DEF_TEST`.
- `DynamicMemoryWStream_detachAsVector` — `tests/StreamTest.cpp:502`; CPU; `DEF_TEST`.
- `StreamCopy` — `tests/StreamTest.cpp:520`; CPU; `DEF_TEST`.
- `StreamEmptyStreamMemoryBase` — `tests/StreamTest.cpp:534`; CPU; `DEF_TEST`.
- `FILEStreamWithOffset` — `tests/StreamTest.cpp:540`; CPU; `DEF_TEST`.
- `RBuffer` — `tests/StreamTest.cpp:701`; CPU; `DEF_TEST`.
- `TArray_Swap` — `tests/TArrayTest.cpp:261`; CPU; `DEF_TEST`.
- `TArray_Move` — `tests/TArrayTest.cpp:320`; CPU; `DEF_TEST`.
- `TArray_NoUnnecessaryAllocs` — `tests/TArrayTest.cpp:338`; CPU; `DEF_TEST`.
- `TArray_SelfAssignment` — `tests/TArrayTest.cpp:410`; CPU; `DEF_TEST`.
- `FixedArray_SelfAssignment` — `tests/TArrayTest.cpp:415`; CPU; `DEF_TEST`.
- `TArray_Comparison` — `tests/TArrayTest.cpp:458`; CPU; `DEF_TEST`.
- `FixedArray_Comparison` — `tests/TArrayTest.cpp:468`; CPU; `DEF_TEST`.
- `TArray_Basic` — `tests/TArrayTest.cpp:557`; CPU; `DEF_TEST`.
- `FixedArray_Basic` — `tests/TArrayTest.cpp:566`; CPU; `DEF_TEST`.
- `TArray_Reserve` — `tests/TArrayTest.cpp:572`; CPU; `DEF_TEST`.
- `TArray_Construction` — `tests/TArrayTest.cpp:584`; CPU; `DEF_TEST`.
- `FixedArray_Construction` — `tests/TArrayTest.cpp:594`; CPU; `DEF_TEST`.
- `TArray_InnerPush` — `tests/TArrayTest.cpp:600`; CPU; `DEF_TEST`.
- `FixedArray_InnerPush` — `tests/TArrayTest.cpp:607`; CPU; `DEF_TEST`.
- `TArray_InnerEmplace` — `tests/TArrayTest.cpp:613`; CPU; `DEF_TEST`.
- `TArray_STArrayCompatibility` — `tests/TArrayTest.cpp:620`; CPU; `DEF_TEST`.
- `TArray_BoundsCheck` — `tests/TArrayTest.cpp:633`; CPU; `DEF_TEST`.
- `TArray_ASANPoisoning` — `tests/TArrayTest.cpp:684`; CPU; `DEF_TEST`.
- `TDPQueueTest` — `tests/TDPQueueTest.cpp:202`; CPU; `DEF_TEST`.
- `Utils` — `tests/UtilsTest.cpp:181`; CPU; `DEF_TEST`.
- `SkEnumerate` — `tests/UtilsTest.cpp:187`; CPU; `DEF_TEST`.
- `SkZip` — `tests/UtilsTest.cpp:306`; CPU; `DEF_TEST`.
- `SkMakeZip` — `tests/UtilsTest.cpp:479`; CPU; `DEF_TEST`.
- `UtilsPreserveBitPatterns` — `tests/UtilsTest.cpp:615`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/bentleyottmann/tests/MyersTest.cpp` — M42.
- `modules/sksg/tests/SGTest.cpp` — M29.
- `modules/skshaper/tests/ShaperTest.cpp` — M20.
- `modules/skunicode/tests/SkUnicodeTest.cpp` — M20.
- `tests/AAClipTest.cpp` — M14.
- `tests/BitmapTest.cpp` — M11.
- `tests/BlurTest.cpp` — M35, M38.
- `tests/CharToGlyphCache.cpp` — M18.
- `tests/ClipStackTest.cpp` — M09.
- `tests/CodecTest.cpp` — M17.
- `tests/ColorFilterTest.cpp` — M32, M38.
- `tests/ColorTest.cpp` — M10.
- `tests/CubicMapTest.cpp` — M06.
- `tests/CullTestTest.cpp` — M31.
- `tests/DrawBitmapRectTest.cpp` — M14.
- `tests/EdgeTest.cpp` — M14.
- `tests/EmptyPathTest.cpp` — M14.
- `tests/ExtendedSkColorTypeTests.cpp` — M32, M38.
- `tests/FilterResultTest.cpp` — M35, M38.
- `tests/FloatingPointTextureTest.cpp` — M38.
- `tests/FontTest.cpp` — M22.
- `tests/GeometryTest.cpp` — M06.
- `tests/GpuRectanizerTest.cpp` — M32.
- `tests/GrClipStackTest.cpp` — M32, M38.
- `tests/GrMemoryPoolTest.cpp` — M32.
- `tests/GrMeshTest.cpp` — M38.
- `tests/GrStyledShapeTest.cpp` — M32.
- `tests/GrSurfaceTest.cpp` — M32, M38.
- `tests/GrThreadSafeCacheTest.cpp` — M38.
- `tests/GradientTest.cpp` — M32, M38.
- `tests/ImageFilterTest.cpp` — M35, M38.
- `tests/ImageTest.cpp` — M32, M34, M38.
- `tests/M44Test.cpp` — M05, M07.
- `tests/MatrixProcsTest.cpp` — M12.
- `tests/MatrixTest.cpp` — M05.
- `tests/MipMapTest.cpp` — M14.
- `tests/OpChainTest.cpp` — M32.
- `tests/PDFDeflateWStreamTest.cpp` — M27.
- `tests/PDFGlyphsToUnicodeTest.cpp` — M27.
- `tests/PDFPrimitivesTest.cpp` — M27.
- `tests/PathBuilderTest.cpp` — M07.
- `tests/PathCoverageTest.cpp` — M07.
- `tests/PathOpsAngleIdeas.cpp` — M23.
- `tests/PathOpsAngleTest.cpp` — M23.
- `tests/PathOpsBuilderConicTest.cpp` — M23.
- `tests/PathOpsChalkboardTest.cpp` — M23.
- `tests/PathOpsCubicLineIntersectionIdeas.cpp` — M23.
- `tests/PathOpsCubicReduceOrderTest.cpp` — M23.
- `tests/PathOpsQuadReduceOrderTest.cpp` — M23.
- `tests/PathOpsTigerTest.cpp` — M23.
- `tests/PathOpsTightBoundsTest.cpp` — M23.
- `tests/PathTest.cpp` — M22.
- `tests/PictureTest.cpp` — M22.
- `tests/Point3Test.cpp` — M04.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/ProgramsTest.cpp` — M32.
- `tests/ProxyTest.cpp` — M38.
- `tests/RTreeTest.cpp` — M09.
- `tests/ReadPixelsTest.cpp` — M14.
- `tests/RegionTest.cpp` — M14.
- `tests/ResourceAllocatorTest.cpp` — M38.
- `tests/ResourceCacheTest.cpp` — M32, M38.
- `tests/RoundRectTest.cpp` — M04, M07, M23.
- `tests/SVGDeviceTest.cpp` — M27.
- `tests/SamplingTest.cpp` — M14.
- `tests/ShadowTest.cpp` — M15.
- `tests/SkIcoRustDecoderTest.cpp` — M16.
- `tests/SkPathRangeIterTest.cpp` — M07.
- `tests/SkRasterPipelineTest.cpp` — M12, M24, M31.
- `tests/SkStrikeTest.cpp` — M22.
- `tests/SrcOverTest.cpp` — M14.
- `tests/StrokerTest.cpp` — M23.
- `tests/SurfaceTest.cpp` — M32, M34, M38.
- `tests/TextBlobCacheTest.cpp` — M32, M38.
- `tests/TextBlobTest.cpp` — M22.
- `tests/Time.cpp` — M27.
- `tests/TopoSortTest.cpp` — M32.
- `tests/TriangulatingPathRendererTests.cpp` — M32, M38.
- `tests/TypefaceMacTest.cpp` — M19.
- `tests/TypefaceTest.cpp` — M19.
- `tests/WangsFormulaTest.cpp` — M31.
- `tests/WindowRectanglesTest.cpp` — M32.
- `tests/WritePixelsTest.cpp` — M35, M38.
- `tests/Writer32Test.cpp` — M22.
- `tests/YUVTest.cpp` — M17.
- `tests/graphite/ImageProviderTest.cpp` — M38.
- `tests/graphite/IntersectionTreeTest.cpp` — M35.
- `tests/graphite/TextureFormatTest.cpp` — M38.
- `tests/graphite/UniformManagerTest.cpp` — M35.
- `tests/graphite/precompile/PaintParamsKeyTest.cpp` — M38.
- `tests/graphite/sparse_strips/FlattenTest.cpp` — M35.
- `tests/graphite/sparse_strips/LUTTest.cpp` — M35.
- `tests/graphite/sparse_strips/OracleTest.cpp` — M35.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 2 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=3`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

Interfaces with later consumers may be declared now, but the accepted feature set must execute real implementations for every reached call. Where a source spans several features (CPU/GPU, serialization, runtime shaders), record completed and pending symbols separately. Deferred portions have explicit integration gates M14, M22, M26, M38 and M47; no pending portion may disappear from the ledger.

## Commands, required evidence and stop condition

Codex runs the milestone tests and all previously accepted tests in debug and release with the exact feature list from the handoff. The standard baseline commands are:

```sh
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

For feature-specific crates/backends, record the additional exact package/target/features commands; do not use an unexamined `--all-features` combination. Native backend, GM, browser, fuzz and benchmark commands are added when their harnesses exist and must be copied into the acceptance report.

Acceptance requires: all assigned macOS cases enumerated; every applicable assertion passing; all earlier accepted tests passing; every source row mapped to implemented Rust or a documented non-code disposition; no unexplained missing resources, untested required backends or unsupported required API; and a Codex-authored evidence report. Formatting/build success alone never constitutes acceptance.

Record PASS, FAIL, BLOCKED or NOT_APPLICABLE per test/configuration. NOT_APPLICABLE requires an original platform/capability condition and a reason. M01–M45 require native Apple Silicon macOS evidence; Intel-specific files may remain explicitly pending until M46, whose gate requires native Intel Mac results. This is a scheduled architecture gate, not an assertion that Intel already passed.

Only after Codex marks M03 ACCEPTED may the user hand Deepseek M04.
