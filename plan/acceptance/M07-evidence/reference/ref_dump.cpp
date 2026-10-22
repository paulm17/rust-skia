#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkPathMeasure.h"
#include "include/core/SkContourMeasure.h"
#include "include/core/SkPathTypes.h"
#include "include/core/SkMatrix.h"
#include "include/core/SkRect.h"
#include "src/core/SkPathData.h"
#include "include/utils/SkParsePath.h"
#include <cstdio>

static void dump_bounds(const char* tag, const SkPath& p) {
    SkRect b = p.getBounds();
    printf("%s bounds %.9g %.9g %.9g %.9g pts %d vbs %d\n", tag, b.fLeft, b.fTop, b.fRight, b.fBottom, p.countPoints(), (int)p.verbs().size());
}

int main() {
    {
        const char* src[] = {
            "M1,1 l-2.58-2.828-3.82-0.113, 1.9-3.3223-1.08-3.6702, 3.75,0.7744,3.16-2.1551,0.42,3.8008,3.02,2.3384-3.48,1.574-1.29,3.601z",
            "", "M0,0L10,10", "M-5.5,-0.5 Q 0 0 6,6.50"};
        for (int i = 0; i < 4; ++i) {
            auto p = SkParsePath::FromSVGString(src[i]);
            if (!p) { printf("parse%d null\n", i); continue; }
            char tag[32]; snprintf(tag, sizeof(tag), "parse%d", i);
            dump_bounds(tag, *p);
        }
    }
    {
        SkRect r = SkRect::MakeLTRB(0, 0, 10, 10.5f);
        SkPathBuilder b;
        b.addRect(r); dump_bounds("rect", b.snapshot());
        b.reset(); b.addOval(r); dump_bounds("oval", b.snapshot());
        b.reset(); b.addRRect(SkRRect::MakeRectXY(r, 4, 4.5f), SkPathDirection::kCW, 6); dump_bounds("rrect", b.snapshot());
    }
    {
        SkPathBuilder b;
        b.moveTo(0,0).lineTo(1,0).lineTo(1,1).lineTo(0,1);
        SkPathMeasure meas(b.snapshot(), true, 1.0f);
        printf("meas1 length %.9g\n", meas.getLength());
        SkPath line = SkPath::Line({0,0},{3,4});
        meas.setPath(&line, false);
        printf("meas2 length %.9g\n", meas.getLength());
        SkPoint pos, tan;
        bool ok = meas.getPosTan(2.5f, &pos, &tan);
        printf("meas2 postan %d %.9g %.9g %.9g %.9g\n", ok, pos.fX, pos.fY, tan.fX, tan.fY);
        SkPathBuilder b2;
        b2.moveTo(10,10).lineTo(10,30).lineTo(30,30).quadTo(40,30,40,40).cubicTo(50,40,50,50,40,50).conicTo(50,50,50,60,1.2f);
        SkContourMeasureIter it(b2.snapshot(), false, 1.0f);
        auto cm = it.next();
        printf("cm length %.9g closed %d\n", cm->length(), cm->isClosed());
        printf("done\n");
    }
    {
        SkPoint pts[3] = {{0,0},{1,1},{2,2}};
        SkPathVerb v[2] = {SkPathVerb::kMove, SkPathVerb::kLine};
        auto d = SkPathData::Make({pts,3}, {v,2}, {});
        printf("make3 %d\n", d != nullptr);
        auto bad = SkPathData::Make({pts,1}, {v,1}, {});
        printf("makemove %d empty %d\n", bad != nullptr, bad ? bad->points().empty() : -1);
        SkPathVerb badv[1] = {SkPathVerb::kLine};
        printf("makeline %d\n", SkPathData::Make({pts,1}, {badv,1}, {}) != nullptr);
    }
    return 0;
}
