//! Port of `modules/skcms/src/skcms_Transform.h`, `Transform_inl.h` and the
//! `skcms_Transform*.cc` translation units.
#![allow(
    clippy::excessive_precision,
    clippy::identity_op,
    clippy::if_same_then_else
)]

//!
//! On Apple Silicon the reference defines `SKCMS_DISABLE_HSW`/`SKCMS_DISABLE_SKX`, so the HSW and
//! SKX entry points forward to the baseline program, and the baseline program is built with NEON.
//! This port executes the same op program one pixel at a time; the arithmetic per lane is the same
//! IEEE float work, and `min_`/`max_` preserve NEON's NaN propagation.

use crate::skcms::{
    SkcmsA2B, SkcmsAlphaFormat, SkcmsB2A, SkcmsCurve, SkcmsCurveTable, SkcmsGrid, SkcmsIccProfile,
    SkcmsMatrix3x3, SkcmsMatrix3x4, SkcmsPixelFormat, SkcmsTfType, SkcmsTransferFunction,
    skcms_bytes_per_pixel, skcms_prep_for_destination, skcms_set_reference_pq_ish_trc,
    skcms_set_sdr_hlg_ish_trc, skcms_srgb_inverse_transfer_function, skcms_srgb_profile,
    skcms_srgb_transfer_function, skcms_transfer_function_get_type,
};

/// The maximum number of ops in a transform program.
pub const SKCMS_MAX_PROGRAM_OPS: usize = 64;
const INFINITY_: f32 = f32::INFINITY;

/// All transform ops, in the reference's declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    LoadA8,
    LoadG8,
    LoadGa88,
    Load4444,
    Load565,
    Load888,
    Load8888,
    Load1010102,
    Load101010xXr,
    Load10101010Xr,
    Load161616Le,
    Load16161616Le,
    Load161616Be,
    Load16161616Be,
    LoadHhh,
    LoadHhhh,
    LoadFff,
    LoadFfff,

    SwapRb,
    Clamp,
    Invert,
    ForceOpaque,
    Premul,
    Unpremul,
    Matrix3x3,
    Matrix3x4,

    LabToXyz,
    XyzToLab,

    GammaR,
    GammaG,
    GammaB,
    GammaA,
    GammaRgb,

    TfR,
    TfG,
    TfB,
    TfA,
    TfRgb,

    PqR,
    PqG,
    PqB,
    PqA,
    PqRgb,

    HlgR,
    HlgG,
    HlgB,
    HlgA,
    HlgRgb,
    HlgOotfScale,

    HlginvR,
    HlginvG,
    HlginvB,
    HlginvA,
    HlginvRgb,
    HlginvOotfScale,

    TableR,
    TableG,
    TableB,
    TableA,

    ClutA2B,
    ClutB2A,

    StoreA8,
    StoreG8,
    StoreGa88,
    Store4444,
    Store565,
    Store888,
    Store8888,
    Store1010102,
    Store161616Le,
    Store16161616Le,
    Store161616Be,
    Store16161616Be,
    Store101010xXr,
    Store10101010Xr,
    StoreHhh,
    StoreHhhh,
    StoreFff,
    StoreFfff,
}

/// Per-op context (`const void*` in the reference).
#[derive(Clone, Copy)]
pub enum OpArg<'a> {
    None,
    // The reference stores raw pointers to every context object. Small, `Copy` contexts are stored
    // by value here to keep the program decoupled from temporary matrices; variable-size data
    // (curves, A2B/B2A tables) is still borrowed from the profiles.
    Tf(SkcmsTransferFunction),
    Curve(&'a SkcmsCurve),
    Matrix3x3(SkcmsMatrix3x3),
    Matrix3x4(SkcmsMatrix3x4),
    A2B(&'a SkcmsA2B),
    B2A(&'a SkcmsB2A),
}

#[derive(Clone, Copy)]
pub struct OpAndArg<'a> {
    op: Op,
    arg: OpArg<'a>,
}

// ~~~~ Vector-shaped math helpers ~~~~

#[inline]
fn min_(x: f32, y: f32) -> f32 {
    // NEON `vminq_f32` propagates NaN and returns -0.0 for min(+0.0, -0.0).
    if x.is_nan() {
        x
    } else if y.is_nan() {
        y
    } else if x == 0.0 && y == 0.0 {
        if x.is_sign_negative() { x } else { y }
    } else if x > y {
        y
    } else {
        x
    }
}

#[inline]
fn max_(x: f32, y: f32) -> f32 {
    // NEON `vmaxq_f32` propagates NaN and returns +0.0 for max(+0.0, -0.0).
    if x.is_nan() {
        x
    } else if y.is_nan() {
        y
    } else if x == 0.0 && y == 0.0 {
        if x.is_sign_negative() { y } else { x }
    } else if x < y {
        y
    } else {
        x
    }
}

#[inline]
fn floor_(x: f32) -> f32 {
    // aarch64 uses `vrndmq_f32`.
    x.floor()
}

fn approx_log2(x: f32) -> f32 {
    let bits = x.to_bits() as i32;

    let e = bits as f32 * (1.0 / (1 << 23) as f32);

    let m = f32::from_bits(((bits & 0x007F_FFFF) | 0x3F00_0000) as u32);

    e - 124.225_514_990 - 1.498_030_302 * m - 1.725_879_990 / (0.352_088_706_8 + m)
}

fn approx_log(x: f32) -> f32 {
    const LN2: f32 = 0.693_147_18;
    LN2 * approx_log2(x)
}

fn approx_exp2(x: f32) -> f32 {
    let fract = x - floor_(x);

    let fbits = (1.0 * (1 << 23) as f32)
        * (x + 121.274_057_500 - 1.490_129_070 * fract + 27.728_023_300 / (4.842_525_68 - fract));
    // `FInfBits` is the integer 0x7f800000 converted to float (2139095040.0 == FLT_MAX).
    let bits = min_(max_(fbits, 0.0), 2139095040.0);

    f32::from_bits(bits as i32 as u32)
}

fn approx_pow(x: f32, y: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        x
    } else {
        approx_exp2(approx_log2(x) * y)
    }
}

fn approx_exp(x: f32) -> f32 {
    const LOG2_E: f32 = 1.442_695_040_888_963_4;
    approx_exp2(LOG2_E * x)
}

#[inline]
fn strip_sign(x: f32) -> (f32, u32) {
    let sign = x.to_bits() & 0x8000_0000;
    (f32::from_bits(x.to_bits() ^ sign), sign)
}

#[inline]
fn apply_sign(x: f32, sign: u32) -> f32 {
    f32::from_bits(sign | x.to_bits())
}

fn apply_tf(tf: &SkcmsTransferFunction, x: f32) -> f32 {
    let (x, sign) = strip_sign(x);

    let v = if x < tf.d {
        tf.c * x + tf.f
    } else {
        approx_pow(tf.a * x + tf.b, tf.g) + tf.e
    };

    apply_sign(v, sign)
}

fn apply_gamma(tf: &SkcmsTransferFunction, x: f32) -> f32 {
    let (x, sign) = strip_sign(x);
    apply_sign(approx_pow(x, tf.g), sign)
}

fn apply_pq(tf: &SkcmsTransferFunction, x: f32) -> f32 {
    let sign = x.to_bits() & 0x8000_0000;
    let x = f32::from_bits(x.to_bits() ^ sign);

    let v = approx_pow(
        max_(tf.a + tf.b * approx_pow(x, tf.c), 0.0) / (tf.d + tf.e * approx_pow(x, tf.c)),
        tf.f,
    );

    apply_sign(v, sign)
}

fn apply_hlg(tf: &SkcmsTransferFunction, x: f32) -> f32 {
    let r = tf.a;
    let g = tf.b;
    let a = tf.c;
    let b = tf.d;
    let c = tf.e;
    let k = tf.f + 1.0;
    let sign = x.to_bits() & 0x8000_0000;
    let x = f32::from_bits(x.to_bits() ^ sign);

    let v = if x * r <= 1.0 {
        approx_pow(x * r, g)
    } else {
        approx_exp((x - c) * a) + b
    };

    k * apply_sign(v, sign)
}

