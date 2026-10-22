// M08 acceptance reference probe (Deepseek). Exercises the three AsADashTest
// registrations and their parameter boundaries against the pinned C++ checkout.
#include "include/core/SkPathEffect.h"
#include "include/effects/SkCornerPathEffect.h"
#include "include/effects/SkDashPathEffect.h"
#include "src/core/SkPathEffectBase.h"

#include <cmath>
#include <cstdio>
#include <cstring>

static unsigned bits(float f) {
    unsigned u;
    std::memcpy(&u, &f, 4);
    return u;
}

static void show(const char* name, sk_sp<SkPathEffect> pe) {
    if (!pe) {
        std::printf("%s null=1\n", name);
        return;
    }
    auto info = as_PEB(pe)->asADash();
    if (!info) {
        std::printf("%s valid=0\n", name);
        return;
    }
    std::printf("%s valid=1 n=%zu phase=%08x\n", name, info->fIntervals.size(),
                bits(info->fPhase));
    for (float v : info->fIntervals) {
        std::printf("  i %08x\n", bits(v));
    }
}

int main() {
    show("corner", SkCornerPathEffect::Make(1.0f));

    const SkScalar intervals[] = {4.0f, 2.0f, 1.0f, 3.0f};
    show("dash", SkDashPathEffect::Make(intervals, 2.0f));
    show("dashneg", SkDashPathEffect::Make(intervals, -7.0f));
    show("dashwrap", SkDashPathEffect::Make(intervals, 27.0f));
    show("dashnegzero", SkDashPathEffect::Make(intervals, -10.0f));
    show("dashtinyneg", SkDashPathEffect::Make(intervals, -1.4012985e-45f));

    const SkScalar odd[] = {4.0f, 2.0f, 1.0f};
    show("odd", SkDashPathEffect::Make(odd, 0.0f));
    const SkScalar zero[] = {0.0f, 0.0f};
    show("zero", SkDashPathEffect::Make(zero, 0.0f));
    const SkScalar naniv[] = {NAN, 1.0f};
    show("nan", SkDashPathEffect::Make(naniv, 0.0f));
    const SkScalar infiv[] = {INFINITY, 1.0f};
    show("inf", SkDashPathEffect::Make(infiv, 0.0f));
    const SkScalar maxiv[] = {3.4028235e38f, 3.4028235e38f};
    show("maxsum", SkDashPathEffect::Make(maxiv, 0.0f));
    show("phnan", SkDashPathEffect::Make(intervals, NAN));
    show("phinf", SkDashPathEffect::Make(intervals, INFINITY));

    const SkScalar zeroton[] = {0.0f, 1.0f};
    show("zeroton", SkDashPathEffect::Make(zeroton, 0.0f));

    show("corner0", SkCornerPathEffect::Make(0.0f));
    show("cornernan", SkCornerPathEffect::Make(NAN));
    show("cornerinf", SkCornerPathEffect::Make(INFINITY));
    show("cornertiny", SkCornerPathEffect::Make(1.4012985e-45f));
    return 0;
}
