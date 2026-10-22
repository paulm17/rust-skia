// Codex runner; upstream tests and reporter implementation compile unchanged.
#include "tests/Test.h"
#include <cstdio>
#include <cstring>
class Reporter final : public skiatest::Reporter {
public:
    int failures = 0;
    void reportFailed(const skiatest::Failure& f) override {
        ++failures;
        std::fprintf(stderr, "%s:%d: %s %s\n", f.fileName, f.lineNo, f.condition, f.message.c_str());
    }
};
int m02_contracts();
int main(int argc, char** argv) {
    if(argc==2 && !std::strcmp(argv[1],"--contracts")) return m02_contracts();
    Reporter r;
    int count = 0;
    for (const auto& t : skiatest::TestRegistry::Range()) {
        const char* type = t.fTestType == skiatest::TestType::kCPUSerial ? "CPUSerial" : "CPU";
        if (argc == 1) { std::printf("REGISTERED %s %s\n", t.fName, type); continue; }
        if (std::strcmp(argv[1], t.fName)) continue;
        ++count;
        t.fCPUProc(&r);
        std::printf("%s %s failures=%d\n", r.failures ? "FAIL" : "PASS", t.fName, r.failures);
    }
    return argc == 1 ? 0 : r.failures || count != 1;
}
