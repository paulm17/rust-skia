//! Port of the bundled first-party `modules/skcms` library:
//! `skcms.h`, `skcms_public.h`, `skcms_internals.h` and `skcms.cc`.
//!
//! `skcms_Transform` itself lives in [`crate::skcms_transform`] (the reference splits it across
//! `Transform_inl.h` and the three `skcms_Transform*.cc` translation units).
//!
//! Ownership adaptations: the reference profile retains raw pointers into the caller's ICC buffer
//! and into tag payloads. This port owns the profile buffer (`Vec<u8>`) and copies curve tables and
//! CLUT grids into owned storage. Curve 16-bit tables are stored host-endian; the reference swaps
//! the big-endian ICC bytes at evaluation time, which is observationally identical.
#![allow(clippy::excessive_precision)]

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::codec::sk_checksum_hash32;

/// A row-major 3x3 matrix (ie `vals[row][col]`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkcmsMatrix3x3 {
    pub vals: [[f32; 3]; 3],
}

impl SkcmsMatrix3x3 {
    /// Bitwise equality, matching the reference's `memcmp` of the nine floats.
    pub fn bits_eq(&self, other: &SkcmsMatrix3x3) -> bool {
        self.vals
            .iter()
            .flatten()
            .zip(other.vals.iter().flatten())
            .all(|(a, b)| a.to_bits() == b.to_bits())
    }
}

/// A row-major 3x4 matrix (ie `vals[row][col]`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkcmsMatrix3x4 {
    pub vals: [[f32; 4]; 3],
}

impl SkcmsMatrix3x4 {
    /// Bitwise equality, matching the reference's `memcmp` of the twelve floats.
    pub fn bits_eq(&self, other: &SkcmsMatrix3x4) -> bool {
        self.vals
            .iter()
            .flatten()
            .zip(other.vals.iter().flatten())
            .all(|(a, b)| a.to_bits() == b.to_bits())
    }
}

