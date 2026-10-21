# Rust Skia: macOS milestone plan

This is the implementation/testing contract for porting the pinned Skia checkout to an owned Rust implementation. Deepseek implements one milestone; the user then asks Codex to translate and run that milestone's tests; failures go back to Deepseek; only Codex acceptance unlocks the next milestone. Codex does not pre-generate the remaining tests while writing this plan.

Reference: `b81c69985fd14bb7bfbd356b7618633ce07dc1a4` in `/Volumes/Data/Users/paul/development/src/github/skia`.
Destination: `/Volumes/Data/Users/paul/development/src/github/rust-skia`.
Scope: full repository accounting, macOS execution only. Read [macOS scope](MACOS-SCOPE.md) for exact platform, browser and dependency boundaries.

## Start here

[Copy-and-paste prompts for Deepseek and Codex](NEXT-STEPS.md) are provided for each handoff.

1. Read [workflow and ownership](WORKFLOW.md).
2. Open the next milestone below and give that file plus this folder to Deepseek.
3. Deepseek returns [an implementation handoff](HANDOFF-TEMPLATE.md) and stops.
4. Ask Codex to translate the listed original tests, run them, and create [an acceptance report](ACCEPTANCE-TEMPLATE.md).
5. Advance only when the gate is ACCEPTED. Keep failed or blocked work on the current milestone.

M01's Scalar/Point tests already passed locally on Apple Silicon in this conversation. Its status is `BASELINE_PASSED_LOCAL`, not a claim of complete source/API parity. Existing tests live in `../tests/`; M01 reconciles that evidence with this plan rather than replacing them.

## Milestones

