#!/bin/bash
# Rebuild the M08 asADash reference probe from the pinned C++ checkout.
# Usage: ./build.sh /Volumes/Data/Users/paul/development/src/github/skia
set -euo pipefail
SKIA="${1:?usage: build.sh <pinned skia checkout>}"
HERE="$(cd "$(dirname "$0")" && pwd)"
OBJ="$HERE/obj"
mkdir -p "$OBJ"
CXXFLAGS=(-std=c++20 -O2 -fno-exceptions -fno-rtti -I"$SKIA")
TUS=(
  src/core/SkBezierCurves.cpp src/core/SkBuffer.cpp src/core/SkContainers.cpp
  src/core/SkContourMeasure.cpp src/core/SkCubics.cpp src/core/SkDebug.cpp
  src/core/SkEdgeClipper.cpp src/core/SkFlattenable.cpp src/core/SkFloatingPoint.cpp
  src/core/SkGeometry.cpp src/core/SkIDChangeListener.cpp src/core/SkLineClipper.cpp
  src/core/SkLog.cpp src/core/SkMalloc.cpp src/core/SkMatrix.cpp src/core/SkPaint.cpp
  src/core/SkPath.cpp src/core/SkPathBuilder.cpp src/core/SkPathData.cpp
  src/core/SkPathDump.cpp src/core/SkPathEffect.cpp src/core/SkPathIter.cpp
  src/core/SkPathMeasure.cpp src/core/SkPathPriv.cpp src/core/SkPathRaw.cpp
  src/core/SkPathRawShapes.cpp src/core/SkPath_interpolate.cpp src/core/SkPath_serial.cpp
  src/core/SkPoint.cpp src/core/SkQuads.cpp src/core/SkRRect.cpp src/core/SkReadBuffer.cpp
  src/core/SkRect.cpp src/core/SkSafeMath.cpp src/core/SkSemaphore.cpp src/core/SkString.cpp
  src/core/SkStringUtils.cpp src/core/SkStroke.cpp src/core/SkStrokeRec.cpp
  src/core/SkStrokerPriv.cpp src/core/SkTDArray.cpp src/core/SkWriteBuffer.cpp
  src/core/SkWriter32.cpp src/effects/Sk1DPathEffect.cpp src/effects/SkCornerPathEffect.cpp
  src/effects/SkDashPathEffect.cpp src/effects/SkDiscretePathEffect.cpp
  src/effects/SkTrimPathEffect.cpp src/utils/SkDashPath.cpp src/utils/SkLogHandler.cpp
  src/ports/SkLog_stdio.cpp src/ports/SkMemory_malloc.cpp
)
for tu in "${TUS[@]}"; do
  out="$OBJ/$(basename "${tu%.cpp}").o"
  clang++ "${CXXFLAGS[@]}" -c "$SKIA/$tu" -o "$out"
done
clang++ "${CXXFLAGS[@]}" -c "$HERE/ref_as_a_dash.cpp" -o "$OBJ/ref_as_a_dash.o"
clang++ -Wl,-dead_strip -o "$HERE/ref_as_a_dash" "$OBJ"/*.o
echo "built $HERE/ref_as_a_dash from $SKIA"
