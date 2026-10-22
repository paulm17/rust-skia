#include "include/core/SkPoint3.h"
#include "include/core/SkRect.h"
#include "include/core/SkSize.h"
#include "src/core/SkRectPriv.h"
#include "src/core/SkScaleToSides.h"

#include <cmath>
#include <cstdio>

static void print_rect(const char* name, const SkRect& r) {
    std::printf("%s=(%.9g,%.9g,%.9g,%.9g) empty=%d sorted=%d finite=%d\n",
                name, r.fLeft, r.fTop, r.fRight, r.fBottom,
                r.isEmpty() ? 1 : 0, r.isSorted() ? 1 : 0, r.isFinite() ? 1 : 0);
}

static void print_irect(const char* name, const SkIRect& r) {
    std::printf("%s=(%d,%d,%d,%d) empty=%d\n",
                name, r.fLeft, r.fTop, r.fRight, r.fBottom, r.isEmpty() ? 1 : 0);
}

int main() {
    print_irect("MakeILarge", SkRectPriv::MakeILarge());
    print_irect("MakeILargestInverted", SkRectPriv::MakeILargestInverted());
    print_rect("MakeLargeS32", SkRectPriv::MakeLargeS32());
    print_rect("MakeLargest", SkRectPriv::MakeLargest());
    print_rect("MakeLargestInverted", SkRectPriv::MakeLargestInverted());

    const SkPoint one[] = {{2.0f, 3.0f}};
    const SkPoint inf[] = {{1.0f, 2.0f}, {SK_FloatInfinity, 3.0f}};
    const SkPoint nan[] = {{1.0f, 2.0f}, {SK_FloatNaN, 3.0f}};
    SkRect bounds = SkRect::MakeWH(10.0f, 10.0f);
    std::printf("setBoundsCheck(empty)=%d ", bounds.setBoundsCheck({}));
    print_rect("bounds", bounds);
    std::printf("setBoundsCheck(one)=%d ", bounds.setBoundsCheck(one));
    print_rect("bounds", bounds);
    std::printf("setBoundsCheck(inf)=%d ", bounds.setBoundsCheck(inf));
    print_rect("bounds", bounds);
    std::printf("setBoundsCheck(nan)=%d ", bounds.setBoundsCheck(nan));
    print_rect("bounds", bounds);
    bounds.setBoundsNoCheck(inf);
    print_rect("setBoundsNoCheck(inf)", bounds);

    const float big = SK_ScalarMax * 0.75f;
    SkRect overflow = SkRect::MakeLTRB(-big, -big, big, big);
    std::printf("whOverflow finite=%d width=%g height=%g cx=%g cy=%g hw=%g hh=%g empty=%d\n",
                overflow.isFinite() ? 1 : 0, overflow.width(), overflow.height(),
                overflow.centerX(), overflow.centerY(),
                SkRectPriv::HalfWidth(overflow), SkRectPriv::HalfHeight(overflow),
                overflow.isEmpty() ? 1 : 0);

    const SkIRect reallyBig = SkIRect::MakeLTRB(-SK_MaxS32 + 1000, 0, SK_MaxS32 - 1000, 100);
    const SkIRect reasonable = SkIRect::MakeLTRB(-10, 10, 10, 20);
    print_irect("reallyBig", reallyBig);
    print_irect("reallyBig-reasonable", SkRectPriv::Subtract(reallyBig, reasonable));
    print_irect("reasonable-reallyBig", SkRectPriv::Subtract(reasonable, reallyBig));

    const SkIRect empty = SkIRect::MakeEmpty();
    print_irect("closest(empty,reasonable)",
                SkRectPriv::ClosestDisjointEdge(empty, reasonable));
    print_irect("closest(reasonable,empty)",
                SkRectPriv::ClosestDisjointEdge(reasonable, empty));

    struct GRec { float x, y, z, len; };
    const GRec g[] = {
        {0.0f, 0.0f, 0.0f, 0.0f},
        {0.3f, 0.4f, 0.5f, SK_ScalarRoot2Over2},
        {1.0e-37f, 1.0e-37f, 1.0e-37f, 0.0f},
        {3.4e38f, 0.0f, 0.0f, 3.4e38f}
    };
    for (const auto& v : g) {
        std::printf("point3 length(%.9g,%.9g,%.9g)=%.9g expected=%.9g\n",
                    v.x, v.y, v.z, SkPoint3::Length(v.x, v.y, v.z), v.len);
    }
    SkPoint3 p = SkPoint3::Make(3.4e38f, 0.0f, 0.0f);
    const bool normalized = p.normalize();
    std::printf("point3 normalizeOverflow ok=%d value=(%.9g,%.9g,%.9g) length=%.9g\n",
                normalized ? 1 : 0, p.fX, p.fY, p.fZ, p.length());

    SkSize size = SkSize::Make(5.0f, 3.0f);
    SkISize rounded = size.toRound();
    std::printf("size round=(%d,%d) empty=%d\n", rounded.fWidth, rounded.fHeight,
                size.isEmpty() ? 1 : 0);

    float a = 10.0f;
    float b = 20.0f;
    SkScaleToSides::AdjustRadii(20.0, 0.5, &a, &b);
    std::printf("adjustRadii limit=20 scale=0.5 a=%.9g b=%.9g sum=%.9g\n", a, b, a + b);
    return 0;
}
