//! Port of `include/core/SkColorSpace.h`, `src/core/SkColorSpace.cpp` and
//! `src/core/SkColorSpacePriv.h`.
#![allow(clippy::excessive_precision, clippy::if_same_then_else)]

use std::sync::{Arc, OnceLock};

use crate::data::SkData;
use crate::skcms::{
    SkcmsIccProfile, SkcmsMatrix3x3, SkcmsTfType, SkcmsTransferFunction, hash_floats,
    skcms_approximately_equal_profiles, skcms_matrix3x3_concat, skcms_matrix3x3_invert,
    skcms_primaries_to_xyzd50, skcms_srgb_inverse_transfer_function, skcms_srgb_profile,
    skcms_transfer_function_get_type, skcms_transfer_function_invert,
    skcms_trcs_are_approximate_inverse,
};

/// `SkColorSpacePrimaries`: a gamut described by primaries and a white point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkColorSpacePrimaries {
    pub f_rx: f32,
    pub f_ry: f32,
    pub f_gx: f32,
    pub f_gy: f32,
    pub f_bx: f32,
    pub f_by: f32,
    pub f_wx: f32,
    pub f_wy: f32,
}

impl SkColorSpacePrimaries {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        rx: f32,
        ry: f32,
        gx: f32,
        gy: f32,
        bx: f32,
        by: f32,
        wx: f32,
        wy: f32,
    ) -> Self {
        Self {
            f_rx: rx,
            f_ry: ry,
            f_gx: gx,
            f_gy: gy,
            f_bx: bx,
            f_by: by,
            f_wx: wx,
            f_wy: wy,
        }
    }

    /// `SkColorSpacePrimaries::toXYZD50`.
    pub fn to_xyzd50(&self) -> Option<SkcmsMatrix3x3> {
        let mut m = SkcmsMatrix3x3::default();
        if skcms_primaries_to_xyzd50(
            self.f_rx, self.f_ry, self.f_gx, self.f_gy, self.f_bx, self.f_by, self.f_wx, self.f_wy,
            &mut m,
        ) {
            Some(m)
        } else {
            None
        }
    }
}

/// `namespace SkNamedPrimaries`.
pub mod sk_named_primaries {
    use super::{SkColorSpacePrimaries, SkcmsMatrix3x3};