fn apply_hlginv(tf: &SkcmsTransferFunction, x: f32) -> f32 {
    let r = tf.a;
    let g = tf.b;
    let a = tf.c;
    let b = tf.d;
    let c = tf.e;
    let k = tf.f + 1.0;
    let sign = x.to_bits() & 0x8000_0000;
    let mut x = f32::from_bits(x.to_bits() ^ sign);
    x /= k;

    let v = if x <= 1.0 {
        r * approx_pow(x, g)
    } else {
        a * approx_log(x - b) + c
    };

    apply_sign(v, sign)
}

/// `compute_Y_in_xyzd50`.
fn compute_y_in_xyzd50(x: f32, y: f32, z: f32) -> f32 {
    -0.028_316_55 * x + 1.009_954_52 * y + 0.021_023_82 * z
}

#[inline]
fn f_from_u8(v: u8) -> f32 {
    v as f32 * (1.0 / 255.0)
}

#[inline]
fn f_from_u16(v: u16) -> f32 {
    v as f32 * (1.0 / 65535.0)
}

#[inline]
fn u16_from_f(v: f32) -> u16 {
    (v * 65535.0 + 0.5) as u16
}

/// `F_from_Half`.
///
/// The reference's Apple Silicon build takes the `USING_NEON_F16C` path
/// (`vcvt_f32_f16`), so this port performs the same exact IEEE half-to-single conversion:
/// denormal halves are preserved (not flushed), and infinities/NaNs keep their payloads.
fn f_from_half(half: u16) -> f32 {
    let sign = ((half as u32) & 0x8000) << 16;
    let exp = ((half >> 10) & 0x1F) as u32;
    let mant = (half & 0x3FF) as u32;

    match exp {
        0 => {
            if mant == 0 {
                f32::from_bits(sign)
            } else {
                // Denormal half: mant * 2^-24, exactly representable in f32.
                let value = mant as f32 * (1.0 / 16_777_216.0);
                if sign != 0 { -value } else { value }
            }
        }
        0x1F => f32::from_bits(sign | 0x7F80_0000 | (mant << 13)),
        _ => f32::from_bits(sign | ((exp + 112) << 23) | (mant << 13)),
    }
}

/// `Half_from_F`, matching `vcvt_f16_f32` (round to nearest, ties to even).
fn half_from_f(f: f32) -> u16 {
    let bits = f.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let mant = bits & 0x007F_FFFF;

    if exp == 0xFF {
        return sign
            | 0x7C00
            | if mant != 0 {
                0x0200 | (mant >> 13) as u16
            } else {
                0
            };
    }

    let half_exp = exp - 127 + 15;
    if half_exp >= 0x1F {
        return sign | 0x7C00;
    }

    if half_exp <= 0 {
        // Result is a denormal half (or zero).
        if half_exp < -10 {
            return sign;
        }
        let mant = mant | 0x0080_0000;
        let shift = (14 - half_exp) as u32;
        let half_mant = mant >> shift;
        let round_bit = 1u32 << (shift - 1);
        let round_up =
            (mant & round_bit) != 0 && ((mant & (round_bit - 1)) != 0 || (half_mant & 1) != 0);
        return sign | (half_mant + round_up as u32) as u16;
    }

    let half_mant = mant >> 13;
    let rem = mant & 0x1FFF;
    let round_up = rem > 0x1000 || (rem == 0x1000 && (half_mant & 1) != 0);
    // Rounding up the largest normal half overflows to infinity.
    if half_exp == 0x1E && half_mant == 0x3FF && round_up {
        return sign | 0x7C00;
    }
    sign | (((half_exp as u32 + round_up as u32) << 10) as u16) | half_mant as u16
}

#[inline]
fn to_fixed(f: f32) -> u32 {
    (f + 0.5) as i32 as u32
}

fn minus_1_ulp(v: f32) -> f32 {
    f32::from_bits(v.to_bits().wrapping_sub(1))
}

fn table(curve: &SkcmsCurve, v: f32) -> f32 {
    let entries = curve.table_entries;
    let ix = max_(0.0, min_(v, 1.0)) * (entries - 1) as f32;

    let lo = ix as i32;
    let hi = minus_1_ulp(ix + 1.0) as i32;
    let t = ix - lo as f32;

    let (l, h) = match &curve.table {
        Some(SkcmsCurveTable::U8(bytes)) => {
            let get = |i: i32| f_from_u8(bytes.get(i as usize).copied().unwrap_or(0));
            (get(lo), get(hi))
        }
        Some(SkcmsCurveTable::U16(values)) => {
            let get = |i: i32| f_from_u16(values.get(i as usize).copied().unwrap_or(0));
            (get(lo), get(hi))
        }
        None => (0.0, 0.0),
    };
    l + (h - l) * t
}

// ~~~~ CLUT sampling ~~~~

fn sample_clut_8_3(grid: &[u8], ix: i32) -> (f32, f32, f32) {
    let base = ix as usize * 3;
    let get = |i: usize| f_from_u8(grid.get(base + i).copied().unwrap_or(0));
    (get(0), get(1), get(2))
}

fn sample_clut_8_4(grid: &[u8], ix: i32) -> (f32, f32, f32, f32) {
    let base = ix as usize * 4;
    let get = |i: usize| f_from_u8(grid.get(base + i).copied().unwrap_or(0));
    (get(0), get(1), get(2), get(3))
}

fn sample_clut_16_3(grid: &[u16], ix: i32) -> (f32, f32, f32) {
    let base = ix as usize * 3;
    // `SkcmsGrid::U16` already holds native-endian values; the reference swaps its raw ICC bytes
    // at sampling time, but this port swaps once during parsing.
    let get = |i: usize| f_from_u16(grid.get(base + i).copied().unwrap_or(0));
    (get(0), get(1), get(2))
}

fn sample_clut_16_4(grid: &[u16], ix: i32) -> (f32, f32, f32, f32) {
    let base = ix as usize * 4;
    let get = |i: usize| f_from_u16(grid.get(base + i).copied().unwrap_or(0));
    (get(0), get(1), get(2), get(3))
}

#[allow(clippy::too_many_arguments)]
fn clut(
    input_channels: u32,
    output_channels: u32,
    grid_points: &[u8; 4],
    grid: Option<&SkcmsGrid>,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
    a: &mut f32,
) {
    let dim = input_channels as i32;
    if !(1..=4).contains(&dim) {
        return;
    }
    debug_assert!(output_channels == 3 || output_channels == 4);

    let mut index = [0i32; 8];
    let mut weight = [0.0f32; 8];

    let inputs = [*r, *g, *b, *a];
    let mut i = dim - 1;
    let mut stride = 1i32;
    while i >= 0 {
        let x = max_(0.0, min_(inputs[i as usize], 1.0))
            * ((grid_points[i as usize] as i32 - 1) as f32);

        let lo = x as i32;
        let hi = minus_1_ulp(x + 1.0) as i32;
        index[i as usize] = lo * stride;
        index[i as usize + 4] = hi * stride;
        stride *= grid_points[i as usize] as i32;

        let t = x - lo as f32;
        weight[i as usize] = 1.0 - t;
        weight[i as usize + 4] = t;
        i -= 1;
    }

    *r = 0.0;
    *g = 0.0;
    *b = 0.0;
    if output_channels == 4 {
        *a = 0.0;
    }

    for combo in 0..(1 << dim) {
        let mut ix = index[(combo & 1) * 4];
        let mut w = weight[(combo & 1) * 4];

        match (dim - 1) & 3 {
            3 => {
                ix += index[3 + (combo & 8) / 2];
                w *= weight[3 + (combo & 8) / 2];
                ix += index[2 + (combo & 4)];
                w *= weight[2 + (combo & 4)];
                ix += index[1 + (combo & 2) * 2];
                w *= weight[1 + (combo & 2) * 2];
            }
            2 => {
                ix += index[2 + (combo & 4)];
                w *= weight[2 + (combo & 4)];
                ix += index[1 + (combo & 2) * 2];
                w *= weight[1 + (combo & 2) * 2];
            }
            1 => {
                ix += index[1 + (combo & 2) * 2];
                w *= weight[1 + (combo & 2) * 2];
            }
            _ => {}
        }

        let (rr, gg, bb, aa) = match grid {
            Some(SkcmsGrid::U8(bytes)) => {
                if output_channels == 3 {
                    let (rr, gg, bb) = sample_clut_8_3(bytes, ix);
                    (rr, gg, bb, 0.0)
                } else {
                    sample_clut_8_4(bytes, ix)
                }
            }
            Some(SkcmsGrid::U16(values)) => {
                if output_channels == 3 {
                    let (rr, gg, bb) = sample_clut_16_3(values, ix);
                    (rr, gg, bb, 0.0)
                } else {
                    sample_clut_16_4(values, ix)
                }
            }
            None => (0.0, 0.0, 0.0, 0.0),
        };

        *r += w * rr;
        *g += w * gg;
        *b += w * bb;
        *a += w * aa;
    }
}

