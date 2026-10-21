# Milestone 28: SVG parsing and rendering module

## Outcome and entry gate

Implement the modules/svg parser, DOM, presentation attributes, paint servers, filters, text and resource resolution. Preserve inheritance, transforms, units, clipping and invalid-value handling. Integrate font shaping and image decode through completed APIs.

Begin implementation only after milestone 27 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement the modules/svg parser, DOM, presentation attributes, paint servers, filters, text and resource resolution. Preserve inheritance, transforms, units, clipping and invalid-value handling. Integrate font shaping and image decode through completed APIs.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `modules/svg/include/SkSVGAttribute.h` — port to Rust.
- `modules/svg/include/SkSVGAttributeParser.h` — port to Rust.
- `modules/svg/include/SkSVGCircle.h` — port to Rust.
- `modules/svg/include/SkSVGClipPath.h` — port to Rust.
- `modules/svg/include/SkSVGContainer.h` — port to Rust.
- `modules/svg/include/SkSVGDOM.h` — port to Rust.
- `modules/svg/include/SkSVGDefs.h` — port to Rust.
- `modules/svg/include/SkSVGEllipse.h` — port to Rust.
- `modules/svg/include/SkSVGFe.h` — port to Rust.
- `modules/svg/include/SkSVGFeBlend.h` — port to Rust.
- `modules/svg/include/SkSVGFeColorMatrix.h` — port to Rust.
- `modules/svg/include/SkSVGFeComponentTransfer.h` — port to Rust.
- `modules/svg/include/SkSVGFeComposite.h` — port to Rust.
- `modules/svg/include/SkSVGFeDisplacementMap.h` — port to Rust.
- `modules/svg/include/SkSVGFeFlood.h` — port to Rust.
- `modules/svg/include/SkSVGFeGaussianBlur.h` — port to Rust.
- `modules/svg/include/SkSVGFeImage.h` — port to Rust.
- `modules/svg/include/SkSVGFeLightSource.h` — port to Rust.
- `modules/svg/include/SkSVGFeLighting.h` — port to Rust.
- `modules/svg/include/SkSVGFeMerge.h` — port to Rust.
- `modules/svg/include/SkSVGFeMorphology.h` — port to Rust.
- `modules/svg/include/SkSVGFeOffset.h` — port to Rust.
- `modules/svg/include/SkSVGFeTurbulence.h` — port to Rust.
- `modules/svg/include/SkSVGFilter.h` — port to Rust.
- `modules/svg/include/SkSVGFilterContext.h` — port to Rust.
- `modules/svg/include/SkSVGG.h` — port to Rust.
- `modules/svg/include/SkSVGGradient.h` — port to Rust.
- `modules/svg/include/SkSVGHiddenContainer.h` — port to Rust.
- `modules/svg/include/SkSVGIDMapper.h` — port to Rust.
- `modules/svg/include/SkSVGImage.h` — port to Rust.
- `modules/svg/include/SkSVGLine.h` — port to Rust.
- `modules/svg/include/SkSVGLinearGradient.h` — port to Rust.
- `modules/svg/include/SkSVGMask.h` — port to Rust.
- `modules/svg/include/SkSVGNode.h` — port to Rust.
- `modules/svg/include/SkSVGOpenTypeSVGDecoder.h` — port to Rust.
- `modules/svg/include/SkSVGPath.h` — port to Rust.
- `modules/svg/include/SkSVGPattern.h` — port to Rust.
- `modules/svg/include/SkSVGPoly.h` — port to Rust.
- `modules/svg/include/SkSVGRadialGradient.h` — port to Rust.
- `modules/svg/include/SkSVGRect.h` — port to Rust.
- `modules/svg/include/SkSVGRenderContext.h` — port to Rust.
- `modules/svg/include/SkSVGSVG.h` — port to Rust.
- `modules/svg/include/SkSVGShape.h` — port to Rust.
- `modules/svg/include/SkSVGStop.h` — port to Rust.
- `modules/svg/include/SkSVGText.h` — port to Rust.
- `modules/svg/include/SkSVGTransformableNode.h` — port to Rust.
- `modules/svg/include/SkSVGTypes.h` — port to Rust.
- `modules/svg/include/SkSVGUse.h` — port to Rust.
- `modules/svg/include/SkSVGValue.h` — port to Rust.
- `modules/svg/src/SkSVGAttribute.cpp` — port to Rust.
- `modules/svg/src/SkSVGAttributeParser.cpp` — port to Rust.
- `modules/svg/src/SkSVGCircle.cpp` — port to Rust.
- `modules/svg/src/SkSVGClipPath.cpp` — port to Rust.
- `modules/svg/src/SkSVGContainer.cpp` — port to Rust.
- `modules/svg/src/SkSVGDOM.cpp` — port to Rust.
- `modules/svg/src/SkSVGEllipse.cpp` — port to Rust.
- `modules/svg/src/SkSVGFe.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeBlend.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeColorMatrix.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeComponentTransfer.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeComposite.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeDisplacementMap.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeFlood.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeGaussianBlur.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeImage.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeLightSource.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeLighting.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeMerge.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeMorphology.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeOffset.cpp` — port to Rust.
- `modules/svg/src/SkSVGFeTurbulence.cpp` — port to Rust.
- `modules/svg/src/SkSVGFilter.cpp` — port to Rust.
- `modules/svg/src/SkSVGFilterContext.cpp` — port to Rust.
- `modules/svg/src/SkSVGGradient.cpp` — port to Rust.
- `modules/svg/src/SkSVGImage.cpp` — port to Rust.
- `modules/svg/src/SkSVGLine.cpp` — port to Rust.
- `modules/svg/src/SkSVGLinearGradient.cpp` — port to Rust.
- `modules/svg/src/SkSVGMask.cpp` — port to Rust.
- `modules/svg/src/SkSVGNode.cpp` — port to Rust.
- `modules/svg/src/SkSVGOpenTypeSVGDecoder.cpp` — port to Rust.
- `modules/svg/src/SkSVGPath.cpp` — port to Rust.
- `modules/svg/src/SkSVGPattern.cpp` — port to Rust.
- `modules/svg/src/SkSVGPoly.cpp` — port to Rust.
- `modules/svg/src/SkSVGRadialGradient.cpp` — port to Rust.
- `modules/svg/src/SkSVGRect.cpp` — port to Rust.
- `modules/svg/src/SkSVGRectPriv.h` — port to Rust.
- `modules/svg/src/SkSVGRenderContext.cpp` — port to Rust.
- `modules/svg/src/SkSVGSVG.cpp` — port to Rust.
- `modules/svg/src/SkSVGShape.cpp` — port to Rust.
- `modules/svg/src/SkSVGStop.cpp` — port to Rust.
- `modules/svg/src/SkSVGText.cpp` — port to Rust.
- `modules/svg/src/SkSVGTextPriv.h` — port to Rust.
- `modules/svg/src/SkSVGTransformableNode.cpp` — port to Rust.
- `modules/svg/src/SkSVGUse.cpp` — port to Rust.
- `modules/svg/src/SkSVGValue.cpp` — port to Rust.
- `modules/svg/utils/SvgTool.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/svg/BUILD.bazel`.
- `modules/svg/BUILD.gn`.
- `modules/svg/include/BUILD.bazel`.
- `modules/svg/src/BUILD.bazel`.
- `modules/svg/svg.gni`.
- `modules/svg/tests/BUILD.bazel`.
- `modules/svg/utils/BUILD.bazel`.

## Codex: acceptance-test contract

Translate the module's Filters, PaintServer and Text tests and relevant rendering fixtures. Exercise nested transforms, gradients, resource cycles/failure and SVG text positioning. Compare equivalent scenes against the pinned C++ renderer.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `modules/svg/tests/Filters.cpp`.
- `modules/svg/tests/PaintServer.cpp`.

### Original test registrations assigned to this gate

- `Svg_Filters_NonePaintInputs` — `modules/svg/tests/Filters.cpp:16`; CPU; `DEF_TEST`.
- `Svg_PaintServer_InheritedCurrentColor` — `modules/svg/tests/PaintServer.cpp:66`; CPU; `DEF_TEST`.
- `Svg_PaintServer_SeededCurrentColor` — `modules/svg/tests/PaintServer.cpp:83`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `modules/svg/tests/Text.cpp` — M29.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 6 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=28`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M28 ACCEPTED may the user hand Deepseek M29.
