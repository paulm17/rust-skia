#!/bin/bash
# Build/run the unchanged pinned ClipStack and RTree registrations.
set -euo pipefail
SKIA=${1:-/Volumes/Data/Users/paul/development/src/github/skia}
OUT=$(cd "$(dirname "$0")" && pwd)
for profile in debug release; do
  if [ "$profile" = debug ]; then OPT=(-O0 -g); else OPT=(-O2); fi
  xcrun clang++ -std=c++20 "${OPT[@]}" -DNDEBUG -Wl,-dead_strip -DSK_DISABLE_TRACING \
    -I"$SKIA" "$OUT/m09_ref.cpp" "$SKIA/tests/ClipStackTest.cpp" "$SKIA/tests/RTreeTest.cpp" \
    "$SKIA/out/M09Ref/libskia.a" "$SKIA/out/M09Ref/libskcms.a" \
    -framework CoreFoundation -framework CoreGraphics -framework CoreText -framework Foundation \
    -o "$OUT/reference-$profile" 2>&1 | tee "$OUT/build-$profile.log"
  "$OUT/reference-$profile" > "$OUT/run-$profile.log" 2>&1 || { echo "reference $profile FAILED"; exit 1; }
done
