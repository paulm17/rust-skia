use crate::floating_point::float_midpoint;
use crate::math_priv::sk_fits_in_fixed;
use crate::point::Point;
use crate::rect::{IRect, Rect};

pub struct RectPriv;

impl RectPriv {
    pub const fn make_i_large() -> IRect {
        let large = 1 << 29;
        IRect::new(-large, -large, large, large)
    }

    pub const fn make_i_largest_inverted() -> IRect {
        IRect::new(i32::MAX, i32::MAX, -i32::MAX, -i32::MAX)
    }

    pub fn make_large_s32() -> Rect {
        Rect::make_irect(Self::make_i_large())
    }

    pub const fn make_largest() -> Rect {
        Rect::new(-f32::MAX, -f32::MAX, f32::MAX, f32::MAX)
    }

    pub const fn make_largest_inverted() -> Rect {
        Rect::new(f32::MAX, f32::MAX, -f32::MAX, -f32::MAX)
    }

    pub fn grow_to_include(r: &mut Rect, pt: &Point) {
        r.left = pt.x.min(r.left);
        r.right = pt.x.max(r.right);
        r.top = pt.y.min(r.top);
        r.bottom = pt.y.max(r.bottom);
    }

    pub fn fits_in_fixed(r: &Rect) -> bool {
        sk_fits_in_fixed(r.left)
            && sk_fits_in_fixed(r.top)
            && sk_fits_in_fixed(r.right)
            && sk_fits_in_fixed(r.bottom)
    }

    pub fn half_width(r: &Rect) -> f32 {
        float_midpoint(-r.left, r.right)
    }

    pub fn half_height(r: &Rect) -> f32 {
        float_midpoint(-r.top, r.bottom)
    }

    pub fn subtract(a: &Rect, b: &Rect, out: &mut Rect) -> bool {
        if a.is_empty() || b.is_empty() || !Rect::intersects_rects(a, b) {
            *out = *a;
            return true;
        }

        let a_height = a.height();
        let a_width = a.width();
        let mut left_area = 0.0f32;
        let mut right_area = 0.0f32;
        let mut top_area = 0.0f32;
        let mut bottom_area = 0.0f32;
        let mut positive_count = 0;

        if b.left > a.left {
            left_area = (b.left - a.left) / a_width;
            positive_count += 1;
        }
        if a.right > b.right {
            right_area = (a.right - b.right) / a_width;
            positive_count += 1;
        }
        if b.top > a.top {
            top_area = (b.top - a.top) / a_height;
            positive_count += 1;
        }
        if a.bottom > b.bottom {
            bottom_area = (a.bottom - b.bottom) / a_height;
            positive_count += 1;
        }

        if positive_count == 0 {
            *out = Rect::make_empty();
            return true;
        }

        *out = *a;
        if left_area > right_area && left_area > top_area && left_area > bottom_area {
            out.right = b.left;
        } else if right_area > top_area && right_area > bottom_area {
            out.left = b.right;
        } else if top_area > bottom_area {
            out.bottom = b.top;
        } else {
            out.top = b.bottom;
        }

        positive_count == 1
    }

    pub fn subtract_value(a: &Rect, b: &Rect) -> Rect {
        let mut diff = Rect::make_empty();
        Self::subtract(a, b, &mut diff);
        diff
    }

    pub fn subtract_irect(a: &IRect, b: &IRect, out: &mut IRect) -> bool {
        if a.is_empty() || b.is_empty() || !IRect::intersects_irects(a, b) {
            *out = *a;
            return true;
        }

        let a_height = a.height() as f32;
        let a_width = a.width() as f32;
        let mut left_area = 0.0f32;
        let mut right_area = 0.0f32;
        let mut top_area = 0.0f32;
        let mut bottom_area = 0.0f32;
        let mut positive_count = 0;

        if b.left > a.left {
            left_area = b.left.wrapping_sub(a.left) as f32 / a_width;
            positive_count += 1;
        }
        if a.right > b.right {
            right_area = a.right.wrapping_sub(b.right) as f32 / a_width;
            positive_count += 1;
        }
        if b.top > a.top {
            top_area = b.top.wrapping_sub(a.top) as f32 / a_height;
            positive_count += 1;
        }
        if a.bottom > b.bottom {
            bottom_area = a.bottom.wrapping_sub(b.bottom) as f32 / a_height;
            positive_count += 1;
        }

        if positive_count == 0 {
            *out = IRect::make_empty();
            return true;
        }

        *out = *a;
        if left_area > right_area && left_area > top_area && left_area > bottom_area {
            out.right = b.left;
        } else if right_area > top_area && right_area > bottom_area {
            out.left = b.right;
        } else if top_area > bottom_area {
            out.bottom = b.top;
        } else {
            out.top = b.bottom;
        }

        positive_count == 1
    }

