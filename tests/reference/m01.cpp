// Codex external runner: compile unchanged upstream tests and production sources.
#include "tests/Test.h"
#include <cstdio>
#include <cstring>
void skiatest::Reporter::bumpTestCount() {}
bool skiatest::Reporter::allowExtendedTest() const { return false; }
bool skiatest::Reporter::verbose() const { return false; }
// Assigned tests never push reporter contexts; keep the original failure fields.
void skiatest::Reporter::reportFailedWithContext(const skiatest::Failure& f) { reportFailed(f); }
class Reporter final : public skiatest::Reporter {
public:
    int failures = 0;
    void reportFailed(const skiatest::Failure& f) override {
        ++failures;
        std::fprintf(stderr, "%s:%d: %s %s\n", f.fileName, f.lineNo, f.condition, f.message.c_str());
    }
};
int contracts();
int main() {
    Reporter reporter;
    int count = 0;
    bool falseFound = false;
    for (const auto& test : skiatest::TestRegistry::Range()) {
        std::printf("REGISTERED %s CPU\n", test.fName);
        falseFound |= std::strcmp(test.fName, "TestFalseCondition") == 0;
        ++count;
        const int before = reporter.failures;
        test.fCPUProc(&reporter);
        std::printf("%s %s\n", before == reporter.failures ? "PASS" : "FAIL", test.fName);
    }
    std::printf("enabled=%d failures=%d false_condition_registered=%d\n", count, reporter.failures, falseFound);
    return contracts() || reporter.failures || falseFound || count != 6;
}
