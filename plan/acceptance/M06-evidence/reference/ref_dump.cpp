// M06 differential reference dumper. Prints the C++ reference results for the
// shared fixture file; the Rust twin (tests/_m06_diff.rs) prints the same layout.
#include "include/core/SkCubicMap.h"
#include "include/core/SkPoint.h"
#include "include/core/SkRect.h"
#include "include/core/SkScalar.h"
#include "include/private/SkTDArray.h"
#include "src/core/SkBezierCurves.h"
#include "src/core/SkCubicClipper.h"
#include "src/core/SkCubics.h"
#include "src/core/SkEdgeClipper.h"
#include "src/core/SkGeometry.h"
#include "src/core/SkLineClipper.h"
#include "src/core/SkQuadClipper.h"
#include "src/core/SkQuads.h"
#include "src/utils/SkPolyUtils.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

static void pf(const char* tag, float v) { printf("%s %.9g\n", tag, v); }
static void pd(const char* tag, double v) { printf("%s %.17g\n", tag, v); }
static void pi(const char* tag, int v) { printf("%s %d\n", tag, v); }

static void dump_points(const char* tag, const SkPoint* pts, int count) {
    printf("%s %d", tag, count);
    for (int i = 0; i < count; ++i) {
        printf(" %.9g %.9g", pts[i].fX, pts[i].fY);
    }
    printf("\n");
}