// ~~~~ Program selection ~~~~

fn tf_is_gamma(tf: &SkcmsTransferFunction) -> bool {
    tf.g > 0.0
        && tf.a == 1.0
        && tf.b == 0.0
        && tf.c == 0.0
        && tf.d == 0.0
        && tf.e == 0.0
        && tf.f == 0.0
}

fn select_curve_op<'a>(curve: &'a SkcmsCurve, channel: usize) -> OpAndArg<'a> {
    const OPS: [[Op; 6]; 4] = [
        [
            Op::GammaR,
            Op::TfR,
            Op::PqR,
            Op::HlgR,
            Op::HlginvR,
            Op::TableR,
        ],
        [
            Op::GammaG,
            Op::TfG,
            Op::PqG,
            Op::HlgG,
            Op::HlginvG,
            Op::TableG,
        ],
        [
            Op::GammaB,
            Op::TfB,
            Op::PqB,
            Op::HlgB,
            Op::HlginvB,
            Op::TableB,
        ],
        [
            Op::GammaA,
            Op::TfA,
            Op::PqA,
            Op::HlgA,
            Op::HlginvA,
            Op::TableA,
        ],
    ];
    let op = &OPS[channel];

    if curve.table_entries == 0 {
        let tf = &curve.parametric;

        if tf_is_gamma(tf) {
            return if tf.g != 1.0 {
                OpAndArg {
                    op: op[0],
                    arg: OpArg::Tf(*tf),
                }
            } else {
                OpAndArg {
                    op: Op::Clamp,
                    arg: OpArg::None,
                }
            };
        }

        match skcms_transfer_function_get_type(tf) {
            SkcmsTfType::Invalid | SkcmsTfType::Pq | SkcmsTfType::Hlg => OpAndArg {
                op: Op::Clamp,
                arg: OpArg::None,
            },
            SkcmsTfType::Srgbish => OpAndArg {
                op: op[1],
                arg: OpArg::Tf(*tf),
            },
            SkcmsTfType::Pqish => OpAndArg {
                op: op[2],
                arg: OpArg::Tf(*tf),
            },
            SkcmsTfType::Hlgish => OpAndArg {
                op: op[3],
                arg: OpArg::Tf(*tf),
            },
            SkcmsTfType::Hlginvish => OpAndArg {
                op: op[4],
                arg: OpArg::Tf(*tf),
            },
        }
    } else {
        // The table_* ops make this assumption.
        debug_assert!(curve.table_entries <= (1 << 24));
        OpAndArg {
            op: op[5],
            arg: OpArg::Curve(curve),
        }
    }
}

fn arg_tf_bits_eq(a: OpArg, b: OpArg) -> bool {
    match (a, b) {
        (OpArg::Tf(x), OpArg::Tf(y)) => x.bits_eq(&y),
        (OpArg::Curve(x), OpArg::Curve(y)) => std::ptr::eq(x, y),
        _ => false,
    }
}

/// Returns negative if any of the curves are malformed.
fn select_curve_ops<'a>(
    curves: &'a [SkcmsCurve],
    num_channels: usize,
) -> Option<Vec<OpAndArg<'a>>> {
    // We process the channels in reverse order, yielding ops in ABGR order.
    let mut ops: Vec<OpAndArg<'a>> = Vec::with_capacity(4);
    for index in (0..num_channels).rev() {
        if curves[index].table_entries > (1 << 24) {
            return None;
        }
        let op = select_curve_op(&curves[index], index);
        if !matches!(op.arg, OpArg::None) {
            ops.push(op);
        }
    }

    // Identify separate B+G+R ops and fuse them into a single RGB op.
    if ops.len() >= 3 {
        const FUSABLE: [(Op, Op, Op, Op); 5] = [
            (Op::GammaR, Op::GammaG, Op::GammaB, Op::GammaRgb),
            (Op::TfR, Op::TfG, Op::TfB, Op::TfRgb),
            (Op::PqR, Op::PqG, Op::PqB, Op::PqRgb),
            (Op::HlgR, Op::HlgG, Op::HlgB, Op::HlgRgb),
            (Op::HlginvR, Op::HlginvG, Op::HlginvB, Op::HlginvRgb),
        ];

        let cursor = ops.len();
        let pos_r = cursor - 1;
        let pos_g = cursor - 2;
        let pos_b = cursor - 3;
        for &(r, g, b, rgb) in &FUSABLE {
            if ops[pos_r].op == r
                && ops[pos_g].op == g
                && ops[pos_b].op == b
                && arg_tf_bits_eq(ops[pos_r].arg, ops[pos_g].arg)
                && arg_tf_bits_eq(ops[pos_r].arg, ops[pos_b].arg)
            {
                ops[pos_b].op = rgb;
                ops.truncate(cursor - 2);
                break;
            }
        }
    }

    Some(ops)
}

struct Program<'a> {
    ops: Vec<OpAndArg<'a>>,
    count: usize,
}

impl<'a> Program<'a> {
    fn new() -> Self {
        Self {
            ops: Vec::with_capacity(SKCMS_MAX_PROGRAM_OPS),
            count: 0,
        }
    }

    fn add(&mut self, op: Op) {
        if self.ops.len() < SKCMS_MAX_PROGRAM_OPS {
            self.ops.push(OpAndArg {
                op,
                arg: OpArg::None,
            });
        }
        self.count += 1;
    }

    fn add_ctx(&mut self, op: Op, arg: OpArg<'a>) {
        if self.ops.len() < SKCMS_MAX_PROGRAM_OPS {
            self.ops.push(OpAndArg { op, arg });
        }
        self.count += 1;
    }

    fn add_curve_ops(&mut self, curves: &'a [SkcmsCurve], num_channels: usize) -> bool {
        let Some(ops) = select_curve_ops(curves, num_channels) else {
            return false;
        };
        for oa in ops {
            self.add_ctx(oa.op, oa.arg);
        }
        true
    }
}

// ~~~~ skcms_Transform ~~~~