    /// ITU-T H.273 table 2 color primaries ids.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum CicpId {
        Rec709 = 1,
        Rec470SystemM = 4,
        Rec470SystemBG = 5,
        Rec601 = 6,
        SmpteSt240 = 7,
        GenericFilm = 8,
        Rec2020 = 9,
        SmpteSt428_1 = 10,
        SmpteRp431_2 = 11,
        SmpteEg432_1 = 12,
        ItuTH273Value22 = 22,
    }

    impl CicpId {
        pub const fn from_u8(value: u8) -> Option<CicpId> {
            Some(match value {
                1 => CicpId::Rec709,
                4 => CicpId::Rec470SystemM,
                5 => CicpId::Rec470SystemBG,
                6 => CicpId::Rec601,
                7 => CicpId::SmpteSt240,
                8 => CicpId::GenericFilm,
                9 => CicpId::Rec2020,
                10 => CicpId::SmpteSt428_1,
                11 => CicpId::SmpteRp431_2,
                12 => CicpId::SmpteEg432_1,
                22 => CicpId::ItuTH273Value22,
                _ => return None,
            })
        }

        pub const fn to_u8(self) -> u8 {
            self as u8
        }
    }

    pub const K_REC709: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.64, 0.33, 0.3, 0.6, 0.15, 0.06, 0.3127, 0.329);
    pub const K_REC470_SYSTEM_M: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.67, 0.33, 0.21, 0.71, 0.14, 0.08, 0.31, 0.316);
    pub const K_REC470_SYSTEM_BG: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.64, 0.33, 0.29, 0.60, 0.15, 0.06, 0.3127, 0.3290);
    pub const K_REC601: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.630, 0.340, 0.310, 0.595, 0.155, 0.070, 0.3127, 0.3290);
    pub const K_SMPTE_ST_240: SkColorSpacePrimaries = K_REC601;
    pub const K_GENERIC_FILM: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.681, 0.319, 0.243, 0.692, 0.145, 0.049, 0.310, 0.316);
    pub const K_REC2020: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.708, 0.292, 0.170, 0.797, 0.131, 0.046, 0.3127, 0.3290);
    pub const K_SMPTE_ST_428_1: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0 / 3.0, 1.0 / 3.0);
    pub const K_SMPTE_RP_431_2: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.680, 0.320, 0.265, 0.690, 0.150, 0.060, 0.314, 0.351);
    pub const K_SMPTE_EG_432_1: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.680, 0.320, 0.265, 0.690, 0.150, 0.060, 0.3127, 0.3290);
    pub const K_ITU_T_H273_VALUE22: SkColorSpacePrimaries =
        SkColorSpacePrimaries::new(0.630, 0.340, 0.295, 0.605, 0.155, 0.077, 0.3127, 0.3290);
    pub const K_PRO_PHOTO_RGB: SkColorSpacePrimaries = SkColorSpacePrimaries::new(
        0.7347, 0.2653, 0.1596, 0.8404, 0.0366, 0.0001, 0.34567, 0.35850,
    );

    pub const K_CICP_ID_APPLICATION_DEFINED: u8 = 2;

    struct TableEntry {
        cicp_id: CicpId,
        sk_primaries: SkColorSpacePrimaries,
        to_xyzd50: Option<SkcmsMatrix3x3>,
    }

    fn table() -> Vec<TableEntry> {
        vec![
            TableEntry {
                cicp_id: CicpId::Rec709,
                sk_primaries: K_REC709,
                to_xyzd50: Some(super::sk_named_gamut::K_SRGB),
            },
            TableEntry {
                cicp_id: CicpId::Rec470SystemM,
                sk_primaries: K_REC470_SYSTEM_M,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::Rec470SystemBG,
                sk_primaries: K_REC470_SYSTEM_BG,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::Rec601,
                sk_primaries: K_REC601,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::SmpteSt240,
                sk_primaries: K_SMPTE_ST_240,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::GenericFilm,
                sk_primaries: K_GENERIC_FILM,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::Rec2020,
                sk_primaries: K_REC2020,
                to_xyzd50: Some(super::sk_named_gamut::K_REC2020),
            },
            TableEntry {
                cicp_id: CicpId::SmpteSt428_1,
                sk_primaries: K_SMPTE_ST_428_1,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::SmpteRp431_2,
                sk_primaries: K_SMPTE_RP_431_2,
                to_xyzd50: None,
            },
            TableEntry {
                cicp_id: CicpId::SmpteEg432_1,
                sk_primaries: K_SMPTE_EG_432_1,
                to_xyzd50: Some(super::sk_named_gamut::K_DISPLAY_P3),
            },
            TableEntry {
                cicp_id: CicpId::ItuTH273Value22,
                sk_primaries: K_ITU_T_H273_VALUE22,
                to_xyzd50: None,
            },
        ]
    }

    /// `SkNamedPrimaries::GetCicp`.
    pub fn get_cicp(primaries: CicpId, to_xyzd50: &mut SkcmsMatrix3x3) -> bool {
        for entry in table() {
            if primaries != entry.cicp_id {
                continue;
            }
            if let Some(m) = entry.to_xyzd50 {
                *to_xyzd50 = m;
                return true;
            }
            if let Some(m) = entry.sk_primaries.to_xyzd50() {
                *to_xyzd50 = m;
                return true;
            }
        }
        false
    }

    /// `SkNamedPrimaries::GetCicpFromMatrix`.
    pub fn get_cicp_from_matrix(m: &SkcmsMatrix3x3, primaries: &mut CicpId) -> bool {
        for entry in table() {
            if let Some(entry_m) = entry.sk_primaries.to_xyzd50() {
                if super::xyz_almost_equal(m, &entry_m) {
                    *primaries = entry.cicp_id;
                    return true;
                }
            }
        }
        false
    }
}

/// `namespace SkNamedTransferFn`.
pub mod sk_named_transfer_fn {
    use crate::skcms::SkcmsTransferFunction;

