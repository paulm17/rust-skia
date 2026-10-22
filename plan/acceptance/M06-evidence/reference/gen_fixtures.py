import math
import random

random.seed(20261022)

lines = []

# Deterministic cubic curves, including degenerate ones (point, line, cusp, loop, serpentine).
cubics = [
    [(0, 0), (1, 1), (1, 0), (0, 1)],
    [(0, 0), (1, 0), (0, 1), (1, 1)],
    [(0, 0), (0, 1), (0, 2), (0, 3)],
    [(0, 0), (6, 2), (10, 2), (12, 0)],
    [(0, 0), (1, 1), (2, 2), (3, 3)],
    [(0, 0), (0, 0), (10, 10), (10, 10)],
    [(149.325, 107.705), (149.325, 103.783), (151.638, 100.127), (156.263, 96.736)],
    [(635.625, 614.687), (171.625, 236.188), (1064.62, 135.688), (516.625, 570.187)],
    [(4.873, 5.581), (5.083, 5.2783), (5.182, 4.8593), (5.177, 4.3242)],
    [(2190.0, 5130.0), (2190.0, 5070.0), (2220.0, 5010.0), (2205.0, 4980.0)],
]
for _ in range(30):
    cubics.append([(random.uniform(-100, 100), random.uniform(-100, 100)) for _ in range(4)])
for curve in cubics:
    lines.append("cubic " + " ".join(f"{x!r} {y!r}" for x, y in curve))

quads = [
    [(0, 0), (10, 20), (20, 0)],
    [(0, 0), (5, -5), (10, 0)],
    [(0, 0), (5, 0), (10, 0)],
]
for _ in range(15):
    quads.append([(random.uniform(-100, 100), random.uniform(-100, 100)) for _ in range(3)])
for curve in quads:
    lines.append("quad " + " ".join(f"{x!r} {y!r}" for x, y in curve))

conics = []
for _ in range(15):
    conics.append(([(random.uniform(-100, 100), random.uniform(-100, 100)) for _ in range(3)],
                   random.choice([0.1, 0.5, 1.0, 2.0, 10.0, random.uniform(0.01, 20.0)])))
conics.append(([(0, 100), (0, 0), (100, 0)], 1.0))
conics.append(([(0, 100), (0, 0), (100, 0)], 0.5))
for curve, w in conics:
    lines.append("conic " + " ".join(f"{x!r} {y!r}" for x, y in curve) + f" {w!r}")

# Simple radial polygons (angles sorted) plus fixed convex/non-convex shapes.
polys = []
for _ in range(4):
    start = random.uniform(0, 2 * math.pi)
    step = 2 * math.pi / 8
    pts = []
    for i in range(8):
        a = start + i * step
        r = random.uniform(20, 60)
        pts.append((r * math.cos(a), r * math.sin(a)))
    polys.append(pts)
polys.append([
    (-100.0, 55.0), (100.0, 55.0), (100.0, 50.0), (105.0, 50.0), (105.0, -50.0),
    (100.0, -50.0), (100.0, -55.0), (-100.0, -55.0), (-105.0, -50.0), (-105.0, 50.0),
])
polys.append([(2, 0), (1, 1), (-1, 1), (-2, 0), (-1, -1), (1, -1)])
for poly in polys:
    lines.append("poly " + str(len(poly)) + " " + " ".join(f"{x!r} {y!r}" for x, y in poly))

with open("target/m06-evidence/fixtures.txt", "w") as f:
    f.write("\n".join(lines) + "\n")
print("wrote", len(lines), "records")