/// A transfer function mapping encoded values to linear values.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkcmsTransferFunction {
    pub g: f32,
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl SkcmsTransferFunction {
    pub const fn new(g: f32, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self {
            g,
            a,
            b,
            c,
            d,
            e,
            f,
        }
    }

    /// Bitwise equality, matching the reference's `memcmp` of the seven floats.
    pub fn bits_eq(&self, other: &SkcmsTransferFunction) -> bool {
        self.g.to_bits() == other.g.to_bits()
            && self.a.to_bits() == other.a.to_bits()
            && self.b.to_bits() == other.b.to_bits()
            && self.c.to_bits() == other.c.to_bits()
            && self.d.to_bits() == other.d.to_bits()
            && self.e.to_bits() == other.e.to_bits()
            && self.f.to_bits() == other.f.to_bits()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkcmsTfType {
    Invalid = 0,
    Srgbish = 1,
    Pqish = 2,
    Hlgish = 3,
    Hlginvish = 4,
    Pq = 5,
    Hlg = 6,
}

/// Table data for an `skcms_Curve`.
#[derive(Clone, Debug, PartialEq)]
pub enum SkcmsCurveTable {
    U8(Vec<u8>),
    /// Native-endian copies of the ICC's big-endian 16-bit entries.
    U16(Vec<u16>),
}

/// Unified representation of `curv` or `para` tag data, or a 1D table from `mft1`/`mft2`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkcmsCurve {
    /// When zero the curve is `parametric`; otherwise the curve is a table.
    pub table_entries: u32,
    pub parametric: SkcmsTransferFunction,
    pub table: Option<SkcmsCurveTable>,
}

impl SkcmsCurve {
    pub fn from_parametric(tf: SkcmsTransferFunction) -> Self {
        Self {
            table_entries: 0,
            parametric: tf,
            table: None,
        }
    }

    /// Evaluate the curve, matching `eval_curve()` in `skcms.cc`.
    pub fn eval(&self, x: f32) -> f32 {
        if self.table_entries == 0 {
            return skcms_transfer_function_eval(&self.parametric, x);
        }

        let entries = self.table_entries;
        let ix = fmaxf_(0.0, fminf_(x, 1.0)) * (entries - 1) as f32;
        let lo = ix as i32;
        let hi = minus_1_ulp(ix + 1.0) as i32;
        let t = ix - lo as f32;

        let (l, h) = match &self.table {
            Some(SkcmsCurveTable::U8(table)) => {
                let get = |i: i32| -> f32 {
                    table.get(i as usize).copied().unwrap_or(0) as f32 * (1.0 / 255.0)
                };
                (get(lo), get(hi))
            }
            Some(SkcmsCurveTable::U16(table)) => {
                let get = |i: i32| -> f32 {
                    table.get(i as usize).copied().unwrap_or(0) as f32 * (1.0 / 65535.0)
                };
                (get(lo), get(hi))
            }
            None => (0.0, 0.0),
        };
        l + (h - l) * t
    }
}

/// The payload of a CLUT grid.
#[derive(Clone, Debug, PartialEq)]
pub enum SkcmsGrid {
    U8(Vec<u8>),
    /// Native-endian copies of the ICC's big-endian 16-bit entries.
    U16(Vec<u16>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkcmsA2B {
    pub input_curves: [SkcmsCurve; 4],
    pub grid: Option<SkcmsGrid>,
    pub input_channels: u32,
    pub grid_points: [u8; 4],
    pub matrix_curves: [SkcmsCurve; 3],
    pub matrix: SkcmsMatrix3x4,
    pub matrix_channels: u32,
    pub output_channels: u32,
    pub output_curves: [SkcmsCurve; 3],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkcmsB2A {
    pub input_curves: [SkcmsCurve; 3],
    pub input_channels: u32,
    pub matrix_channels: u32,
    pub matrix_curves: [SkcmsCurve; 3],
    pub matrix: SkcmsMatrix3x4,
    pub output_curves: [SkcmsCurve; 4],
    pub grid: Option<SkcmsGrid>,
    pub grid_points: [u8; 4],
    pub output_channels: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkcmsCicp {
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub video_full_range_flag: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkcmsHagc {
    pub size: u32,
    pub buffer: Vec<u8>,
}

/// Parsed ICC profile. The owned `buffer` replaces the reference's borrowed `const uint8_t*`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkcmsIccProfile {
    pub buffer: Vec<u8>,
    pub size: u32,
    pub data_color_space: u32,
    pub pcs: u32,
    pub tag_count: u32,
    pub trc: [SkcmsCurve; 3],
    pub to_xyzd50: SkcmsMatrix3x3,
    pub a2b: SkcmsA2B,
    pub b2a: SkcmsB2A,
    pub cicp: SkcmsCicp,
    pub hagc: SkcmsHagc,
    pub has_trc: bool,
    pub has_to_xyzd50: bool,
    pub has_a2b: bool,
    pub has_b2a: bool,
    pub has_cicp: bool,
    pub has_hagc: bool,
}

impl SkcmsIccProfile {
    /// `skcms_Init`.
    pub fn init() -> Self {
        Self {
            data_color_space: SKCMS_SIGNATURE_RGB,
            pcs: SKCMS_SIGNATURE_XYZ,
            ..Default::default()
        }
    }

    /// `skcms_SetTransferFunction`.
    pub fn set_transfer_function(&mut self, tf: &SkcmsTransferFunction) {
        self.has_trc = true;
        for curve in &mut self.trc {
            curve.table_entries = 0;
            curve.parametric = *tf;
            curve.table = None;
        }
    }

    /// `skcms_SetXYZD50`.
    pub fn set_xyzd50(&mut self, m: &SkcmsMatrix3x3) {
        self.has_to_xyzd50 = true;
        self.to_xyzd50 = *m;
    }
}

// ~~~~ Common ICC signatures ~~~~

pub const SKCMS_SIGNATURE_CMYK: u32 = 0x434D_594B;
pub const SKCMS_SIGNATURE_GRAY: u32 = 0x4752_4159;
pub const SKCMS_SIGNATURE_RGB: u32 = 0x5247_4220;
pub const SKCMS_SIGNATURE_LAB: u32 = 0x4C61_6220;
pub const SKCMS_SIGNATURE_XYZ: u32 = 0x5859_5A20;
pub const SKCMS_SIGNATURE_CIELUV: u32 = 0x4C75_7620;
pub const SKCMS_SIGNATURE_YCBCR: u32 = 0x5943_6272;
pub const SKCMS_SIGNATURE_CIEXYY: u32 = 0x5978_7920;
pub const SKCMS_SIGNATURE_HSV: u32 = 0x4853_5620;
pub const SKCMS_SIGNATURE_HLS: u32 = 0x484C_5320;
pub const SKCMS_SIGNATURE_CMY: u32 = 0x434D_5920;
pub const SKCMS_SIGNATURE_2CLR: u32 = 0x3243_4C52;
pub const SKCMS_SIGNATURE_3CLR: u32 = 0x3343_4C52;
pub const SKCMS_SIGNATURE_4CLR: u32 = 0x3443_4C52;
pub const SKCMS_SIGNATURE_5CLR: u32 = 0x3543_4C52;
pub const SKCMS_SIGNATURE_6CLR: u32 = 0x3643_4C52;
pub const SKCMS_SIGNATURE_7CLR: u32 = 0x3743_4C52;
pub const SKCMS_SIGNATURE_8CLR: u32 = 0x3843_4C52;
pub const SKCMS_SIGNATURE_9CLR: u32 = 0x3943_4C52;
pub const SKCMS_SIGNATURE_10CLR: u32 = 0x4143_4C52;
pub const SKCMS_SIGNATURE_11CLR: u32 = 0x4243_4C52;
pub const SKCMS_SIGNATURE_12CLR: u32 = 0x4343_4C52;
pub const SKCMS_SIGNATURE_13CLR: u32 = 0x4443_4C52;
pub const SKCMS_SIGNATURE_14CLR: u32 = 0x4543_4C52;
pub const SKCMS_SIGNATURE_15CLR: u32 = 0x4643_4C52;

// Additional signatures used only internally by skcms.
const SKCMS_SIGNATURE_ACSP: u32 = 0x6163_7370;
const SKCMS_SIGNATURE_RTRC: u32 = 0x7254_5243;
const SKCMS_SIGNATURE_GTRC: u32 = 0x6754_5243;
const SKCMS_SIGNATURE_BTRC: u32 = 0x6254_5243;
const SKCMS_SIGNATURE_KTRC: u32 = 0x6B54_5243;
const SKCMS_SIGNATURE_RXYZ: u32 = 0x7258_595A;
const SKCMS_SIGNATURE_GXYZ: u32 = 0x6758_595A;
const SKCMS_SIGNATURE_BXYZ: u32 = 0x6258_595A;
const SKCMS_SIGNATURE_A2B0: u32 = 0x4132_4230;
const SKCMS_SIGNATURE_B2A0: u32 = 0x4232_4130;
const SKCMS_SIGNATURE_CHAD: u32 = 0x6368_6164;
const SKCMS_SIGNATURE_WTPT: u32 = 0x7774_7074;
const SKCMS_SIGNATURE_CICP: u32 = 0x6369_6370;
const SKCMS_SIGNATURE_HAGC: u32 = 0x4841_4743;
const SKCMS_SIGNATURE_CURV: u32 = 0x6375_7276;
const SKCMS_SIGNATURE_HAGC_TYPE: u32 = 0x6861_6763;
const SKCMS_SIGNATURE_MFT1: u32 = 0x6D66_7431;
const SKCMS_SIGNATURE_MFT2: u32 = 0x6D66_7432;
const SKCMS_SIGNATURE_MAB: u32 = 0x6D41_4220;
const SKCMS_SIGNATURE_MBA: u32 = 0x6D42_4120;
const SKCMS_SIGNATURE_PARA: u32 = 0x7061_7261;
const SKCMS_SIGNATURE_SF32: u32 = 0x7366_3332;

/// A potential vulnerability exists where a large CLUT can cause an integer overflow. Limit the
/// total number of grid points to a safe value.
const SKCMS_MAX_GRID_POINTS: u64 = 350_000_000;

static ALLOW_RUNTIME_CPU_DETECTION: AtomicBool = AtomicBool::new(true);

/// `skcms_DisableRuntimeCPUDetection`. On Apple Silicon the reference always selects the baseline
/// program (`cpu_type()` returns `Baseline` for non-x86-64 targets); this records the flag.
pub fn skcms_disable_runtime_cpu_detection() {
    ALLOW_RUNTIME_CPU_DETECTION.store(false, Ordering::Relaxed);
}

fn allow_runtime_cpu_detection() -> bool {
    ALLOW_RUNTIME_CPU_DETECTION.load(Ordering::Relaxed)
}

// ~~~~ Portable math ~~~~

#[inline]
pub fn floorf_(x: f32) -> f32 {
    let roundtrip = x as i32 as f32;
    if roundtrip > x {
        roundtrip - 1.0
    } else {
        roundtrip
    }
}

#[inline]
pub fn fabsf_(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[inline]
fn fmaxf_(x: f32, y: f32) -> f32 {
    if x > y { x } else { y }
}

#[inline]
fn fminf_(x: f32, y: f32) -> f32 {
    if x < y { x } else { y }
}

#[inline]
fn isfinitef_(x: f32) -> bool {
    x * 0.0 == 0.0
}

fn minus_1_ulp(x: f32) -> f32 {
    f32::from_bits(x.to_bits().wrapping_sub(1))
}

pub fn log2f_(x: f32) -> f32 {
    // The first approximation of log2(x) is its exponent 'e', minus 127.
    let bits = x.to_bits() as i32;

    let e = bits as f32 * (1.0 / (1 << 23) as f32);

    // If we use the mantissa too we can refine the error significantly.
    let m_bits = ((bits & 0x007F_FFFF) | 0x3F00_0000) as u32;
    let m = f32::from_bits(m_bits);

    e - 124.225_514_990 - 1.498_030_302 * m - 1.725_879_990 / (0.352_088_706_8 + m)
}

fn logf_(x: f32) -> f32 {
    const LN2: f32 = 0.693_147_18;
    LN2 * log2f_(x)
}

fn exp2f_(x: f32) -> f32 {
    if x > 128.0 {
        return f32::INFINITY;
    } else if x < -127.0 {
        return 0.0;
    }
    let fract = x - floorf_(x);

    let fbits = (1.0 * (1 << 23) as f32)
        * (x + 121.274_057_500 - 1.490_129_070 * fract + 27.728_023_300 / (4.842_525_68 - fract));

    // Before casting to int32_t, check for out of range values.
    if fbits >= i32::MAX as f32 {
        return f32::INFINITY;
    } else if fbits < 0.0 {
        return 0.0;
    }

    let bits = fbits as i32;
    f32::from_bits(bits as u32)
}

/// `powf_`, used by `skcms_TransferFunction_eval`. Not static in the reference because test tools
/// use it.
pub fn powf_(x: f32, y: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    exp2f_(log2f_(x) * y)
}

fn expf_(x: f32) -> f32 {
    const LOG2_E: f32 = 1.442_695_040_888_963_4;
    exp2f_(LOG2_E * x)
}

// ~~~~ Transfer functions ~~~~

#[derive(Clone, Copy)]
struct TfPqish {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

#[derive(Clone, Copy)]
struct TfHlgish {
    r: f32,
    g: f32,
    a: f32,
    b: f32,
    c: f32,
    k_minus_1: f32,
}

fn tf_kind_marker(kind: SkcmsTfType) -> f32 {
    -(kind as i32) as f32
}

fn classify(tf: &SkcmsTransferFunction) -> SkcmsTfType {
    let mut pq = None;
    let mut hlg = None;
    classify_into(tf, &mut pq, &mut hlg)
}

fn classify_into(
    tf: &SkcmsTransferFunction,
    pq: &mut Option<TfPqish>,
    hlg: &mut Option<TfHlgish>,
) -> SkcmsTfType {
    if tf.g < 0.0 {
        // Negative "g" is mapped to enum values; large negative are for sure invalid.
        if tf.g < -128.0 {
            return SkcmsTfType::Invalid;
        }
        let enum_g = -(tf.g as i32);
        // Non-whole "g" values are invalid as well.
        if -enum_g as f32 != tf.g {
            return SkcmsTfType::Invalid;
        }
        match enum_g {
            2 => {
                *pq = Some(TfPqish {
                    a: tf.a,
                    b: tf.b,
                    c: tf.c,
                    d: tf.d,
                    e: tf.e,
                    f: tf.f,
                });
                SkcmsTfType::Pqish
            }
            3 => {
                *hlg = Some(TfHlgish {
                    r: tf.a,
                    g: tf.b,
                    a: tf.c,
                    b: tf.d,
                    c: tf.e,
                    k_minus_1: tf.f,
                });
                SkcmsTfType::Hlgish
            }
            4 => {
                *hlg = Some(TfHlgish {
                    r: tf.a,
                    g: tf.b,
                    a: tf.c,
                    b: tf.d,
                    c: tf.e,
                    k_minus_1: tf.f,
                });
                SkcmsTfType::Hlginvish
            }
            5 => {
                if tf.b != 0.0 || tf.c != 0.0 || tf.d != 0.0 || tf.e != 0.0 || tf.f != 0.0 {
                    return SkcmsTfType::Invalid;
                }
                SkcmsTfType::Pq
            }
            6 => {
                if tf.d != 0.0 || tf.e != 0.0 || tf.f != 0.0 {
                    return SkcmsTfType::Invalid;
                }
                SkcmsTfType::Hlg
            }
            _ => SkcmsTfType::Invalid,
        }
    } else {
        // Basic soundness checks for sRGBish transfer functions.
        if isfinitef_(tf.a + tf.b + tf.c + tf.d + tf.e + tf.f + tf.g)
            && tf.a >= 0.0
            && tf.c >= 0.0
            && tf.d >= 0.0
            && tf.g >= 0.0
            && tf.a * tf.d + tf.b >= 0.0
        {
            SkcmsTfType::Srgbish
        } else {
            SkcmsTfType::Invalid
        }
    }
}

pub fn skcms_transfer_function_get_type(tf: &SkcmsTransferFunction) -> SkcmsTfType {
    classify(tf)
}

pub fn skcms_transfer_function_is_srgbish(tf: &SkcmsTransferFunction) -> bool {
    classify(tf) == SkcmsTfType::Srgbish
}
pub fn skcms_transfer_function_is_pqish(tf: &SkcmsTransferFunction) -> bool {
    classify(tf) == SkcmsTfType::Pqish
}
pub fn skcms_transfer_function_is_hlgish(tf: &SkcmsTransferFunction) -> bool {
    classify(tf) == SkcmsTfType::Hlgish
}
pub fn skcms_transfer_function_is_pq(tf: &SkcmsTransferFunction) -> bool {
    classify(tf) == SkcmsTfType::Pq
}
pub fn skcms_transfer_function_is_hlg(tf: &SkcmsTransferFunction) -> bool {
    classify(tf) == SkcmsTfType::Hlg
}

pub fn skcms_transfer_function_make_pqish(
    tf: &mut SkcmsTransferFunction,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
) -> bool {
    *tf = SkcmsTransferFunction::new(tf_kind_marker(SkcmsTfType::Pqish), a, b, c, d, e, f);
    debug_assert!(skcms_transfer_function_is_pqish(tf));
    true
}

pub fn skcms_transfer_function_make_scaled_hlgish(
    tf: &mut SkcmsTransferFunction,
    k: f32,
    r: f32,
    g: f32,
    a: f32,
    b: f32,
    c: f32,
) -> bool {
    *tf = SkcmsTransferFunction::new(tf_kind_marker(SkcmsTfType::Hlgish), r, g, a, b, c, k - 1.0);
    debug_assert!(skcms_transfer_function_is_hlgish(tf));
    true
}

pub fn skcms_transfer_function_make_hlgish(
    tf: &mut SkcmsTransferFunction,
    r: f32,
    g: f32,
    a: f32,
    b: f32,
    c: f32,
) -> bool {
    skcms_transfer_function_make_scaled_hlgish(tf, 1.0, r, g, a, b, c)
}

pub fn skcms_transfer_function_make_pq(
    tf: &mut SkcmsTransferFunction,
    hdr_reference_white_luminance: f32,
) {
    *tf = SkcmsTransferFunction::new(
        tf_kind_marker(SkcmsTfType::Pq),
        hdr_reference_white_luminance,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    debug_assert!(skcms_transfer_function_is_pq(tf));
}

pub fn skcms_transfer_function_make_hlg(
    tf: &mut SkcmsTransferFunction,
    hdr_reference_white_luminance: f32,
    peak_luminance: f32,
    system_gamma: f32,
) {
    *tf = SkcmsTransferFunction::new(
        tf_kind_marker(SkcmsTfType::Hlg),
        hdr_reference_white_luminance,
        peak_luminance,
        system_gamma,
        0.0,
        0.0,
        0.0,
    );
    debug_assert!(skcms_transfer_function_is_hlg(tf));
}

/// `skcms_TransferFunction_eval`.
pub fn skcms_transfer_function_eval(tf: &SkcmsTransferFunction, x: f32) -> f32 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let mut x = x * sign;

    let mut pq = None;
    let mut hlg = None;
    match classify_into(tf, &mut pq, &mut hlg) {
        SkcmsTfType::Invalid => 0.0,
        SkcmsTfType::Hlg => {
            const A: f32 = 0.178_832_77;
            const B: f32 = 0.284_668_92;
            const C: f32 = 0.559_910_73;
            sign * (if x <= 0.5 {
                x * x / 3.0
            } else {
                (expf_((x - C) / A) + B) / 12.0
            })
        }
        SkcmsTfType::Hlgish => {
            let hlg = hlg.unwrap();
            let k = hlg.k_minus_1 + 1.0;
            k * sign
                * (if x * hlg.r <= 1.0 {
                    powf_(x * hlg.r, hlg.g)
                } else {
                    expf_((x - hlg.c) * hlg.a) + hlg.b
                })
        }
        SkcmsTfType::Hlginvish => {
            let hlg = hlg.unwrap();
            let k = hlg.k_minus_1 + 1.0;
            x /= k;
            sign * (if x <= 1.0 {
                hlg.r * powf_(x, hlg.g)
            } else {
                hlg.a * logf_(x - hlg.b) + hlg.c
            })
        }
        SkcmsTfType::Srgbish => {
            sign * (if x < tf.d {
                tf.c * x + tf.f
            } else {
                powf_(tf.a * x + tf.b, tf.g) + tf.e
            })
        }
        SkcmsTfType::Pq => {
            const C1: f32 = 107.0 / 128.0;
            const C2: f32 = 2413.0 / 128.0;
            const C3: f32 = 2392.0 / 128.0;
            const M1: f32 = 1305.0 / 8192.0;
            const M2: f32 = 2523.0 / 32.0;
            let p = powf_(x, 1.0 / M2);
            powf_((p - C1) / (C2 - C3 * p), 1.0 / M1)
        }
        SkcmsTfType::Pqish => {
            let pq = pq.unwrap();
            sign * powf_(
                (pq.a + pq.b * powf_(x, pq.c)) / (pq.d + pq.e * powf_(x, pq.c)),
                pq.f,
            )
        }
    }
}

/// `eval_curve` (`skcms.cc`), used by the curve-fitting helpers.
pub fn skcms_eval_curve(curve: &SkcmsCurve, x: f32) -> f32 {
    curve.eval(x)
}

/// `skcms_MaxRoundtripError`.
pub fn skcms_max_roundtrip_error(curve: &SkcmsCurve, inv_tf: &SkcmsTransferFunction) -> f32 {
    let n = if curve.table_entries > 256 {
        curve.table_entries
    } else {
        256
    };
    let dx = 1.0 / (n - 1) as f32;
    let mut err = 0.0f32;
    for i in 0..n {
        let x = i as f32 * dx;
        let y = skcms_eval_curve(curve, x);
        err = fmaxf_(err, fabsf_(x - skcms_transfer_function_eval(inv_tf, y)));
    }
    err
}

/// `skcms_AreApproximateInverses`.
pub fn skcms_are_approximate_inverses(curve: &SkcmsCurve, inv_tf: &SkcmsTransferFunction) -> bool {
    skcms_max_roundtrip_error(curve, inv_tf) < (1.0 / 512.0)
}

// ~~~~ Matrix helpers ~~~~

#[derive(Clone, Copy)]
struct SkcmsVector3 {
    vals: [f32; 3],
}

fn mv_mul(m: &SkcmsMatrix3x3, v: &SkcmsVector3) -> SkcmsVector3 {
    let mut dst = SkcmsVector3 { vals: [0.0; 3] };
    for row in 0..3 {
        dst.vals[row] =
            m.vals[row][0] * v.vals[0] + m.vals[row][1] * v.vals[1] + m.vals[row][2] * v.vals[2];
    }
    dst
}

/// `skcms_Matrix3x3_isfinite`.
pub fn skcms_matrix3x3_isfinite(m: &SkcmsMatrix3x3) -> bool {
    for row in &m.vals {
        for &value in row {
            if !isfinitef_(value) {
                return false;
            }
        }
    }
    true
}

/// `skcms_Matrix3x3_invert`.
pub fn skcms_matrix3x3_invert(src: &SkcmsMatrix3x3, dst: &mut SkcmsMatrix3x3) -> bool {
    let a00 = src.vals[0][0] as f64;
    let a01 = src.vals[1][0] as f64;
    let a02 = src.vals[2][0] as f64;
    let a10 = src.vals[0][1] as f64;
    let a11 = src.vals[1][1] as f64;
    let a12 = src.vals[2][1] as f64;
    let a20 = src.vals[0][2] as f64;
    let a21 = src.vals[1][2] as f64;
    let a22 = src.vals[2][2] as f64;

    let mut b0 = a00 * a11 - a01 * a10;
    let mut b1 = a00 * a12 - a02 * a10;
    let mut b2 = a01 * a12 - a02 * a11;
    let mut b3 = a20;
    let mut b4 = a21;
    let mut b5 = a22;

    let determinant = b0 * b5 - b1 * b4 + b2 * b3;

    if determinant == 0.0 {
        return false;
    }

    let invdet = 1.0 / determinant;
    if invdet > f32::MAX as f64 || invdet < -(f32::MAX as f64) || !isfinitef_(invdet as f32) {
        return false;
    }

    b0 *= invdet;
    b1 *= invdet;
    b2 *= invdet;
    b3 *= invdet;
    b4 *= invdet;
    b5 *= invdet;

    dst.vals[0][0] = (a11 * b5 - a12 * b4) as f32;
    dst.vals[1][0] = (a02 * b4 - a01 * b5) as f32;
    dst.vals[2][0] = (b2) as f32;
    dst.vals[0][1] = (a12 * b3 - a10 * b5) as f32;
    dst.vals[1][1] = (a00 * b5 - a02 * b3) as f32;
    dst.vals[2][1] = (-b1) as f32;
    dst.vals[0][2] = (a10 * b4 - a11 * b3) as f32;
    dst.vals[1][2] = (a01 * b3 - a00 * b4) as f32;
    dst.vals[2][2] = (b0) as f32;

    skcms_matrix3x3_isfinite(dst)
}

/// `skcms_Matrix3x3_concat`.
pub fn skcms_matrix3x3_concat(a: &SkcmsMatrix3x3, b: &SkcmsMatrix3x3) -> SkcmsMatrix3x3 {
    let mut m = SkcmsMatrix3x3::default();
    for r in 0..3 {
        for c in 0..3 {
            m.vals[r][c] = a.vals[r][0] * b.vals[0][c]
                + a.vals[r][1] * b.vals[1][c]
                + a.vals[r][2] * b.vals[2][c];
        }
    }
    m
}

/// `skcms_TransferFunction_invert`.
pub fn skcms_transfer_function_invert(
    src: &SkcmsTransferFunction,
    dst: &mut SkcmsTransferFunction,
) -> bool {
    let mut pq = None;
    let mut hlg = None;
    match classify_into(src, &mut pq, &mut hlg) {
        SkcmsTfType::Invalid | SkcmsTfType::Pq | SkcmsTfType::Hlg => return false,
        SkcmsTfType::Srgbish => {}
        SkcmsTfType::Pqish => {
            let pq = pq.unwrap();
            *dst = SkcmsTransferFunction::new(
                tf_kind_marker(SkcmsTfType::Pqish),
                -pq.a,
                pq.d,
                1.0 / pq.f,
                pq.b,
                -pq.e,
                1.0 / pq.c,
            );
            return true;
        }
        SkcmsTfType::Hlgish => {
            let hlg = hlg.unwrap();
            *dst = SkcmsTransferFunction::new(
                tf_kind_marker(SkcmsTfType::Hlginvish),
                1.0 / hlg.r,
                1.0 / hlg.g,
                1.0 / hlg.a,
                hlg.b,
                hlg.c,
                hlg.k_minus_1,
            );
            return true;
        }
        SkcmsTfType::Hlginvish => {
            let hlg = hlg.unwrap();
            *dst = SkcmsTransferFunction::new(
                tf_kind_marker(SkcmsTfType::Hlgish),
                1.0 / hlg.r,
                1.0 / hlg.g,
                1.0 / hlg.a,
                hlg.b,
                hlg.c,
                hlg.k_minus_1,
            );
            return true;
        }
    }

    debug_assert!(classify(src) == SkcmsTfType::Srgbish);

    let mut inv = SkcmsTransferFunction::default();

    let d_l = src.c * src.d + src.f;
    let d_r = powf_(src.a * src.d + src.b, src.g) + src.e;
    if fabsf_(d_l - d_r) > 1.0 / 512.0 {
        return false;
    }
    inv.d = d_l;

    if inv.d > 0.0 {
        inv.c = 1.0 / src.c;
        inv.f = -src.f / src.c;
    }

    let k = powf_(src.a, -src.g);
    inv.g = 1.0 / src.g;
    inv.a = k;
    inv.b = -k * src.e;
    inv.e = -src.b / src.a;

    if inv.a < 0.0 {
        return false;
    }
    if inv.a * inv.d + inv.b < 0.0 {
        inv.b = -inv.a * inv.d;
    }

    if classify(&inv) != SkcmsTfType::Srgbish {
        return false;
    }

    let s = skcms_transfer_function_eval(src, 1.0);
    if !isfinitef_(s) {
        return false;
    }

    let sign = if s < 0.0 { -1.0 } else { 1.0 };
    let s = s * sign;
    if s < inv.d {
        inv.f = 1.0 - sign * inv.c * s;
    } else {
        inv.e = 1.0 - sign * powf_(inv.a * s + inv.b, inv.g);
    }

    *dst = inv;
    classify(dst) == SkcmsTfType::Srgbish
}

// ~~~~ Fitting curves ~~~~

const K_MAX_TABLE_ENTRIES: u32 = 1 << 24; // 16,777,216

// `fit_linear` mirrors the iterative slope-interval fit in `skcms.cc`.
fn fit_linear(curve: &SkcmsCurve, n: i32, tol: f32, c: &mut f32, d: &mut f32) -> i32 {
    debug_assert!(n > 1);
    let dx = 1.0 / (n - 1) as f32;

    let mut lin_points = 1;

    let f_zero = 0.0f32;
    let f = &f_zero;

    let mut slope_min = f32::NEG_INFINITY;
    let mut slope_max = f32::INFINITY;
    for i in 1..n {
        let x = i as f32 * dx;
        let y = skcms_eval_curve(curve, x);

        let slope_max_i = (y + tol - *f) / x;
        let slope_min_i = (y - tol - *f) / x;
        if slope_max_i < slope_min || slope_max < slope_min_i {
            break;
        }
        slope_max = fminf_(slope_max, slope_max_i);
        slope_min = fmaxf_(slope_min, slope_min_i);

        let cur_slope = (y - *f) / x;
        if slope_min <= cur_slope && cur_slope <= slope_max {
            lin_points = i + 1;
            *c = cur_slope;
        }
    }

    *d = (lin_points - 1) as f32 * dx;
    lin_points
}

fn rg_nonlinear(
    x: f32,
    curve: &SkcmsCurve,
    tf: &SkcmsTransferFunction,
    dfd_p: &mut [f32; 3],
) -> f32 {
    let y = skcms_eval_curve(curve, x);

    let g = tf.g;
    let a = tf.a;
    let b = tf.b;
    let c = tf.c;
    let d = tf.d;
    let f = tf.f;

    let big_y = fmaxf_(a * y + b, 0.0);
    let big_d = a * d + b;
    debug_assert!(big_d >= 0.0);

    dfd_p[0] = logf_(big_y) * powf_(big_y, g) - logf_(big_d) * powf_(big_d, g);
    dfd_p[1] = y * g * powf_(big_y, g - 1.0) - d * g * powf_(big_d, g - 1.0);
    dfd_p[2] = g * powf_(big_y, g - 1.0) - g * powf_(big_d, g - 1.0);

    let f_inv = powf_(big_y, g) - powf_(big_d, g) + c * d + f;
    x - f_inv
}

fn gauss_newton_step(
    curve: &SkcmsCurve,
    tf: &mut SkcmsTransferFunction,
    x0: f32,
    dx: f32,
    n: i32,
) -> bool {
    let mut lhs = SkcmsMatrix3x3::default();
    let mut rhs = [0.0f32; 3];

    for i in 0..n {
        let x = x0 + i as f32 * dx;

        let mut dfd_p = [0.0f32; 3];
        let resid = rg_nonlinear(x, curve, tf, &mut dfd_p);

        for r in 0..3 {
            for col in 0..3 {
                lhs.vals[r][col] += dfd_p[r] * dfd_p[col];
            }
            rhs[r] += dfd_p[r] * resid;
        }
    }

    // If any of the 3 P parameters are unused, this matrix will be singular.
    for k in 0..3 {
        if lhs.vals[0][k] == 0.0
            && lhs.vals[1][k] == 0.0
            && lhs.vals[2][k] == 0.0
            && lhs.vals[k][0] == 0.0
            && lhs.vals[k][1] == 0.0
            && lhs.vals[k][2] == 0.0
        {
            lhs.vals[k][k] = 1.0;
        }
    }

    let mut lhs_inv = SkcmsMatrix3x3::default();
    if !skcms_matrix3x3_invert(&lhs, &mut lhs_inv) {
        return false;
    }

    let d_p = mv_mul(&lhs_inv, &SkcmsVector3 { vals: rhs });
    tf.g += d_p.vals[0];
    tf.a += d_p.vals[1];
    tf.b += d_p.vals[2];
    isfinitef_(tf.g) && isfinitef_(tf.a) && isfinitef_(tf.b)
}

fn max_roundtrip_error_checked(curve: &SkcmsCurve, tf_inv: &SkcmsTransferFunction) -> f32 {
    let mut tf = SkcmsTransferFunction::default();
    if !skcms_transfer_function_invert(tf_inv, &mut tf) || classify(&tf) != SkcmsTfType::Srgbish {
        return f32::INFINITY;
    }

    let mut tf_inv_again = SkcmsTransferFunction::default();
    if !skcms_transfer_function_invert(&tf, &mut tf_inv_again) {
        return f32::INFINITY;
    }

    skcms_max_roundtrip_error(curve, &tf_inv_again)
}

fn fit_nonlinear(curve: &SkcmsCurve, l: i32, n: i32, tf: &mut SkcmsTransferFunction) -> bool {
    let fixup_tf = |tf: &mut SkcmsTransferFunction| -> bool {
        if tf.a < 0.0 {
            return false;
        }
        if tf.a * tf.d + tf.b < 0.0 {
            tf.b = -tf.a * tf.d;
        }

        tf.e = tf.c * tf.d + tf.f - powf_(tf.a * tf.d + tf.b, tf.g);
        isfinitef_(tf.e)
    };

    if !fixup_tf(tf) {
        return false;
    }

    let dx = 1.0 / (n - 1) as f32;

    let mut best_tf = *tf;
    let mut best_max_error = f32::INFINITY;

    let init_error = max_roundtrip_error_checked(curve, tf);
    if init_error < best_max_error {
        best_max_error = init_error;
        best_tf = *tf;
    }

    for _ in 0..8 {
        if !gauss_newton_step(curve, tf, l as f32 * dx, dx, n - l) || !fixup_tf(tf) {
            *tf = best_tf;
            return isfinitef_(best_max_error);
        }

        let max_error = max_roundtrip_error_checked(curve, tf);
        if max_error < best_max_error {
            best_max_error = max_error;
            best_tf = *tf;
        }
    }

    *tf = best_tf;
    isfinitef_(best_max_error)
}

/// `skcms_ApproximateCurve`.
pub fn skcms_approximate_curve(
    curve: &SkcmsCurve,
    approx: &mut SkcmsTransferFunction,
    max_error: &mut f32,
) -> bool {
    if curve.table_entries == 0 {
        return false;
    }
    if curve.table_entries == 1 || curve.table_entries > K_MAX_TABLE_ENTRIES {
        return false;
    }

    let n = curve.table_entries as i32;
    let dx = 1.0 / (n - 1) as f32;

    *max_error = f32::INFINITY;
    let tolerances = [1.5 / 65535.0, 1.0 / 512.0];
    for &tol in &tolerances {
        let mut tf = SkcmsTransferFunction::default();
        let mut tf_inv = SkcmsTransferFunction::default();

        // It's problematic to fit curves with non-zero f, so always force it to zero explicitly.
        tf.f = 0.0;
        let mut c = 0.0;
        let mut d = 0.0;
        let l = fit_linear(curve, n, tol, &mut c, &mut d);
        tf.c = c;
        tf.d = d;

        if l == n {
            // If the entire data set was linear, move the coefficients to the nonlinear portion
            // with G == 1. This lets us use a canonical representation with d == 0.
            tf.g = 1.0;
            tf.a = tf.c;
            tf.b = tf.f;
            tf.c = 0.0;
            tf.d = 0.0;
            tf.e = 0.0;
            tf.f = 0.0;
        } else if l == n - 1 {
            // Degenerate case with only two points in the nonlinear segment.
            tf.g = 1.0;
            tf.a = (skcms_eval_curve(curve, (n - 1) as f32 * dx)
                - skcms_eval_curve(curve, (n - 2) as f32 * dx))
                / dx;
            tf.b = skcms_eval_curve(curve, (n - 2) as f32 * dx) - tf.a * (n - 2) as f32 * dx;
            tf.e = 0.0;
        } else {
            let mid = (l + n) / 2;
            let mid_x = mid as f32 / (n - 1) as f32;
            let mid_y = skcms_eval_curve(curve, mid_x);
            tf.g = log2f_(mid_y) / log2f_(mid_x);
            tf.a = 1.0;
            tf.b = 0.0;
            tf.e = tf.c * tf.d + tf.f - powf_(tf.a * tf.d + tf.b, tf.g);

            if !skcms_transfer_function_invert(&tf, &mut tf_inv)
                || !fit_nonlinear(curve, l, n, &mut tf_inv)
            {
                continue;
            }

            if !skcms_transfer_function_invert(&tf_inv, &mut tf) {
                debug_assert!(false);
                continue;
            }
        }

        if classify(&tf) != SkcmsTfType::Srgbish {
            continue;
        }

        if !skcms_transfer_function_invert(&tf, &mut tf_inv) {
            continue;
        }

        let err = skcms_max_roundtrip_error(curve, &tf_inv);
        if *max_error > err {
            *max_error = err;
            *approx = tf;
        }
    }
    isfinitef_(*max_error)
}

// ~~~~ ICC parsing ~~~~

const HEADER_LAYOUT_SIZE: usize = 132;
const TAG_LAYOUT_SIZE: usize = 12;

fn read_big_u16(ptr: &[u8]) -> u16 {
    u16::from_be_bytes([ptr[0], ptr[1]])
}

fn read_big_u32(ptr: &[u8]) -> u32 {
    u32::from_be_bytes([ptr[0], ptr[1], ptr[2], ptr[3]])
}

fn read_big_i32(ptr: &[u8]) -> i32 {
    read_big_u32(ptr) as i32
}

fn read_big_fixed(ptr: &[u8]) -> f32 {
    read_big_i32(ptr) as f32 * (1.0 / 65536.0)
}

/// `skcms_ICCTag`, with an offset instead of a pointer into the profile buffer.
#[derive(Clone, Copy, Debug)]
pub struct SkcmsIcctag {
    pub signature: u32,
    pub type_: u32,
    pub size: u32,
    pub offset: u32,
}

fn tag_at(profile: &SkcmsIccProfile, index: u32) -> Option<SkcmsIcctag> {
    if index >= profile.tag_count {
        return None;
    }
    let table = profile.buffer.get(HEADER_LAYOUT_SIZE..)?;
    let entry = table.get(index as usize * TAG_LAYOUT_SIZE..)?;
    let signature = read_big_u32(entry);
    let offset = read_big_u32(&entry[4..]);
    let size = read_big_u32(&entry[8..]);
    let type_ = profile
        .buffer
        .get(offset as usize..)?
        .get(..4)
        .map(read_big_u32)
        .unwrap_or(0);
    Some(SkcmsIcctag {
        signature,
        type_,
        size,
        offset,
    })
}

/// `skcms_GetTagByIndex`.
pub fn skcms_get_tag_by_index(profile: &SkcmsIccProfile, index: u32) -> Option<SkcmsIcctag> {
    tag_at(profile, index)
}

/// `skcms_GetTagBySignature`.
pub fn skcms_get_tag_by_signature(profile: &SkcmsIccProfile, sig: u32) -> Option<SkcmsIcctag> {
    for i in 0..profile.tag_count {
        if let Some(tag) = tag_at(profile, i) {
            if tag.signature == sig {
                return Some(tag);
            }
        }
    }
    None
}

impl SkcmsIccProfile {
    fn tag_size(&self, tag: &SkcmsIcctag) -> &[u8] {
        let start = tag.offset as usize;
        let end = start.saturating_add(tag.size as usize);
        self.buffer.get(start..end).unwrap_or(&[])
    }
}

/// `skcms_GetCHAD`.
pub fn skcms_get_chad(profile: &SkcmsIccProfile, m: &mut SkcmsMatrix3x3) -> bool {
    let Some(tag) = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_CHAD) else {
        return false;
    };
    let sf32_layout_size = 44usize; // type, reserved, nine s15Fixed16 values
    if tag.type_ != SKCMS_SIGNATURE_SF32 || tag.size < sf32_layout_size as u32 {
        return false;
    }
    let buf = profile.tag_size(&tag);
    if buf.len() < sf32_layout_size {
        return false;
    }
    let mut values = 8;
    for r in 0..3 {
        for c in 0..3 {
            m.vals[r][c] = read_big_fixed(&buf[values..]);
            values += 4;
        }
    }
    true
}

fn read_tag_xyz(tag: &SkcmsIcctag, buf: &[u8]) -> Option<(f32, f32, f32)> {
    let xyz_layout_size = 20usize; // type, reserved, X, Y, Z
    if tag.type_ != SKCMS_SIGNATURE_XYZ
        || tag.size < xyz_layout_size as u32
        || buf.len() < xyz_layout_size
    {
        return None;
    }
    Some((
        read_big_fixed(&buf[8..]),
        read_big_fixed(&buf[12..]),
        read_big_fixed(&buf[16..]),
    ))
}

/// `skcms_GetWTPT`.
pub fn skcms_get_wtpt(profile: &SkcmsIccProfile, xyz: &mut [f32; 3]) -> bool {
    let Some(tag) = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_WTPT) else {
        return false;
    };
    let Some((x, y, z)) = read_tag_xyz(&tag, profile.tag_size(&tag)) else {
        return false;
    };
    xyz[0] = x;
    xyz[1] = y;
    xyz[2] = z;
    true
}

fn data_color_space_channel_count(data_color_space: u32) -> i32 {
    match data_color_space {
        SKCMS_SIGNATURE_CMYK => 4,
        SKCMS_SIGNATURE_GRAY => 1,
        SKCMS_SIGNATURE_RGB
        | SKCMS_SIGNATURE_LAB
        | SKCMS_SIGNATURE_XYZ
        | SKCMS_SIGNATURE_CIELUV
        | SKCMS_SIGNATURE_YCBCR
        | SKCMS_SIGNATURE_CIEXYY
        | SKCMS_SIGNATURE_HSV
        | SKCMS_SIGNATURE_HLS
        | SKCMS_SIGNATURE_CMY => 3,
        SKCMS_SIGNATURE_2CLR => 2,
        SKCMS_SIGNATURE_3CLR => 3,
        SKCMS_SIGNATURE_4CLR => 4,
        SKCMS_SIGNATURE_5CLR => 5,
        SKCMS_SIGNATURE_6CLR => 6,
        SKCMS_SIGNATURE_7CLR => 7,
        SKCMS_SIGNATURE_8CLR => 8,
        SKCMS_SIGNATURE_9CLR => 9,
        SKCMS_SIGNATURE_10CLR => 10,
        SKCMS_SIGNATURE_11CLR => 11,
        SKCMS_SIGNATURE_12CLR => 12,
        SKCMS_SIGNATURE_13CLR => 13,
        SKCMS_SIGNATURE_14CLR => 14,
        SKCMS_SIGNATURE_15CLR => 15,
        _ => -1,
    }
}

/// `skcms_GetInputChannelCount`.
pub fn skcms_get_input_channel_count(profile: &SkcmsIccProfile) -> i32 {
    let a2b_count = if profile.has_a2b {
        if profile.a2b.input_channels != 0 {
            profile.a2b.input_channels as i32
        } else {
            3
        }
    } else {
        0
    };

    let trc_count = if skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_KTRC).is_some() {
        1
    } else if profile.has_trc {
        3
    } else {
        0
    };

    let dcs_count = data_color_space_channel_count(profile.data_color_space);

    if dcs_count < 0 {
        return -1;
    }
    if a2b_count > 0 && a2b_count != dcs_count {
        return -1;
    }
    if trc_count > 0 && trc_count != dcs_count {
        return -1;
    }
    dcs_count
}

fn tag_bounds(tag: &SkcmsIcctag) -> (usize, usize) {
    let start = tag.offset as usize;
    (start, start + tag.size as usize)
}

fn read_to_xyzd50(
    profile: &SkcmsIccProfile,
    rxyz: &SkcmsIcctag,
    gxyz: &SkcmsIcctag,
    bxyz: &SkcmsIcctag,
    to_xyz: &mut SkcmsMatrix3x3,
) -> bool {
    let (start, end) = tag_bounds(rxyz);
    let Some((x, y, z)) = read_tag_xyz(rxyz, &profile.buffer[start..end]) else {
        return false;
    };
    to_xyz.vals[0][0] = x;
    to_xyz.vals[1][0] = y;
    to_xyz.vals[2][0] = z;
    let (start, end) = tag_bounds(gxyz);
    let Some((x, y, z)) = read_tag_xyz(gxyz, &profile.buffer[start..end]) else {
        return false;
    };
    to_xyz.vals[0][1] = x;
    to_xyz.vals[1][1] = y;
    to_xyz.vals[2][1] = z;
    let (start, end) = tag_bounds(bxyz);
    let Some((x, y, z)) = read_tag_xyz(bxyz, &profile.buffer[start..end]) else {
        return false;
    };
    to_xyz.vals[0][2] = x;
    to_xyz.vals[1][2] = y;
    to_xyz.vals[2][2] = z;
    true
}

const PARA_LAYOUT_FIXED: usize = 12;

fn read_curve_para(buf: &[u8], size: usize) -> Option<(SkcmsCurve, u32)> {
    if size < PARA_LAYOUT_FIXED || buf.len() < size {
        return None;
    }
    let buf = &buf[..size];

    let function_type = read_big_u16(&buf[8..]);
    if function_type > 4 {
        return None;
    }

    const CURVE_BYTES: [usize; 5] = [4, 12, 16, 20, 28];
    let used = PARA_LAYOUT_FIXED + CURVE_BYTES[function_type as usize];
    if size < used {
        return None;
    }

    let mut curve = SkcmsCurve {
        table_entries: 0,
        parametric: SkcmsTransferFunction::default(),
        table: None,
    };
    let p = &mut curve.parametric;
    p.a = 1.0;
    p.b = 0.0;
    p.c = 0.0;
    p.d = 0.0;
    p.e = 0.0;
    p.f = 0.0;
    p.g = read_big_fixed(&buf[12..]);

    let v = &buf[12..];
    match function_type {
        1 => {
            p.a = read_big_fixed(&v[4..]);
            p.b = read_big_fixed(&v[8..]);
            if p.a == 0.0 {
                return None;
            }
            p.d = -p.b / p.a;
        }
        2 => {
            p.a = read_big_fixed(&v[4..]);
            p.b = read_big_fixed(&v[8..]);
            p.e = read_big_fixed(&v[12..]);
            if p.a == 0.0 {
                return None;
            }
            p.d = -p.b / p.a;
            p.f = p.e;
        }
        3 => {
            p.a = read_big_fixed(&v[4..]);
            p.b = read_big_fixed(&v[8..]);
            p.c = read_big_fixed(&v[12..]);
            p.d = read_big_fixed(&v[16..]);
        }
        4 => {
            p.a = read_big_fixed(&v[4..]);
            p.b = read_big_fixed(&v[8..]);
            p.c = read_big_fixed(&v[12..]);
            p.d = read_big_fixed(&v[16..]);
            p.e = read_big_fixed(&v[20..]);
            p.f = read_big_fixed(&v[24..]);
        }
        _ => {}
    }
    if !skcms_transfer_function_is_srgbish(&curve.parametric) {
        return None;
    }
    Some((curve, used as u32))
}

const CURV_LAYOUT_FIXED: usize = 12;

fn read_curve_curv(buf: &[u8], size: usize) -> Option<(SkcmsCurve, u32)> {
    if size < CURV_LAYOUT_FIXED || buf.len() < size {
        return None;
    }
    let buf = &buf[..size];

    let value_count = read_big_u32(&buf[8..]);
    let used = CURV_LAYOUT_FIXED + value_count as usize * 2;
    if size < used {
        return None;
    }

    let mut curve = SkcmsCurve::default();
    if value_count < 2 {
        curve.table_entries = 0;
        let p = &mut curve.parametric;
        p.a = 1.0;
        p.b = 0.0;
        p.c = 0.0;
        p.d = 0.0;
        p.e = 0.0;
        p.f = 0.0;
        if value_count == 0 {
            // Empty tables are a shorthand for an identity curve.
            p.g = 1.0;
        } else {
            // Single entry tables are a shorthand for simple gamma.
            p.g = read_big_u16(&buf[CURV_LAYOUT_FIXED..]) as f32 * (1.0 / 256.0);
        }
        return Some((curve, used as u32));
    }
    if value_count > K_MAX_TABLE_ENTRIES {
        return None;
    }

    let mut table = Vec::with_capacity(value_count as usize);
    for i in 0..value_count as usize {
        let offset = CURV_LAYOUT_FIXED + 2 * i;
        table.push(read_big_u16(&buf[offset..]));
    }
    curve.table_entries = value_count;
    curve.table = Some(SkcmsCurveTable::U16(table));
    Some((curve, used as u32))
}

/// Parses both `curveType` and `parametricCurveType` data. At most `size` bytes are read.
fn read_curve(buf: &[u8], size: usize) -> Option<(SkcmsCurve, u32)> {
    if size < 4 || buf.len() < 4 {
        return None;
    }
    let type_ = read_big_u32(buf);
    if type_ == SKCMS_SIGNATURE_PARA {
        read_curve_para(buf, size)
    } else if type_ == SKCMS_SIGNATURE_CURV {
        read_curve_curv(buf, size)
    } else {
        None
    }
}

const MFT1_LAYOUT_FIXED: usize = 48;
const MFT2_LAYOUT_FIXED: usize = 52;

struct Tables {
    input_curves: Vec<SkcmsCurve>,
    grid: SkcmsGrid,
    output_curves: Vec<SkcmsCurve>,
}

fn init_tables(
    table_base: &[u8],
    byte_width: u32,
    input_table_entries: u32,
    output_table_entries: u32,
    input_channels: u32,
    output_channels: u32,
    grid_points: &[u8; 4],
) -> Option<Tables> {
    let byte_len_per_input_table = input_table_entries as usize * byte_width as usize;
    let byte_len_per_output_table = output_table_entries as usize * byte_width as usize;

    let byte_len_all_input_tables = input_channels as usize * byte_len_per_input_table;
    let byte_len_all_output_tables = output_channels as usize * byte_len_per_output_table;

    let mut grid_size = output_channels as u64 * byte_width as u64;
    for axis in 0..input_channels as usize {
        grid_size *= grid_points[axis] as u64;
    }

    if (table_base.len() as u64)
        < byte_len_all_input_tables as u64 + grid_size + byte_len_all_output_tables as u64
    {
        return None;
    }

    let make_curve = |offset: usize, entries: u32| -> SkcmsCurve {
        SkcmsCurve {
            table_entries: entries,
            parametric: SkcmsTransferFunction::default(),
            table: if byte_width == 1 {
                Some(SkcmsCurveTable::U8(
                    table_base[offset..offset + entries as usize].to_vec(),
                ))
            } else {
                let count = entries as usize;
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    let at = offset + 2 * i;
                    values.push(read_big_u16(&table_base[at..]));
                }
                Some(SkcmsCurveTable::U16(values))
            },
        }
    };

    let mut input_curves = Vec::with_capacity(input_channels as usize);
    for i in 0..input_channels as usize {
        input_curves.push(make_curve(
            i * byte_len_per_input_table,
            input_table_entries,
        ));
    }

    let grid_offset = byte_len_all_input_tables;
    let grid = if byte_width == 1 {
        SkcmsGrid::U8(table_base[grid_offset..grid_offset + grid_size as usize].to_vec())
    } else {
        let count = grid_size as usize / 2;
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let at = grid_offset + 2 * i;
            values.push(read_big_u16(&table_base[at..]));
        }
        SkcmsGrid::U16(values)
    };

    let output_table_base = grid_offset + grid_size as usize;
    let mut output_curves = Vec::with_capacity(output_channels as usize);
    for i in 0..output_channels as usize {
        output_curves.push(make_curve(
            output_table_base + i * byte_len_per_output_table,
            output_table_entries,
        ));
    }

    Some(Tables {
        input_curves,
        grid,
        output_curves,
    })
}

#[allow(clippy::too_many_arguments)]
fn read_mft_common(
    mft_tag: &[u8],
    is_a2b: bool,
    input_channels: &mut u32,
    output_channels: &mut u32,
    grid_points: &mut [u8; 4],
) -> bool {
    *input_channels = mft_tag[8] as u32;
    *output_channels = mft_tag[9] as u32;

    if is_a2b {
        // We require exactly three (ie XYZ/Lab/RGB) output channels.
        if *output_channels != 3 {
            return false;
        }
        // We require at least one, and no more than four (ie CMYK) input channels.
        if *input_channels < 1 || *input_channels > 4 {
            return false;
        }
    } else {
        // For B2A, exactly 3 input channels (XYZ) and 3 (RGB) or 4 (CMYK) output channels.
        if *input_channels != 3 {
            return false;
        }
        if *output_channels < 3 || *output_channels > 4 {
            return false;
        }
    }

    let mut total_grid_points: u64 = 1;
    for i in 0..*input_channels as usize {
        grid_points[i] = mft_tag[10];
        total_grid_points *= grid_points[i] as u64;
    }
    if grid_points[0] < 2 || total_grid_points > SKCMS_MAX_GRID_POINTS {
        return false;
    }
    true
}

fn read_tag_mft1_a2b(tag: &SkcmsIcctag, buf: &[u8], a2b: &mut SkcmsA2B) -> bool {
    if tag.size < MFT1_LAYOUT_FIXED as u32 || buf.len() < MFT1_LAYOUT_FIXED {
        return false;
    }
    let fixed = &buf[..MFT1_LAYOUT_FIXED];
    a2b.matrix_channels = 0;
    if !read_mft_common(
        fixed,
        true,
        &mut a2b.input_channels,
        &mut a2b.output_channels,
        &mut a2b.grid_points,
    ) {
        return false;
    }
    let variable = &buf[MFT1_LAYOUT_FIXED..];
    let Some(tables) = init_tables(
        variable,
        1,
        256,
        256,
        a2b.input_channels,
        a2b.output_channels,
        &a2b.grid_points,
    ) else {
        return false;
    };
    for (slot, curve) in a2b.input_curves.iter_mut().zip(tables.input_curves) {
        *slot = curve;
    }
    for (slot, curve) in a2b.output_curves.iter_mut().zip(tables.output_curves) {
        *slot = curve;
    }
    a2b.grid = Some(tables.grid);
    true
}

fn read_tag_mft2_a2b(tag: &SkcmsIcctag, buf: &[u8], a2b: &mut SkcmsA2B) -> bool {
    if tag.size < MFT2_LAYOUT_FIXED as u32 || buf.len() < MFT2_LAYOUT_FIXED {
        return false;
    }
    let fixed = &buf[..MFT2_LAYOUT_FIXED];
    a2b.matrix_channels = 0;
    if !read_mft_common(
        fixed,
        true,
        &mut a2b.input_channels,
        &mut a2b.output_channels,
        &mut a2b.grid_points,
    ) {
        return false;
    }

    let input_table_entries = read_big_u16(&fixed[48..]) as u32;
    let output_table_entries = read_big_u16(&fixed[50..]) as u32;

    // ICC spec mandates that 2 <= table_entries <= 4096.
    if !(2..=4096).contains(&input_table_entries) || !(2..=4096).contains(&output_table_entries) {
        return false;
    }

    let variable = &buf[MFT2_LAYOUT_FIXED..];
    let Some(tables) = init_tables(
        variable,
        2,
        input_table_entries,
        output_table_entries,
        a2b.input_channels,
        a2b.output_channels,
        &a2b.grid_points,
    ) else {
        return false;
    };
    for (slot, curve) in a2b.input_curves.iter_mut().zip(tables.input_curves) {
        *slot = curve;
    }
    for (slot, curve) in a2b.output_curves.iter_mut().zip(tables.output_curves) {
        *slot = curve;
    }
    a2b.grid = Some(tables.grid);
    true
}

fn read_tag_mft1_b2a(tag: &SkcmsIcctag, buf: &[u8], b2a: &mut SkcmsB2A) -> bool {
    if tag.size < MFT1_LAYOUT_FIXED as u32 || buf.len() < MFT1_LAYOUT_FIXED {
        return false;
    }
    let fixed = &buf[..MFT1_LAYOUT_FIXED];
    b2a.matrix_channels = 0;
    if !read_mft_common(
        fixed,
        false,
        &mut b2a.input_channels,
        &mut b2a.output_channels,
        &mut b2a.grid_points,
    ) {
        return false;
    }
    let variable = &buf[MFT1_LAYOUT_FIXED..];
    let Some(tables) = init_tables(
        variable,
        1,
        256,
        256,
        b2a.input_channels,
        b2a.output_channels,
        &b2a.grid_points,
    ) else {
        return false;
    };
    for (slot, curve) in b2a.input_curves.iter_mut().zip(tables.input_curves) {
        *slot = curve;
    }
    for (slot, curve) in b2a.output_curves.iter_mut().zip(tables.output_curves) {
        *slot = curve;
    }
    b2a.grid = Some(tables.grid);
    true
}

fn read_tag_mft2_b2a(tag: &SkcmsIcctag, buf: &[u8], b2a: &mut SkcmsB2A) -> bool {
    if tag.size < MFT2_LAYOUT_FIXED as u32 || buf.len() < MFT2_LAYOUT_FIXED {
        return false;
    }
    let fixed = &buf[..MFT2_LAYOUT_FIXED];
    b2a.matrix_channels = 0;
    if !read_mft_common(
        fixed,
        false,
        &mut b2a.input_channels,
        &mut b2a.output_channels,
        &mut b2a.grid_points,
    ) {
        return false;
    }

    let input_table_entries = read_big_u16(&fixed[48..]) as u32;
    let output_table_entries = read_big_u16(&fixed[50..]) as u32;

    if !(2..=4096).contains(&input_table_entries) || !(2..=4096).contains(&output_table_entries) {
        return false;
    }

    let variable = &buf[MFT2_LAYOUT_FIXED..];
    let Some(tables) = init_tables(
        variable,
        2,
        input_table_entries,
        output_table_entries,
        b2a.input_channels,
        b2a.output_channels,
        &b2a.grid_points,
    ) else {
        return false;
    };
    for (slot, curve) in b2a.input_curves.iter_mut().zip(tables.input_curves) {
        *slot = curve;
    }
    for (slot, curve) in b2a.output_curves.iter_mut().zip(tables.output_curves) {
        *slot = curve;
    }
    b2a.grid = Some(tables.grid);
    true
}

fn read_curves(
    buf: &[u8],
    size: u32,
    mut curve_offset: u32,
    num_curves: usize,
    curves: &mut [SkcmsCurve],
) -> bool {
    for curve in curves.iter_mut().take(num_curves) {
        if curve_offset > size {
            return false;
        }

        let Some((parsed, curve_bytes)) = read_curve(
            &buf[curve_offset as usize..],
            (size - curve_offset) as usize,
        ) else {
            return false;
        };

        if curve_bytes > u32::MAX - 3 {
            return false;
        }
        let curve_bytes = (curve_bytes + 3) & !3;

        let new_offset_64 = curve_offset as u64 + curve_bytes as u64;
        curve_offset = new_offset_64 as u32;
        if new_offset_64 != curve_offset as u64 {
            return false;
        }
        *curve = parsed;
    }
    true
}

const MAB_LAYOUT_SIZE: usize = 32;
const CLUT_LAYOUT_FIXED: usize = 20;

#[allow(clippy::too_many_arguments)]
fn read_tag_mab(
    tag: &SkcmsIcctag,
    buf: &[u8],
    a2b: &mut SkcmsA2B,
    pcs_is_xyz: bool,
    buffer_len: usize,
) -> bool {
    if tag.size < MAB_LAYOUT_SIZE as u32 {
        return false;
    }
    let buf = &buf[..tag.size as usize];
    if buf.len() < MAB_LAYOUT_SIZE {
        return false;
    }

    a2b.input_channels = buf[8] as u32;
    a2b.output_channels = buf[9] as u32;

    // We require exactly three (ie XYZ/Lab/RGB) output channels.
    if a2b.output_channels != 3 {
        return false;
    }
    // We require no more than four (ie CMYK) input channels.
    if a2b.input_channels > 4 {
        return false;
    }

    let b_curve_offset = read_big_u32(&buf[12..]);
    let matrix_offset = read_big_u32(&buf[16..]);
    let m_curve_offset = read_big_u32(&buf[20..]);
    let clut_offset = read_big_u32(&buf[24..]);
    let a_curve_offset = read_big_u32(&buf[28..]);

    // "B" curves must be present.
    if b_curve_offset == 0 {
        return false;
    }
    if !read_curves(
        buf,
        tag.size,
        b_curve_offset,
        a2b.output_channels as usize,
        &mut a2b.output_curves,
    ) {
        return false;
    }

    // "M" curves and Matrix must be used together.
    if m_curve_offset != 0 {
        if matrix_offset == 0 {
            return false;
        }
        a2b.matrix_channels = a2b.output_channels;
        if !read_curves(
            buf,
            tag.size,
            m_curve_offset,
            a2b.matrix_channels as usize,
            &mut a2b.matrix_curves,
        ) {
            return false;
        }

        if (tag.size as usize) < matrix_offset as usize + 12 * 4 {
            return false;
        }
        let encoding_factor = if pcs_is_xyz { 65535.0 / 32768.0 } else { 1.0 };
        let mtx = &buf[matrix_offset as usize..];
        for r in 0..3 {
            for c in 0..3 {
                a2b.matrix.vals[r][c] = encoding_factor * read_big_fixed(&mtx[4 * (r * 3 + c)..]);
            }
        }
        for (r, row) in a2b.matrix.vals.iter_mut().enumerate() {
            row[3] = encoding_factor * read_big_fixed(&mtx[36 + 4 * r..]);
        }
    } else {
        if matrix_offset != 0 {
            return false;
        }
        a2b.matrix_channels = 0;
    }

    // "A" curves and CLUT must be used together.
    if a_curve_offset != 0 {
        if clut_offset == 0 {
            return false;
        }
        if !read_curves(
            buf,
            tag.size,
            a_curve_offset,
            a2b.input_channels as usize,
            &mut a2b.input_curves,
        ) {
            return false;
        }

        if (tag.size as usize) < clut_offset as usize + CLUT_LAYOUT_FIXED {
            return false;
        }
        let clut = &buf[clut_offset as usize..];
        let grid_byte_width = clut[16] as u64;
        if grid_byte_width == 1 {
            a2b.grid = Some(SkcmsGrid::U8(Vec::new()));
        } else if grid_byte_width == 2 {
            a2b.grid = Some(SkcmsGrid::U16(Vec::new()));
        } else {
            return false;
        }

        let mut grid_size = a2b.output_channels as u64 * grid_byte_width;
        let mut total_grid_points: u64 = 1;
        for i in 0..a2b.input_channels as usize {
            a2b.grid_points[i] = clut[i];
            if a2b.grid_points[i] < 2 {
                return false;
            }
            grid_size *= a2b.grid_points[i] as u64;
            total_grid_points *= a2b.grid_points[i] as u64;
        }
        if total_grid_points > SKCMS_MAX_GRID_POINTS {
            return false;
        }

        let table_size = clut_offset as u64 + CLUT_LAYOUT_FIXED as u64 + grid_size;
        if table_size > tag.size as u64 {
            return false;
        }

        // gather_24 and gather_48 read 1 or 2 extra bytes.
        let mut slack = 0u64;
        if a2b.output_channels == 3 {
            slack = if grid_byte_width == 1 { 1 } else { 2 };
        }
        if tag.offset as u64 + table_size + slack > buffer_len as u64 {
            return false;
        }

        let payload = &clut[CLUT_LAYOUT_FIXED..];
        if grid_byte_width == 1 {
            a2b.grid = Some(SkcmsGrid::U8(payload[..grid_size as usize].to_vec()));
        } else {
            let count = grid_size as usize / 2;
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                values.push(read_big_u16(&payload[2 * i..]));
            }
            a2b.grid = Some(SkcmsGrid::U16(values));
        }
    } else {
        if clut_offset != 0 {
            return false;
        }
        // If there is no CLUT, the number of input and output channels must match.
        if a2b.input_channels != a2b.output_channels {
            return false;
        }
        // Zero out the number of input channels to signal that we're skipping this stage.
        a2b.input_channels = 0;
    }

    true
}

#[allow(clippy::too_many_arguments)]
fn read_tag_mba(
    tag: &SkcmsIcctag,
    buf: &[u8],
    b2a: &mut SkcmsB2A,
    pcs_is_xyz: bool,
    buffer_len: usize,
) -> bool {
    if tag.size < MAB_LAYOUT_SIZE as u32 {
        return false;
    }
    let buf = &buf[..tag.size as usize];
    if buf.len() < MAB_LAYOUT_SIZE {
        return false;
    }

    b2a.input_channels = buf[8] as u32;
    b2a.output_channels = buf[9] as u32;

    // Require exactly 3 inputs (XYZ) and 3 (RGB) or 4 (CMYK) outputs.
    if b2a.input_channels != 3 {
        return false;
    }
    if b2a.output_channels < 3 || b2a.output_channels > 4 {
        return false;
    }

    let b_curve_offset = read_big_u32(&buf[12..]);
    let matrix_offset = read_big_u32(&buf[16..]);
    let m_curve_offset = read_big_u32(&buf[20..]);
    let clut_offset = read_big_u32(&buf[24..]);
    let a_curve_offset = read_big_u32(&buf[28..]);

    if b_curve_offset == 0 {
        return false;
    }

    // "B" curves are our inputs, not outputs.
    if !read_curves(
        buf,
        tag.size,
        b_curve_offset,
        b2a.input_channels as usize,
        &mut b2a.input_curves,
    ) {
        return false;
    }

    if m_curve_offset != 0 {
        if matrix_offset == 0 {
            return false;
        }
        // Matrix channels is tied to input_channels (3), not output_channels.
        b2a.matrix_channels = b2a.input_channels;

        if !read_curves(
            buf,
            tag.size,
            m_curve_offset,
            b2a.matrix_channels as usize,
            &mut b2a.matrix_curves,
        ) {
            return false;
        }

        if (tag.size as usize) < matrix_offset as usize + 12 * 4 {
            return false;
        }
        let encoding_factor = if pcs_is_xyz { 32768.0 / 65535.0 } else { 1.0 };
        let mtx = &buf[matrix_offset as usize..];
        for r in 0..3 {
            for c in 0..3 {
                b2a.matrix.vals[r][c] = encoding_factor * read_big_fixed(&mtx[4 * (r * 3 + c)..]);
            }
        }
        for (r, row) in b2a.matrix.vals.iter_mut().enumerate() {
            row[3] = encoding_factor * read_big_fixed(&mtx[36 + 4 * r..]);
        }
    } else {
        if matrix_offset != 0 {
            return false;
        }
        b2a.matrix_channels = 0;
    }

    if a_curve_offset != 0 {
        if clut_offset == 0 {
            return false;
        }

        // "A" curves are our output, not input.
        if !read_curves(
            buf,
            tag.size,
            a_curve_offset,
            b2a.output_channels as usize,
            &mut b2a.output_curves,
        ) {
            return false;
        }

        if (tag.size as usize) < clut_offset as usize + CLUT_LAYOUT_FIXED {
            return false;
        }
        let clut = &buf[clut_offset as usize..];
        let grid_byte_width = clut[16] as u64;
        if grid_byte_width == 1 {
            b2a.grid = Some(SkcmsGrid::U8(Vec::new()));
        } else if grid_byte_width == 2 {
            b2a.grid = Some(SkcmsGrid::U16(Vec::new()));
        } else {
            return false;
        }

        let mut grid_size = b2a.output_channels as u64 * grid_byte_width;
        let mut total_grid_points: u64 = 1;
        for i in 0..b2a.input_channels as usize {
            b2a.grid_points[i] = clut[i];
            if b2a.grid_points[i] < 2 {
                return false;
            }
            grid_size *= b2a.grid_points[i] as u64;
            total_grid_points *= b2a.grid_points[i] as u64;
        }
        if total_grid_points > SKCMS_MAX_GRID_POINTS {
            return false;
        }
        if (tag.size as u64) < clut_offset as u64 + CLUT_LAYOUT_FIXED as u64 + grid_size {
            return false;
        }

        let mut slack = 0u64;
        if b2a.output_channels == 3 {
            slack = if grid_byte_width == 1 { 1 } else { 2 };
        }
        if tag.offset as u64 + clut_offset as u64 + CLUT_LAYOUT_FIXED as u64 + grid_size + slack
            > buffer_len as u64
        {
            return false;
        }

        let payload = &clut[CLUT_LAYOUT_FIXED..];
        if grid_byte_width == 1 {
            b2a.grid = Some(SkcmsGrid::U8(payload[..grid_size as usize].to_vec()));
        } else {
            let count = grid_size as usize / 2;
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                values.push(read_big_u16(&payload[2 * i..]));
            }
            b2a.grid = Some(SkcmsGrid::U16(values));
        }
    } else {
        if clut_offset != 0 {
            return false;
        }
        if b2a.input_channels != b2a.output_channels {
            return false;
        }
        // Zero out *output* channels to skip this stage.
        b2a.output_channels = 0;
    }
    true
}

