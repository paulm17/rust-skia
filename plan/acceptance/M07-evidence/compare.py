#!/usr/bin/env python3
"""Compare the M07 C++ reference dump with the Rust dump (numbers, integers)."""
import sys, re

def parse(path):
    out = []
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        if parts[0] == 'done':
            continue
        key = parts[0] + ((' ' + parts[1]) if parts[0] in ('meas1', 'meas2', 'cm') else '')
        nums = []
        for tok in parts[1:]:
            try:
                nums.append(float(tok))
            except ValueError:
                nums.append(tok)
        out.append((key, nums))
    return out

ref = parse(sys.argv[1])
rust = parse(sys.argv[2])
problems = 0
maxrel = 0.0
if len(ref) != len(rust):
    print(f'line count mismatch {len(ref)} vs {len(rust)}')
    problems += 1
for (k1, a), (k2, b) in zip(ref, rust):
    if k1 != k2:
        print(f'key mismatch {k1!r} vs {k2!r}')
        problems += 1
        continue
    if len(a) != len(b):
        print(f'{k1}: field count {len(a)} vs {len(b)}')
        problems += 1
        continue
    for x, y in zip(a, b):
        if isinstance(x, str) or isinstance(y, str):
            if x != y:
                print(f'{k1}: {x!r} vs {y!r}')
                problems += 1
            continue
        if float(x).is_integer() and float(y).is_integer():
            if int(x) != int(y):
                print(f'{k1}: int {x} vs {y}')
                problems += 1
            continue
        denom = max(abs(x), 1e-30)
        rel = abs(x - y) / denom
        maxrel = max(maxrel, rel)
        if rel > 1e-6:
            print(f'{k1}: {x} vs {y} (rel {rel:.3g})')
            problems += 1
print(f'compared {len(ref)} records, {problems} problems, max relative difference {maxrel:.3g}')
sys.exit(1 if problems else 0)
