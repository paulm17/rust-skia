//! Port of `src/utils/SkParseColor.cpp` (`SkParse::FindNamedColor` and `SkParse::FindColor`).

use crate::sk_color::{SkColor, sk_color_get_a, sk_color_set_argb, sk_color_set_rgb};

/// Named CSS/SVG colors, sorted by name exactly as in the reference's `gColorNames`.
static G_COLOR_NAMES: [(&str, [u8; 3]); 140] = [
    ("aliceblue", [0xf0, 0xf8, 0xff]),
    ("antiquewhite", [0xfa, 0xeb, 0xd7]),
    ("aqua", [0x00, 0xff, 0xff]),
    ("aquamarine", [0x7f, 0xff, 0xd4]),
    ("azure", [0xf0, 0xff, 0xff]),
    ("beige", [0xf5, 0xf5, 0xdc]),
    ("bisque", [0xff, 0xe4, 0xc4]),
    ("black", [0x00, 0x00, 0x00]),
    ("blanchedalmond", [0xff, 0xeb, 0xcd]),
    ("blue", [0x00, 0x00, 0xff]),
    ("blueviolet", [0x8a, 0x2b, 0xe2]),
    ("brown", [0xa5, 0x2a, 0x2a]),
    ("burlywood", [0xde, 0xb8, 0x87]),
    ("cadetblue", [0x5f, 0x9e, 0xa0]),
    ("chartreuse", [0x7f, 0xff, 0x00]),
    ("chocolate", [0xd2, 0x69, 0x1e]),
    ("coral", [0xff, 0x7f, 0x50]),
    ("cornflowerblue", [0x64, 0x95, 0xed]),
    ("cornsilk", [0xff, 0xf8, 0xdc]),
    ("crimson", [0xdc, 0x14, 0x3c]),
    ("cyan", [0x00, 0xff, 0xff]),
    ("darkblue", [0x00, 0x00, 0x8b]),
    ("darkcyan", [0x00, 0x8b, 0x8b]),
    ("darkgoldenrod", [0xb8, 0x86, 0x0b]),
    ("darkgray", [0xa9, 0xa9, 0xa9]),
    ("darkgreen", [0x00, 0x64, 0x00]),
    ("darkkhaki", [0xbd, 0xb7, 0x6b]),
    ("darkmagenta", [0x8b, 0x00, 0x8b]),
    ("darkolivegreen", [0x55, 0x6b, 0x2f]),
    ("darkorange", [0xff, 0x8c, 0x00]),
    ("darkorchid", [0x99, 0x32, 0xcc]),
    ("darkred", [0x8b, 0x00, 0x00]),
    ("darksalmon", [0xe9, 0x96, 0x7a]),
    ("darkseagreen", [0x8f, 0xbc, 0x8f]),
    ("darkslateblue", [0x48, 0x3d, 0x8b]),
    ("darkslategray", [0x2f, 0x4f, 0x4f]),
    ("darkturquoise", [0x00, 0xce, 0xd1]),
    ("darkviolet", [0x94, 0x00, 0xd3]),
    ("deeppink", [0xff, 0x14, 0x93]),
    ("deepskyblue", [0x00, 0xbf, 0xff]),
    ("dimgray", [0x69, 0x69, 0x69]),
    ("dodgerblue", [0x1e, 0x90, 0xff]),
    ("firebrick", [0xb2, 0x22, 0x22]),
    ("floralwhite", [0xff, 0xfa, 0xf0]),
    ("forestgreen", [0x22, 0x8b, 0x22]),
    ("fuchsia", [0xff, 0x00, 0xff]),
    ("gainsboro", [0xdc, 0xdc, 0xdc]),
    ("ghostwhite", [0xf8, 0xf8, 0xff]),
    ("gold", [0xff, 0xd7, 0x00]),
    ("goldenrod", [0xda, 0xa5, 0x20]),
    ("gray", [0x80, 0x80, 0x80]),
    ("green", [0x00, 0x80, 0x00]),
    ("greenyellow", [0xad, 0xff, 0x2f]),
    ("honeydew", [0xf0, 0xff, 0xf0]),
    ("hotpink", [0xff, 0x69, 0xb4]),
    ("indianred", [0xcd, 0x5c, 0x5c]),
    ("indigo", [0x4b, 0x00, 0x82]),
    ("ivory", [0xff, 0xff, 0xf0]),
    ("khaki", [0xf0, 0xe6, 0x8c]),
    ("lavender", [0xe6, 0xe6, 0xfa]),
    ("lavenderblush", [0xff, 0xf0, 0xf5]),
    ("lawngreen", [0x7c, 0xfc, 0x00]),
    ("lemonchiffon", [0xff, 0xfa, 0xcd]),
    ("lightblue", [0xad, 0xd8, 0xe6]),
    ("lightcoral", [0xf0, 0x80, 0x80]),
    ("lightcyan", [0xe0, 0xff, 0xff]),
    ("lightgoldenrodyellow", [0xfa, 0xfa, 0xd2]),
    ("lightgreen", [0x90, 0xee, 0x90]),
    ("lightgrey", [0xd3, 0xd3, 0xd3]),
    ("lightpink", [0xff, 0xb6, 0xc1]),
    ("lightsalmon", [0xff, 0xa0, 0x7a]),
    ("lightseagreen", [0x20, 0xb2, 0xaa]),
    ("lightskyblue", [0x87, 0xce, 0xfa]),
    ("lightslategray", [0x77, 0x88, 0x99]),
    ("lightsteelblue", [0xb0, 0xc4, 0xde]),
    ("lightyellow", [0xff, 0xff, 0xe0]),
    ("lime", [0x00, 0xff, 0x00]),
    ("limegreen", [0x32, 0xcd, 0x32]),
    ("linen", [0xfa, 0xf0, 0xe6]),
    ("magenta", [0xff, 0x00, 0xff]),
    ("maroon", [0x80, 0x00, 0x00]),
    ("mediumaquamarine", [0x66, 0xcd, 0xaa]),
    ("mediumblue", [0x00, 0x00, 0xcd]),
    ("mediumorchid", [0xba, 0x55, 0xd3]),
    ("mediumpurple", [0x93, 0x70, 0xdb]),
    ("mediumseagreen", [0x3c, 0xb3, 0x71]),
    ("mediumslateblue", [0x7b, 0x68, 0xee]),
    ("mediumspringgreen", [0x00, 0xfa, 0x9a]),
    ("mediumturquoise", [0x48, 0xd1, 0xcc]),
    ("mediumvioletred", [0xc7, 0x15, 0x85]),
    ("midnightblue", [0x19, 0x19, 0x70]),
    ("mintcream", [0xf5, 0xff, 0xfa]),
    ("mistyrose", [0xff, 0xe4, 0xe1]),
    ("moccasin", [0xff, 0xe4, 0xb5]),
    ("navajowhite", [0xff, 0xde, 0xad]),
    ("navy", [0x00, 0x00, 0x80]),
    ("oldlace", [0xfd, 0xf5, 0xe6]),
    ("olive", [0x80, 0x80, 0x00]),
    ("olivedrab", [0x6b, 0x8e, 0x23]),
    ("orange", [0xff, 0xa5, 0x00]),
    ("orangered", [0xff, 0x45, 0x00]),
    ("orchid", [0xda, 0x70, 0xd6]),
    ("palegoldenrod", [0xee, 0xe8, 0xaa]),
    ("palegreen", [0x98, 0xfb, 0x98]),
    ("paleturquoise", [0xaf, 0xee, 0xee]),
    ("palevioletred", [0xdb, 0x70, 0x93]),
    ("papayawhip", [0xff, 0xef, 0xd5]),
    ("peachpuff", [0xff, 0xda, 0xb9]),
    ("peru", [0xcd, 0x85, 0x3f]),
    ("pink", [0xff, 0xc0, 0xcb]),
    ("plum", [0xdd, 0xa0, 0xdd]),
    ("powderblue", [0xb0, 0xe0, 0xe6]),
    ("purple", [0x80, 0x00, 0x80]),
    ("red", [0xff, 0x00, 0x00]),
    ("rosybrown", [0xbc, 0x8f, 0x8f]),
    ("royalblue", [0x41, 0x69, 0xe1]),
    ("saddlebrown", [0x8b, 0x45, 0x13]),
    ("salmon", [0xfa, 0x80, 0x72]),
    ("sandybrown", [0xf4, 0xa4, 0x60]),
    ("seagreen", [0x2e, 0x8b, 0x57]),
    ("seashell", [0xff, 0xf5, 0xee]),
    ("sienna", [0xa0, 0x52, 0x2d]),
    ("silver", [0xc0, 0xc0, 0xc0]),
    ("skyblue", [0x87, 0xce, 0xeb]),
    ("slateblue", [0x6a, 0x5a, 0xcd]),
    ("slategray", [0x70, 0x80, 0x90]),
    ("snow", [0xff, 0xfa, 0xfa]),
    ("springgreen", [0x00, 0xff, 0x7f]),
    ("steelblue", [0x46, 0x82, 0xb4]),
    ("tan", [0xd2, 0xb4, 0x8c]),
    ("teal", [0x00, 0x80, 0x80]),
    ("thistle", [0xd8, 0xbf, 0xd8]),
    ("tomato", [0xff, 0x63, 0x47]),
    ("turquoise", [0x40, 0xe0, 0xd0]),
    ("violet", [0xee, 0x82, 0xee]),
    ("wheat", [0xf5, 0xde, 0xb3]),
    ("white", [0xff, 0xff, 0xff]),
    ("whitesmoke", [0xf5, 0xf5, 0xf5]),
    ("yellow", [0xff, 0xff, 0x00]),
    ("yellowgreen", [0x9a, 0xcd, 0x32]),
];