fn read_a2b(
    tag: &SkcmsIcctag,
    buf: &[u8],
    a2b: &mut SkcmsA2B,
    pcs_is_xyz: bool,
    buffer_len: usize,
) -> bool {
    if tag.type_ == SKCMS_SIGNATURE_MFT1 {
        return read_tag_mft1_a2b(tag, buf, a2b);
    }
    if tag.type_ == SKCMS_SIGNATURE_MFT2 {
        return read_tag_mft2_a2b(tag, buf, a2b);
    }
    if tag.type_ == SKCMS_SIGNATURE_MAB {
        return read_tag_mab(tag, buf, a2b, pcs_is_xyz, buffer_len);
    }
    false
}

fn read_b2a(
    tag: &SkcmsIcctag,
    buf: &[u8],
    b2a: &mut SkcmsB2A,
    pcs_is_xyz: bool,
    buffer_len: usize,
) -> bool {
    if tag.type_ == SKCMS_SIGNATURE_MFT1 {
        return read_tag_mft1_b2a(tag, buf, b2a);
    }
    if tag.type_ == SKCMS_SIGNATURE_MFT2 {
        return read_tag_mft2_b2a(tag, buf, b2a);
    }
    if tag.type_ == SKCMS_SIGNATURE_MBA {
        return read_tag_mba(tag, buf, b2a, pcs_is_xyz, buffer_len);
    }
    false
}

