#!/usr/bin/env python3
"""Build/run the actual pinned CPU registrations; outputs never enter upstream."""
import argparse
import subprocess
import re
from pathlib import Path
parser = argparse.ArgumentParser()
parser.add_argument("--source", required=True)
parser.add_argument("--output", default="target/m01-acceptance")
args = parser.parse_args()
source = Path(args.source).resolve()
out = Path(args.output).resolve()
out.mkdir(parents=True, exist_ok=True)
SOURCES = ['tests/ScalarTest.cpp', 'tests/PointTest.cpp', 'tests/TestTest.cpp', 'tests/CtsEnforcement.cpp', 'src/core/SkPoint.cpp', 'src/core/SkScalar.cpp', 'src/core/SkFloatingPoint.cpp', 'src/core/SkRect.cpp', 'src/core/SkString.cpp', 'src/core/SkMalloc.cpp', 'src/core/SkDebug.cpp', 'src/ports/SkMemory_malloc.cpp', 'src/core/SkLog.cpp', 'src/ports/SkLog_stdio.cpp', 'src/core/SkSafeMath.cpp', 'src/utils/SkLogHandler.cpp']
def run(name, command):
    result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (out / (name + ".log")).write_text("COMMAND: " + repr(command) + "\n" + result.stdout + "\nEXIT: " + str(result.returncode) + "\n")
    if name.startswith("reference-run-") and not result.returncode:
        names = re.findall(r"^REGISTERED (\w+) CPU$", result.stdout, re.M)
        expected = {"Scalar", "Point", "Point_setLengthFast", "TestNormal", "TestTrueCondition", "TestCtsEnforcement"}
        if len(names) != 6 or set(names) != expected:
            raise RuntimeError("Reference registration mismatch: " + repr(names))
    print(name, "exit", result.returncode, flush=True)
    return result.returncode
failed = False
for profile, flags in [("debug", ["-O0", "-g", "-DSK_DEBUG"]), ("release", ["-O3", "-DNDEBUG"])]:
    binary = str(out / ("reference-" + profile))
    command = ["xcrun", "clang++", "-std=c++20", "-Wl,-dead_strip", "-DSK_DISABLE_TRACING", "-I" + str(source)] + flags + ["tests/reference/m01.cpp", "tests/reference/contracts.cpp"] + [str(source / s) for s in SOURCES] + ["-o", binary]
    code = run("reference-build-" + profile, command)
    failed |= bool(code)
    if not code:
        failed |= bool(run("reference-run-" + profile, [binary]))
raise SystemExit(int(failed))
