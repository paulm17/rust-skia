# Milestone 30: Skottie animation and audio interfaces

## Outcome and entry gate

Implement Skottie JSON parsing, animation properties, keyframes, expressions, text/images, observers, external properties, audio interfaces and resource loading. Preserve time and frame semantics and the optional-feature behavior in this checkout.

Begin implementation only after milestone 29 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement Skottie JSON parsing, animation properties, keyframes, expressions, text/images, observers, external properties, audio interfaces and resource loading. Preserve time and frame semantics and the optional-feature behavior in this checkout.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `modules/audioplayer/SkAudioPlayer.cpp` — port to Rust.
- `modules/audioplayer/SkAudioPlayer.h` — port to Rust.
- `modules/audioplayer/SkAudioPlayer_mac.mm` — port to Rust.
- `modules/audioplayer/SkAudioPlayer_none.cpp` — port to Rust.
- `modules/audioplayer/SkAudioPlayer_oboe.cpp` — port to Rust.
- `modules/audioplayer/SkAudioPlayer_sfml.cpp` — port to Rust.
- `modules/skottie/include/ExternalLayer.h` — port to Rust.
- `modules/skottie/include/Skottie.h` — port to Rust.
- `modules/skottie/include/SkottieProperty.h` — port to Rust.
- `modules/skottie/include/SlotManager.h` — port to Rust.
- `modules/skottie/include/TextShaper.h` — port to Rust.
- `modules/skottie/src/Adapter.h` — port to Rust.
- `modules/skottie/src/BlendModes.cpp` — port to Rust.
- `modules/skottie/src/Camera.cpp` — port to Rust.
- `modules/skottie/src/Camera.h` — port to Rust.
- `modules/skottie/src/Composition.cpp` — port to Rust.
- `modules/skottie/src/Composition.h` — port to Rust.
- `modules/skottie/src/Layer.cpp` — port to Rust.
- `modules/skottie/src/Layer.h` — port to Rust.
- `modules/skottie/src/Path.cpp` — port to Rust.
- `modules/skottie/src/Skottie.cpp` — port to Rust.
- `modules/skottie/src/SkottieJson.cpp` — port to Rust.
- `modules/skottie/src/SkottieJson.h` — port to Rust.
- `modules/skottie/src/SkottiePriv.h` — port to Rust.
- `modules/skottie/src/SkottieProperty.cpp` — port to Rust.
- `modules/skottie/src/SkottieTool.cpp` — port to Rust.
- `modules/skottie/src/SkottieValue.h` — port to Rust.
- `modules/skottie/src/SlotManager.cpp` — port to Rust.
- `modules/skottie/src/Transform.cpp` — port to Rust.
- `modules/skottie/src/Transform.h` — port to Rust.
- `modules/skottie/src/animator/Animator.cpp` — port to Rust.
- `modules/skottie/src/animator/Animator.h` — port to Rust.
- `modules/skottie/src/animator/KeyframeAnimator.cpp` — port to Rust.
- `modules/skottie/src/animator/KeyframeAnimator.h` — port to Rust.
- `modules/skottie/src/animator/ScalarKeyframeAnimator.cpp` — port to Rust.
- `modules/skottie/src/animator/ShapeKeyframeAnimator.cpp` — port to Rust.
- `modules/skottie/src/animator/TextKeyframeAnimator.cpp` — port to Rust.
- `modules/skottie/src/animator/Vec2KeyframeAnimator.cpp` — port to Rust.
- `modules/skottie/src/animator/VectorKeyframeAnimator.cpp` — port to Rust.
- `modules/skottie/src/animator/VectorKeyframeAnimator.h` — port to Rust.
- `modules/skottie/src/effects/BlackAndWhiteEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/BrightnessContrastEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/BulgeEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/CCTonerEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/ColorOverlayStyles.cpp` — port to Rust.
- `modules/skottie/src/effects/CornerPinEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/DirectionalBlur.cpp` — port to Rust.
- `modules/skottie/src/effects/DisplacementMapEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/DropShadowEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/Effects.cpp` — port to Rust.
- `modules/skottie/src/effects/Effects.h` — port to Rust.
- `modules/skottie/src/effects/FillEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/FractalNoiseEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/GaussianBlurEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/GlowStyles.cpp` — port to Rust.
- `modules/skottie/src/effects/GradientEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/HueSaturationEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/InvertEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/LevelsEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/LinearWipeEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/MotionTileEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/RadialWipeEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/ShadowStyles.cpp` — port to Rust.
- `modules/skottie/src/effects/SharpenEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/ShiftChannelsEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/SkSLEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/SphereEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/ThresholdEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/TintEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/TransformEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/TritoneEffect.cpp` — port to Rust.
- `modules/skottie/src/effects/VenetianBlindsEffect.cpp` — port to Rust.
- `modules/skottie/src/layers/AudioLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/FootageLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/NullLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/PrecompLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/SolidLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/TextLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/Ellipse.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/FillStroke.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/Gradient.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/MergePaths.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/OffsetPaths.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/Polystar.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/PuckerBloat.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/Rectangle.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/Repeater.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/RoundCorners.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/ShapeLayer.cpp` — port to Rust.
- `modules/skottie/src/layers/shapelayer/ShapeLayer.h` — port to Rust.
- `modules/skottie/src/layers/shapelayer/TrimPaths.cpp` — port to Rust.
- `modules/skottie/src/text/Font.cpp` — port to Rust.
- `modules/skottie/src/text/Font.h` — port to Rust.
- `modules/skottie/src/text/RangeSelector.cpp` — port to Rust.
- `modules/skottie/src/text/RangeSelector.h` — port to Rust.
- `modules/skottie/src/text/TextAdapter.cpp` — port to Rust.
- `modules/skottie/src/text/TextAdapter.h` — port to Rust.
- `modules/skottie/src/text/TextAnimator.cpp` — port to Rust.
- `modules/skottie/src/text/TextAnimator.h` — port to Rust.
- `modules/skottie/src/text/TextShaper.cpp` — port to Rust.
- `modules/skottie/src/text/TextValue.cpp` — port to Rust.
- `modules/skottie/src/text/TextValue.h` — port to Rust.
- `modules/skottie/src/text/Unicode.cpp` — port to Rust.
- `modules/skottie/utils/PreshapeTool.cpp` — port to Rust.
- `modules/skottie/utils/SkottieUtils.cpp` — port to Rust.
- `modules/skottie/utils/SkottieUtils.h` — port to Rust.
- `modules/skottie/utils/TextEditor.cpp` — port to Rust.
- `modules/skottie/utils/TextEditor.h` — port to Rust.
- `modules/skottie/utils/TextPreshape.cpp` — port to Rust.
- `modules/skottie/utils/TextPreshape.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `modules/audioplayer/BUILD.bazel`.
- `modules/audioplayer/BUILD.gn`.
- `modules/skottie/BUILD.bazel`.
- `modules/skottie/BUILD.gn`.
- `modules/skottie/fuzz/BUILD.bazel`.
- `modules/skottie/gm/BUILD.bazel`.
- `modules/skottie/include/BUILD.bazel`.
- `modules/skottie/skottie.gni`.
- `modules/skottie/src/BUILD.bazel`.
- `modules/skottie/src/animator/BUILD.bazel`.
- `modules/skottie/src/effects/BUILD.bazel`.
- `modules/skottie/src/layers/BUILD.bazel`.
- `modules/skottie/src/layers/shapelayer/BUILD.bazel`.
- `modules/skottie/src/text/BUILD.bazel`.
- `modules/skottie/tests/BUILD.bazel`.
- `modules/skottie/utils/BUILD.bazel`.

## Codex: acceptance-test contract

Translate every Skottie unit test and module GM, including expression, keyframe, shaper, observer, image, text and audio-layer cases. Seek the same times/frame boundaries and retain original resources. Fuzz entrypoints are completed with M44.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `modules/skottie/fuzz/FuzzSkottieJSON.cpp`.
- `modules/skottie/gm/ExternalProperties.cpp`.
- `modules/skottie/gm/SkottieGM.cpp`.
- `modules/skottie/tests/AudioLayer.cpp`.
- `modules/skottie/tests/Expression.cpp`.
- `modules/skottie/tests/Image.cpp`.
- `modules/skottie/tests/Keyframe.cpp`.
- `modules/skottie/tests/PropertyObserver.cpp`.
- `modules/skottie/tests/Shaper.cpp`.
- `modules/skottie/tests/SkottieTest.cpp`.
- `modules/skottie/tests/Text.cpp`.

### Original test registrations assigned to this gate

- `Skottie_AudioLayer` — `modules/skottie/tests/AudioLayer.cpp:15`; CPU; `DEF_TEST`.
- `Skottie_Expression` — `modules/skottie/tests/Expression.cpp:82`; CPU; `DEF_TEST`.
- `Skottie_ExpressionText` — `modules/skottie/tests/Expression.cpp:269`; CPU; `DEF_TEST`.
- `Skottie_Image_CustomTransform` — `modules/skottie/tests/Image.cpp:16`; CPU; `DEF_TEST`.
- `Skottie_Keyframe` — `modules/skottie/tests/Keyframe.cpp:47`; CPU; `DEF_TEST`.
- `Skottie_Props` — `modules/skottie/tests/PropertyObserver.cpp:270`; CPU; `DEF_TEST`.
- `Skottie_Props_Revalidation` — `modules/skottie/tests/PropertyObserver.cpp:372`; CPU; `DEF_TEST`.
- `Skottie_Shaper_Clusters` — `modules/skottie/tests/Shaper.cpp:27`; CPU; `DEF_TEST`.
- `Skottie_Shaper_HAlign` — `modules/skottie/tests/Shaper.cpp:79`; CPU; `DEF_TEST`.
- `Skottie_Shaper_VAlign` — `modules/skottie/tests/Shaper.cpp:152`; CPU; `DEF_TEST`.
- `Skottie_Shaper_FragmentGlyphs` — `modules/skottie/tests/Shaper.cpp:225`; CPU; `DEF_TEST`.
- `Skottie_Shaper_ExplicitFontMgr` — `modules/skottie/tests/Shaper.cpp:271`; CPU; `DEF_TEST`.
- `Skottie_Shaper_breakiter` — `modules/skottie/tests/Shaper.cpp:369`; CPU; `DEF_TEST`.
- `Skottie_Shaper_fallback_metrics` — `modules/skottie/tests/Shaper.cpp:498`; CPU; `DEF_TEST`.
- `Skottie_Shaper_ZeroGlyphRuns` — `modules/skottie/tests/Shaper.cpp:520`; CPU; `DEF_TEST`.
- `Skottie_Shaper_CTStrict` — `modules/skottie/tests/Shaper.cpp:577`; CPU; `DEF_TEST`.
- `Skottie_OssFuzz8956` — `modules/skottie/tests/SkottieTest.cpp:19`; CPU; `DEF_TEST`.
- `Skottie_Annotations` — `modules/skottie/tests/SkottieTest.cpp:31`; CPU; `DEF_TEST`.
- `Skottie_Image_Loading` — `modules/skottie/tests/SkottieTest.cpp:97`; CPU; `DEF_TEST`.
- `Skottie_Layer_NoType` — `modules/skottie/tests/SkottieTest.cpp:246`; CPU; `DEF_TEST`.
- `Skottie_Gradient_InvalidCount` — `modules/skottie/tests/SkottieTest.cpp:272`; CPU; `DEF_TEST`.
- `Skottie_Text_Style` — `modules/skottie/tests/Text.cpp:77`; CPU; `DEF_TEST`.
- `Skottie_Text_LayoutError` — `modules/skottie/tests/Text.cpp:151`; CPU; `DEF_TEST`.
- `Skottie_Text_FontFamily` — `modules/skottie/tests/Text.cpp:231`; CPU; `DEF_TEST`.

### Preserve these fixtures, metadata and references

Use the immutable fixtures reached by the assigned tests. Shared `resources/` files are indexed under M43 for final audit but are available from the first consuming milestone.

## Dependency and integration notes

The initial scan found 55 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=30`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M30 ACCEPTED may the user hand Deepseek M31.