const CICP_LAYOUT_SIZE: usize = 12;

fn read_cicp_from_buffer(tag: &SkcmsIcctag, buf: &[u8], cicp: &mut SkcmsCicp) -> bool {
    if tag.type_ != SKCMS_SIGNATURE_CICP || tag.size < CICP_LAYOUT_SIZE as u32 {
        return false;
    }
    if buf.len() < CICP_LAYOUT_SIZE {
        return false;
    }
    cicp.color_primaries = buf[8];
    cicp.transfer_characteristics = buf[9];
    cicp.matrix_coefficients = buf[10];
    cicp.video_full_range_flag = buf[11];
    true
}

const HAGC_LAYOUT_SIZE: usize = 12;

fn read_hagc(tag: &SkcmsIcctag, buf: &[u8], hagc: &mut SkcmsHagc) -> bool {
    if tag.type_ != SKCMS_SIGNATURE_HAGC_TYPE || tag.size < HAGC_LAYOUT_SIZE as u32 {
        return false;
    }
    if buf.len() < HAGC_LAYOUT_SIZE {
        return false;
    }
    let size = read_big_u32(&buf[8..]);
    if size > tag.size - HAGC_LAYOUT_SIZE as u32 {
        return false;
    }
    hagc.size = size;
    hagc.buffer = buf[HAGC_LAYOUT_SIZE..HAGC_LAYOUT_SIZE + size as usize].to_vec();
    true
}

