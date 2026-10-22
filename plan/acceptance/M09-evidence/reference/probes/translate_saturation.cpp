#include "include/core/SkRegion.h"
#include <cstdint>
#include <cstdio>
int main() {
    SkRegion r(SkIRect{INT32_MIN+2, INT32_MIN+3, INT32_MIN+12, INT32_MIN+13});
    r.translate(-100, -100);
    const SkIRect& b = r.getBounds();
    printf("%d,%d,%d,%d\n", b.fLeft, b.fTop, b.fRight, b.fBottom);
    SkRegion h(SkIRect{INT32_MAX-12, 0, INT32_MAX-2, 10});
    h.translate(100, 0);
    printf("hi empty=%d bounds=%d,%d,%d,%d\n", h.isEmpty(), h.getBounds().fLeft, h.getBounds().fTop, h.getBounds().fRight, h.getBounds().fBottom);
    return 0;
}