static void dump_cubic(const SkPoint p[4], int index) {
    printf("C %d\n", index);
    double t[2], s[2], d[4];
    int type = (int)SkClassifyCubic(p, t, s, d);
    pi("type", type);
    printf("ts %.17g %.17g %.17g %.17g\n", t[0], t[1], s[0], s[1]);
    printf("d %.17g %.17g %.17g %.17g\n", d[0], d[1], d[2], d[3]);

    SkScalar tValues[3];
    int inflections = SkFindCubicInflections(p, tValues);
    printf("inf %d", inflections);
    for (int i = 0; i < inflections; ++i) printf(" %.9g", tValues[i]);
    printf("\n");

    int maxCurvature = SkFindCubicMaxCurvature(p, tValues);
    printf("mc %d", maxCurvature);
    for (int i = 0; i < maxCurvature; ++i) printf(" %.9g", tValues[i]);
    printf("\n");

    pf("cusp", SkFindCubicCusp(p));

    SkPoint dst[13];
    int chopped = SkChopCubicAtInflections(p, dst);
    dump_points("ichop", dst, chopped * 3 + 1);
    chopped = SkChopCubicAtMaxCurvature(p, dst);
    dump_points("mchop", dst, chopped * 3 + 1);

    int yExtrema = SkChopCubicAtYExtrema(p, dst);
    dump_points("yext", dst, yExtrema * 3 + 1);
    int xExtrema = SkChopCubicAtXExtrema(p, dst);
    dump_points("xext", dst, xExtrema * 3 + 1);

    SkRect bounds = SkRect::BoundsOrEmpty({p, 4});
    SkPoint mono[7];
    bool choppedY = SkChopMonoCubicAtY(p, bounds.centerY(), mono);
    printf("monoy %d", choppedY ? 1 : 0);
    if (choppedY) {
        for (int i = 0; i < 7; ++i) printf(" %.9g %.9g", mono[i].fX, mono[i].fY);
    }
    printf("\n");
    bool choppedX = SkChopMonoCubicAtX(p, bounds.centerX(), mono);
    printf("monox %d", choppedX ? 1 : 0);
    if (choppedX) {
        for (int i = 0; i < 7; ++i) printf(" %.9g %.9g", mono[i].fX, mono[i].fY);
    }
    printf("\n");

    const SkRect clip = SkRect::MakeLTRB(0, 0, 10, 10);
    SkPoint lines[SkLineClipper::kMaxPoints];
    SkPoint endpoints[2] = {p[0], p[3]};
    SkPoint intersection[2];
    bool intersects = SkLineClipper::IntersectLine(endpoints, clip, intersection);
    printf("intersect %d", intersects ? 1 : 0);
    if (intersects) {
        printf(" %.9g %.9g %.9g %.9g", intersection[0].fX, intersection[0].fY,
               intersection[1].fX, intersection[1].fY);
    }
    printf("\n");
    int segmentCount = SkLineClipper::ClipLine(endpoints, clip, lines, false);
    if (segmentCount > 0) {
        dump_points("clip", lines, segmentCount + 1);
    } else {
        printf("clip 0\n");
    }

    SkEdgeClipper clipper(false);
    bool clipped = clipper.clipCubic(p, clip);
    printf("edge %d", clipped ? 1 : 0);
    if (clipped) {
        SkPoint verbPts[4];
        for (;;) {
            auto verb = clipper.next(verbPts);
            if (!verb) break;
            int verbInt = (int)*verb;
            printf(" %d", verbInt);
            int n = 0;
            switch (*verb) {
                case SkPathVerb::kMove: n = 1; break;
                case SkPathVerb::kLine: n = 2; break;
                case SkPathVerb::kQuad: n = 3; break;
                case SkPathVerb::kCubic: n = 4; break;
                default: break;
            }
            for (int i = 0; i < n; ++i) printf(" %.9g %.9g", verbPts[i].fX, verbPts[i].fY);
        }
    }
    printf("\n");

    SkEdgeClipper lineClipper(false);
    bool lineClipped = lineClipper.clipLine(p[0], p[1], clip);
    printf("edgeline %d", lineClipped ? 1 : 0);
    if (lineClipped) {
        SkPoint verbPts[4];
        for (;;) {
            auto verb = lineClipper.next(verbPts);
            if (!verb) break;
            printf(" %d", (int)*verb);
            int n = (*verb == SkPathVerb::kLine) ? 2 : 3;
            for (int i = 0; i < n; ++i) printf(" %.9g %.9g", verbPts[i].fX, verbPts[i].fY);
        }
    }
    printf("\n");

    SkScalar chopT = -1;
    bool chopOk = SkCubicClipper::ChopMonoAtY(p, bounds.centerY(), &chopT);
    printf("cubicchop %d %.17g\n", chopOk ? 1 : 0, chopT);

    SkCubicClipper cubicClipper;
    cubicClipper.setClip(SkIRect::MakeLTRB(0, 0, 10, 10));
    SkPoint clippedCubic[4];
    bool ok = cubicClipper.clipCubic(p, clippedCubic);
    printf("cubicclip %d", ok ? 1 : 0);
    if (ok) {
        for (int i = 0; i < 4; ++i) printf(" %.9g %.9g", clippedCubic[i].fX, clippedCubic[i].fY);
    }
    printf("\n");

    // Bezier cubic utilities over the same control points.
    double curve[8] = {p[0].fX, p[0].fY, p[1].fX, p[1].fY,
                       p[2].fX, p[2].fY, p[3].fX, p[3].fY};
    auto eval = SkBezierCubic::EvalAt(curve, 0.37);
    printf("beval %.17g %.17g\n", eval[0], eval[1]);
    double two[14];
    SkBezierCubic::Subdivide(curve, 0.37, two);
    printf("bsub");
    for (double v : two) printf(" %.17g", v);
    printf("\n");
    auto polyX = SkBezierCubic::ConvertToPolynomial(curve, false);
    auto polyY = SkBezierCubic::ConvertToPolynomial(curve, true);
    printf("bpoly %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g\n",
           polyX[0], polyX[1], polyX[2], polyX[3], polyY[0], polyY[1], polyY[2], polyY[3]);
    // Midtangent, rotation, angles and the cubic map over the same points.
    float angle = SkMeasureAngleBetweenVectors(p[1] - p[0], p[2] - p[1]);
    pf("angle", angle);
    SkVector bisector = SkFindBisector(p[1] - p[0], p[2] - p[1]);
    printf("bisector %.9g %.9g\n", bisector.fX, bisector.fY);
    pf("rot", SkMeasureNonInflectCubicRotation(p));
    float midTangent = SkFindCubicMidTangent(p);
    pf("midtangent", midTangent);
    SkPoint midChopped[7];
    SkChopCubicAtMidTangent(p, midChopped);
    dump_points("midchop", midChopped, 7);

    SkCubicMap cubicMap(p[0], p[1]);
    printf("cmap %.9g %.9g %.9g\n", cubicMap.computeYFromX(0.1f),
           cubicMap.computeYFromX(0.5f), cubicMap.computeYFromX(0.9f));
    SkPoint fromT = cubicMap.computeFromT(0.37f);
    printf("cmapfromt %.9g %.9g\n", fromT.fX, fromT.fY);

    float storage[3];
    auto cubicHits = SkBezierCubic::IntersectWithHorizontalLine({p, 4}, 0.0f, storage);
    printf("bhit %d", (int)cubicHits.size());
    for (float v : cubicHits) printf(" %.9g", v);
    printf("\n");
}