| Gate | Milestone | Initial status |
| --- | --- | --- |
| M01 | [Scalar and point baseline](milestones/01-scalar-and-point-baseline.md) | BASELINE_PASSED_LOCAL |
| M02 | [Core ownership, containers, data and streams](milestones/02-core-ownership-containers-data-and-streams.md) | NOT_STARTED |
| M03 | [Numeric primitives, SIMD contracts and deterministic random](milestones/03-numeric-primitives-simd-contracts-and-deterministic-random.md) | NOT_STARTED |
| M04 | [Rectangles, rounded rectangles, sizes and 3D points](milestones/04-rectangles-rounded-rectangles-sizes-and-3d-points.md) | NOT_STARTED |
| M05 | [Matrices, camera transforms and coordinate mapping](milestones/05-matrices-camera-transforms-and-coordinate-mapping.md) | NOT_STARTED |
| M06 | [Bezier geometry, roots and clipping primitives](milestones/06-bezier-geometry-roots-and-clipping-primitives.md) | NOT_STARTED |
| M07 | [Paths, builders, iteration and measurement](milestones/07-paths-builders-iteration-and-measurement.md) | NOT_STARTED |
| M08 | [Stroking and path effects](milestones/08-stroking-and-path-effects.md) | NOT_STARTED |
| M09 | [Regions, clip storage and spatial indexes](milestones/09-regions-clip-storage-and-spatial-indexes.md) | NOT_STARTED |
| M10 | [Colors, profiles and skcms](milestones/10-colors-profiles-and-skcms.md) | NOT_STARTED |
| M11 | [Pixel storage, image information and sampling data](milestones/11-pixel-storage-image-information-and-sampling-data.md) | NOT_STARTED |
| M12 | [Raster pipeline and CPU dispatch](milestones/12-raster-pipeline-and-cpu-dispatch.md) | NOT_STARTED |
| M13 | [Paint, shaders, gradients and color filters](milestones/13-paint-shaders-gradients-and-color-filters.md) | NOT_STARTED |
| M14 | [CPU canvas, images, surfaces and drawing](milestones/14-cpu-canvas-images-surfaces-and-drawing.md) | NOT_STARTED |
| M15 | [Image filters, blur, masks and shadows](milestones/15-image-filters-blur-masks-and-shadows.md) | NOT_STARTED |
| M16 | [Image codecs, metadata and lazy decoding](milestones/16-image-codecs-metadata-and-lazy-decoding.md) | NOT_STARTED |
| M17 | [Image encoding and round trips](milestones/17-image-encoding-and-round-trips.md) | NOT_STARTED |
| M18 | [Font objects, glyph data and SFNT](milestones/18-font-objects-glyph-data-and-sfnt.md) | NOT_STARTED |
| M19 | [macOS fonts, CoreText and text drawing](milestones/19-macos-fonts-coretext-and-text-drawing.md) | NOT_STARTED |
| M20 | [Unicode and shaping](milestones/20-unicode-and-shaping.md) | NOT_STARTED |
| M21 | [Paragraph layout and editing](milestones/21-paragraph-layout-and-editing.md) | NOT_STARTED |
| M22 | [Pictures, records, serialization and capture](milestones/22-pictures-records-serialization-and-capture.md) | NOT_STARTED |
| M23 | [Path operations and boolean geometry](milestones/23-path-operations-and-boolean-geometry.md) | NOT_STARTED |
| M24 | [SkSL frontend, IR and analysis](milestones/24-sksl-frontend-ir-and-analysis.md) | NOT_STARTED |
| M25 | [SkSL code generation and shader fixtures](milestones/25-sksl-code-generation-and-shader-fixtures.md) | NOT_STARTED |
| M26 | [Runtime effects, meshes and deferred shader integration](milestones/26-runtime-effects-meshes-and-deferred-shader-integration.md) | NOT_STARTED |
| M27 | [PDF, SVG output and XML utilities](milestones/27-pdf-svg-output-and-xml-utilities.md) | NOT_STARTED |
| M28 | [SVG parsing and rendering module](milestones/28-svg-parsing-and-rendering-module.md) | NOT_STARTED |
| M29 | [Scene graph and resource providers](milestones/29-scene-graph-and-resource-providers.md) | NOT_STARTED |
| M30 | [Skottie animation and audio interfaces](milestones/30-skottie-animation-and-audio-interfaces.md) | NOT_STARTED |
| M31 | [Shared GPU infrastructure and tessellation](milestones/31-shared-gpu-infrastructure-and-tessellation.md) | NOT_STARTED |
| M32 | [Ganesh engine and mock backend](milestones/32-ganesh-engine-and-mock-backend.md) | NOT_STARTED |
| M33 | [Ganesh Metal backend](milestones/33-ganesh-metal-backend.md) | NOT_STARTED |
| M34 | [Ganesh macOS OpenGL backend](milestones/34-ganesh-macos-opengl-backend.md) | NOT_STARTED |
| M35 | [Graphite engine and scheduling](milestones/35-graphite-engine-and-scheduling.md) | NOT_STARTED |
| M36 | [Graphite Metal backend](milestones/36-graphite-metal-backend.md) | NOT_STARTED |
| M37 | [Graphite Dawn with Metal](milestones/37-graphite-dawn-with-metal.md) | NOT_STARTED |
| M38 | [Cross-backend image, surface and text integration](milestones/38-cross-backend-image-surface-and-text-integration.md) | NOT_STARTED |
| M39 | [macOS application and platform adapters](milestones/39-macos-application-and-platform-adapters.md) | NOT_STARTED |
| M40 | [Developer tools, inspection and debug clients](milestones/40-developer-tools-inspection-and-debug-clients.md) | NOT_STARTED |
| M41 | [CanvasKit and browser-facing bindings on macOS](milestones/41-canvaskit-and-browser-facing-bindings-on-macos.md) | NOT_STARTED |
| M42 | [Remaining optional modules and experiments](milestones/42-remaining-optional-modules-and-experiments.md) | NOT_STARTED |
| M43 | [Complete GM visual regression suite](milestones/43-complete-gm-visual-regression-suite.md) | NOT_STARTED |
| M44 | [Fuzz harnesses and regression corpora](milestones/44-fuzz-harnesses-and-regression-corpora.md) | NOT_STARTED |
| M45 | [Benchmarks and performance acceptance](milestones/45-benchmarks-and-performance-acceptance.md) | NOT_STARTED |
| M46 | [Repository build, examples, documentation and packaging](milestones/46-repository-build-examples-documentation-and-packaging.md) | NOT_STARTED |
| M47 | [Final macOS completeness and release gate](milestones/47-final-macos-completeness-and-release-gate.md) | NOT_STARTED |

## File accounting and limits of the initial scan

- [source-files.csv](inventory/source-files.csv): every Git-tracked reference file, its owner, milestone and disposition. Production source lists and test lists are separate in every milestone.
- [test-registrations.csv](inventory/test-registrations.csv): visible registration sites with initial gates. Conditional/macro-expanded registrations must be reconciled with the actual C++ runner before acceptance.
- [backend-validation.csv](inventory/backend-validation.csv): required native backend variants at M32–M37, before the M38 consolidated matrix. Context filters must be preserved.
- [forward-includes.csv](inventory/forward-includes.csv): dependency-review evidence. It deliberately retains unresolved preprocessor branches; it is not a topologically sorted build graph.
- [source-hashes.csv](inventory/source-hashes.csv): immutable reference hashes.
- [external-dependencies.csv](inventory/external-dependencies.csv): references extracted from DEPS; the original DEPS and build manifests remain authoritative for dynamic/CIPD/transitive dependencies.
- [inventory audit](INVENTORY-AUDIT.md): accounting checks and explicitly unresolved build-selection work.

Each milestone is an acceptance unit, not an estimate of equal effort. Large compiler, GPU and codec gates may take many implementation iterations. Do not split them into silently accepted partial milestones. If a real dependency requires reordering, amend the plan and both ledgers before proceeding; never hide it behind placeholders or weakened tests.