fn usable_as_src(profile: &SkcmsIccProfile) -> bool {
    profile.has_a2b || (profile.has_trc && profile.has_to_xyzd50)
}

/// `skcms_ParseWithA2BPriority`.
pub fn skcms_parse_with_a2b_priority(
    buf: &[u8],
    priority: &[i32],
    profile: &mut SkcmsIccProfile,
) -> bool {
    const _: () = assert!(HEADER_LAYOUT_SIZE == 132, "need to update header code");

    *profile = SkcmsIccProfile::default();

    if buf.len() < HEADER_LAYOUT_SIZE {
        return false;
    }

    let header = &buf[..HEADER_LAYOUT_SIZE];
    profile.buffer = buf.to_vec();
    profile.size = read_big_u32(&header[0..]);
    let version = read_big_u32(&header[8..]);
    profile.data_color_space = read_big_u32(&header[16..]);
    profile.pcs = read_big_u32(&header[20..]);
    let signature = read_big_u32(&header[36..]);
    let illuminant_x = read_big_fixed(&header[68..]);
    let illuminant_y = read_big_fixed(&header[72..]);
    let illuminant_z = read_big_fixed(&header[76..]);
    profile.tag_count = read_big_u32(&header[128..]);

    // Validate signature, size (smaller than buffer, large enough to hold tag table), and major
    // version.
    let tag_table_size = profile.tag_count as u64 * TAG_LAYOUT_SIZE as u64;
    if signature != SKCMS_SIGNATURE_ACSP
        || profile.size as u64 > buf.len() as u64
        || (profile.size as u64) < HEADER_LAYOUT_SIZE as u64 + tag_table_size
        || (version >> 24) > 4
    {
        return false;
    }

    // Validate that illuminant is D50 white.
    if fabsf_(illuminant_x - 0.9642) > 0.0100
        || fabsf_(illuminant_y - 1.0000) > 0.0100
        || fabsf_(illuminant_z - 0.8249) > 0.0100
    {
        return false;
    }

    // Validate that all tag entries have sane offset + size.
    for i in 0..profile.tag_count {
        let entry =
            &profile.buffer[HEADER_LAYOUT_SIZE + i as usize * TAG_LAYOUT_SIZE..][..TAG_LAYOUT_SIZE];
        let tag_offset = read_big_u32(&entry[4..]);
        let tag_size = read_big_u32(&entry[8..]);
        let tag_end = tag_offset as u64 + tag_size as u64;
        if tag_size < 4 || tag_end > profile.size as u64 {
            return false;
        }
    }

    if profile.pcs != SKCMS_SIGNATURE_XYZ && profile.pcs != SKCMS_SIGNATURE_LAB {
        return false;
    }

    let pcs_is_xyz = profile.pcs == SKCMS_SIGNATURE_XYZ;

    // Pre-parse commonly used tags.
    if profile.data_color_space == SKCMS_SIGNATURE_GRAY {
        if let Some(ktag) = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_KTRC) {
            let (start, end) = tag_bounds(&ktag);
            let Some((curve, _)) = read_curve(&profile.buffer[start..end], ktag.size as usize)
            else {
                return false;
            };
            profile.trc[0] = curve.clone();
            profile.trc[1] = curve.clone();
            profile.trc[2] = curve;
            profile.has_trc = true;

            if pcs_is_xyz {
                profile.to_xyzd50.vals[0][0] = illuminant_x;
                profile.to_xyzd50.vals[1][1] = illuminant_y;
                profile.to_xyzd50.vals[2][2] = illuminant_z;
                profile.has_to_xyzd50 = true;
            }
        }
    } else {
        let rtrc = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_RTRC);
        let gtrc = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_GTRC);
        let btrc = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_BTRC);
        if let (Some(rtrc), Some(gtrc), Some(btrc)) = (rtrc, gtrc, btrc) {
            let (rs, re) = tag_bounds(&rtrc);
            let (gs, ge) = tag_bounds(&gtrc);
            let (bs, be) = tag_bounds(&btrc);
            let rcurve = read_curve(&profile.buffer[rs..re], rtrc.size as usize);
            let gcurve = read_curve(&profile.buffer[gs..ge], gtrc.size as usize);
            let bcurve = read_curve(&profile.buffer[bs..be], btrc.size as usize);
            let (Some((rcurve, _)), Some((gcurve, _)), Some((bcurve, _))) =
                (rcurve, gcurve, bcurve)
            else {
                // Malformed TRC tags.
                return false;
            };
            profile.trc = [rcurve, gcurve, bcurve];
            profile.has_trc = true;
        }

        let rxyz = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_RXYZ);
        let gxyz = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_GXYZ);
        let bxyz = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_BXYZ);
        if let (Some(rxyz), Some(gxyz), Some(bxyz)) = (rxyz, gxyz, bxyz) {
            let mut to_xyz = SkcmsMatrix3x3::default();
            if !read_to_xyzd50(profile, &rxyz, &gxyz, &bxyz, &mut to_xyz) {
                // Malformed XYZ tags.
                return false;
            }
            profile.to_xyzd50 = to_xyz;
            profile.has_to_xyzd50 = true;
        }
    }

    let end_of_buffer = buf.len();
    for &priority_index in priority {
        // enum { perceptual, relative_colormetric, saturation }
        if !(0..=2).contains(&priority_index) {
            return false;
        }
        let sig = SKCMS_SIGNATURE_A2B0 + priority_index as u32;
        if let Some(tag) = skcms_get_tag_by_signature(profile, sig) {
            let (start, end) = tag_bounds(&tag);
            if !read_a2b(
                &tag,
                &profile.buffer[start..end],
                &mut profile.a2b,
                pcs_is_xyz,
                end_of_buffer,
            ) {
                // Malformed A2B tag.
                return false;
            }
            profile.has_a2b = true;
            break;
        }
    }

    for &priority_index in priority {
        if !(0..=2).contains(&priority_index) {
            return false;
        }
        let sig = SKCMS_SIGNATURE_B2A0 + priority_index as u32;
        if let Some(tag) = skcms_get_tag_by_signature(profile, sig) {
            let (start, end) = tag_bounds(&tag);
            if !read_b2a(
                &tag,
                &profile.buffer[start..end],
                &mut profile.b2a,
                pcs_is_xyz,
                end_of_buffer,
            ) {
                // Malformed B2A tag.
                return false;
            }
            profile.has_b2a = true;
            break;
        }
    }

    if let Some(tag) = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_CICP) {
        let mut cicp = SkcmsCicp::default();
        let (start, end) = tag_bounds(&tag);
        if !read_cicp_from_buffer(&tag, &profile.buffer[start..end], &mut cicp) {
            // Malformed CICP tag.
            return false;
        }
        profile.cicp = cicp;
        profile.has_cicp = true;
    }

    if let Some(tag) = skcms_get_tag_by_signature(profile, SKCMS_SIGNATURE_HAGC) {
        let mut hagc = SkcmsHagc::default();
        let (start, end) = tag_bounds(&tag);
        if !read_hagc(&tag, &profile.buffer[start..end], &mut hagc) {
            // Malformed HAGC tag.
            return false;
        }
        profile.hagc = hagc;
        profile.has_hagc = true;
    }

    usable_as_src(profile)
}

