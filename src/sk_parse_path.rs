//! Port of `include/utils/SkParsePath.h` and `src/utils/SkParsePath.cpp`.
//!
//! The incremental scalar scanning mirrors `SkParse::FindScalar`/`FindScalars`
//! (pointer-advancing `strtod` semantics) without needing C strings.

use crate::floating_point::{is_finite, is_nan};
use crate::point::Point;
use crate::scalar::Scalar;
use crate::sk_geometry::SkAutoConicToQuads;
use crate::sk_path::{Iter, SkPath};
use crate::sk_path_builder::{ArcSize, SkPathBuilder};
use crate::sk_path_types::SkPathDirection;
use crate::string::{SkFormatArg, SkString};

#[inline]
fn is_between(c: u8, min: u8, max: u8) -> bool {
    c >= min && c <= max
}

#[inline]
fn is_ws(c: u8) -> bool {
    is_between(c, 1, 32)
}

#[inline]
fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

#[inline]
fn is_sep(c: u8) -> bool {
    is_ws(c) || c == b','
}

/// `SkParse`'s separator set (used by `SkParse::FindScalars`).
#[inline]
fn is_sep_parse(c: u8) -> bool {
    is_ws(c) || c == b',' || c == b';'
}

#[inline]
fn is_lower(c: u8) -> bool {
    is_between(c, b'a', b'z')
}

#[inline]
fn to_upper(c: u8) -> u8 {
    c - b'a' + b'A'
}

#[inline]
fn byte_at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while is_ws(byte_at(b, i)) {
        i += 1;
    }
    i
}

fn skip_sep(b: &[u8], mut i: usize) -> usize {
    while is_sep(byte_at(b, i)) {
        i += 1;
    }
    i
}

fn skip_sep_parse(b: &[u8], mut i: usize) -> usize {
    while is_sep_parse(byte_at(b, i)) {
        i += 1;
    }
    i
}

/// `SkParse::FindScalar`: returns the parsed value and the next offset, or `None`
/// (leaving the offset untouched) when no scalar can be read.
fn find_scalar(b: &[u8], pos: usize) -> Option<(Scalar, usize)> {
    let mut i = skip_ws(b, pos);

    let start = i;
    if byte_at(b, i) == b'+' || byte_at(b, i) == b'-' {
        i += 1;
    }
    let mut digits = 0;
    while is_digit(byte_at(b, i)) {
        i += 1;
        digits += 1;
    }
    if byte_at(b, i) == b'.' {
        i += 1;
        while is_digit(byte_at(b, i)) {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return None;
    }
    // An exponent only counts when at least one digit follows it (strtod semantics).
    if byte_at(b, i) == b'e' || byte_at(b, i) == b'E' {
        let mut j = i + 1;
        if byte_at(b, j) == b'+' || byte_at(b, j) == b'-' {
            j += 1;
        }
        let mut exp_digits = 0;
        while is_digit(byte_at(b, j)) {
            j += 1;
            exp_digits += 1;
        }
        if exp_digits > 0 {
            i = j;
        }
    }

    let text = std::str::from_utf8(&b[start..i]).ok()?;
    let value: f64 = text.parse().ok()?;
    Some((value as Scalar, i))
}

/// `SkParse::FindScalars`.
fn find_scalars(b: &[u8], pos: usize, values: &mut [Scalar]) -> Option<usize> {
    if values.is_empty() {
        return Some(pos);
    }
    let mut i = pos;
    let count = values.len();
    for index in 0..count {
        let (value, next) = find_scalar(b, i)?;
        values[index] = value;
        i = next;
        if index + 1 == count {
            break;
        }
        i = skip_sep_parse(b, i);
    }
    Some(i)
}

/// `find_points` from `SkParsePath.cpp`.
fn find_points(
    b: &[u8],
    pos: usize,
    points: &mut [Point],
    is_relative: bool,
    relative: Point,
) -> Option<usize> {
    let mut values = vec![0.0 as Scalar; points.len() * 2];
    let next = find_scalars(b, pos, &mut values)?;
    for (index, point) in points.iter_mut().enumerate() {
        let mut x = values[index * 2];
        let mut y = values[index * 2 + 1];
        if is_relative {
            x += relative.x;
            y += relative.y;
        }
        point.set(x, y);
    }
    Some(next)
}

/// `find_scalar` from `SkParsePath.cpp`.
fn find_scalar_rel(
    b: &[u8],
    pos: usize,
    is_relative: bool,
    relative: Scalar,
) -> Option<(Scalar, usize)> {
    let (mut value, next) = find_scalar(b, pos)?;
    if is_relative {
        value += relative;
    }
    Some((value, skip_sep(b, next)))
}

/// `find_flag` from `SkParsePath.cpp`.
fn find_flag(b: &[u8], pos: usize) -> Option<(bool, usize)> {
    let c = byte_at(b, pos);
    if c != b'1' && c != b'0' {
        return None;
    }
    Some((c != b'0', skip_sep(b, pos + 1)))
}

/// `SkParsePath`.
pub struct SkParsePath;

/// `SkParsePath::PathEncoding`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathEncoding {
    Absolute,
    Relative,
}

