//! Port of `src/core/SkLineClipper.h` and `src/core/SkLineClipper.cpp`.

use crate::floating_point::float_midpoint;
use crate::point::Point;
use crate::rect::Rect;
use crate::scalar::{Scalar, scalar_nearly_zero};

pub const K_MAX_POINTS: usize = 4;
pub const K_MAX_CLIPPED_LINE_SEGMENTS: usize = K_MAX_POINTS - 1;

fn pin_unsorted<T: PartialOrd + Copy>(mut value: T, mut limit0: T, mut limit1: T) -> T {
    if limit1 < limit0 {
        core::mem::swap(&mut limit0, &mut limit1);
    }
    debug_assert!(limit0 <= limit1);

    if value < limit0 {
        value = limit0;
    } else if value > limit1 {
        value = limit1;
    }
    value
}

// return X coordinate of intersection with horizontal line at Y
fn sect_with_horizontal(src: &[Point], y: Scalar) -> Scalar {
    let dy = src[1].y - src[0].y;
    if scalar_nearly_zero(dy) {
        float_midpoint(src[0].x, src[1].x)
    } else {
        // need the extra precision so we don't compute a value that exceeds our original limits
        let x0 = src[0].x as f64;
        let y0 = src[0].y as f64;
        let x1 = src[1].x as f64;
        let y1 = src[1].y as f64;
        let result = x0 + ((y as f64) - y0) * (x1 - x0) / (y1 - y0);

        // The computed X value might still exceed [X0..X1] due to quantum flux
        // when the doubles were added and subtracted, so we have to pin the answer :(
        pin_unsorted(result as Scalar, x0 as Scalar, x1 as Scalar)
    }
}

// return Y coordinate of intersection with vertical line at X
fn sect_with_vertical(src: &[Point], x: Scalar) -> Scalar {
    let dx = src[1].x - src[0].x;
    if scalar_nearly_zero(dx) {
        float_midpoint(src[0].y, src[1].y)
    } else {
        let x0 = src[0].x as f64;
        let y0 = src[0].y as f64;
        let x1 = src[1].x as f64;
        let y1 = src[1].y as f64;
        let result = y0 + ((x as f64) - x0) * (y1 - y0) / (x1 - x0);
        result as Scalar
    }
}

fn sect_clamp_with_vertical(src: &[Point], x: Scalar) -> Scalar {
    let y = sect_with_vertical(src, x);
    // Our caller expects y to be between src[0].fY and src[1].fY (unsorted), but due to the
    // numerics of floats/doubles, we might have computed a value slightly outside of that,
    // so we have to manually clamp afterwards.
    pin_unsorted(y, src[0].y, src[1].y)
}

fn nested_lt(a: Scalar, b: Scalar, dim: Scalar) -> bool {
    a <= b && (a < b || dim > 0.0)
}

// returns true if outer contains inner, even if inner is empty.
// note: outer.contains(inner) always returns false if inner is empty.
fn contains_no_empty_check(outer: &Rect, inner: &Rect) -> bool {
    outer.left <= inner.left
        && outer.top <= inner.top
        && outer.right >= inner.right
        && outer.bottom >= inner.bottom
}

/// Intersect the line segment against the rect. If there is a non-empty resulting segment,
/// return true and set `dst` to that segment. `clip_line` is specialized for scan-conversion.
pub fn intersect_line(src: &[Point; 2], clip: &Rect, dst: &mut [Point; 2]) -> bool {
    let mut bounds = Rect::default();
    bounds.set_points(src[0], src[1]);
    if contains_no_empty_check(clip, &bounds) {
        *dst = *src;
        return true;
    }
    // check for no overlap, and only permit coincident edges if the line and the edge are colinear
    if nested_lt(bounds.right, clip.left, bounds.width())
        || nested_lt(clip.right, bounds.left, bounds.width())
        || nested_lt(bounds.bottom, clip.top, bounds.height())
        || nested_lt(clip.bottom, bounds.top, bounds.height())
    {
        return false;
    }

    let (mut index0, mut index1);

    if src[0].y < src[1].y {
        index0 = 0;
        index1 = 1;
    } else {
        index0 = 1;
        index1 = 0;
    }

    let mut tmp = [src[0], src[1]];

    // now compute Y intersections
    if tmp[index0].y < clip.top {
        tmp[index0] = Point::new(sect_with_horizontal(src, clip.top), clip.top);
    }
    if tmp[index1].y > clip.bottom {
        tmp[index1] = Point::new(sect_with_horizontal(src, clip.bottom), clip.bottom);
    }

    if tmp[0].x < tmp[1].x {
        index0 = 0;
        index1 = 1;
    } else {
        index0 = 1;
        index1 = 0;
    }

    // check for quick-reject in X again, now that we may have been chopped
    if tmp[index1].x <= clip.left || tmp[index0].x >= clip.right {
        // usually we will return false, but we don't if the line is vertical and coincident
        // with the clip.
        if tmp[0].x != tmp[1].x || tmp[0].x < clip.left || tmp[0].x > clip.right {
            return false;
        }
    }

    if tmp[index0].x < clip.left {
        tmp[index0] = Point::new(clip.left, sect_with_vertical(&tmp, clip.left));
    }
    if tmp[index1].x > clip.right {
        tmp[index1] = Point::new(clip.right, sect_with_vertical(&tmp, clip.right));
    }
    debug_assert!({
        let mut b = Rect::default();
        b.set_points(tmp[0], tmp[1]);
        contains_no_empty_check(clip, &b)
    });
    *dst = tmp;
    true
}

