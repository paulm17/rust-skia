#include "src/core/SkAAClip.h"
#include "include/core/SkRegion.h"
#include <cstdio>
int main() {
    SkRegion rgn;
    rgn.setRects(SkSpan<const SkIRect>(std::initializer_list<SkIRect>{{0,0,2,2},{4,0,6,2}}));
    SkAAClip aa;
    aa.setRegion(rgn);
    printf("bounds=%d,%d,%d,%d isEmpty=%d isRect=%d\n", aa.getBounds().fLeft, aa.getBounds().fTop, aa.getBounds().fRight, aa.getBounds().fBottom, aa.isEmpty(), aa.isRect());
    printf("qc(0,0,2,2)=%d qc(0,0,1,2)=%d qc(0,0,2,1)=%d qc(4,0,6,2)=%d\n",
        aa.quickContains(SkIRect{0,0,2,2}), aa.quickContains(SkIRect{0,0,1,2}), aa.quickContains(SkIRect{0,0,2,1}), aa.quickContains(SkIRect{4,0,6,2}));
    return 0;
}
