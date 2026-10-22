//! Port of `src/core/SkRegionPriv.h`.

use crate::rect::IRect;
use crate::sk_region::{SK_REGION_K_RUN_TYPE_SENTINEL, SkRegion, sk_region_value_is_sentinel};

/// `SkRegionPriv`.
pub struct SkRegionPriv;

impl SkRegionPriv {
    pub const K_RUN_TYPE_SENTINEL: i32 = SK_REGION_K_RUN_TYPE_SENTINEL;

    /// `SkRegionPriv::Validate` (`SK_DEBUG` only in the reference).
    #[inline]
    pub fn validate(rgn: &SkRegion) {
        debug_assert!(rgn.is_valid());
    }

    /// `SkRegionPriv::VisitSpans`: call `visitor` with each span in Y -> X ascending order.
    pub fn visit_spans<F: FnMut(&IRect)>(rgn: &SkRegion, mut visitor: F) {
        if rgn.is_empty() {
            return;
        }
        if rgn.is_rect() {
            visitor(rgn.get_bounds());
            return;
        }

        let runs = rgn.complex_runs().unwrap();
        let mut p = 0usize;
        let mut top = runs[p];
        p += 1;
        let mut bot = runs[p];
        p += 1;
        loop {
            let pair_count = runs[p];
            p += 1;
            if pair_count == 1 {
                visitor(&IRect::new(runs[p], top, runs[p + 1], bot));
                p += 2;
            } else if pair_count > 1 {
                // we have to loop repeated in Y, sending each interval in Y -> X order
                for y in top..bot {
                    for i in 0..pair_count as usize {
                        visitor(&IRect::new(runs[p + i * 2], y, runs[p + i * 2 + 1], y + 1));
                    }
                }
                p += pair_count as usize * 2;
            }
            debug_assert!(sk_region_value_is_sentinel(runs[p]));
            p += 1; // skip sentinel

            // read next bottom or sentinel
            top = bot;
            bot = runs[p];
            p += 1;
            if sk_region_value_is_sentinel(bot) {
                break;
            }
        }
    }
}