/// Clip the line `pts[0]...pts[1]` against `clip`, ignoring segments that lie completely above
/// or below the clip. For portions to the left or right, turn those into vertical line segments
/// that are aligned to the edge of the clip.
///
/// Return the number of line segments that result, and store the end-points of those segments
/// sequentially in `lines` as follows:
/// 1st segment: lines[0]..lines[1]; 2nd: lines[1]..lines[2]; 3rd: lines[2]..lines[3].
pub fn clip_line(
    pts: &[Point; 2],
    clip: &Rect,
    lines: &mut [Point; K_MAX_POINTS],
    can_cull_to_the_right: bool,
) -> i32 {
    let (mut index0, mut index1);

    if pts[0].y < pts[1].y {
        index0 = 0;
        index1 = 1;
    } else {
        index0 = 1;
        index1 = 0;
    }

    // Check if we're completely clipped out in Y (above or below)
    if pts[index1].y <= clip.top {
        // we're above the clip
        return 0;
    }
    if pts[index0].y >= clip.bottom {
        // we're below the clip
        return 0;
    }

    // Chop in Y to produce a single segment, stored in tmp[0..1]
    let mut tmp = [pts[0], pts[1]];

    // now compute intersections
    if pts[index0].y < clip.top {
        tmp[index0] = Point::new(sect_with_horizontal(pts, clip.top), clip.top);
    }
    if tmp[index1].y > clip.bottom {
        tmp[index1] = Point::new(sect_with_horizontal(pts, clip.bottom), clip.bottom);
    }

    // Chop it into 1..3 segments that are wholly within the clip in X.
    let mut result_storage = [Point::default(); K_MAX_POINTS];
    let line_count;
    let mut reverse;

    if pts[0].x < pts[1].x {
        index0 = 0;
        index1 = 1;
        reverse = false;
    } else {
        index0 = 1;
        index1 = 0;
        reverse = true;
    }

    if tmp[index1].x <= clip.left {
        // wholly to the left
        tmp[0].x = clip.left;
        tmp[1].x = clip.left;
        result_storage[0] = tmp[0];
        result_storage[1] = tmp[1];
        line_count = 1;
        reverse = false;
    } else if tmp[index0].x >= clip.right {
        // wholly to the right
        if can_cull_to_the_right {
            return 0;
        }
        tmp[0].x = clip.right;
        tmp[1].x = clip.right;
        result_storage[0] = tmp[0];
        result_storage[1] = tmp[1];
        line_count = 1;
        reverse = false;
    } else {
        let mut r = 0usize;

        if tmp[index0].x < clip.left {
            result_storage[r] = Point::new(clip.left, tmp[index0].y);
            r += 1;
            result_storage[r] = Point::new(clip.left, sect_clamp_with_vertical(&tmp, clip.left));
        } else {
            result_storage[r] = tmp[index0];
        }
        r += 1;

        if tmp[index1].x > clip.right {
            result_storage[r] = Point::new(clip.right, sect_clamp_with_vertical(&tmp, clip.right));
            r += 1;
            result_storage[r] = Point::new(clip.right, tmp[index1].y);
        } else {
            result_storage[r] = tmp[index1];
        }

        line_count = r as i32;
    }

    // Now copy the results into the caller's lines[] parameter
    if reverse {
        // copy the pts in reverse order to maintain winding order
        for i in 0..=line_count as usize {
            lines[line_count as usize - i] = result_storage[i];
        }
    } else {
        lines[..line_count as usize + 1]
            .copy_from_slice(&result_storage[..line_count as usize + 1]);
    }
    line_count
}