impl Default for PathEncoding {
    fn default() -> Self {
        PathEncoding::Absolute
    }
}

impl SkParsePath {
    /// `SkParsePath::FromSVGString` (optional-returning form).
    pub fn from_svg_string(data: &str) -> Option<SkPath> {
        let b = data.as_bytes();
        // We will write all data to this local path and only return it
        // if the whole parsing succeeds.
        let mut builder = SkPathBuilder::new();
        let mut first = Point::new(0.0, 0.0);
        let mut c = Point::new(0.0, 0.0);
        let mut lastc = Point::new(0.0, 0.0);
        let mut points = [Point::new(0.0, 0.0); 3];
        let mut scratch: Scalar = 0.0;
        let mut op: u8 = 0;
        let mut previous_op: u8 = 0;
        let mut relative = false;
        let mut data_pos: Option<usize> = Some(0);

        loop {
            // Truncated data
            let pos = data_pos?;
            let pos = skip_ws(b, pos);
            if byte_at(b, pos) == 0 {
                break;
            }
            let ch = byte_at(b, pos);
            if is_digit(ch) || ch == b'-' || ch == b'+' || ch == b'.' {
                if op == 0 || op == b'Z' {
                    return None;
                }
            } else if is_sep(ch) {
                data_pos = Some(skip_sep(b, pos));
            } else {
                op = ch;
                relative = false;
                if is_lower(op) {
                    op = to_upper(op);
                    relative = true;
                }
                data_pos = Some(skip_sep(b, pos + 1));
            }

            match op {
                b'M' => {
                    // Move
                    data_pos = find_points(b, data_pos.unwrap(), &mut points[0..1], relative, c);
                    // find_points might have failed, in which case `data_pos` is None and the
                    // next iteration bails out; `points[0]` then holds a stale value.
                    builder.move_to(points[0]);
                    previous_op = 0;
                    op = b'L';
                    c = points[0];
                }
                b'L' => {
                    // Line
                    data_pos = find_points(b, data_pos.unwrap(), &mut points[0..1], relative, c);
                    builder.line_to(points[0]);
                    c = points[0];
                }
                b'H' => {
                    // Horizontal Line
                    match find_scalar_rel(b, data_pos.unwrap(), relative, c.x) {
                        Some((value, next)) => {
                            scratch = value;
                            data_pos = Some(next);
                        }
                        None => data_pos = None,
                    }
                    builder.line_to_xy(scratch, c.y);
                    c.x = scratch;
                }
                b'V' => {
                    // Vertical Line
                    match find_scalar_rel(b, data_pos.unwrap(), relative, c.y) {
                        Some((value, next)) => {
                            scratch = value;
                            data_pos = Some(next);
                        }
                        None => data_pos = None,
                    }
                    builder.line_to_xy(c.x, scratch);
                    c.y = scratch;
                }
                b'C' => {
                    // Cubic Bezier Curve
                    data_pos = find_points(b, data_pos.unwrap(), &mut points[0..3], relative, c);
                    builder.cubic_to(points[0], points[1], points[2]);
                    lastc = points[1];
                    c = points[2];
                }
                b'S' => {
                    // Continued "Smooth" Cubic Bezier Curve
                    data_pos = find_points(b, data_pos.unwrap(), &mut points[1..3], relative, c);
                    points[0] = c;
                    if previous_op == b'C' || previous_op == b'S' {
                        points[0].x -= lastc.x - c.x;
                        points[0].y -= lastc.y - c.y;
                    }
                    builder.cubic_to(points[0], points[1], points[2]);
                    lastc = points[1];
                    c = points[2];
                }
                b'Q' => {
                    // Quadratic Bezier Curve
                    data_pos = find_points(b, data_pos.unwrap(), &mut points[0..2], relative, c);
                    builder.quad_to(points[0], points[1]);
                    lastc = points[0];
                    c = points[1];
                }
                b'T' => {
                    // Continued Quadratic Bezier Curve
                    data_pos = find_points(b, data_pos.unwrap(), &mut points[1..2], relative, c);
                    points[0] = c;
                    if previous_op == b'Q' || previous_op == b'T' {
                        points[0].x -= lastc.x - c.x;
                        points[0].y -= lastc.y - c.y;
                    }
                    builder.quad_to(points[0], points[1]);
                    lastc = points[0];
                    c = points[1];
                }
                b'A' => {
                    // Arc (Elliptical)
                    let mut radii = [Point::new(0.0, 0.0)];
                    let mut end = [Point::new(0.0, 0.0)];
                    let mut angle: Scalar = 0.0;
                    let mut large_arc = false;
                    let mut sweep = false;
                    let arc = (|| -> Option<usize> {
                        let pos =
                            find_points(b, data_pos?, &mut radii, false, Point::new(0.0, 0.0))?;
                        let pos = skip_sep(b, pos);
                        let (value, pos) = find_scalar_rel(b, pos, false, 0.0)?;
                        angle = value;
                        let pos = skip_sep(b, pos);
                        let (value, pos) = find_flag(b, pos)?;
                        large_arc = value;
                        let pos = skip_sep(b, pos);
                        let (value, pos) = find_flag(b, pos)?;
                        sweep = value;
                        let pos = skip_sep(b, pos);
                        find_points(b, pos, &mut end, relative, c)
                    })();
                    data_pos = arc;
                    if arc.is_some() {
                        builder.arc_to_svg(
                            radii[0],
                            angle,
                            if large_arc {
                                ArcSize::Large
                            } else {
                                ArcSize::Small
                            },
                            // SVG "sweep-flag" uses 1 for clockwise, while kCW_Direction == 0.
                            if sweep {
                                SkPathDirection::Cw
                            } else {
                                SkPathDirection::Ccw
                            },
                            end[0],
                        );
                        c = *builder.points().last().unwrap();
                    }
                }
                b'Z' => {
                    // Close Path
                    builder.close();
                    c = first;
                }
                _ => return None,
            }
            if previous_op == 0 {
                first = c;
            }
            previous_op = op;
        }

        Some(builder.detach(None))
    }

