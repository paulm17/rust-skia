# macOS scope and dependency boundaries

## Required coverage

- Native CPU rendering and libraries on Apple Silicon and Intel macOS. Current planning/initial-test host: arm64, macOS 15.6. Do not invent a lower deployment target; record and validate deployment versions during packaging.
- Ganesh Metal, Ganesh native macOS OpenGL/CGL, Graphite Metal, and Graphite Dawn explicitly using its Metal backend. Test each backend on a Mac that supports it and retain upstream capability guards.
- CoreText, CoreGraphics, Cocoa/macOS windows, macOS font discovery, and platform-neutral optional modules/backends that build on macOS (including FreeType/Fontations/ICU/HarfBuzz adapters).
- All macOS-eligible tools, examples, experiments, benchmarks, fuzzers, GMs and documentation. CanvasKit/WebAssembly bindings are retained, but browser tests run on macOS hosts only. They do not add Windows/Linux test coverage.
- Offline shader compiler outputs can be tested on macOS even when the corresponding GPU runtime is excluded. Preserve applicable HLSL/SPIR-V golden/codegen tests without requiring Direct3D/Vulkan hardware or runtime backends.

## Excluded execution and ports

Windows, Linux, Android, iOS, Fuchsia, their native APIs and launchers; Direct3D and Vulkan runtime backends; GLX/EGL/Wayland/X11 context implementations; non-Mac CPU architecture kernels. Keep portable POSIX services used by macOS. Shared files that contain non-Mac branches remain listed: port only macOS-reachable behavior and record branch exclusions with the original guards. A filename-based exclusion is an initial classification; confirm ambiguous shared files against the pinned build configuration before accepting their disposition.

`inventory/source-files.csv` explicitly records excluded files. Do not interpret complete repository accounting as a requirement to port excluded operating-system code. Existing JS/Python/Go orchestration, docs, licenses and immutable assets may remain in their original format, adapted to Rust outputs where necessary.

## External code

Third-party libraries are not part of the owned Skia rewrite. Track versions, licenses, feature flags, transitive requirements and macOS build results. Dependency adapters authored by Skia are part of the Rust port. Bundled skcms is explicitly included as owned port work in M10. Existing first-party Rust modules must be integrated so their Skia-facing policy no longer depends on C++ wrappers.

Apple framework FFI and external codec, shaping, compression, Dawn and validation libraries are permitted. C++ Skia itself and other Skia Rust ports are not implementation dependencies. C/C++ third-party dependency code does not invalidate the owned Rust Skia implementation, but must be disclosed in the final release inventory.

The DEPS extraction lists pinned URL references without fetching them. Conditional/CIPD entries and dependency transitives must be taken from DEPS and the relevant build files. Excluded dependencies such as Vulkan-only tooling must not be pulled in as required runtime coverage merely because they appear in upstream DEPS.
