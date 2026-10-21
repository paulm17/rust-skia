// Codex supplemental observations, separate from faithful upstream cases.
#include "include/core/SkPoint.h"
#include "include/core/SkScalar.h"
#include "include/private/SkFloatingPoint.h"
#include "src/core/SkFloatBits.h"
#include "src/core/SkPointPriv.h"
#include <cstdio>
#include <limits>
#include <cstring>
// Upstream defines this external function in SkScalar.cpp without a header declaration.
float SkFloatInterpFunc(float, const float[], const float[], int);
int contracts() {
    int failures = 0;
    auto check = [&](bool ok, const char* name) { if (!ok) { ++failures; std::fprintf(stderr, "CONTRACT FAIL %s\n", name); } };
    for (uint32_t bits : {0u, 0x80000000u, 0x3f800000u, 0xbf800000u, 0x7f800000u, 0xff800000u, 0x7fc01234u, 1u, 0x80000001u}) {
        float value = SkBits2Float(bits);
        int expected = bits & 0x80000000u ? -int(bits & 0x7fffffffu) : int(bits);
        check(SkFloat2Bits(value) == bits, "bits");
        check(SkFloatAs2sCompliment(value) == expected, "signed encoding");
        check(SkFloat2Bits(Sk2sComplimentAsFloat(expected)) == (bits == 0x80000000u ? 0u : bits), "roundtrip");
    }
    check(sk_float_saturate2int(SK_FloatNaN) == SK_MaxS32FitsInFloat, "nan saturation");
    check(sk_float_saturate2int(SK_FloatInfinity) == SK_MaxS32FitsInFloat, "inf saturation");
    check(sk_float_saturate2int(-SK_FloatInfinity) == SK_MinS32FitsInFloat, "negative saturation");
    check(sk_float_saturate2int(-1.75f) == -1, "truncation");
    check(sk_double_round(-0.5) == 0 && sk_double_round2int(-0.5) == -1, "double rounding");
    check(sk_float_midpoint(SK_ScalarMax, SK_ScalarMax) == SK_ScalarMax, "midpoint overflow");
    check(sk_float_midpoint(SK_ScalarMax, -SK_ScalarMax) == 0, "midpoint cancel");
    float keys[] = {0,1,2}, values[] = {10,20,40};
    float searches[] = {-1,0,0.5f,1,1.5f,2,3}, expected[] = {10,10,15,20,30,40,40};
    for (int i=0;i<7;++i) check(SkFloatInterpFunc(searches[i], keys, values, 3) == expected[i], "interpolation");
    double one = 1; uint64_t bits; std::memcpy(&bits, &one, 8);
    double d16, d17; auto b16=bits+16, b17=bits+17; std::memcpy(&d16,&b16,8); std::memcpy(&d17,&b17,8);
    check(sk_doubles_nearly_equal_ulps(1,d16,16) && !sk_doubles_nearly_equal_ulps(1,d17,16), "ULP boundary");
    check(sk_double_nearly_zero(double(std::numeric_limits<float>::epsilon())/2) && !sk_double_nearly_zero(std::numeric_limits<float>::epsilon()), "nearly zero boundary");
    SkPoint p={1,3}, a={0,0}, b={2,0}; SkPointPriv::Side side;
    check(SkPointPriv::DistanceToLineBetweenSqd(p,a,b,&side)==9 && side==SkPointPriv::kRight_Side, "line side");
    check(SkPointPriv::DistanceToLineSegmentBetweenSqd({4,3},a,b)==13, "segment");
    check(SkPointPriv::DistanceToLineBetweenSqd(p,a,a)==10, "degenerate line");
    auto ip = SkIPoint::Make(INT32_MAX, INT32_MIN) + SkIPoint::Make(1,-1);
    check(ip == SkIPoint::Make(INT32_MAX,-INT32_MAX), "integer saturation");
    std::printf("supplemental_contract_failures=%d\n",failures);
    return failures;
}
