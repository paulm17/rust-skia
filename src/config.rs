//! Early core configuration and contract types for the macOS CPU build.
//!
//! These declarations cover the M02 `SkUserConfig.h`, `SkFeatures.h`,
//! `SkTypes.h`, `SkCPUTypes.h`, `SkMilestone.h`, `SkCoverageMode.h`, and
//! `SkFourByteTag.h` contracts.

pub const SK_MILESTONE: i32 = 158;

pub type U8CPU = u32;
pub type U16CPU = u32;
pub type SkFourByteTag = u32;

pub const fn sk_set_four_byte_tag(a: u8, b: u8, c: u8, d: u8) -> SkFourByteTag {
    ((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | d as u32
}

pub const SK_MASK_8: U8CPU = 0xFF;
pub const SK_MASK_16: U16CPU = 0xFFFF;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkCoverageMode {
    Union = 0,
    Intersect = 1,
    Difference = 2,
    ReverseDifference = 3,
    Xor = 4,
}

impl SkCoverageMode {
    pub const LAST: Self = Self::Xor;

    pub const fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Union,
            1 => Self::Intersect,
            2 => Self::Difference,
            3 => Self::ReverseDifference,
            4 => Self::Xor,
            _ => return None,
        })
    }
}

pub const SK_CPU_LENDIAN: bool = cfg!(target_endian = "little");
pub const SK_CPU_BENDIAN: bool = cfg!(target_endian = "big");
pub const SK_BUILD_FOR_MAC: bool = cfg!(target_os = "macos");
pub const SK_BUILD_FOR_UNIX: bool = cfg!(unix);
pub const SK_CPU_ARM64: bool = cfg!(target_arch = "aarch64");
pub const SK_CPU_X86_64: bool = cfg!(target_arch = "x86_64");

pub const SK_DEBUG: bool = cfg!(debug_assertions);
pub const SK_RELEASE: bool = !SK_DEBUG;

#[cfg(not(target_os = "macos"))]
compile_error!("rust-skia M02 supports macOS only");

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
compile_error!("rust-skia supports 64-bit arm64 and x86_64 hosts");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_core_contracts_and_tags() {
        assert_eq!(SK_MILESTONE, 158);
        assert_eq!(sk_set_four_byte_tag(b'a', b'b', b'c', b'd'), 0x61626364);
        assert_eq!(SkCoverageMode::from_u32(4), Some(SkCoverageMode::Xor));
        assert_eq!(SkCoverageMode::from_u32(5), None);
        assert!(SK_CPU_LENDIAN);
    }
}