/// `skcms_Parse`: prefers A2B0 (perceptual) over A2B1 (relative colorimetric).
pub fn skcms_parse(buf: &[u8], profile: &mut SkcmsIccProfile) -> bool {
    const PRIORITY: [i32; 2] = [0, 1];
    skcms_parse_with_a2b_priority(buf, &PRIORITY, profile)
}

// ~~~~ Canonical profiles ~~~~

const SRGB_TRC: SkcmsTransferFunction = SkcmsTransferFunction::new(
    2.4,
    1.0 / 1.055,
    0.055 / 1.055,
    1.0 / 12.92,
    0.04045,
    0.0,
    0.0,
);

static SRGB_PROFILE: OnceLock<SkcmsIccProfile> = OnceLock::new();
static XYZD50_PROFILE: OnceLock<SkcmsIccProfile> = OnceLock::new();

fn build_srgb_profile() -> SkcmsIccProfile {
    let mut profile = SkcmsIccProfile {
        data_color_space: SKCMS_SIGNATURE_RGB,
        pcs: SKCMS_SIGNATURE_XYZ,
        has_trc: true,
        has_to_xyzd50: true,
        ..Default::default()
    };
    let curve = SkcmsCurve::from_parametric(SRGB_TRC);
    profile.trc = [curve.clone(), curve.clone(), curve];
    profile.to_xyzd50 = SkcmsMatrix3x3 {
        vals: [
            [0.436_065_674, 0.385_147_095, 0.143_066_406],
            [0.222_488_403, 0.716_873_169, 0.060_607_910],
            [0.013_916_016, 0.097_076_416, 0.714_096_069],
        ],
    };
    profile
}

