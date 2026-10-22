# M10 test correspondence

Pinned reference: `b81c69985fd14bb7bfbd356b7618633ce07dc1a4`.
These are test-writing artifacts only. No execution, discovery, hashes or acceptance claims were produced.

| Original registration and source | Rust test in `colors.rs` unless specified |
| --- | --- |
| ColorPrivTest.cpp:18 Splay | Splay |
| ColorPrivTest.cpp:32 FourByteInterp | FourByteInterp |
| ColorPrivTest.cpp:56 SkBlendARGB32_SameOpaqueSrcAndDstWithAnyAlpha_ProducesSameColor | same name |
| ColorTest.cpp:19 ColorPremul | ColorPremul |
| ColorTest.cpp:40 SkPMColor_SetAndRetrieveChannels | same name |
| ColorTest.cpp:60 ColorInterp | ColorInterp (original false branch retained) |
| ColorTest.cpp:78 ColorFastIterp | ColorFastIterp |
| HSVRoundTripTest.cpp:14 ColorToHSVRoundTrip | ColorToHSVRoundTrip |
| ParseColorTest.cpp:168 ParseNamedColor | parse.rs: ParseNamedColor |
| SkColor4fTest.cpp:11 SkColor4f_FromColor | SkColor4f_FromColor |

All registrations are CPU/default-feature cases. Existing Reporter preserves nonfatal assertions; HSV ERRORF records failures and continues. Original SkRandom uses seed zero and the existing faithful production RNG. Packed-color conditional uses SK_R32_SHIFT to select the original RGBA/BGRA expectations. Parse named-color end-pointer/null returns map to Option over a slice of exactly the original explicit length; output color is retained across calls. The entire original named-color table is independent expected test data. No original registration uses external fixtures or numerical tolerances.

`supplemental.rs` separately covers integer channel boundaries with widened arithmetic, both packed interpolation widths, explicit byte order, short buffers and overflowing transform counts, table copies/Arc lifetimes, serialization truncation/version rejection, malformed ICC counts/offsets/sizes, and owned ICC profile use after source overwrite/drop and owner cloning/drop.

`fixtures/AdobeRGB1998.icc` is copied verbatim from `resources/icc_profiles/AdobeRGB1998.icc` in the pinned reference checkout. Deepseek owns hash/resource verification before execution. The remaining ICC/rendering/transform registrations retain the later gates listed in the M10 milestone; this correspondence makes no whole-file or later-gate completion claim.

Deepseek commands, from repository root:

```sh
cargo test --locked --test m10_test
cargo test --release --locked --test m10_test
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

Deepseek also owns reference registration discovery, resource/hash verification, differential execution, auditing and acceptance/status updates specified by the plan. No reference harness or acceptance runner is added here.
