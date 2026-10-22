//! Port of `include/private/chromium/SkPMColor.h`.
//!
//! The implementations live with the rest of `include/core/SkColor.h` in [`crate::sk_color`];
//! this module preserves the private header's own declarations.

pub use crate::sk_color::{
    SkPMColor, sk_pm_color_get_a, sk_pm_color_get_b, sk_pm_color_get_g, sk_pm_color_get_r,
    sk_pm_color_set_argb,
};
