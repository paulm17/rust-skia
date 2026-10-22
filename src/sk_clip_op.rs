//! Port of `include/core/SkClipOp.h`.

/// `enum class SkClipOp`.
///
/// The numeric values intentionally match `SkRegion::Op` for the two clip operations that can be
/// expressed as clip-stack elements, exactly as in the reference (`kDifference = 0`,
/// `kIntersect = 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SkClipOp {
    Difference = 0,
    Intersect = 1,
}

impl SkClipOp {
    pub const K_MAX_ENUM_VALUE: SkClipOp = SkClipOp::Intersect;

    #[inline]
    pub const fn from_i32(value: i32) -> SkClipOp {
        match value {
            0 => SkClipOp::Difference,
            _ => SkClipOp::Intersect,
        }
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sk_clip_op_values() {
        assert_eq!(SkClipOp::Difference.to_i32(), 0);
        assert_eq!(SkClipOp::Intersect.to_i32(), 1);
        assert_eq!(SkClipOp::K_MAX_ENUM_VALUE, SkClipOp::Intersect);
        assert_eq!(SkClipOp::from_i32(0), SkClipOp::Difference);
        assert_eq!(SkClipOp::from_i32(1), SkClipOp::Intersect);
    }
}