fn build_xyzd50_profile() -> SkcmsIccProfile {
    let mut profile = SkcmsIccProfile {
        data_color_space: SKCMS_SIGNATURE_RGB,
        pcs: SKCMS_SIGNATURE_XYZ,
        has_trc: true,
        has_to_xyzd50: true,
        ..Default::default()
    };
    let curve = SkcmsCurve::from_parametric(SkcmsTransferFunction::new(
        1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ));
    profile.trc = [curve.clone(), curve.clone(), curve];
    profile.to_xyzd50 = SkcmsMatrix3x3 {
        vals: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    profile
}

/// `skcms_sRGB_profile`.
pub fn skcms_srgb_profile() -> &'static SkcmsIccProfile {
    SRGB_PROFILE.get_or_init(build_srgb_profile)
}

/// `skcms_XYZD50_profile`.
pub fn skcms_xyzd50_profile() -> &'static SkcmsIccProfile {
    XYZD50_PROFILE.get_or_init(build_xyzd50_profile)
}

/// `skcms_sRGB_TransferFunction`.
pub fn skcms_srgb_transfer_function() -> &'static SkcmsTransferFunction {
    &skcms_srgb_profile().trc[0].parametric
}

/// `skcms_sRGB_Inverse_TransferFunction`.
pub fn skcms_srgb_inverse_transfer_function() -> &'static SkcmsTransferFunction {
    static SRGB_INV: SkcmsTransferFunction = SkcmsTransferFunction::new(
        0.416_666_657,
        1.137_283_325,
        -0.0,
        12.920_000_076,
        0.003_130_805,
        -0.054_969_788,
        -0.0,
    );
    &SRGB_INV
}

/// `skcms_Identity_TransferFunction`.
pub fn skcms_identity_transfer_function() -> &'static SkcmsTransferFunction {
    static IDENTITY: SkcmsTransferFunction =
        SkcmsTransferFunction::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    &IDENTITY
}

/// `skcms_252_random_bytes`, used for profile equivalence testing.
pub static SKCMS_252_RANDOM_BYTES: [u8; 252] = [
    8, 179, 128, 204, 253, 38, 134, 184, 68, 102, 32, 138, 99, 39, 169, 215, 119, 26, 3, 223, 95,
    239, 52, 132, 114, 74, 81, 234, 97, 116, 244, 205, 30, 154, 173, 12, 51, 159, 122, 153, 61,
    226, 236, 178, 229, 55, 181, 220, 191, 194, 160, 126, 168, 82, 131, 18, 180, 245, 163, 22, 246,
    69, 235, 252, 57, 108, 14, 6, 152, 240, 255, 171, 242, 20, 227, 177, 238, 96, 85, 16, 211, 70,
    200, 149, 155, 146, 127, 145, 100, 151, 109, 19, 165, 208, 195, 164, 137, 254, 182, 248, 64,
    201, 45, 209, 5, 147, 207, 210, 113, 162, 83, 225, 9, 31, 15, 231, 115, 37, 58, 53, 24, 49,
    197, 56, 120, 172, 48, 21, 214, 129, 111, 11, 50, 187, 196, 34, 60, 103, 71, 144, 47, 203, 77,
    80, 232, 140, 222, 250, 206, 166, 247, 139, 249, 221, 72, 106, 27, 199, 117, 54, 219, 135, 118,
    40, 79, 41, 251, 46, 93, 212, 92, 233, 148, 28, 121, 63, 123, 158, 105, 59, 29, 42, 143, 23, 0,
    107, 176, 87, 104, 183, 156, 193, 189, 90, 188, 65, 190, 17, 198, 7, 186, 161, 1, 124, 78, 125,
    170, 133, 174, 218, 67, 157, 75, 101, 89, 217, 62, 33, 141, 228, 25, 35, 91, 230, 4, 2, 13, 73,
    86, 167, 237, 84, 243, 44, 185, 66, 130, 110, 150, 142, 216, 88, 112, 36, 224, 136, 202, 76,
    94, 98, 175, 213,
];

/// `skcms_TRCs_AreApproximateInverse`.
pub fn skcms_trcs_are_approximate_inverse(
    profile: &SkcmsIccProfile,
    inv_tf: &SkcmsTransferFunction,
) -> bool {
    if !profile.has_trc {
        return false;
    }
    skcms_are_approximate_inverses(&profile.trc[0], inv_tf)
        && skcms_are_approximate_inverses(&profile.trc[1], inv_tf)
        && skcms_are_approximate_inverses(&profile.trc[2], inv_tf)
}

/// `skcms_ApproximatelyEqualProfiles`.
pub fn skcms_approximately_equal_profiles(a: &SkcmsIccProfile, b: &SkcmsIccProfile) -> bool {
    if std::ptr::eq(a, b) || a == b {
        return true;
    }

    const CMYK: u32 = SKCMS_SIGNATURE_CMYK;
    if (a.data_color_space == CMYK) != (b.data_color_space == CMYK) {
        return false;
    }

    let (fmt, npixels) = if a.data_color_space == CMYK {
        (crate::skcms::SkcmsPixelFormat::Rgba8888, 63)
    } else {
        (crate::skcms::SkcmsPixelFormat::Rgb888, 84)
    };

    let mut dst_a = [0u8; 252];
    let mut dst_b = [0u8; 252];
    if !crate::skcms_transform::skcms_transform(
        &SKCMS_252_RANDOM_BYTES,
        fmt,
        SkcmsAlphaFormat::Unpremul,
        Some(a),
        &mut dst_a,
        SkcmsPixelFormat::Rgb888,
        SkcmsAlphaFormat::Unpremul,
        Some(skcms_xyzd50_profile()),
        npixels,
    ) {
        return false;
    }
    if !crate::skcms_transform::skcms_transform(
        &SKCMS_252_RANDOM_BYTES,
        fmt,
        SkcmsAlphaFormat::Unpremul,
        Some(b),
        &mut dst_b,
        SkcmsPixelFormat::Rgb888,
        SkcmsAlphaFormat::Unpremul,
        Some(skcms_xyzd50_profile()),
        npixels,
    ) {
        return false;
    }

    for i in 0..252 {
        if (dst_a[i] as i32 - dst_b[i] as i32).abs() > 1 {
            return false;
        }
    }
    true
}

// ~~~~ xy / primaries helpers ~~~~

fn is_zero_to_one(x: f32) -> bool {
    (0.0..=1.0).contains(&x)
}

fn is_valid_white_point(wx: f32, wy: f32) -> bool {
    is_zero_to_one(wx) && is_zero_to_one(wy) && wy != 0.0
}

/// `skcms_AdaptToXYZD50`.
pub fn skcms_adapt_to_xyzd50(wx: f32, wy: f32, to_xyzd50: &mut SkcmsMatrix3x3) -> bool {
    if !is_valid_white_point(wx, wy) {
        return false;
    }

    // Assumes that Y is 1.0f.
    let w_xyz = SkcmsVector3 {
        vals: [wx / wy, 1.0, (1.0 - wx - wy) / wy],
    };

    // Now convert toXYZ matrix to toXYZD50.
    let w_xyzd50 = SkcmsVector3 {
        vals: [0.96422, 1.0, 0.82521],
    };

    // Calculate the chromatic adaptation matrix. We use the Bradford method.
    let xyz_to_lms = SkcmsMatrix3x3 {
        vals: [
            [0.8951, 0.2664, -0.1614],
            [-0.7502, 1.7135, 0.0367],
            [0.0389, -0.0685, 1.0296],
        ],
    };
    let lms_to_xyz = SkcmsMatrix3x3 {
        vals: [
            [0.986_992_9, -0.147_054_3, 0.159_962_7],
            [0.432_305_3, 0.518_360_3, 0.049_291_2],
            [-0.008_528_7, 0.040_042_8, 0.968_486_7],
        ],
    };

    let src_cone = mv_mul(&xyz_to_lms, &w_xyz);
    let dst_cone = mv_mul(&xyz_to_lms, &w_xyzd50);

    // In-range white points far outside the spectral locus can have a zero cone response.
    if src_cone.vals[0] == 0.0 || src_cone.vals[1] == 0.0 || src_cone.vals[2] == 0.0 {
        return false;
    }

    let mut m = SkcmsMatrix3x3 {
        vals: [
            [dst_cone.vals[0] / src_cone.vals[0], 0.0, 0.0],
            [0.0, dst_cone.vals[1] / src_cone.vals[1], 0.0],
            [0.0, 0.0, dst_cone.vals[2] / src_cone.vals[2]],
        ],
    };
    m = skcms_matrix3x3_concat(&m, &xyz_to_lms);
    m = skcms_matrix3x3_concat(&lms_to_xyz, &m);
    if !skcms_matrix3x3_isfinite(&m) {
        return false;
    }

    *to_xyzd50 = m;
    true
}