/// `SkParse::FindNamedColor`: the whole `name` must name a color. The reference returns
/// `name + len`; this port reports success through `Option`.
pub fn sk_parse_find_named_color(name: &str) -> Option<SkColor> {
    match G_COLOR_NAMES.binary_search_by(|(candidate, _)| (*candidate).cmp(name)) {
        Ok(index) => {
            let [r, g, b] = G_COLOR_NAMES[index].1;
            Some(sk_color_set_rgb(r, g, b))
        }
        Err(_) => None,
    }
}

#[inline]
fn to_hex(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as u32),
        b'a'..=b'f' => Some((c - b'a' + 10) as u32),
        b'A'..=b'F' => Some((c - b'A' + 10) as u32),
        _ => None,
    }
}

#[inline]
fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\r' | b'\n')
}

/// `SkParse::FindHex`: returns the value and the index just past the parsed digits.
/// At most eight digits are accepted, and the parse must end at the string end or whitespace.
fn find_hex(s: &[u8]) -> Option<(u32, usize)> {
    let mut index = 0;
    while index < s.len() && is_ws(s[index]) {
        index += 1;
    }
    if index >= s.len() || to_hex(s[index]).is_none() {
        return None;
    }

    let mut value: u32 = 0;
    let mut max_digits = 8;
    while index < s.len() {
        let Some(digit) = to_hex(s[index]) else {
            break;
        };
        max_digits -= 1;
        if max_digits < 0 {
            return None;
        }
        value = (value << 4) | digit;
        index += 1;
    }

    if index == s.len() || is_ws(s[index]) {
        Some((value, index))
    } else {
        None
    }
}

