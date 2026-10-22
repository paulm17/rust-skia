// M09 acceptance reference runner (Deepseek). Compiles the unchanged pinned
// tests/ClipStackTest.cpp and tests/RTreeTest.cpp and invokes their registry entries.
// The pinned Skia library is a release (NDEBUG) GN build; both runner profiles link it.
#include "tests/Test.h"

#include <cstdio>
#include <cstring>

void skiatest::Reporter::bumpTestCount() {}
bool skiatest::Reporter::allowExtendedTest() const { return false; }
bool skiatest::Reporter::verbose() const { return false; }
void skiatest::Reporter::reportFailedWithContext(const skiatest::Failure& f) { this->reportFailed(f); }

class Reporter final : public skiatest::Reporter {
public:
    int failures = 0;
    void reportFailed(const skiatest::Failure& f) override {
        ++failures;
        std::fprintf(stderr, "%s:%d: %s %s\n", f.fileName, f.lineNo, f.condition, f.message.c_str());
    }
};

int main() {
    Reporter reporter;
    int count = 0;
    bool sawClipStack = false;
    bool sawRTree = false;
    for (const auto& test : skiatest::TestRegistry::Range()) {
        if (test.fTestType != skiatest::TestType::kCPU) {
            std::printf("SKIP-NONCPU %s\n", test.fName);
            continue;
        }
        std::printf("REGISTERED %s CPU\n", test.fName);
        sawClipStack |= std::strcmp(test.fName, "ClipStack") == 0;
        sawRTree |= std::strcmp(test.fName, "RTree") == 0;
        ++count;
        const int before = reporter.failures;
        test.fCPUProc(&reporter);
        std::printf("%s %s\n", before == reporter.failures ? "PASS" : "FAIL", test.fName);
    }
    std::printf("enabled=%d failures=%d clipstack=%d rtree=%d\n",
                count, reporter.failures, sawClipStack, sawRTree);
    return reporter.failures || count != 2 || !sawClipStack || !sawRTree;
}
