# Milestone 40: Developer tools, inspection and debug clients

## Outcome and entry gate

Port first-party executable logic for debugging, tracing, SKP utilities, viewer, shader tools, image diff, font tools and command-line support. Preserve file formats, command-line semantics and output/error behavior; scripts may remain in their original language when they orchestrate Rust binaries.

Begin implementation only after milestone 39 is marked ACCEPTED by Codex and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Port first-party executable logic for debugging, tracing, SKP utilities, viewer, shader tools, image diff, font tools and command-line support. Preserve file formats, command-line semantics and output/error behavior; scripts may remain in their original language when they orchestrate Rust binaries.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures or pass records. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `tools/AutoreleasePool.h` — port to Rust.
- `tools/AutoreleasePool.mm` — port to Rust.
- `tools/CodecUtils.h` — port to Rust.
- `tools/CrashHandler.cpp` — port to Rust.
- `tools/CrashHandler.h` — port to Rust.
- `tools/DecodeUtils.cpp` — port to Rust.
- `tools/DecodeUtils.h` — port to Rust.
- `tools/EncodeUtils.cpp` — port to Rust.
- `tools/EncodeUtils.h` — port to Rust.
- `tools/GpuToolUtils.h` — port to Rust.
- `tools/HashAndEncode.cpp` — port to Rust.
- `tools/HashAndEncode.h` — port to Rust.
- `tools/LsanSuppressions.cpp` — port to Rust.
- `tools/MSKPPlayer.cpp` — port to Rust.
- `tools/MSKPPlayer.h` — port to Rust.
- `tools/OverwriteLine.h` — port to Rust.
- `tools/ProcStats.cpp` — port to Rust.
- `tools/ProcStats.h` — port to Rust.
- `tools/ProcsUtils.cpp` — port to Rust.
- `tools/ProcsUtils.h` — port to Rust.
- `tools/ResourceFactory.h` — port to Rust.
- `tools/RuntimeBlendUtils.cpp` — port to Rust.
- `tools/RuntimeBlendUtils.h` — port to Rust.
- `tools/SkMetaData.cpp` — port to Rust.
- `tools/SkMetaData.h` — port to Rust.
- `tools/SkSharingProc.cpp` — port to Rust.
- `tools/SkSharingProc.h` — port to Rust.
- `tools/Stats.h` — port to Rust.
- `tools/SvgPathExtractor.cpp` — port to Rust.
- `tools/SvgPathExtractor.h` — port to Rust.
- `tools/TsanSuppressions.cpp` — port to Rust.
- `tools/UrlDataManager.cpp` — port to Rust.
- `tools/UrlDataManager.h` — port to Rust.
- `tools/blob_cache_sim.cpp` — port to Rust.
- `tools/chrome_fuzz.cpp` — port to Rust.
- `tools/convert-to-nia.cpp` — port to Rust.
- `tools/debugger/DebugCanvas.cpp` — port to Rust.
- `tools/debugger/DebugCanvas.h` — port to Rust.
- `tools/debugger/DebugLayerManager.cpp` — port to Rust.
- `tools/debugger/DebugLayerManager.h` — port to Rust.
- `tools/debugger/DrawCommand.cpp` — port to Rust.
- `tools/debugger/DrawCommand.h` — port to Rust.
- `tools/debugger/JsonWriteBuffer.cpp` — port to Rust.
- `tools/debugger/JsonWriteBuffer.h` — port to Rust.
- `tools/dump_base.h` — port to Rust.
- `tools/dump_capture.cpp` — port to Rust.
- `tools/dump_record.cpp` — port to Rust.
- `tools/fiddle/all_examples.cpp` — port to Rust.
- `tools/fiddle/draw.cpp` — port to Rust.
- `tools/fiddle/egl_context.cpp` — port to Rust.
- `tools/fiddle/examples.cpp` — port to Rust.
- `tools/fiddle/examples.h` — port to Rust.
- `tools/fiddle/fiddle_main.cpp` — port to Rust.
- `tools/fiddle/fiddle_main.h` — port to Rust.
- `tools/fiddle/null_context.cpp` — port to Rust.
- `tools/flags/CommandLineFlags.cpp` — port to Rust.
- `tools/flags/CommandLineFlags.h` — port to Rust.
- `tools/flags/CommonFlags.h` — port to Rust.
- `tools/flags/CommonFlagsConfig.cpp` — port to Rust.
- `tools/flags/CommonFlagsConfig.h` — port to Rust.
- `tools/flags/CommonFlagsGanesh.cpp` — port to Rust.
- `tools/flags/CommonFlagsGanesh.h` — port to Rust.
- `tools/flags/CommonFlagsGraphite.cpp` — port to Rust.
- `tools/flags/CommonFlagsGraphite.h` — port to Rust.
- `tools/flags/CommonFlagsImages.cpp` — port to Rust.
- `tools/force_older_glibc_math.h` — port to Rust.
- `tools/get_current_monitor_profile.cpp` — port to Rust.
- `tools/get_images_from_skps.cpp` — port to Rust.
- `tools/image_diff_metric.cpp` — port to Rust.
- `tools/imgcvt.cpp` — port to Rust.
- `tools/ios_test_app_main.mm` — port to Rust.
- `tools/ios_utils.h` — port to Rust.
- `tools/ios_utils.m` — port to Rust.
- `tools/library/LoadDynamicLibrary.h` — port to Rust.
- `tools/library/LoadDynamicLibrary_posix.cpp` — port to Rust.
- `tools/public_headers_warnings_check.cpp` — port to Rust.
- `tools/skdiff/skdiff.cpp` — port to Rust.
- `tools/skdiff/skdiff.h` — port to Rust.
- `tools/skdiff/skdiff_html.cpp` — port to Rust.
- `tools/skdiff/skdiff_html.h` — port to Rust.
- `tools/skdiff/skdiff_main.cpp` — port to Rust.
- `tools/skdiff/skdiff_utils.cpp` — port to Rust.
- `tools/skdiff/skdiff_utils.h` — port to Rust.
- `tools/skottie2movie.cpp` — port to Rust.
- `tools/skp_parser.cpp` — port to Rust.
- `tools/skpbench/skpbench.cpp` — port to Rust.
- `tools/skpinfo.cpp` — port to Rust.
- `tools/skqp/src/jni_skqp.cpp` — port to Rust.
- `tools/skqp/src/skqp.cpp` — port to Rust.
- `tools/skqp/src/skqp.h` — port to Rust.
- `tools/skqp/src/skqp_GpuTestProcs.cpp` — port to Rust.
- `tools/skqp/src/skqp_main.cpp` — port to Rust.
- `tools/sksl-minify/SkSLMinify.cpp` — port to Rust.
- `tools/skslc/Main.cpp` — port to Rust.
- `tools/skslc/ProcessWorklist.cpp` — port to Rust.
- `tools/skslc/ProcessWorklist.h` — port to Rust.
- `tools/sksltrace/SkSLTraceUtils.cpp` — port to Rust.
- `tools/sksltrace/SkSLTraceUtils.h` — port to Rust.
- `tools/skui/InputState.h` — port to Rust.
- `tools/skui/Key.h` — port to Rust.
- `tools/skui/ModifierKey.h` — port to Rust.
- `tools/text/SkTextBlobTrace.cpp` — port to Rust.
- `tools/text/SkTextBlobTrace.h` — port to Rust.
- `tools/timer/TimeUtils.h` — port to Rust.
- `tools/timer/Timer.cpp` — port to Rust.
- `tools/timer/Timer.h` — port to Rust.
- `tools/trace/ChromeTracingTracer.cpp` — port to Rust.
- `tools/trace/ChromeTracingTracer.h` — port to Rust.
- `tools/trace/EventTracingPriv.cpp` — port to Rust.
- `tools/trace/EventTracingPriv.h` — port to Rust.
- `tools/trace/SkDebugfTracer.cpp` — port to Rust.
- `tools/trace/SkDebugfTracer.h` — port to Rust.
- `tools/trace/SkPerfettoTrace.cpp` — port to Rust.
- `tools/trace/SkPerfettoTrace.h` — port to Rust.
- `tools/unicode_comparison/cpp/bridge.cpp` — port to Rust.
- `tools/unicode_comparison/cpp/bridge.h` — port to Rust.
- `tools/using_skia_and_harfbuzz.cpp` — port to Rust.
- `tools/viewer/3DSlide.cpp` — port to Rust.
- `tools/viewer/AnimBlurSlide.cpp` — port to Rust.
- `tools/viewer/AnimTimer.h` — port to Rust.
- `tools/viewer/AnimatedImageSlide.cpp` — port to Rust.
- `tools/viewer/AnimatedImageSlide.h` — port to Rust.
- `tools/viewer/AnimatedRectsSlide.cpp` — port to Rust.
- `tools/viewer/AnimatedTextSlide.cpp` — port to Rust.
- `tools/viewer/ArcSlide.cpp` — port to Rust.
- `tools/viewer/AtlasSlide.cpp` — port to Rust.
- `tools/viewer/AudioSlide.cpp` — port to Rust.
- `tools/viewer/BisectSlide.cpp` — port to Rust.
- `tools/viewer/BisectSlide.h` — port to Rust.
- `tools/viewer/CameraSlide.cpp` — port to Rust.
- `tools/viewer/CaptureSlide.cpp` — port to Rust.
- `tools/viewer/CaptureSlide.h` — port to Rust.
- `tools/viewer/ChartSlide.cpp` — port to Rust.
- `tools/viewer/ChineseFlingSlide.cpp` — port to Rust.
- `tools/viewer/ClickHandlerSlide.cpp` — port to Rust.
- `tools/viewer/ClickHandlerSlide.h` — port to Rust.
- `tools/viewer/ClipSlide.cpp` — port to Rust.
- `tools/viewer/ClockSlide.cpp` — port to Rust.
- `tools/viewer/CowboySlide.cpp` — port to Rust.
- `tools/viewer/DegenerateQuadsSlide.cpp` — port to Rust.
- `tools/viewer/DegenerateTwoPtRadialsSlide.cpp` — port to Rust.
- `tools/viewer/EdgeBuilderVizSlide.cpp` — port to Rust.
- `tools/viewer/FatBitsSlide.cpp` — port to Rust.
- `tools/viewer/FilterBoundsSlide.cpp` — port to Rust.
- `tools/viewer/FitCubicToCircleSlide.cpp` — port to Rust.
- `tools/viewer/FlutterAnimateSlide.cpp` — port to Rust.
- `tools/viewer/GMSlide.cpp` — port to Rust.
- `tools/viewer/GMSlide.h` — port to Rust.
- `tools/viewer/GlyphTransformSlide.cpp` — port to Rust.
- `tools/viewer/GradientsSlide.cpp` — port to Rust.
- `tools/viewer/GraphitePrimitivesSlide.cpp` — port to Rust.
- `tools/viewer/ImGuiLayer.cpp` — port to Rust.
- `tools/viewer/ImGuiLayer.h` — port to Rust.
- `tools/viewer/ImageFilterDAGSlide.cpp` — port to Rust.
- `tools/viewer/ImageSlide.cpp` — port to Rust.
- `tools/viewer/ImageSlide.h` — port to Rust.
- `tools/viewer/LayersSlide.cpp` — port to Rust.
- `tools/viewer/MSKPSlide.cpp` — port to Rust.
- `tools/viewer/MSKPSlide.h` — port to Rust.
- `tools/viewer/ManyRectsSlide.cpp` — port to Rust.
- `tools/viewer/MaterialShadowsSlide.cpp` — port to Rust.
- `tools/viewer/MegaStrokeSlide.cpp` — port to Rust.
- `tools/viewer/MeshGradientSlide.cpp` — port to Rust.
- `tools/viewer/MeshSlide.cpp` — port to Rust.
- `tools/viewer/MixerSlide.cpp` — port to Rust.
- `tools/viewer/MotionMarkSlide.cpp` — port to Rust.
- `tools/viewer/PatchSlide.cpp` — port to Rust.
- `tools/viewer/PathClipSlide.cpp` — port to Rust.
- `tools/viewer/PathEffectsSlide.cpp` — port to Rust.
- `tools/viewer/PathLerpSlide.cpp` — port to Rust.
- `tools/viewer/PathOverstrokeSlide.cpp` — port to Rust.
- `tools/viewer/PathSlide.cpp` — port to Rust.
- `tools/viewer/PathTessellatorsSlide.cpp` — port to Rust.
- `tools/viewer/PathTextSlide.cpp` — port to Rust.
- `tools/viewer/ProtectedSlide.cpp` — port to Rust.
- `tools/viewer/QuadStrokerSlide.cpp` — port to Rust.
- `tools/viewer/RasterPipelineVizSlide.cpp` — port to Rust.
- `tools/viewer/RectanizerSlide.cpp` — port to Rust.
- `tools/viewer/RepeatTileSlide.cpp` — port to Rust.
- `tools/viewer/SBIXSlide.cpp` — port to Rust.
- `tools/viewer/SGSlide.cpp` — port to Rust.
- `tools/viewer/SKPSlide.cpp` — port to Rust.
- `tools/viewer/SKPSlide.h` — port to Rust.
- `tools/viewer/SVGFileSlide.cpp` — port to Rust.
- `tools/viewer/ShadowColorSlide.cpp` — port to Rust.
- `tools/viewer/ShadowReferenceSlide.cpp` — port to Rust.
- `tools/viewer/ShadowUtilsSlide.cpp` — port to Rust.
- `tools/viewer/ShipSlide.cpp` — port to Rust.
- `tools/viewer/SimpleStrokerSlide.cpp` — port to Rust.
- `tools/viewer/SkSLDebuggerSlide.cpp` — port to Rust.
- `tools/viewer/SkSLDebuggerSlide.h` — port to Rust.
- `tools/viewer/SkSLSlide.cpp` — port to Rust.
- `tools/viewer/SkSLSlide.h` — port to Rust.
- `tools/viewer/SkottieSlide.cpp` — port to Rust.
- `tools/viewer/SkottieSlide.h` — port to Rust.
- `tools/viewer/Slide.h` — port to Rust.
- `tools/viewer/SlideDir.cpp` — port to Rust.
- `tools/viewer/SlideDir.h` — port to Rust.
- `tools/viewer/SlidesSlide.cpp` — port to Rust.
- `tools/viewer/StatsLayer.cpp` — port to Rust.
- `tools/viewer/StatsLayer.h` — port to Rust.
- `tools/viewer/StringArtSlide.cpp` — port to Rust.
- `tools/viewer/StrokeVerbSlide.cpp` — port to Rust.
- `tools/viewer/SvgSlide.cpp` — port to Rust.
- `tools/viewer/SvgSlide.h` — port to Rust.
- `tools/viewer/TextBoxSlide.cpp` — port to Rust.
- `tools/viewer/TextureUploadSlide.cpp` — port to Rust.
- `tools/viewer/ThinAASlide.cpp` — port to Rust.
- `tools/viewer/TimingSlide.cpp` — port to Rust.
- `tools/viewer/TouchGesture.cpp` — port to Rust.
- `tools/viewer/TouchGesture.h` — port to Rust.
- `tools/viewer/TypefaceSlide.cpp` — port to Rust.
- `tools/viewer/VariableWidthStrokerSlide.cpp` — port to Rust.
- `tools/viewer/Viewer.cpp` — port to Rust.
- `tools/viewer/Viewer.h` — port to Rust.
- `tools/viewer/XferSlide.cpp` — port to Rust.
- `tools/viewer/ZoomInSlide.cpp` — port to Rust.
- `tools/viewer/ZoomInSlide.h` — port to Rust.
- `tools/win_lcid.cpp` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `tools/BUILD.bazel`.
- `tools/BUILD_simulator.py`.
- `tools/__init__.py`.
- `tools/abandon_gerrit_cls.py`.
- `tools/bloaty_treemap.py`.
- `tools/build_with_reclient.sh`.
- `tools/build_workaround_header.py`.
- `tools/calmbench/ab.py`.
- `tools/calmbench/calmbench.py`.
- `tools/clang-tidy.sh`.
- `tools/debugger/BUILD.bazel`.
- `tools/doxygen/footer.html`.
- `tools/embed_resources.py`.
- `tools/fiddle/animate.sh`.
- `tools/fiddle/make_all_examples_cpp.py`.
- `tools/find_run_binary.py`.
- `tools/fix_pythonpath.py`.
- `tools/flags/BUILD.bazel`.
- `tools/gdb/bitmap.py`.
- `tools/generate_patches.py`.
- `tools/infra/__init__.py`.
- `tools/infra/git.py`.
- `tools/infra/go.py`.
- `tools/install_dependencies.sh`.
- `tools/library/BUILD.bazel`.
- `tools/lottie-web-perf/lottie-web-canvas-perf.html`.
- `tools/lottie-web-perf/lottie-web-perf.html`.
- `tools/lottie-web-perf/lottie-web-perf.js`.
- `tools/lottie-web-perf/package-lock.json`.
- `tools/lottie-web-perf/package.json`.
- `tools/lottiecap/driver.html`.
- `tools/lottiecap/lottiecap.js`.
- `tools/lottiecap/package-lock.json`.
- `tools/lottiecap/package.json`.
- `tools/macsdk_dir.py`.
- `tools/malisc/malisc.py`.
- `tools/merge_static_libs.py`.
- `tools/milestone.py`.
- `tools/mirror-dev.sh`.
- `tools/misc_utils.py`.
- `tools/parse_llvm_coverage.py`.
- `tools/partition_alloc/BUILD.gn`.
- `tools/perf-canvaskit-puppeteer/benchmark.js`.
- `tools/perf-canvaskit-puppeteer/canvas_perf.html`.
- `tools/perf-canvaskit-puppeteer/canvas_perf.js`.
- `tools/perf-canvaskit-puppeteer/package-lock.json`.
- `tools/perf-canvaskit-puppeteer/package.json`.
- `tools/perf-canvaskit-puppeteer/path-transform.html`.
- `tools/perf-canvaskit-puppeteer/perf-canvaskit-with-puppeteer.js`.
- `tools/perf-canvaskit-puppeteer/perf_all_skps.sh`.
- `tools/perf-canvaskit-puppeteer/render-skp.html`.
- `tools/perf-canvaskit-puppeteer/skottie-frames.html`.
- `tools/perf-canvaskit-puppeteer/skp_data_prep.js`.
- `tools/raster_pipeline/llvm_mca_analysis.py`.
- `tools/raster_pipeline/run_benchmarks.py`.
- `tools/reformat-json.py`.
- `tools/rewrite_includes.py`.
- `tools/run-wasm-gm-tests/package-lock.json`.
- `tools/run-wasm-gm-tests/package.json`.
- `tools/run-wasm-gm-tests/run-wasm-gm-tests.html`.
- `tools/run-wasm-gm-tests/run-wasm-gm-tests.js`.
- `tools/sanitize_source_files.py`.
- `tools/serve_wasm.py`.
- `tools/skdiff/BUILD.bazel`.
- `tools/skia-client-search.html`.
- `tools/skottie-wasm-perf/package-lock.json`.
- `tools/skottie-wasm-perf/package.json`.
- `tools/skottie-wasm-perf/parse_perf_csvs.py`.
- `tools/skottie-wasm-perf/skottie-wasm-perf.html`.
- `tools/skottie-wasm-perf/skottie-wasm-perf.js`.
- `tools/skp/generate_page_set.py`.
- `tools/skp/page_sets/__init__.py`.
- `tools/skp/page_sets/skia_amazon_mobile.py`.
- `tools/skp/page_sets/skia_baidu_mobile.py`.
- `tools/skp/page_sets/skia_booking_mobile.py`.
- `tools/skp/page_sets/skia_capitalvolkswagen_mobile.py`.
- `tools/skp/page_sets/skia_carsvg_desktop.py`.
- `tools/skp/page_sets/skia_chalkboard_desktop.py`.
- `tools/skp/page_sets/skia_cnn_desktop.py`.
- `tools/skp/page_sets/skia_cnn_mobile.py`.
- `tools/skp/page_sets/skia_cnnarticle_mobile.py`.
- `tools/skp/page_sets/skia_css3gradients_desktop.py`.
- `tools/skp/page_sets/skia_deviantart_mobile.py`.
- `tools/skp/page_sets/skia_digg_tablet.py`.
- `tools/skp/page_sets/skia_ebay_desktop.py`.
- `tools/skp/page_sets/skia_espn_desktop.py`.
- `tools/skp/page_sets/skia_facebook_desktop.py`.
- `tools/skp/page_sets/skia_facebook_mobile.py`.
- `tools/skp/page_sets/skia_forecastio_mobile.py`.
- `tools/skp/page_sets/skia_gmail_desktop.py`.
- `tools/skp/page_sets/skia_googlecalendar_desktop.py`.
- `tools/skp/page_sets/skia_googledocs_desktop.py`.
- `tools/skp/page_sets/skia_googleimagesearch_desktop.py`.
- `tools/skp/page_sets/skia_googlenews_mobile.py`.
- `tools/skp/page_sets/skia_googlesearch_desktop.py`.
- `tools/skp/page_sets/skia_googlesearch_mobile.py`.
- `tools/skp/page_sets/skia_googlespreadsheet_desktop.py`.
- `tools/skp/page_sets/skia_gujuratiwiki_desktop.py`.
- `tools/skp/page_sets/skia_linkedin_desktop.py`.
- `tools/skp/page_sets/skia_mapsvg_desktop.py`.
- `tools/skp/page_sets/skia_micrographygirlsvg_desktop.py`.
- `tools/skp/page_sets/skia_motionmarkarcs_desktop.py`.
- `tools/skp/page_sets/skia_motionmarkpaths_desktop.py`.
- `tools/skp/page_sets/skia_motionmarksuits_desktop.py`.
- `tools/skp/page_sets/skia_motionmarksuitsclip_desktop.py`.
- `tools/skp/page_sets/skia_mozilla_tablet.py`.
- `tools/skp/page_sets/skia_nytimes_desktop.py`.
- `tools/skp/page_sets/skia_pravda_tablet.py`.
- `tools/skp/page_sets/skia_reddit_mobile.py`.
- `tools/skp/page_sets/skia_samoasvg_desktop.py`.
- `tools/skp/page_sets/skia_slashdot_mobile.py`.
- `tools/skp/page_sets/skia_techcrunch_mobile.py`.
- `tools/skp/page_sets/skia_theverge_desktop.py`.
- `tools/skp/page_sets/skia_theverge_mobile.py`.
- `tools/skp/page_sets/skia_tiger8svg_desktop.py`.
- `tools/skp/page_sets/skia_tigersvg_desktop.py`.
- `tools/skp/page_sets/skia_twitter_desktop.py`.
- `tools/skp/page_sets/skia_weather_desktop.py`.
- `tools/skp/page_sets/skia_wikipedia_desktop.py`.
- `tools/skp/page_sets/skia_wikipedia_mobile.py`.
- `tools/skp/page_sets/skia_worldjournal_tablet.py`.
- `tools/skp/page_sets/skia_wowwiki_desktop.py`.
- `tools/skp/page_sets/skia_yahooanswers_desktop.py`.
- `tools/skp/page_sets/skia_yahoosports_desktop.py`.
- `tools/skp/page_sets/skia_ynevsvg_desktop.py`.
- `tools/skp/page_sets/skia_youtube_desktop.py`.
- `tools/skp/page_sets/skia_youtube_mobile.py`.
- `tools/skp/webpages_playback.py`.
- `tools/skpbench/__init__.py`.
- `tools/skpbench/_adb.py`.
- `tools/skpbench/_adb_path.py`.
- `tools/skpbench/_benchresult.py`.
- `tools/skpbench/_hardware.py`.
- `tools/skpbench/_hardware_nexus_6p.py`.
- `tools/skpbench/_hardware_pixel.py`.
- `tools/skpbench/_hardware_pixel2.py`.
- `tools/skpbench/_hardware_pixel_c.py`.
- `tools/skpbench/_os_path.py`.
- `tools/skpbench/sheet.py`.
- `tools/skpbench/skiaperf.py`.
- `tools/skpbench/skpbench.py`.
- `tools/skqp/clean_app.sh`.
- `tools/skqp/create_apk.py`.
- `tools/skqp/docker_build_universal_apk.sh`.
- `tools/skqp/docker_run_apk.sh`.
- `tools/skqp/make_apk.sh`.
- `tools/skqp/make_universal_apk.py`.
- `tools/skqp/run_apk.sh`.
- `tools/skqp/test_apk.sh`.
- `tools/sksl-minify/BUILD.bazel`.
- `tools/skslc/BUILD.bazel`.
- `tools/skslc/compile_sksl.bzl`.
- `tools/sksltrace/BUILD.bazel`.
- `tools/skui/BUILD.bazel`.
- `tools/svg/svg_downloader.py`.
- `tools/test_all.py`.
- `tools/test_pdfs.py`.
- `tools/timer/BUILD.bazel`.
- `tools/trace/BUILD.bazel`.
- `tools/unicode_comparison/go/bridge/bridge.go`.
- `tools/unicode_comparison/go/download_wiki/main.go`.
- `tools/unicode_comparison/go/extract_info/main.go`.
- `tools/unicode_comparison/go/generate_table/main.go`.
- `tools/unicode_comparison/go/helpers/helpers.go`.
- `tools/unicode_comparison/html/index.html`.
- `tools/unicode_comparison/html/scripts.html`.
- `tools/unicode_comparison/html/styles.html`.
- `tools/unicode_comparison/html/tbody.html`.
- `tools/viewer/BUILD.bazel`.