/// `skcms_PrimariesToXYZD50`.
#[allow(clippy::too_many_arguments)]
pub fn skcms_primaries_to_xyzd50(
    rx: f32,
    ry: f32,
    gx: f32,
    gy: f32,
    bx: f32,
    by: f32,
    wx: f32,
    wy: f32,
    to_xyzd50: &mut SkcmsMatrix3x3,
) -> bool {
    if !is_zero_to_one(rx)
        || !is_zero_to_one(ry)
        || !is_zero_to_one(gx)
        || !is_zero_to_one(gy)
        || !is_zero_to_one(bx)
        || !is_zero_to_one(by)
        || !is_valid_white_point(wx, wy)
    {
        return false;
    }

    // First, we need to convert xy values (primaries) to XYZ.
    let primaries = SkcmsMatrix3x3 {
        vals: [
            [rx, gx, bx],
            [ry, gy, by],
            [1.0 - rx - ry, 1.0 - gx - gy, 1.0 - bx - by],
        ],
    };
    let mut primaries_inv = SkcmsMatrix3x3::default();
    if !skcms_matrix3x3_invert(&primaries, &mut primaries_inv) {
        return false;
    }

    // Assumes that Y is 1.0f.
    let w_xyz = SkcmsVector3 {
        vals: [wx / wy, 1.0, (1.0 - wx - wy) / wy],
    };
    let xyz = mv_mul(&primaries_inv, &w_xyz);

    let mut to_xyz = SkcmsMatrix3x3 {
        vals: [
            [xyz.vals[0], 0.0, 0.0],
            [0.0, xyz.vals[1], 0.0],
            [0.0, 0.0, xyz.vals[2]],
        ],
    };
    to_xyz = skcms_matrix3x3_concat(&primaries, &to_xyz);

    let mut dx_to_d50 = SkcmsMatrix3x3::default();
    if !skcms_adapt_to_xyzd50(wx, wy, &mut dx_to_d50) {
        return false;
    }

    let m = skcms_matrix3x3_concat(&dx_to_d50, &to_xyz);
    if !skcms_matrix3x3_isfinite(&m) {
        return false;
    }

    *to_xyzd50 = m;
    true
}

// ~~~~ Pixel formats ~~~~

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SkcmsPixelFormat {
    A8 = 0,
    A8Alt = 1,
    G8 = 2,
    G8Alt = 3,
    Ga88 = 4,
    Ga88Alt = 5,
    Rgb565 = 6,
    Bgr565 = 7,
    Abgr4444 = 8,
    Argb4444 = 9,
    Rgb888 = 10,
    Bgr888 = 11,
    Rgba8888 = 12,
    Bgra8888 = 13,
    Rgba8888Srgb = 14,
    Bgra8888Srgb = 15,
    Rgba1010102 = 16,
    Bgra1010102 = 17,
    Rgb161616Le = 18,
    Bgr161616Le = 19,
    Rgba16161616Le = 20,
    Bgra16161616Le = 21,
    Rgb161616Be = 22,
    Bgr161616Be = 23,
    Rgba16161616Be = 24,
    Bgra16161616Be = 25,
    RgbHhhNorm = 26,
    BgrHhhNorm = 27,
    RgbaHhhhNorm = 28,
    BgraHhhhNorm = 29,
    RgbHhh = 30,
    BgrHhh = 31,
    RgbaHhhh = 32,
    BgraHhhh = 33,
    RgbFff = 34,
    BgrFff = 35,
    RgbaFfff = 36,
    BgraFfff = 37,
    Rgb101010xXr = 38,
    Bgr101010xXr = 39,
    Rgba10101010Xr = 40,
    Bgra10101010Xr = 41,
}

impl SkcmsPixelFormat {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    /// The reference's `fmt & 1` swap-RB bit.
    pub const fn swaps_rb(self) -> bool {
        self as u32 & 1 != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SkcmsAlphaFormat {
    Opaque = 0,
    Unpremul = 1,
    PremulAsEncoded = 2,
}

/// `bytes_per_pixel`.
pub fn skcms_bytes_per_pixel(fmt: SkcmsPixelFormat) -> usize {
    use SkcmsPixelFormat::*;
    match fmt {
        A8 | A8Alt | G8 | G8Alt => 1,
        Ga88 | Ga88Alt | Abgr4444 | Argb4444 | Rgb565 | Bgr565 => 2,
        Rgb888 | Bgr888 => 3,
        Rgba8888 | Bgra8888 | Rgba8888Srgb | Bgra8888Srgb | Rgba1010102 | Bgra1010102
        | Rgb101010xXr | Bgr101010xXr => 4,
        Rgb161616Le | Bgr161616Le | Rgb161616Be | Bgr161616Be | RgbHhhNorm | BgrHhhNorm
        | RgbHhh | BgrHhh => 6,
        Rgba16161616Le | Bgra16161616Le | Rgba16161616Be | Bgra16161616Be | RgbaHhhhNorm
        | BgraHhhhNorm | RgbaHhhh | BgraHhhh | Rgba10101010Xr | Bgra10101010Xr => 8,
        RgbFff | BgrFff => 12,
        RgbaFfff | BgraFfff => 16,
    }
}

// ~~~~ Destination preparation ~~~~

/// `has_cicp_pq_trc`.
pub fn skcms_has_cicp_pq_trc(profile: &SkcmsIccProfile) -> bool {
    profile.has_cicp && profile.cicp.transfer_characteristics == 16
}

/// `has_cicp_hlg_trc`.
pub fn skcms_has_cicp_hlg_trc(profile: &SkcmsIccProfile) -> bool {
    profile.has_cicp && profile.cicp.transfer_characteristics == 18
}

/// `set_reference_pq_ish_trc`.
pub fn skcms_set_reference_pq_ish_trc(tf: &mut SkcmsTransferFunction) {
    skcms_transfer_function_make_pqish(
        tf,
        -107.0 / 128.0,
        1.0,
        32.0 / 2523.0,
        2413.0 / 128.0,
        -2392.0 / 128.0,
        8192.0 / 1305.0,
    );

    const W: f32 = 10000.0 / 203.0;
    let ws = powf_(W, 1.0 / tf.f);
    tf.a *= ws;
    tf.b *= ws;
}

/// `set_sdr_hlg_ish_trc`.
pub fn skcms_set_sdr_hlg_ish_trc(tf: &mut SkcmsTransferFunction) {
    skcms_transfer_function_make_hlgish(
        tf,
        2.0,
        2.0,
        1.0 / 0.178_832_77,
        0.284_668_92,
        0.559_910_73,
    );
    tf.f = 1.0 / 12.0 - 1.0;
}

/// The result of `prep_for_destination`.
pub struct SkcmsDestination {
    pub from_xyzd50: SkcmsMatrix3x3,
    pub inv: [SkcmsTransferFunction; 3],
    pub using_b2a: bool,
    pub using_hlg_ootf: bool,
}

/// `prep_for_destination`.
pub fn skcms_prep_for_destination(profile: &SkcmsIccProfile) -> Option<SkcmsDestination> {
    let mut from_xyzd50 = SkcmsMatrix3x3::default();
    let has_xyzd50 =
        profile.has_to_xyzd50 && skcms_matrix3x3_invert(&profile.to_xyzd50, &mut from_xyzd50);

    // CICP-specified PQ or HLG transfer functions take precedence.
    if skcms_has_cicp_pq_trc(profile) && has_xyzd50 {
        let mut trc_pq = SkcmsTransferFunction::default();
        skcms_set_reference_pq_ish_trc(&mut trc_pq);
        let mut inv = [SkcmsTransferFunction::default(); 3];
        for slot in &mut inv {
            if !skcms_transfer_function_invert(&trc_pq, slot) {
                return None;
            }
        }
        return Some(SkcmsDestination {
            from_xyzd50,
            inv,
            using_b2a: false,
            using_hlg_ootf: false,
        });
    }
    if skcms_has_cicp_hlg_trc(profile) && has_xyzd50 {
        let mut trc_hlg = SkcmsTransferFunction::default();
        skcms_set_sdr_hlg_ish_trc(&mut trc_hlg);
        let mut inv = [SkcmsTransferFunction::default(); 3];
        for slot in &mut inv {
            if !skcms_transfer_function_invert(&trc_hlg, slot) {
                return None;
            }
        }
        return Some(SkcmsDestination {
            from_xyzd50,
            inv,
            using_b2a: false,
            using_hlg_ootf: true,
        });
    }

    // Then prefer the B2A transformation.
    if profile.has_b2a {
        return Some(SkcmsDestination {
            from_xyzd50,
            inv: [SkcmsTransferFunction::default(); 3],
            using_b2a: true,
            using_hlg_ootf: false,
        });
    }

    // Finally use parametric transfer functions.
    if has_xyzd50
        && profile.has_trc
        && profile.trc[0].table_entries == 0
        && profile.trc[1].table_entries == 0
        && profile.trc[2].table_entries == 0
    {
        let mut inv = [SkcmsTransferFunction::default(); 3];
        for i in 0..3 {
            if !skcms_transfer_function_invert(&profile.trc[i].parametric, &mut inv[i]) {
                return None;
            }
        }
        return Some(SkcmsDestination {
            from_xyzd50,
            inv,
            using_b2a: false,
            using_hlg_ootf: false,
        });
    }

    None
}

/// `skcms_MakeUsableAsDestination`.
pub fn skcms_make_usable_as_destination(profile: &mut SkcmsIccProfile) -> bool {
    if !profile.has_b2a {
        let mut from_xyzd50 = SkcmsMatrix3x3::default();
        if !profile.has_trc
            || !profile.has_to_xyzd50
            || !skcms_matrix3x3_invert(&profile.to_xyzd50, &mut from_xyzd50)
        {
            return false;
        }

        let mut tf = [SkcmsTransferFunction::default(); 3];
        for i in 0..3 {
            let mut inv = SkcmsTransferFunction::default();
            if profile.trc[i].table_entries == 0
                && skcms_transfer_function_invert(&profile.trc[i].parametric, &mut inv)
            {
                tf[i] = profile.trc[i].parametric;
                continue;
            }

            let mut max_error = 0.0;
            // Parametric curves from skcms_ApproximateCurve() are guaranteed to be invertible.
            if !skcms_approximate_curve(&profile.trc[i], &mut tf[i], &mut max_error) {
                return false;
            }
        }

        for i in 0..3 {
            profile.trc[i].table_entries = 0;
            profile.trc[i].parametric = tf[i];
            profile.trc[i].table = None;
        }
    }
    debug_assert!(skcms_prep_for_destination(profile).is_some());
    true
}

/// `skcms_MakeUsableAsDestinationWithSingleCurve`.
pub fn skcms_make_usable_as_destination_with_single_curve(profile: &mut SkcmsIccProfile) -> bool {
    // Call skcms_MakeUsableAsDestination() with B2A disabled.
    let mut result = profile.clone();
    result.has_b2a = false;
    if !skcms_make_usable_as_destination(&mut result) {
        return false;
    }

    // Of the three, pick the transfer function that best fits the other two.
    let mut best_tf = 0usize;
    let mut min_max_error = f32::INFINITY;
    for i in 0..3 {
        let mut inv = SkcmsTransferFunction::default();
        if !skcms_transfer_function_invert(&result.trc[i].parametric, &mut inv) {
            return false;
        }

        let mut err = 0.0f32;
        for j in 0..3 {
            err = fmaxf_(err, skcms_max_roundtrip_error(&profile.trc[j], &inv));
        }
        if min_max_error > err {
            min_max_error = err;
            best_tf = i;
        }
    }

    for i in 0..3 {
        result.trc[i].parametric = result.trc[best_tf].parametric;
    }

    *profile = result;
    debug_assert!(skcms_prep_for_destination(profile).is_some());
    true
}

/// Whether runtime CPU detection is still enabled (used only by the x86-64 dispatch, which is
/// excluded on macOS arm64; exposed for parity checks).
pub fn skcms_runtime_cpu_detection_enabled() -> bool {
    allow_runtime_cpu_detection()
}

/// Hash helper used by `SkColorSpace`.
pub(crate) fn hash_floats(values: &[f32]) -> u32 {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    sk_checksum_hash32(&bytes, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_transfer_function_classification() {
        assert_eq!(
            skcms_transfer_function_get_type(skcms_srgb_transfer_function()),
            SkcmsTfType::Srgbish
        );
        let mut pq = SkcmsTransferFunction::default();
        skcms_transfer_function_make_pq(&mut pq, 203.0);
        assert!(skcms_transfer_function_is_pq(&pq));
        assert_eq!(skcms_transfer_function_eval(&pq, 1.0), 1.0);
    }

    #[test]
    fn dev_matrix_invert_and_concat() {
        let m = SkcmsMatrix3x3 {
            vals: [[2.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 0.5]],
        };
        let mut inv = SkcmsMatrix3x3::default();
        assert!(skcms_matrix3x3_invert(&m, &mut inv));
        let identity = skcms_matrix3x3_concat(&m, &inv);
        assert!((identity.vals[0][0] - 1.0).abs() < 1e-6);
        assert!((identity.vals[1][1] - 1.0).abs() < 1e-6);
        assert!((identity.vals[2][2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dev_primaries_to_xyzd50_srgb() {
        let mut m = SkcmsMatrix3x3::default();
        assert!(skcms_primaries_to_xyzd50(
            0.64, 0.33, 0.30, 0.60, 0.15, 0.06, 0.3127, 0.329, &mut m
        ));
        let srgb = skcms_srgb_profile().to_xyzd50;
        for r in 0..3 {
            for c in 0..3 {
                assert!((m.vals[r][c] - srgb.vals[r][c]).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn dev_srgb_profile_transform_roundtrip() {
        let mut profile = SkcmsIccProfile::init();
        profile.set_transfer_function(skcms_srgb_transfer_function());
        profile.set_xyzd50(&skcms_srgb_profile().to_xyzd50);
        assert!(skcms_approximately_equal_profiles(
            &profile,
            skcms_srgb_profile()
        ));
    }
}