/// `skcms_Transform`.
#[allow(clippy::too_many_arguments)]
pub fn skcms_transform(
    src: &[u8],
    src_fmt: SkcmsPixelFormat,
    mut src_alpha: SkcmsAlphaFormat,
    src_profile: Option<&SkcmsIccProfile>,
    dst: &mut [u8],
    dst_fmt: SkcmsPixelFormat,
    mut dst_alpha: SkcmsAlphaFormat,
    dst_profile: Option<&SkcmsIccProfile>,
    nz: usize,
) -> bool {
    let src_bpp = skcms_bytes_per_pixel(src_fmt);
    let dst_bpp = skcms_bytes_per_pixel(dst_fmt);
    // Let's just refuse if the request is absurdly big.
    if nz.saturating_mul(dst_bpp) > i32::MAX as usize
        || nz.saturating_mul(src_bpp) > i32::MAX as usize
    {
        return false;
    }
    if src.len() < nz * src_bpp || dst.len() < nz * dst_bpp {
        return false;
    }

    let src_profile = src_profile.unwrap_or(skcms_srgb_profile());
    let dst_profile = dst_profile.unwrap_or(skcms_srgb_profile());

    let mut program: Program = Program::new();

    // If the source has a TRC that is specified by CICP and not the TRC entries, then store it
    // here for future use.
    let mut src_cicp_trc = SkcmsTransferFunction::default();

    // These are always parametric curves of some sort.
    let mut dst_curves = [
        SkcmsCurve::default(),
        SkcmsCurve::default(),
        SkcmsCurve::default(),
    ];

    match src_fmt {
        SkcmsPixelFormat::A8 | SkcmsPixelFormat::A8Alt => program.add(Op::LoadA8),
        SkcmsPixelFormat::G8 | SkcmsPixelFormat::G8Alt => program.add(Op::LoadG8),
        SkcmsPixelFormat::Ga88 | SkcmsPixelFormat::Ga88Alt => program.add(Op::LoadGa88),
        SkcmsPixelFormat::Abgr4444 | SkcmsPixelFormat::Argb4444 => program.add(Op::Load4444),
        SkcmsPixelFormat::Rgb565 | SkcmsPixelFormat::Bgr565 => program.add(Op::Load565),
        SkcmsPixelFormat::Rgb888 | SkcmsPixelFormat::Bgr888 => program.add(Op::Load888),
        SkcmsPixelFormat::Rgba8888 | SkcmsPixelFormat::Bgra8888 => program.add(Op::Load8888),
        SkcmsPixelFormat::Rgba1010102 | SkcmsPixelFormat::Bgra1010102 => {
            program.add(Op::Load1010102)
        }
        SkcmsPixelFormat::Rgb101010xXr | SkcmsPixelFormat::Bgr101010xXr => {
            program.add(Op::Load101010xXr)
        }
        SkcmsPixelFormat::Rgba10101010Xr | SkcmsPixelFormat::Bgra10101010Xr => {
            program.add(Op::Load10101010Xr)
        }
        SkcmsPixelFormat::Rgb161616Le | SkcmsPixelFormat::Bgr161616Le => {
            program.add(Op::Load161616Le)
        }
        SkcmsPixelFormat::Rgba16161616Le | SkcmsPixelFormat::Bgra16161616Le => {
            program.add(Op::Load16161616Le)
        }
        SkcmsPixelFormat::Rgb161616Be | SkcmsPixelFormat::Bgr161616Be => {
            program.add(Op::Load161616Be)
        }
        SkcmsPixelFormat::Rgba16161616Be | SkcmsPixelFormat::Bgra16161616Be => {
            program.add(Op::Load16161616Be)
        }
        SkcmsPixelFormat::RgbHhhNorm | SkcmsPixelFormat::BgrHhhNorm => program.add(Op::LoadHhh),
        SkcmsPixelFormat::RgbaHhhhNorm | SkcmsPixelFormat::BgraHhhhNorm => {
            program.add(Op::LoadHhhh)
        }
        SkcmsPixelFormat::RgbHhh | SkcmsPixelFormat::BgrHhh => program.add(Op::LoadHhh),
        SkcmsPixelFormat::RgbaHhhh | SkcmsPixelFormat::BgraHhhh => program.add(Op::LoadHhhh),
        SkcmsPixelFormat::RgbFff | SkcmsPixelFormat::BgrFff => program.add(Op::LoadFff),
        SkcmsPixelFormat::RgbaFfff | SkcmsPixelFormat::BgraFfff => program.add(Op::LoadFfff),
        SkcmsPixelFormat::Rgba8888Srgb | SkcmsPixelFormat::Bgra8888Srgb => {
            program.add(Op::Load8888);
            program.add_ctx(Op::TfRgb, OpArg::Tf(*skcms_srgb_transfer_function()));
        }
    }
    if src_fmt == SkcmsPixelFormat::RgbHhhNorm || src_fmt == SkcmsPixelFormat::RgbaHhhhNorm {
        program.add(Op::Clamp);
    }
    if src_fmt.swaps_rb() {
        program.add(Op::SwapRb);
    }

    // When transforming to gray, stop at XYZ (by setting toXYZ to identity), then transform
    // luminance (Y) by the destination transfer function.
    let gray_dst_profile = match dst_fmt {
        SkcmsPixelFormat::G8
        | SkcmsPixelFormat::G8Alt
        | SkcmsPixelFormat::Ga88
        | SkcmsPixelFormat::Ga88Alt => {
            let mut profile = dst_profile.clone();
            profile.set_xyzd50(&crate::skcms::skcms_xyzd50_profile().to_xyzd50);
            Some(profile)
        }
        _ => None,
    };
    let dst_profile: &SkcmsIccProfile = gray_dst_profile.as_ref().unwrap_or(dst_profile);

    if src_profile.data_color_space == crate::skcms::SKCMS_SIGNATURE_CMYK {
        // Photoshop creates CMYK images as inverse CMYK.
        program.add(Op::Invert);
        // With CMYK, ignore the alpha type.
        src_alpha = SkcmsAlphaFormat::Unpremul;
    }

    match src_alpha {
        SkcmsAlphaFormat::Opaque => program.add(Op::ForceOpaque),
        SkcmsAlphaFormat::PremulAsEncoded => program.add(Op::Unpremul),
        SkcmsAlphaFormat::Unpremul => {}
    }

    if !std::ptr::eq(dst_profile, src_profile) {
        // Track whether or not the A2B or B2A transforms are used.
        let mut src_using_a2b = false;
        let mut src_using_hlg_ootf = false;

        let Some(destination) = skcms_prep_for_destination(dst_profile) else {
            return false;
        };
        let dst_using_b2a = destination.using_b2a;
        let dst_using_hlg_ootf = destination.using_hlg_ootf;
        let dst_from_xyz_matrix = destination.from_xyzd50;
        let dst_inv = destination.inv;

        for i in 0..3 {
            dst_curves[i].table_entries = 0;
            dst_curves[i].parametric = dst_inv[i];
        }

        if crate::skcms::skcms_has_cicp_pq_trc(src_profile) && src_profile.has_to_xyzd50 {
            skcms_set_reference_pq_ish_trc(&mut src_cicp_trc);
            program.add_ctx(Op::PqRgb, OpArg::Tf(src_cicp_trc));
        } else if crate::skcms::skcms_has_cicp_hlg_trc(src_profile) && src_profile.has_to_xyzd50 {
            src_using_hlg_ootf = true;
            skcms_set_sdr_hlg_ish_trc(&mut src_cicp_trc);
            program.add_ctx(Op::HlgRgb, OpArg::Tf(src_cicp_trc));
        } else if src_profile.has_a2b {
            src_using_a2b = true;
            if src_profile.a2b.input_channels != 0 {
                if !program.add_curve_ops(
                    &src_profile.a2b.input_curves,
                    src_profile.a2b.input_channels as usize,
                ) {
                    return false;
                }
                program.add(Op::Clamp);
                program.add_ctx(Op::ClutA2B, OpArg::A2B(&src_profile.a2b));
            }

            if src_profile.a2b.matrix_channels == 3 {
                if !program.add_curve_ops(&src_profile.a2b.matrix_curves, 3) {
                    return false;
                }

                let identity = SkcmsMatrix3x4 {
                    vals: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                    ],
                };
                if !identity.bits_eq(&src_profile.a2b.matrix) {
                    program.add_ctx(Op::Matrix3x4, OpArg::Matrix3x4(src_profile.a2b.matrix));
                }
            }

            if src_profile.a2b.output_channels == 3 {
                if !program.add_curve_ops(&src_profile.a2b.output_curves, 3) {
                    return false;
                }
            }

            if src_profile.pcs == crate::skcms::SKCMS_SIGNATURE_LAB {
                program.add(Op::LabToXyz);
            }
        } else if src_profile.has_trc && src_profile.has_to_xyzd50 {
            if !program.add_curve_ops(&src_profile.trc, 3) {
                return false;
            }
        } else {
            return false;
        }

        // A2B sources are in XYZD50 by now, but TRC sources are still in their original gamut.
        debug_assert!(src_profile.has_a2b || src_profile.has_to_xyzd50);

        if dst_using_b2a {
            // B2A needs its input in XYZD50, so transform TRC sources now.
            if !src_using_a2b {
                program.add_ctx(Op::Matrix3x3, OpArg::Matrix3x3(src_profile.to_xyzd50));
                // Apply the HLG OOTF in XYZD50 space, if needed.
                if src_using_hlg_ootf {
                    program.add(Op::HlgOotfScale);
                }
            }

            if dst_profile.pcs == crate::skcms::SKCMS_SIGNATURE_LAB {
                program.add(Op::XyzToLab);
            }

            if dst_profile.b2a.input_channels == 3 {
                if !program.add_curve_ops(&dst_profile.b2a.input_curves, 3) {
                    return false;
                }
            }

            if dst_profile.b2a.matrix_channels == 3 {
                let identity = SkcmsMatrix3x4 {
                    vals: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                    ],
                };
                if !identity.bits_eq(&dst_profile.b2a.matrix) {
                    program.add_ctx(Op::Matrix3x4, OpArg::Matrix3x4(dst_profile.b2a.matrix));
                }

                if !program.add_curve_ops(&dst_profile.b2a.matrix_curves, 3) {
                    return false;
                }
            }

            if dst_profile.b2a.output_channels != 0 {
                program.add(Op::Clamp);
                program.add_ctx(Op::ClutB2A, OpArg::B2A(&dst_profile.b2a));

                if !program.add_curve_ops(
                    &dst_profile.b2a.output_curves,
                    dst_profile.b2a.output_channels as usize,
                ) {
                    return false;
                }
            }
        } else {
            // This is a TRC destination.
            if src_using_hlg_ootf != dst_using_hlg_ootf {
                // If just the src or the dst has an HLG OOTF then we apply the OOTF in XYZD50
                // space. If both have an HLG OOTF then they cancel.
                if !src_using_a2b {
                    program.add_ctx(Op::Matrix3x3, OpArg::Matrix3x3(src_profile.to_xyzd50));
                }
                if src_using_hlg_ootf {
                    program.add(Op::HlgOotfScale);
                }
                if dst_using_hlg_ootf {
                    program.add(Op::HlginvOotfScale);
                }
                program.add_ctx(Op::Matrix3x3, OpArg::Matrix3x3(dst_from_xyz_matrix));
            } else if src_using_a2b {
                // If the source is A2B then we are already in XYZD50. Just apply the xyz->dst
                // matrix.
                program.add_ctx(Op::Matrix3x3, OpArg::Matrix3x3(dst_from_xyz_matrix));
            } else {
                let to_xyz = &src_profile.to_xyzd50;
                // There's a chance the source and destination gamuts are identical, in which case
                // we can skip the gamut transform.
                if !dst_profile.to_xyzd50.bits_eq(to_xyz) {
                    // Concat the entire gamut transform into dst_from_src.
                    let dst_from_src =
                        crate::skcms::skcms_matrix3x3_concat(&dst_from_xyz_matrix, to_xyz);
                    program.add_ctx(Op::Matrix3x3, OpArg::Matrix3x3(dst_from_src));
                }
            }

            // Encode back to dst RGB using its parametric transfer functions.
            let Some(oa) = select_curve_ops(&dst_curves, 3) else {
                debug_assert!(false);
                return false;
            };
            for entry in oa {
                debug_assert!(!matches!(
                    entry.op,
                    Op::TableR | Op::TableG | Op::TableB | Op::TableA
                ));
                program.add_ctx(entry.op, entry.arg);
            }
        }
    }

    // Clamp here before premul to make sure we're clamping to normalized values and gamut, not
    // just to values that fit in [0,1].
    if (dst_fmt as u32) < (SkcmsPixelFormat::RgbHhh as u32) {
        program.add(Op::Clamp);
    }

    if dst_profile.data_color_space == crate::skcms::SKCMS_SIGNATURE_CMYK {
        // Photoshop creates CMYK images as inverse CMYK.
        program.add(Op::Invert);
        // CMYK has no alpha channel, so make sure dstAlpha is a no-op.
        dst_alpha = SkcmsAlphaFormat::Unpremul;
    }

    match dst_alpha {
        SkcmsAlphaFormat::Opaque => program.add(Op::ForceOpaque),
        SkcmsAlphaFormat::PremulAsEncoded => program.add(Op::Premul),
        SkcmsAlphaFormat::Unpremul => {}
    }
    if dst_fmt.swaps_rb() {
        program.add(Op::SwapRb);
    }
    match dst_fmt {
        SkcmsPixelFormat::A8 | SkcmsPixelFormat::A8Alt => program.add(Op::StoreA8),
        SkcmsPixelFormat::G8 | SkcmsPixelFormat::G8Alt => program.add(Op::StoreG8),
        SkcmsPixelFormat::Ga88 | SkcmsPixelFormat::Ga88Alt => program.add(Op::StoreGa88),
        SkcmsPixelFormat::Abgr4444 | SkcmsPixelFormat::Argb4444 => program.add(Op::Store4444),
        SkcmsPixelFormat::Rgb565 | SkcmsPixelFormat::Bgr565 => program.add(Op::Store565),
        SkcmsPixelFormat::Rgb888 | SkcmsPixelFormat::Bgr888 => program.add(Op::Store888),
        SkcmsPixelFormat::Rgba8888 | SkcmsPixelFormat::Bgra8888 => program.add(Op::Store8888),
        SkcmsPixelFormat::Rgba1010102 | SkcmsPixelFormat::Bgra1010102 => {
            program.add(Op::Store1010102)
        }
        SkcmsPixelFormat::Rgb101010xXr | SkcmsPixelFormat::Bgr101010xXr => {
            program.add(Op::Store101010xXr)
        }
        SkcmsPixelFormat::Rgba10101010Xr | SkcmsPixelFormat::Bgra10101010Xr => {
            program.add(Op::Store10101010Xr)
        }
        SkcmsPixelFormat::Rgb161616Le | SkcmsPixelFormat::Bgr161616Le => {
            program.add(Op::Store161616Le)
        }
        SkcmsPixelFormat::Rgba16161616Le | SkcmsPixelFormat::Bgra16161616Le => {
            program.add(Op::Store16161616Le)
        }
        SkcmsPixelFormat::Rgb161616Be | SkcmsPixelFormat::Bgr161616Be => {
            program.add(Op::Store161616Be)
        }
        SkcmsPixelFormat::Rgba16161616Be | SkcmsPixelFormat::Bgra16161616Be => {
            program.add(Op::Store16161616Be)
        }
        SkcmsPixelFormat::RgbHhhNorm | SkcmsPixelFormat::BgrHhhNorm => program.add(Op::StoreHhh),
        SkcmsPixelFormat::RgbaHhhhNorm | SkcmsPixelFormat::BgraHhhhNorm => {
            program.add(Op::StoreHhhh)
        }
        SkcmsPixelFormat::RgbHhh | SkcmsPixelFormat::BgrHhh => program.add(Op::StoreHhh),
        SkcmsPixelFormat::RgbaHhhh | SkcmsPixelFormat::BgraHhhh => program.add(Op::StoreHhhh),
        SkcmsPixelFormat::RgbFff | SkcmsPixelFormat::BgrFff => program.add(Op::StoreFff),
        SkcmsPixelFormat::RgbaFfff | SkcmsPixelFormat::BgraFfff => program.add(Op::StoreFfff),
        SkcmsPixelFormat::Rgba8888Srgb | SkcmsPixelFormat::Bgra8888Srgb => {
            program.add_ctx(
                Op::TfRgb,
                OpArg::Tf(*skcms_srgb_inverse_transfer_function()),
            );
            program.add(Op::Store8888);
        }
    }

    if program.count > SKCMS_MAX_PROGRAM_OPS {
        return false;
    }

    run_program(&program.ops, src, dst, nz, src_bpp, dst_bpp);
    true
}

