#include "include/core/SkColor.h"
#include "include/core/SkUnPreMultiply.h"
#include "include/private/chromium/SkPMColor.h"
#include "include/utils/SkParse.h"
#include "src/core/SkColorData.h"
#include "src/core/SkColorPriv.h"
#include <cstdio>
#include <cstring>
#include <bit>
int main() {
    const SkPMColor src = 0xAB998877, dst = 0x66334455;
    for (unsigned scale = 0; scale < 256; scale++)
        printf("interp %u %08x %08x\n", scale, SkFourByteInterp(src, dst, scale),
               SkFastFourByteInterp(src, dst, scale));
    for (int a = 0; a <= 255; a++) for (int x = 0; x <= 255; x++) {
        SkPMColor p0 = SkPreMultiplyColor(SkColorSetARGB(a, x, x, x));
        SkColor c1 = SkUnPreMultiply::PMColorToColor(p0);
        printf("premul %d %d %08x %08x %08x\n", a, x, p0, c1, SkPreMultiplyColor(c1));
    }
    for (unsigned c = 0; c < (1u << 24); c += 4096) {
        SkColor color = 0xFF000000u | c;
        SkScalar hsv[3];
        SkColorToHSV(color, hsv);
        SkColor back = SkHSVToColor(0xFF, hsv);
        printf("hsv %08x %08x %08x %08x %08x\n", color,
               std::bit_cast<uint32_t>(hsv[0]), std::bit_cast<uint32_t>(hsv[1]),
               std::bit_cast<uint32_t>(hsv[2]), back);
    }
    const char* names[] = {"aliceblue", "black", "lightgrey", "yellowgreen", "notacolor"};
    for (const char* n : names) {
        SkColor c = 0;
        auto end = SkParse::FindNamedColor(n, strlen(n), &c);
        printf("named %s %d %08x\n", n, end != nullptr, c);
    }
    for (SkColor c : {SK_ColorBLACK, SK_ColorWHITE, 0u, 0x12345678u, 0xFFFF0000u}) {
        SkColor4f c4 = SkColor4f::FromColor(c);
        printf("c4 %08x %08x %08x %08x %08x\n", c, std::bit_cast<uint32_t>(c4.fR),
               std::bit_cast<uint32_t>(c4.fG), std::bit_cast<uint32_t>(c4.fB),
               std::bit_cast<uint32_t>(c4.fA));
    }
    return 0;
}
