#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"
#include "src/core/SkRRectPriv.h"

#include <cstdio>
#include <cstring>

static void print_bool(const char* name, bool value) {
    std::printf("%s=%d\n", name, value ? 1 : 0);
}

int main() {
    const SkRect bounds = SkRect::MakeWH(100.0f, 80.0f);
    SkVector exact[4] = {{50.0f, 40.0f}, {50.0f, 40.0f}, {50.0f, 40.0f}, {50.0f, 40.0f}};
    SkVector excess[4];
    std::memcpy(excess, exact, sizeof(excess));
    union { float f; uint32_t u; } v;
    v.f = 50.0f;
    v.u += 1;
    excess[0].fX = v.f;
    std::printf("excess.x=%.9g bits=0x%08x\n", excess[0].fX, v.u);
    print_bool("AreRectAndRadiiValid(exact)", SkRRectPriv::AreRectAndRadiiValid(bounds, exact));
    print_bool("AreRectAndRadiiValid(excess)", SkRRectPriv::AreRectAndRadiiValid(bounds, excess));

    SkRRect exactRR;
    exactRR.setRectRadii(bounds, exact);
    print_bool("exactRR.isValid", exactRR.isValid());
    print_bool("exactRR.isOval", exactRR.isOval());
    SkRRect excessRR;
    excessRR.setRectRadii(bounds, excess);
    print_bool("excessRR.isValid", excessRR.isValid());
    print_bool("excessRR.isOval", excessRR.isOval());

    const unsigned char fuzz1[] = {
        0x0a, 0x00, 0x00, 0xff, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f,
        0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
        0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
        0x7f, 0x7f, 0x7f, 0x02, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x02, 0x00
    };
    const unsigned char fuzz2[] = {
        0x5d, 0xff, 0xff, 0x5d, 0x0a, 0x60, 0x0a, 0x0a, 0x0a, 0x7e, 0x0a, 0x5a,
        0x0a, 0x12, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a,
        0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x00, 0x00, 0x00, 0x0a,
        0x0a, 0x0a, 0x0a, 0x26, 0x0a, 0x0a, 0x0a, 0x0a, 0xff, 0xff, 0x0a, 0x0a
    };
    const unsigned char fuzz3[] = {
        0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x04, 0xdd, 0xdd, 0x15,
        0xfe, 0x00, 0x00, 0x04, 0x05, 0x7e, 0x00, 0x00, 0x00, 0xff, 0x08, 0x04,
        0xff, 0xff, 0xfe, 0xfe, 0xff, 0x32, 0x32, 0x32, 0x32, 0x00, 0x32, 0x32,
        0x04, 0xdd, 0x3d, 0x1c, 0xfe, 0x89, 0x04, 0x0a, 0x0e, 0x05, 0x7e, 0x0a
    };
    const unsigned char* buffers[] = {fuzz1, fuzz2, fuzz3};
    for (int i = 0; i < 3; ++i) {
        SkRRect rr;
        size_t got = rr.readFromMemory(buffers[i], sizeof(fuzz1));
        std::printf("fuzz%d.read=%zu valid=%d\n", i + 1, got, rr.isValid() ? 1 : 0);
    }

    union { float f; uint32_t u; } a, b;
    a.u = 0x4b4b4b4a;
    b.u = 0x4b4b4b4b;
    SkRRect xy;
    xy.setRectXY(SkRect::MakeLTRB(0, 0, b.f, b.f), a.f, a.f);
    print_bool("b511 setRectXY valid", xy.isValid());
    SkRRect nine;
    nine.setNinePatch(SkRect::MakeLTRB(0, 0, b.f, b.f), a.f, a.f, a.f, a.f);
    print_bool("b511 setNinePatch valid", nine.isValid());
    SkVector r4[4] = {{a.f, a.f}, {a.f, a.f}, {a.f, a.f}, {a.f, a.f}};
    SkRRect radiiRR;
    radiiRR.setRectRadii(SkRect::MakeLTRB(0, 0, b.f, b.f), r4);
    print_bool("b511 setRectRadii valid", radiiRR.isValid());

    SkRRect oval = SkRRect::MakeOval(SkRect::MakeWH(37.408840f, 73.093933f));
    SkRRect offset = oval.makeOffset(-5430.867188f, -26967.046875f);
    print_bool("b527 offset valid", offset.isValid());
    print_bool("b527 offset oval", offset.isOval());

    return 0;
}