    /// Deprecated C++ form: `FromSVGString(str, outPath)`.
    pub fn from_svg_string_into(data: &str, out_path: &mut SkPath) -> bool {
        match Self::from_svg_string(data) {
            Some(path) => {
                *out_path = path;
                true
            }
            None => false,
        }
    }

    /// `SkParsePath::ToSVGString`.
    pub fn to_svg_string(path: &SkPath, encoding: PathEncoding) -> SkString {
        let mut out = SkString::new();
        let mut current_point = Point::new(0.0, 0.0);
        let rel_selector = encoding == PathEncoding::Relative;

        let mut iter = Iter::from_path(path, false);
        while let Some(rec) = iter.next_rec() {
            let pts = rec.f_points;
            match rec.f_verb {
                crate::sk_path_types::SkPathVerb::Conic => {
                    let tol: Scalar = 1.0 / 1024.0; // how close to a quad
                    let mut quadder = SkAutoConicToQuads::new();
                    let quad_pts = quadder
                        .compute_quads_pts(pts, rec.conic_weight(), tol)
                        .to_vec();
                    for i in 0..quadder.count_quads() as usize {
                        append_command(
                            &mut out,
                            b'Q',
                            &quad_pts[i * 2 + 1..],
                            2,
                            rel_selector,
                            &mut current_point,
                        );
                    }
                }
                crate::sk_path_types::SkPathVerb::Move => {
                    append_command(
                        &mut out,
                        b'M',
                        &pts[0..1],
                        1,
                        rel_selector,
                        &mut current_point,
                    );
                }
                crate::sk_path_types::SkPathVerb::Line => {
                    append_command(
                        &mut out,
                        b'L',
                        &pts[1..2],
                        1,
                        rel_selector,
                        &mut current_point,
                    );
                }
                crate::sk_path_types::SkPathVerb::Quad => {
                    append_command(
                        &mut out,
                        b'Q',
                        &pts[1..3],
                        2,
                        rel_selector,
                        &mut current_point,
                    );
                }
                crate::sk_path_types::SkPathVerb::Cubic => {
                    append_command(
                        &mut out,
                        b'C',
                        &pts[1..4],
                        3,
                        rel_selector,
                        &mut current_point,
                    );
                }
                crate::sk_path_types::SkPathVerb::Close => {
                    out.append_str("Z");
                }
                crate::sk_path_types::SkPathVerb::Done => {}
            }
        }

        out
    }
}

/// `SkStrAppendScalar` (the text form written by `SkWStream::writeScalarAsText`).
pub fn append_scalar_as_text(out: &mut SkString, value: Scalar) {
    // Handle infinity and NaN ourselves to ensure consistent cross-platform results.
    if is_nan(value) {
        out.append_str("nan");
        return;
    }
    if !is_finite(value) {
        if value > 0.0 {
            out.append_str("inf");
        } else {
            out.append_str("-inf");
        }
        return;
    }
    // since floats have at most 8 significant digits, we limit our %g to that.
    out.appendf("%.8g", &[SkFormatArg::F64(value as f64)]);
}

fn append_command(
    out: &mut SkString,
    cmd: u8,
    pts: &[Point],
    count: usize,
    rel: bool,
    current_point: &mut Point,
) {
    // Use lower case cmds for relative encoding.
    let cmd = cmd + 32 * rel as u8;
    out.append(&[cmd]);

    for i in 0..count {
        let pt = if rel { pts[i] - *current_point } else { pts[i] };
        if i > 0 {
            out.append_str(" ");
        }
        append_scalar_as_text(out, pt.x);
        out.append_str(" ");
        append_scalar_as_text(out, pt.y);
    }

    debug_assert!(count > 0);
    // For relative encoding, track the current point (otherwise == origin).
    *current_point = if rel {
        pts[count - 1]
    } else {
        Point::new(0.0, 0.0)
    };
}