    pub fn subtract_irect_value(a: &IRect, b: &IRect) -> IRect {
        let mut diff = IRect::make_empty();
        Self::subtract_irect(a, b, &mut diff);
        diff
    }

    pub fn closest_disjoint_edge(src: &IRect, dst: &IRect) -> IRect {
        if src.is_empty() || dst.is_empty() {
            return IRect::make_empty();
        }

        let mut l = src.left;
        let mut r = src.right;
        if r <= dst.left {
            l = r.wrapping_sub(1);
        } else if l >= dst.right {
            r = l.wrapping_add(1);
        } else {
            l = l.clamp(dst.left, dst.right);
            r = r.clamp(dst.left, dst.right);
        }

        let mut t = src.top;
        let mut b = src.bottom;
        if b <= dst.top {
            t = b.wrapping_sub(1);
        } else if t >= dst.bottom {
            b = t.wrapping_add(1);
        } else {
            t = t.clamp(dst.top, dst.bottom);
            b = b.clamp(dst.top, dst.bottom);
        }

        IRect::make_ltrb(l, t, r, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_rect_priv_subtract_and_closest_edge() {
        let a = IRect::new(2, 3, 12, 15);
        let mut out = IRect::make_empty();
        assert!(RectPriv::subtract_irect(
            &a,
            &IRect::new(0, 0, 6, 20),
            &mut out
        ));
        assert_eq!(out, IRect::new(6, 3, 12, 15));

        let src = IRect::new(0, 0, 10, 10);
        assert_eq!(
            RectPriv::closest_disjoint_edge(&src, &IRect::new(-15, -5, -2, 15)),
            IRect::new(0, 0, 1, 10)
        );

        assert!(!RectPriv::make_i_large().is_empty());
        assert!(RectPriv::make_i_largest_inverted().is_empty());
        assert!(!RectPriv::make_largest().is_empty());
        assert!(!RectPriv::make_large_s32().is_empty());
        assert!(RectPriv::make_largest_inverted().is_empty());
    }

    #[test]
    fn dev_rect_subtract_reference_cases() {
        let a = IRect::new(2, 3, 12, 15);
        let cases = [
            (a, IRect::new(0, 1, 14, 17), IRect::make_empty(), true),
            (
                a,
                IRect::new(6, 6, 10, 10),
                IRect::new(2, 10, 12, 15),
                false,
            ),
            (IRect::make_empty(), a, IRect::make_empty(), true),
            (a, IRect::make_empty(), a, true),
            (
                IRect::make_empty(),
                IRect::make_empty(),
                IRect::make_empty(),
                true,
            ),
            (a, IRect::new(15, 17, 20, 40), a, true),
            (a, IRect::new(0, 0, 6, 20), IRect::new(6, 3, 12, 15), true),
            (a, IRect::new(6, 0, 20, 20), IRect::new(2, 3, 6, 15), true),
            (a, IRect::new(0, 0, 20, 6), IRect::new(2, 6, 12, 15), true),
            (a, IRect::new(0, 10, 20, 20), IRect::new(2, 3, 12, 10), true),
            (
                a,
                IRect::new(0, 6, 20, 10),
                IRect::new(2, 10, 12, 15),
                false,
            ),
            (a, IRect::new(6, 0, 10, 20), IRect::new(2, 3, 6, 15), false),
            (a, IRect::new(0, 0, 4, 4), IRect::new(2, 4, 12, 15), false),
            (a, IRect::new(10, 0, 14, 7), IRect::new(2, 3, 10, 15), false),
            (a, IRect::new(0, 12, 5, 20), IRect::new(2, 3, 12, 12), false),
            (
                a,
                IRect::new(10, 12, 20, 20),
                IRect::new(2, 3, 10, 15),
                false,
            ),
            (a, IRect::new(0, 6, 8, 10), IRect::new(2, 10, 12, 15), false),
            (a, IRect::new(8, 6, 20, 10), IRect::new(2, 3, 8, 15), false),
            (a, IRect::new(6, 0, 10, 7), IRect::new(2, 7, 12, 15), false),
            (a, IRect::new(4, 6, 8, 20), IRect::new(8, 3, 12, 15), false),
        ];
        for (a, b, expected, expected_exact) in cases {
            let mut out = IRect::make_empty();
            assert_eq!(RectPriv::subtract_irect(&a, &b, &mut out), expected_exact);
            assert_eq!(out, expected);

            let (af, bf, ef) = (
                Rect::make_irect(a).make_offset(0.5, 0.5),
                Rect::make_irect(b).make_offset(0.5, 0.5),
                Rect::make_irect(expected).make_offset(0.5, 0.5),
            );
            let mut df = Rect::make_empty();
            assert_eq!(RectPriv::subtract(&af, &bf, &mut df), expected_exact);
            assert!((df.is_empty() && ef.is_empty()) || df == ef);
        }

        let really_big = IRect::new(-i32::MAX + 1000, 0, i32::MAX - 1000, 100);
        let reasonable = IRect::new(-50, -5, 50, 125);
        let mut difference = IRect::make_empty();
        assert!(RectPriv::subtract_irect(
            &really_big,
            &reasonable,
            &mut difference
        ));
        assert_eq!(difference, really_big);
        assert!(RectPriv::subtract_irect(
            &reasonable,
            &really_big,
            &mut difference
        ));
        assert_eq!(difference, reasonable);
    }

    #[test]
    fn dev_closest_disjoint_edge_reference_cases() {
        let src = IRect::new(0, 0, 10, 10);
        let cases = [
            (IRect::new(-15, -5, -2, 15), IRect::new(0, 0, 1, 10)),
            (IRect::new(-15, 2, -2, 8), IRect::new(0, 2, 1, 8)),
            (IRect::new(-15, -15, -2, -2), IRect::new(0, 0, 1, 1)),
            (IRect::new(-5, -10, 15, -2), IRect::new(0, 0, 10, 1)),
            (IRect::new(2, -10, 8, -2), IRect::new(2, 0, 8, 1)),
            (IRect::new(15, -15, 20, -2), IRect::new(9, 0, 10, 1)),
            (IRect::new(15, -5, 20, 15), IRect::new(9, 0, 10, 10)),
            (IRect::new(15, 2, 20, 8), IRect::new(9, 2, 10, 8)),
            (IRect::new(15, 15, 20, 20), IRect::new(9, 9, 10, 10)),
            (IRect::new(-5, 15, 15, 20), IRect::new(0, 9, 10, 10)),
            (IRect::new(2, 15, 8, 20), IRect::new(2, 9, 8, 10)),
            (IRect::new(-15, 15, -2, 20), IRect::new(0, 9, 1, 10)),
            (IRect::new(2, 2, 15, 15), IRect::new(2, 2, 10, 10)),
            (IRect::new(-5, -5, 8, 8), IRect::new(0, 0, 8, 8)),
            (IRect::new(2, 2, 8, 8), IRect::new(2, 2, 8, 8)),
            (IRect::new(-5, -5, 15, 15), IRect::new(0, 0, 10, 10)),
        ];
        for (dst, expected) in cases {
            assert_eq!(RectPriv::closest_disjoint_edge(&src, &dst), expected);
        }
        assert!(
            RectPriv::closest_disjoint_edge(&IRect::make_empty(), &IRect::new(0, 0, 8, 8))
                .is_empty()
        );
        assert!(
            RectPriv::closest_disjoint_edge(&IRect::new(0, 0, 8, 8), &IRect::make_empty())
                .is_empty()
        );
        assert!(
            RectPriv::closest_disjoint_edge(&IRect::new(10, 10, -1, 2), &IRect::new(15, 8, -2, 20))
                .is_empty()
        );
    }
}
