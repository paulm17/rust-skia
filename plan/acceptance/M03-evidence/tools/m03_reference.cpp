#include "include/private/SkFixed.h"
#include "src/core/SkFDot6.h"
#include "include/private/SkTo.h"
#include "include/private/SkTPin.h"
#include "src/core/SkRandom.h"
#include "src/core/SkHalf.h"
#include "include/private/SkFloatingPoint.h"
#include <cstdio>
#include <cstdint>

int main() {
    std::printf("SkFDot6Div(-32768,1)=%d\n", SkFDot6Div(-32768, 1));
    std::printf("SkFDot6Div(0,1)=%d\n", SkFDot6Div(0, 1));
    std::printf("SkFixedDiv(-32768,1)=%d\n", SkFixedDiv(-32768, 1));
    std::printf("SkFixedDiv(i32min,1)=%d\n", SkFixedDiv(INT32_MIN, 1));
    std::printf("SkIntToFixed(1)=%d SkFixedRoundToInt(32768)=%d\n", SkIntToFixed(1), SkFixedRoundToInt(32768));
    std::printf("SkFloatToHalf(NaN)=0x%04x SkHalfToFloat(0x3c00)=%f\n",
                SkFloatToHalf(NAN), SkHalfToFloat(0x3c00));
    SkRandom rand(0);
    std::printf("rand(0):");
    for (int i = 0; i < 4; ++i) std::printf(" %u", rand.nextU());
    std::printf("\n");
    SkRandom r2(123456);
    std::printf("rand(123456):");
    for (int i = 0; i < 4; ++i) std::printf(" %u", r2.nextU());
    std::printf("\n");
    std::printf("pin(NaN,0,100)=%f\n", (double)SkTPin((float)NAN, 0.0f, 100.0f));
    std::printf("floatSaturateInt(NaN)=%d\n", sk_float_saturate2int((float)NAN));
    return 0;
}