fn read_u16_le(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([
        bytes.get(at).copied().unwrap_or(0),
        bytes.get(at + 1).copied().unwrap_or(0),
    ])
}

fn read_u16_be(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([
        bytes.get(at).copied().unwrap_or(0),
        bytes.get(at + 1).copied().unwrap_or(0),
    ])
}

fn read_u32_le(bytes: &[u8], at: usize) -> u32 {
    let mut b = [0u8; 4];
    for (i, slot) in b.iter_mut().enumerate() {
        *slot = bytes.get(at + i).copied().unwrap_or(0);
    }
    u32::from_le_bytes(b)
}

fn read_u64_le(bytes: &[u8], at: usize) -> u64 {
    let mut b = [0u8; 8];
    for (i, slot) in b.iter_mut().enumerate() {
        *slot = bytes.get(at + i).copied().unwrap_or(0);
    }
    u64::from_le_bytes(b)
}

fn write_u16_le(dst: &mut [u8], at: usize, value: u16) {
    let b = value.to_le_bytes();
    if let Some(slot) = dst.get_mut(at..at + 2) {
        slot.copy_from_slice(&b);
    }
}

fn write_u16_be(dst: &mut [u8], at: usize, value: u16) {
    let b = value.to_be_bytes();
    if let Some(slot) = dst.get_mut(at..at + 2) {
        slot.copy_from_slice(&b);
    }
}

