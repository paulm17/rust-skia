#include "include/core/SkM44.h"
#include "include/core/SkMatrix.h"
#include "include/core/SkRSXform.h"
#include "include/core/SkPoint.h"

#include <cmath>
#include <cstdio>
#include <limits>

static void print_matrix(const char* name, const SkMatrix& m) {
    std::printf("%s=[%.9g %.9g %.9g; %.9g %.9g %.9g; %.9g %.9g %.9g] mask=0x%x type=0x%x\n",
                name,
                m[SkMatrix::kMScaleX], m[SkMatrix::kMSkewX], m[SkMatrix::kMTransX],
                m[SkMatrix::kMSkewY], m[SkMatrix::kMScaleY], m[SkMatrix::kMTransY],
                m[SkMatrix::kMPersp0], m[SkMatrix::kMPersp1], m[SkMatrix::kMPersp2],
                m.getType(), m.getType());
}

static void test_invert_case(const char* name, const SkMatrix& m) {
    SkMatrix inv;
    bool ok = m.invert(&inv);
    std::printf("invert %s ok=%d finite=%d\n", name, ok ? 1 : 0, inv.isFinite() ? 1 : 0);
    print_matrix("  inv", inv);
}

int main() {
    const float inf = std::numeric_limits<float>::infinity();
    const float nan = std::numeric_limits<float>::quiet_NaN();
    test_invert_case("Scale(+inf,1)", SkMatrix::Scale(inf, 1.0f));
    test_invert_case("Scale(-inf,1)", SkMatrix::Scale(-inf, 1.0f));
    test_invert_case("Scale(nan,1)", SkMatrix::Scale(nan, 1.0f));
    test_invert_case("Scale(0,1)", SkMatrix::Scale(0.0f, 1.0f));
    test_invert_case("Scale(-0,1)", SkMatrix::Scale(-0.0f, 1.0f));

    SkMatrix aniso = SkMatrix::MakeAll(0.001f, 0.0f, 0.0f,
                                       0.0f, 5000.0f, 0.0f,
                                       0.0f, 0.0f, 1.0f);
    SkSize scale;
    SkMatrix remaining;
    bool ok = aniso.decomposeScale(&scale, &remaining);
    std::printf("decompose aniso ok=%d sx=%.9g sy=%.9g\n", ok ? 1 : 0, scale.width(), scale.height());
    print_matrix("  remaining", remaining);

    SkRSXform xform = SkRSXform::Make(2.0f, 3.0f, 5.0f, 7.0f);
    SkPoint quad[4];
    xform.toQuad(4.0f, 6.0f, quad);
    std::printf("quad=[(%.9g,%.9g),(%.9g,%.9g),(%.9g,%.9g),(%.9g,%.9g)]\n",
                quad[0].x(), quad[0].y(), quad[1].x(), quad[1].y(),
                quad[2].x(), quad[2].y(), quad[3].x(), quad[3].y());
    SkPoint strip[4];
    xform.toTriStrip(4.0f, 6.0f, strip);
    std::printf("strip=[(%.9g,%.9g),(%.9g,%.9g),(%.9g,%.9g),(%.9g,%.9g)]\n",
                strip[0].x(), strip[0].y(), strip[1].x(), strip[1].y(),
                strip[2].x(), strip[2].y(), strip[3].x(), strip[3].y());

    SkM44 m44 = SkM44::Scale(2.0f, 3.0f, 4.0f);
    SkV4 v = m44 * SkV4{1.0f, 2.0f, 3.0f, 1.0f};
    std::printf("m44 scale map=(%.9g,%.9g,%.9g,%.9g)\n", v.x, v.y, v.z, v.w);
    for (float sx : {0.0f, inf, -inf, nan}) {
        SkM44 a = SkM44::Scale(sx, 1.0f, 1.0f);
        SkM44 out;
        bool aok = a.invert(&out);
        std::printf("m44 invert Scale(%.9g) ok=%d\n", sx, aok ? 1 : 0);
    }
    return 0;
}
