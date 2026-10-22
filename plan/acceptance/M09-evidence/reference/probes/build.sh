#!/bin/bash
set -euo pipefail
SKIA=${1:-/Volumes/Data/Users/paul/development/src/github/skia}
OUT=$(cd "$(dirname "$0")" && pwd)
for probe in translate_saturation aaclip_quick_contains; do
  xcrun clang++ -std=c++20 -O1 -DNDEBUG -I"$SKIA" \
    "$OUT/$probe.cpp" "$SKIA/out/M09Ref/libskia.a" "$SKIA/out/M09Ref/libskcms.a" \
    -framework CoreFoundation -framework CoreGraphics -framework CoreText -framework Foundation \
    -o "$OUT/$probe" 2>&1 | tee "$OUT/$probe-build.log"
  "$OUT/$probe" > "$OUT/$probe.out" 2>&1
done
