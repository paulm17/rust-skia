//! Port of `include/core/SkAlphaType.h`.

/// `enum SkAlphaType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SkAlphaType {
    #[default]
    Unknown = 0,
    Opaque = 1,
    Premul = 2,
    Unpremul = 3,
}

impl SkAlphaType {
    pub const K_LAST_ENUM: SkAlphaType = SkAlphaType::Unpremul;

    pub const fn from_i32(value: i32) -> SkAlphaType {
        match value {
            1 => SkAlphaType::Opaque,
            2 => SkAlphaType::Premul,
            3 => SkAlphaType::Unpremul,
            _ => SkAlphaType::Unknown,
        }
    }

    pub const fn to_i32(self) -> i32 {
        self as i32
    }
}

#[inline]
pub const fn sk_alpha_type_is_opaque(at: SkAlphaType) -> bool {
    matches!(at, SkAlphaType::Opaque)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sk_alpha_type_values() {
        assert_eq!(SkAlphaType::Unknown.to_i32(), 0);
        assert_eq!(SkAlphaType::Opaque.to_i32(), 1);
        assert_eq!(SkAlphaType::Premul.to_i32(), 2);
        assert_eq!(SkAlphaType::Unpremul.to_i32(), 3);
        assert_eq!(SkAlphaType::K_LAST_ENUM, SkAlphaType::Unpremul);
        assert!(sk_alpha_type_is_opaque(SkAlphaType::Opaque));
    }
}