    /// ITU-T H.273 table 3 transfer characteristics ids.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum CicpId {
        Rec709 = 1,
        Rec470SystemM = 4,
        Rec470SystemBG = 5,
        Rec601 = 6,
        SmpteSt240 = 7,
        Linear = 8,
        Iec61966_2_4 = 11,
        Iec61966_2_1 = 13,
        Rec2020_10Bit = 14,
        Rec2020_12Bit = 15,
        Pq = 16,
        SmpteSt428_1 = 17,
        Hlg = 18,
    }

    impl CicpId {
        /// `kSRGB == kIEC61966_2_1` in the reference; Rust enum variants cannot alias, so the
        /// equivalent name is an associated constant.
        pub const K_SRGB: CicpId = CicpId::Iec61966_2_1;

        pub const fn from_u8(value: u8) -> Option<CicpId> {
            Some(match value {
                1 => CicpId::Rec709,
                4 => CicpId::Rec470SystemM,
                5 => CicpId::Rec470SystemBG,
                6 => CicpId::Rec601,
                7 => CicpId::SmpteSt240,
                8 => CicpId::Linear,
                11 => CicpId::Iec61966_2_4,
                13 => CicpId::Iec61966_2_1,
                14 => CicpId::Rec2020_10Bit,
                15 => CicpId::Rec2020_12Bit,
                16 => CicpId::Pq,
                17 => CicpId::SmpteSt428_1,
                18 => CicpId::Hlg,
                _ => return None,
            })
        }

        pub const fn to_u8(self) -> u8 {
            self as u8
        }
    }

    pub const K_SRGB: SkcmsTransferFunction = SkcmsTransferFunction::new(
        2.4,
        1.0 / 1.055,
        0.055 / 1.055,
        1.0 / 12.92,
        0.04045,
        0.0,
        0.0,
    );
    pub const K_2DOT2: SkcmsTransferFunction =
        SkcmsTransferFunction::new(2.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_REC2020: SkcmsTransferFunction = SkcmsTransferFunction::new(
        2.22222,
        0.909_672,
        0.090_327_6,
        0.222_222,
        0.081_242_9,
        0.0,
        0.0,
    );

    pub const K_REC709: SkcmsTransferFunction =
        SkcmsTransferFunction::new(2.4, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_REC470_SYSTEM_M: SkcmsTransferFunction =
        SkcmsTransferFunction::new(2.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_REC470_SYSTEM_BG: SkcmsTransferFunction =
        SkcmsTransferFunction::new(2.8, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_REC601: SkcmsTransferFunction = K_REC709;
    pub const K_SMPTE_ST_240: SkcmsTransferFunction = SkcmsTransferFunction::new(
        2.222_222_222_222,
        0.899_626_676_224,
        0.100_373_323_776,
        0.25,
        0.091_286_342_118,
        0.0,
        0.0,
    );
    pub const K_LINEAR: SkcmsTransferFunction =
        SkcmsTransferFunction::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_IEC61966_2_4: SkcmsTransferFunction = K_REC709;
    pub const K_IEC61966_2_1: SkcmsTransferFunction = K_SRGB;
    pub const K_REC2020_10BIT: SkcmsTransferFunction = K_REC709;
    pub const K_REC2020_12BIT: SkcmsTransferFunction = K_REC709;
    pub const K_PQ: SkcmsTransferFunction =
        SkcmsTransferFunction::new(-5.0, 203.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_SMPTE_ST_428_1: SkcmsTransferFunction =
        SkcmsTransferFunction::new(2.6, 1.034_080_527_699, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_HLG: SkcmsTransferFunction =
        SkcmsTransferFunction::new(-6.0, 203.0, 1000.0, 1.2, 0.0, 0.0, 0.0);
    pub const K_PRO_PHOTO_RGB: SkcmsTransferFunction =
        SkcmsTransferFunction::new(1.8, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    pub const K_A98RGB: SkcmsTransferFunction = K_2DOT2;

    pub const K_CICP_ID_APPLICATION_DEFINED: u8 = 2;

    /// `SkNamedTransferFn::GetCicp`.
    pub fn get_cicp(transfer_characteristics: CicpId, trfn: &mut SkcmsTransferFunction) -> bool {
        let table: [(CicpId, SkcmsTransferFunction); 13] = [
            (CicpId::Rec709, K_REC709),
            (CicpId::Rec470SystemM, K_REC470_SYSTEM_M),
            (CicpId::Rec470SystemBG, K_REC470_SYSTEM_BG),
            (CicpId::Rec601, K_REC601),
            (CicpId::SmpteSt240, K_SMPTE_ST_240),
            (CicpId::Linear, K_LINEAR),
            (CicpId::Iec61966_2_4, K_IEC61966_2_4),
            (CicpId::Iec61966_2_1, K_IEC61966_2_1),
            (CicpId::Rec2020_10Bit, K_REC2020_10BIT),
            (CicpId::Rec2020_12Bit, K_REC2020_12BIT),
            (CicpId::Pq, K_PQ),
            (CicpId::SmpteSt428_1, K_SMPTE_ST_428_1),
            (CicpId::Hlg, K_HLG),
        ];
        for &(id, tf) in &table {
            if transfer_characteristics == id {
                *trfn = tf;
                return true;
            }
        }
        false
    }
}

/// `namespace SkNamedGamut`.
pub mod sk_named_gamut {
    use crate::skcms::SkcmsMatrix3x3;

    const fn fixed_to_float(value: i32) -> f32 {
        value as f32 / 65536.0
    }

    pub const K_SRGB: SkcmsMatrix3x3 = SkcmsMatrix3x3 {
        vals: [
            [
                fixed_to_float(0x6FA2),
                fixed_to_float(0x6299),
                fixed_to_float(0x24A0),
            ],
            [
                fixed_to_float(0x38F5),
                fixed_to_float(0xB785),
                fixed_to_float(0x0F84),
            ],
            [
                fixed_to_float(0x0390),
                fixed_to_float(0x18DA),
                fixed_to_float(0xB6CF),
            ],
        ],
    };

    pub const K_ADOBE_RGB: SkcmsMatrix3x3 = SkcmsMatrix3x3 {
        vals: [
            [
                fixed_to_float(0x9C18),
                fixed_to_float(0x348D),
                fixed_to_float(0x2631),
            ],
            [
                fixed_to_float(0x4FA5),
                fixed_to_float(0xA02C),
                fixed_to_float(0x102F),
            ],
            [
                fixed_to_float(0x04FC),
                fixed_to_float(0x0F95),
                fixed_to_float(0xBE9C),
            ],
        ],
    };

    pub const K_DISPLAY_P3: SkcmsMatrix3x3 = SkcmsMatrix3x3 {
        vals: [
            [0.515_102, 0.291_965, 0.157_153],
            [0.241_182, 0.692_236, 0.066_581_9],
            [-0.001_049_41, 0.041_881_8, 0.784_378],
        ],
    };

    pub const K_REC2020: SkcmsMatrix3x3 = SkcmsMatrix3x3 {
        vals: [
            [0.673_459, 0.165_661, 0.125_100],
            [0.279_033, 0.675_338, 0.045_628_8],
            [-0.001_931_39, 0.029_979_4, 0.797_162],
        ],
    };

    pub const K_XYZ: SkcmsMatrix3x3 = SkcmsMatrix3x3 {
        vals: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
}

fn xyz_almost_equal(a: &SkcmsMatrix3x3, b: &SkcmsMatrix3x3) -> bool {
    for r in 0..3 {
        for c in 0..3 {
            if !color_space_almost_equal(a.vals[r][c], b.vals[r][c]) {
                return false;
            }
        }
    }
    true
}

/// `color_space_almost_equal`.
pub fn color_space_almost_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

/// `transfer_fn_almost_equal`.
pub fn transfer_fn_almost_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

fn is_almost_srgb(tf: &SkcmsTransferFunction) -> bool {
    let s = sk_named_transfer_fn::K_SRGB;
    transfer_fn_almost_equal(s.a, tf.a)
        && transfer_fn_almost_equal(s.b, tf.b)
        && transfer_fn_almost_equal(s.c, tf.c)
        && transfer_fn_almost_equal(s.d, tf.d)
        && transfer_fn_almost_equal(s.e, tf.e)
        && transfer_fn_almost_equal(s.f, tf.f)
        && transfer_fn_almost_equal(s.g, tf.g)
}

fn is_almost_2dot2(tf: &SkcmsTransferFunction) -> bool {
    transfer_fn_almost_equal(1.0, tf.a)
        && transfer_fn_almost_equal(0.0, tf.b)
        && transfer_fn_almost_equal(0.0, tf.e)
        && transfer_fn_almost_equal(2.2, tf.g)
        && tf.d <= 0.0
}

fn is_almost_linear(tf: &SkcmsTransferFunction) -> bool {
    // OutputVal = InputVal ^ 1.0f
    let linear_exp = transfer_fn_almost_equal(1.0, tf.a)
        && transfer_fn_almost_equal(0.0, tf.b)
        && transfer_fn_almost_equal(0.0, tf.e)
        && transfer_fn_almost_equal(1.0, tf.g)
        && tf.d <= 0.0;

    // OutputVal = 1.0f * InputVal
    let linear_fn =
        transfer_fn_almost_equal(1.0, tf.c) && transfer_fn_almost_equal(0.0, tf.f) && tf.d >= 1.0;

    linear_exp || linear_fn
}

#[derive(Clone, Copy, Debug)]
struct LazyDstFields {
    inv_transfer_fn: SkcmsTransferFunction,
    from_xyzd50: SkcmsMatrix3x3,
}

/// `SkColorSpace`.
pub struct SkColorSpace {
    f_transfer_fn_hash: u32,
    f_to_xyzd50_hash: u32,
    f_transfer_fn: SkcmsTransferFunction,
    f_to_xyzd50: SkcmsMatrix3x3,
    f_lazy: OnceLock<LazyDstFields>,
}

impl std::fmt::Debug for SkColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkColorSpace")
            .field("transfer_fn", &self.f_transfer_fn)
            .field("to_xyzd50", &self.f_to_xyzd50)
            .field("hash", &self.hash())
            .finish()
    }
}

fn hash_transfer_fn(tf: &SkcmsTransferFunction) -> u32 {
    let values = [tf.g, tf.a, tf.b, tf.c, tf.d, tf.e, tf.f];
    hash_floats(&values)
}

fn hash_matrix(m: &SkcmsMatrix3x3) -> u32 {
    let mut values = [0.0f32; 9];
    for r in 0..3 {
        for c in 0..3 {
            values[r * 3 + c] = m.vals[r][c];
        }
    }
    hash_floats(&values)
}

impl SkColorSpace {
    fn new(transfer_fn: SkcmsTransferFunction, to_xyzd50: SkcmsMatrix3x3) -> Self {
        Self {
            f_transfer_fn_hash: hash_transfer_fn(&transfer_fn),
            f_to_xyzd50_hash: hash_matrix(&to_xyzd50),
            f_transfer_fn: transfer_fn,
            f_to_xyzd50: to_xyzd50,
            f_lazy: OnceLock::new(),
        }
    }

    /// `SkColorSpace::MakeSRGB`.
    pub fn make_srgb() -> Arc<SkColorSpace> {
        static SRGB: OnceLock<Arc<SkColorSpace>> = OnceLock::new();
        SRGB.get_or_init(|| {
            Arc::new(SkColorSpace::new(
                sk_named_transfer_fn::K_SRGB,
                sk_named_gamut::K_SRGB,
            ))
        })
        .clone()
    }

    /// `SkColorSpace::MakeSRGBLinear`.
    pub fn make_srgb_linear() -> Arc<SkColorSpace> {
        static SRGB_LINEAR: OnceLock<Arc<SkColorSpace>> = OnceLock::new();
        SRGB_LINEAR
            .get_or_init(|| {
                Arc::new(SkColorSpace::new(
                    sk_named_transfer_fn::K_LINEAR,
                    sk_named_gamut::K_SRGB,
                ))
            })
            .clone()
    }

    /// `SkColorSpace::MakeRGB`.
    pub fn make_rgb(
        transfer_fn: &SkcmsTransferFunction,
        to_xyz: &SkcmsMatrix3x3,
    ) -> Option<Arc<SkColorSpace>> {
        if skcms_transfer_function_get_type(transfer_fn) == SkcmsTfType::Invalid {
            return None;
        }

        let mut tf = *transfer_fn;

        if is_almost_srgb(transfer_fn) {
            if xyz_almost_equal(to_xyz, &sk_named_gamut::K_SRGB) {
                return Some(SkColorSpace::make_srgb());
            }
            tf = sk_named_transfer_fn::K_SRGB;
        } else if is_almost_2dot2(transfer_fn) {
            tf = sk_named_transfer_fn::K_2DOT2;
        } else if is_almost_linear(transfer_fn) {
            if xyz_almost_equal(to_xyz, &sk_named_gamut::K_SRGB) {
                return Some(SkColorSpace::make_srgb_linear());
            }
            tf = sk_named_transfer_fn::K_LINEAR;
        }

        Some(Arc::new(SkColorSpace::new(tf, *to_xyz)))
    }

    /// `SkColorSpace::MakeCICP`.
    pub fn make_cicp(
        color_primaries: sk_named_primaries::CicpId,
        transfer_characteristics: sk_named_transfer_fn::CicpId,
    ) -> Option<Arc<SkColorSpace>> {
        let mut trfn = SkcmsTransferFunction::default();
        if !sk_named_transfer_fn::get_cicp(transfer_characteristics, &mut trfn) {
            return None;
        }

        let mut to_xyzd50 = SkcmsMatrix3x3::default();
        if !sk_named_primaries::get_cicp(color_primaries, &mut to_xyzd50) {
            return None;
        }

        SkColorSpace::make_rgb(&trfn, &to_xyzd50)
    }

    /// `SkColorSpace::Make(const skcms_ICCProfile&)`.
    pub fn make(profile: &SkcmsIccProfile) -> Option<Arc<SkColorSpace>> {
        // The CICP values are only valid for full-range, with no matrix.
        let use_cicp = profile.has_cicp
            && profile.cicp.matrix_coefficients == 0
            && profile.cicp.video_full_range_flag == 1;
        let cicp_color_primaries = profile.cicp.color_primaries;
        let cicp_transfer_characteristics = profile.cicp.transfer_characteristics;

        // Early checks for exact sRGB matches.
        if use_cicp
            && cicp_color_primaries == sk_named_primaries::CicpId::Rec709.to_u8()
            && cicp_transfer_characteristics == sk_named_transfer_fn::CicpId::Iec61966_2_4.to_u8()
        {
            return Some(SkColorSpace::make_srgb());
        } else if skcms_approximately_equal_profiles(profile, skcms_srgb_profile()) {
            return Some(SkColorSpace::make_srgb());
        }

        // Set the toXYZD50 matrix, preferring CICP over the matrix itself.
        let mut to_xyzd50 = SkcmsMatrix3x3::default();
        let mut has_set_to_xyzd50 = false;
        if use_cicp {
            if let Some(primaries) = sk_named_primaries::CicpId::from_u8(cicp_color_primaries) {
                if sk_named_primaries::get_cicp(primaries, &mut to_xyzd50) {
                    has_set_to_xyzd50 = true;
                }
            }
            if !has_set_to_xyzd50
                && cicp_color_primaries != sk_named_primaries::K_CICP_ID_APPLICATION_DEFINED
            {
                return None;
            }
        }
        if profile.has_to_xyzd50 && !has_set_to_xyzd50 {
            let mut inv = SkcmsMatrix3x3::default();
            to_xyzd50 = profile.to_xyzd50;
            if skcms_matrix3x3_invert(&to_xyzd50, &mut inv) {
                has_set_to_xyzd50 = true;
            }
        }
        if !has_set_to_xyzd50 {
            return None;
        }

        // Set the transfer function, preferring CICP over the curves.
        let mut trfn = SkcmsTransferFunction::default();
        let mut has_set_trfn = false;
        if use_cicp {
            if let Some(characteristics) =
                sk_named_transfer_fn::CicpId::from_u8(cicp_transfer_characteristics)
            {
                if sk_named_transfer_fn::get_cicp(characteristics, &mut trfn) {
                    has_set_trfn = true;
                }
            }
            if !has_set_trfn
                && cicp_transfer_characteristics
                    != sk_named_transfer_fn::K_CICP_ID_APPLICATION_DEFINED
            {
                return None;
            }
        }
        if profile.has_trc && !has_set_trfn {
            // We can't work with tables or mismatched parametric curves.
            let trc = &profile.trc;
            if trc[0].table_entries == 0
                && trc[1].table_entries == 0
                && trc[2].table_entries == 0
                && trc[0].parametric.bits_eq(&trc[1].parametric)
                && trc[0].parametric.bits_eq(&trc[2].parametric)
            {
                trfn = trc[0].parametric;
                has_set_trfn = true;
            } else if skcms_trcs_are_approximate_inverse(
                profile,
                skcms_srgb_inverse_transfer_function(),
            ) {
                trfn = sk_named_transfer_fn::K_SRGB;
                has_set_trfn = true;
            }
        }
        if !has_set_trfn {
            return None;
        }

        SkColorSpace::make_rgb(&trfn, &to_xyzd50)
    }

    fn compute_lazy_dst_fields(&self) -> &LazyDstFields {
        self.f_lazy.get_or_init(|| {
            // Invert 3x3 gamut, defaulting to sRGB if we can't.
            let mut from_xyzd50 = SkcmsMatrix3x3::default();
            if !skcms_matrix3x3_invert(&self.f_to_xyzd50, &mut from_xyzd50) {
                skcms_matrix3x3_invert(&skcms_srgb_profile().to_xyzd50, &mut from_xyzd50);
            }

            // Invert transfer function, defaulting to sRGB if we can't.
            let mut inv_transfer_fn = SkcmsTransferFunction::default();
            if !skcms_transfer_function_invert(&self.f_transfer_fn, &mut inv_transfer_fn) {
                inv_transfer_fn = *skcms_srgb_inverse_transfer_function();
            }

            LazyDstFields {
                inv_transfer_fn,
                from_xyzd50,
            }
        })
    }

    /// `toProfile(skcms_ICCProfile*)`.
    pub fn to_profile(&self, profile: &mut SkcmsIccProfile) {
        *profile = SkcmsIccProfile::init();
        profile.set_transfer_function(&self.f_transfer_fn);
        profile.set_xyzd50(&self.f_to_xyzd50);

        match skcms_transfer_function_get_type(&self.f_transfer_fn) {
            SkcmsTfType::Pq | SkcmsTfType::Pqish => {
                profile.has_cicp = true;
                profile.cicp.transfer_characteristics = sk_named_transfer_fn::CicpId::Pq.to_u8();
            }
            SkcmsTfType::Hlg | SkcmsTfType::Hlgish => {
                profile.has_cicp = true;
                profile.cicp.transfer_characteristics = sk_named_transfer_fn::CicpId::Hlg.to_u8();
            }
            _ => {}
        }
        if profile.has_cicp {
            profile.cicp.matrix_coefficients = 0;
            profile.cicp.video_full_range_flag = 1;
            let mut primaries_id = sk_named_primaries::CicpId::Rec709;
            if sk_named_primaries::get_cicp_from_matrix(&self.f_to_xyzd50, &mut primaries_id) {
                profile.cicp.color_primaries = primaries_id.to_u8();
            } else {
                profile.cicp.color_primaries = sk_named_primaries::K_CICP_ID_APPLICATION_DEFINED;
            }
        }
    }

    /// `gammaCloseToSRGB`.
    pub fn gamma_close_to_srgb(&self) -> bool {
        // Nearly-equal transfer functions were snapped at construction time.
        self.f_transfer_fn.bits_eq(&sk_named_transfer_fn::K_SRGB)
    }

    /// `gammaIsLinear`.
    pub fn gamma_is_linear(&self) -> bool {
        self.f_transfer_fn.bits_eq(&sk_named_transfer_fn::K_LINEAR)
    }

    /// `isNumericalTransferFn`.
    pub fn is_numerical_transfer_fn(&self) -> Option<SkcmsTransferFunction> {
        let tf = self.f_transfer_fn;
        if skcms_transfer_function_get_type(&tf) == SkcmsTfType::Srgbish {
            Some(tf)
        } else {
            None
        }
    }

    /// `toXYZD50`.
    pub fn to_xyzd50(&self) -> SkcmsMatrix3x3 {
        self.f_to_xyzd50
    }

    /// `toXYZD50Hash`.
    pub fn to_xyzd50_hash(&self) -> u32 {
        self.f_to_xyzd50_hash
    }

    /// `makeLinearGamma`.
    pub fn make_linear_gamma(self: &Arc<Self>) -> Arc<SkColorSpace> {
        if self.gamma_is_linear() {
            return self.clone();
        }
        SkColorSpace::make_rgb(&sk_named_transfer_fn::K_LINEAR, &self.f_to_xyzd50)
            .expect("linear transfer function is always valid")
    }

    /// `makeSRGBGamma`.
    pub fn make_srgb_gamma(self: &Arc<Self>) -> Arc<SkColorSpace> {
        if self.gamma_close_to_srgb() {
            return self.clone();
        }
        SkColorSpace::make_rgb(&sk_named_transfer_fn::K_SRGB, &self.f_to_xyzd50)
            .expect("sRGB transfer function is always valid")
    }

    /// `makeColorSpin`.
    pub fn make_color_spin(self: &Arc<Self>) -> Arc<SkColorSpace> {
        let spin = SkcmsMatrix3x3 {
            vals: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        };
        let spun = skcms_matrix3x3_concat(&self.f_to_xyzd50, &spin);
        Arc::new(SkColorSpace::new(self.f_transfer_fn, spun))
    }

    /// `isSRGB`.
    pub fn is_srgb(&self) -> bool {
        std::ptr::eq(self, SkColorSpace::make_srgb().as_ref())
    }

    /// `writeToMemory`: `nullptr` (here `None`) returns the required size.
    pub fn write_to_memory(&self, memory: Option<&mut [u8]>) -> usize {
        const SIZE: usize = 4 + 16 * 4;
        if let Some(memory) = memory {
            let memory = &mut memory[..SIZE];
            memory[0] = 1; // k1_Version
            memory[1] = 0;
            memory[2] = 0;
            memory[3] = 0;
            let tf = [
                self.f_transfer_fn.g,
                self.f_transfer_fn.a,
                self.f_transfer_fn.b,
                self.f_transfer_fn.c,
                self.f_transfer_fn.d,
                self.f_transfer_fn.e,
                self.f_transfer_fn.f,
            ];
            let mut offset = 4;
            for value in tf {
                memory[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
                offset += 4;
            }
            for r in 0..3 {
                for c in 0..3 {
                    memory[offset..offset + 4]
                        .copy_from_slice(&self.f_to_xyzd50.vals[r][c].to_ne_bytes());
                    offset += 4;
                }
            }
        }
        SIZE
    }

    /// `serialize`.
    pub fn serialize(&self) -> SkData {
        let mut bytes = vec![0u8; self.write_to_memory(None)];
        self.write_to_memory(Some(&mut bytes));
        SkData::make_with_copy(&bytes)
    }

    /// `SkColorSpace::Deserialize`.
    pub fn deserialize(data: &[u8]) -> Option<Arc<SkColorSpace>> {
        const HEADER: usize = 4;
        if data.len() < HEADER {
            return None;
        }
        if data[0] != 1 {
            return None;
        }
        if data.len() < HEADER + 16 * 4 {
            return None;
        }

        let read_f32 = |offset: usize| -> f32 {
            f32::from_ne_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        };

        let mut offset = HEADER;
        let mut tf = [0.0f32; 7];
        for value in &mut tf {
            *value = read_f32(offset);
            offset += 4;
        }
        let transfer_fn =
            SkcmsTransferFunction::new(tf[0], tf[1], tf[2], tf[3], tf[4], tf[5], tf[6]);

        let mut to_xyz = SkcmsMatrix3x3::default();
        for r in 0..3 {
            for c in 0..3 {
                to_xyz.vals[r][c] = read_f32(offset);
                offset += 4;
            }
        }

        SkColorSpace::make_rgb(&transfer_fn, &to_xyz)
    }

    /// `SkColorSpace::Equals`.
    pub fn equals(a: Option<&SkColorSpace>, b: Option<&SkColorSpace>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                if std::ptr::eq(a, b) {
                    return true;
                }
                a.hash() == b.hash()
            }
            _ => false,
        }
    }

    /// `transferFn(float gabcdef[7])`.
    pub fn transfer_fn_array(&self) -> [f32; 7] {
        let tf = self.f_transfer_fn;
        [tf.g, tf.a, tf.b, tf.c, tf.d, tf.e, tf.f]
    }

    /// `transferFn(skcms_TransferFunction*)`.
    pub fn transfer_fn(&self) -> SkcmsTransferFunction {
        self.f_transfer_fn
    }

    /// `invTransferFn`.
    pub fn inv_transfer_fn(&self) -> SkcmsTransferFunction {
        self.compute_lazy_dst_fields().inv_transfer_fn
    }

    /// `gamutTransformTo`.
    pub fn gamut_transform_to(&self, dst: &SkColorSpace) -> SkcmsMatrix3x3 {
        let dst_lazy = dst.compute_lazy_dst_fields();
        skcms_matrix3x3_concat(&dst_lazy.from_xyzd50, &self.f_to_xyzd50)
    }

    /// `transferFnHash`.
    pub fn transfer_fn_hash(&self) -> u32 {
        self.f_transfer_fn_hash
    }

    /// `hash`.
    pub fn hash(&self) -> u64 {
        (self.f_transfer_fn_hash as u64) << 32 | self.f_to_xyzd50_hash as u64
    }
}

impl PartialEq for SkColorSpace {
    fn eq(&self, other: &Self) -> bool {
        SkColorSpace::equals(Some(self), Some(other))
    }
}

/// `sk_srgb_singleton`.
pub fn sk_srgb_singleton() -> Arc<SkColorSpace> {
    SkColorSpace::make_srgb()
}

/// `sk_srgb_linear_singleton`.
pub fn sk_srgb_linear_singleton() -> Arc<SkColorSpace> {
    SkColorSpace::make_srgb_linear()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_srgb_singleton_matches_make_rgb() {
        let srgb = SkColorSpace::make_srgb();
        assert!(srgb.is_srgb());
        let made =
            SkColorSpace::make_rgb(&sk_named_transfer_fn::K_SRGB, &sk_named_gamut::K_SRGB).unwrap();
        assert!(made.is_srgb());
        assert!(SkColorSpace::equals(Some(&srgb), Some(&made)));
        assert!(srgb.gamma_close_to_srgb());
        assert!(!srgb.gamma_is_linear());
    }

    #[test]
    fn dev_serialize_roundtrip() {
        let cs = SkColorSpace::make_rgb(
            &sk_named_transfer_fn::K_2DOT2,
            &sk_named_gamut::K_DISPLAY_P3,
        )
        .unwrap();
        let data = cs.serialize();
        let restored = SkColorSpace::deserialize(data.bytes()).unwrap();
        assert!(SkColorSpace::equals(Some(&cs), Some(&restored)));
        assert!(SkColorSpace::deserialize(&[1, 0, 0, 0]).is_none());
        assert!(SkColorSpace::deserialize(&[2, 0, 0, 0]).is_none());
    }

    #[test]
    fn dev_cicp_and_profile_roundtrip() {
        let cs = SkColorSpace::make_cicp(
            sk_named_primaries::CicpId::Rec2020,
            sk_named_transfer_fn::CicpId::Pq,
        )
        .unwrap();
        assert!(!cs.gamma_is_linear());
        let mut profile = SkcmsIccProfile::default();
        cs.to_profile(&mut profile);
        assert!(profile.has_cicp);
        assert_eq!(profile.cicp.transfer_characteristics, 16);
        assert_eq!(profile.cicp.color_primaries, 9);
        assert_eq!(profile.data_color_space, crate::skcms::SKCMS_SIGNATURE_RGB);
        assert_eq!(profile.pcs, crate::skcms::SKCMS_SIGNATURE_XYZ);
        let from_profile = SkColorSpace::make(&profile).unwrap();
        assert!(SkColorSpace::equals(Some(&cs), Some(&from_profile)));
    }

    #[test]
    fn dev_gamut_transform_srgb_to_p3() {
        let srgb = SkColorSpace::make_srgb();
        let p3 =
            SkColorSpace::make_rgb(&sk_named_transfer_fn::K_SRGB, &sk_named_gamut::K_DISPLAY_P3)
                .unwrap();
        let m = srgb.gamut_transform_to(&p3);
        assert!(m.vals.iter().flatten().all(|v| v.is_finite()));
        assert_ne!(m, sk_named_gamut::K_XYZ);
    }
}
