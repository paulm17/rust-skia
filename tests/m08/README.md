# M08 test correspondence

Reference: Skia `b81c69985fd14bb7bfbd356b7618633ce07dc1a4`.
`tests/AsADashTest.cpp` source hash from the existing inventory (not recomputed):
`056e6a9fa8a90e0524870b6f5be20cc5e2a8cf5ba7cafa11549a35ee3d87f05f`.
License: `tests/LICENSE.skia`.

| Original registration | Original location | Rust registration in `tests/m08_test.rs` | Configuration |
| --- | --- | --- | --- |
| AsADashTest_noneDash | tests/AsADashTest.cpp:19 | AsADashTest_noneDash | CPU, no features |
| AsADashTest_nullInfo | tests/AsADashTest.cpp:27 | AsADashTest_nullInfo | CPU, no features |
| AsADashTest_usingDash | tests/AsADashTest.cpp:36 | AsADashTest_usingDash | CPU, no features |

All three are written and unexecuted. No reached helpers, resources, guards or
fixtures occur in the original bodies. `base()` adapts `as_PEB`; `Option`
adapts the optional DashInfo, and the existing Reporter retains nonfatal
assertions. Factory/optional unwraps adapt subsequent C++ dereferences.
The unused DashInfo local is explicitly initialized in Rust. All original
inputs and assertions retain their order and exact comparisons.

`supplemental.rs` separately checks rejected and boundary parameters, float
sum overflow, phase wrapping and the negative tiny-phase rounding boundary,
source interval copying, shared effect lifetime and independently owned
DashInfo lifetime. The packed stroke-record check covers all cap/join and
stroke-and-fill bits in native byte order (Apple Silicon is little endian).
Serialization and its wire byte order remain M22; this check concerns only
the M08 dense in-memory layout contract. Later registrations remain at the
gates listed in the milestone plan.

No tests, builds, references, discovery, formatting, validators or acceptance
workflows were run. Deepseek owns execution and acceptance updates.

Commands for Deepseek, from the repository root:

```sh
cargo test --locked --test m08_test
cargo test --release --locked --test m08_test
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```