fn write_u32_le(dst: &mut [u8], at: usize, value: u32) {
    let b = value.to_le_bytes();
    if let Some(slot) = dst.get_mut(at..at + 4) {
        slot.copy_from_slice(&b);
    }
}

fn write_u64_le(dst: &mut [u8], at: usize, value: u64) {
    let b = value.to_le_bytes();
    if let Some(slot) = dst.get_mut(at..at + 8) {
        slot.copy_from_slice(&b);
    }
}

fn write_f32_le(dst: &mut [u8], at: usize, value: f32) {
    write_u32_le(dst, at, value.to_bits());
}

/// Executes a selected program over `n` pixels.
fn run_program(
    program: &[OpAndArg],
    src: &[u8],
    dst: &mut [u8],
    n: usize,
    src_bpp: usize,
    dst_bpp: usize,
) {
    for i in 0..n {
        let mut r = 0.0f32;
        let mut g = 0.0f32;
        let mut b = 0.0f32;
        let mut a = 1.0f32;

        let src_at = i * src_bpp;
        let dst_at = i * dst_bpp;

        for entry in program {
            match entry.op {
                Op::LoadA8 => a = f_from_u8(src[src_at]),
                Op::LoadG8 => {
                    let v = f_from_u8(src[src_at]);
                    r = v;
                    g = v;
                    b = v;
                }
                Op::LoadGa88 => {
                    let v = read_u16_le(src, src_at);
                    let l = f_from_u8((v & 0xFF) as u8);
                    r = l;
                    g = l;
                    b = l;
                    a = f_from_u8((v >> 8) as u8);
                }
                Op::Load4444 => {
                    let v = read_u16_le(src, src_at);
                    r = ((v >> 12) & 0xF) as f32 * (1.0 / 15.0);
                    g = ((v >> 8) & 0xF) as f32 * (1.0 / 15.0);
                    b = ((v >> 4) & 0xF) as f32 * (1.0 / 15.0);
                    a = (v & 0xF) as f32 * (1.0 / 15.0);
                }
                Op::Load565 => {
                    let v = read_u16_le(src, src_at) as u32;
                    r = (v & (31 << 0)) as f32 * (1.0 / (31 << 0) as f32);
                    g = (v & (63 << 5)) as f32 * (1.0 / (63 << 5) as f32);
                    b = (v & (31 << 11)) as f32 * (1.0 / (31 << 11) as f32);
                }
                Op::Load888 => {
                    r = f_from_u8(src[src_at]);
                    g = f_from_u8(src[src_at + 1]);
                    b = f_from_u8(src[src_at + 2]);
                }
                Op::Load8888 => {
                    let v = read_u32_le(src, src_at);
                    r = ((v >> 0) & 0xFF) as f32 * (1.0 / 255.0);
                    g = ((v >> 8) & 0xFF) as f32 * (1.0 / 255.0);
                    b = ((v >> 16) & 0xFF) as f32 * (1.0 / 255.0);
                    a = ((v >> 24) & 0xFF) as f32 * (1.0 / 255.0);
                }
                Op::Load1010102 => {
                    let v = read_u32_le(src, src_at);
                    r = ((v >> 0) & 0x3FF) as f32 * (1.0 / 1023.0);
                    g = ((v >> 10) & 0x3FF) as f32 * (1.0 / 1023.0);
                    b = ((v >> 20) & 0x3FF) as f32 * (1.0 / 1023.0);
                    a = ((v >> 30) & 0x3) as f32 * (1.0 / 3.0);
                }
                Op::Load101010xXr => {
                    let v = read_u32_le(src, src_at);
                    r = (((v >> 0) & 0x3FF) as i32 - 384) as f32 / 510.0;
                    g = (((v >> 10) & 0x3FF) as i32 - 384) as f32 / 510.0;
                    b = (((v >> 20) & 0x3FF) as i32 - 384) as f32 / 510.0;
                }
                Op::Load10101010Xr => {
                    let v = read_u64_le(src, src_at);
                    r = (((v >> 6) & 0x3FF) as i32 - 384) as f32 / 510.0;
                    g = (((v >> 22) & 0x3FF) as i32 - 384) as f32 / 510.0;
                    b = (((v >> 38) & 0x3FF) as i32 - 384) as f32 / 510.0;
                    a = (((v >> 54) & 0x3FF) as i32 - 384) as f32 / 510.0;
                }
                Op::Load161616Le => {
                    r = read_u16_le(src, src_at) as f32 * (1.0 / 65535.0);
                    g = read_u16_le(src, src_at + 2) as f32 * (1.0 / 65535.0);
                    b = read_u16_le(src, src_at + 4) as f32 * (1.0 / 65535.0);
                }
                Op::Load16161616Le => {
                    r = read_u16_le(src, src_at) as f32 * (1.0 / 65535.0);
                    g = read_u16_le(src, src_at + 2) as f32 * (1.0 / 65535.0);
                    b = read_u16_le(src, src_at + 4) as f32 * (1.0 / 65535.0);
                    a = read_u16_le(src, src_at + 6) as f32 * (1.0 / 65535.0);
                }
                Op::Load161616Be => {
                    r = read_u16_be(src, src_at) as f32 * (1.0 / 65535.0);
                    g = read_u16_be(src, src_at + 2) as f32 * (1.0 / 65535.0);
                    b = read_u16_be(src, src_at + 4) as f32 * (1.0 / 65535.0);
                }
                Op::Load16161616Be => {
                    r = read_u16_be(src, src_at) as f32 * (1.0 / 65535.0);
                    g = read_u16_be(src, src_at + 2) as f32 * (1.0 / 65535.0);
                    b = read_u16_be(src, src_at + 4) as f32 * (1.0 / 65535.0);
                    a = read_u16_be(src, src_at + 6) as f32 * (1.0 / 65535.0);
                }
                Op::LoadHhh => {
                    r = f_from_half(read_u16_le(src, src_at));
                    g = f_from_half(read_u16_le(src, src_at + 2));
                    b = f_from_half(read_u16_le(src, src_at + 4));
                }
                Op::LoadHhhh => {
                    r = f_from_half(read_u16_le(src, src_at));
                    g = f_from_half(read_u16_le(src, src_at + 2));
                    b = f_from_half(read_u16_le(src, src_at + 4));
                    a = f_from_half(read_u16_le(src, src_at + 6));
                }
                Op::LoadFff => {
                    r = f32::from_bits(read_u32_le(src, src_at));
                    g = f32::from_bits(read_u32_le(src, src_at + 4));
                    b = f32::from_bits(read_u32_le(src, src_at + 8));
                }
                Op::LoadFfff => {
                    r = f32::from_bits(read_u32_le(src, src_at));
                    g = f32::from_bits(read_u32_le(src, src_at + 4));
                    b = f32::from_bits(read_u32_le(src, src_at + 8));
                    a = f32::from_bits(read_u32_le(src, src_at + 12));
                }

                Op::SwapRb => std::mem::swap(&mut r, &mut b),
                Op::Clamp => {
                    r = max_(0.0, min_(r, 1.0));
                    g = max_(0.0, min_(g, 1.0));
                    b = max_(0.0, min_(b, 1.0));
                    a = max_(0.0, min_(a, 1.0));
                }
                Op::Invert => {
                    r = 1.0 - r;
                    g = 1.0 - g;
                    b = 1.0 - b;
                    a = 1.0 - a;
                }
                Op::ForceOpaque => a = 1.0,
                Op::Premul => {
                    r *= a;
                    g *= a;
                    b *= a;
                }
                Op::Unpremul => {
                    let scale = if 1.0 / a < INFINITY_ { 1.0 / a } else { 0.0 };
                    r *= scale;
                    g *= scale;
                    b *= scale;
                }
                Op::Matrix3x3 => {
                    if let OpArg::Matrix3x3(m) = entry.arg {
                        let rr = m.vals[0][0] * r + m.vals[0][1] * g + m.vals[0][2] * b;
                        let gg = m.vals[1][0] * r + m.vals[1][1] * g + m.vals[1][2] * b;
                        let bb = m.vals[2][0] * r + m.vals[2][1] * g + m.vals[2][2] * b;
                        r = rr;
                        g = gg;
                        b = bb;
                    }
                }
                Op::Matrix3x4 => {
                    if let OpArg::Matrix3x4(m) = entry.arg {
                        let rr =
                            m.vals[0][0] * r + m.vals[0][1] * g + m.vals[0][2] * b + m.vals[0][3];
                        let gg =
                            m.vals[1][0] * r + m.vals[1][1] * g + m.vals[1][2] * b + m.vals[1][3];
                        let bb =
                            m.vals[2][0] * r + m.vals[2][1] * g + m.vals[2][2] * b + m.vals[2][3];
                        r = rr;
                        g = gg;
                        b = bb;
                    }
                }
                Op::LabToXyz => {
                    let l = r * 100.0;
                    let a_ = g * 255.0 - 128.0;
                    let b_ = b * 255.0 - 128.0;

                    let mut y = (l + 16.0) * (1.0 / 116.0);
                    let mut x = y + a_ * (1.0 / 500.0);
                    let mut z = y - b_ * (1.0 / 200.0);

                    x = if x * x * x > 0.008_856 {
                        x * x * x
                    } else {
                        (x - (16.0 / 116.0)) * (1.0 / 7.787)
                    };
                    y = if y * y * y > 0.008_856 {
                        y * y * y
                    } else {
                        (y - (16.0 / 116.0)) * (1.0 / 7.787)
                    };
                    z = if z * z * z > 0.008_856 {
                        z * z * z
                    } else {
                        (z - (16.0 / 116.0)) * (1.0 / 7.787)
                    };

                    r = x * 0.9642;
                    g = y;
                    b = z * 0.8249;
                }
                Op::XyzToLab => {
                    let mut x = r * (1.0 / 0.9642);
                    let mut y = g;
                    let mut z = b * (1.0 / 0.8249);

                    x = if x > 0.008_856 {
                        approx_pow(x, 1.0 / 3.0)
                    } else {
                        x * 7.787 + (16.0 / 116.0)
                    };
                    y = if y > 0.008_856 {
                        approx_pow(y, 1.0 / 3.0)
                    } else {
                        y * 7.787 + (16.0 / 116.0)
                    };
                    z = if z > 0.008_856 {
                        approx_pow(z, 1.0 / 3.0)
                    } else {
                        z * 7.787 + (16.0 / 116.0)
                    };

                    let l = y * 116.0 - 16.0;
                    let a_ = (x - y) * 500.0;
                    let b_ = (y - z) * 200.0;

                    r = l * (1.0 / 100.0);
                    g = (a_ + 128.0) * (1.0 / 255.0);
                    b = (b_ + 128.0) * (1.0 / 255.0);
                }

                Op::GammaR => r = apply_gamma(&tf_arg(entry), r),
                Op::GammaG => g = apply_gamma(&tf_arg(entry), g),
                Op::GammaB => b = apply_gamma(&tf_arg(entry), b),
                Op::GammaA => a = apply_gamma(&tf_arg(entry), a),
                Op::GammaRgb => {
                    let tf = tf_arg(entry);
                    r = apply_gamma(&tf, r);
                    g = apply_gamma(&tf, g);
                    b = apply_gamma(&tf, b);
                }

                Op::TfR => r = apply_tf(&tf_arg(entry), r),
                Op::TfG => g = apply_tf(&tf_arg(entry), g),
                Op::TfB => b = apply_tf(&tf_arg(entry), b),
                Op::TfA => a = apply_tf(&tf_arg(entry), a),
                Op::TfRgb => {
                    let tf = tf_arg(entry);
                    r = apply_tf(&tf, r);
                    g = apply_tf(&tf, g);
                    b = apply_tf(&tf, b);
                }

                Op::PqR => r = apply_pq(&tf_arg(entry), r),
                Op::PqG => g = apply_pq(&tf_arg(entry), g),
                Op::PqB => b = apply_pq(&tf_arg(entry), b),
                Op::PqA => a = apply_pq(&tf_arg(entry), a),
                Op::PqRgb => {
                    let tf = tf_arg(entry);
                    r = apply_pq(&tf, r);
                    g = apply_pq(&tf, g);
                    b = apply_pq(&tf, b);
                }

                Op::HlgR => r = apply_hlg(&tf_arg(entry), r),
                Op::HlgG => g = apply_hlg(&tf_arg(entry), g),
                Op::HlgB => b = apply_hlg(&tf_arg(entry), b),
                Op::HlgA => a = apply_hlg(&tf_arg(entry), a),
                Op::HlgRgb => {
                    let tf = tf_arg(entry);
                    r = apply_hlg(&tf, r);
                    g = apply_hlg(&tf, g);
                    b = apply_hlg(&tf, b);
                }

                Op::HlgOotfScale => {
                    let y = compute_y_in_xyzd50(r, g, b);
                    let (y, sign) = strip_sign(y);
                    let y_to_gamma_minus1 = apply_sign(approx_pow(y, 0.2), sign);
                    r *= y_to_gamma_minus1;
                    g *= y_to_gamma_minus1;
                    b *= y_to_gamma_minus1;

                    r *= 1000.0 / 203.0;
                    g *= 1000.0 / 203.0;
                    b *= 1000.0 / 203.0;
                }

                Op::HlginvR => r = apply_hlginv(&tf_arg(entry), r),
                Op::HlginvG => g = apply_hlginv(&tf_arg(entry), g),
                Op::HlginvB => b = apply_hlginv(&tf_arg(entry), b),
                Op::HlginvA => a = apply_hlginv(&tf_arg(entry), a),
                Op::HlginvRgb => {
                    let tf = tf_arg(entry);
                    r = apply_hlginv(&tf, r);
                    g = apply_hlginv(&tf, g);
                    b = apply_hlginv(&tf, b);
                }

                Op::HlginvOotfScale => {
                    r *= 203.0 / 1000.0;
                    g *= 203.0 / 1000.0;
                    b *= 203.0 / 1000.0;

                    let gamma_inv_minus_1 = 1.0 / 1.2 - 1.0;
                    let y = compute_y_in_xyzd50(r, g, b);
                    let (y, sign) = strip_sign(y);
                    let y_to_gamma_minus1 = apply_sign(approx_pow(y, gamma_inv_minus_1), sign);

                    r *= y_to_gamma_minus1;
                    g *= y_to_gamma_minus1;
                    b *= y_to_gamma_minus1;
                }

                Op::TableR => r = table(curve_arg(entry), r),
                Op::TableG => g = table(curve_arg(entry), g),
                Op::TableB => b = table(curve_arg(entry), b),
                Op::TableA => a = table(curve_arg(entry), a),

                Op::ClutA2B => {
                    if let OpArg::A2B(a2b) = entry.arg {
                        clut(
                            a2b.input_channels,
                            a2b.output_channels,
                            &a2b.grid_points,
                            a2b.grid.as_ref(),
                            &mut r,
                            &mut g,
                            &mut b,
                            &mut a,
                        );
                        if a2b.input_channels == 4 {
                            // CMYK is opaque.
                            a = 1.0;
                        }
                    }
                }
                Op::ClutB2A => {
                    if let OpArg::B2A(b2a) = entry.arg {
                        clut(
                            b2a.input_channels,
                            b2a.output_channels,
                            &b2a.grid_points,
                            b2a.grid.as_ref(),
                            &mut r,
                            &mut g,
                            &mut b,
                            &mut a,
                        );
                    }
                }

                Op::StoreA8 => {
                    dst[dst_at] = to_fixed(a * 255.0) as u8;
                    break;
                }
                Op::StoreG8 => {
                    // g should be holding luminance (Y) (r,g,b ~~~> X,Y,Z)
                    dst[dst_at] = to_fixed(g * 255.0) as u8;
                    break;
                }
                Op::StoreGa88 => {
                    let v = (to_fixed(g * 255.0) as u16) | ((to_fixed(a * 255.0) as u16) << 8);
                    write_u16_le(dst, dst_at, v);
                    break;
                }
                Op::Store4444 => {
                    let v = ((to_fixed(r * 15.0) as u16) << 12)
                        | ((to_fixed(g * 15.0) as u16) << 8)
                        | ((to_fixed(b * 15.0) as u16) << 4)
                        | (to_fixed(a * 15.0) as u16);
                    write_u16_le(dst, dst_at, v);
                    break;
                }
                Op::Store565 => {
                    let v = (to_fixed(r * 31.0) as u16)
                        | ((to_fixed(g * 63.0) as u16) << 5)
                        | ((to_fixed(b * 31.0) as u16) << 11);
                    write_u16_le(dst, dst_at, v);
                    break;
                }
                Op::Store888 => {
                    dst[dst_at] = to_fixed(r * 255.0) as u8;
                    dst[dst_at + 1] = to_fixed(g * 255.0) as u8;
                    dst[dst_at + 2] = to_fixed(b * 255.0) as u8;
                    break;
                }
                Op::Store8888 => {
                    let v = (to_fixed(r * 255.0) << 0)
                        | (to_fixed(g * 255.0) << 8)
                        | (to_fixed(b * 255.0) << 16)
                        | (to_fixed(a * 255.0) << 24);
                    write_u32_le(dst, dst_at, v);
                    break;
                }
                Op::Store101010xXr => {
                    let v = (to_fixed(r * 510.0 + 384.0) << 0)
                        | (to_fixed(g * 510.0 + 384.0) << 10)
                        | (to_fixed(b * 510.0 + 384.0) << 20);
                    write_u32_le(dst, dst_at, v);
                    break;
                }
                Op::Store10101010Xr => {
                    let v = ((to_fixed(r * 510.0 + 384.0) as u64) << 6)
                        | ((to_fixed(g * 510.0 + 384.0) as u64) << 22)
                        | ((to_fixed(b * 510.0 + 384.0) as u64) << 38)
                        | ((to_fixed(a * 510.0 + 384.0) as u64) << 54);
                    write_u64_le(dst, dst_at, v);
                    break;
                }
                Op::Store1010102 => {
                    let v = (to_fixed(r * 1023.0) << 0)
                        | (to_fixed(g * 1023.0) << 10)
                        | (to_fixed(b * 1023.0) << 20)
                        | (to_fixed(a * 3.0) << 30);
                    write_u32_le(dst, dst_at, v);
                    break;
                }
                Op::Store161616Le => {
                    write_u16_le(dst, dst_at, u16_from_f(r));
                    write_u16_le(dst, dst_at + 2, u16_from_f(g));
                    write_u16_le(dst, dst_at + 4, u16_from_f(b));
                    break;
                }
                Op::Store16161616Le => {
                    write_u16_le(dst, dst_at, u16_from_f(r));
                    write_u16_le(dst, dst_at + 2, u16_from_f(g));
                    write_u16_le(dst, dst_at + 4, u16_from_f(b));
                    write_u16_le(dst, dst_at + 6, u16_from_f(a));
                    break;
                }
                Op::Store161616Be => {
                    let r16 = to_fixed(r * 65535.0) as u16;
                    let g16 = to_fixed(g * 65535.0) as u16;
                    let b16 = to_fixed(b * 65535.0) as u16;
                    write_u16_be(dst, dst_at, r16);
                    write_u16_be(dst, dst_at + 2, g16);
                    write_u16_be(dst, dst_at + 4, b16);
                    break;
                }
                Op::Store16161616Be => {
                    let v = ((to_fixed(r * 65535.0) as u64) << 0)
                        | ((to_fixed(g * 65535.0) as u64) << 16)
                        | ((to_fixed(b * 65535.0) as u64) << 32)
                        | ((to_fixed(a * 65535.0) as u64) << 48);
                    // swap_endian_16x4
                    let swapped =
                        ((v & 0x00FF_00FF_00FF_00FF) << 8) | ((v & 0xFF00_FF00_FF00_FF00) >> 8);
                    write_u64_le(dst, dst_at, swapped);
                    break;
                }
                Op::StoreHhh => {
                    write_u16_le(dst, dst_at, half_from_f(r));
                    write_u16_le(dst, dst_at + 2, half_from_f(g));
                    write_u16_le(dst, dst_at + 4, half_from_f(b));
                    break;
                }
                Op::StoreHhhh => {
                    write_u16_le(dst, dst_at, half_from_f(r));
                    write_u16_le(dst, dst_at + 2, half_from_f(g));
                    write_u16_le(dst, dst_at + 4, half_from_f(b));
                    write_u16_le(dst, dst_at + 6, half_from_f(a));
                    break;
                }
                Op::StoreFff => {
                    write_f32_le(dst, dst_at, r);
                    write_f32_le(dst, dst_at + 4, g);
                    write_f32_le(dst, dst_at + 8, b);
                    break;
                }
                Op::StoreFfff => {
                    write_f32_le(dst, dst_at, r);
                    write_f32_le(dst, dst_at + 4, g);
                    write_f32_le(dst, dst_at + 8, b);
                    write_f32_le(dst, dst_at + 12, a);
                    break;
                }
            }
        }
    }
}

