fn nextafterf_toward_zero(x: f32) -> f32 {
    if x == 0.0 {
        x
    } else if x > 0.0 {
        f32::from_bits(x.to_bits() - 1)
    } else {
        f32::from_bits(x.to_bits() + 1)
    }
}

pub fn adjust_radii(limit: f64, scale: f64, a: &mut f32, b: &mut f32) {
    debug_assert!(scale < 1.0 && scale > 0.0);

    *a = (*a as f64 * scale) as f32;
    *b = (*b as f64 * scale) as f32;

    if ((*a + *b) as f64) > limit {
        let swap = *a > *b;
        let new_min_radius = if swap { *b } else { *a };
        let mut new_max_radius = (limit - new_min_radius as f64) as f32;

        while ((new_max_radius + new_min_radius) as f64) > limit {
            new_max_radius = nextafterf_toward_zero(new_max_radius);
        }
        if swap {
            *b = new_min_radius;
            *a = new_max_radius;
        } else {
            *a = new_min_radius;
            *b = new_max_radius;
        }
    }

    debug_assert!(*a >= 0.0 && *b >= 0.0);
    debug_assert!(((*a + *b) as f64) <= limit);
}

pub struct ScaleToSides;

impl ScaleToSides {
    pub fn adjust_radii(limit: f64, scale: f64, a: &mut f32, b: &mut f32) {
        adjust_radii(limit, scale, a, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_adjust_radii_fits_limit() {
        let mut a = 10.0f32;
        let mut b = 20.0f32;
        adjust_radii(20.0, 0.5, &mut a, &mut b);
        assert!(a >= 0.0 && b >= 0.0);
        assert!((a + b) as f64 <= 20.0);
    }
}
