import sys


def tokens(line):
    return line.split()


def is_number(tok):
    try:
        float(tok)
        return True
    except ValueError:
        return False


ref = open("target/m06-evidence/ref.txt").read().splitlines()
rust = open("target/m06-evidence/rust.txt").read().splitlines()

if len(ref) != len(rust):
    print(f"line-count mismatch: ref={len(ref)} rust={len(rust)}")
    for i in range(min(len(ref), len(rust))):
        if ref[i] != rust[i]:
            print(f"first divergent line {i + 1}:\n  ref : {ref[i][:200]}\n  rust: {rust[i][:200]}")
            break
    sys.exit(1)

problems = 0
max_rel = 0.0
for lineno, (a_line, b_line) in enumerate(zip(ref, rust), 1):
    a = tokens(a_line)
    b = tokens(b_line)
    if a[:1] != b[:1]:
        print(f"line {lineno}: tag mismatch {a[:1]} vs {b[:1]}")
        problems += 1
        continue
    if len(a) != len(b):
        print(f"line {lineno}: length {len(a)} vs {len(b)}: {a_line[:120]} | {b_line[:120]}")
        problems += 1
        continue
    for i, (x, y) in enumerate(zip(a, b)):
        if i == 0:
            continue
        if x == y:
            continue
        if not (is_number(x) and is_number(y)):
            print(f"line {lineno}: token {i} text mismatch {x!r} vs {y!r}")
            problems += 1
            break
        fx = float(x)
        fy = float(y)
        scale = max(abs(fx), abs(fy), 1.0)
        diff = abs(fx - fy)
        rel = diff / scale
        if rel > max_rel:
            max_rel = rel
        tolerance = 2e-6
        if abs(fx) > 1e6 or abs(fy) > 1e6:
            tolerance = 2e-5
        if diff > tolerance * scale:
            print(f"line {lineno}: value {i} differs: {fx!r} vs {fy!r} (rel {rel:.3g})")
            print(f"  ref : {a_line[:200]}")
            print(f"  rust: {b_line[:200]}")
            problems += 1
            break

print(f"compared {len(ref)} lines, {problems} problems, max relative difference {max_rel:.3g}")
sys.exit(1 if problems else 0)
