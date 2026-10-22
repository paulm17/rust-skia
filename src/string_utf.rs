//! UTF-8/UTF-16 helpers matching `src/core/SkUTF.h/.cpp` behavior.

pub fn sk_utf8_to_utf16(value: u32) -> Vec<u16> {
    if value < 0x10000 {
        vec![value as u16]
    } else {
        let v = value - 0x10000;
        vec![0xD800 | ((v >> 10) as u16), 0xDC00 | ((v & 0x3FF) as u16)]
    }
}

pub fn sk_utf8_encode(value: u32, buf: &mut [u8; 4]) -> usize {
    if value > 0x10FFFF {
        return 0;
    }
    if value < 0x80 {
        buf[0] = value as u8;
        1
    } else if value < 0x800 {
        buf[0] = 0xC0 | (value >> 6) as u8;
        buf[1] = 0x80 | (value & 0x3F) as u8;
        2
    } else if value < 0x10000 {
        buf[0] = 0xE0 | (value >> 12) as u8;
        buf[1] = 0x80 | ((value >> 6) & 0x3F) as u8;
        buf[2] = 0x80 | (value & 0x3F) as u8;
        3
    } else {
        buf[0] = 0xF0 | (value >> 18) as u8;
        buf[1] = 0x80 | ((value >> 12) & 0x3F) as u8;
        buf[2] = 0x80 | ((value >> 6) & 0x3F) as u8;
        buf[3] = 0x80 | (value & 0x3F) as u8;
        4
    }
}

/// Returns the codepoint and advances `pos`. On failure returns -1 and sets `pos` to end.
pub fn sk_utf8_next(src: &[u8], pos: &mut usize) -> i32 {
    let start = *pos;
    let end = src.len();
    if start >= end {
        *pos = end;
        return -1;
    }
    let c = src[start];
    let (len, mut value) = if c < 0x80 {
        *pos = start + 1;
        return c as i32;
    } else if (0xC2..=0xDF).contains(&c) {
        (2, (c & 0x1F) as u32)
    } else if (0xE0..=0xEF).contains(&c) {
        (3, (c & 0x0F) as u32)
    } else if (0xF0..=0xF4).contains(&c) {
        (4, (c & 0x07) as u32)
    } else {
        *pos = end;
        return -1;
    };
    if start + len > end {
        *pos = end;
        return -1;
    }
    for i in 1..len {
        let b = src[start + i];
        if b & 0xC0 != 0x80 {
            *pos = end;
            return -1;
        }
        value = (value << 6) | (b & 0x3F) as u32;
    }
    *pos = start + len;
    value as i32
}

pub fn sk_utf8_count(src: &[u8]) -> i32 {
    let mut count = 0;
    let mut pos = 0;
    while pos < src.len() {
        if sk_utf8_next(src, &mut pos) < 0 {
            return -1;
        }
        count += 1;
    }
    count
}

pub fn sk_utf16_next(src: &[u16], pos: &mut usize) -> i32 {
    let start = *pos;
    if start >= src.len() {
        *pos = src.len();
        return -1;
    }
    let cu = src[start] as u32;
    if (0xD800..0xDC00).contains(&cu) {
        if start + 1 < src.len() {
            let cu2 = src[start + 1] as u32;
            if (0xDC00..0xE000).contains(&cu2) {
                *pos = start + 2;
                return (0x10000 + (((cu - 0xD800) << 10) | (cu2 - 0xDC00))) as i32;
            }
        }
        *pos = src.len();
        return -1;
    }
    if (0xDC00..0xE000).contains(&cu) {
        *pos = src.len();
        return -1;
    }
    *pos = start + 1;
    cu as i32
}

pub fn sk_utf16_count(src: &[u16]) -> i32 {
    let mut count = 0;
    let mut pos = 0;
    while pos < src.len() {
        if sk_utf16_next(src, &mut pos) < 0 {
            return -1;
        }
        count += 1;
    }
    count
}

pub fn sk_utf8_to_utf16_slice(src: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < src.len() {
        let cp = sk_utf8_next(src, &mut pos);
        if cp < 0 {
            break;
        }
        out.extend(sk_utf8_to_utf16(cp as u32));
    }
    out
}

pub fn sk_utf16_to_utf8(src: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < src.len() {
        let cp = sk_utf16_next(src, &mut pos);
        if cp < 0 {
            break;
        }
        let mut buf = [0u8; 4];
        let n = sk_utf8_encode(cp as u32, &mut buf);
        out.extend_from_slice(&buf[..n]);
    }
    out
}

pub struct SkUTF;

impl SkUTF {
    pub fn next_utf8(src: &[u8], pos: &mut usize) -> i32 {
        sk_utf8_next(src, pos)
    }
    pub fn next_utf16(src: &[u16], pos: &mut usize) -> i32 {
        sk_utf16_next(src, pos)
    }
    pub fn to_utf8(value: u32) -> usize {
        let mut buf = [0u8; 4];
        sk_utf8_encode(value, &mut buf)
    }
    pub fn to_utf16(value: u32) -> Vec<u16> {
        sk_utf8_to_utf16(value)
    }
    pub fn count_utf8(src: &[u8]) -> i32 {
        sk_utf8_count(src)
    }
    pub fn count_utf16(src: &[u16]) -> i32 {
        sk_utf16_count(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_utf8_and_utf16() {
        let mut pos = 0;
        assert_eq!(sk_utf8_next(b"A", &mut pos), 65);
        let mut pos = 0;
        assert_eq!(sk_utf8_next(&[0xE2, 0x82, 0xAC], &mut pos), 0x20AC);
        let mut pos = 0;
        assert_eq!(sk_utf8_next(&[0xC0, 0x80], &mut pos), -1);
        assert_eq!(pos, 2);
        assert_eq!(sk_utf8_to_utf16(0x1F600), vec![0xD83D, 0xDE00]);
        let mut pos = 0;
        assert_eq!(sk_utf16_next(&[0xD83D, 0xDE00], &mut pos), 0x1F600);
        assert_eq!(sk_utf16_count(&[0xD83D, 0xDE00]), 1);
    }
}