static void dump_quad(const SkPoint p[3], int index) {
    printf("Q %d\n", index);
    pf("maxcurv", SkFindQuadMaxCurvature(p));
    pf("midtan", SkFindQuadMidTangent(p));
    SkPoint dst[5];
    int chopped = SkChopQuadAtYExtrema(p, dst);
    dump_points("ychop", dst, chopped * 2 + 1);
    chopped = SkChopQuadAtXExtrema(p, dst);
    dump_points("xchop", dst, chopped * 2 + 1);
    chopped = SkChopQuadAtMaxCurvature(p, dst);
    dump_points("mchop", dst, chopped * 2 + 1);
    SkPoint midChopped[5];
    SkChopQuadAtMidTangent(p, midChopped);
    dump_points("qmidchop", midChopped, 5);
    const SkRect clip = SkRect::MakeLTRB(0, 0, 10, 10);
    SkQuadClipper quadClipper;
    quadClipper.setClip(SkIRect::MakeLTRB(0, 0, 10, 10));
    SkPoint clipped[3];
    bool ok = quadClipper.clipQuad(p, clipped);
    printf("qclip %d", ok ? 1 : 0);
    if (ok) {
        for (int i = 0; i < 3; ++i) printf(" %.9g %.9g", clipped[i].fX, clipped[i].fY);
    }
    printf("\n");
    SkEdgeClipper clipper(false);
    bool edgeClipped = clipper.clipQuad(p, clip);
    printf("qedge %d", edgeClipped ? 1 : 0);
    if (edgeClipped) {
        SkPoint verbPts[4];
        for (;;) {
            auto verb = clipper.next(verbPts);
            if (!verb) break;
            printf(" %d", (int)*verb);
            int n = (*verb == SkPathVerb::kLine) ? 2 : 3;
            for (int i = 0; i < n; ++i) printf(" %.9g %.9g", verbPts[i].fX, verbPts[i].fY);
        }
    }
    printf("\n");

    double curve[8] = {p[0].fX, p[0].fY, p[1].fX, p[1].fY, p[2].fX, p[2].fY, 0, 0};
    float storage[2];
    auto hits = SkBezierQuad::IntersectWithHorizontalLine({p, 3}, 0.0f, storage);
    printf("qhit %d", (int)hits.size());
    for (float v : hits) printf(" %.9g", v);
    printf("\n");
}

static void dump_conic(const SkPoint p[3], float w, int index) {
    printf("K %d\n", index);
    SkConic conic;
    conic.set(p, w);
    pf("w", conic.fW);
    pi("pow2_1", conic.computeQuadPOW2(1.0f));
    pi("pow2_tenth", conic.computeQuadPOW2(0.1f));
    SkPoint pos;
    SkVector tangent;
    conic.evalAt(0.37f, &pos, &tangent);
    printf("eval %.9g %.9g %.9g %.9g\n", pos.fX, pos.fY, tangent.fX, tangent.fY);

    SkConic halves[2];
    bool chopped = conic.chopAt(0.37f, halves);
    printf("chop %d", chopped ? 1 : 0);
    if (chopped) {
        for (int i = 0; i < 2; ++i) {
            for (int j = 0; j < 3; ++j) printf(" %.9g %.9g", halves[i].fPts[j].fX, halves[i].fPts[j].fY);
            printf(" %.9g", halves[i].fW);
        }
    }
    printf("\n");

    int pow2 = conic.computeQuadPOW2(1.0f);
    if (pow2 <= 10) {
        std::vector<SkPoint> quads(1 + 2 * (1 << pow2));
        int quadCount = conic.chopIntoQuadsPOW2(quads.data(), pow2);
        printf("qpow %d", quadCount);
        for (int i = 0; i < 1 + 2 * quadCount; ++i) printf(" %.9g %.9g", quads[i].fX, quads[i].fY);
        printf("\n");
    } else {
        printf("qpow skipped\n");
    }

    SkConic choppedTwice;
    conic.chopAt(0.3f, 0.6f, &choppedTwice);
    printf("chop2");
    for (int j = 0; j < 3; ++j) printf(" %.9g %.9g", choppedTwice.fPts[j].fX, choppedTwice.fPts[j].fY);
    printf(" %.9g\n", choppedTwice.fW);

    SkConic xHalves[2], yHalves[2];
    bool xChop = conic.chopAtXExtrema(xHalves);
    printf("xchop %d", xChop ? 1 : 0);
    if (xChop) {
        for (int i = 0; i < 2; ++i) {
            for (int j = 0; j < 3; ++j) printf(" %.9g %.9g", xHalves[i].fPts[j].fX, xHalves[i].fPts[j].fY);
        }
    }
    printf("\n");
    bool yChop = conic.chopAtYExtrema(yHalves);
    printf("ychopc %d", yChop ? 1 : 0);
    if (yChop) {
        for (int i = 0; i < 2; ++i) {
            for (int j = 0; j < 3; ++j) printf(" %.9g %.9g", yHalves[i].fPts[j].fX, yHalves[i].fPts[j].fY);
        }
    }
    printf("\n");

    SkVector quadError;
    conic.computeAsQuadError(&quadError);
    printf("quaderr %.9g %.9g\n", quadError.fX, quadError.fY);
    pi("quadtol1", conic.asQuadTol(1.0f) ? 1 : 0);
    pi("quadtol01", conic.asQuadTol(0.1f) ? 1 : 0);
    pf("transformw", SkConic::TransformW(p, conic.fW, SkMatrix::Scale(2.0f, 3.0f)));

    pf("midtan", conic.findMidTangent());
    SkScalar t = -1;
    bool xExt = conic.findXExtrema(&t);
    printf("xext %d %.9g\n", xExt ? 1 : 0, t);
    t = -1;
    bool yExt = conic.findYExtrema(&t);
    printf("yext %d %.9g\n", yExt ? 1 : 0, t);
    SkRect tight;
    conic.computeTightBounds(&tight);
    printf("tight %.9g %.9g %.9g %.9g\n", tight.fLeft, tight.fTop, tight.fRight, tight.fBottom);
}

