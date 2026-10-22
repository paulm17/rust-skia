from pathlib import Path
p=Path('plan/fix-evidence/M01')
# Frozen IEEE inputs and exact comparison: NaNs classified, all other float bits exact.
bits=[0,0x80000000,1,0x80000001,0x007fffff,0x00800000,0x3effffff,0x3f000000,0xbf000000,0x3f800000,0xbf800000,0x40400000,0x40800000,0x4affffff,0x4b000001,0x4effffff,0xceffffff,0x7f7fffff,0xff7fffff,0x7f800000,0xff800000,0x7fc12345,0xffc12345]
(p/'inputs.txt').write_text('\n'.join(f'{b:08x}' for b in bits)+'\n')
ops=[('floor','SkScalarFloorToScalar(x)','scalar_floor_to_scalar(x)'),('ceil','SkScalarCeilToScalar(x)','scalar_ceil_to_scalar(x)'),('round','SkScalarRoundToScalar(x)','scalar_round_to_scalar(x)'),('trunc','SkScalarTruncToScalar(x)','scalar_trunc_to_scalar(x)'),('fraction','SkScalarFraction(x)','scalar_fraction(x)'),('sqrt','SkScalarSqrt(x)','scalar_sqrt(x)'),('rsqrt','sk_float_rsqrt(x)','float_rsqrt(x)'),('sin','SkScalarSin(x)','scalar_sin(x)'),('cos','SkScalarCos(x)','scalar_cos(x)'),('exp','SkScalarExp(x)','scalar_exp(x)'),('log','SkScalarLog(x)','scalar_log(x)'),('log2','SkScalarLog2(x)','scalar_log2(x)'),('tan','SkScalarTan(x)','scalar_tan(x)'),('asin','SkScalarASin(x)','scalar_asin(x)'),('acos','SkScalarACos(x)','scalar_acos(x)'),('degrees','SkDegreesToRadians(x)','degrees_to_radians(x)'),('radians','SkRadiansToDegrees(x)','radians_to_degrees(x)'),('sign','SkScalarSignAsScalar(x)','scalar_sign_as_scalar(x)')]
cpp='''#include "include/core/SkPoint.h"
#include "src/core/SkPointPriv.h"
#include "src/core/SkFloatBits.h"
#include <cstdio>
float SkFloatInterpFunc(float,const float*,const float*,int);
void emit(float x) { if (SkIsNaN(x)) printf("nan "); else printf("%08x ",SkFloat2Bits(x)); }
int main(){ unsigned inputs[]={'''+','.join(hex(b) for b in bits)+'''};
for(unsigned bits:inputs){float x=SkBits2Float(bits);
'''
rs='''use rust_skia::*;
fn emit(x:f32) {if x.is_nan(){print!("nan ")}else{print!("{:08x} ",x.to_bits())}}
fn main(){let inputs:[u32;'''+str(len(bits))+']=['+','.join(hex(b) for b in bits)+'''];
for bits in inputs{let x=f32::from_bits(bits);
'''
for _,c,r in ops: cpp+=f'emit({c});\n';rs+=f'emit({r});\n'
cpp+='printf("%d %d %d %lld %d ",SkScalarFloorToInt(x),SkScalarCeilToInt(x),SkScalarRoundToInt(x),(long long)sk_float_saturate2int64(x),SkFloatAs2sCompliment(x));\n'
rs+='print!("{} {} {} {} {} ",scalar_floor_to_int(x),scalar_ceil_to_int(x),scalar_round_to_int(x),float_saturate_to_int64(x),float_as_2s_compliment(x));\n'
cpp+='for(unsigned by:inputs){float y=SkBits2Float(by); SkPoint a{x,y},b=a; emit(a.length());emit(SkPoint::Normalize(&a));emit(a.fX);emit(a.fY);for(float len:{0.0f,-0.0f,1.0f,-2.0f,1e-37f,3.402823466e38f}){a=b;bool ok=a.setLength(len);printf("%d ",ok);emit(a.fX);emit(a.fY);a=b;ok=SkPointPriv::SetLengthFast(&a,len);printf("%d ",ok);emit(a.fX);emit(a.fY);} emit(SkPoint::DotProduct(b,{3,4}));emit(SkPoint::CrossProduct(b,{3,4}));for(float t:{0.f,.25f,.5f,1.f})emit(SkScalarInterp(x,y,t));emit(SkScalarAve(x,y));emit(SkPointPriv::DistanceToLineBetweenSqd(b,{0,0},{3,4}));emit(SkPointPriv::DistanceToLineSegmentBetweenSqd(b,{0,0},{3,4}));}printf("\\n");}}'
rs+='for by in inputs{let y=f32::from_bits(by);let b=Point::new(x,y);let mut a=b;emit(a.length());emit(a.normalize_returning_length());emit(a.x);emit(a.y);for len in [0.0f32,-0.0,1.0,-2.0,1e-37,f32::MAX]{a=b;let ok=a.set_length(len);print!("{} ",ok as u8);emit(a.x);emit(a.y);a=b;let ok=PointPriv::set_length_fast(&mut a,len);print!("{} ",ok as u8);emit(a.x);emit(a.y);}emit(Point::dot_product(&b,&Point::new(3.,4.)));emit(Point::cross_product(&b,&Point::new(3.,4.)));for t in [0.,0.25,0.5,1.]{emit(scalar_interp(x,y,t));}emit(scalar_ave(x,y));emit(PointPriv::distance_to_line_between_sqd(&b,&Point::new(0.,0.),&Point::new(3.,4.),None));emit(PointPriv::distance_to_line_segment_between_sqd(&b,&Point::new(0.,0.),&Point::new(3.,4.)));}println!();}}'
cpp=cpp[:-1]+"float keys[]={0,1,1,2}, vals[]={-3,4,7,9};for(float key:{-1.f,0.f,.5f,1.f,1.5f,2.f,3.f})emit(SkFloatInterpFunc(key,keys,vals,4));printf(\"\\n\");}"
rs=rs[:-1]+"let keys=[0.,1.,1.,2.];let vals=[-3.,4.,7.,9.];for key in [-1.,0.,0.5,1.,1.5,2.,3.]{emit(float_interp_func(key,&keys,&vals));}println!();}"
(p/'boundary.cpp').write_text(cpp);(p/'boundary.rs').write_text(rs)
