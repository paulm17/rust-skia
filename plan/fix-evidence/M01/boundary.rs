use rust_skia::*;
fn emit(x:f32) {if x.is_nan(){print!("nan ")}else{print!("{:08x} ",x.to_bits())}}
fn main(){let inputs:[u32;23]=[0x0,0x80000000,0x1,0x80000001,0x7fffff,0x800000,0x3effffff,0x3f000000,0xbf000000,0x3f800000,0xbf800000,0x40400000,0x40800000,0x4affffff,0x4b000001,0x4effffff,0xceffffff,0x7f7fffff,0xff7fffff,0x7f800000,0xff800000,0x7fc12345,0xffc12345];
for bits in inputs{let x=f32::from_bits(bits);
emit(scalar_floor_to_scalar(x));
emit(scalar_ceil_to_scalar(x));
emit(scalar_round_to_scalar(x));
emit(scalar_trunc_to_scalar(x));
emit(scalar_fraction(x));
emit(scalar_sqrt(x));
emit(float_rsqrt(x));
emit(scalar_sin(x));
emit(scalar_cos(x));
emit(scalar_exp(x));
emit(scalar_log(x));
emit(scalar_log2(x));
emit(scalar_tan(x));
emit(scalar_asin(x));
emit(scalar_acos(x));
emit(degrees_to_radians(x));
emit(radians_to_degrees(x));
emit(scalar_sign_as_scalar(x));
print!("{} {} {} {} {} ",scalar_floor_to_int(x),scalar_ceil_to_int(x),scalar_round_to_int(x),float_saturate_to_int64(x),float_as_2s_compliment(x));
for by in inputs{let y=f32::from_bits(by);let b=Point::new(x,y);let mut a=b;emit(a.length());emit(a.normalize_returning_length());emit(a.x);emit(a.y);for len in [0.0f32,-0.0,1.0,-2.0,1e-37,f32::MAX]{a=b;let ok=a.set_length(len);print!("{} ",ok as u8);emit(a.x);emit(a.y);a=b;let ok=PointPriv::set_length_fast(&mut a,len);print!("{} ",ok as u8);emit(a.x);emit(a.y);}emit(Point::dot_product(&b,&Point::new(3.,4.)));emit(Point::cross_product(&b,&Point::new(3.,4.)));for t in [0.,0.25,0.5,1.]{emit(scalar_interp(x,y,t));}emit(scalar_ave(x,y));emit(PointPriv::distance_to_line_between_sqd(&b,&Point::new(0.,0.),&Point::new(3.,4.),None));emit(PointPriv::distance_to_line_segment_between_sqd(&b,&Point::new(0.,0.),&Point::new(3.,4.)));}println!();}let keys=[0.,1.,1.,2.];let vals=[-3.,4.,7.,9.];for key in [-1.,0.,0.5,1.,1.5,2.,3.]{emit(float_interp_func(key,&keys,&vals));}println!();}