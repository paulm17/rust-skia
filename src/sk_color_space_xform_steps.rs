//! Port of `src/core/SkColorSpaceXformSteps.h` and `src/core/SkColorSpaceXformSteps.cpp`.
#![allow(clippy::excessive_precision)]

use crate::floating_point;
use crate::sk_alpha_type::SkAlphaType;
use crate::sk_color_space::{
    SkColorSpace, sk_named_gamut, sk_named_transfer_fn, sk_srgb_singleton,
};
use crate::skcms::{
    SkcmsTfType, SkcmsTransferFunction, skcms_transfer_function_eval,
    skcms_transfer_function_get_type, skcms_transfer_function_invert,
};

/// `SkColorSpaceXformSteps::Flags`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub unpremul: bool,
    pub linearize: bool,
    pub src_ootf: bool,
    pub gamut_transform: bool,
    pub dst_ootf: bool,
    pub encode: bool,
    pub premul: bool,
}

impl Flags {
    pub const fn mask(&self) -> u32 {
        (if self.unpremul { 1 } else { 0 })
            | (if self.linearize { 2 } else { 0 })
            | (if self.src_ootf { 32 } else { 0 })
            | (if self.gamut_transform { 4 } else { 0 })
            | (if self.dst_ootf { 64 } else { 0 })
            | (if self.encode { 8 } else { 0 })
            | (if self.premul { 16 } else { 0 })
    }
}

/// `SkColorSpaceXformSteps`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SkColorSpaceXformSteps {
    pub f_flags: Flags,
    /// Applied for `linearize`.
    pub f_src_tf: SkcmsTransferFunction,
    /// Applied for `encode`.
    pub f_dst_tf_inv: SkcmsTransferFunction,
    /// A 3x3 *column*-major matrix, applied for `gamut_transform`.
    pub f_src_to_dst_matrix: [f32; 9],
    /// r,g,b coefficients and gamma before `gamut_transform`.
    pub f_src_ootf: [f32; 4],
    /// r,g,b coefficients and gamma after `gamut_transform`.
    pub f_dst_ootf: [f32; 4],
}

/// `set_ootf_Y`: the Y vector for the HLG OOTF in the primaries of `cs`.
fn set_ootf_y(cs: &SkColorSpace) -> [f32; 3] {
    let rec2020 =
        SkColorSpace::make_rgb(&sk_named_transfer_fn::K_LINEAR, &sk_named_gamut::K_REC2020)
            .expect("Rec2020 is a valid gamut");
    let m = cs.gamut_transform_to(&rec2020);
    const Y_REC2020: [f32; 3] = [0.262_700, 0.678_000, 0.059_300];
    let mut y = [0.0f32; 3];
    for (i, slot) in y.iter_mut().enumerate() {
        for (j, &coefficient) in Y_REC2020.iter().enumerate() {
            *slot += m.vals[j][i] * coefficient;
        }
    }
    y
}