## Codex: acceptance-test contract

Port tool-specific tests and compare CLI fixtures with C++ tools. Verify malformed input, exit codes, deterministic output and resource paths. Test viewer interactions on macOS and rerun relevant library tests to catch tooling integration regressions.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/DebugLayerManagerTest.cpp`.
- `tools/partition_alloc/TestSupport.cpp`.
- `tools/partition_alloc/TestSupport.h`.

### Original test registrations assigned to this gate

- `DebugLayerManagerTest` — `tests/DebugLayerManagerTest.cpp:87`; CPU; `DEF_TEST`.

### Preserve these fixtures, metadata and references

- `tools/check-headers-self-sufficient` — preserve.
- `tools/doxygen/Doxyfile` — preserve.
- `tools/doxygen/ProdDoxyfile` — preserve.
- `tools/doxygen/README.md` — preserve.
- `tools/doxygen/customdoxygen.css` — preserve.
- `tools/doxygen/logo.png` — preserve.
- `tools/doxygen/mainpage/mainpage.dox` — preserve.
- `tools/doxygen_footer.txt` — preserve.
- `tools/fiddle/.gitignore` — preserve.
- `tools/fiddle/parse-fiddle-output` — preserve.
- `tools/gcov_shim` — preserve.
- `tools/git-sync-deps` — preserve.
- `tools/lottie-web-perf/.gitignore` — preserve.
- `tools/lottiecap/.gitignore` — preserve.
- `tools/lottiecap/README.md` — preserve.
- `tools/pathops_sorter.htm` — preserve.
- `tools/pathops_visualizer.htm` — preserve.
- `tools/perf-canvaskit-puppeteer/Makefile` — preserve.
- `tools/perf-canvaskit-puppeteer/README.md` — preserve.
- `tools/perf-canvaskit-puppeteer/canvas_perf_assets/Roboto-Regular.ttf` — preserve.
- `tools/perf-canvaskit-puppeteer/canvas_perf_assets/Roboto-Regular.woff` — preserve.
- `tools/perf-canvaskit-puppeteer/canvas_perf_assets/Roboto-Regular.woff2` — preserve.
- `tools/perf-canvaskit-puppeteer/canvas_perf_assets/test_1500x959.jpg` — preserve.
- `tools/perf-canvaskit-puppeteer/canvas_perf_assets/test_512x512.png` — preserve.
- `tools/perf-canvaskit-puppeteer/canvas_perf_assets/test_64x64.png` — preserve.
- `tools/perf-canvaskit-puppeteer/path_translate_assets/car.svg` — preserve.
- `tools/raster_pipeline/README.md` — preserve.
- `tools/raster_pipeline/llvm_mca_analysis.png` — preserve.
- `tools/run-wasm-gm-tests/Makefile` — preserve.
- `tools/skottie-wasm-perf/.gitignore` — preserve.
- `tools/skottie-wasm-perf/Makefile` — preserve.
- `tools/skp/page_set_template` — preserve.
- `tools/skp/page_sets/data/README` — preserve.
- `tools/skp/page_sets/other/CRASHING-skia_vnexpress_nexus10.json` — preserve.
- `tools/skp/page_sets/other/CRASHING-skia_wikipedia_galaxynexus.json` — preserve.
- `tools/skp/page_sets/other/DOWN-skia_br337_desktop.json` — preserve.
- `tools/skp/page_sets/other/LAYERS-skia_yahoonews_desktop.json` — preserve.
- `tools/skp/page_sets/other/OLD-skia_desktop_set.json` — preserve.
- `tools/skp/page_sets/other/OLD-skia_galaxynexus_set.json` — preserve.
- `tools/skp/page_sets/other/OLD-skia_nexus10_set.json` — preserve.
- `tools/skp/page_sets/other/POPUP-skia_mercurynews_nexus10.json` — preserve.
- `tools/skp/page_sets/other/skia_maybe_later_set.json` — preserve.
- `tools/skp/page_sets/other/skia_onlinewsj_nexus10.json` — preserve.
- `tools/skpbench/README.md` — preserve.
- `tools/skqp/README.md` — preserve.
- `tools/skqp/make_universal_apk` — preserve.
- `tools/skqp/run_skqp_exe` — preserve.
- `tools/skqp/setup_resources` — preserve.
- `tools/skslc/Makefile` — preserve.
- `tools/svg/README.md` — preserve.
- `tools/svg/svg_images.txt` — preserve.
- `tools/svg/svgs.txt` — preserve.
- `tools/svg/svgs_parse_only.txt` — preserve.
- `tools/unicode_comparison/README.md` — preserve.
- `tools/unicode_comparison/go/Makefile` — preserve.
- `tools/unsafe_buffers/unsafe_buffers_mappings.txt` — preserve.
- `tools/xsan.supp` — preserve.

## Dependency and integration notes

The initial scan found 1012 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=40`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

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

Only after Codex marks M40 ACCEPTED may the user hand Deepseek M41.
