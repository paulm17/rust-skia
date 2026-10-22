#include "include/core/SkPoint.h"
#include "src/core/SkPointPriv.h"
#include "src/core/SkFloatBits.h"
#include <cstdio>
float SkFloatInterpFunc(float,const float*,const float*,int);
void emit(float x) { if (SkIsNaN(x)) printf("nan "); else printf("%08x ",SkFloat2Bits(x)); }
int main(){ unsigned inputs[]={0x0,0x80000000,0x1,0x80000001,0x7fffff,0x800000,0x3effffff,0x3f000000,0xbf000000,0x3f800000,0xbf800000,0x40400000,0x40800000,0x4affffff,0x4b000001,0x4effffff,0xceffffff,0x7f7fffff,0xff7fffff,0x7f800000,0xff800000,0x7fc12345,0xffc12345};
for(unsigned bits:inputs){float x=SkBits2Float(bits);
emit(SkScalarFloorToScalar(x));
emit(SkScalarCeilToScalar(x));
emit(SkScalarRoundToScalar(x));
emit(SkScalarTruncToScalar(x));
emit(SkScalarFraction(x));
emit(SkScalarSqrt(x));
emit(sk_float_rsqrt(x));
emit(SkScalarSin(x));
emit(SkScalarCos(x));
emit(SkScalarExp(x));
emit(SkScalarLog(x));
emit(SkScalarLog2(x));
emit(SkScalarTan(x));
emit(SkScalarASin(x));
emit(SkScalarACos(x));
emit(SkDegreesToRadians(x));
emit(SkRadiansToDegrees(x));
emit(SkScalarSignAsScalar(x));
printf("%d %d %d %lld %d ",SkScalarFloorToInt(x),SkScalarCeilToInt(x),SkScalarRoundToInt(x),(long long)sk_float_saturate2int64(x),SkFloatAs2sCompliment(x));
for(unsigned by:inputs){float y=SkBits2Float(by); SkPoint a{x,y},b=a; emit(a.length());emit(SkPoint::Normalize(&a));emit(a.fX);emit(a.fY);for(float len:{0.0f,-0.0f,1.0f,-2.0f,1e-37f,3.402823466e38f}){a=b;bool ok=a.setLength(len);printf("%d ",ok);emit(a.fX);emit(a.fY);a=b;ok=SkPointPriv::SetLengthFast(&a,len);printf("%d ",ok);emit(a.fX);emit(a.fY);} emit(SkPoint::DotProduct(b,{3,4}));emit(SkPoint::CrossProduct(b,{3,4}));for(float t:{0.f,.25f,.5f,1.f})emit(SkScalarInterp(x,y,t));emit(SkScalarAve(x,y));emit(SkPointPriv::DistanceToLineBetweenSqd(b,{0,0},{3,4}));emit(SkPointPriv::DistanceToLineSegmentBetweenSqd(b,{0,0},{3,4}));}printf("\n");}float keys[]={0,1,1,2}, vals[]={-3,4,7,9};for(float key:{-1.f,0.f,.5f,1.f,1.5f,2.f,3.f})emit(SkFloatInterpFunc(key,keys,vals,4));printf("\n");}