#[inline]
fn nib2byte(n: u32) -> u8 {
    debug_assert!(n & !0xF == 0);
    ((n << 4) | n) as u8
}

/// `SkParse::FindColor`: parses `#rgb`, `#argb`, `#rrggbb`, `#aarrggbb`, or a named color.
///
/// `color` supplies the alpha used by short/6-digit forms and receives the result.
/// Returns the number of bytes consumed on success.
pub fn sk_parse_find_color(value: &str, color: &mut SkColor) -> Option<usize> {
    let old_alpha = sk_color_get_a(*color);
    let bytes = value.as_bytes();

    if bytes.first() == Some(&b'#') {
        let (mut hex, end) = find_hex(&bytes[1..])?;
        let len = end;
        if len == 3 || len == 4 {
            let a = if len == 4 {
                nib2byte(hex >> 12)
            } else {
                old_alpha
            };
            let r = nib2byte((hex >> 8) & 0xF);
            let g = nib2byte((hex >> 4) & 0xF);
            let b = nib2byte(hex & 0xF);
            *color = sk_color_set_argb(a, r, g, b);
            return Some(1 + end);
        } else if len == 6 || len == 8 {
            if len == 6 {
                hex |= (old_alpha as u32) << 24;
            }
            *color = hex;
            return Some(1 + end);
        }
        None
    } else {
        let named = sk_parse_find_named_color(value)?;
        *color = named;
        Some(value.len())
    }
}

/// `SkParse::FindNamedColor` with an explicit byte length, mirroring the reference's
/// `(name, len)` overload used by `FindColor`.
pub fn sk_parse_find_named_color_n(name: &[u8]) -> Option<SkColor> {
    let name = std::str::from_utf8(name).ok()?;
    sk_parse_find_named_color(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_named_colors_are_sorted_and_parse() {
        assert!(G_COLOR_NAMES.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(sk_parse_find_named_color("aliceblue"), Some(0xFFF0_F8FF));
        assert_eq!(sk_parse_find_named_color("yellowgreen"), Some(0xFF9A_CD32));
        assert_eq!(sk_parse_find_named_color("lightgrey"), Some(0xFFD3_D3D3));
        assert_eq!(sk_parse_find_named_color("lightgray"), None);
        assert_eq!(sk_parse_find_named_color(""), None);
        assert_eq!(sk_parse_find_named_color("aquaA"), None);
    }

    #[test]
    fn dev_find_color_hex_forms() {
        let mut c = 0xFF00_0000;
        assert_eq!(sk_parse_find_color("#abc", &mut c), Some(4));
        assert_eq!(c, 0xFFAA_BBCC);
        assert_eq!(sk_parse_find_color("#fabc", &mut c), Some(5));
        assert_eq!(c, 0xFFAA_BBCC);
        assert_eq!(sk_parse_find_color("#112233", &mut c), Some(7));
        assert_eq!(c, 0xFF11_2233);
        assert_eq!(sk_parse_find_color("#55112233", &mut c), Some(9));
        assert_eq!(c, 0x5511_2233);
        assert_eq!(sk_parse_find_color("#12", &mut c), None);
        assert_eq!(sk_parse_find_color("red", &mut c), Some(3));
        assert_eq!(c, 0xFFFF_0000);
    }
}