fn tf_arg(entry: &OpAndArg) -> SkcmsTransferFunction {
    match entry.arg {
        OpArg::Tf(tf) => tf,
        _ => unreachable!("transfer-function op without a transfer function"),
    }
}

fn curve_arg<'a>(entry: &OpAndArg<'a>) -> &'a SkcmsCurve {
    match entry.arg {
        OpArg::Curve(curve) => curve,
        _ => unreachable!("table op without a curve"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skcms::{
        SkcmsIccProfile, skcms_approximately_equal_profiles, skcms_srgb_profile,
        skcms_transfer_function_eval,
    };

    #[test]
    fn dev_transform_identity_srgb_888() {
        let src = [255u8, 0, 0, 0, 255, 0, 0, 0, 255];
        let mut dst = [0u8; 9];
        assert!(skcms_transform(
            &src,
            SkcmsPixelFormat::Rgb888,
            SkcmsAlphaFormat::Unpremul,
            Some(skcms_srgb_profile()),
            &mut dst,
            SkcmsPixelFormat::Rgb888,
            SkcmsAlphaFormat::Unpremul,
            Some(skcms_srgb_profile()),
            3,
        ));
        assert_eq!(dst, src);
    }

    #[test]
    fn dev_transform_linear_roundtrip() {
        let mut linear = SkcmsIccProfile::init();
        let identity = SkcmsTransferFunction::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        linear.set_transfer_function(&identity);
        linear.set_xyzd50(&skcms_srgb_profile().to_xyzd50);

        let src = [64u8, 128, 192];
        let mut dst = [0u8; 3];
        assert!(skcms_transform(
            &src,
            SkcmsPixelFormat::Rgb888,
            SkcmsAlphaFormat::Unpremul,
            Some(skcms_srgb_profile()),
            &mut dst,
            SkcmsPixelFormat::Rgb888,
            SkcmsAlphaFormat::Unpremul,
            Some(&linear),
            1,
        ));
        assert!(dst[0] < src[0]);
        assert!(dst[1] < src[1]);
        assert!(dst[2] < src[2]);
        assert!(skcms_approximately_equal_profiles(
            skcms_srgb_profile(),
            skcms_srgb_profile()
        ));
        assert!((skcms_transfer_function_eval(&identity, 0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn dev_transform_in_place_not_allowed_but_same_bpp_supported_by_api_shape() {
        // The safe Rust entry point takes distinct slices; this check documents that a transform
        // with identical formats preserves the pixel values.
        let src = [1u8, 2, 3, 4];
        let mut dst = [0u8; 4];
        assert!(skcms_transform(
            &src,
            SkcmsPixelFormat::Rgba8888,
            SkcmsAlphaFormat::Unpremul,
            Some(skcms_srgb_profile()),
            &mut dst,
            SkcmsPixelFormat::Rgba8888,
            SkcmsAlphaFormat::Unpremul,
            Some(skcms_srgb_profile()),
            1,
        ));
        assert_eq!(dst, src);
    }
}