/// `SkRGBA4f` friendship isn't needed; `SkAlphaType` values drive the alpha steps.
impl SkColorSpaceXformSteps {
    /// `SkColorSpaceXformSteps(src, srcAT, dst, dstAT)`.
    pub fn new(
        src: Option<&SkColorSpace>,
        src_at: SkAlphaType,
        dst: Option<&SkColorSpace>,
        mut dst_at: SkAlphaType,
    ) -> Self {
        // Opaque outputs are treated as the same alpha type as the source input.
        if dst_at == SkAlphaType::Opaque {
            dst_at = src_at;
        }

        // This pair of defaults seems to be the most consistent with legacy expectations.
        let src_owned;
        let src = match src {
            Some(src) => src,
            None => {
                src_owned = sk_srgb_singleton();
                src_owned.as_ref()
            }
        };
        let dst = dst.unwrap_or(src);

        let mut steps = SkColorSpaceXformSteps::default();

        if src.hash() == dst.hash() && src_at == dst_at {
            debug_assert!(SkColorSpace::equals(Some(src), Some(dst)));
            return steps;
        }

        let src_trfn = src.transfer_fn();
        let dst_trfn = dst.transfer_fn();

        // The scale factor accommodates peak luminance and HDR reference white luminance.
        let mut scale_factor = 1.0f32;

        let pqish = SkcmsTransferFunction::new(
            -2.0,
            -107.0 / 128.0,
            1.0,
            32.0 / 2523.0,
            2413.0 / 128.0,
            -2392.0 / 128.0,
            8192.0 / 1305.0,
        );
        let hlgish = SkcmsTransferFunction::new(
            -3.0,
            2.0,
            2.0,
            1.0 / 0.178_832_77,
            0.284_668_92,
            0.559_910_73,
            0.0,
        );

        let src_tf_type = skcms_transfer_function_get_type(&src_trfn);
        match src_tf_type {
            SkcmsTfType::Pq => {
                // PQ is always scaled by a peak luminance of 10,000 nits, then divided by the HDR
                // reference white luminance (a).
                scale_factor *= 10000.0 / src_trfn.a;
                steps.f_src_tf = pqish;
                steps.f_flags.linearize = true;
            }
            SkcmsTfType::Hlg => {
                scale_factor *= src_trfn.b / src_trfn.a;
                steps.f_flags.linearize = true;
                // Use the HLGish transfer function scaled by 1/12.
                steps.f_src_tf = hlgish;
                steps.f_src_tf.f = 1.0 / 12.0 - 1.0;
                // If the system gamma is not 1.0, compute the parameters for the OOTF.
                if src_trfn.c != 1.0 {
                    steps.f_flags.src_ootf = true;
                    steps.f_src_ootf[3] = src_trfn.c - 1.0;
                    let y = set_ootf_y(src);
                    steps.f_src_ootf[..3].copy_from_slice(&y);
                }
            }
            _ => {
                steps.f_flags.linearize = !src_trfn.bits_eq(&sk_named_transfer_fn::K_LINEAR);
                if steps.f_flags.linearize {
                    steps.f_src_tf = src.transfer_fn();
                }
            }
        }

        let dst_tf_type = skcms_transfer_function_get_type(&dst_trfn);
        match dst_tf_type {
            SkcmsTfType::Pq => {
                // This is the inverse of the treatment of source PQ.
                scale_factor /= 10000.0 / dst_trfn.a;
                steps.f_flags.encode = true;
                let mut inv = pqish;
                let fwd = inv;
                skcms_transfer_function_invert(&fwd, &mut inv);
                steps.f_dst_tf_inv = inv;
            }
            SkcmsTfType::Hlg => {
                scale_factor /= dst_trfn.b / dst_trfn.a;
                steps.f_flags.encode = true;
                let mut inv = hlgish;
                inv.f = 1.0 / 12.0 - 1.0;
                let fwd = inv;
                skcms_transfer_function_invert(&fwd, &mut inv);
                steps.f_dst_tf_inv = inv;
                if dst_trfn.c != 1.0 {
                    steps.f_flags.dst_ootf = true;
                    steps.f_dst_ootf[3] = 1.0 / dst_trfn.c - 1.0;
                    let y = set_ootf_y(dst);
                    steps.f_dst_ootf[..3].copy_from_slice(&y);
                }
            }
            _ => {
                steps.f_flags.encode = !dst_trfn.bits_eq(&sk_named_transfer_fn::K_LINEAR);
                if steps.f_flags.encode {
                    steps.f_dst_tf_inv = dst.inv_transfer_fn();
                }
            }
        }

        steps.f_flags.unpremul = src_at == SkAlphaType::Premul;
        steps.f_flags.gamut_transform =
            src.to_xyzd50_hash() != dst.to_xyzd50_hash() || scale_factor != 1.0;
        steps.f_flags.premul = src_at != SkAlphaType::Opaque && dst_at == SkAlphaType::Premul;

        if steps.f_flags.gamut_transform {
            let src_to_dst = src.gamut_transform_to(dst);

            steps.f_src_to_dst_matrix[0] = src_to_dst.vals[0][0] * scale_factor;
            steps.f_src_to_dst_matrix[1] = src_to_dst.vals[1][0] * scale_factor;
            steps.f_src_to_dst_matrix[2] = src_to_dst.vals[2][0] * scale_factor;

            steps.f_src_to_dst_matrix[3] = src_to_dst.vals[0][1] * scale_factor;
            steps.f_src_to_dst_matrix[4] = src_to_dst.vals[1][1] * scale_factor;
            steps.f_src_to_dst_matrix[5] = src_to_dst.vals[2][1] * scale_factor;

            steps.f_src_to_dst_matrix[6] = src_to_dst.vals[0][2] * scale_factor;
            steps.f_src_to_dst_matrix[7] = src_to_dst.vals[1][2] * scale_factor;
            steps.f_src_to_dst_matrix[8] = src_to_dst.vals[2][2] * scale_factor;
        } else {
            debug_assert!(src.to_xyzd50() == dst.to_xyzd50(), "Hash collision");
        }

        // If the source and destination OOTFs cancel each other out, skip both.
        if steps.f_flags.src_ootf && !steps.f_flags.gamut_transform && steps.f_flags.dst_ootf {
            // If there is no gamut transform, then the r,g,b coefficients must be the same.
            debug_assert_eq!(
                &steps.f_src_ootf[..3],
                &steps.f_dst_ootf[..3],
                "OOTF coefficients differ without a gamut transform"
            );
            if (steps.f_src_ootf[3] + 1.0) * (steps.f_dst_ootf[3] + 1.0) == 1.0 {
                steps.f_flags.src_ootf = false;
                steps.f_flags.dst_ootf = false;
            }
        }

        // If we linearize then immediately reencode with the same transfer function, skip both.
        if steps.f_flags.linearize
            && !steps.f_flags.src_ootf
            && !steps.f_flags.gamut_transform
            && !steps.f_flags.dst_ootf
            && steps.f_flags.encode
            && src.transfer_fn_hash() == dst.transfer_fn_hash()
        {
            debug_assert!(
                src_tf_type == SkcmsTfType::Pq
                    || src_tf_type == SkcmsTfType::Hlg
                    || steps.f_src_tf.bits_eq(&dst.transfer_fn()),
                "Hash collision"
            );
            steps.f_flags.linearize = false;
            steps.f_flags.encode = false;
        }

        // Skip unpremul...premul if there are no non-linear operations between.
        if steps.f_flags.unpremul
            && !steps.f_flags.linearize
            && !steps.f_flags.encode
            && steps.f_flags.premul
        {
            steps.f_flags.unpremul = false;
            steps.f_flags.premul = false;
        }

        steps
    }

