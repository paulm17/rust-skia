// M10 acceptance reference runner (Deepseek). Compiles the unchanged pinned
// tests/ColorPrivTest.cpp, tests/ColorTest.cpp, tests/HSVRoundTripTest.cpp,
// tests/ParseColorTest.cpp and tests/SkColor4fTest.cpp, then invokes their ten
// CPU registry entries. The pinned Skia library is a release (NDEBUG) GN build;
// both runner profiles link it.
#include "tests/Test.h"

#include <cstdio>
#include <cstring>

void skiatest::Reporter::bumpTestCount() {}
bool skiatest::Reporter::allowExtendedTest() const { return false; }
bool skiatest::Reporter::verbose() const { return false; }
void skiatest::Reporter::reportFailedWithContext(const skiatest::Failure& f) {
    this->reportFailed(f);
}

class Reporter final : public skiatest::Reporter {
public:
    int failures = 0;
    void reportFailed(const skiatest::Failure& f) override {
        ++failures;
        std::fprintf(stderr, "%s:%d: %s %s\n", f.fileName, f.lineNo, f.condition,
                     f.message.c_str());
    }
};

static const char* kExpected[] = {
    "Splay",
    "FourByteInterp",
    "SkBlendARGB32_SameOpaqueSrcAndDstWithAnyAlpha_ProducesSameColor",
    "ColorPremul",
    "SkPMColor_SetAndRetrieveChannels",
    "ColorInterp",
    "ColorFastIterp",
    "ColorToHSVRoundTrip",
    "ParseNamedColor",
    "SkColor4f_FromColor",
};

int main() {
    Reporter reporter;
    int count = 0;
    int expectedSeen = 0;
    for (const auto& test : skiatest::TestRegistry::Range()) {
        if (test.fTestType != skiatest::TestType::kCPU) {
            std::printf("SKIP-NONCPU %s\n", test.fName);
            continue;
        }
        bool expected = false;
        for (const char* name : kExpected) {
            if (std::strcmp(test.fName, name) == 0) {
                expected = true;
                break;
            }
        }
        if (!expected) {
            std::printf("SKIP-UNASSIGNED %s\n", test.fName);
            continue;
        }
        std::printf("REGISTERED %s CPU\n", test.fName);
        ++count;
        ++expectedSeen;
        const int before = reporter.failures;
        test.fCPUProc(&reporter);
        std::printf("%s %s\n", before == reporter.failures ? "PASS" : "FAIL", test.fName);
    }
    std::printf("enabled=%d expected=%d failures=%d\n", count, expectedSeen,
                reporter.failures);
    return reporter.failures || count != 10 || expectedSeen != 10;
}