static void dump_poly(const std::vector<SkPoint>& pts, int index) {
    printf("P %d\n", index);
    int size = (int)pts.size();
    pi("winding", SkGetPolygonWinding(pts.data(), size));
    pi("convex", SkIsConvexPolygon(pts.data(), size) ? 1 : 0);
    pi("simple", SkIsSimplePolygon(pts.data(), size) ? 1 : 0);

    SkScalar rotSin = 0, rotCos = 0;
    int steps = 0;
    bool radialOk = SkComputeRadialSteps(SkVector{1, 0}, SkVector{0, 1}, 5.0f,
                                         &rotSin, &rotCos, &steps);
    printf("radial %d %.9g %.9g %d\n", radialOk ? 1 : 0, rotSin, rotCos, steps);

    SkTDArray<SkPoint> inset;
    bool insetOk = SkInsetConvexPolygon(pts.data(), size, 5.0f, &inset);
    printf("inset %d", insetOk ? 1 : 0);
    if (insetOk) {
        for (int i = 0; i < inset.size(); ++i) printf(" %.9g %.9g", inset[i].fX, inset[i].fY);
    }
    printf("\n");

    SkRect bounds;
    std::vector<SkPoint> boundsSrc = pts;
    bounds = SkRect::BoundsOrEmpty({boundsSrc.data(), size});
    for (float offset : {5.0f, -5.0f}) {
        SkTDArray<SkPoint> offsetPoly;
        SkTDArray<int> indices;
        bool offsetOk = SkOffsetSimplePolygon(pts.data(), size, bounds, offset, &offsetPoly, &indices);
        printf("offset %d", offsetOk ? 1 : 0);
        if (offsetOk) {
            for (int i = 0; i < offsetPoly.size(); ++i) {
                printf(" %.9g %.9g", offsetPoly[i].fX, offsetPoly[i].fY);
            }
            printf(" idx");
            for (int i = 0; i < indices.size(); ++i) printf(" %d", indices[i]);
        }
        printf("\n");
    }

    std::vector<uint16_t> indexMap(size);
    for (int i = 0; i < size; ++i) indexMap[i] = (uint16_t)i;
    SkTDArray<uint16_t> triangles;
    bool triOk = SkTriangulateSimplePolygon(pts.data(), indexMap.data(), size, &triangles);
    printf("tri %d", triOk ? 1 : 0);
    if (triOk) {
        printf(" %d", triangles.size());
        for (int i = 0; i < triangles.size(); ++i) printf(" %d", (int)triangles[i]);
    }
    printf("\n");
}

int main(int argc, char** argv) {
    setvbuf(stdout, nullptr, _IONBF, 0);
    const char* path = argc > 1 ? argv[1] : "target/m06-evidence/fixtures.txt";
    FILE* file = fopen(path, "r");
    if (!file) {
        fprintf(stderr, "cannot open %s\n", path);
        return 1;
    }
    char kind[32];
    int index = 0;
    while (fscanf(file, "%31s", kind) == 1) {
        if (strcmp(kind, "cubic") == 0) {
            SkPoint p[4];
            for (int i = 0; i < 4; ++i) {
                fscanf(file, "%f %f", &p[i].fX, &p[i].fY);
            }
            dump_cubic(p, index++);
        } else if (strcmp(kind, "quad") == 0) {
            SkPoint p[3];
            for (int i = 0; i < 3; ++i) {
                fscanf(file, "%f %f", &p[i].fX, &p[i].fY);
            }
            dump_quad(p, index++);
        } else if (strcmp(kind, "conic") == 0) {
            SkPoint p[3];
            float w = 1;
            for (int i = 0; i < 3; ++i) {
                fscanf(file, "%f %f", &p[i].fX, &p[i].fY);
            }
            fscanf(file, "%f", &w);
            dump_conic(p, w, index++);
        } else if (strcmp(kind, "poly") == 0) {
            int n = 0;
            fscanf(file, "%d", &n);
            std::vector<SkPoint> pts(n);
            for (int i = 0; i < n; ++i) {
                fscanf(file, "%f %f", &pts[i].fX, &pts[i].fY);
            }
            dump_poly(pts, index++);
        }
    }
    fclose(file);
    return 0;
}