    /// `explicit operator bool()`.
    pub fn has_steps(&self) -> bool {
        self.f_flags.mask() != 0
    }

    /// `apply(float rgba[4])`.
    pub fn apply(&self, rgba: &mut [f32; 4]) {
        if self.f_flags.unpremul {
            // I don't know why isfinite(x) stopped working on the Chromecast bots...
            let is_finite = |x: f32| x * 0.0 == 0.0;

            let mut inv_a = floating_point::ieee_float_divide(1.0, rgba[3]);
            inv_a = if is_finite(inv_a) { inv_a } else { 0.0 };
            rgba[0] *= inv_a;
            rgba[1] *= inv_a;
            rgba[2] *= inv_a;
        }
        if self.f_flags.linearize {
            rgba[0] = skcms_transfer_function_eval(&self.f_src_tf, rgba[0]);
            rgba[1] = skcms_transfer_function_eval(&self.f_src_tf, rgba[1]);
            rgba[2] = skcms_transfer_function_eval(&self.f_src_tf, rgba[2]);
        }
        if self.f_flags.src_ootf {
            let y = self.f_src_ootf[0] * rgba[0]
                + self.f_src_ootf[1] * rgba[1]
                + self.f_src_ootf[2] * rgba[2];
            let y_to_gamma_minus_1 = y.powf(self.f_src_ootf[3]);
            rgba[0] *= y_to_gamma_minus_1;
            rgba[1] *= y_to_gamma_minus_1;
            rgba[2] *= y_to_gamma_minus_1;
        }
        if self.f_flags.gamut_transform {
            let temp = [rgba[0], rgba[1], rgba[2]];
            for i in 0..3 {
                rgba[i] = self.f_src_to_dst_matrix[i] * temp[0]
                    + self.f_src_to_dst_matrix[3 + i] * temp[1]
                    + self.f_src_to_dst_matrix[6 + i] * temp[2];
            }
        }
        if self.f_flags.dst_ootf {
            let y = self.f_dst_ootf[0] * rgba[0]
                + self.f_dst_ootf[1] * rgba[1]
                + self.f_dst_ootf[2] * rgba[2];
            let y_to_gamma_minus_1 = y.powf(self.f_dst_ootf[3]);
            rgba[0] *= y_to_gamma_minus_1;
            rgba[1] *= y_to_gamma_minus_1;
            rgba[2] *= y_to_gamma_minus_1;
        }
        if self.f_flags.encode {
            rgba[0] = skcms_transfer_function_eval(&self.f_dst_tf_inv, rgba[0]);
            rgba[1] = skcms_transfer_function_eval(&self.f_dst_tf_inv, rgba[1]);
            rgba[2] = skcms_transfer_function_eval(&self.f_dst_tf_inv, rgba[2]);
        }
        if self.f_flags.premul {
            rgba[0] *= rgba[3];
            rgba[1] *= rgba[3];
            rgba[2] *= rgba[3];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_same_space_has_no_steps() {
        let srgb = SkColorSpace::make_srgb();
        let steps = SkColorSpaceXformSteps::new(
            Some(&srgb),
            SkAlphaType::Unpremul,
            Some(&srgb),
            SkAlphaType::Unpremul,
        );
        assert!(!steps.has_steps());
        assert_eq!(steps.f_flags.mask(), 0);
    }

    #[test]
    fn dev_linearize_and_encode() {
        let srgb = SkColorSpace::make_srgb();
        let linear = SkColorSpace::make_srgb_linear();
        let to_linear = SkColorSpaceXformSteps::new(
            Some(&srgb),
            SkAlphaType::Unpremul,
            Some(&linear),
            SkAlphaType::Unpremul,
        );
        assert!(to_linear.f_flags.linearize);
        assert!(!to_linear.f_flags.encode);
        let mut rgba = [0.5, 0.5, 0.5, 1.0];
        to_linear.apply(&mut rgba);
        assert!((rgba[0] - 0.214_041).abs() < 1e-4);

        let back = SkColorSpaceXformSteps::new(
            Some(&linear),
            SkAlphaType::Unpremul,
            Some(&srgb),
            SkAlphaType::Unpremul,
        );
        back.apply(&mut rgba);
        assert!((rgba[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn dev_premul_opaque_skip() {
        let srgb = SkColorSpace::make_srgb();
        let linear = SkColorSpace::make_srgb_linear();
        let steps = SkColorSpaceXformSteps::new(
            Some(&srgb),
            SkAlphaType::Premul,
            Some(&linear),
            SkAlphaType::Unpremul,
        );
        assert!(steps.f_flags.unpremul);
        assert!(!steps.f_flags.premul);

        let opaque = SkColorSpaceXformSteps::new(
            Some(&srgb),
            SkAlphaType::Premul,
            Some(&linear),
            SkAlphaType::Opaque,
        );
        // dstAT becomes srcAT (Premul), and linearization sits between the unpremul/premul pair,
        // so that pair is not skipped.
        assert!(opaque.f_flags.unpremul);
        assert!(opaque.f_flags.premul);
    }
}
