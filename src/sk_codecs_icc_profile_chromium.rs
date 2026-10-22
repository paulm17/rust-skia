//! Port of `include/private/chromium/SkCodecsICCProfileChromium.h`.
//!
//! Allows Chromium code that does not use `SkCodec` to parse ICC profiles using the same parser
//! that `SkCodec` does (here: the bundled `skcms` port).
//!
//! Ownership adaptation: the reference builds an `std::unique_ptr<ICCProfileChromium>` over a
//! retained `sk_sp<SkData>`. This port owns the profile (`SkcmsIccProfile` copies its ICC buffer)
//! and drops the `SkData` after parsing, so all `GetProfile` pointers stay valid for the lifetime
//! of the object.

use crate::data::SkData;
use crate::skcms::{SkcmsIccProfile, skcms_parse};

/// `SkCodecs::ICCProfileChromium`.
#[derive(Clone, Debug)]
pub struct IccProfileChromium {
    profile: SkcmsIccProfile,
}

impl IccProfileChromium {
    /// `ICCProfileChromium::Make`.
    ///
    /// Returns `None` if parsing fails. May retain `data`; this port copies what it needs instead.
    pub fn make(data: &SkData) -> Option<Self> {
        let mut profile = SkcmsIccProfile::default();
        if !skcms_parse(data.bytes(), &mut profile) {
            return None;
        }
        Some(Self { profile })
    }

    /// Convenience equivalent for callers that hold raw ICC bytes.
    pub fn make_from_bytes(bytes: &[u8]) -> Option<Self> {
        let data = SkData::make_with_copy(bytes);
        Self::make(&data)
    }

    /// `GetProfile`: the parsed profile.
    pub fn get_profile(&self) -> &SkcmsIccProfile {
        &self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_make_rejects_garbage() {
        let data = SkData::make_with_copy(b"not an icc profile");
        assert!(IccProfileChromium::make(&data).is_none());
    }
}
