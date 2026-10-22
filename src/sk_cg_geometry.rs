//! Port of `src/utils/mac/SkCGGeometry.h`.
//!
//! Inline replacements for the `CGRect` helpers that CoreGraphics exposes as functions. The
//! reference header is compiled only for `SK_BUILD_FOR_MAC`/`SK_BUILD_FOR_IOS` and includes
//! `<ApplicationServices/ApplicationServices.h>`/`<CoreGraphics/CoreGraphics.h>`.
//!
//! The `#[repr(C)]` types below are the CoreGraphics value layouts (`CGFloat` is `f64` on
//! 64-bit Apple platforms) and are FFI-compatible with the framework structures. The M39
//! platform adapters reuse these definitions rather than introducing a second copy.

/// `CGFloat` on 64-bit Apple platforms.
pub type CGFloat = f64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGPoint {
    pub x: CGFloat,
    pub y: CGFloat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGSize {
    pub width: CGFloat,
    pub height: CGFloat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

const _: () = {
    assert!(core::mem::size_of::<CGPoint>() == 2 * core::mem::size_of::<CGFloat>());
    assert!(core::mem::size_of::<CGSize>() == 2 * core::mem::size_of::<CGFloat>());
    assert!(core::mem::size_of::<CGRect>() == 4 * core::mem::size_of::<CGFloat>());
    assert!(core::mem::offset_of!(CGRect, origin) == 0);
    assert!(core::mem::offset_of!(CGRect, size) == 2 * core::mem::size_of::<CGFloat>());
};

pub fn sk_cg_rect_is_empty(rect: &CGRect) -> bool {
    rect.size.width <= 0.0 || rect.size.height <= 0.0
}

pub fn sk_cg_rect_get_min_x(rect: &CGRect) -> CGFloat {
    rect.origin.x
}

pub fn sk_cg_rect_get_max_x(rect: &CGRect) -> CGFloat {
    rect.origin.x + rect.size.width
}

pub fn sk_cg_rect_get_min_y(rect: &CGRect) -> CGFloat {
    rect.origin.y
}

pub fn sk_cg_rect_get_max_y(rect: &CGRect) -> CGFloat {
    rect.origin.y + rect.size.height
}

pub fn sk_cg_rect_get_width(rect: &CGRect) -> CGFloat {
    rect.size.width
}
